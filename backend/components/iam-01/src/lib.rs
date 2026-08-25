//! IAM-01 package boundary.
//!
//! This skeleton declares ownership of the IAM contract family. It does not
//! implement identity verification, key material, grants or sessions.

use ppl_core::{ComponentDescriptor, Maturity};

const IAM_CONTRACTS: &[&str] = &["I-001", "I-002", "I-003", "I-004", "I-005"];

/// Describes the IAM-01 boundary without claiming an operational capability.
#[must_use]
pub const fn descriptor() -> ComponentDescriptor {
    ComponentDescriptor {
        id: "IAM-01",
        name: "Identity, Trust and Synthetic Session Broker",
        maturity: Maturity::RepositorySkeleton,
        contracts: IAM_CONTRACTS,
    }
}

#[cfg(test)]
mod tests {
    use super::descriptor;

    #[test]
    fn descriptor_lists_the_complete_initial_identity_contract_family() {
        assert_eq!(
            descriptor().contracts,
            &["I-001", "I-002", "I-003", "I-004", "I-005"]
        );
    }
}
