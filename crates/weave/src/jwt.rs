use anyhow::Result;
use jsonwebtoken as jwt;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde::{Deserialize, Serialize};

pub(crate) struct Issuer {
    encoding_key: jwt::EncodingKey,
    decoding_key: jwt::DecodingKey,
    header: jwt::Header,
    validation: jwt::Validation,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    iss: String,
    exp: u64,
}

const JWT_SUB: &str = "user";
const JWT_ISS: &str = "weave";

impl Issuer {
    pub(crate) fn new() -> Result<Self> {
        let key_pair = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
            .map_err(|_| anyhow::anyhow!("failed to generate key pair"))?;
        let encoding_key = jwt::EncodingKey::from_ed_der(key_pair.as_ref());

        let key_pair = Ed25519KeyPair::from_pkcs8(key_pair.as_ref())
            .map_err(|_| anyhow::anyhow!("failed to parse key pair"))?;
        let decoding_key = jwt::DecodingKey::from_ed_der(key_pair.public_key().as_ref());

        let header = jwt::Header::new(jwt::Algorithm::EdDSA);
        let validation = jwt::Validation::new(jwt::Algorithm::EdDSA);

        Ok(Self {
            encoding_key,
            decoding_key,
            header,
            validation,
        })
    }

    pub(crate) fn new_token(&self) -> String {
        let claims = Claims {
            sub: JWT_SUB.into(),
            iss: JWT_ISS.into(),
            exp: jwt::get_current_timestamp() + 60 * 60 * 24 * 30,
        };
        jsonwebtoken::encode(&self.header, &claims, &self.encoding_key).unwrap()
    }

    pub(crate) fn is_valid(&self, token: &str) -> bool {
        jwt::decode::<Claims>(token, &self.decoding_key, &self.validation)
            .ok()
            .map(|data| data.claims.sub == JWT_SUB && data.claims.iss == JWT_ISS)
            .unwrap_or(false)
    }
}
