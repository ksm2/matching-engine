use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha384, Sha512};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    #[serde(rename = "HS256")]
    HmacSha256,
    #[serde(rename = "HS384")]
    HmacSha384,
    #[serde(rename = "HS512")]
    HmacSha512,
}

impl Algorithm {
    pub fn encode(&self, body: &[u8], secret: &[u8]) -> String {
        match self {
            Algorithm::HmacSha256 => {
                let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("Create encoder");
                mac.update(body);
                let result = mac.finalize();
                super::base64::encode(&result.into_bytes())
            }
            Algorithm::HmacSha384 => {
                let mut mac = Hmac::<Sha384>::new_from_slice(secret).expect("Create encoder");
                mac.update(body);
                let result = mac.finalize();
                super::base64::encode(&result.into_bytes())
            }
            Algorithm::HmacSha512 => {
                let mut mac = Hmac::<Sha512>::new_from_slice(secret).expect("Create encoder");
                mac.update(body);
                let result = mac.finalize();
                super::base64::encode(&result.into_bytes())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_hs256() {
        let algorithm = Algorithm::HmacSha256;
        assert_eq!(
            "c0zGLzKEFWj0VxWuufTXiRMk5tlI5MbGDAYhzaxIYjo".to_string(),
            algorithm.encode(b"hello world", b"secret")
        );
    }

    #[test]
    fn it_encodes_hs384() {
        let algorithm = Algorithm::HmacSha384;
        assert_eq!(
            "LaO7F3uSqumMOrInJ9f2DJBb4br_cftLAKbkEJI-ZVg3ZZDB-vki_1HsSb53QJrG".to_string(),
            algorithm.encode(b"hello world", b"secret")
        );
    }

    #[test]
    fn it_encodes_hs512() {
        let algorithm = Algorithm::HmacSha512;
        assert_eq!(
            "bTIjmwHdF1BVchFikxPZXk9Py47lF-RDmQrBr8dWK_10_6YRg4fv2eFo_4bR2lzvSlXtxjzEuiicTDqLT3vfwg".to_string(),
            algorithm.encode(b"hello world", b"secret")
        );
    }
}
