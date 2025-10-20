use crate::tool::{Output, Tool};
use anyhow::{Context, Result, bail};
use clap::{Command, CommandFactory, Parser, Subcommand, ValueEnum};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode_header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(name = "jwt", about = "JWT (JSON Web Token) utilities")]
pub struct JwtTool {
    #[command(subcommand)]
    command: JwtCommand,
}

#[derive(Subcommand, Debug)]
enum JwtCommand {
    /// Decode a JWT without verification (inspect only)
    Decode {
        /// JWT token to decode
        token: String,
    },
    /// Encode and sign a JWT
    Encode {
        /// JSON payload for the JWT (must be valid JSON)
        #[arg(short, long)]
        payload: String,

        /// Secret key for signing (for HMAC algorithms)
        #[arg(short, long)]
        secret: String,

        /// Algorithm to use for signing
        #[arg(short, long, value_enum, default_value = "hs256")]
        algorithm: JwtAlgorithm,

        /// Issuer claim (iss)
        #[arg(long)]
        issuer: Option<String>,

        /// Subject claim (sub)
        #[arg(long)]
        subject: Option<String>,

        /// Audience claim (aud)
        #[arg(long)]
        audience: Option<String>,

        /// Expiration time in seconds from now (exp)
        #[arg(long)]
        expires_in: Option<i64>,
    },
    /// Verify and decode a JWT
    Verify {
        /// JWT token to verify
        token: String,

        /// Secret key for verification (for HMAC algorithms)
        #[arg(short, long)]
        secret: String,

        /// Algorithm to use for verification
        #[arg(short, long, value_enum, default_value = "hs256")]
        algorithm: JwtAlgorithm,

        /// Expected issuer (iss)
        #[arg(long)]
        issuer: Option<String>,

        /// Expected subject (sub)
        #[arg(long)]
        subject: Option<String>,

        /// Expected audience (aud)
        #[arg(long)]
        audience: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum JwtAlgorithm {
    /// HMAC using SHA-256
    HS256,
    /// HMAC using SHA-384
    HS384,
    /// HMAC using SHA-512
    HS512,
}

impl From<JwtAlgorithm> for Algorithm {
    fn from(alg: JwtAlgorithm) -> Self {
        match alg {
            JwtAlgorithm::HS256 => Algorithm::HS256,
            JwtAlgorithm::HS384 => Algorithm::HS384,
            JwtAlgorithm::HS512 => Algorithm::HS512,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    #[serde(flatten)]
    custom: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iat: Option<i64>,
}

impl Tool for JwtTool {
    fn cli() -> Command {
        JwtTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        match &self.command {
            JwtCommand::Decode { token } => decode_jwt(token),
            JwtCommand::Encode {
                payload,
                secret,
                algorithm,
                issuer,
                subject,
                audience,
                expires_in,
            } => encode_jwt(
                payload,
                secret,
                *algorithm,
                issuer.clone(),
                subject.clone(),
                audience.clone(),
                *expires_in,
            ),
            JwtCommand::Verify {
                token,
                secret,
                algorithm,
                issuer,
                subject,
                audience,
            } => verify_jwt(
                token,
                secret,
                *algorithm,
                issuer.clone(),
                subject.clone(),
                audience.clone(),
            ),
        }
    }
}

fn decode_jwt(token: &str) -> Result<Option<Output>> {
    // Split the token to check if it's valid format
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        bail!("Invalid JWT format. Expected 3 parts separated by dots");
    }

    // Decode header
    let header = decode_header(token).context("Failed to decode JWT header")?;

    // Decode payload without verification using a validation that doesn't validate signature
    let mut validation = Validation::new(header.alg);
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.validate_aud = false;
    validation.required_spec_claims.clear();  // Don't require any standard claims
    
    // Use an empty key since we're not validating the signature
    let token_data = jsonwebtoken::decode::<Value>(
        token,
        &DecodingKey::from_secret(&[]),
        &validation,
    ).context("Failed to decode JWT payload")?;

    let result = json!({
        "header": {
            "alg": format!("{:?}", header.alg),
            "typ": header.typ.unwrap_or_else(|| "JWT".to_string()),
        },
        "payload": token_data.claims,
        "signature": parts[2],
        "note": "Token decoded without verification"
    });

    Ok(Some(Output::JsonValue(result)))
}

fn encode_jwt(
    payload: &str,
    secret: &str,
    algorithm: JwtAlgorithm,
    issuer: Option<String>,
    subject: Option<String>,
    audience: Option<String>,
    expires_in: Option<i64>,
) -> Result<Option<Output>> {
    // Parse the payload as JSON
    let custom_payload: Value = serde_json::from_str(payload)
        .context("Invalid JSON payload. Please provide valid JSON")?;

    // Get current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Build claims
    let claims = Claims {
        custom: custom_payload,
        iss: issuer,
        sub: subject,
        aud: audience,
        exp: expires_in.map(|exp| now + exp),
        iat: Some(now),
    };

    // Create header
    let header = Header::new(algorithm.into());

    // Encode token
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to encode JWT")?;

    Ok(Some(Output::JsonValue(json!(token))))
}

fn verify_jwt(
    token: &str,
    secret: &str,
    algorithm: JwtAlgorithm,
    issuer: Option<String>,
    subject: Option<String>,
    audience: Option<String>,
) -> Result<Option<Output>> {
    // Configure validation
    let mut validation = Validation::new(algorithm.into());

    // Set optional validations
    if let Some(iss) = issuer {
        validation.set_issuer(&[iss]);
    } else {
        validation.validate_exp = true;
        validation.validate_nbf = false;
        validation.iss = None;
    }

    if let Some(sub) = subject {
        validation.sub = Some(sub);
    }

    if let Some(aud) = audience {
        validation.set_audience(&[aud]);
    } else {
        validation.validate_aud = false;
    }

    // Decode and verify
    match jsonwebtoken::decode::<Value>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            let result = json!({
                "valid": true,
                "header": {
                    "alg": format!("{:?}", token_data.header.alg),
                    "typ": token_data.header.typ.unwrap_or_else(|| "JWT".to_string()),
                },
                "payload": token_data.claims,
            });
            Ok(Some(Output::JsonValue(result)))
        }
        Err(err) => {
            let result = json!({
                "valid": false,
                "error": err.to_string(),
            });
            Ok(Some(Output::JsonValue(result)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple() {
        let tool = JwtTool {
            command: JwtCommand::Encode {
                payload: r#"{"user":"alice"}"#.to_string(),
                secret: "my-secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: None,
                subject: None,
                audience: None,
                expires_in: None,
            },
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let token = val.as_str().unwrap();
        assert!(token.contains('.'));
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_encode_with_claims() {
        let tool = JwtTool {
            command: JwtCommand::Encode {
                payload: r#"{"user":"bob"}"#.to_string(),
                secret: "secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: Some("test-issuer".to_string()),
                subject: Some("test-subject".to_string()),
                audience: Some("test-audience".to_string()),
                expires_in: Some(3600),
            },
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let token = val.as_str().unwrap();
        assert!(token.contains('.'));
    }

    #[test]
    fn test_encode_invalid_json() {
        let tool = JwtTool {
            command: JwtCommand::Encode {
                payload: "not-json".to_string(),
                secret: "secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: None,
                subject: None,
                audience: None,
                expires_in: None,
            },
        };

        let result = tool.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_decode_valid_token() {
        // First encode a token
        let encode_tool = JwtTool {
            command: JwtCommand::Encode {
                payload: r#"{"user":"charlie","role":"admin"}"#.to_string(),
                secret: "my-secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: None,
                subject: None,
                audience: None,
                expires_in: None,
            },
        };

        let encode_result = encode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = encode_result else {
            panic!("Expected JsonValue output");
        };
        let token = val.as_str().unwrap();

        // Now decode it
        let decode_tool = JwtTool {
            command: JwtCommand::Decode {
                token: token.to_string(),
            },
        };

        let decode_result = decode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(decoded) = decode_result else {
            panic!("Expected JsonValue output");
        };

        assert_eq!(decoded["payload"]["user"], "charlie");
        assert_eq!(decoded["payload"]["role"], "admin");
        assert!(decoded["header"]["alg"].as_str().is_some());
    }

    #[test]
    fn test_decode_invalid_token() {
        let tool = JwtTool {
            command: JwtCommand::Decode {
                token: "invalid.token".to_string(),
            },
        };

        let result = tool.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_valid_token() {
        // First encode a token
        let encode_tool = JwtTool {
            command: JwtCommand::Encode {
                payload: r#"{"user":"dave"}"#.to_string(),
                secret: "verify-secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: Some("my-issuer".to_string()),
                subject: None,
                audience: None,
                expires_in: Some(3600),
            },
        };

        let encode_result = encode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = encode_result else {
            panic!("Expected JsonValue output");
        };
        let token = val.as_str().unwrap();

        // Verify it with correct secret and issuer
        let verify_tool = JwtTool {
            command: JwtCommand::Verify {
                token: token.to_string(),
                secret: "verify-secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: Some("my-issuer".to_string()),
                subject: None,
                audience: None,
            },
        };

        let verify_result = verify_tool.execute().unwrap().unwrap();
        let Output::JsonValue(verified) = verify_result else {
            panic!("Expected JsonValue output");
        };

        assert_eq!(verified["valid"], true);
        assert_eq!(verified["payload"]["user"], "dave");
    }

    #[test]
    fn test_verify_wrong_secret() {
        // Encode with one secret
        let encode_tool = JwtTool {
            command: JwtCommand::Encode {
                payload: r#"{"user":"eve"}"#.to_string(),
                secret: "correct-secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: None,
                subject: None,
                audience: None,
                expires_in: Some(3600),
            },
        };

        let encode_result = encode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = encode_result else {
            panic!("Expected JsonValue output");
        };
        let token = val.as_str().unwrap();

        // Verify with wrong secret
        let verify_tool = JwtTool {
            command: JwtCommand::Verify {
                token: token.to_string(),
                secret: "wrong-secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: None,
                subject: None,
                audience: None,
            },
        };

        let verify_result = verify_tool.execute().unwrap().unwrap();
        let Output::JsonValue(verified) = verify_result else {
            panic!("Expected JsonValue output");
        };

        assert_eq!(verified["valid"], false);
        assert!(verified["error"].as_str().is_some());
    }

    #[test]
    fn test_verify_wrong_issuer() {
        // Encode with specific issuer
        let encode_tool = JwtTool {
            command: JwtCommand::Encode {
                payload: r#"{"data":"test"}"#.to_string(),
                secret: "secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: Some("correct-issuer".to_string()),
                subject: None,
                audience: None,
                expires_in: Some(3600),
            },
        };

        let encode_result = encode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = encode_result else {
            panic!("Expected JsonValue output");
        };
        let token = val.as_str().unwrap();

        // Verify with wrong issuer
        let verify_tool = JwtTool {
            command: JwtCommand::Verify {
                token: token.to_string(),
                secret: "secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: Some("wrong-issuer".to_string()),
                subject: None,
                audience: None,
            },
        };

        let verify_result = verify_tool.execute().unwrap().unwrap();
        let Output::JsonValue(verified) = verify_result else {
            panic!("Expected JsonValue output");
        };

        assert_eq!(verified["valid"], false);
    }

    #[test]
    fn test_different_algorithms() {
        for algorithm in [JwtAlgorithm::HS256, JwtAlgorithm::HS384, JwtAlgorithm::HS512] {
            let encode_tool = JwtTool {
                command: JwtCommand::Encode {
                    payload: r#"{"test":"data"}"#.to_string(),
                    secret: "secret".to_string(),
                    algorithm,
                    issuer: None,
                    subject: None,
                    audience: None,
                    expires_in: None,
                },
            };

            let result = encode_tool.execute().unwrap().unwrap();
            let Output::JsonValue(val) = result else {
                panic!("Expected JsonValue output");
            };

            let token = val.as_str().unwrap();
            assert_eq!(token.split('.').count(), 3);
        }
    }

    #[test]
    fn test_encode_complex_payload() {
        let complex_payload = r#"{
            "user": "alice",
            "roles": ["admin", "user"],
            "metadata": {
                "age": 30,
                "active": true
            }
        }"#;

        let tool = JwtTool {
            command: JwtCommand::Encode {
                payload: complex_payload.to_string(),
                secret: "secret".to_string(),
                algorithm: JwtAlgorithm::HS256,
                issuer: None,
                subject: None,
                audience: None,
                expires_in: None,
            },
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let token = val.as_str().unwrap();
        
        // Decode to verify structure
        let decode_tool = JwtTool {
            command: JwtCommand::Decode {
                token: token.to_string(),
            },
        };

        let decode_result = decode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(decoded) = decode_result else {
            panic!("Expected JsonValue output");
        };

        assert_eq!(decoded["payload"]["user"], "alice");
        assert!(decoded["payload"]["roles"].is_array());
        assert!(decoded["payload"]["metadata"].is_object());
    }
}

