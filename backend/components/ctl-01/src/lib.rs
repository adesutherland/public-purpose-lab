//! CTL-01 Scenario Director and its component-owned SQLite repository.
//!
//! The M3.3 runtime is single-writer and synthetic-only. Inbox, state and
//! outbox decisions commit together; presentation progress never becomes a
//! business, legal, clinical or compliance assertion.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use ppl_contracts::{
    CommandOutcome, PresentationCueOutcome, PresentationOutcomeResult, PresentationRegistration,
    ScenarioControlCommand, ScenarioLifecycleAction, ScenarioLifecycleCommand, ScenarioPackage,
    ScenarioState,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const MAX_DOCUMENT_BYTES: u64 = 65_536;
const SQLITE_BUSY_TIMEOUT_SECONDS: u64 = 5;
const ASSURANCE_PACKAGE_ID: &str = "presentation-control-assurance";
const ASSURANCE_PACKAGE_VERSION: &str = "1.2.1";

#[derive(Debug, thiserror::Error)]
pub enum DirectorError {
    #[error("state-unavailable")]
    StateUnavailable,
    #[error("state-inconsistent")]
    StateInconsistent,
    #[error("package-refused:{0}")]
    PackageRefused(&'static str),
    #[error("operation-refused:{0}")]
    OperationRefused(&'static str),
}

#[derive(Clone, Debug)]
pub struct DirectorRuntime {
    database_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PackageAdmission {
    pub package_id: String,
    pub package_version: String,
    pub package_digest: String,
    pub scenario_digest: String,
    pub source_revision: String,
    pub image_digest: String,
    pub initial_logical_time: String,
    pub maximum_advance_seconds: u64,
    pub admitted_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub session_id: String,
    pub package_id: String,
    pub package_version: String,
    pub state: ScenarioState,
    pub revision: u64,
    pub logical_time: String,
    pub logical_time_initialised: bool,
    pub predecessor_session_id: Option<String>,
    pub successor_session_id: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LifecycleOutcome {
    pub operation_id: String,
    pub status: String,
    pub code: String,
    pub session: Option<SessionSnapshot>,
    pub successor: Option<SessionSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioTimeOutcome {
    pub operation_id: String,
    pub status: String,
    pub code: String,
    pub session_id: String,
    pub session_revision: u64,
    pub logical_time: String,
    pub observed_operational_time: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckpointSnapshot {
    pub session_id: String,
    pub claim_class: String,
    pub claim_id: String,
    pub result: String,
    pub source_reference: Option<String>,
    pub observed_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OutboxRecord {
    pub sequence: i64,
    pub event_type: String,
    pub payload: Value,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PackageManifest {
    manifest_version: String,
    package_id: String,
    package_version: String,
    canonicalisation: String,
    digest_algorithm: String,
    scenario: PackageFile,
    fixtures: Vec<PackageFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PackageFile {
    path: String,
    media_type: String,
    schema_id: String,
    digest: String,
    size_bytes: u64,
}

impl DirectorRuntime {
    /// Opens or creates CTL-01 state and verifies basic database integrity.
    ///
    /// # Errors
    /// Returns a safe state error if storage cannot be migrated or checked.
    pub fn open(database_path: impl Into<PathBuf>) -> Result<Self, DirectorError> {
        let runtime = Self {
            database_path: database_path.into(),
        };
        runtime.migrate()?;
        Ok(runtime)
    }

    fn connection(&self) -> Result<Connection, DirectorError> {
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent).map_err(|_| DirectorError::StateUnavailable)?;
        }
        let connection =
            Connection::open(&self.database_path).map_err(|_| DirectorError::StateUnavailable)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(SQLITE_BUSY_TIMEOUT_SECONDS))
            .map_err(|_| DirectorError::StateUnavailable)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|_| DirectorError::StateUnavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| DirectorError::StateUnavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(connection)
    }

    fn migrate(&self) -> Result<(), DirectorError> {
        let connection = self.connection()?;
        connection
            .execute_batch(
                "BEGIN IMMEDIATE;
                 CREATE TABLE IF NOT EXISTS package_admissions (
                   package_id TEXT NOT NULL, package_version TEXT NOT NULL,
                   package_digest TEXT NOT NULL, scenario_digest TEXT NOT NULL,
                   source_revision TEXT NOT NULL, image_digest TEXT NOT NULL,
                   initial_logical_time TEXT NOT NULL,
                   maximum_advance_seconds INTEGER NOT NULL,
                   admitted_at TEXT NOT NULL,
                   PRIMARY KEY(package_id,package_version));
                 CREATE TABLE IF NOT EXISTS sessions (
                   session_id TEXT PRIMARY KEY, package_id TEXT NOT NULL,
                   package_version TEXT NOT NULL, state TEXT NOT NULL,
                   revision INTEGER NOT NULL, logical_time TEXT NOT NULL,
                   logical_time_initialised INTEGER NOT NULL,
                   predecessor_session_id TEXT, successor_session_id TEXT,
                   created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS inbox (
                   operation_id TEXT PRIMARY KEY, fingerprint TEXT NOT NULL,
                   outcome_json TEXT NOT NULL, recorded_at TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS surface_observations (
                   session_id TEXT NOT NULL, surface_slot TEXT NOT NULL,
                   registration_json TEXT NOT NULL,
                   registration_revision INTEGER NOT NULL,
                   connection_generation INTEGER NOT NULL,
                   observed_at TEXT NOT NULL,
                   PRIMARY KEY(session_id,surface_slot));
                 CREATE TABLE IF NOT EXISTS presentation_outcomes (
                   outcome_id TEXT PRIMARY KEY, cue_id TEXT NOT NULL,
                   session_id TEXT NOT NULL, outcome_json TEXT NOT NULL,
                   observed_at TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS issued_cues (
                   cue_id TEXT PRIMARY KEY, fingerprint TEXT NOT NULL,
                   cue_json TEXT NOT NULL, issued_at TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS control_requests (
                   operation_id TEXT PRIMARY KEY, fingerprint TEXT NOT NULL,
                   command_json TEXT NOT NULL, outcome_json TEXT,
                   requested_at TEXT NOT NULL, concluded_at TEXT);
                 CREATE TABLE IF NOT EXISTS checkpoints (
                   session_id TEXT NOT NULL, claim_class TEXT NOT NULL,
                   claim_id TEXT NOT NULL, result TEXT NOT NULL,
                   source_reference TEXT, observed_at TEXT NOT NULL,
                   PRIMARY KEY(session_id,claim_class,claim_id));
                 CREATE TABLE IF NOT EXISTS outbox (
                   sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                   event_type TEXT NOT NULL, payload_json TEXT NOT NULL,
                   created_at TEXT NOT NULL, published_at TEXT);
                 PRAGMA user_version=1;
                 COMMIT;",
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        let check: String = connection
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(|_| DirectorError::StateUnavailable)?;
        if check == "ok" {
            Ok(())
        } else {
            Err(DirectorError::StateInconsistent)
        }
    }

    /// Validates and admits the one image-bundled assurance package.
    ///
    /// # Errors
    /// Refuses unsafe paths, files, JSON, content, profile or digest conflicts.
    pub fn admit_bundled_package(
        &self,
        directory: &Path,
        source_revision: &str,
        image_digest: &str,
        now: OffsetDateTime,
    ) -> Result<PackageAdmission, DirectorError> {
        validate_bundle_entries(directory)?;
        let manifest_bytes = read_bounded(&directory.join("manifest.json"))?;
        let scenario_bytes = read_bounded(&directory.join("scenario.json"))?;
        let manifest_value = parse_strict_json(&manifest_bytes)?;
        let scenario_value = parse_strict_json(&scenario_bytes)?;
        let manifest: PackageManifest = serde_json::from_value(manifest_value.clone())
            .map_err(|_| DirectorError::PackageRefused("manifest-invalid"))?;
        let scenario: ScenarioPackage = serde_json::from_value(scenario_value.clone())
            .map_err(|_| DirectorError::PackageRefused("scenario-invalid"))?;
        if !manifest_matches_scenario(&manifest, &scenario, scenario_bytes.len()) {
            return Err(DirectorError::PackageRefused("package-profile-invalid"));
        }
        refuse_prohibited_content(&scenario_value)?;
        let scenario_digest = canonical_digest(&scenario_value)?;
        if scenario_digest != manifest.scenario.digest {
            return Err(DirectorError::PackageRefused("scenario-digest-conflict"));
        }
        let proposed = PackageAdmission {
            package_id: manifest.package_id,
            package_version: manifest.package_version,
            package_digest: canonical_digest(&manifest_value)?,
            scenario_digest,
            source_revision: source_revision.to_owned(),
            image_digest: image_digest.to_owned(),
            initial_logical_time: scenario.controls.logical_time.initial_instant,
            maximum_advance_seconds: scenario.controls.logical_time.maximum_advance_seconds,
            admitted_at: format_time(now)?,
        };
        let connection = self.connection()?;
        let existing: Option<String> = connection
            .query_row(
                "SELECT package_digest FROM package_admissions WHERE package_id=?1 AND package_version=?2",
                params![proposed.package_id, proposed.package_version],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some(digest) = existing {
            if digest != proposed.package_digest {
                return Err(DirectorError::PackageRefused("admission-digest-conflict"));
            }
            return self.package_admission(&proposed.package_id, &proposed.package_version);
        }
        connection
            .execute(
                "INSERT INTO package_admissions VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    proposed.package_id,
                    proposed.package_version,
                    proposed.package_digest,
                    proposed.scenario_digest,
                    proposed.source_revision,
                    proposed.image_digest,
                    proposed.initial_logical_time,
                    sql_int(proposed.maximum_advance_seconds)?,
                    proposed.admitted_at
                ],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(proposed)
    }

    /// Returns one recorded package admission.
    ///
    /// # Errors
    /// Returns a safe refusal when no matching admission exists.
    pub fn package_admission(
        &self,
        package_id: &str,
        package_version: &str,
    ) -> Result<PackageAdmission, DirectorError> {
        self.connection()?
            .query_row(
                "SELECT package_id,package_version,package_digest,scenario_digest,source_revision,image_digest,initial_logical_time,maximum_advance_seconds,admitted_at FROM package_admissions WHERE package_id=?1 AND package_version=?2",
                params![package_id, package_version],
                |row| Ok(PackageAdmission {
                    package_id: row.get(0)?, package_version: row.get(1)?,
                    package_digest: row.get(2)?, scenario_digest: row.get(3)?,
                    source_revision: row.get(4)?, image_digest: row.get(5)?,
                    initial_logical_time: row.get(6)?,
                    maximum_advance_seconds: u64::try_from(row.get::<_, i64>(7)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(7, 0))?,
                    admitted_at: row.get(8)?,
                }),
            )
            .map_err(|_| DirectorError::OperationRefused("package-not-admitted"))
    }

    /// Applies one lifecycle command in an inbox/state/outbox transaction.
    ///
    /// # Errors
    /// Refuses changed duplicates, stale revisions and invalid transitions.
    pub fn apply_lifecycle(
        &self,
        command: &ScenarioLifecycleCommand,
        now: OffsetDateTime,
    ) -> Result<LifecycleOutcome, DirectorError> {
        if command.contract_id != "D-002" || command.contract_version != "1.0.0" {
            return Err(DirectorError::OperationRefused("contract-unsupported"));
        }
        let fingerprint = canonical_digest(command)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some(outcome) = duplicate_outcome(&transaction, &command.operation_id, &fingerprint)?
        {
            return Ok(outcome);
        }
        let recorded_at = format_time(now)?;
        let outcome = match command.action {
            ScenarioLifecycleAction::Create => create_session(&transaction, command, &recorded_at)?,
            ScenarioLifecycleAction::Reset => reset_session(&transaction, command, &recorded_at)?,
            action => transition_session(&transaction, command, action, &recorded_at)?,
        };
        record_operation_event(
            &transaction,
            &command.operation_id,
            &fingerprint,
            "ppl.demonstration.lifecycle.changed",
            &outcome,
            &recorded_at,
        )?;
        transaction
            .commit()
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(outcome)
    }

    /// Advances scenario logical time without changing operational expiry.
    ///
    /// # Errors
    /// Refuses stale revisions and advances outside the package bound.
    pub fn advance_logical_time(
        &self,
        operation_id: &str,
        session_id: &str,
        expected_revision: u64,
        seconds: u64,
        now: OffsetDateTime,
    ) -> Result<ScenarioTimeOutcome, DirectorError> {
        if seconds == 0 || seconds > 86_400 {
            return Err(DirectorError::OperationRefused(
                "logical-time-bound-exceeded",
            ));
        }
        let fingerprint = canonical_digest(&serde_json::json!({
            "operation": "advance",
            "sessionId": session_id,
            "expectedRevision": expected_revision,
            "advanceSeconds": seconds
        }))?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some(outcome) = duplicate_time_outcome(&transaction, operation_id, &fingerprint)? {
            return Ok(outcome);
        }
        let current = read_session(&transaction, session_id)?;
        if current.revision != expected_revision {
            return Err(DirectorError::OperationRefused("stale-revision"));
        }
        if !current.logical_time_initialised {
            return Err(DirectorError::OperationRefused(
                "logical-time-not-initialised",
            ));
        }
        let (initial_logical_time, maximum_advance_seconds): (String, i64) = transaction
            .query_row(
                "SELECT initial_logical_time,maximum_advance_seconds FROM package_admissions
                 WHERE package_id=?1 AND package_version=?2",
                params![current.package_id, current.package_version],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| DirectorError::StateInconsistent)?;
        let parsed = OffsetDateTime::parse(&current.logical_time, &Rfc3339)
            .map_err(|_| DirectorError::StateInconsistent)?;
        let duration = Duration::seconds(
            i64::try_from(seconds)
                .map_err(|_| DirectorError::OperationRefused("logical-time-bound-exceeded"))?,
        );
        let next = parsed
            .checked_add(duration)
            .ok_or(DirectorError::OperationRefused(
                "logical-time-bound-exceeded",
            ))?;
        let initial = OffsetDateTime::parse(&initial_logical_time, &Rfc3339)
            .map_err(|_| DirectorError::StateInconsistent)?;
        let maximum = initial
            .checked_add(Duration::seconds(maximum_advance_seconds))
            .ok_or(DirectorError::StateInconsistent)?;
        if next > maximum {
            return Err(DirectorError::OperationRefused(
                "logical-time-bound-exceeded",
            ));
        }
        let logical_time = format_time(next)?;
        let observed_operational_time = format_time(now)?;
        transaction
            .execute(
                "UPDATE sessions SET logical_time=?1,revision=revision+1,updated_at=?2 WHERE session_id=?3 AND revision=?4",
                params![
                    logical_time,
                    observed_operational_time,
                    session_id,
                    sql_int(expected_revision)?
                ],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        let outcome = ScenarioTimeOutcome {
            operation_id: operation_id.to_owned(),
            status: "accepted".to_owned(),
            code: "logical-time-advanced".to_owned(),
            session_id: session_id.to_owned(),
            session_revision: expected_revision + 1,
            logical_time,
            observed_operational_time: observed_operational_time.clone(),
        };
        record_operation_event(
            &transaction,
            operation_id,
            &fingerprint,
            "ppl.demonstration.logical-time.changed",
            &outcome,
            &observed_operational_time,
        )?;
        transaction
            .commit()
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(outcome)
    }

    /// Establishes the package-declared initial logical instant once.
    ///
    /// # Errors
    /// Refuses a different instant, stale revision or a second initialisation.
    pub fn set_initial_logical_time(
        &self,
        operation_id: &str,
        session_id: &str,
        expected_revision: u64,
        logical_instant: &str,
        now: OffsetDateTime,
    ) -> Result<ScenarioTimeOutcome, DirectorError> {
        let fingerprint = canonical_digest(&serde_json::json!({
            "operation": "set",
            "sessionId": session_id,
            "expectedRevision": expected_revision,
            "logicalInstant": logical_instant
        }))?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some(outcome) = duplicate_time_outcome(&transaction, operation_id, &fingerprint)? {
            return Ok(outcome);
        }
        let current = read_session(&transaction, session_id)?;
        if current.revision != expected_revision {
            return Err(DirectorError::OperationRefused("stale-revision"));
        }
        if current.logical_time_initialised {
            return Err(DirectorError::OperationRefused(
                "logical-time-already-initialised",
            ));
        }
        let declared: String = transaction
            .query_row(
                "SELECT initial_logical_time FROM package_admissions
                 WHERE package_id=?1 AND package_version=?2",
                params![current.package_id, current.package_version],
                |row| row.get(0),
            )
            .map_err(|_| DirectorError::StateInconsistent)?;
        if logical_instant != declared || OffsetDateTime::parse(logical_instant, &Rfc3339).is_err()
        {
            return Err(DirectorError::OperationRefused(
                "logical-time-initial-instant-refused",
            ));
        }
        let observed_operational_time = format_time(now)?;
        transaction
            .execute(
                "UPDATE sessions SET logical_time=?1,logical_time_initialised=1,
                 revision=revision+1,updated_at=?2 WHERE session_id=?3 AND revision=?4",
                params![
                    logical_instant,
                    observed_operational_time,
                    session_id,
                    sql_int(expected_revision)?
                ],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        let outcome = ScenarioTimeOutcome {
            operation_id: operation_id.to_owned(),
            status: "accepted".to_owned(),
            code: "logical-time-initialised".to_owned(),
            session_id: session_id.to_owned(),
            session_revision: expected_revision + 1,
            logical_time: logical_instant.to_owned(),
            observed_operational_time: observed_operational_time.clone(),
        };
        record_operation_event(
            &transaction,
            operation_id,
            &fingerprint,
            "ppl.demonstration.logical-time.changed",
            &outcome,
            &observed_operational_time,
        )?;
        transaction
            .commit()
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(outcome)
    }

    /// Records the current CTL-02 registration as a local observation.
    ///
    /// # Errors
    /// Refuses registrations for unknown or terminal sessions.
    pub fn observe_registration(
        &self,
        registration: &PresentationRegistration,
        observed_at: OffsetDateTime,
    ) -> Result<(), DirectorError> {
        let connection = self.connection()?;
        let session = read_session(&connection, &registration.session_id)?;
        if !matches!(
            session.state,
            ScenarioState::Preparing
                | ScenarioState::Ready
                | ScenarioState::Running
                | ScenarioState::Paused
        ) {
            return Err(DirectorError::OperationRefused("session-not-registrable"));
        }
        let payload =
            serde_json::to_string(registration).map_err(|_| DirectorError::StateInconsistent)?;
        connection
            .execute(
                "INSERT INTO surface_observations VALUES(?1,?2,?3,?4,?5,?6)
                 ON CONFLICT(session_id,surface_slot) DO UPDATE SET
                   registration_json=excluded.registration_json,
                   registration_revision=excluded.registration_revision,
                   connection_generation=excluded.connection_generation,
                   observed_at=excluded.observed_at
                 WHERE excluded.registration_revision>surface_observations.registration_revision
                    OR (excluded.registration_revision=surface_observations.registration_revision
                        AND excluded.connection_generation>=surface_observations.connection_generation)",
                params![
                    registration.session_id,
                    registration.surface_slot,
                    payload,
                    sql_int(registration.registration_revision)?,
                    sql_int(registration.connection_generation)?,
                    format_time(observed_at)?
                ],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(())
    }

    /// Returns CTL-01's current registration observation for cue creation.
    ///
    /// # Errors
    /// Returns a safe refusal when the slot is not observed.
    pub fn current_registration(
        &self,
        session_id: &str,
        surface_slot: &str,
    ) -> Result<PresentationRegistration, DirectorError> {
        let json: String = self
            .connection()?
            .query_row(
                "SELECT registration_json FROM surface_observations WHERE session_id=?1 AND surface_slot=?2",
                params![session_id, surface_slot],
                |row| row.get(0),
            )
            .map_err(|_| DirectorError::OperationRefused("surface-unregistered"))?;
        serde_json::from_str(&json).map_err(|_| DirectorError::StateInconsistent)
    }

    /// Validates, records and enqueues one semantic presentation cue.
    ///
    /// # Errors
    /// Refuses inactive sessions, stale registration bindings, expiry and
    /// changed-content duplicates.
    pub fn issue_cue(
        &self,
        cue: &ppl_contracts::PresentationCue,
        now: OffsetDateTime,
    ) -> Result<ppl_contracts::PresentationCue, DirectorError> {
        let session = self.session(&cue.session_id)?;
        if session.state != ScenarioState::Running || session.revision != cue.session_revision {
            return Err(DirectorError::OperationRefused("session-revision-stale"));
        }
        let registration = self.current_registration(&cue.session_id, &cue.surface_slot)?;
        if registration.registration_id != cue.registration_id
            || registration.registration_revision != cue.registration_revision
            || registration.connection_generation != cue.connection_generation
            || !registration.supported_views.contains(&cue.semantic_view)
        {
            return Err(DirectorError::OperationRefused(
                "registration-generation-stale",
            ));
        }
        let expiry = OffsetDateTime::parse(&cue.expires_at, &Rfc3339)
            .map_err(|_| DirectorError::OperationRefused("expiry-invalid"))?;
        if expiry <= now {
            return Err(DirectorError::OperationRefused("expired"));
        }
        let fingerprint = canonical_digest(cue)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some((stored_fingerprint, stored_json)) = transaction
            .query_row(
                "SELECT fingerprint,cue_json FROM issued_cues WHERE cue_id=?1",
                [&cue.cue_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?
        {
            if stored_fingerprint != fingerprint {
                return Err(DirectorError::OperationRefused(
                    "duplicate-content-conflict",
                ));
            }
            return serde_json::from_str(&stored_json)
                .map_err(|_| DirectorError::StateInconsistent);
        }
        let json = serde_json::to_string(cue).map_err(|_| DirectorError::StateInconsistent)?;
        let issued_at = format_time(now)?;
        transaction
            .execute(
                "INSERT INTO issued_cues VALUES(?1,?2,?3,?4)",
                params![cue.cue_id, fingerprint, json, issued_at],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        transaction
            .execute(
                "INSERT INTO outbox(event_type,payload_json,created_at)
                 VALUES('ppl.presentation.cue.requested',?1,?2)",
                params![json, issued_at],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        transaction
            .commit()
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(cue.clone())
    }

    /// Records and enqueues one component-owned D-003 control request.
    ///
    /// # Errors
    /// Refuses changed-content duplicates or unavailable durable state.
    pub fn request_control(
        &self,
        command: &ScenarioControlCommand,
        now: OffsetDateTime,
    ) -> Result<(), DirectorError> {
        if command.contract_id != "D-003" || command.contract_version != "1.0.0" {
            return Err(DirectorError::OperationRefused("contract-unsupported"));
        }
        let fingerprint = canonical_digest(command)?;
        let json = serde_json::to_string(command).map_err(|_| DirectorError::StateInconsistent)?;
        let requested_at = format_time(now)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| DirectorError::StateUnavailable)?;
        let existing: Option<String> = transaction
            .query_row(
                "SELECT fingerprint FROM control_requests WHERE operation_id=?1",
                [&command.operation_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some(existing) = existing {
            if existing == fingerprint {
                return Ok(());
            }
            return Err(DirectorError::OperationRefused(
                "duplicate-content-conflict",
            ));
        }
        transaction
            .execute(
                "INSERT INTO control_requests(operation_id,fingerprint,command_json,requested_at)
                 VALUES(?1,?2,?3,?4)",
                params![command.operation_id, fingerprint, json, requested_at],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        transaction
            .execute(
                "INSERT INTO outbox(event_type,payload_json,created_at)
                 VALUES('ppl.presentation.control.requested',?1,?2)",
                params![json, requested_at],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        transaction
            .commit()
            .map_err(|_| DirectorError::StateUnavailable)
    }

    /// Records a conclusive C-003 result for a prior D-003 request.
    ///
    /// # Errors
    /// Refuses unknown request identities and conflicting duplicate outcomes.
    pub fn observe_control_outcome(&self, outcome: &CommandOutcome) -> Result<(), DirectorError> {
        let json = serde_json::to_string(outcome).map_err(|_| DirectorError::StateInconsistent)?;
        let connection = self.connection()?;
        let existing: Option<Option<String>> = connection
            .query_row(
                "SELECT outcome_json FROM control_requests WHERE operation_id=?1",
                [&outcome.command_message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?;
        match existing {
            None => Err(DirectorError::OperationRefused("control-request-unknown")),
            Some(Some(existing)) if existing != json => Err(DirectorError::OperationRefused(
                "duplicate-content-conflict",
            )),
            Some(Some(_)) => Ok(()),
            Some(None) => {
                connection
                    .execute(
                        "UPDATE control_requests SET outcome_json=?1,concluded_at=?2
                         WHERE operation_id=?3 AND outcome_json IS NULL",
                        params![json, outcome.completed_at, outcome.command_message_id],
                    )
                    .map_err(|_| DirectorError::StateUnavailable)?;
                Ok(())
            }
        }
    }

    /// Reads a conclusive control result when available.
    ///
    /// # Errors
    /// Returns a state error for corrupt stored output.
    pub fn control_outcome(
        &self,
        operation_id: &str,
    ) -> Result<Option<CommandOutcome>, DirectorError> {
        let json: Option<Option<String>> = self
            .connection()?
            .query_row(
                "SELECT outcome_json FROM control_requests WHERE operation_id=?1",
                [operation_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?;
        json.flatten()
            .map(|value| serde_json::from_str(&value).map_err(|_| DirectorError::StateInconsistent))
            .transpose()
    }

    /// Records P-004 and evaluates only the presentation-progress checkpoint.
    ///
    /// # Errors
    /// Refuses business claims, stale sessions and conflicting duplicates.
    pub fn observe_presentation_outcome(
        &self,
        outcome: &PresentationCueOutcome,
    ) -> Result<CheckpointSnapshot, DirectorError> {
        if outcome.contract_id != "P-004"
            || outcome.contract_version != "1.0.0"
            || outcome.business_completion_claimed
        {
            return Err(DirectorError::OperationRefused(
                "presentation-outcome-invalid",
            ));
        }
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| DirectorError::StateUnavailable)?;
        let session = read_session(&transaction, &outcome.session_id)?;
        if session.revision != outcome.session_revision || session.state != ScenarioState::Running {
            return Err(DirectorError::OperationRefused("session-revision-stale"));
        }
        validate_presentation_outcome_binding(&transaction, outcome)?;
        let payload =
            serde_json::to_string(outcome).map_err(|_| DirectorError::StateInconsistent)?;
        let existing: Option<String> = transaction
            .query_row(
                "SELECT outcome_json FROM presentation_outcomes WHERE outcome_id=?1",
                [&outcome.outcome_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?;
        if let Some(existing) = existing {
            if existing != payload {
                return Err(DirectorError::OperationRefused(
                    "duplicate-content-conflict",
                ));
            }
        } else {
            transaction
                .execute(
                    "INSERT INTO presentation_outcomes VALUES(?1,?2,?3,?4,?5)",
                    params![
                        outcome.outcome_id,
                        outcome.cue_id,
                        outcome.session_id,
                        payload,
                        outcome.concluded_at
                    ],
                )
                .map_err(|_| DirectorError::StateUnavailable)?;
        }
        let result = match outcome.result {
            PresentationOutcomeResult::Applied | PresentationOutcomeResult::Duplicate => {
                "satisfied"
            }
            PresentationOutcomeResult::Uncertain => "uncertain",
            _ => "not-satisfied",
        };
        let checkpoint = CheckpointSnapshot {
            session_id: outcome.session_id.clone(),
            claim_class: "presentation-progress".to_owned(),
            claim_id: "welcome-presented".to_owned(),
            result: result.to_owned(),
            source_reference: Some(outcome.outcome_id.clone()),
            observed_at: outcome.concluded_at.clone(),
        };
        transaction
            .execute(
                "INSERT INTO checkpoints VALUES(?1,?2,?3,?4,?5,?6)
                 ON CONFLICT(session_id,claim_class,claim_id) DO UPDATE SET
                   result=excluded.result,source_reference=excluded.source_reference,
                   observed_at=excluded.observed_at",
                params![
                    checkpoint.session_id,
                    checkpoint.claim_class,
                    checkpoint.claim_id,
                    checkpoint.result,
                    checkpoint.source_reference,
                    checkpoint.observed_at
                ],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        transaction
            .commit()
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(checkpoint)
    }

    /// Reads one authoritative session snapshot.
    ///
    /// # Errors
    /// Returns a safe not-found or state error.
    pub fn session(&self, session_id: &str) -> Result<SessionSnapshot, DirectorError> {
        read_session(&self.connection()?, session_id)
    }

    /// Reads the current presentation-progress checkpoint when one exists.
    ///
    /// # Errors
    /// Returns a state error only when stored checkpoint data is inconsistent.
    pub fn presentation_checkpoint(
        &self,
        session_id: &str,
    ) -> Result<Option<CheckpointSnapshot>, DirectorError> {
        let row: Option<(String, String, String, Option<String>, String)> = self
            .connection()?
            .query_row(
                "SELECT claim_class,claim_id,result,source_reference,observed_at
                 FROM checkpoints WHERE session_id=?1 AND claim_class='presentation-progress'",
                [session_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| DirectorError::StateUnavailable)?;
        Ok(row.map(
            |(claim_class, claim_id, result, source_reference, observed_at)| CheckpointSnapshot {
                session_id: session_id.to_owned(),
                claim_class,
                claim_id,
                result,
                source_reference,
                observed_at,
            },
        ))
    }

    /// Reads unpublished events in sequence order.
    ///
    /// # Errors
    /// Returns a state error when the outbox is unavailable.
    pub fn pending_outbox(&self, limit: usize) -> Result<Vec<OutboxRecord>, DirectorError> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT sequence,event_type,payload_json,created_at FROM outbox
                 WHERE published_at IS NULL ORDER BY sequence LIMIT ?1",
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let rows = statement
            .query_map([limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|_| DirectorError::StateUnavailable)?;
        rows.map(|row| {
            let (sequence, event_type, payload, created_at) =
                row.map_err(|_| DirectorError::StateUnavailable)?;
            Ok(OutboxRecord {
                sequence,
                event_type,
                payload: serde_json::from_str(&payload)
                    .map_err(|_| DirectorError::StateInconsistent)?,
                created_at,
            })
        })
        .collect()
    }

    /// Marks one event published after broker acknowledgement.
    ///
    /// # Errors
    /// Returns a safe error when the marker cannot be written.
    pub fn mark_outbox_published(
        &self,
        sequence: i64,
        now: OffsetDateTime,
    ) -> Result<(), DirectorError> {
        let changed = self
            .connection()?
            .execute(
                "UPDATE outbox SET published_at=?1 WHERE sequence=?2 AND published_at IS NULL",
                params![format_time(now)?, sequence],
            )
            .map_err(|_| DirectorError::StateUnavailable)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(DirectorError::OperationRefused("outbox-record-unavailable"))
        }
    }
}

fn validate_presentation_outcome_binding(
    transaction: &Transaction<'_>,
    outcome: &PresentationCueOutcome,
) -> Result<(), DirectorError> {
    let cue_json: String = transaction
        .query_row(
            "SELECT cue_json FROM issued_cues WHERE cue_id=?1",
            [&outcome.cue_id],
            |row| row.get(0),
        )
        .map_err(|_| DirectorError::OperationRefused("cue-unavailable"))?;
    let cue: ppl_contracts::PresentationCue =
        serde_json::from_str(&cue_json).map_err(|_| DirectorError::StateInconsistent)?;
    if cue.cue_digest != outcome.cue_digest
        || cue.session_id != outcome.session_id
        || cue.session_revision != outcome.session_revision
        || cue.surface_slot != outcome.surface_slot
        || cue.registration_id != outcome.registration_id
        || cue.registration_revision != outcome.registration_revision
        || cue.connection_generation != outcome.connection_generation
        || cue.semantic_view != outcome.semantic_view
    {
        Err(DirectorError::OperationRefused(
            "presentation-outcome-binding-conflict",
        ))
    } else {
        Ok(())
    }
}

fn duplicate_outcome(
    transaction: &Transaction<'_>,
    operation_id: &str,
    fingerprint: &str,
) -> Result<Option<LifecycleOutcome>, DirectorError> {
    let existing: Option<(String, String)> = transaction
        .query_row(
            "SELECT fingerprint,outcome_json FROM inbox WHERE operation_id=?1",
            [operation_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| DirectorError::StateUnavailable)?;
    let Some((existing_fingerprint, outcome)) = existing else {
        return Ok(None);
    };
    if existing_fingerprint != fingerprint {
        return Err(DirectorError::OperationRefused(
            "duplicate-content-conflict",
        ));
    }
    serde_json::from_str(&outcome)
        .map(Some)
        .map_err(|_| DirectorError::StateInconsistent)
}

fn duplicate_time_outcome(
    transaction: &Transaction<'_>,
    operation_id: &str,
    fingerprint: &str,
) -> Result<Option<ScenarioTimeOutcome>, DirectorError> {
    let existing: Option<(String, String)> = transaction
        .query_row(
            "SELECT fingerprint,outcome_json FROM inbox WHERE operation_id=?1",
            [operation_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| DirectorError::StateUnavailable)?;
    let Some((existing_fingerprint, outcome)) = existing else {
        return Ok(None);
    };
    if existing_fingerprint != fingerprint {
        return Err(DirectorError::OperationRefused(
            "duplicate-content-conflict",
        ));
    }
    serde_json::from_str(&outcome)
        .map(Some)
        .map_err(|_| DirectorError::StateInconsistent)
}

fn create_session(
    transaction: &Transaction<'_>,
    command: &ScenarioLifecycleCommand,
    now: &str,
) -> Result<LifecycleOutcome, DirectorError> {
    if command.expected_revision != 0 {
        return Err(DirectorError::OperationRefused("stale-revision"));
    }
    let initial_logical_time: String = transaction
        .query_row(
            "SELECT initial_logical_time FROM package_admissions WHERE package_id=?1 AND package_version=?2",
            params![command.package_id, command.package_version],
            |row| row.get(0),
        )
        .map_err(|_| DirectorError::OperationRefused("package-not-admitted"))?;
    transaction
        .execute(
            "INSERT INTO sessions(session_id,package_id,package_version,state,revision,
             logical_time,logical_time_initialised,created_at,updated_at)
             VALUES(?1,?2,?3,'preparing',1,?4,0,?5,?5)",
            params![
                command.session_id,
                command.package_id,
                command.package_version,
                initial_logical_time,
                now,
            ],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                DirectorError::OperationRefused("session-already-exists")
            } else {
                DirectorError::StateUnavailable
            }
        })?;
    Ok(LifecycleOutcome {
        operation_id: command.operation_id.clone(),
        status: "accepted".to_owned(),
        code: "session-created".to_owned(),
        session: Some(read_session(transaction, &command.session_id)?),
        successor: None,
    })
}

fn transition_session(
    transaction: &Transaction<'_>,
    command: &ScenarioLifecycleCommand,
    action: ScenarioLifecycleAction,
    now: &str,
) -> Result<LifecycleOutcome, DirectorError> {
    let current = read_session(transaction, &command.session_id)?;
    if current.package_id != command.package_id
        || current.package_version != command.package_version
    {
        return Err(DirectorError::OperationRefused("package-binding-conflict"));
    }
    if current.revision != command.expected_revision
        || command
            .expected_state
            .is_some_and(|state| state != current.state)
    {
        return Err(DirectorError::OperationRefused("stale-revision"));
    }
    if action == ScenarioLifecycleAction::Prepare && !current.logical_time_initialised {
        return Err(DirectorError::OperationRefused(
            "logical-time-not-initialised",
        ));
    }
    let next = transition(current.state, action)
        .ok_or(DirectorError::OperationRefused("transition-invalid"))?;
    transaction
        .execute(
            "UPDATE sessions SET state=?1,revision=revision+1,updated_at=?2
             WHERE session_id=?3 AND revision=?4",
            params![
                state_name(next),
                now,
                command.session_id,
                sql_int(current.revision)?
            ],
        )
        .map_err(|_| DirectorError::StateUnavailable)?;
    Ok(LifecycleOutcome {
        operation_id: command.operation_id.clone(),
        status: "accepted".to_owned(),
        code: format!("session-{}", state_name(next)),
        session: Some(read_session(transaction, &command.session_id)?),
        successor: None,
    })
}

fn reset_session(
    transaction: &Transaction<'_>,
    command: &ScenarioLifecycleCommand,
    now: &str,
) -> Result<LifecycleOutcome, DirectorError> {
    let current = read_session(transaction, &command.session_id)?;
    if current.revision != command.expected_revision {
        return Err(DirectorError::OperationRefused("stale-revision"));
    }
    if !matches!(
        current.state,
        ScenarioState::Completed | ScenarioState::Stopped | ScenarioState::Failed
    ) {
        return Err(DirectorError::OperationRefused("reset-state-invalid"));
    }
    let successor_id = format!("session:{}", Uuid::new_v4());
    let initial_logical_time: String = transaction
        .query_row(
            "SELECT initial_logical_time FROM package_admissions
             WHERE package_id=?1 AND package_version=?2",
            params![current.package_id, current.package_version],
            |row| row.get(0),
        )
        .map_err(|_| DirectorError::StateInconsistent)?;
    transaction
        .execute(
            "UPDATE sessions SET state='superseded',revision=revision+1,
             successor_session_id=?1,updated_at=?2 WHERE session_id=?3 AND revision=?4",
            params![
                successor_id,
                now,
                command.session_id,
                sql_int(current.revision)?
            ],
        )
        .map_err(|_| DirectorError::StateUnavailable)?;
    transaction
        .execute(
            "INSERT INTO sessions(session_id,package_id,package_version,state,revision,
             logical_time,logical_time_initialised,predecessor_session_id,created_at,updated_at)
             VALUES(?1,?2,?3,'preparing',1,?4,0,?5,?6,?6)",
            params![
                successor_id,
                current.package_id,
                current.package_version,
                initial_logical_time,
                current.session_id,
                now
            ],
        )
        .map_err(|_| DirectorError::StateUnavailable)?;
    Ok(LifecycleOutcome {
        operation_id: command.operation_id.clone(),
        status: "accepted".to_owned(),
        code: "session-reset-successor-created".to_owned(),
        session: Some(read_session(transaction, &command.session_id)?),
        successor: Some(read_session(transaction, &successor_id)?),
    })
}

fn read_session(
    connection: &Connection,
    session_id: &str,
) -> Result<SessionSnapshot, DirectorError> {
    let row = connection
        .query_row(
            "SELECT session_id,package_id,package_version,state,revision,logical_time,
             logical_time_initialised,predecessor_session_id,successor_session_id,updated_at FROM sessions
             WHERE session_id=?1",
            [session_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                ))
            },
        )
        .map_err(|_| DirectorError::OperationRefused("session-not-found"))?;
    Ok(SessionSnapshot {
        session_id: row.0,
        package_id: row.1,
        package_version: row.2,
        state: parse_state(&row.3)?,
        revision: u64::try_from(row.4).map_err(|_| DirectorError::StateInconsistent)?,
        logical_time: row.5,
        logical_time_initialised: row.6 == 1,
        predecessor_session_id: row.7,
        successor_session_id: row.8,
        updated_at: row.9,
    })
}

fn transition(state: ScenarioState, action: ScenarioLifecycleAction) -> Option<ScenarioState> {
    match (state, action) {
        (ScenarioState::Preparing, ScenarioLifecycleAction::Prepare) => Some(ScenarioState::Ready),
        (ScenarioState::Ready, ScenarioLifecycleAction::Start)
        | (ScenarioState::Paused, ScenarioLifecycleAction::Resume) => Some(ScenarioState::Running),
        (ScenarioState::Running, ScenarioLifecycleAction::Pause) => Some(ScenarioState::Paused),
        (ScenarioState::Running, ScenarioLifecycleAction::Complete) => {
            Some(ScenarioState::Completed)
        }
        (
            ScenarioState::Preparing
            | ScenarioState::Ready
            | ScenarioState::Running
            | ScenarioState::Paused,
            ScenarioLifecycleAction::Stop,
        ) => Some(ScenarioState::Stopped),
        _ => None,
    }
}

fn state_name(state: ScenarioState) -> &'static str {
    match state {
        ScenarioState::Preparing => "preparing",
        ScenarioState::Ready => "ready",
        ScenarioState::Running => "running",
        ScenarioState::Paused => "paused",
        ScenarioState::Completed => "completed",
        ScenarioState::Stopped => "stopped",
        ScenarioState::Failed => "failed",
        ScenarioState::Superseded => "superseded",
    }
}

fn parse_state(value: &str) -> Result<ScenarioState, DirectorError> {
    match value {
        "preparing" => Ok(ScenarioState::Preparing),
        "ready" => Ok(ScenarioState::Ready),
        "running" => Ok(ScenarioState::Running),
        "paused" => Ok(ScenarioState::Paused),
        "completed" => Ok(ScenarioState::Completed),
        "stopped" => Ok(ScenarioState::Stopped),
        "failed" => Ok(ScenarioState::Failed),
        "superseded" => Ok(ScenarioState::Superseded),
        _ => Err(DirectorError::StateInconsistent),
    }
}

fn record_operation_event<T: Serialize>(
    transaction: &Transaction<'_>,
    operation_id: &str,
    fingerprint: &str,
    event_type: &str,
    outcome: &T,
    now: &str,
) -> Result<(), DirectorError> {
    let json = serde_json::to_string(outcome).map_err(|_| DirectorError::StateInconsistent)?;
    transaction
        .execute(
            "INSERT INTO inbox VALUES(?1,?2,?3,?4)",
            params![operation_id, fingerprint, json, now],
        )
        .map_err(|_| DirectorError::StateUnavailable)?;
    transaction
        .execute(
            "INSERT INTO outbox(event_type,payload_json,created_at) VALUES(?1,?2,?3)",
            params![event_type, json, now],
        )
        .map_err(|_| DirectorError::StateUnavailable)?;
    Ok(())
}

fn format_time(value: OffsetDateTime) -> Result<String, DirectorError> {
    value
        .format(&Rfc3339)
        .map_err(|_| DirectorError::StateInconsistent)
}

fn sql_int(value: u64) -> Result<i64, DirectorError> {
    i64::try_from(value).map_err(|_| DirectorError::OperationRefused("value-out-of-range"))
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, DirectorError> {
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| DirectorError::StateInconsistent)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_bundle_entries(directory: &Path) -> Result<(), DirectorError> {
    let entries =
        fs::read_dir(directory).map_err(|_| DirectorError::PackageRefused("bundle-unavailable"))?;
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|_| DirectorError::PackageRefused("bundle-unavailable"))?;
        let metadata = entry
            .path()
            .symlink_metadata()
            .map_err(|_| DirectorError::PackageRefused("bundle-unavailable"))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(DirectorError::PackageRefused("unsafe-bundle-entry"));
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name != "manifest.json" && name != "scenario.json" {
            return Err(DirectorError::PackageRefused("unlisted-bundle-entry"));
        }
        names.insert(name);
    }
    if names == BTreeSet::from(["manifest.json".to_owned(), "scenario.json".to_owned()]) {
        Ok(())
    } else {
        Err(DirectorError::PackageRefused("bundle-incomplete"))
    }
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, DirectorError> {
    let metadata = path
        .symlink_metadata()
        .map_err(|_| DirectorError::PackageRefused("bundle-unavailable"))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_DOCUMENT_BYTES
    {
        return Err(DirectorError::PackageRefused("unsafe-or-oversize-document"));
    }
    fs::read(path).map_err(|_| DirectorError::PackageRefused("bundle-unavailable"))
}

fn manifest_matches_scenario(
    manifest: &PackageManifest,
    scenario: &ScenarioPackage,
    scenario_size: usize,
) -> bool {
    manifest.manifest_version == "1.0.0"
        && manifest.canonicalisation == "RFC8785"
        && manifest.digest_algorithm == "sha-256"
        && manifest.package_id == ASSURANCE_PACKAGE_ID
        && manifest.package_version == ASSURANCE_PACKAGE_VERSION
        && manifest.package_id == scenario.package_id
        && manifest.package_version == scenario.package_version
        && manifest.scenario.path == "scenario.json"
        && manifest.scenario.media_type == "application/json"
        && manifest.scenario.schema_id == "urn:public-purpose-lab:contract:D-001:1.0.0"
        && manifest.scenario.size_bytes == scenario_size as u64
        && manifest.fixtures.is_empty()
        && scenario.contract_id == "D-001"
        && scenario.contract_version == "1.0.0"
        && scenario.information_profile == "synthetic-only"
}

fn refuse_prohibited_content(value: &Value) -> Result<(), DirectorError> {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                let normalised = key.replace('-', "").to_ascii_lowercase();
                if matches!(
                    normalised.as_str(),
                    "password"
                        | "secret"
                        | "token"
                        | "cookie"
                        | "credential"
                        | "privatekey"
                        | "apikey"
                        | "route"
                        | "url"
                        | "broker"
                        | "subject"
                        | "shell"
                        | "sql"
                        | "script"
                ) {
                    return Err(DirectorError::PackageRefused(
                        "hidden-route-or-secret-material",
                    ));
                }
                refuse_prohibited_content(item)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                refuse_prohibited_content(item)?;
            }
        }
        Value::String(text) if text.contains("http://") || text.contains("https://") => {
            return Err(DirectorError::PackageRefused(
                "hidden-route-or-secret-material",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn parse_strict_json(bytes: &[u8]) -> Result<Value, DirectorError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = NoDuplicates::deserialize(&mut deserializer)
        .map_err(|_| DirectorError::PackageRefused("json-invalid-or-duplicate-key"))?
        .0;
    deserializer
        .end()
        .map_err(|_| DirectorError::PackageRefused("json-trailing-content"))?;
    Ok(value)
}

struct NoDuplicates(Value);

impl<'de> Deserialize<'de> for NoDuplicates {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NoDuplicatesVisitor)
    }
}

struct NoDuplicatesVisitor;

impl<'de> de::Visitor<'de> for NoDuplicatesVisitor {
    type Value = NoDuplicates;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("I-JSON without duplicate object keys")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::Bool(value)))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::Number(value.into())))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::Number(value.into())))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(|number| NoDuplicates(Value::Number(number)))
            .ok_or_else(|| E::custom("non-I-JSON number"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::String(value.to_owned())))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::String(value)))
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::Null))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicates(Value::Null))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<NoDuplicates>()? {
            values.push(value.0);
        }
        Ok(NoDuplicates(Value::Array(values)))
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut map = Map::new();
        while let Some(key) = access.next_key::<String>()? {
            if map.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate key: {key}")));
            }
            let value = access.next_value::<NoDuplicates>()?;
            map.insert(key, value.0);
        }
        Ok(NoDuplicates(Value::Object(map)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime() -> (tempfile::TempDir, DirectorRuntime) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let runtime = DirectorRuntime::open(directory.path().join("ctl-01.sqlite"))
            .expect("director runtime");
        (directory, runtime)
    }

    fn lifecycle_command(
        operation_id: &str,
        session_id: &str,
        action: ScenarioLifecycleAction,
        expected_state: Option<ScenarioState>,
        expected_revision: u64,
    ) -> ScenarioLifecycleCommand {
        ScenarioLifecycleCommand {
            contract_id: "D-002".to_owned(),
            contract_version: "1.0.0".to_owned(),
            operation_id: operation_id.to_owned(),
            session_id: session_id.to_owned(),
            package_id: ASSURANCE_PACKAGE_ID.to_owned(),
            package_version: ASSURANCE_PACKAGE_VERSION.to_owned(),
            action,
            expected_state,
            expected_revision,
            requested_at: "2026-08-27T12:00:00Z".to_owned(),
            reason: None,
        }
    }

    fn admit_test_package(runtime: &DirectorRuntime, now: OffsetDateTime) {
        let package_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../scenarios/presentation-control-assurance");
        runtime
            .admit_bundled_package(&package_directory, "test-source", "test-image", now)
            .expect("package admission");
    }

    fn create_running_session_from_revision_two(
        runtime: &DirectorRuntime,
        session_id: &str,
        now: OffsetDateTime,
    ) {
        for command in [
            lifecycle_command(
                "operation:prepare:reset",
                session_id,
                ScenarioLifecycleAction::Prepare,
                Some(ScenarioState::Preparing),
                2,
            ),
            lifecycle_command(
                "operation:start:reset",
                session_id,
                ScenarioLifecycleAction::Start,
                Some(ScenarioState::Ready),
                3,
            ),
        ] {
            runtime
                .apply_lifecycle(&command, now)
                .expect("lifecycle transition");
        }
    }

    fn record_presentation_checkpoint(
        runtime: &DirectorRuntime,
        session_id: &str,
        now: OffsetDateTime,
    ) {
        let mut registration: PresentationRegistration = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-002-audience-registration.json"
        ))
        .expect("registration");
        registration.session_id = session_id.to_owned();
        runtime
            .observe_registration(&registration, now)
            .expect("observe registration");
        let mut cue: ppl_contracts::PresentationCue = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-003-welcome-cue.json"
        ))
        .expect("cue");
        cue.session_id = session_id.to_owned();
        cue.session_revision = 4;
        runtime.issue_cue(&cue, now).expect("issue cue");
        let mut outcome: PresentationCueOutcome = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-004-welcome-applied.json"
        ))
        .expect("outcome");
        outcome.session_id = session_id.to_owned();
        outcome.session_revision = 4;
        runtime
            .observe_presentation_outcome(&outcome)
            .expect("presentation checkpoint");
    }

    #[test]
    fn duplicate_json_keys_are_refused() {
        assert!(parse_strict_json(br#"{"a":1,"a":2}"#).is_err());
    }

    #[test]
    fn package_digest_matches_the_repository_checker() {
        let (_directory, runtime) = runtime();
        let package_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../scenarios/presentation-control-assurance");
        let admission = runtime
            .admit_bundled_package(
                &package_directory,
                "test-source",
                "test-image",
                OffsetDateTime::UNIX_EPOCH,
            )
            .expect("package admission");
        assert_eq!(
            admission.package_digest,
            "15b78a9bfb7290eeb256e91ea61d24f404fe375e5adb2e0316023fe20b4547d4"
        );
    }

    #[test]
    fn lifecycle_is_revisioned_idempotent_and_restart_safe() {
        let (directory, runtime) = runtime();
        let package_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../scenarios/presentation-control-assurance");
        runtime
            .admit_bundled_package(
                &package_directory,
                "test-source",
                "test-image",
                OffsetDateTime::UNIX_EPOCH,
            )
            .expect("package admission");
        let command = ScenarioLifecycleCommand {
            contract_id: "D-002".to_owned(),
            contract_version: "1.0.0".to_owned(),
            operation_id: "op:create:test".to_owned(),
            session_id: "session:test:001".to_owned(),
            package_id: ASSURANCE_PACKAGE_ID.to_owned(),
            package_version: ASSURANCE_PACKAGE_VERSION.to_owned(),
            action: ScenarioLifecycleAction::Create,
            expected_state: None,
            expected_revision: 0,
            requested_at: "2026-08-27T12:00:00Z".to_owned(),
            reason: None,
        };
        let first = runtime
            .apply_lifecycle(&command, OffsetDateTime::UNIX_EPOCH)
            .expect("create session");
        let duplicate = runtime
            .apply_lifecycle(&command, OffsetDateTime::UNIX_EPOCH)
            .expect("duplicate session command");
        assert_eq!(first, duplicate);
        let reopened =
            DirectorRuntime::open(directory.path().join("ctl-01.sqlite")).expect("reopen director");
        assert_eq!(
            reopened
                .session("session:test:001")
                .expect("session")
                .revision,
            1
        );
        let mut changed = command;
        changed.reason = Some("changed".to_owned());
        assert!(matches!(
            reopened.apply_lifecycle(&changed, OffsetDateTime::UNIX_EPOCH),
            Err(DirectorError::OperationRefused(
                "duplicate-content-conflict"
            ))
        ));
    }

    #[test]
    fn brief_outbox_writer_overlap_waits_before_director_mutation() {
        let (_directory, runtime) = runtime();
        let now = OffsetDateTime::parse("2026-08-27T12:00:00Z", &Rfc3339).expect("time");
        admit_test_package(&runtime, now);
        let session_id = "session:writer-overlap:001";
        runtime
            .apply_lifecycle(
                &lifecycle_command(
                    "operation:create:writer-overlap",
                    session_id,
                    ScenarioLifecycleAction::Create,
                    None,
                    0,
                ),
                now,
            )
            .expect("create");

        let blocker = Connection::open(&runtime.database_path).expect("outbox connection");
        blocker
            .execute_batch(
                "BEGIN IMMEDIATE;
                 UPDATE outbox SET published_at='2026-08-27T12:00:01Z'
                 WHERE sequence=(SELECT MIN(sequence) FROM outbox);",
            )
            .expect("hold outbox writer");

        let worker = runtime.clone();
        let (started_sender, started_receiver) = std::sync::mpsc::channel();
        let (result_sender, result_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            started_sender.send(()).expect("signal worker");
            let result = worker.set_initial_logical_time(
                "operation:set:writer-overlap",
                session_id,
                1,
                "2030-01-01T09:00:00Z",
                now,
            );
            result_sender.send(result).expect("return worker result");
        });
        started_receiver.recv().expect("worker started");
        assert!(matches!(
            result_receiver.recv_timeout(std::time::Duration::from_millis(200)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));

        blocker.execute_batch("COMMIT;").expect("release writer");
        let outcome = result_receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("worker completed")
            .expect("logical time accepted after writer overlap");
        assert_eq!(outcome.session_revision, 2);
    }

    #[test]
    fn logical_time_and_reset_preserve_history_without_cross_session_progress() {
        let (_directory, runtime) = runtime();
        let now = OffsetDateTime::parse("2026-08-27T12:00:00Z", &Rfc3339).expect("time");
        admit_test_package(&runtime, now);
        let session_id = "session:reset:001";
        runtime
            .apply_lifecycle(
                &lifecycle_command(
                    "operation:create:reset",
                    session_id,
                    ScenarioLifecycleAction::Create,
                    None,
                    0,
                ),
                now,
            )
            .expect("create");
        assert!(matches!(
            runtime.advance_logical_time("operation:early", session_id, 1, 60, now),
            Err(DirectorError::OperationRefused(
                "logical-time-not-initialised"
            ))
        ));
        let set = runtime
            .set_initial_logical_time(
                "operation:set:001",
                session_id,
                1,
                "2030-01-01T09:00:00Z",
                now,
            )
            .expect("set logical time");
        assert_eq!(set.session_revision, 2);
        assert_eq!(
            runtime
                .set_initial_logical_time(
                    "operation:set:001",
                    session_id,
                    1,
                    "2030-01-01T09:00:00Z",
                    now,
                )
                .expect("idempotent set"),
            set
        );
        assert!(matches!(
            runtime.advance_logical_time("operation:too-far", session_id, 2, 86_401, now),
            Err(DirectorError::OperationRefused(
                "logical-time-bound-exceeded"
            ))
        ));
        create_running_session_from_revision_two(&runtime, session_id, now);
        record_presentation_checkpoint(&runtime, session_id, now);

        runtime
            .apply_lifecycle(
                &lifecycle_command(
                    "operation:stop:reset",
                    session_id,
                    ScenarioLifecycleAction::Stop,
                    Some(ScenarioState::Running),
                    4,
                ),
                now,
            )
            .expect("stop");
        let reset = runtime
            .apply_lifecycle(
                &lifecycle_command(
                    "operation:reset:director",
                    session_id,
                    ScenarioLifecycleAction::Reset,
                    Some(ScenarioState::Stopped),
                    5,
                ),
                now,
            )
            .expect("reset");
        let successor = reset.successor.expect("successor");
        assert_ne!(successor.session_id, session_id);
        assert!(!successor.logical_time_initialised);
        assert!(
            runtime
                .presentation_checkpoint(&successor.session_id)
                .expect("successor checkpoint")
                .is_none()
        );
        assert!(
            runtime
                .presentation_checkpoint(session_id)
                .expect("prior checkpoint retained")
                .is_some()
        );
        assert_eq!(
            runtime.session(session_id).expect("prior session").state,
            ScenarioState::Superseded
        );
    }
}
