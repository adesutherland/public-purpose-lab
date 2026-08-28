//! Cloud KMS Ed25519 signer used only by the managed identity-broker profile.

use ed25519_dalek::{VerifyingKey, pkcs8::DecodePublicKey};
use google_cloud_kms_v1::{
    client::KeyManagementService,
    model::{
        AsymmetricSignResponse, CryptoKeyVersion, ProtectionLevel, PublicKey,
        crypto_key_version::{CryptoKeyVersionAlgorithm, CryptoKeyVersionState},
    },
};
use sha2::{Digest, Sha256};

use crate::AppError;

#[derive(Clone)]
pub struct KmsEd25519Signer {
    client: KeyManagementService,
    key_version: String,
}

impl KmsEd25519Signer {
    pub async fn new(
        key_version: String,
        project_id: &str,
        expected_public_key: &str,
        expected_fingerprint: &str,
    ) -> Result<Self, AppError> {
        let expected_prefix = format!("projects/{project_id}/locations/");
        if !key_version.starts_with(&expected_prefix)
            || !key_version.contains("/keyRings/")
            || !key_version.contains("/cryptoKeys/")
            || !key_version.contains("/cryptoKeyVersions/")
        {
            return Err(AppError::configuration("kms-key-version-invalid"));
        }
        let client = KeyManagementService::builder()
            .build()
            .await
            .map_err(|_| AppError::configuration("kms-client-unavailable"))?;
        let version = client
            .get_crypto_key_version()
            .set_name(&key_version)
            .send()
            .await
            .map_err(|_| AppError::configuration("kms-key-version-unavailable"))?;
        let public_key = client
            .get_public_key()
            .set_name(&key_version)
            .send()
            .await
            .map_err(|_| AppError::configuration("kms-public-key-unavailable"))?;
        validate_key_binding(
            &version,
            &public_key,
            &key_version,
            expected_public_key,
            expected_fingerprint,
        )?;
        Ok(Self {
            client,
            key_version,
        })
    }

    pub async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, AppError> {
        let checksum = i64::from(crc32c::crc32c(payload));
        let response = self
            .client
            .asymmetric_sign()
            .set_name(&self.key_version)
            .set_data(payload.to_vec())
            .set_data_crc32c(checksum)
            .send()
            .await
            .map_err(|_| AppError::configuration("kms-signing-unavailable"))?;
        validate_response(&response, &self.key_version)?;
        Ok(response.signature.to_vec())
    }
}

fn validate_key_binding(
    version: &CryptoKeyVersion,
    public_key: &PublicKey,
    key_version: &str,
    expected_public_key: &str,
    expected_fingerprint: &str,
) -> Result<(), AppError> {
    if version.name != key_version
        || version.state != CryptoKeyVersionState::Enabled
        || version.algorithm != CryptoKeyVersionAlgorithm::EcSignEd25519
        || version.protection_level != ProtectionLevel::Software
        || public_key.name != key_version
        || public_key.algorithm != CryptoKeyVersionAlgorithm::EcSignEd25519
        || public_key.protection_level != ProtectionLevel::Software
    {
        return Err(AppError::configuration("kms-key-binding-refused"));
    }
    let returned_checksum = public_key
        .pem_crc32c
        .ok_or(AppError::configuration("kms-key-binding-refused"))?;
    if returned_checksum != i64::from(crc32c::crc32c(public_key.pem.as_bytes())) {
        return Err(AppError::configuration("kms-key-binding-refused"));
    }
    let verifying_key = VerifyingKey::from_public_key_pem(&public_key.pem)
        .map_err(|_| AppError::configuration("kms-key-binding-refused"))?;
    let raw_public_key = verifying_key.to_bytes();
    let public_key_hex = hex(&raw_public_key);
    let fingerprint = format!("sha256:{}", hex(&Sha256::digest(raw_public_key)));
    if public_key_hex != expected_public_key || fingerprint != expected_fingerprint {
        return Err(AppError::configuration("kms-key-binding-refused"));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn validate_response(response: &AsymmetricSignResponse, key_version: &str) -> Result<(), AppError> {
    if !response.verified_data_crc32c
        || response.name != key_version
        || response.protection_level != ProtectionLevel::Software
        || response.signature.len() != 64
    {
        return Err(AppError::configuration("kms-signing-response-refused"));
    }
    let returned_checksum = response
        .signature_crc32c
        .ok_or(AppError::configuration("kms-signing-response-refused"))?;
    if returned_checksum != i64::from(crc32c::crc32c(&response.signature)) {
        return Err(AppError::configuration("kms-signing-response-refused"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, pkcs8::EncodePublicKey};
    use pkcs8::LineEnding;

    const KEY_VERSION: &str =
        "projects/p/locations/global/keyRings/r/cryptoKeys/k/cryptoKeyVersions/1";

    #[test]
    fn refuses_wrong_key_or_integrity_metadata() {
        let response = AsymmetricSignResponse::new()
            .set_name(KEY_VERSION)
            .set_signature(vec![0_u8; 64])
            .set_signature_crc32c(i64::from(crc32c::crc32c(&[0_u8; 64])))
            .set_verified_data_crc32c(true)
            .set_protection_level(ProtectionLevel::Software);
        assert!(validate_response(&response, "wrong-key").is_err());
        assert!(validate_response(&response, KEY_VERSION).is_ok());
    }

    #[test]
    fn accepts_only_exact_enabled_ed25519_public_trust_binding() {
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .expect("public key PEM");
        let public_bytes = verifying_key.to_bytes();
        let version = CryptoKeyVersion::new()
            .set_name(KEY_VERSION)
            .set_state(CryptoKeyVersionState::Enabled)
            .set_algorithm(CryptoKeyVersionAlgorithm::EcSignEd25519)
            .set_protection_level(ProtectionLevel::Software);
        let public_key = PublicKey::new()
            .set_name(KEY_VERSION)
            .set_pem(&pem)
            .set_pem_crc32c(i64::from(crc32c::crc32c(pem.as_bytes())))
            .set_algorithm(CryptoKeyVersionAlgorithm::EcSignEd25519)
            .set_protection_level(ProtectionLevel::Software);
        let expected_hex = hex(&public_bytes);
        let expected_fingerprint = format!("sha256:{}", hex(&Sha256::digest(public_bytes)));

        assert!(
            validate_key_binding(
                &version,
                &public_key,
                KEY_VERSION,
                &expected_hex,
                &expected_fingerprint
            )
            .is_ok()
        );
        assert!(
            validate_key_binding(
                &version.clone().set_state(CryptoKeyVersionState::Disabled),
                &public_key,
                KEY_VERSION,
                &expected_hex,
                &expected_fingerprint
            )
            .is_err()
        );
        assert!(
            validate_key_binding(
                &version,
                &public_key,
                KEY_VERSION,
                &hex(&[0_u8; 32]),
                &expected_fingerprint
            )
            .is_err()
        );
    }
}
