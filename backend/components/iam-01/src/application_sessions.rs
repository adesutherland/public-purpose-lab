//! Durable ordinary application sessions for the M3.4 identity boundary.
//!
//! This store deliberately keeps external-human authentication separate from
//! synthetic actor bindings. It stores hashes of browser credentials and the
//! privacy-minimised `I-001`/`I-005` contexts, never provider tokens or signed
//! demonstration grants.

use std::{fmt, path::PathBuf};

use ppl_contracts::{
    ExternalHumanIdentityContext, SyntheticSessionOutcome, SyntheticSessionStatus,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

const SESSION_LIFETIME: Duration = Duration::minutes(30);

#[derive(Clone, Debug)]
pub struct ApplicationSessionStore {
    database_path: PathBuf,
    environment_id: String,
    audience: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionCredentials {
    pub token: String,
    pub csrf_token: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorisedApplicationSession {
    pub session_id: String,
    pub external_identity: ExternalHumanIdentityContext,
    pub synthetic_identity: Option<SyntheticSessionOutcome>,
    pub expires_at: String,
    pub bound_surface_id: Option<String>,
    pub bound_demonstration_session_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    StateUnavailable,
    StateInconsistent,
    InvalidIdentity,
    SessionRequired,
    SessionExpired,
    SessionRevoked,
    RoleRefused,
    MappingChanged,
    CsrfRefused,
    SyntheticBindingRefused,
    RandomUnavailable,
}

impl SessionError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::StateUnavailable => "application-session-state-unavailable",
            Self::StateInconsistent => "application-session-state-inconsistent",
            Self::InvalidIdentity => "external-identity-context-invalid",
            Self::SessionRequired => "application-session-required",
            Self::SessionExpired => "application-session-expired",
            Self::SessionRevoked => "application-session-revoked",
            Self::RoleRefused => "application-session-role-refused",
            Self::MappingChanged => "application-session-mapping-changed",
            Self::CsrfRefused => "csrf-refused",
            Self::SyntheticBindingRefused => "synthetic-session-binding-refused",
            Self::RandomUnavailable => "operating-system-randomness-unavailable",
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for SessionError {}

impl ApplicationSessionStore {
    /// Opens one component-owned session store and verifies its schema.
    ///
    /// # Errors
    /// Returns a safe error if the state cannot be opened or reconciled.
    pub fn open(
        database_path: impl Into<PathBuf>,
        environment_id: impl Into<String>,
        audience: impl Into<String>,
    ) -> Result<Self, SessionError> {
        let store = Self {
            database_path: database_path.into(),
            environment_id: environment_id.into(),
            audience: audience.into(),
        };
        if store.environment_id.is_empty() || store.audience.is_empty() {
            return Err(SessionError::StateInconsistent);
        }
        store.prepare()?;
        Ok(store)
    }

    /// Rotates a verified external identity into a new ordinary application
    /// session.
    ///
    /// # Errors
    /// Refuses mismatched, expired or role-less identity contexts and any
    /// unavailable durable state.
    pub fn establish(
        &self,
        identity: &ExternalHumanIdentityContext,
        now: OffsetDateTime,
    ) -> Result<SessionCredentials, SessionError> {
        validate_identity(identity, &self.environment_id, &self.audience, now)?;
        let token = random_credential("app")?;
        let csrf_token = random_credential("csrf")?;
        let session_id = random_credential("application-session")?;
        let identity_expiry = parse_time(&identity.expires_at)?;
        let expires_at = identity_expiry.min(now + SESSION_LIFETIME);
        let identity_json =
            serde_json::to_string(identity).map_err(|_| SessionError::StateInconsistent)?;
        self.connection()?
            .execute(
                "INSERT INTO application_sessions(
                   session_id,token_hash,csrf_hash,environment_id,audience,mapping_version,
                   external_identity_json,created_at,last_used_at,expires_at,revoked_at,
                   revoke_reason,synthetic_identity_json
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8,?9,NULL,NULL,NULL)",
                params![
                    session_id,
                    digest(&token),
                    digest(&csrf_token),
                    self.environment_id,
                    self.audience,
                    identity.mapping_version,
                    identity_json,
                    format_time(now)?,
                    format_time(expires_at)?,
                ],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        Ok(SessionCredentials {
            token,
            csrf_token,
            expires_at: format_time(expires_at)?,
        })
    }

    /// Validates an ordinary application session for a read operation.
    ///
    /// # Errors
    /// Refuses missing, expired, revoked, stale-mapping or insufficient-role
    /// sessions without disclosing whether another identity exists.
    pub fn authorise_read(
        &self,
        token: &str,
        required_role: &str,
        current_mapping_version: &str,
        now: OffsetDateTime,
    ) -> Result<AuthorisedApplicationSession, SessionError> {
        self.authorise(token, None, required_role, current_mapping_version, now)
    }

    /// Validates a session and its per-session CSRF credential for a write.
    ///
    /// # Errors
    /// Returns the same safe refusals as `authorise_read` plus CSRF refusal.
    pub fn authorise_write(
        &self,
        token: &str,
        csrf_token: &str,
        required_role: &str,
        current_mapping_version: &str,
        now: OffsetDateTime,
    ) -> Result<AuthorisedApplicationSession, SessionError> {
        self.authorise(
            token,
            Some(csrf_token),
            required_role,
            current_mapping_version,
            now,
        )
    }

    fn authorise(
        &self,
        token: &str,
        csrf_token: Option<&str>,
        required_role: &str,
        current_mapping_version: &str,
        now: OffsetDateTime,
    ) -> Result<AuthorisedApplicationSession, SessionError> {
        let connection = self.connection()?;
        let stored = connection
            .query_row(
                "SELECT session_id,csrf_hash,environment_id,audience,mapping_version,
                        external_identity_json,synthetic_identity_json,expires_at,revoked_at,
                        bound_surface_id,bound_demonstration_session_id
                 FROM application_sessions WHERE token_hash=?1",
                params![digest(token)],
                |row| {
                    Ok(StoredSession {
                        session_id: row.get(0)?,
                        csrf_hash: row.get(1)?,
                        environment_id: row.get(2)?,
                        audience: row.get(3)?,
                        mapping_version: row.get(4)?,
                        external_identity_json: row.get(5)?,
                        synthetic_identity_json: row.get(6)?,
                        expires_at: row.get(7)?,
                        revoked_at: row.get(8)?,
                        bound_surface_id: row.get(9)?,
                        bound_demonstration_session_id: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(|_| SessionError::StateUnavailable)?
            .ok_or(SessionError::SessionRequired)?;
        if stored.environment_id != self.environment_id || stored.audience != self.audience {
            return Err(SessionError::StateInconsistent);
        }
        if stored.revoked_at.is_some() {
            return Err(SessionError::SessionRevoked);
        }
        if stored.mapping_version != current_mapping_version {
            self.revoke_by_id(&stored.session_id, "role-mapping-changed", now)?;
            return Err(SessionError::MappingChanged);
        }
        if parse_time(&stored.expires_at)? <= now {
            self.revoke_by_id(&stored.session_id, "session-expired", now)?;
            return Err(SessionError::SessionExpired);
        }
        if let Some(supplied) = csrf_token
            && !constant_time_equal(&stored.csrf_hash, &digest(supplied))
        {
            return Err(SessionError::CsrfRefused);
        }
        let identity: ExternalHumanIdentityContext =
            serde_json::from_str(&stored.external_identity_json)
                .map_err(|_| SessionError::StateInconsistent)?;
        validate_identity(&identity, &self.environment_id, &self.audience, now)?;
        if !identity.roles.iter().any(|role| role == required_role) {
            return Err(SessionError::RoleRefused);
        }
        let synthetic_identity = stored
            .synthetic_identity_json
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(|_| SessionError::StateInconsistent)?;
        connection
            .execute(
                "UPDATE application_sessions SET last_used_at=?1 WHERE session_id=?2",
                params![format_time(now)?, stored.session_id],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        Ok(AuthorisedApplicationSession {
            session_id: stored.session_id,
            external_identity: identity,
            synthetic_identity,
            expires_at: stored.expires_at,
            bound_surface_id: stored.bound_surface_id,
            bound_demonstration_session_id: stored.bound_demonstration_session_id,
        })
    }

    /// Binds one already validated `I-005` outcome to the current application
    /// session. The signed grant is not accepted by this method or persisted.
    ///
    /// # Errors
    /// Refuses non-established, wrong-environment or conflicting bindings.
    pub fn bind_synthetic(
        &self,
        token: &str,
        outcome: &SyntheticSessionOutcome,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        if outcome.status != SyntheticSessionStatus::Established
            || outcome.environment_id != self.environment_id
            || outcome.application_id != self.audience
            || outcome.maximum_valid_until.as_deref().is_none()
            || parse_time(outcome.maximum_valid_until.as_deref().unwrap_or_default())? <= now
        {
            return Err(SessionError::SyntheticBindingRefused);
        }
        let payload =
            serde_json::to_string(outcome).map_err(|_| SessionError::StateInconsistent)?;
        let changed = self
            .connection()?
            .execute(
                "UPDATE application_sessions SET synthetic_identity_json=?1,last_used_at=?2
                 ,synthetic_demonstration_session_id=?4
                 WHERE token_hash=?3 AND revoked_at IS NULL AND expires_at>?2
                   AND (synthetic_identity_json IS NULL OR synthetic_identity_json=?1)",
                params![
                    payload,
                    format_time(now)?,
                    digest(token),
                    outcome.demonstration_session_id
                ],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(SessionError::SyntheticBindingRefused)
        }
    }

    /// Claims one presentation surface for this ordinary application session.
    /// Any earlier live claim for the same surface and Demonstration Session is
    /// displaced before a new synthetic grant can be bound.
    ///
    /// # Errors
    /// Refuses an unknown/revoked/expired session or unavailable state.
    pub fn bind_surface(
        &self,
        token: &str,
        surface_id: &str,
        demonstration_session_id: &str,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        if surface_id.is_empty() || demonstration_session_id.is_empty() {
            return Err(SessionError::SyntheticBindingRefused);
        }
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| SessionError::StateUnavailable)?;
        transaction
            .execute(
                "UPDATE application_sessions SET bound_surface_id=NULL,
                 bound_demonstration_session_id=NULL,synthetic_identity_json=NULL,
                 synthetic_demonstration_session_id=NULL,last_used_at=?1
                 WHERE bound_surface_id=?2 AND bound_demonstration_session_id=?3",
                params![format_time(now)?, surface_id, demonstration_session_id],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        let changed = transaction
            .execute(
                "UPDATE application_sessions SET bound_surface_id=?1,
                 bound_demonstration_session_id=?2,last_used_at=?3
                 WHERE token_hash=?4 AND revoked_at IS NULL AND expires_at>?3",
                params![
                    surface_id,
                    demonstration_session_id,
                    format_time(now)?,
                    digest(token)
                ],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        if changed != 1 {
            return Err(SessionError::SessionRequired);
        }
        transaction
            .commit()
            .map_err(|_| SessionError::StateUnavailable)
    }

    /// Binds a validated synthetic outcome to the ordinary session currently
    /// claiming the signed surface and scenario.
    ///
    /// # Errors
    /// Refuses absent, expired or ambiguous surface claims and conflicting
    /// synthetic outcomes.
    pub fn bind_synthetic_to_surface(
        &self,
        outcome: &SyntheticSessionOutcome,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        if outcome.status != SyntheticSessionStatus::Established
            || outcome.environment_id != self.environment_id
            || outcome.application_id != self.audience
            || outcome.maximum_valid_until.as_deref().is_none()
            || parse_time(outcome.maximum_valid_until.as_deref().unwrap_or_default())? <= now
        {
            return Err(SessionError::SyntheticBindingRefused);
        }
        let payload =
            serde_json::to_string(outcome).map_err(|_| SessionError::StateInconsistent)?;
        let changed = self
            .connection()?
            .execute(
                "UPDATE application_sessions SET synthetic_identity_json=?1,
                 synthetic_demonstration_session_id=?2,last_used_at=?3
                 WHERE bound_surface_id=?4 AND bound_demonstration_session_id=?2
                   AND revoked_at IS NULL AND expires_at>?3
                   AND (synthetic_identity_json IS NULL OR synthetic_identity_json=?1)",
                params![
                    payload,
                    outcome.demonstration_session_id,
                    format_time(now)?,
                    outcome.surface_id
                ],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(SessionError::SyntheticBindingRefused)
        }
    }

    /// Revokes an ordinary session idempotently.
    ///
    /// # Errors
    /// Returns a safe state error if revocation cannot be committed.
    pub fn revoke(
        &self,
        token: &str,
        reason: &str,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        self.connection()?
            .execute(
                "UPDATE application_sessions SET revoked_at=COALESCE(revoked_at,?1),
                 revoke_reason=COALESCE(revoke_reason,?2),synthetic_identity_json=NULL,
                 synthetic_demonstration_session_id=NULL,bound_surface_id=NULL,
                 bound_demonstration_session_id=NULL
                 WHERE token_hash=?3",
                params![format_time(now)?, safe_reason(reason), digest(token)],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        Ok(())
    }

    /// Removes synthetic authority for one stopped, reset or superseded
    /// Demonstration Session while preserving the external operator session.
    ///
    /// # Errors
    /// Returns a safe state error if the change cannot be committed.
    pub fn clear_synthetic_bindings(
        &self,
        demonstration_session_id: &str,
        now: OffsetDateTime,
    ) -> Result<usize, SessionError> {
        self.connection()?
            .execute(
                "UPDATE application_sessions SET synthetic_identity_json=NULL,
                 synthetic_demonstration_session_id=NULL,last_used_at=?1
                 WHERE synthetic_demonstration_session_id=?2",
                params![format_time(now)?, demonstration_session_id],
            )
            .map_err(|_| SessionError::StateUnavailable)
    }

    fn revoke_by_id(
        &self,
        session_id: &str,
        reason: &str,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        self.connection()?
            .execute(
                "UPDATE application_sessions SET revoked_at=COALESCE(revoked_at,?1),
                 revoke_reason=COALESCE(revoke_reason,?2),synthetic_identity_json=NULL,
                 synthetic_demonstration_session_id=NULL,bound_surface_id=NULL,
                 bound_demonstration_session_id=NULL
                 WHERE session_id=?3",
                params![format_time(now)?, safe_reason(reason), session_id],
            )
            .map_err(|_| SessionError::StateUnavailable)?;
        Ok(())
    }

    fn prepare(&self) -> Result<(), SessionError> {
        if let Some(parent) = self.database_path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| SessionError::StateUnavailable)?;
        }
        let connection = self.connection()?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS application_sessions (
                   session_id TEXT PRIMARY KEY,
                   token_hash TEXT NOT NULL UNIQUE,
                   csrf_hash TEXT NOT NULL,
                   environment_id TEXT NOT NULL,
                   audience TEXT NOT NULL,
                   mapping_version TEXT NOT NULL,
                   external_identity_json TEXT NOT NULL,
                   created_at TEXT NOT NULL,
                   last_used_at TEXT NOT NULL,
                   expires_at TEXT NOT NULL,
                   revoked_at TEXT,
                   revoke_reason TEXT,
                   synthetic_identity_json TEXT,
                   synthetic_demonstration_session_id TEXT,
                   bound_surface_id TEXT,
                   bound_demonstration_session_id TEXT
                 );
                 CREATE INDEX IF NOT EXISTS application_sessions_expiry
                 ON application_sessions(expires_at);",
            )
            .map_err(|_| SessionError::StateUnavailable)
    }

    fn connection(&self) -> Result<Connection, SessionError> {
        let connection =
            Connection::open(&self.database_path).map_err(|_| SessionError::StateUnavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| SessionError::StateUnavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| SessionError::StateUnavailable)?;
        Ok(connection)
    }
}

struct StoredSession {
    session_id: String,
    csrf_hash: String,
    environment_id: String,
    audience: String,
    mapping_version: String,
    external_identity_json: String,
    synthetic_identity_json: Option<String>,
    expires_at: String,
    revoked_at: Option<String>,
    bound_surface_id: Option<String>,
    bound_demonstration_session_id: Option<String>,
}

fn validate_identity(
    identity: &ExternalHumanIdentityContext,
    environment_id: &str,
    audience: &str,
    now: OffsetDateTime,
) -> Result<(), SessionError> {
    if identity.contract_id != "I-001"
        || identity.contract_version != "1.0.0"
        || identity.environment_id != environment_id
        || identity.audience != audience
        || identity.issuer.is_empty()
        || identity.subject_id.is_empty()
        || identity.principal_id.is_empty()
        || identity.roles.is_empty()
        || identity.mapping_version.is_empty()
    {
        return Err(SessionError::InvalidIdentity);
    }
    let issued_at = parse_time(&identity.issued_at)?;
    let expires_at = parse_time(&identity.expires_at)?;
    if issued_at > now || expires_at <= now || expires_at <= issued_at {
        return Err(SessionError::InvalidIdentity);
    }
    Ok(())
}

fn random_credential(prefix: &str) -> Result<String, SessionError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| SessionError::RandomUnavailable)?;
    Ok(format!("{prefix}-{}", hex(&bytes)))
}

fn digest(value: &str) -> String {
    hex(Sha256::digest(value.as_bytes()).as_slice())
}

fn constant_time_equal(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .as_bytes()
            .iter()
            .zip(right.as_bytes())
            .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}

fn safe_reason(reason: &str) -> &str {
    if reason.is_empty() || reason.len() > 80 {
        "session-revoked"
    } else {
        reason
    }
}

fn parse_time(value: &str) -> Result<OffsetDateTime, SessionError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| SessionError::StateInconsistent)
}

fn format_time(value: OffsetDateTime) -> Result<String, SessionError> {
    value
        .format(&Rfc3339)
        .map_err(|_| SessionError::StateInconsistent)
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use ppl_contracts::{AuthenticationStrength, I005_VERSION};
    use tempfile::tempdir;

    fn identity(now: OffsetDateTime) -> ExternalHumanIdentityContext {
        ExternalHumanIdentityContext {
            contract_id: "I-001".to_owned(),
            contract_version: "1.0.0".to_owned(),
            context_id: "external-context-test".to_owned(),
            environment_id: "environment-a".to_owned(),
            issuer: "https://accounts.example.test".to_owned(),
            subject_id: "subject-1".to_owned(),
            principal_id: "presenter-one".to_owned(),
            roles: vec!["presenter".to_owned()],
            audience: "scenario-director".to_owned(),
            authentication_strength: AuthenticationStrength::SingleFactor,
            mapping_version: "mapping-v1".to_owned(),
            issued_at: format_time(now).expect("time"),
            expires_at: format_time(now + Duration::hours(1)).expect("time"),
            decision_reference: Some("decision-test".to_owned()),
        }
    }

    fn outcome(now: OffsetDateTime) -> SyntheticSessionOutcome {
        SyntheticSessionOutcome {
            contract_id: "I-005".to_owned(),
            contract_version: I005_VERSION.to_owned(),
            outcome_id: "outcome-1".to_owned(),
            grant_id: "grant-1".to_owned(),
            establishment_operation_id: "operation-1".to_owned(),
            environment_id: "environment-a".to_owned(),
            application_id: "scenario-director".to_owned(),
            surface_id: "director-console".to_owned(),
            demonstration_session_id: "scenario-1".to_owned(),
            actor_id: "synthetic-presenter".to_owned(),
            roles: vec!["synthetic-presenter".to_owned()],
            synthetic_realm: "demonstration".to_owned(),
            status: SyntheticSessionStatus::Established,
            occurred_at: format_time(now).expect("time"),
            maximum_valid_until: Some(format_time(now + Duration::minutes(20)).expect("time")),
            session_reference: Some("protected-session-reference".to_owned()),
            reason_code: None,
            decision_reference: "decision-1".to_owned(),
            original_outcome_id: None,
            evidence_references: vec!["evidence-1".to_owned()],
        }
    }

    #[test]
    fn survives_restart_and_never_persists_raw_browser_credentials() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("sessions.sqlite");
        let now = OffsetDateTime::parse("2030-01-01T09:00:00Z", &Rfc3339).expect("time");
        let store = ApplicationSessionStore::open(&path, "environment-a", "scenario-director")
            .expect("store");
        let credentials = store.establish(&identity(now), now).expect("session");
        drop(store);

        let reopened = ApplicationSessionStore::open(&path, "environment-a", "scenario-director")
            .expect("reopened store");
        let authorised = reopened
            .authorise_write(
                &credentials.token,
                &credentials.csrf_token,
                "presenter",
                "mapping-v1",
                now + Duration::minutes(1),
            )
            .expect("authorised");
        assert_eq!(authorised.external_identity.subject_id, "subject-1");
        let bytes = std::fs::read(&path).expect("database");
        assert!(
            !bytes
                .windows(credentials.token.len())
                .any(|window| window == credentials.token.as_bytes())
        );
        assert!(
            !bytes
                .windows(credentials.csrf_token.len())
                .any(|window| window == credentials.csrf_token.as_bytes())
        );
    }

    #[test]
    fn refuses_csrf_role_mapping_and_expiry_failures() {
        let directory = tempdir().expect("temporary directory");
        let now = OffsetDateTime::parse("2030-01-01T09:00:00Z", &Rfc3339).expect("time");
        let store = ApplicationSessionStore::open(
            directory.path().join("sessions.sqlite"),
            "environment-a",
            "scenario-director",
        )
        .expect("store");
        let credentials = store.establish(&identity(now), now).expect("session");
        assert_eq!(
            store.authorise_write(&credentials.token, "wrong", "presenter", "mapping-v1", now,),
            Err(SessionError::CsrfRefused)
        );
        assert_eq!(
            store.authorise_read(&credentials.token, "administrator", "mapping-v1", now,),
            Err(SessionError::RoleRefused)
        );
        assert_eq!(
            store.authorise_read(&credentials.token, "presenter", "mapping-v2", now,),
            Err(SessionError::MappingChanged)
        );
        assert_eq!(
            store.authorise_read(
                &credentials.token,
                "presenter",
                "mapping-v1",
                now + Duration::hours(2),
            ),
            Err(SessionError::SessionRevoked)
        );
    }

    #[test]
    fn synthetic_binding_is_durable_and_conflict_safe() {
        let directory = tempdir().expect("temporary directory");
        let now = OffsetDateTime::parse("2030-01-01T09:00:00Z", &Rfc3339).expect("time");
        let store = ApplicationSessionStore::open(
            directory.path().join("sessions.sqlite"),
            "environment-a",
            "scenario-director",
        )
        .expect("store");
        let credentials = store.establish(&identity(now), now).expect("session");
        store
            .bind_synthetic(&credentials.token, &outcome(now), now)
            .expect("binding");
        store
            .bind_synthetic(&credentials.token, &outcome(now), now)
            .expect("duplicate binding");
        let mut conflict = outcome(now);
        conflict.outcome_id = "outcome-conflict".to_owned();
        assert_eq!(
            store.bind_synthetic(&credentials.token, &conflict, now),
            Err(SessionError::SyntheticBindingRefused)
        );
        let authorised = store
            .authorise_read(&credentials.token, "presenter", "mapping-v1", now)
            .expect("authorised");
        assert_eq!(
            authorised
                .synthetic_identity
                .expect("synthetic identity")
                .actor_id,
            "synthetic-presenter"
        );
    }
}
