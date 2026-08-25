//! Dependency-light types shared by Public Purpose Lab host components.

use std::fmt;

/// The implementation maturity of a repository component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Maturity {
    /// A compilable boundary with no operational capability claim.
    RepositorySkeleton,
    /// Executable behaviour with incomplete milestone or profile evidence.
    InDevelopment,
}

impl fmt::Display for Maturity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepositorySkeleton => formatter.write_str("repository-skeleton"),
            Self::InDevelopment => formatter.write_str("in-development"),
        }
    }
}

/// A machine-readable description exposed by an implemented package boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub maturity: Maturity,
    pub contracts: &'static [&'static str],
}

impl fmt::Display for ComponentDescriptor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}|{}|{}|{}",
            self.id,
            self.name,
            self.maturity,
            self.contracts.join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentDescriptor, Maturity};

    #[test]
    fn descriptor_is_stable_and_machine_readable() {
        let descriptor = ComponentDescriptor {
            id: "TST-01",
            name: "Test component",
            maturity: Maturity::RepositorySkeleton,
            contracts: &["T-001"],
        };

        assert_eq!(
            descriptor.to_string(),
            "TST-01|Test component|repository-skeleton|T-001"
        );
    }

    #[test]
    fn in_development_is_not_described_as_demonstrated() {
        assert_eq!(Maturity::InDevelopment.to_string(), "in-development");
    }
}
