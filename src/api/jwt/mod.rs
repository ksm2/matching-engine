use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

mod alg;
mod base64;

pub use alg::*;

type Result<T> = std::result::Result<T, JwtError>;

#[derive(Debug, PartialEq, Eq)]
pub struct Jwt {
    pub header: JwtHeader,
    pub payload: JwtPayload,
}

impl Jwt {
    pub fn new(algorithm: Algorithm, subject: String) -> Self {
        let header = JwtHeader::new(algorithm);
        let payload = JwtPayload { subject };
        Self { header, payload }
    }

    pub fn decode(token: &str, secret: &[u8]) -> Result<Self> {
        let split = token.splitn(3, '.').collect::<Vec<_>>();
        let header = base64::decode(split[0]).ok_or(JwtError::Base64Decode)?;
        let header = serde_json::from_slice::<JwtHeader>(&header)?;
        let payload = base64::decode(split[1]).ok_or(JwtError::Base64Decode)?;
        let payload = serde_json::from_slice::<JwtPayload>(&payload)?;

        let jwt = Self { header, payload };
        if !jwt.verify(secret, split[2].as_bytes()) {
            return Err(JwtError::Signature);
        }

        Ok(jwt)
    }

    pub fn encode(&self, secret: &[u8]) -> Result<String> {
        let body = self.body();
        let signature = self.header.algorithm.encode(body.as_bytes(), secret);
        Ok(body + "." + &signature)
    }

    pub fn verify(&self, secret: &[u8], expected_signature: &[u8]) -> bool {
        let body = self.body();
        let actual_signature = self.header.algorithm.encode(body.as_bytes(), secret);
        actual_signature.as_bytes() == expected_signature
    }

    fn body(&self) -> String {
        let header = base64::encode(&serde_json::to_vec(&self.header).unwrap());
        let payload = base64::encode(&serde_json::to_vec(&self.payload).unwrap());
        header + "." + &payload
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct JwtHeader {
    #[serde(rename = "typ")]
    jwt_type: JwtType,
    #[serde(rename = "alg")]
    pub algorithm: Algorithm,
}

impl JwtHeader {
    pub fn new(algorithm: Algorithm) -> Self {
        Self {
            jwt_type: JwtType::default(),
            algorithm,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub enum JwtType {
    #[default]
    #[serde(rename = "JWT")]
    Jwt,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct JwtPayload {
    #[serde(rename = "sub")]
    pub subject: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum JwtError {
    Base64Decode,
    JsonDecode,
    Signature,
}

impl Display for JwtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            JwtError::Base64Decode => "JwtError: failed to decode Base64",
            JwtError::JsonDecode => "JwtError: failed to decode JSON",
            JwtError::Signature => "JwtError: signature did not match",
        };

        f.write_str(msg)
    }
}

impl Error for JwtError {}

impl<E> From<E> for JwtError
where
    E: serde::de::Error,
{
    fn from(_value: E) -> Self {
        JwtError::JsonDecode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_decode_a_token() {
        let expected = Jwt {
            header: JwtHeader::new(Algorithm::HmacSha256),
            payload: JwtPayload {
                subject: "1234567890".into(),
            },
        };
        let actual = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.2M5F2Dsert2X3RPSAnrornOiSDp8gZonEpF1g1P_v-k";

        assert_eq!(Ok(expected), Jwt::decode(actual, b"secret"));
    }

    #[test]
    fn it_should_encode_a_token() {
        let expected = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.2M5F2Dsert2X3RPSAnrornOiSDp8gZonEpF1g1P_v-k";
        let actual = Jwt {
            header: JwtHeader::new(Algorithm::HmacSha256),
            payload: JwtPayload {
                subject: "1234567890".into(),
            },
        };

        assert_eq!(Ok(expected.to_string()), actual.encode(b"secret"));
    }
}
