//! Target-application validation and replay state for `I-004`/`I-005`.
//!
//! Issuer and target application deliberately do not share a private journal.
//! The target receives a public trust bundle and owns durable establishment
//! evidence for its application boundary.

use std::{fmt, path::PathBuf};

use ed25519_dalek::{Signature, VerifyingKey};
use ppl_contracts::{
    DemonstrationSignInGrant, I004_VERSION, I005_VERSION, SyntheticSessionOutcome,
    SyntheticSessionStatus, TrustStatus,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

use crate::SyntheticTrustBundle;

const SIGNATURE_PROFILE: &str = "ppl-i004-ed25519-v1";
const SESSION_LIFETIME: Duration = Duration::minutes(30);

#[derive(Clone, Debug)]
pub struct ApplicationGrantStore {
    database_path: PathBuf,
    trust: SyntheticTrustBundle,
    application_id: String,
    audience: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrantStoreError {
    StateUnavailable,
    StateInconsistent,
    TrustNotReady,
    RandomUnavailable,
}

impl fmt::Display for GrantStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StateUnavailable => "synthetic-establishment-state-unavailable",
            Self::StateInconsistent => "synthetic-establishment-state-inconsistent",
            Self::TrustNotReady => "synthetic-trust-not-ready",
            Self::RandomUnavailable => "operating-system-randomness-unavailable",
        })
    }
}

impl std::error::Error for GrantStoreError {}

impl ApplicationGrantStore {
    /// Opens a target-owned replay store using only public trust material.
    ///
    /// # Errors
    /// Refuses unusable trust or unavailable/inconsistent state.
    pub fn open(
        database_path: impl Into<PathBuf>,
        trust: SyntheticTrustBundle,
        application_id: impl Into<String>,
        audience: impl Into<String>,
    ) -> Result<Self, GrantStoreError> {
        if trust.record.status != TrustStatus::Ready
            || !trust.record.compatible
            || trust.record.environment_id.is_empty()
            || trust.record.trust_domain.is_empty()
            || trust.record.trust_epoch == 0
        {
            return Err(GrantStoreError::TrustNotReady);
        }
        validate_public_key(&trust)?;
        let store = Self {
            database_path: database_path.into(),
            trust,
            application_id: application_id.into(),
            audience: audience.into(),
        };
        if store.application_id.is_empty() || store.audience.is_empty() {
            return Err(GrantStoreError::StateInconsistent);
        }
        store.prepare()?;
        Ok(store)
    }

    #[must_use]
    pub fn environment_id(&self) -> &str {
        &self.trust.record.environment_id
    }

    /// Validates and durably reconciles one backend-delivered grant.
    ///
    /// Invalid grants produce a privacy-minimised refused outcome. Protected
    /// state failures return an error and establish no session.
    ///
    /// # Errors
    /// Returns an error only when protected trust/replay state is unavailable
    /// or inconsistent.
    pub fn establish(
        &self,
        grant: &DemonstrationSignInGrant,
        expected_surface_id: &str,
        expected_demonstration_session_id: &str,
        now: OffsetDateTime,
    ) -> Result<SyntheticSessionOutcome, GrantStoreError> {
        if let Err(reason) = self.validate_grant(
            grant,
            expected_surface_id,
            expected_demonstration_session_id,
            now,
        ) {
            return refused_outcome(grant, reason, now);
        }
        let operation_digest = digest(&grant.claims.establishment_operation_id);
        let grant_digest = grant_digest(grant)?;
        let connection = self.connection()?;
        if let Some((stored_grant_digest, outcome_json)) = connection
            .query_row(
                "SELECT grant_digest,outcome_json FROM synthetic_establishments
                 WHERE operation_digest=?1",
                params![operation_digest],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| GrantStoreError::StateUnavailable)?
        {
            if stored_grant_digest != grant_digest {
                return refused_outcome(grant, "establishment-operation-conflict", now);
            }
            let mut outcome: SyntheticSessionOutcome = serde_json::from_str(&outcome_json)
                .map_err(|_| GrantStoreError::StateInconsistent)?;
            outcome.original_outcome_id = Some(outcome.outcome_id.clone());
            outcome.occurred_at = format_time(now)?;
            return Ok(outcome);
        }

        let mut random = [0_u8; 32];
        getrandom::fill(&mut random).map_err(|_| GrantStoreError::RandomUnavailable)?;
        let grant_expiry = parse_time(&grant.claims.expires_at)?;
        let maximum_valid_until = grant_expiry.min(now + SESSION_LIFETIME);
        let outcome = SyntheticSessionOutcome {
            contract_id: "I-005".to_owned(),
            contract_version: I005_VERSION.to_owned(),
            outcome_id: random_id("session-outcome")?,
            grant_id: grant.claims.grant_id.clone(),
            establishment_operation_id: grant.claims.establishment_operation_id.clone(),
            environment_id: grant.claims.environment_id.clone(),
            application_id: grant.claims.application_id.clone(),
            surface_id: grant.claims.surface_id.clone(),
            demonstration_session_id: grant.claims.demonstration_session_id.clone(),
            actor_id: grant.claims.actor_id.clone(),
            roles: grant.claims.roles.clone(),
            synthetic_realm: grant.claims.synthetic_realm.clone(),
            status: SyntheticSessionStatus::Established,
            occurred_at: format_time(now)?,
            maximum_valid_until: Some(format_time(maximum_valid_until)?),
            session_reference: Some(format!(
                "session-{}",
                hex(Sha256::digest(random).as_slice())
            )),
            reason_code: None,
            decision_reference: grant.claims.decision_reference.clone(),
            original_outcome_id: None,
            evidence_references: vec![format!(
                "target-establishment-{}",
                digest_prefix(&grant.claims.establishment_operation_id)
            )],
        };
        let outcome_json =
            serde_json::to_string(&outcome).map_err(|_| GrantStoreError::StateInconsistent)?;
        connection
            .execute(
                "INSERT INTO synthetic_establishments(
                   operation_digest,grant_digest,demonstration_session_id,actor_digest,
                   outcome_json,established_at,terminal_at,terminal_reason
                 ) VALUES(?1,?2,?3,?4,?5,?6,NULL,NULL)",
                params![
                    operation_digest,
                    grant_digest,
                    grant.claims.demonstration_session_id,
                    digest(&grant.claims.actor_id),
                    outcome_json,
                    format_time(now)?,
                ],
            )
            .map_err(|_| GrantStoreError::StateUnavailable)?;
        Ok(outcome)
    }

    /// Marks every establishment for one stopped/reset/superseded scenario as
    /// terminal. Ordinary application-session cleanup is owned separately.
    ///
    /// # Errors
    /// Returns a safe state error when termination cannot be committed.
    pub fn terminate_demonstration_session(
        &self,
        demonstration_session_id: &str,
        reason: &str,
        now: OffsetDateTime,
    ) -> Result<usize, GrantStoreError> {
        self.connection()?
            .execute(
                "UPDATE synthetic_establishments SET terminal_at=COALESCE(terminal_at,?1),
                 terminal_reason=COALESCE(terminal_reason,?2)
                 WHERE demonstration_session_id=?3",
                params![
                    format_time(now)?,
                    safe_reason(reason),
                    demonstration_session_id
                ],
            )
            .map_err(|_| GrantStoreError::StateUnavailable)
    }

    fn validate_grant(
        &self,
        grant: &DemonstrationSignInGrant,
        expected_surface_id: &str,
        expected_demonstration_session_id: &str,
        now: OffsetDateTime,
    ) -> Result<(), &'static str> {
        let record = &self.trust.record;
        if grant.contract_id != "I-004"
            || grant.contract_version != I004_VERSION
            || grant.signature.profile != SIGNATURE_PROFILE
            || grant.signature.algorithm != "Ed25519"
            || grant.signature.signer_id != record.signer_id
            || grant.signature.public_key_fingerprint != record.public_key_fingerprint
        {
            return Err("grant-signature-profile-invalid");
        }
        if grant.claims.environment_id != record.environment_id
            || grant.claims.trust_domain != record.trust_domain
            || grant.claims.trust_epoch != record.trust_epoch
        {
            return Err("grant-environment-mismatch");
        }
        if grant.claims.application_id != self.application_id
            || grant.claims.audience != self.audience
        {
            return Err("grant-application-or-audience-mismatch");
        }
        if grant.claims.surface_id != expected_surface_id
            || grant.claims.demonstration_session_id != expected_demonstration_session_id
        {
            return Err("grant-surface-or-session-mismatch");
        }
        let verifying_key = validating_key(&self.trust)?;
        let signature_bytes =
            decode_hex(&grant.signature.value).ok_or("grant-signature-invalid")?;
        let signature = Signature::try_from(signature_bytes.as_slice())
            .map_err(|_| "grant-signature-invalid")?;
        verifying_key
            .verify_strict(&canonical_claims(&grant.claims), &signature)
            .map_err(|_| "grant-signature-invalid")?;
        let issued_at = parse_time_safe(&grant.claims.issued_at)?;
        let not_before = parse_time_safe(&grant.claims.not_before)?;
        let expires_at = parse_time_safe(&grant.claims.expires_at)?;
        if issued_at > not_before
            || now < not_before
            || now >= expires_at
            || expires_at <= issued_at
        {
            return Err("grant-expired-or-premature");
        }
        Ok(())
    }

    fn prepare(&self) -> Result<(), GrantStoreError> {
        if let Some(parent) = self.database_path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| GrantStoreError::StateUnavailable)?;
        }
        self.connection()?
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS synthetic_establishments (
                   operation_digest TEXT PRIMARY KEY,
                   grant_digest TEXT NOT NULL,
                   demonstration_session_id TEXT NOT NULL,
                   actor_digest TEXT NOT NULL,
                   outcome_json TEXT NOT NULL,
                   established_at TEXT NOT NULL,
                   terminal_at TEXT,
                   terminal_reason TEXT
                 );
                 CREATE INDEX IF NOT EXISTS synthetic_establishments_session
                 ON synthetic_establishments(demonstration_session_id);",
            )
            .map_err(|_| GrantStoreError::StateUnavailable)
    }

    fn connection(&self) -> Result<Connection, GrantStoreError> {
        let connection =
            Connection::open(&self.database_path).map_err(|_| GrantStoreError::StateUnavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| GrantStoreError::StateUnavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| GrantStoreError::StateUnavailable)?;
        Ok(connection)
    }
}

fn validate_public_key(trust: &SyntheticTrustBundle) -> Result<(), GrantStoreError> {
    let key = validating_key(trust).map_err(|_| GrantStoreError::TrustNotReady)?;
    let fingerprint = format!("sha256:{}", hex(Sha256::digest(key.as_bytes()).as_slice()));
    if fingerprint == trust.record.public_key_fingerprint {
        Ok(())
    } else {
        Err(GrantStoreError::TrustNotReady)
    }
}

fn validating_key(trust: &SyntheticTrustBundle) -> Result<VerifyingKey, &'static str> {
    let bytes: [u8; 32] = decode_hex(&trust.public_key)
        .and_then(|value| value.try_into().ok())
        .ok_or("grant-public-key-invalid")?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| "grant-public-key-invalid")
}

fn canonical_claims(claims: &ppl_contracts::DemonstrationSignInGrantClaims) -> Vec<u8> {
    let mut roles = claims.roles.clone();
    roles.sort();
    let fields = [
        SIGNATURE_PROFILE.to_owned(),
        I004_VERSION.to_owned(),
        claims.grant_id.clone(),
        claims.establishment_operation_id.clone(),
        claims.environment_id.clone(),
        claims.trust_domain.clone(),
        claims.trust_epoch.to_string(),
        claims.actor_id.clone(),
        claims.application_id.clone(),
        claims.audience.clone(),
        claims.surface_id.clone(),
        claims.demonstration_session_id.clone(),
        roles.join("\u{001f}"),
        claims.purpose.clone(),
        claims.synthetic_realm.clone(),
        claims.decision_reference.clone(),
        claims.issued_at.clone(),
        claims.not_before.clone(),
        claims.expires_at.clone(),
    ];
    let mut encoded = Vec::new();
    for field in fields {
        let length = u32::try_from(field.len()).unwrap_or(u32::MAX);
        encoded.extend_from_slice(&length.to_be_bytes());
        encoded.extend_from_slice(field.as_bytes());
    }
    encoded
}

fn refused_outcome(
    grant: &DemonstrationSignInGrant,
    reason: &'static str,
    now: OffsetDateTime,
) -> Result<SyntheticSessionOutcome, GrantStoreError> {
    Ok(SyntheticSessionOutcome {
        contract_id: "I-005".to_owned(),
        contract_version: I005_VERSION.to_owned(),
        outcome_id: random_id("session-outcome")?,
        grant_id: format!("grant-ref-{}", digest_prefix(&grant.claims.grant_id)),
        establishment_operation_id: format!(
            "operation-ref-{}",
            digest_prefix(&grant.claims.establishment_operation_id)
        ),
        environment_id: format!(
            "environment-ref-{}",
            digest_prefix(&grant.claims.environment_id)
        ),
        application_id: format!(
            "application-ref-{}",
            digest_prefix(&grant.claims.application_id)
        ),
        surface_id: format!("surface-ref-{}", digest_prefix(&grant.claims.surface_id)),
        demonstration_session_id: format!(
            "demonstration-ref-{}",
            digest_prefix(&grant.claims.demonstration_session_id)
        ),
        actor_id: format!("actor-ref-{}", digest_prefix(&grant.claims.actor_id)),
        roles: vec!["refused".to_owned()],
        synthetic_realm: format!("realm-ref-{}", digest_prefix(&grant.claims.synthetic_realm)),
        status: if reason == "grant-expired-or-premature" {
            SyntheticSessionStatus::Expired
        } else {
            SyntheticSessionStatus::Refused
        },
        occurred_at: format_time(now)?,
        maximum_valid_until: None,
        session_reference: None,
        reason_code: Some(reason.to_owned()),
        decision_reference: format!(
            "decision-ref-{}",
            digest_prefix(&grant.claims.decision_reference)
        ),
        original_outcome_id: None,
        evidence_references: vec![format!(
            "target-refusal-{}",
            digest_prefix(&grant.claims.establishment_operation_id)
        )],
    })
}

fn grant_digest(grant: &DemonstrationSignInGrant) -> Result<String, GrantStoreError> {
    serde_json::to_vec(grant)
        .map(|bytes| hex(Sha256::digest(bytes).as_slice()))
        .map_err(|_| GrantStoreError::StateInconsistent)
}

fn random_id(prefix: &str) -> Result<String, GrantStoreError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| GrantStoreError::RandomUnavailable)?;
    Ok(format!("{prefix}-{}", hex(&bytes)))
}

fn digest(value: &str) -> String {
    hex(Sha256::digest(value.as_bytes()).as_slice())
}

fn digest_prefix(value: &str) -> String {
    digest(value)[..16].to_owned()
}

fn parse_time(value: &str) -> Result<OffsetDateTime, GrantStoreError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| GrantStoreError::StateInconsistent)
}

fn parse_time_safe(value: &str) -> Result<OffsetDateTime, &'static str> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| "grant-time-invalid")
}

fn format_time(value: OffsetDateTime) -> Result<String, GrantStoreError> {
    value
        .format(&Rfc3339)
        .map_err(|_| GrantStoreError::StateInconsistent)
}

fn safe_reason(reason: &str) -> &str {
    if reason.is_empty() || reason.len() > 80 {
        "synthetic-session-terminated"
    } else {
        reason
    }
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16)?;
            let low = char::from(pair[1]).to_digit(16)?;
            u8::try_from((high << 4) | low).ok()
        })
        .collect()
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
    use crate::{
        DemonstrationConfiguration, GrantRequest, IamRuntime, SyntheticActorRegistration,
        WorkloadRegistration,
    };
    use ppl_contracts::{AssertionStatus, AssertionType, AuthoritativeAssertion};
    use tempfile::tempdir;

    fn configured_issuer(
        directory: &std::path::Path,
        now: OffsetDateTime,
    ) -> (IamRuntime, SyntheticTrustBundle) {
        let issuer = IamRuntime::new(directory.join("issuer"));
        let bootstrap = issuer.bootstrap_local(now).expect("bootstrap");
        issuer
            .configure(
                &DemonstrationConfiguration {
                    environment_id: bootstrap.environment_id,
                    policy_version: "policy-v1".to_owned(),
                    relationship_source: "synthetic-relationships".to_owned(),
                    consent_source: "synthetic-consents".to_owned(),
                    supported_obligations: vec![
                        "mark-synthetic".to_owned(),
                        "restrict-realm".to_owned(),
                    ],
                    workloads: vec![WorkloadRegistration {
                        workload_id: "scenario-director".to_owned(),
                        audiences: vec!["iam-01-service".to_owned()],
                        contract_actions: vec!["I-004:request-grant".to_owned()],
                        enabled: true,
                    }],
                    actors: vec![SyntheticActorRegistration {
                        actor_id: "synthetic-audience-user".to_owned(),
                        display_name: "Synthetic Audience User".to_owned(),
                        roles: vec!["portal-viewer".to_owned()],
                        applications: vec!["presentation-gateway".to_owned()],
                        purposes: vec!["demonstration".to_owned()],
                        synthetic_realm: "environment-fixture".to_owned(),
                        enabled: true,
                    }],
                    assertions: vec![
                        AuthoritativeAssertion {
                            source_id: "synthetic-relationships".to_owned(),
                            assertion_type: AssertionType::Relationship,
                            subject_id: "synthetic-audience-user".to_owned(),
                            resource_id: "presentation-gateway".to_owned(),
                            purpose_codes: vec!["demonstration".to_owned()],
                            status: AssertionStatus::Active,
                            effective_at: format_time(now - Duration::minutes(1)).expect("time"),
                            expires_at: format_time(now + Duration::hours(1)).expect("time"),
                            version: "fixture-v1".to_owned(),
                        },
                        AuthoritativeAssertion {
                            source_id: "synthetic-consents".to_owned(),
                            assertion_type: AssertionType::Consent,
                            subject_id: "synthetic-audience-user".to_owned(),
                            resource_id: "presentation-gateway".to_owned(),
                            purpose_codes: vec!["demonstration".to_owned()],
                            status: AssertionStatus::Active,
                            effective_at: format_time(now - Duration::minutes(1)).expect("time"),
                            expires_at: format_time(now + Duration::hours(1)).expect("time"),
                            version: "fixture-v1".to_owned(),
                        },
                    ],
                },
                now,
            )
            .expect("configuration");
        let trust = issuer.trust_bundle().expect("trust bundle");
        (issuer, trust)
    }

    fn request(session: &str) -> GrantRequest {
        GrantRequest {
            workload_id: "scenario-director".to_owned(),
            actor_id: "synthetic-audience-user".to_owned(),
            application_id: "presentation-gateway".to_owned(),
            audience: "presentation-gateway".to_owned(),
            surface_id: "audience-display".to_owned(),
            demonstration_session_id: session.to_owned(),
            roles: vec!["portal-viewer".to_owned()],
            purpose: "demonstration".to_owned(),
            synthetic_realm: "environment-fixture".to_owned(),
        }
    }

    #[test]
    fn target_establishes_once_and_reconciles_after_restart() {
        let directory = tempdir().expect("temporary directory");
        let now = OffsetDateTime::parse("2030-01-01T09:00:00Z", &Rfc3339).expect("time");
        let (issuer, trust) = configured_issuer(directory.path(), now);
        let grant = issuer
            .issue_grant(&request("scenario-1"), now)
            .expect("grant");
        let path = directory.path().join("application.sqlite");
        let store = ApplicationGrantStore::open(
            &path,
            trust.clone(),
            "presentation-gateway",
            "presentation-gateway",
        )
        .expect("store");
        let first = store
            .establish(&grant, "audience-display", "scenario-1", now)
            .expect("established");
        drop(store);
        let reopened = ApplicationGrantStore::open(
            &path,
            trust,
            "presentation-gateway",
            "presentation-gateway",
        )
        .expect("reopened");
        let duplicate = reopened
            .establish(
                &grant,
                "audience-display",
                "scenario-1",
                now + Duration::seconds(1),
            )
            .expect("duplicate");
        assert_eq!(first.outcome_id, duplicate.outcome_id);
        assert_eq!(duplicate.original_outcome_id, Some(first.outcome_id));
    }

    #[test]
    fn refuses_wrong_session_expiry_and_cross_environment() {
        let directory = tempdir().expect("temporary directory");
        let now = OffsetDateTime::parse("2030-01-01T09:00:00Z", &Rfc3339).expect("time");
        let (issuer, trust) = configured_issuer(directory.path(), now);
        let grant = issuer
            .issue_grant(&request("scenario-1"), now)
            .expect("grant");
        let store = ApplicationGrantStore::open(
            directory.path().join("application.sqlite"),
            trust,
            "presentation-gateway",
            "presentation-gateway",
        )
        .expect("store");
        let wrong = store
            .establish(&grant, "audience-display", "scenario-2", now)
            .expect("safe refusal");
        assert_eq!(
            wrong.reason_code.as_deref(),
            Some("grant-surface-or-session-mismatch")
        );
        let expired = store
            .establish(
                &grant,
                "audience-display",
                "scenario-1",
                now + Duration::minutes(3),
            )
            .expect("safe refusal");
        assert_eq!(expired.status, SyntheticSessionStatus::Expired);

        let (_, other_trust) = configured_issuer(&directory.path().join("other"), now);
        let other_store = ApplicationGrantStore::open(
            directory.path().join("other-application.sqlite"),
            other_trust,
            "presentation-gateway",
            "presentation-gateway",
        )
        .expect("other store");
        let crossed = other_store
            .establish(&grant, "audience-display", "scenario-1", now)
            .expect("safe refusal");
        assert_eq!(
            crossed.reason_code.as_deref(),
            Some("grant-signature-profile-invalid")
        );
    }
}
