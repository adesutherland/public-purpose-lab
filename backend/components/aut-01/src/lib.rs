//! Bounded M2 reference implementation of AUT-01.
//!
//! The adapter evaluates a deliberately small synthetic demonstration policy.
//! It is transport-neutral and replaceable; it is not a general policy engine
//! or evidence of legal, professional or regulatory authority.

use ppl_contracts::{
    AZ001_VERSION, AssertionStatus, AssertionType, AuthorisationDecision,
    AuthorisationDecisionRequest, AuthorisationDecisionStatus, AuthorisationObligation,
    PrincipalType,
};
use ppl_core::{ComponentDescriptor, Maturity};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const AUT_CONTRACTS: &[&str] = &["AZ-001", "C-002", "C-003", "C-004"];

#[must_use]
pub const fn descriptor() -> ComponentDescriptor {
    ComponentDescriptor {
        id: "AUT-01",
        name: "Policy Decision and Authorisation",
        maturity: Maturity::InDevelopment,
        contracts: AUT_CONTRACTS,
    }
}

#[derive(Clone, Debug)]
pub struct PolicyConfig {
    pub environment_id: String,
    pub policy_version: String,
    pub allowed_action: String,
    pub allowed_resources: Vec<String>,
    pub relationship_source: String,
    pub consent_source: String,
    pub obligations: Vec<AuthorisationObligation>,
    pub dependency_available: bool,
}

#[derive(Clone, Debug)]
pub struct PolicyAdapter {
    config: PolicyConfig,
}

impl PolicyAdapter {
    #[must_use]
    pub const fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn evaluate(
        &self,
        request: &AuthorisationDecisionRequest,
        now: OffsetDateTime,
    ) -> AuthorisationDecision {
        let (status, reason, valid_until) = self.decide(request, now);
        let obligations = if status == AuthorisationDecisionStatus::Permit {
            self.config.obligations.clone()
        } else {
            Vec::new()
        };
        AuthorisationDecision {
            contract_id: "AZ-001".to_owned(),
            contract_version: AZ001_VERSION.to_owned(),
            kind: "decision".to_owned(),
            decision_id: format!("decision-{}", digest_prefix(&request.request_id)),
            request_id: request.request_id.clone(),
            status,
            reason_code: reason.to_owned(),
            obligations,
            policy_version: self.config.policy_version.clone(),
            decided_at: format_time(now),
            valid_until,
            evidence_references: vec![format!(
                "evidence-decision-{}",
                digest_prefix(&request.request_id)
            )],
        }
    }

    #[allow(clippy::too_many_lines)]
    fn decide(
        &self,
        request: &AuthorisationDecisionRequest,
        now: OffsetDateTime,
    ) -> (AuthorisationDecisionStatus, &'static str, Option<String>) {
        if !self.config.dependency_available {
            return (
                AuthorisationDecisionStatus::Indeterminate,
                "dependency-unavailable",
                None,
            );
        }
        if request.contract_id != "AZ-001"
            || request.contract_version != AZ001_VERSION
            || request.kind != "decision-request"
            || request.environment_id != self.config.environment_id
            || request.requester.environment_id != self.config.environment_id
            || request.actor.environment_id != self.config.environment_id
        {
            return (
                AuthorisationDecisionStatus::Indeterminate,
                "decision-context-invalid",
                None,
            );
        }
        if request.requester.principal_type != PrincipalType::Workload
            || request.actor.principal_type != PrincipalType::SyntheticHuman
        {
            return (
                AuthorisationDecisionStatus::Deny,
                "principal-types-refused",
                None,
            );
        }
        if request.policy_version != self.config.policy_version {
            return (
                AuthorisationDecisionStatus::Indeterminate,
                "policy-version-unavailable",
                None,
            );
        }
        if request.action != self.config.allowed_action
            || !self.config.allowed_resources.contains(&request.resource)
        {
            return (
                AuthorisationDecisionStatus::NotApplicable,
                "policy-not-applicable",
                None,
            );
        }
        if request.requested_roles.is_empty() || request.purpose.is_empty() {
            return (
                AuthorisationDecisionStatus::Deny,
                "purpose-or-role-refused",
                None,
            );
        }

        let mut relationship_expiry = None;
        let mut consent_expiry = None;
        for assertion in &request.assertions {
            if assertion.subject_id != request.actor.principal_id
                || assertion.resource_id != request.resource
                || !assertion.purpose_codes.contains(&request.purpose)
            {
                continue;
            }
            if assertion.status == AssertionStatus::Revoked {
                return (
                    AuthorisationDecisionStatus::Deny,
                    "authoritative-assertion-revoked",
                    None,
                );
            }
            let Ok(effective) = parse_time(&assertion.effective_at) else {
                return (
                    AuthorisationDecisionStatus::Indeterminate,
                    "authoritative-assertion-invalid",
                    None,
                );
            };
            let Ok(expiry) = parse_time(&assertion.expires_at) else {
                return (
                    AuthorisationDecisionStatus::Indeterminate,
                    "authoritative-assertion-invalid",
                    None,
                );
            };
            if now < effective || now >= expiry {
                return (
                    AuthorisationDecisionStatus::Indeterminate,
                    "authoritative-assertion-stale",
                    None,
                );
            }
            match assertion.assertion_type {
                AssertionType::Relationship
                    if assertion.source_id == self.config.relationship_source =>
                {
                    relationship_expiry = Some(expiry);
                }
                AssertionType::Consent if assertion.source_id == self.config.consent_source => {
                    consent_expiry = Some(expiry);
                }
                AssertionType::Restriction => {
                    return (
                        AuthorisationDecisionStatus::Deny,
                        "active-restriction",
                        None,
                    );
                }
                AssertionType::Relationship
                | AssertionType::Consent
                | AssertionType::Organisation => {}
            }
        }

        let (Some(relationship_expiry), Some(consent_expiry)) =
            (relationship_expiry, consent_expiry)
        else {
            return (
                AuthorisationDecisionStatus::Indeterminate,
                "required-assertion-unavailable",
                None,
            );
        };
        if self.config.obligations.is_empty() {
            return (
                AuthorisationDecisionStatus::Indeterminate,
                "obligation-configuration-invalid",
                None,
            );
        }
        let expiry = relationship_expiry.min(consent_expiry);
        (
            AuthorisationDecisionStatus::Permit,
            "policy-permit",
            Some(format_time(expiry)),
        )
    }
}

fn parse_time(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(value, &Rfc3339)
}

fn format_time(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn digest_prefix(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex(&digest[..8])
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
    use super::{PolicyAdapter, PolicyConfig, descriptor, parse_time};
    use ppl_contracts::{
        AssertionStatus, AssertionType, AuthorisationDecisionRequest, AuthorisationDecisionStatus,
        AuthorisationObligation, AuthoritativeAssertion, PrincipalReference, PrincipalType,
    };

    fn now() -> time::OffsetDateTime {
        parse_time("2030-08-26T10:00:00Z").expect("test timestamp")
    }

    fn request() -> AuthorisationDecisionRequest {
        let principal = |principal_type, principal_id: &str| PrincipalReference {
            principal_type,
            principal_id: principal_id.to_owned(),
            environment_id: "env-local-001".to_owned(),
            issuer: "iam-env-local-001".to_owned(),
            trust_domain: Some("trust-env-local-001".to_owned()),
        };
        let assertion = |assertion_type, source_id: &str| AuthoritativeAssertion {
            source_id: source_id.to_owned(),
            assertion_type,
            subject_id: "synthetic-reviewer".to_owned(),
            resource_id: "workbench-app".to_owned(),
            purpose_codes: vec!["demonstrate-discovery".to_owned()],
            status: AssertionStatus::Active,
            effective_at: "2030-08-26T09:00:00Z".to_owned(),
            expires_at: "2030-08-26T11:00:00Z".to_owned(),
            version: "1.0.0".to_owned(),
        };
        AuthorisationDecisionRequest {
            contract_id: "AZ-001".to_owned(),
            contract_version: "1.0.0".to_owned(),
            kind: "decision-request".to_owned(),
            request_id: "authorisation-request-001".to_owned(),
            environment_id: "env-local-001".to_owned(),
            requester: principal(PrincipalType::Workload, "workload-director"),
            actor: principal(PrincipalType::SyntheticHuman, "synthetic-reviewer"),
            action: "issue-synthetic-grant".to_owned(),
            resource: "workbench-app".to_owned(),
            purpose: "demonstrate-discovery".to_owned(),
            requested_roles: vec!["reviewer".to_owned()],
            assertions: vec![
                assertion(AssertionType::Relationship, "source-relationships"),
                assertion(AssertionType::Consent, "source-consents"),
            ],
            policy_version: "1.0.0".to_owned(),
            requested_at: "2030-08-26T10:00:00Z".to_owned(),
        }
    }

    fn adapter() -> PolicyAdapter {
        PolicyAdapter::new(PolicyConfig {
            environment_id: "env-local-001".to_owned(),
            policy_version: "1.0.0".to_owned(),
            allowed_action: "issue-synthetic-grant".to_owned(),
            allowed_resources: vec!["workbench-app".to_owned()],
            relationship_source: "source-relationships".to_owned(),
            consent_source: "source-consents".to_owned(),
            obligations: vec![AuthorisationObligation {
                code: "mark-synthetic".to_owned(),
                value: None,
            }],
            dependency_available: true,
        })
    }

    #[test]
    fn descriptor_is_in_development() {
        assert_eq!(
            descriptor().to_string(),
            "AUT-01|Policy Decision and Authorisation|in-development|AZ-001,C-002,C-003,C-004"
        );
    }

    #[test]
    fn current_relationship_and_consent_permit_with_obligations() {
        let decision = adapter().evaluate(&request(), now());
        assert_eq!(decision.status, AuthorisationDecisionStatus::Permit);
        assert_eq!(decision.obligations.len(), 1);
    }

    #[test]
    fn stale_assertion_and_wrong_principal_fail_closed() {
        let mut stale = request();
        stale.assertions[0].expires_at = "2030-08-26T09:30:00Z".to_owned();
        assert_eq!(
            adapter().evaluate(&stale, now()).status,
            AuthorisationDecisionStatus::Indeterminate
        );

        let mut substituted = request();
        substituted.requester.principal_type = PrincipalType::SyntheticHuman;
        assert_eq!(
            adapter().evaluate(&substituted, now()).status,
            AuthorisationDecisionStatus::Deny
        );
    }

    #[test]
    fn unsupported_action_is_not_applicable_and_dependency_failure_is_indeterminate() {
        let mut unsupported = request();
        unsupported.action = "release-report".to_owned();
        assert_eq!(
            adapter().evaluate(&unsupported, now()).status,
            AuthorisationDecisionStatus::NotApplicable
        );

        let mut config = adapter().config;
        config.dependency_available = false;
        assert_eq!(
            PolicyAdapter::new(config)
                .evaluate(&request(), now())
                .status,
            AuthorisationDecisionStatus::Indeterminate
        );
    }
}
