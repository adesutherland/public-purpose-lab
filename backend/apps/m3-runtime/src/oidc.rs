//! Managed external-human identity adapter for ADR-0014.

use std::{fmt, fs, path::PathBuf};

use openidconnect::{
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest,
};
use ppl_contracts::{AuthenticationStrength, ExternalHumanIdentityContext};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const FLOW_LIFETIME: Duration = Duration::minutes(10);

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub environment_id: String,
    pub audience: String,
    pub required_role: String,
    pub role_mapping_path: PathBuf,
    pub flow_state_path: PathBuf,
}

#[derive(Clone)]
pub struct OidcAuthenticator {
    config: OidcConfig,
    metadata: CoreProviderMetadata,
    http_client: reqwest::Client,
    mappings: RoleMappings,
}

pub struct OidcStart {
    pub authorisation_url: String,
    pub flow_cookie: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RoleMappings {
    mapping_version: String,
    identities: Vec<RoleMapping>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RoleMapping {
    issuer: String,
    subject_id: String,
    principal_id: String,
    roles: Vec<String>,
    audiences: Vec<String>,
    enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OidcError {
    Configuration,
    Discovery,
    StateUnavailable,
    StateRefused,
    ProviderRefused,
    TokenRefused,
    IdentityUnmapped,
    RandomUnavailable,
}

impl OidcError {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Configuration => "oidc-configuration-invalid",
            Self::Discovery => "oidc-provider-discovery-failed",
            Self::StateUnavailable => "oidc-flow-state-unavailable",
            Self::StateRefused => "oidc-flow-state-refused",
            Self::ProviderRefused => "oidc-provider-refused",
            Self::TokenRefused => "oidc-token-refused",
            Self::IdentityUnmapped => "external-identity-not-authorised",
            Self::RandomUnavailable => "operating-system-randomness-unavailable",
        }
    }
}

impl fmt::Display for OidcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for OidcError {}

impl OidcAuthenticator {
    pub async fn discover(config: OidcConfig) -> Result<Self, OidcError> {
        validate_config(&config)?;
        let mappings: RoleMappings = serde_json::from_slice(
            &fs::read(&config.role_mapping_path).map_err(|_| OidcError::Configuration)?,
        )
        .map_err(|_| OidcError::Configuration)?;
        validate_mappings(&mappings)?;
        prepare_flow_store(&config.flow_state_path)?;
        let issuer = IssuerUrl::new(config.issuer.clone()).map_err(|_| OidcError::Configuration)?;
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| OidcError::Configuration)?;
        let metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
            .await
            .map_err(|_| OidcError::Discovery)?;
        if metadata.issuer().as_str() != config.issuer {
            return Err(OidcError::Discovery);
        }
        Ok(Self {
            config,
            metadata,
            http_client,
            mappings,
        })
    }

    pub fn mapping_version(&self) -> &str {
        &self.mappings.mapping_version
    }

    pub fn begin(&self, now: OffsetDateTime) -> Result<OidcStart, OidcError> {
        let client = self.client()?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorisation_url, state, nonce) = client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_owned()))
            .add_scope(Scope::new("profile".to_owned()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        let flow_cookie = random_credential("oidc-flow")?;
        let expires_at = now + FLOW_LIFETIME;
        flow_connection(&self.config.flow_state_path)?
            .execute(
                "INSERT INTO oidc_flows(
                   flow_hash,state_hash,nonce,pkce_verifier,created_at,expires_at,consumed_at
                 ) VALUES(?1,?2,?3,?4,?5,?6,NULL)",
                params![
                    digest(&flow_cookie),
                    digest(state.secret()),
                    nonce.secret(),
                    pkce_verifier.secret(),
                    format_time(now)?,
                    format_time(expires_at)?,
                ],
            )
            .map_err(|_| OidcError::StateUnavailable)?;
        Ok(OidcStart {
            authorisation_url: authorisation_url.to_string(),
            flow_cookie,
        })
    }

    pub async fn complete(
        &self,
        flow_cookie: &str,
        returned_state: &str,
        code: &str,
        now: OffsetDateTime,
    ) -> Result<ExternalHumanIdentityContext, OidcError> {
        if flow_cookie.is_empty() || returned_state.is_empty() || code.is_empty() {
            return Err(OidcError::StateRefused);
        }
        let flow = consume_flow(
            &self.config.flow_state_path,
            flow_cookie,
            returned_state,
            now,
        )?;
        let client = self.client()?;
        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
            .request_async(&self.http_client)
            .await
            .map_err(|_| OidcError::ProviderRefused)?;
        let id_token = token_response.id_token().ok_or(OidcError::TokenRefused)?;
        let verifier = client.id_token_verifier();
        let claims = id_token
            .claims(&verifier, &Nonce::new(flow.nonce))
            .map_err(|_| OidcError::TokenRefused)?;
        if let Some(expected_hash) = claims.access_token_hash() {
            let actual_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                id_token
                    .signing_alg()
                    .map_err(|_| OidcError::TokenRefused)?,
                id_token
                    .signing_key(&verifier)
                    .map_err(|_| OidcError::TokenRefused)?,
            )
            .map_err(|_| OidcError::TokenRefused)?;
            if actual_hash != *expected_hash {
                return Err(OidcError::TokenRefused);
            }
        }
        let issuer = claims.issuer().as_str();
        let subject = claims.subject().as_str();
        let mapping = self
            .mappings
            .identities
            .iter()
            .find(|mapping| {
                mapping.enabled
                    && mapping.issuer == issuer
                    && mapping.subject_id == subject
                    && mapping.audiences.contains(&self.config.audience)
                    && mapping.roles.contains(&self.config.required_role)
            })
            .ok_or(OidcError::IdentityUnmapped)?;
        let token_expiry = OffsetDateTime::from_unix_timestamp(claims.expiration().timestamp())
            .map_err(|_| OidcError::TokenRefused)?;
        Ok(ExternalHumanIdentityContext {
            contract_id: "I-001".to_owned(),
            contract_version: "1.0.0".to_owned(),
            context_id: format!("external-context-{}", Uuid::new_v4()),
            environment_id: self.config.environment_id.clone(),
            issuer: issuer.to_owned(),
            subject_id: subject.to_owned(),
            principal_id: mapping.principal_id.clone(),
            roles: mapping.roles.clone(),
            audience: self.config.audience.clone(),
            authentication_strength: AuthenticationStrength::SingleFactor,
            mapping_version: self.mappings.mapping_version.clone(),
            issued_at: format_time(now)?,
            expires_at: format_time(token_expiry.min(now + Duration::minutes(30)))?,
            decision_reference: Some(format!(
                "oidc-role-map-{}",
                digest_prefix(&self.mappings.mapping_version)
            )),
        })
    }

    fn client(
        &self,
    ) -> Result<
        CoreClient<
            EndpointSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointSet,
            EndpointMaybeSet,
        >,
        OidcError,
    > {
        let token_endpoint = self
            .metadata
            .token_endpoint()
            .cloned()
            .ok_or(OidcError::Discovery)?;
        Ok(CoreClient::from_provider_metadata(
            self.metadata.clone(),
            ClientId::new(self.config.client_id.clone()),
            Some(ClientSecret::new(self.config.client_secret.clone())),
        )
        .set_token_uri(token_endpoint)
        .set_redirect_uri(
            RedirectUrl::new(self.config.redirect_uri.clone())
                .map_err(|_| OidcError::Configuration)?,
        ))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct StoredFlow {
    nonce: String,
    pkce_verifier: String,
}

fn consume_flow(
    path: &PathBuf,
    flow_cookie: &str,
    returned_state: &str,
    now: OffsetDateTime,
) -> Result<StoredFlow, OidcError> {
    let mut connection = flow_connection(path)?;
    let transaction = connection
        .transaction()
        .map_err(|_| OidcError::StateUnavailable)?;
    let flow = transaction
        .query_row(
            "SELECT nonce,pkce_verifier,expires_at,consumed_at,state_hash
             FROM oidc_flows WHERE flow_hash=?1",
            params![digest(flow_cookie)],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()
        .map_err(|_| OidcError::StateUnavailable)?
        .ok_or(OidcError::StateRefused)?;
    if flow.3.is_some()
        || parse_time(&flow.2)? <= now
        || !constant_time_equal(&flow.4, &digest(returned_state))
    {
        return Err(OidcError::StateRefused);
    }
    transaction
        .execute(
            "UPDATE oidc_flows SET consumed_at=?1 WHERE flow_hash=?2 AND consumed_at IS NULL",
            params![format_time(now)?, digest(flow_cookie)],
        )
        .map_err(|_| OidcError::StateUnavailable)?;
    transaction
        .commit()
        .map_err(|_| OidcError::StateUnavailable)?;
    Ok(StoredFlow {
        nonce: flow.0,
        pkce_verifier: flow.1,
    })
}

fn validate_config(config: &OidcConfig) -> Result<(), OidcError> {
    if config.issuer != "https://accounts.google.com"
        || config.client_id.is_empty()
        || config.client_secret.is_empty()
        || !config.redirect_uri.starts_with("https://")
        || config.environment_id.is_empty()
        || config.audience.is_empty()
        || config.required_role.is_empty()
    {
        return Err(OidcError::Configuration);
    }
    Ok(())
}

fn validate_mappings(mappings: &RoleMappings) -> Result<(), OidcError> {
    if mappings.mapping_version.is_empty()
        || mappings.identities.iter().any(|mapping| {
            mapping.issuer != "https://accounts.google.com"
                || mapping.subject_id.is_empty()
                || mapping.principal_id.is_empty()
                || mapping.roles.is_empty()
                || mapping.audiences.is_empty()
        })
    {
        return Err(OidcError::Configuration);
    }
    Ok(())
}

fn prepare_flow_store(path: &PathBuf) -> Result<(), OidcError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| OidcError::StateUnavailable)?;
    }
    flow_connection(path)?
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS oidc_flows (
               flow_hash TEXT PRIMARY KEY,
               state_hash TEXT NOT NULL,
               nonce TEXT NOT NULL,
               pkce_verifier TEXT NOT NULL,
               created_at TEXT NOT NULL,
               expires_at TEXT NOT NULL,
               consumed_at TEXT
             );",
        )
        .map_err(|_| OidcError::StateUnavailable)
}

fn flow_connection(path: &PathBuf) -> Result<Connection, OidcError> {
    let connection = Connection::open(path).map_err(|_| OidcError::StateUnavailable)?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|_| OidcError::StateUnavailable)?;
    connection
        .pragma_update(None, "synchronous", "FULL")
        .map_err(|_| OidcError::StateUnavailable)?;
    Ok(connection)
}

fn random_credential(prefix: &str) -> Result<String, OidcError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| OidcError::RandomUnavailable)?;
    Ok(format!("{prefix}-{}", hex(&bytes)))
}

fn digest(value: &str) -> String {
    hex(Sha256::digest(value.as_bytes()).as_slice())
}

fn digest_prefix(value: &str) -> String {
    digest(value)[..16].to_owned()
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

fn parse_time(value: &str) -> Result<OffsetDateTime, OidcError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| OidcError::StateUnavailable)
}

fn format_time(value: OffsetDateTime) -> Result<String, OidcError> {
    value
        .format(&Rfc3339)
        .map_err(|_| OidcError::StateUnavailable)
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
    use tempfile::tempdir;

    #[test]
    fn flow_state_is_single_use_and_state_bound() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("oidc.sqlite");
        prepare_flow_store(&path).expect("store");
        let now = OffsetDateTime::parse("2030-01-01T09:00:00Z", &Rfc3339).expect("time");
        flow_connection(&path)
            .expect("connection")
            .execute(
                "INSERT INTO oidc_flows VALUES(?1,?2,?3,?4,?5,?6,NULL)",
                params![
                    digest("cookie"),
                    digest("state"),
                    "nonce",
                    "pkce",
                    format_time(now).expect("time"),
                    format_time(now + FLOW_LIFETIME).expect("time")
                ],
            )
            .expect("flow");
        assert_eq!(
            consume_flow(&path, "cookie", "wrong", now),
            Err(OidcError::StateRefused)
        );
        let flow = consume_flow(&path, "cookie", "state", now).expect("accepted");
        assert_eq!(flow.nonce, "nonce");
        assert_eq!(
            consume_flow(&path, "cookie", "state", now),
            Err(OidcError::StateRefused)
        );
    }
}
