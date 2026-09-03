//! Gate C source-governance reference binding.
//!
//! This module owns one bounded synthetic source-intake store. It records the
//! source body only in the component-owned database and returns metadata-only
//! outcomes and lifecycle events to callers.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use ppl_contracts::{
    A001_VERSION, SourceAcquisitionMode, SourceIntakeCommand, SourceIntakeOutcome,
    SourceIntakeStatus, SourceVersionSummary,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const MAX_SOURCE_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub struct SourceStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceLifecycleEvent {
    pub event_id: String,
    pub event_type: String,
    pub command_id: String,
    pub source_version_id: String,
    pub correlation_id: String,
    pub causation_id: String,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SourceStoreError {
    #[error("source-store-unavailable")]
    Unavailable,
    #[error("source-store-schema-unsupported")]
    SchemaUnsupported,
    #[error("source-operation-not-found")]
    NotFound,
    #[error("source-outcome-invalid")]
    OutcomeInvalid,
}

impl SourceStore {
    /// Opens or creates the component-owned Gate C store.
    ///
    /// # Errors
    /// Fails closed when the path, schema or database settings are unavailable.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SourceStoreError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| SourceStoreError::Unavailable)?;
        }
        let store = Self { path };
        let connection = store.connect()?;
        initialise(&connection)?;
        Ok(store)
    }

    /// Applies one source-intake command transactionally.
    ///
    /// Exact redelivery returns the recorded semantic outcome. Changed content
    /// under the same idempotency key is refused without creating a new source.
    ///
    /// # Errors
    /// Fails closed when durable state cannot be read or written.
    pub fn apply(
        &self,
        command: &SourceIntakeCommand,
        expected_environment: &str,
        recorded_at: &str,
    ) -> Result<SourceIntakeOutcome, SourceStoreError> {
        let fingerprint = semantic_fingerprint(command)?;
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(|_| SourceStoreError::Unavailable)?;

        if let Some((previous_fingerprint, previous_json)) = transaction
            .query_row(
                "SELECT fingerprint, outcome_json FROM source_operations WHERE idempotency_key = ?1",
                [&command.idempotency_key],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| SourceStoreError::Unavailable)?
        {
            if previous_fingerprint == fingerprint {
                return serde_json::from_str(&previous_json)
                    .map_err(|_| SourceStoreError::OutcomeInvalid);
            }
            return Ok(refused(command, recorded_at, "idempotency-content-conflict"));
        }

        let outcome = if let Some(code) = validate(command, expected_environment) {
            refused(command, recorded_at, code)
        } else {
            quarantine(&transaction, command, recorded_at)?
        };
        record_operation(&transaction, command, &fingerprint, &outcome, recorded_at)?;
        transaction
            .commit()
            .map_err(|_| SourceStoreError::Unavailable)?;
        Ok(outcome)
    }

    /// Returns a previously recorded source outcome by command identifier.
    ///
    /// # Errors
    /// Returns a not-found error without inventing state when the operation is absent.
    pub fn outcome(
        &self,
        command_id: &str,
        expected_environment: &str,
    ) -> Result<SourceIntakeOutcome, SourceStoreError> {
        let connection = self.connect()?;
        let outcome_json = connection
            .query_row(
                "SELECT outcome_json FROM source_operations WHERE command_id = ?1 AND environment_id = ?2",
                params![command_id, expected_environment],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| SourceStoreError::Unavailable)?
            .ok_or(SourceStoreError::NotFound)?;
        serde_json::from_str(&outcome_json).map_err(|_| SourceStoreError::OutcomeInvalid)
    }

    /// Returns metadata-only lifecycle events that still require publication.
    ///
    /// # Errors
    /// Fails closed if the outbox cannot be read.
    pub fn pending_events(&self) -> Result<Vec<SourceLifecycleEvent>, SourceStoreError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT event_json FROM source_outbox WHERE published_at IS NULL ORDER BY rowid",
            )
            .map_err(|_| SourceStoreError::Unavailable)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| SourceStoreError::Unavailable)?;
        rows.map(|row| {
            let json = row.map_err(|_| SourceStoreError::Unavailable)?;
            serde_json::from_str(&json).map_err(|_| SourceStoreError::OutcomeInvalid)
        })
        .collect()
    }

    /// Marks one lifecycle event as published after broker confirmation.
    ///
    /// # Errors
    /// Fails closed if the outbox update cannot be committed.
    pub fn mark_published(
        &self,
        event_id: &str,
        published_at: &str,
    ) -> Result<(), SourceStoreError> {
        let connection = self.connect()?;
        let changed = connection
            .execute(
                "UPDATE source_outbox SET published_at = ?2 WHERE event_id = ?1 AND published_at IS NULL",
                params![event_id, published_at],
            )
            .map_err(|_| SourceStoreError::Unavailable)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(SourceStoreError::NotFound)
        }
    }

    fn connect(&self) -> Result<Connection, SourceStoreError> {
        let connection = Connection::open(&self.path).map_err(|_| SourceStoreError::Unavailable)?;
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(|_| SourceStoreError::Unavailable)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|_| SourceStoreError::Unavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| SourceStoreError::Unavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| SourceStoreError::Unavailable)?;
        Ok(connection)
    }
}

fn initialise(connection: &Connection) -> Result<(), SourceStoreError> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| SourceStoreError::Unavailable)?;
    if version > 1 {
        return Err(SourceStoreError::SchemaUnsupported);
    }
    connection
        .execute_batch(
            "BEGIN IMMEDIATE;
             CREATE TABLE IF NOT EXISTS source_versions (
               source_version_id TEXT PRIMARY KEY,
               source_id TEXT NOT NULL,
               version INTEGER NOT NULL CHECK (version = 1),
               environment_id TEXT NOT NULL,
               demonstration_session_id TEXT NOT NULL,
               engagement_id TEXT NOT NULL,
               actor_id TEXT NOT NULL,
               acquisition_mode TEXT NOT NULL,
               original_name TEXT,
               media_type TEXT NOT NULL,
               size_bytes INTEGER NOT NULL,
               digest_value TEXT NOT NULL,
               title TEXT NOT NULL,
               owner TEXT NOT NULL,
               rights TEXT NOT NULL,
               provenance TEXT NOT NULL,
               classification TEXT NOT NULL CHECK (classification = 'synthetic'),
               status TEXT NOT NULL CHECK (status = 'quarantined'),
               content_text TEXT NOT NULL,
               recorded_at TEXT NOT NULL,
               UNIQUE (source_id, version)
             );
             CREATE TABLE IF NOT EXISTS source_operations (
               idempotency_key TEXT PRIMARY KEY,
               command_id TEXT NOT NULL UNIQUE,
               fingerprint TEXT NOT NULL,
               environment_id TEXT NOT NULL,
               outcome_json TEXT NOT NULL,
               recorded_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS source_outbox (
               event_id TEXT PRIMARY KEY,
               command_id TEXT NOT NULL,
               event_json TEXT NOT NULL,
               published_at TEXT,
               FOREIGN KEY (command_id) REFERENCES source_operations(command_id)
             );
             PRAGMA user_version = 1;
             COMMIT;",
        )
        .map_err(|_| SourceStoreError::Unavailable)
}

fn validate(command: &SourceIntakeCommand, expected_environment: &str) -> Option<&'static str> {
    if command.contract_id != "A-001" || command.contract_version != A001_VERSION {
        return Some("contract-version-unsupported");
    }
    if command.message_type != "source-intake.command" || command.action != "submit-to-quarantine" {
        return Some("source-action-unsupported");
    }
    if command.environment_id != expected_environment {
        return Some("environment-mismatch");
    }
    if command.demonstration_session_id.len() < 8
        || command.engagement_id != "engagement:harbour-support-review"
        || command.actor_id != "synthetic-reviewer"
        || command.actor_role != "workbench-reviewer"
        || command.authority_reference.len() < 8
        || command.purpose != "governed-source-intake"
        || command.correlation_id != command.demonstration_session_id
        || command.causation_id.len() < 8
        || command.idempotency_key.len() < 8
    {
        return Some("authority-or-purpose-refused");
    }
    if command.source.classification != "synthetic" {
        return Some("information-classification-refused");
    }
    if !matches!(
        command.source.media_type.as_str(),
        "text/plain" | "text/markdown"
    ) {
        return Some("media-type-unsupported");
    }
    if command.source.content.trim().is_empty() {
        return Some("source-empty");
    }
    if command.source.content.contains('\0') {
        return Some("source-malformed");
    }
    let actual_size = command.source.content.len();
    if actual_size > MAX_SOURCE_BYTES || command.source.size_bytes != actual_size as u64 {
        return Some("source-size-refused");
    }
    if command.source.title.trim().is_empty()
        || command.source.owner.trim().is_empty()
        || command.source.rights.trim().is_empty()
        || command.source.provenance.trim().is_empty()
    {
        return Some("source-metadata-required");
    }
    if matches!(
        command.source.acquisition_mode,
        SourceAcquisitionMode::Upload
    ) && command
        .source
        .original_name
        .as_deref()
        .is_none_or(str::is_empty)
    {
        return Some("source-filename-required");
    }
    None
}

fn quarantine(
    transaction: &Transaction<'_>,
    command: &SourceIntakeCommand,
    recorded_at: &str,
) -> Result<SourceIntakeOutcome, SourceStoreError> {
    let source_id = format!("source:{}", Uuid::new_v4());
    let source_version_id = format!("source-version:{}", Uuid::new_v4());
    let digest_value = format!("{:x}", Sha256::digest(command.source.content.as_bytes()));
    transaction
        .execute(
            "INSERT INTO source_versions (
               source_version_id, source_id, version, environment_id,
               demonstration_session_id, engagement_id, actor_id,
               acquisition_mode, original_name, media_type, size_bytes,
               digest_value, title, owner, rights, provenance, classification,
               status, content_text, recorded_at
             ) VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                       ?12, ?13, ?14, ?15, ?16, 'quarantined', ?17, ?18)",
            params![
                source_version_id,
                source_id,
                command.environment_id,
                command.demonstration_session_id,
                command.engagement_id,
                command.actor_id,
                acquisition_mode_name(command.source.acquisition_mode),
                command.source.original_name,
                command.source.media_type,
                i64::try_from(command.source.size_bytes)
                    .map_err(|_| SourceStoreError::OutcomeInvalid)?,
                digest_value,
                command.source.title,
                command.source.owner,
                command.source.rights,
                command.source.provenance,
                command.source.classification,
                command.source.content,
                recorded_at,
            ],
        )
        .map_err(|_| SourceStoreError::Unavailable)?;

    Ok(SourceIntakeOutcome {
        contract_id: "A-001".to_owned(),
        contract_version: A001_VERSION.to_owned(),
        message_type: "source-intake.outcome".to_owned(),
        outcome_id: format!("source-outcome:{}", Uuid::new_v4()),
        command_id: command.command_id.clone(),
        status: SourceIntakeStatus::Quarantined,
        code: "source-quarantined".to_owned(),
        environment_id: command.environment_id.clone(),
        demonstration_session_id: command.demonstration_session_id.clone(),
        engagement_id: command.engagement_id.clone(),
        actor_id: command.actor_id.clone(),
        correlation_id: command.correlation_id.clone(),
        recorded_at: recorded_at.to_owned(),
        source_version: Some(SourceVersionSummary {
            source_id,
            source_version_id,
            version: 1,
            status: "quarantined".to_owned(),
            digest_algorithm: "sha-256".to_owned(),
            digest_value,
            acquisition_mode: command.source.acquisition_mode,
            original_name: command.source.original_name.clone(),
            media_type: command.source.media_type.clone(),
            size_bytes: command.source.size_bytes,
            title: command.source.title.clone(),
            owner: command.source.owner.clone(),
            rights: command.source.rights.clone(),
            provenance: command.source.provenance.clone(),
            classification: command.source.classification.clone(),
        }),
        event_types: vec![
            "source.received".to_owned(),
            "source.quarantined".to_owned(),
        ],
    })
}

fn refused(command: &SourceIntakeCommand, recorded_at: &str, code: &str) -> SourceIntakeOutcome {
    SourceIntakeOutcome {
        contract_id: "A-001".to_owned(),
        contract_version: A001_VERSION.to_owned(),
        message_type: "source-intake.outcome".to_owned(),
        outcome_id: format!("source-outcome:{}", Uuid::new_v4()),
        command_id: command.command_id.clone(),
        status: SourceIntakeStatus::Refused,
        code: code.to_owned(),
        environment_id: command.environment_id.clone(),
        demonstration_session_id: command.demonstration_session_id.clone(),
        engagement_id: command.engagement_id.clone(),
        actor_id: command.actor_id.clone(),
        correlation_id: command.correlation_id.clone(),
        recorded_at: recorded_at.to_owned(),
        source_version: None,
        event_types: vec!["source.intake-refused".to_owned()],
    }
}

fn record_operation(
    transaction: &Transaction<'_>,
    command: &SourceIntakeCommand,
    fingerprint: &str,
    outcome: &SourceIntakeOutcome,
    recorded_at: &str,
) -> Result<(), SourceStoreError> {
    let outcome_json =
        serde_json::to_string(outcome).map_err(|_| SourceStoreError::OutcomeInvalid)?;
    transaction
        .execute(
            "INSERT INTO source_operations (
               idempotency_key, command_id, fingerprint, environment_id,
               outcome_json, recorded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                command.idempotency_key,
                command.command_id,
                fingerprint,
                command.environment_id,
                outcome_json,
                recorded_at,
            ],
        )
        .map_err(|_| SourceStoreError::Unavailable)?;
    let subject_reference = outcome.source_version.as_ref().map_or_else(
        || command.command_id.clone(),
        |source| source.source_version_id.clone(),
    );
    for event_type in &outcome.event_types {
        let event = SourceLifecycleEvent {
            event_id: format!("event:{}", Uuid::new_v4()),
            event_type: event_type.clone(),
            command_id: command.command_id.clone(),
            source_version_id: subject_reference.clone(),
            correlation_id: command.correlation_id.clone(),
            causation_id: command.causation_id.clone(),
            occurred_at: recorded_at.to_owned(),
            reason_code: matches!(outcome.status, SourceIntakeStatus::Refused)
                .then(|| outcome.code.clone()),
        };
        let event_json =
            serde_json::to_string(&event).map_err(|_| SourceStoreError::OutcomeInvalid)?;
        transaction
            .execute(
                "INSERT INTO source_outbox (event_id, command_id, event_json) VALUES (?1, ?2, ?3)",
                params![event.event_id, command.command_id, event_json],
            )
            .map_err(|_| SourceStoreError::Unavailable)?;
    }
    Ok(())
}

const fn acquisition_mode_name(mode: SourceAcquisitionMode) -> &'static str {
    match mode {
        SourceAcquisitionMode::Upload => "upload",
        SourceAcquisitionMode::Paste => "paste",
    }
}

fn semantic_fingerprint(command: &SourceIntakeCommand) -> Result<String, SourceStoreError> {
    // Delivery identifiers and timestamps may legitimately change when a
    // caller retries after losing a response. The idempotency decision binds
    // the authorised operation and source semantics instead.
    let value = serde_json::json!({
        "action": command.action,
        "environmentId": command.environment_id,
        "demonstrationSessionId": command.demonstration_session_id,
        "engagementId": command.engagement_id,
        "actorId": command.actor_id,
        "actorRole": command.actor_role,
        "authorityReference": command.authority_reference,
        "purpose": command.purpose,
        "source": command.source,
    });
    let bytes = serde_json::to_vec(&value).map_err(|_| SourceStoreError::OutcomeInvalid)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn command(content: &str, key: &str) -> SourceIntakeCommand {
        SourceIntakeCommand {
            contract_id: "A-001".to_owned(),
            contract_version: A001_VERSION.to_owned(),
            message_type: "source-intake.command".to_owned(),
            command_id: format!("source-command:{}", Uuid::new_v4()),
            action: "submit-to-quarantine".to_owned(),
            environment_id: "environment-test-0001".to_owned(),
            demonstration_session_id: "session:test-0001".to_owned(),
            engagement_id: "engagement:harbour-support-review".to_owned(),
            actor_id: "synthetic-reviewer".to_owned(),
            actor_role: "workbench-reviewer".to_owned(),
            authority_reference: "application-session:test-0001".to_owned(),
            purpose: "governed-source-intake".to_owned(),
            correlation_id: "session:test-0001".to_owned(),
            causation_id: "user-action:test-0001".to_owned(),
            idempotency_key: key.to_owned(),
            issued_at: "2026-09-01T15:00:00Z".to_owned(),
            source: ppl_contracts::SourceIntakePayload {
                acquisition_mode: SourceAcquisitionMode::Paste,
                original_name: None,
                media_type: "text/plain".to_owned(),
                size_bytes: content.len() as u64,
                content: content.to_owned(),
                title: "Synthetic policy note".to_owned(),
                owner: "Harbour Community Support".to_owned(),
                rights: "Synthetic demonstration fixture".to_owned(),
                provenance: "Created for Gate C system testing".to_owned(),
                classification: "synthetic".to_owned(),
            },
        }
    }

    #[test]
    fn quarantines_one_immutable_version_without_returning_content() {
        let directory = tempdir().expect("temporary directory");
        let store = SourceStore::open(directory.path().join("source.sqlite")).expect("store");
        let outcome = store
            .apply(
                &command("Synthetic policy text", "source-intake:test-0001"),
                "environment-test-0001",
                "2026-09-01T15:00:01Z",
            )
            .expect("accepted source");
        assert_eq!(outcome.status, SourceIntakeStatus::Quarantined);
        assert_eq!(
            outcome.event_types,
            ["source.received", "source.quarantined"]
        );
        let encoded = serde_json::to_string(&outcome).expect("outcome JSON");
        assert!(!encoded.contains("Synthetic policy text"));
        assert_eq!(store.pending_events().expect("outbox").len(), 2);
    }

    #[test]
    fn exact_redelivery_survives_reopen_and_changed_content_is_refused() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("source.sqlite");
        let accepted = command("Synthetic policy text", "source-intake:test-0002");
        let first = SourceStore::open(&path)
            .expect("store")
            .apply(&accepted, "environment-test-0001", "2026-09-01T15:00:01Z")
            .expect("first outcome");
        let reopened = SourceStore::open(&path).expect("reopened store");
        let duplicate = reopened
            .apply(&accepted, "environment-test-0001", "2026-09-01T15:00:02Z")
            .expect("duplicate outcome");
        assert_eq!(duplicate, first);

        let mut changed = accepted;
        changed.source.content = "Changed synthetic text".to_owned();
        changed.source.size_bytes = changed.source.content.len() as u64;
        let conflict = reopened
            .apply(&changed, "environment-test-0001", "2026-09-01T15:00:03Z")
            .expect("conflict outcome");
        assert_eq!(conflict.status, SourceIntakeStatus::Refused);
        assert_eq!(conflict.code, "idempotency-content-conflict");
    }

    #[test]
    fn empty_oversized_and_non_synthetic_sources_are_refused() {
        let directory = tempdir().expect("temporary directory");
        let store = SourceStore::open(directory.path().join("source.sqlite")).expect("store");
        let empty = store
            .apply(
                &command("", "source-intake:test-empty"),
                "environment-test-0001",
                "2026-09-01T15:00:01Z",
            )
            .expect("empty refusal");
        assert_eq!(empty.code, "source-empty");

        let large_text = "x".repeat(MAX_SOURCE_BYTES + 1);
        let oversized = store
            .apply(
                &command(&large_text, "source-intake:test-large"),
                "environment-test-0001",
                "2026-09-01T15:00:02Z",
            )
            .expect("size refusal");
        assert_eq!(oversized.code, "source-size-refused");

        let mut classified = command("Synthetic text", "source-intake:test-classification");
        classified.source.classification = "internal".to_owned();
        let refused = store
            .apply(&classified, "environment-test-0001", "2026-09-01T15:00:03Z")
            .expect("classification refusal");
        assert_eq!(refused.code, "information-classification-refused");
    }

    #[test]
    fn query_is_environment_bound_and_outbox_is_conclusive() {
        let directory = tempdir().expect("temporary directory");
        let store = SourceStore::open(directory.path().join("source.sqlite")).expect("store");
        let command = command("Synthetic text", "source-intake:test-query");
        let outcome = store
            .apply(&command, "environment-test-0001", "2026-09-01T15:00:01Z")
            .expect("source outcome");
        assert_eq!(
            store
                .outcome(&command.command_id, "environment-test-0001")
                .expect("query outcome"),
            outcome,
        );
        assert!(matches!(
            store.outcome(&command.command_id, "environment-another"),
            Err(SourceStoreError::NotFound)
        ));
        let events = store.pending_events().expect("pending events");
        store
            .mark_published(&events[0].event_id, "2026-09-01T15:00:02Z")
            .expect("published event");
        assert_eq!(store.pending_events().expect("remaining events").len(), 1);
    }
}
