//! CTL-02 Presentation Gateway and Screen Registry with component-owned state.

use std::{fs, path::PathBuf};

use ppl_contracts::{
    CommandOutcome, OutcomeStatus, PresentationCapabilityManifest, PresentationCue,
    PresentationCueOutcome, PresentationRegistration, ScenarioControlCommand,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PresentationError {
    #[error("state-unavailable")]
    StateUnavailable,
    #[error("state-inconsistent")]
    StateInconsistent,
    #[error("operation-refused:{0}")]
    OperationRefused(&'static str),
}

#[derive(Clone, Debug)]
pub struct PresentationRuntime {
    database_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RegistrationOutcome {
    pub status: String,
    pub code: String,
    pub registration: PresentationRegistration,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CueDelivery {
    pub status: String,
    pub code: String,
    pub cue: PresentationCue,
    pub delay_milliseconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OutboxRecord {
    pub sequence: i64,
    pub event_type: String,
    pub payload: Value,
    pub created_at: String,
}

impl PresentationRuntime {
    /// Opens or creates CTL-02 state and verifies basic database integrity.
    ///
    /// # Errors
    /// Returns a safe state error if storage cannot be migrated or checked.
    pub fn open(database_path: impl Into<PathBuf>) -> Result<Self, PresentationError> {
        let runtime = Self {
            database_path: database_path.into(),
        };
        runtime.migrate()?;
        Ok(runtime)
    }

    fn connection(&self) -> Result<Connection, PresentationError> {
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent).map_err(|_| PresentationError::StateUnavailable)?;
        }
        let connection = Connection::open(&self.database_path)
            .map_err(|_| PresentationError::StateUnavailable)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|_| PresentationError::StateUnavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| PresentationError::StateUnavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(connection)
    }

    fn migrate(&self) -> Result<(), PresentationError> {
        let connection = self.connection()?;
        connection
            .execute_batch(
                "BEGIN IMMEDIATE;
                 CREATE TABLE IF NOT EXISTS manifests (
                   manifest_id TEXT NOT NULL, manifest_version TEXT NOT NULL,
                   digest TEXT NOT NULL, manifest_json TEXT NOT NULL,
                   admitted_at TEXT NOT NULL,
                   PRIMARY KEY(manifest_id,manifest_version));
                 CREATE TABLE IF NOT EXISTS registrations (
                   registration_id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
                   surface_slot TEXT NOT NULL, registration_revision INTEGER NOT NULL,
                   connection_generation INTEGER NOT NULL, state TEXT NOT NULL,
                   registration_json TEXT NOT NULL, updated_at TEXT NOT NULL);
                 CREATE UNIQUE INDEX IF NOT EXISTS registrations_one_active_slot
                   ON registrations(session_id,surface_slot) WHERE state='active';
                 CREATE TABLE IF NOT EXISTS cues (
                   cue_id TEXT PRIMARY KEY, fingerprint TEXT NOT NULL,
                   session_id TEXT NOT NULL, surface_slot TEXT NOT NULL,
                   registration_id TEXT NOT NULL, connection_generation INTEGER NOT NULL,
                   cue_json TEXT NOT NULL, state TEXT NOT NULL,
                   received_at TEXT NOT NULL, outcome_json TEXT);
                 CREATE TABLE IF NOT EXISTS inbox (
                   operation_id TEXT PRIMARY KEY, fingerprint TEXT NOT NULL,
                   outcome_json TEXT NOT NULL, recorded_at TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS outbox (
                   sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                   event_type TEXT NOT NULL, payload_json TEXT NOT NULL,
                   created_at TEXT NOT NULL, published_at TEXT);
                 CREATE TABLE IF NOT EXISTS active_faults (
                   session_id TEXT PRIMARY KEY, fault_name TEXT NOT NULL,
                   delay_milliseconds INTEGER NOT NULL, remaining_uses INTEGER NOT NULL,
                   activated_at TEXT NOT NULL, expires_at TEXT NOT NULL);
                 PRAGMA user_version=1;
                 COMMIT;",
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
        let check: String = connection
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(|_| PresentationError::StateUnavailable)?;
        if check == "ok" {
            Ok(())
        } else {
            Err(PresentationError::StateInconsistent)
        }
    }

    /// Admits a closed P-001 manifest for the bundled frontend release.
    ///
    /// # Errors
    /// Refuses unsupported contracts, profiles, roles or digest conflicts.
    pub fn admit_manifest(
        &self,
        manifest: &PresentationCapabilityManifest,
        now: OffsetDateTime,
    ) -> Result<String, PresentationError> {
        if manifest.contract_id != "P-001"
            || manifest.contract_version != "1.0.0"
            || manifest.information_profiles != ["synthetic-only"]
            || manifest.views.is_empty()
            || manifest.surface_roles.is_empty()
        {
            return Err(PresentationError::OperationRefused("manifest-unaccepted"));
        }
        let digest = canonical_digest(manifest)?;
        let json =
            serde_json::to_string(manifest).map_err(|_| PresentationError::StateInconsistent)?;
        let connection = self.connection()?;
        let existing: Option<String> = connection
            .query_row(
                "SELECT digest FROM manifests WHERE manifest_id=?1 AND manifest_version=?2",
                params![manifest.manifest_id, manifest.manifest_version],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| PresentationError::StateUnavailable)?;
        if let Some(existing) = existing {
            if existing == digest {
                return Ok(digest);
            }
            return Err(PresentationError::OperationRefused(
                "manifest-digest-conflict",
            ));
        }
        connection
            .execute(
                "INSERT INTO manifests VALUES(?1,?2,?3,?4,?5)",
                params![
                    manifest.manifest_id,
                    manifest.manifest_version,
                    digest,
                    json,
                    format_time(now)?
                ],
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(digest)
    }

    /// Registers one active surface for one session slot.
    ///
    /// # Errors
    /// Refuses stale leases, unknown manifests and second active slot bindings.
    pub fn register(
        &self,
        registration: &PresentationRegistration,
        now: OffsetDateTime,
    ) -> Result<RegistrationOutcome, PresentationError> {
        validate_registration(registration, now)?;
        let fingerprint = canonical_digest(registration)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| PresentationError::StateUnavailable)?;
        if let Some(outcome) =
            duplicate_registration(&transaction, &registration.registration_id, &fingerprint)?
        {
            return Ok(outcome);
        }
        let manifest = read_manifest(
            &transaction,
            &registration.manifest_id,
            &registration.manifest_version,
        )?;
        if !manifest.surface_roles.contains(&registration.surface_role)
            || registration
                .supported_views
                .iter()
                .any(|view| !manifest.views.iter().any(|item| item.view_id == *view))
        {
            return Err(PresentationError::OperationRefused(
                "capability-incompatible",
            ));
        }
        let json = serde_json::to_string(registration)
            .map_err(|_| PresentationError::StateInconsistent)?;
        transaction
            .execute(
                "INSERT INTO registrations VALUES(?1,?2,?3,?4,?5,'active',?6,?7)",
                params![
                    registration.registration_id,
                    registration.session_id,
                    registration.surface_slot,
                    sql_int(registration.registration_revision)?,
                    sql_int(registration.connection_generation)?,
                    json,
                    format_time(now)?
                ],
            )
            .map_err(|error| {
                if error.to_string().contains("UNIQUE") {
                    PresentationError::OperationRefused("slot-already-bound")
                } else {
                    PresentationError::StateUnavailable
                }
            })?;
        let outcome = RegistrationOutcome {
            status: "accepted".to_owned(),
            code: "surface-registered".to_owned(),
            registration: registration.clone(),
        };
        record_operation_event(
            &transaction,
            &registration.registration_id,
            &fingerprint,
            "ppl.presentation.surface.registered",
            &outcome,
            &format_time(now)?,
        )?;
        transaction
            .commit()
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(outcome)
    }

    /// Validates and records one P-003 cue for backend-to-browser delivery.
    ///
    /// # Errors
    /// Refuses expiry, stale bindings, unsupported views and changed duplicates.
    pub fn accept_cue(
        &self,
        cue: &PresentationCue,
        now: OffsetDateTime,
    ) -> Result<CueDelivery, PresentationError> {
        validate_cue_shape(cue)?;
        let expires = OffsetDateTime::parse(&cue.expires_at, &Rfc3339)
            .map_err(|_| PresentationError::OperationRefused("expiry-invalid"))?;
        if expires <= now {
            return Err(PresentationError::OperationRefused("expired"));
        }
        let fingerprint = canonical_digest(cue)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| PresentationError::StateUnavailable)?;
        if let Some((existing_fingerprint, json)) = transaction
            .query_row(
                "SELECT fingerprint,cue_json FROM cues WHERE cue_id=?1",
                [&cue.cue_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| PresentationError::StateUnavailable)?
        {
            if existing_fingerprint != fingerprint {
                return Err(PresentationError::OperationRefused(
                    "duplicate-content-conflict",
                ));
            }
            let cue: PresentationCue =
                serde_json::from_str(&json).map_err(|_| PresentationError::StateInconsistent)?;
            return Ok(CueDelivery {
                status: "duplicate".to_owned(),
                code: "cue-already-recorded".to_owned(),
                cue,
                delay_milliseconds: 0,
            });
        }
        let registration = current_registration(&transaction, &cue.session_id, &cue.surface_slot)?;
        if registration.registration_id != cue.registration_id
            || registration.registration_revision != cue.registration_revision
            || registration.connection_generation != cue.connection_generation
        {
            return Err(PresentationError::OperationRefused(
                "registration-generation-stale",
            ));
        }
        if !registration.supported_views.contains(&cue.semantic_view) {
            return Err(PresentationError::OperationRefused("view-unsupported"));
        }
        let delay = consume_delay_fault(&transaction, &cue.session_id, now)?;
        let json = serde_json::to_string(cue).map_err(|_| PresentationError::StateInconsistent)?;
        transaction
            .execute(
                "INSERT INTO cues VALUES(?1,?2,?3,?4,?5,?6,?7,'accepted',?8,NULL)",
                params![
                    cue.cue_id,
                    fingerprint,
                    cue.session_id,
                    cue.surface_slot,
                    cue.registration_id,
                    sql_int(cue.connection_generation)?,
                    json,
                    format_time(now)?
                ],
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
        transaction
            .commit()
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(CueDelivery {
            status: "accepted".to_owned(),
            code: "cue-ready-for-delivery".to_owned(),
            cue: cue.clone(),
            delay_milliseconds: delay,
        })
    }

    /// Validates and records a P-004 outcome for the current cue generation.
    ///
    /// # Errors
    /// Refuses stale, conflicting or business-claiming outcomes.
    pub fn record_outcome(
        &self,
        outcome: &PresentationCueOutcome,
        now: OffsetDateTime,
    ) -> Result<PresentationCueOutcome, PresentationError> {
        if outcome.contract_id != "P-004"
            || outcome.contract_version != "1.0.0"
            || outcome.business_completion_claimed
        {
            return Err(PresentationError::OperationRefused("outcome-invalid"));
        }
        let fingerprint = canonical_digest(outcome)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| PresentationError::StateUnavailable)?;
        if let Some((stored_fingerprint, stored_json)) = transaction
            .query_row(
                "SELECT fingerprint,outcome_json FROM inbox WHERE operation_id=?1",
                [&outcome.outcome_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| PresentationError::StateUnavailable)?
        {
            if stored_fingerprint != fingerprint {
                return Err(PresentationError::OperationRefused(
                    "duplicate-content-conflict",
                ));
            }
            return serde_json::from_str(&stored_json)
                .map_err(|_| PresentationError::StateInconsistent);
        }
        let cue = read_cue(&transaction, &outcome.cue_id)?;
        if cue.cue_digest != outcome.cue_digest
            || cue.session_id != outcome.session_id
            || cue.session_revision != outcome.session_revision
            || cue.surface_slot != outcome.surface_slot
            || cue.registration_id != outcome.registration_id
            || cue.registration_revision != outcome.registration_revision
            || cue.connection_generation != outcome.connection_generation
            || cue.semantic_view != outcome.semantic_view
        {
            return Err(PresentationError::OperationRefused(
                "outcome-binding-conflict",
            ));
        }
        let registration =
            current_registration(&transaction, &outcome.session_id, &outcome.surface_slot)?;
        if registration.registration_id != outcome.registration_id
            || registration.registration_revision != outcome.registration_revision
            || registration.connection_generation != outcome.connection_generation
        {
            return Err(PresentationError::OperationRefused("generation-superseded"));
        }
        let json =
            serde_json::to_string(outcome).map_err(|_| PresentationError::StateInconsistent)?;
        transaction
            .execute(
                "UPDATE cues SET state='concluded',outcome_json=?1 WHERE cue_id=?2 AND outcome_json IS NULL",
                params![json, outcome.cue_id],
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
        record_operation_event(
            &transaction,
            &outcome.outcome_id,
            &fingerprint,
            "ppl.presentation.cue.concluded",
            outcome,
            &format_time(now)?,
        )?;
        transaction
            .commit()
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(outcome.clone())
    }

    /// Activates the one-shot bounded presentation delay fault.
    ///
    /// # Errors
    /// Refuses values outside 50 to 3000 milliseconds.
    pub fn activate_cue_delay(
        &self,
        session_id: &str,
        delay_milliseconds: u64,
        now: OffsetDateTime,
    ) -> Result<(), PresentationError> {
        if !(50..=3_000).contains(&delay_milliseconds) {
            return Err(PresentationError::OperationRefused(
                "fault-parameter-out-of-range",
            ));
        }
        self.connection()?
            .execute(
                "INSERT INTO active_faults VALUES(?1,'presentation-cue-delay',?2,1,?3,?4)
                 ON CONFLICT(session_id) DO UPDATE SET fault_name=excluded.fault_name,
                   delay_milliseconds=excluded.delay_milliseconds,remaining_uses=1,
                   activated_at=excluded.activated_at,expires_at=excluded.expires_at",
                params![
                    session_id,
                    sql_int(delay_milliseconds)?,
                    format_time(now)?,
                    format_time(now + Duration::seconds(30))?
                ],
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(())
    }

    /// Applies an allow-listed D-003 Presentation control idempotently.
    ///
    /// # Errors
    /// Returns a state error if the conclusive result cannot commit with its
    /// inbox and outbox records.
    pub fn apply_control(
        &self,
        command: &ScenarioControlCommand,
        now: OffsetDateTime,
    ) -> Result<CommandOutcome, PresentationError> {
        if command.contract_id != "D-003" || command.contract_version != "1.0.0" {
            return Err(PresentationError::OperationRefused("contract-unsupported"));
        }
        let fingerprint = canonical_digest(command)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| PresentationError::StateUnavailable)?;
        if let Some((stored_fingerprint, stored_json)) = transaction
            .query_row(
                "SELECT fingerprint,outcome_json FROM inbox WHERE operation_id=?1",
                [&command.operation_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| PresentationError::StateUnavailable)?
        {
            if stored_fingerprint != fingerprint {
                return Err(PresentationError::OperationRefused(
                    "duplicate-content-conflict",
                ));
            }
            return serde_json::from_str(&stored_json)
                .map_err(|_| PresentationError::StateInconsistent);
        }

        let (status, code, summary) = apply_control_effect(&transaction, command, now)?;
        let outcome = CommandOutcome {
            contract_id: "C-003".to_owned(),
            contract_version: "1.0.0".to_owned(),
            outcome_id: format!("outcome:{}", Uuid::new_v4()),
            command_message_id: command.operation_id.clone(),
            status,
            code: code.to_owned(),
            summary: summary.to_owned(),
            retryable: false,
            completed_at: format_time(now)?,
            original_outcome_id: None,
            recovery_owner: None,
            evidence: Vec::new(),
        };
        record_operation_event(
            &transaction,
            &command.operation_id,
            &fingerprint,
            "ppl.presentation.control.concluded",
            &outcome,
            &format_time(now)?,
        )?;
        transaction
            .commit()
            .map_err(|_| PresentationError::StateUnavailable)?;
        Ok(outcome)
    }

    /// Semantically resets disposable CTL-02 state for one session.
    ///
    /// Historical cues, outcomes and inbox evidence are retained.
    ///
    /// # Errors
    /// Returns a safe error when reset cannot commit atomically.
    pub fn reset_session(
        &self,
        session_id: &str,
        now: OffsetDateTime,
    ) -> Result<(), PresentationError> {
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| PresentationError::StateUnavailable)?;
        reset_in_transaction(&transaction, session_id, &format_time(now)?)?;
        transaction
            .commit()
            .map_err(|_| PresentationError::StateUnavailable)
    }

    /// Reads one current registration.
    ///
    /// # Errors
    /// Returns a safe refusal when no active binding exists.
    pub fn current_registration(
        &self,
        session_id: &str,
        surface_slot: &str,
    ) -> Result<PresentationRegistration, PresentationError> {
        current_registration(&self.connection()?, session_id, surface_slot)
    }

    /// Reads unpublished component events in sequence order.
    ///
    /// # Errors
    /// Returns a state error when the outbox is unavailable.
    pub fn pending_outbox(&self, limit: usize) -> Result<Vec<OutboxRecord>, PresentationError> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT sequence,event_type,payload_json,created_at FROM outbox
                 WHERE published_at IS NULL ORDER BY sequence LIMIT ?1",
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
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
            .map_err(|_| PresentationError::StateUnavailable)?;
        rows.map(|row| {
            let (sequence, event_type, payload, created_at) =
                row.map_err(|_| PresentationError::StateUnavailable)?;
            Ok(OutboxRecord {
                sequence,
                event_type,
                payload: serde_json::from_str(&payload)
                    .map_err(|_| PresentationError::StateInconsistent)?,
                created_at,
            })
        })
        .collect()
    }

    /// Marks an outbox record published after broker acknowledgement.
    ///
    /// # Errors
    /// Returns a safe error when the marker cannot be written.
    pub fn mark_outbox_published(
        &self,
        sequence: i64,
        now: OffsetDateTime,
    ) -> Result<(), PresentationError> {
        let changed = self
            .connection()?
            .execute(
                "UPDATE outbox SET published_at=?1 WHERE sequence=?2 AND published_at IS NULL",
                params![format_time(now)?, sequence],
            )
            .map_err(|_| PresentationError::StateUnavailable)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(PresentationError::OperationRefused(
                "outbox-record-unavailable",
            ))
        }
    }
}

fn apply_control_effect(
    transaction: &Transaction<'_>,
    command: &ScenarioControlCommand,
    now: OffsetDateTime,
) -> Result<(OutcomeStatus, &'static str, &'static str), PresentationError> {
    match (
        command.kind.as_str(),
        command.operation.as_str(),
        command.target.as_deref(),
    ) {
        ("reset", "execute", Some("presentation-registry-baseline")) => {
            reset_in_transaction(transaction, &command.session_id, &format_time(now)?)?;
            Ok((
                OutcomeStatus::Accepted,
                "presentation-registry-reset",
                "Disposable presentation state was superseded and evidence retained.",
            ))
        }
        ("fault", "activate", Some("presentation-cue-delay")) => {
            activate_controlled_cue_delay(transaction, command, now)?;
            Ok((
                OutcomeStatus::Accepted,
                "presentation-cue-delay-armed",
                "One bounded presentation cue delay was armed.",
            ))
        }
        ("fault", "clear", Some("presentation-cue-delay")) => {
            transaction
                .execute(
                    "DELETE FROM active_faults WHERE session_id=?1
                     AND fault_name='presentation-cue-delay'",
                    [&command.session_id],
                )
                .map_err(|_| PresentationError::StateUnavailable)?;
            Ok((
                OutcomeStatus::Accepted,
                "presentation-cue-delay-cleared",
                "The session-scoped presentation cue delay was cleared.",
            ))
        }
        _ => Ok((
            OutcomeStatus::Refused,
            "control-unsupported",
            "The requested control is not an allow-listed CTL-02 capability.",
        )),
    }
}

fn activate_controlled_cue_delay(
    transaction: &Transaction<'_>,
    command: &ScenarioControlCommand,
    now: OffsetDateTime,
) -> Result<(), PresentationError> {
    let delay = command
        .delay_milliseconds
        .filter(|value| (50..=3_000).contains(value))
        .ok_or(PresentationError::OperationRefused(
            "fault-parameter-out-of-range",
        ))?;
    let active_registration: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM registrations
             WHERE session_id=?1 AND state='active')",
            [&command.session_id],
            |row| row.get(0),
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    if !active_registration {
        return Err(PresentationError::OperationRefused(
            "fault-session-unregistered",
        ));
    }
    transaction
        .execute(
            "INSERT INTO active_faults VALUES(?1,'presentation-cue-delay',?2,1,?3,?4)
             ON CONFLICT(session_id) DO UPDATE SET
               delay_milliseconds=excluded.delay_milliseconds,remaining_uses=1,
               activated_at=excluded.activated_at,expires_at=excluded.expires_at",
            params![
                command.session_id,
                sql_int(delay)?,
                format_time(now)?,
                format_time(now + Duration::seconds(30))?
            ],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    Ok(())
}

fn validate_registration(
    registration: &PresentationRegistration,
    now: OffsetDateTime,
) -> Result<(), PresentationError> {
    let expires = OffsetDateTime::parse(&registration.lease_expires_at, &Rfc3339)
        .map_err(|_| PresentationError::OperationRefused("lease-invalid"))?;
    if registration.contract_id != "P-002"
        || registration.contract_version != "1.0.0"
        || registration.binding_mode != "development-assurance"
        || registration.registration_revision == 0
        || registration.connection_generation == 0
        || expires <= now
    {
        return Err(PresentationError::OperationRefused("registration-invalid"));
    }
    Ok(())
}

fn reset_in_transaction(
    transaction: &Transaction<'_>,
    session_id: &str,
    now: &str,
) -> Result<(), PresentationError> {
    transaction
        .execute(
            "UPDATE registrations SET state='superseded',updated_at=?1
             WHERE session_id=?2 AND state='active'",
            params![now, session_id],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    transaction
        .execute(
            "UPDATE cues SET state='superseded' WHERE session_id=?1 AND state!='concluded'",
            [session_id],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    transaction
        .execute(
            "DELETE FROM active_faults WHERE session_id=?1",
            [session_id],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    Ok(())
}

fn validate_cue_shape(cue: &PresentationCue) -> Result<(), PresentationError> {
    if cue.contract_id != "P-003"
        || cue.contract_version != "1.0.0"
        || cue.context.keys().any(|key| {
            matches!(
                key.as_str(),
                "route" | "url" | "token" | "cookie" | "credential" | "subject"
            )
        })
        || cue.context.values().any(|value| {
            value.contains("http://") || value.contains("https://") || value.starts_with('/')
        })
    {
        return Err(PresentationError::OperationRefused("prohibited-content"));
    }
    Ok(())
}

fn duplicate_registration(
    transaction: &Transaction<'_>,
    operation_id: &str,
    fingerprint: &str,
) -> Result<Option<RegistrationOutcome>, PresentationError> {
    let existing: Option<(String, String)> = transaction
        .query_row(
            "SELECT fingerprint,outcome_json FROM inbox WHERE operation_id=?1",
            [operation_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| PresentationError::StateUnavailable)?;
    let Some((existing_fingerprint, outcome)) = existing else {
        return Ok(None);
    };
    if existing_fingerprint != fingerprint {
        return Err(PresentationError::OperationRefused(
            "duplicate-content-conflict",
        ));
    }
    serde_json::from_str(&outcome)
        .map(Some)
        .map_err(|_| PresentationError::StateInconsistent)
}

fn read_manifest(
    connection: &Connection,
    manifest_id: &str,
    manifest_version: &str,
) -> Result<PresentationCapabilityManifest, PresentationError> {
    let json: String = connection
        .query_row(
            "SELECT manifest_json FROM manifests WHERE manifest_id=?1 AND manifest_version=?2",
            params![manifest_id, manifest_version],
            |row| row.get(0),
        )
        .map_err(|_| PresentationError::OperationRefused("manifest-unaccepted"))?;
    serde_json::from_str(&json).map_err(|_| PresentationError::StateInconsistent)
}

fn current_registration(
    connection: &Connection,
    session_id: &str,
    surface_slot: &str,
) -> Result<PresentationRegistration, PresentationError> {
    let json: String = connection
        .query_row(
            "SELECT registration_json FROM registrations WHERE session_id=?1
             AND surface_slot=?2 AND state='active'",
            params![session_id, surface_slot],
            |row| row.get(0),
        )
        .map_err(|_| PresentationError::OperationRefused("surface-unregistered"))?;
    serde_json::from_str(&json).map_err(|_| PresentationError::StateInconsistent)
}

fn read_cue(connection: &Connection, cue_id: &str) -> Result<PresentationCue, PresentationError> {
    let json: String = connection
        .query_row(
            "SELECT cue_json FROM cues WHERE cue_id=?1",
            [cue_id],
            |row| row.get(0),
        )
        .map_err(|_| PresentationError::OperationRefused("cue-unavailable"))?;
    serde_json::from_str(&json).map_err(|_| PresentationError::StateInconsistent)
}

fn consume_delay_fault(
    transaction: &Transaction<'_>,
    session_id: &str,
    now: OffsetDateTime,
) -> Result<u64, PresentationError> {
    let fault: Option<(i64, String)> = transaction
        .query_row(
            "SELECT delay_milliseconds,expires_at FROM active_faults WHERE session_id=?1
             AND fault_name='presentation-cue-delay' AND remaining_uses=1",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| PresentationError::StateUnavailable)?;
    let Some((delay, expires_at)) = fault else {
        return Ok(0);
    };
    transaction
        .execute(
            "DELETE FROM active_faults WHERE session_id=?1",
            [session_id],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    let expiry = OffsetDateTime::parse(&expires_at, &Rfc3339)
        .map_err(|_| PresentationError::StateInconsistent)?;
    if expiry <= now {
        return Ok(0);
    }
    u64::try_from(delay).map_err(|_| PresentationError::StateInconsistent)
}

fn record_operation_event<T: Serialize>(
    transaction: &Transaction<'_>,
    operation_id: &str,
    fingerprint: &str,
    event_type: &str,
    outcome: &T,
    now: &str,
) -> Result<(), PresentationError> {
    let json = serde_json::to_string(outcome).map_err(|_| PresentationError::StateInconsistent)?;
    transaction
        .execute(
            "INSERT INTO inbox VALUES(?1,?2,?3,?4)",
            params![operation_id, fingerprint, json, now],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    transaction
        .execute(
            "INSERT INTO outbox(event_type,payload_json,created_at) VALUES(?1,?2,?3)",
            params![event_type, json, now],
        )
        .map_err(|_| PresentationError::StateUnavailable)?;
    Ok(())
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, PresentationError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| PresentationError::StateInconsistent)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn format_time(value: OffsetDateTime) -> Result<String, PresentationError> {
    value
        .format(&Rfc3339)
        .map_err(|_| PresentationError::StateInconsistent)
}

fn sql_int(value: u64) -> Result<i64, PresentationError> {
    i64::try_from(value).map_err(|_| PresentationError::OperationRefused("value-out-of-range"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ppl_contracts::PresentationOutcomeResult;

    fn manifest() -> PresentationCapabilityManifest {
        serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-001-assurance-surface.json"
        ))
        .expect("manifest")
    }

    fn registration() -> PresentationRegistration {
        serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-002-audience-registration.json"
        ))
        .expect("registration")
    }

    fn runtime() -> (tempfile::TempDir, PresentationRuntime) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let runtime = PresentationRuntime::open(directory.path().join("ctl-02.sqlite"))
            .expect("presentation runtime");
        (directory, runtime)
    }

    #[test]
    fn one_active_registration_per_session_slot() {
        let (_directory, runtime) = runtime();
        let now = OffsetDateTime::parse("2026-08-27T12:00:00Z", &Rfc3339).expect("time");
        runtime.admit_manifest(&manifest(), now).expect("manifest");
        let registration = registration();
        runtime
            .register(&registration, now)
            .expect("first registration");
        let duplicate = runtime
            .register(&registration, now)
            .expect("identical duplicate");
        assert_eq!(duplicate.code, "surface-registered");
        let mut second = registration;
        second.registration_id = "registration:audience:002".to_owned();
        assert!(matches!(
            runtime.register(&second, now),
            Err(PresentationError::OperationRefused("slot-already-bound"))
        ));
    }

    #[test]
    fn cue_delay_is_bounded_and_one_shot() {
        let (_directory, runtime) = runtime();
        let now = OffsetDateTime::parse("2026-08-27T12:00:00Z", &Rfc3339).expect("time");
        runtime.admit_manifest(&manifest(), now).expect("manifest");
        runtime
            .register(&registration(), now)
            .expect("registration");
        runtime
            .activate_cue_delay("session:assurance:001", 200, now)
            .expect("fault");
        let cue: PresentationCue = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-003-welcome-cue.json"
        ))
        .expect("cue");
        assert_eq!(
            runtime
                .accept_cue(&cue, now)
                .expect("cue")
                .delay_milliseconds,
            200
        );
        assert_eq!(
            runtime.accept_cue(&cue, now).expect("duplicate cue").status,
            "duplicate"
        );
        let mut second = cue;
        second.cue_id = "cue:welcome:002".to_owned();
        second.idempotency_key = "idem:cue:welcome:002".to_owned();
        assert_eq!(
            runtime
                .accept_cue(&second, now)
                .expect("cue")
                .delay_milliseconds,
            0
        );
    }

    #[test]
    fn control_fault_expires_and_reset_supersedes_only_disposable_state() {
        let (_directory, runtime) = runtime();
        let now = OffsetDateTime::parse("2026-08-27T12:00:00Z", &Rfc3339).expect("time");
        runtime.admit_manifest(&manifest(), now).expect("manifest");
        runtime
            .register(&registration(), now)
            .expect("registration");
        let fault = ScenarioControlCommand {
            contract_id: "D-003".to_owned(),
            contract_version: "1.0.0".to_owned(),
            operation_id: "operation:fault:001".to_owned(),
            session_id: "session:assurance:001".to_owned(),
            kind: "fault".to_owned(),
            operation: "activate".to_owned(),
            target: Some("presentation-cue-delay".to_owned()),
            logical_instant: None,
            advance_seconds: None,
            delay_milliseconds: Some(200),
            expected_revision: 3,
            requested_at: "2026-08-27T12:00:00Z".to_owned(),
        };
        let first = runtime.apply_control(&fault, now).expect("fault control");
        let duplicate = runtime
            .apply_control(&fault, now)
            .expect("duplicate fault control");
        assert_eq!(first, duplicate);

        let cue: PresentationCue = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-003-welcome-cue.json"
        ))
        .expect("cue");
        assert_eq!(
            runtime
                .accept_cue(&cue, now + Duration::seconds(31))
                .expect("cue after automatic fault expiry")
                .delay_milliseconds,
            0
        );

        let reset = ScenarioControlCommand {
            operation_id: "operation:reset:001".to_owned(),
            kind: "reset".to_owned(),
            operation: "execute".to_owned(),
            target: Some("presentation-registry-baseline".to_owned()),
            delay_milliseconds: None,
            ..fault
        };
        assert_eq!(
            runtime
                .apply_control(&reset, now + Duration::seconds(32))
                .expect("reset control")
                .status,
            OutcomeStatus::Accepted
        );
        assert!(matches!(
            runtime.current_registration("session:assurance:001", "audience-display"),
            Err(PresentationError::OperationRefused("surface-unregistered"))
        ));
        assert_eq!(
            runtime
                .apply_control(&reset, now + Duration::seconds(33))
                .expect("idempotent reset"),
            runtime
                .apply_control(&reset, now + Duration::seconds(34))
                .expect("second idempotent reset")
        );
    }

    #[test]
    fn operational_expiry_is_not_scenario_time() {
        let (_directory, runtime) = runtime();
        let now = OffsetDateTime::parse("2026-08-27T12:10:00Z", &Rfc3339).expect("time");
        let cue: PresentationCue = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-003-welcome-cue.json"
        ))
        .expect("cue");
        assert!(matches!(
            runtime.accept_cue(&cue, now),
            Err(PresentationError::OperationRefused("expired"))
        ));
    }

    #[test]
    fn applied_outcome_never_claims_business_completion() {
        let outcome: PresentationCueOutcome = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-004-welcome-applied.json"
        ))
        .expect("outcome");
        assert_eq!(outcome.result, PresentationOutcomeResult::Applied);
        assert!(!outcome.business_completion_claimed);
    }
}
