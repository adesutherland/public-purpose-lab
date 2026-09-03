//! Gate C bounded knowledge-processing reference binding.
//!
//! `KNO-01` consumes metadata-only staged-source facts, keeps an idempotent
//! processing record and stores only a bounded derived preview. Source content
//! is accepted solely through the protected `CNT-01` workload exchange and is
//! never included in lifecycle events or general logs.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use ppl_contracts::{
    BoundedProcessingResult, K001_VERSION, OperationalEvent, ProcessingLifecycleStage,
    ProcessingLifecycleState, ProcessingLifecycleStatus, StagedSourceContent,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const SAFE_PREVIEW_CHARACTERS: usize = 240;
pub const PROCESSING_FAILURE_FIXTURE: &str = "[[PPL_PROCESSING_FAILURE]]";

#[derive(Clone, Debug)]
pub struct ProcessingStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessingLifecycleEvent {
    pub event_id: String,
    pub event_type: String,
    pub processing_id: String,
    pub source_version_id: String,
    pub correlation_id: String,
    pub causation_id: String,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessingStoreError {
    #[error("processing-store-unavailable")]
    Unavailable,
    #[error("processing-store-schema-unsupported")]
    SchemaUnsupported,
    #[error("processing-record-not-found")]
    NotFound,
    #[error("processing-record-invalid")]
    Invalid,
    #[error("processing-event-refused")]
    EventRefused,
}

impl ProcessingStore {
    /// Opens or creates the component-owned processing store.
    ///
    /// # Errors
    /// Fails closed when the path, schema or database settings are unavailable.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ProcessingStoreError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| ProcessingStoreError::Unavailable)?;
        }
        let store = Self { path };
        initialise(&store.connect()?)?;
        Ok(store)
    }

    /// Records one exact staged-source fact at most once.
    ///
    /// # Errors
    /// Refuses non-staged, cross-environment or incomplete facts.
    pub fn accept(
        &self,
        event: &OperationalEvent,
        expected_environment: &str,
        recorded_at: &str,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        let correlation_id = event
            .correlation_id
            .as_deref()
            .ok_or(ProcessingStoreError::EventRefused)?;
        let source_version_id = event
            .subject_reference
            .as_deref()
            .ok_or(ProcessingStoreError::EventRefused)?;
        if event.event_type != "source.staged"
            || event.component_id != "CNT-01"
            || event.environment_id != expected_environment
            || !source_version_id.starts_with("source-version:")
            || !correlation_id.starts_with("session:")
        {
            return Err(ProcessingStoreError::EventRefused);
        }

        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        if let Some(existing) =
            status_optional(&transaction, source_version_id, expected_environment)?
        {
            return Ok(existing);
        }

        let processing_id = format!("processing:{}", Uuid::new_v4());
        let stage = ProcessingLifecycleStage {
            state: ProcessingLifecycleState::Accepted,
            occurred_at: recorded_at.to_owned(),
            reason_code: None,
        };
        let stages_json =
            serde_json::to_string(&vec![stage]).map_err(|_| ProcessingStoreError::Invalid)?;
        transaction
            .execute(
                "INSERT INTO processing_records (
                   processing_id, source_version_id, source_event_id, environment_id,
                   demonstration_session_id, engagement_id, correlation_id, causation_id,
                   lifecycle_status, stages_json, terminal_count, recorded_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'accepted', ?9, 0, ?10)",
                params![
                    processing_id,
                    source_version_id,
                    event.event_id,
                    expected_environment,
                    correlation_id,
                    "engagement:harbour-support-review",
                    correlation_id,
                    event.event_id,
                    stages_json,
                    recorded_at,
                ],
            )
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        insert_event(
            &transaction,
            &ProcessingLifecycleEvent {
                event_id: format!("event:{}", Uuid::new_v4()),
                event_type: "processing.accepted".to_owned(),
                processing_id: processing_id.clone(),
                source_version_id: source_version_id.to_owned(),
                correlation_id: correlation_id.to_owned(),
                causation_id: event.event_id.clone(),
                occurred_at: recorded_at.to_owned(),
                reason_code: None,
            },
        )?;
        transaction
            .commit()
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        self.status(source_version_id, expected_environment)
    }

    /// Marks accepted work as processing, without duplicating the transition.
    ///
    /// # Errors
    /// Fails closed when the record cannot be read or updated.
    pub fn start(
        &self,
        source_version_id: &str,
        expected_environment: &str,
        recorded_at: &str,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        self.transition(
            source_version_id,
            expected_environment,
            ProcessingLifecycleState::Processing,
            recorded_at,
            None,
            None,
        )
    }

    /// Records one terminal completed result at most once.
    ///
    /// # Errors
    /// Fails closed when the record cannot be read or updated.
    pub fn complete(
        &self,
        source_version_id: &str,
        expected_environment: &str,
        result: &BoundedProcessingResult,
        recorded_at: &str,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        self.transition(
            source_version_id,
            expected_environment,
            ProcessingLifecycleState::Completed,
            recorded_at,
            Some(result),
            None,
        )
    }

    /// Records one terminal failed result at most once.
    ///
    /// # Errors
    /// Fails closed when the record cannot be read or updated.
    pub fn fail(
        &self,
        source_version_id: &str,
        expected_environment: &str,
        reason_code: &str,
        recorded_at: &str,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        self.transition(
            source_version_id,
            expected_environment,
            ProcessingLifecycleState::Failed,
            recorded_at,
            None,
            Some(reason_code),
        )
    }

    fn transition(
        &self,
        source_version_id: &str,
        expected_environment: &str,
        requested_state: ProcessingLifecycleState,
        recorded_at: &str,
        result: Option<&BoundedProcessingResult>,
        reason_code: Option<&str>,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        let current = status(&transaction, source_version_id, expected_environment)?;
        if matches!(
            current.lifecycle_status,
            ProcessingLifecycleState::Completed | ProcessingLifecycleState::Failed
        ) || current.lifecycle_status == requested_state
        {
            return Ok(current);
        }
        if requested_state == ProcessingLifecycleState::Processing
            && current.lifecycle_status != ProcessingLifecycleState::Accepted
            || matches!(
                requested_state,
                ProcessingLifecycleState::Completed | ProcessingLifecycleState::Failed
            ) && !matches!(
                current.lifecycle_status,
                ProcessingLifecycleState::Accepted | ProcessingLifecycleState::Processing
            )
        {
            return Err(ProcessingStoreError::Invalid);
        }

        let mut stages = current.stages;
        stages.push(ProcessingLifecycleStage {
            state: requested_state,
            occurred_at: recorded_at.to_owned(),
            reason_code: reason_code.map(str::to_owned),
        });
        let stages_json =
            serde_json::to_string(&stages).map_err(|_| ProcessingStoreError::Invalid)?;
        let result_json = result
            .map(serde_json::to_string)
            .transpose()
            .map_err(|_| ProcessingStoreError::Invalid)?;
        let terminal = matches!(
            requested_state,
            ProcessingLifecycleState::Completed | ProcessingLifecycleState::Failed
        );
        transaction
            .execute(
                "UPDATE processing_records SET lifecycle_status = ?2, stages_json = ?3,
                   result_json = ?4, reason_code = ?5,
                   terminal_count = terminal_count + ?6, recorded_at = ?7
                 WHERE source_version_id = ?1 AND environment_id = ?8",
                params![
                    source_version_id,
                    state_name(requested_state),
                    stages_json,
                    result_json,
                    reason_code,
                    i64::from(terminal),
                    recorded_at,
                    expected_environment,
                ],
            )
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        let event_type = match requested_state {
            ProcessingLifecycleState::Accepted => return Err(ProcessingStoreError::Invalid),
            ProcessingLifecycleState::Processing => "processing.started",
            ProcessingLifecycleState::Completed => "processing.completed",
            ProcessingLifecycleState::Failed => "processing.failed",
        };
        insert_event(
            &transaction,
            &ProcessingLifecycleEvent {
                event_id: format!("event:{}", Uuid::new_v4()),
                event_type: event_type.to_owned(),
                processing_id: current.processing_id,
                source_version_id: source_version_id.to_owned(),
                correlation_id: current.correlation_id,
                causation_id: current.causation_id,
                occurred_at: recorded_at.to_owned(),
                reason_code: reason_code.map(str::to_owned),
            },
        )?;
        transaction
            .commit()
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        self.status(source_version_id, expected_environment)
    }

    /// Returns one processing status by immutable source version.
    ///
    /// # Errors
    /// Returns not found without inventing processing state.
    pub fn status(
        &self,
        source_version_id: &str,
        expected_environment: &str,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        status(&self.connect()?, source_version_id, expected_environment)
    }

    /// Returns the latest processing status for one Demonstration Session.
    ///
    /// # Errors
    /// Returns not found when the session has no accepted processing work.
    pub fn latest_for_session(
        &self,
        demonstration_session_id: &str,
        expected_environment: &str,
    ) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
        let connection = self.connect()?;
        let source_version_id = connection
            .query_row(
                "SELECT source_version_id FROM processing_records
                 WHERE demonstration_session_id = ?1 AND environment_id = ?2
                 ORDER BY rowid DESC LIMIT 1",
                params![demonstration_session_id, expected_environment],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| ProcessingStoreError::Unavailable)?
            .ok_or(ProcessingStoreError::NotFound)?;
        status(&connection, &source_version_id, expected_environment)
    }

    /// Returns work that was accepted or interrupted while processing.
    ///
    /// # Errors
    /// Fails closed when component state cannot be read.
    pub fn reconcilable(&self) -> Result<Vec<String>, ProcessingStoreError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT source_version_id FROM processing_records
                 WHERE lifecycle_status IN ('accepted', 'processing') ORDER BY rowid",
            )
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| ProcessingStoreError::Unavailable)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ProcessingStoreError::Unavailable)
    }

    /// Returns metadata-only lifecycle events awaiting confirmed publication.
    ///
    /// # Errors
    /// Fails closed when component state cannot be read.
    pub fn pending_events(&self) -> Result<Vec<ProcessingLifecycleEvent>, ProcessingStoreError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare("SELECT event_json FROM processing_outbox WHERE published_at IS NULL ORDER BY rowid")
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        rows.map(|row| {
            serde_json::from_str(&row.map_err(|_| ProcessingStoreError::Unavailable)?)
                .map_err(|_| ProcessingStoreError::Invalid)
        })
        .collect()
    }

    /// Marks one lifecycle event published after durable and operational acknowledgement.
    ///
    /// # Errors
    /// Returns not found when the event was already concluded or never existed.
    pub fn mark_published(
        &self,
        event_id: &str,
        published_at: &str,
    ) -> Result<(), ProcessingStoreError> {
        let changed = self
            .connect()?
            .execute(
                "UPDATE processing_outbox SET published_at = ?2
                 WHERE event_id = ?1 AND published_at IS NULL",
                params![event_id, published_at],
            )
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(ProcessingStoreError::NotFound)
        }
    }

    fn connect(&self) -> Result<Connection, ProcessingStoreError> {
        let connection =
            Connection::open(&self.path).map_err(|_| ProcessingStoreError::Unavailable)?;
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| ProcessingStoreError::Unavailable)?;
        Ok(connection)
    }
}

/// Performs the deliberately basic and inspectable Gate C text processing.
///
/// # Errors
/// Returns a safe reason code for digest mismatch or the disclosed failure fixture.
pub fn inspect_content(
    input: &StagedSourceContent,
) -> Result<BoundedProcessingResult, &'static str> {
    if input.content.contains(PROCESSING_FAILURE_FIXTURE) {
        return Err("processing-fixture-failure");
    }
    let digest = format!("{:x}", Sha256::digest(input.content.as_bytes()));
    if input.digest_algorithm != "sha-256"
        || digest != input.digest_value
        || input.size_bytes != input.content.len() as u64
    {
        return Err("source-digest-mismatch");
    }
    let character_count = input.content.chars().count();
    let safe_preview = input
        .content
        .chars()
        .take(SAFE_PREVIEW_CHARACTERS)
        .collect::<String>();
    let line_count = input.content.lines().count().max(1) as u64;
    let heading_count = input
        .content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            let hashes = trimmed
                .chars()
                .take_while(|character| *character == '#')
                .count();
            hashes > 0
                && hashes <= 6
                && trimmed.chars().nth(hashes).is_some_and(char::is_whitespace)
        })
        .count();
    let section_count = if heading_count > 0 {
        heading_count
    } else {
        input
            .content
            .split("\n\n")
            .filter(|section| !section.trim().is_empty())
            .count()
            .max(1)
    } as u64;
    Ok(BoundedProcessingResult {
        digest_verified: true,
        byte_count: input.content.len() as u64,
        line_count,
        section_count,
        safe_preview,
        preview_truncated: character_count > SAFE_PREVIEW_CHARACTERS,
    })
}

fn initialise(connection: &Connection) -> Result<(), ProcessingStoreError> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| ProcessingStoreError::Unavailable)?;
    if version > 1 {
        return Err(ProcessingStoreError::SchemaUnsupported);
    }
    connection
        .execute_batch(
            "BEGIN IMMEDIATE;
             CREATE TABLE IF NOT EXISTS processing_records (
               processing_id TEXT PRIMARY KEY,
               source_version_id TEXT NOT NULL UNIQUE,
               source_event_id TEXT NOT NULL UNIQUE,
               environment_id TEXT NOT NULL,
               demonstration_session_id TEXT NOT NULL,
               engagement_id TEXT NOT NULL,
               correlation_id TEXT NOT NULL,
               causation_id TEXT NOT NULL,
               lifecycle_status TEXT NOT NULL CHECK (lifecycle_status IN ('accepted', 'processing', 'completed', 'failed')),
               stages_json TEXT NOT NULL,
               result_json TEXT,
               reason_code TEXT,
               terminal_count INTEGER NOT NULL CHECK (terminal_count IN (0, 1)),
               recorded_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS processing_outbox (
               event_id TEXT PRIMARY KEY,
               processing_id TEXT NOT NULL,
               event_type TEXT NOT NULL,
               event_json TEXT NOT NULL,
               published_at TEXT,
               UNIQUE (processing_id, event_type),
               FOREIGN KEY (processing_id) REFERENCES processing_records(processing_id)
             );
             PRAGMA user_version = 1;
             COMMIT;",
        )
        .map_err(|_| ProcessingStoreError::Unavailable)
}

fn status_optional(
    connection: &Connection,
    source_version_id: &str,
    expected_environment: &str,
) -> Result<Option<ProcessingLifecycleStatus>, ProcessingStoreError> {
    let row = connection
        .query_row(
            "SELECT processing_id, environment_id, demonstration_session_id,
                    engagement_id, correlation_id, causation_id, lifecycle_status,
                    stages_json, result_json, reason_code, terminal_count, recorded_at
             FROM processing_records WHERE source_version_id = ?1 AND environment_id = ?2",
            params![source_version_id, expected_environment],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, String>(11)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ProcessingStoreError::Unavailable)?;
    row.map(
        |(
            processing_id,
            environment_id,
            demonstration_session_id,
            engagement_id,
            correlation_id,
            causation_id,
            lifecycle_status,
            stages_json,
            result_json,
            reason_code,
            terminal_count,
            recorded_at,
        )| {
            Ok(ProcessingLifecycleStatus {
                contract_id: "K-001".to_owned(),
                contract_version: K001_VERSION.to_owned(),
                message_type: "processing-lifecycle.status".to_owned(),
                status_id: format!("processing-status:{}", Uuid::new_v4()),
                processing_id,
                environment_id,
                demonstration_session_id,
                engagement_id,
                source_version_id: source_version_id.to_owned(),
                correlation_id,
                causation_id,
                lifecycle_status: parse_state(&lifecycle_status)?,
                stages: serde_json::from_str(&stages_json)
                    .map_err(|_| ProcessingStoreError::Invalid)?,
                result: result_json
                    .map(|value| serde_json::from_str(&value))
                    .transpose()
                    .map_err(|_| ProcessingStoreError::Invalid)?,
                reason_code,
                terminal_count: u64::try_from(terminal_count)
                    .map_err(|_| ProcessingStoreError::Invalid)?,
                recorded_at,
            })
        },
    )
    .transpose()
}

fn status(
    connection: &Connection,
    source_version_id: &str,
    expected_environment: &str,
) -> Result<ProcessingLifecycleStatus, ProcessingStoreError> {
    status_optional(connection, source_version_id, expected_environment)?
        .ok_or(ProcessingStoreError::NotFound)
}

fn insert_event(
    transaction: &Transaction<'_>,
    event: &ProcessingLifecycleEvent,
) -> Result<(), ProcessingStoreError> {
    let json = serde_json::to_string(event).map_err(|_| ProcessingStoreError::Invalid)?;
    transaction
        .execute(
            "INSERT INTO processing_outbox (event_id, processing_id, event_type, event_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![event.event_id, event.processing_id, event.event_type, json],
        )
        .map_err(|_| ProcessingStoreError::Unavailable)?;
    Ok(())
}

const fn state_name(state: ProcessingLifecycleState) -> &'static str {
    match state {
        ProcessingLifecycleState::Accepted => "accepted",
        ProcessingLifecycleState::Processing => "processing",
        ProcessingLifecycleState::Completed => "completed",
        ProcessingLifecycleState::Failed => "failed",
    }
}

fn parse_state(value: &str) -> Result<ProcessingLifecycleState, ProcessingStoreError> {
    match value {
        "accepted" => Ok(ProcessingLifecycleState::Accepted),
        "processing" => Ok(ProcessingLifecycleState::Processing),
        "completed" => Ok(ProcessingLifecycleState::Completed),
        "failed" => Ok(ProcessingLifecycleState::Failed),
        _ => Err(ProcessingStoreError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staged_event() -> OperationalEvent {
        OperationalEvent {
            contract_id: "O-001".to_owned(),
            contract_version: "0.1.0".to_owned(),
            event_id: "event:source-staged-0001".to_owned(),
            event_type: "source.staged".to_owned(),
            component_id: "CNT-01".to_owned(),
            component_name: "source-governance".to_owned(),
            instance_id: "source-governance-1".to_owned(),
            workload_identity: "workload:source-governance".to_owned(),
            environment_id: "environment-gate-c-test".to_owned(),
            status: "accepted".to_owned(),
            capability: "governed source intake".to_owned(),
            source_revision: "test".to_owned(),
            image_digest: "sha256:test".to_owned(),
            occurred_at: "2026-09-03T10:00:00Z".to_owned(),
            information_profile: "synthetic-only".to_owned(),
            command_name: Some("release-to-staging".to_owned()),
            correlation_id: Some("session:gate-c-test".to_owned()),
            causation_id: Some("stage-command:test".to_owned()),
            idempotency_key: None,
            reason_code: None,
            subject_reference: Some("source-version:gate-c-test".to_owned()),
        }
    }

    fn content(text: &str) -> StagedSourceContent {
        StagedSourceContent {
            contract_id: "K-001".to_owned(),
            contract_version: K001_VERSION.to_owned(),
            message_type: "staged-source-content.response".to_owned(),
            response_id: "content-response:test".to_owned(),
            query_id: "content-query:test".to_owned(),
            environment_id: "environment-gate-c-test".to_owned(),
            demonstration_session_id: "session:gate-c-test".to_owned(),
            engagement_id: "engagement:harbour-support-review".to_owned(),
            source_version_id: "source-version:gate-c-test".to_owned(),
            media_type: "text/markdown".to_owned(),
            size_bytes: text.len() as u64,
            digest_algorithm: "sha-256".to_owned(),
            digest_value: format!("{:x}", Sha256::digest(text.as_bytes())),
            content: text.to_owned(),
            released_to_component: "KNO-01".to_owned(),
            purpose: "bounded-source-processing".to_owned(),
            recorded_at: "2026-09-03T10:00:01Z".to_owned(),
        }
    }

    #[test]
    fn exact_staged_redelivery_and_restart_create_one_terminal_result() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("processing.sqlite");
        let store = ProcessingStore::open(&path).expect("store");
        let event = staged_event();
        let first = store
            .accept(&event, &event.environment_id, "2026-09-03T10:00:00Z")
            .expect("accepted");
        let duplicate = store
            .accept(&event, &event.environment_id, "2026-09-03T10:00:02Z")
            .expect("duplicate");
        assert_eq!(first.processing_id, duplicate.processing_id);
        store
            .start(
                "source-version:gate-c-test",
                &event.environment_id,
                "2026-09-03T10:00:03Z",
            )
            .expect("started");
        let result =
            inspect_content(&content("# Synthetic section\n\nTwo lines.")).expect("result");
        let completed = store
            .complete(
                "source-version:gate-c-test",
                &event.environment_id,
                &result,
                "2026-09-03T10:00:04Z",
            )
            .expect("completed");
        assert_eq!(completed.terminal_count, 1);
        assert_eq!(completed.stages.len(), 3);

        let reopened = ProcessingStore::open(&path).expect("reopened");
        let reconciled = reopened
            .complete(
                "source-version:gate-c-test",
                &event.environment_id,
                &result,
                "2026-09-03T10:01:00Z",
            )
            .expect("reconciled");
        assert_eq!(completed.processing_id, reconciled.processing_id);
        assert_eq!(reconciled.terminal_count, 1);
        assert_eq!(reopened.pending_events().expect("events").len(), 3);
        let event_json = serde_json::to_string(&reopened.pending_events().expect("events"))
            .expect("metadata-only events");
        assert!(!event_json.contains("Two lines"));
    }

    #[test]
    fn bounded_processing_counts_and_failure_are_explicit() {
        let text = "# One\nSynthetic line.\n\n## Two\nAnother line.";
        let result = inspect_content(&content(text)).expect("bounded result");
        assert!(result.digest_verified);
        assert_eq!(result.byte_count, text.len() as u64);
        assert_eq!(result.line_count, 5);
        assert_eq!(result.section_count, 2);
        assert_eq!(result.safe_preview, text);
        assert!(!result.preview_truncated);
        assert_eq!(
            inspect_content(&content(PROCESSING_FAILURE_FIXTURE)),
            Err("processing-fixture-failure")
        );
    }
}
