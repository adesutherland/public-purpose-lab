//! NATS `JetStream` physical adapter for the M3.4 Director/Identity/Presentation path.

use std::{path::PathBuf, time::Duration};

use async_nats::jetstream::{
    self,
    consumer::{AckPolicy, PullConsumer, pull},
    stream::{Config as StreamConfig, StorageType},
};
use ppl_contracts::OperationalEvent;
use serde::Serialize;

const STREAM_NAME: &str = "PPL_M3_PRESENTATION";
const SOURCE_STREAM_NAME: &str = "PPL_GATE_C_SOURCE";
pub const REGISTRATION_SUBJECT: &str = "ppl.m3.to-director.registration";
pub const CUE_SUBJECT: &str = "ppl.m3.to-presentation.cue";
pub const OUTCOME_SUBJECT: &str = "ppl.m3.to-director.outcome";
pub const DIRECTOR_EVENT_SUBJECT: &str = "ppl.m3.events.director";
pub const CONTROL_SUBJECT: &str = "ppl.m3.to-presentation.control";
pub const CONTROL_OUTCOME_SUBJECT: &str = "ppl.m3.to-director.control-outcome";
pub const GRANT_REQUEST_SUBJECT: &str = "ppl.m3.to-identity.grant-request";
pub const SYNTHETIC_GRANT_SUBJECT: &str = "ppl.m3.to-presentation.synthetic-grant";
pub const IDENTITY_OUTCOME_SUBJECT: &str = "ppl.m3.to-director.identity-outcome";
pub const SYNTHETIC_TERMINATION_SUBJECT: &str = "ppl.m3.to-presentation.synthetic-termination";
const PRESENTATION_CONSUMER_FILTER: &str = "ppl.m3.to-presentation.*";
pub const DIRECTOR_OPERATIONAL_SUBJECT: &str = "ppl.gate-a.events.CTL-01";
pub const PRESENTATION_OPERATIONAL_SUBJECT: &str = "ppl.gate-a.events.CTL-02";
pub const IDENTITY_OPERATIONAL_SUBJECT: &str = "ppl.gate-a.events.IAM-01";
pub const SOURCE_INTAKE_COMMAND_SUBJECT: &str = "ppl.gate-c.commands.CNT-01";
pub const SOURCE_INTAKE_QUERY_SUBJECT: &str = "ppl.gate-c.queries.CNT-01";
pub const SOURCE_AUTHORISATION_SUBJECT: &str = "ppl.gate-c.decisions.AUT-01";
pub const SOURCE_LIFECYCLE_EVENT_SUBJECT: &str = "ppl.gate-c.events.CNT-01";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkloadMode {
    ScenarioDirector,
    PresentationGateway,
    IdentityBroker,
}

#[derive(Clone, Debug)]
pub struct BrokerConfig {
    pub url: String,
    pub credentials_file: Option<PathBuf>,
    pub nkey_seed_file: Option<PathBuf>,
    pub root_certificate: Option<PathBuf>,
    pub client_certificate: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
    pub workload_mode: WorkloadMode,
}

#[derive(Clone)]
pub struct Broker {
    client: async_nats::Client,
    context: jetstream::Context,
    workload_mode: WorkloadMode,
}

#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("broker-configuration-invalid")]
    ConfigurationInvalid,
    #[error("broker-unavailable")]
    Unavailable,
    #[error("broker-stream-unavailable")]
    StreamUnavailable,
    #[error("broker-consumer-unavailable")]
    ConsumerUnavailable,
    #[error("broker-publish-unconfirmed")]
    PublishUnconfirmed,
    #[error("broker-payload-invalid")]
    PayloadInvalid,
    #[error("broker-action-not-permitted")]
    ActionNotPermitted,
    #[error("broker-request-timed-out")]
    RequestTimedOut,
}

impl Broker {
    /// Connects with the workload's `NKey` credentials and optional TLS material.
    ///
    /// # Errors
    /// Returns a safe error without exposing credential paths or broker details.
    pub async fn connect(config: BrokerConfig) -> Result<Self, BrokerError> {
        let mut options = async_nats::ConnectOptions::new().name(match config.workload_mode {
            WorkloadMode::ScenarioDirector => "ppl-scenario-director",
            WorkloadMode::PresentationGateway => "ppl-presentation-gateway",
            WorkloadMode::IdentityBroker => "ppl-identity-broker",
        });
        if let Some(credentials) = config.credentials_file {
            options = options
                .credentials_file(credentials)
                .await
                .map_err(|_| BrokerError::ConfigurationInvalid)?;
        }
        if let Some(seed_file) = config.nkey_seed_file {
            let seed = tokio::fs::read_to_string(seed_file)
                .await
                .map_err(|_| BrokerError::ConfigurationInvalid)?;
            options = options.nkey(seed.trim().to_owned());
        }
        if let Some(root) = config.root_certificate {
            options = options.add_root_certificates(root).require_tls(true);
        }
        match (config.client_certificate, config.client_key) {
            (Some(certificate), Some(key)) => {
                options = options.add_client_certificate(certificate, key);
            }
            (None, None) => {}
            _ => return Err(BrokerError::ConfigurationInvalid),
        }
        let client = options
            .connect(config.url)
            .await
            .map_err(|_| BrokerError::Unavailable)?;
        Ok(Self {
            context: jetstream::new(client.clone()),
            client,
            workload_mode: config.workload_mode,
        })
    }

    /// Creates or binds the bounded file-backed M3 presentation stream.
    ///
    /// # Errors
    /// Returns a safe error if `JetStream` is unavailable or policy refuses setup.
    pub async fn ensure_stream(&self) -> Result<(), BrokerError> {
        self.context
            .create_or_update_stream(StreamConfig {
                name: STREAM_NAME.to_owned(),
                subjects: vec![
                    REGISTRATION_SUBJECT.to_owned(),
                    CUE_SUBJECT.to_owned(),
                    OUTCOME_SUBJECT.to_owned(),
                    DIRECTOR_EVENT_SUBJECT.to_owned(),
                    CONTROL_SUBJECT.to_owned(),
                    CONTROL_OUTCOME_SUBJECT.to_owned(),
                    GRANT_REQUEST_SUBJECT.to_owned(),
                    SYNTHETIC_GRANT_SUBJECT.to_owned(),
                    IDENTITY_OUTCOME_SUBJECT.to_owned(),
                    SYNTHETIC_TERMINATION_SUBJECT.to_owned(),
                    DIRECTOR_OPERATIONAL_SUBJECT.to_owned(),
                    PRESENTATION_OPERATIONAL_SUBJECT.to_owned(),
                    IDENTITY_OPERATIONAL_SUBJECT.to_owned(),
                ],
                storage: StorageType::File,
                max_messages: 10_000,
                max_bytes: 16 * 1024 * 1024,
                max_age: Duration::from_hours(24),
                num_replicas: 1,
                ..Default::default()
            })
            .await
            .map_err(|_| BrokerError::StreamUnavailable)?;
        self.context
            .create_or_update_stream(StreamConfig {
                name: SOURCE_STREAM_NAME.to_owned(),
                subjects: vec![SOURCE_LIFECYCLE_EVENT_SUBJECT.to_owned()],
                storage: StorageType::File,
                max_messages: 1_000,
                max_bytes: 4 * 1024 * 1024,
                max_age: Duration::from_hours(24 * 7),
                num_replicas: 1,
                ..Default::default()
            })
            .await
            .map_err(|_| BrokerError::StreamUnavailable)?;
        Ok(())
    }

    /// Binds the durable, explicitly acknowledged consumer for this workload.
    ///
    /// # Errors
    /// Returns a safe error when stream or consumer binding fails.
    pub async fn consumer(&self) -> Result<PullConsumer, BrokerError> {
        let stream = self
            .context
            .get_stream(STREAM_NAME)
            .await
            .map_err(|_| BrokerError::StreamUnavailable)?;
        let (name, filter_subject) = match self.workload_mode {
            WorkloadMode::ScenarioDirector => ("scenario-director", "ppl.m3.to-director.*"),
            WorkloadMode::PresentationGateway => {
                ("presentation-gateway", PRESENTATION_CONSUMER_FILTER)
            }
            WorkloadMode::IdentityBroker => ("identity-broker", "ppl.m3.to-identity.*"),
        };
        stream
            .get_or_create_consumer(
                name,
                pull::Config {
                    durable_name: Some(name.to_owned()),
                    description: Some(
                        "Public Purpose Lab M3.4 durable contract consumer".to_owned(),
                    ),
                    ack_policy: AckPolicy::Explicit,
                    ack_wait: Duration::from_secs(15),
                    max_deliver: 5,
                    filter_subject: filter_subject.to_owned(),
                    max_ack_pending: 128,
                    ..Default::default()
                },
            )
            .await
            .map_err(|_| BrokerError::ConsumerUnavailable)
    }

    /// Publishes one typed JSON event and waits for the `JetStream` acknowledgement.
    ///
    /// # Errors
    /// Refuses subjects outside the current workload's publish policy.
    pub async fn publish<T: Serialize>(
        &self,
        subject: &'static str,
        payload: &T,
    ) -> Result<(), BrokerError> {
        if !self.can_publish(subject) {
            return Err(BrokerError::ActionNotPermitted);
        }
        let bytes = serde_json::to_vec(payload).map_err(|_| BrokerError::PayloadInvalid)?;
        self.context
            .publish(subject, bytes.into())
            .await
            .map_err(|_| BrokerError::PublishUnconfirmed)?
            .await
            .map_err(|_| BrokerError::PublishUnconfirmed)?;
        Ok(())
    }

    /// Publishes the current workload's Gate A operational event.
    ///
    /// # Errors
    /// Returns a safe error if the event does not match the connected workload
    /// or cannot be durably acknowledged by the bounded stream.
    pub async fn publish_operational_event(
        &self,
        event: &OperationalEvent,
    ) -> Result<(), BrokerError> {
        let subject = match self.workload_mode {
            WorkloadMode::ScenarioDirector if event.component_id == "CTL-01" => {
                DIRECTOR_OPERATIONAL_SUBJECT
            }
            WorkloadMode::PresentationGateway if event.component_id == "CTL-02" => {
                PRESENTATION_OPERATIONAL_SUBJECT
            }
            WorkloadMode::IdentityBroker if event.component_id == "IAM-01" => {
                IDENTITY_OPERATIONAL_SUBJECT
            }
            _ => return Err(BrokerError::ActionNotPermitted),
        };
        self.publish(subject, event).await
    }

    /// Sends one bounded Gate C request over the authenticated component
    /// channel and returns the component-owned JSON response.
    ///
    /// # Errors
    /// Only the Presentation Gateway may call the source-intake subjects. The
    /// operation fails closed when no component response arrives in five seconds.
    pub async fn request<T: Serialize>(
        &self,
        subject: &'static str,
        payload: &T,
    ) -> Result<Vec<u8>, BrokerError> {
        if self.workload_mode != WorkloadMode::PresentationGateway
            || !matches!(
                subject,
                SOURCE_INTAKE_COMMAND_SUBJECT | SOURCE_INTAKE_QUERY_SUBJECT
            )
        {
            return Err(BrokerError::ActionNotPermitted);
        }
        let bytes = serde_json::to_vec(payload).map_err(|_| BrokerError::PayloadInvalid)?;
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            self.client.request(subject, bytes.into()),
        )
        .await
        .map_err(|_| BrokerError::RequestTimedOut)?
        .map_err(|_| BrokerError::Unavailable)?;
        Ok(response.payload.to_vec())
    }

    #[must_use]
    pub fn can_publish(&self, subject: &str) -> bool {
        can_publish(self.workload_mode, subject)
    }
}

fn can_publish(mode: WorkloadMode, subject: &str) -> bool {
    match mode {
        WorkloadMode::ScenarioDirector => {
            subject == CUE_SUBJECT
                || subject == CONTROL_SUBJECT
                || subject == DIRECTOR_EVENT_SUBJECT
                || subject == GRANT_REQUEST_SUBJECT
                || subject == SYNTHETIC_TERMINATION_SUBJECT
                || subject == DIRECTOR_OPERATIONAL_SUBJECT
        }
        WorkloadMode::PresentationGateway => {
            subject == REGISTRATION_SUBJECT
                || subject == OUTCOME_SUBJECT
                || subject == CONTROL_OUTCOME_SUBJECT
                || subject == IDENTITY_OUTCOME_SUBJECT
                || subject == PRESENTATION_OPERATIONAL_SUBJECT
                || subject == SOURCE_INTAKE_COMMAND_SUBJECT
                || subject == SOURCE_INTAKE_QUERY_SUBJECT
        }
        WorkloadMode::IdentityBroker => {
            subject == SYNTHETIC_GRANT_SUBJECT
                || subject == IDENTITY_OUTCOME_SUBJECT
                || subject == IDENTITY_OPERATIONAL_SUBJECT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workload_publish_policy_is_separate() {
        assert!(can_publish(WorkloadMode::ScenarioDirector, CUE_SUBJECT));
        assert!(!can_publish(
            WorkloadMode::ScenarioDirector,
            OUTCOME_SUBJECT
        ));
        assert!(can_publish(
            WorkloadMode::PresentationGateway,
            REGISTRATION_SUBJECT
        ));
        assert!(can_publish(
            WorkloadMode::PresentationGateway,
            OUTCOME_SUBJECT
        ));
        assert!(!can_publish(WorkloadMode::PresentationGateway, CUE_SUBJECT));
        assert!(can_publish(
            WorkloadMode::PresentationGateway,
            SOURCE_INTAKE_COMMAND_SUBJECT
        ));
        assert!(CONTROL_SUBJECT.starts_with("ppl.m3.to-presentation."));
        assert!(CUE_SUBJECT.starts_with("ppl.m3.to-presentation."));
        assert!(can_publish(
            WorkloadMode::IdentityBroker,
            SYNTHETIC_GRANT_SUBJECT
        ));
        assert!(!can_publish(WorkloadMode::IdentityBroker, CUE_SUBJECT));
        assert!(can_publish(
            WorkloadMode::ScenarioDirector,
            DIRECTOR_OPERATIONAL_SUBJECT
        ));
        assert!(!can_publish(
            WorkloadMode::PresentationGateway,
            DIRECTOR_OPERATIONAL_SUBJECT
        ));
    }
}
