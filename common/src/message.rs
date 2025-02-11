use bytes::BytesMut;
use core::fmt;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use thiserror::Error;
use zstd::stream::Decoder;
use zstd::stream::Encoder;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    #[serde(alias = "ping")]
    PING,

    #[serde(alias = "dump")]
    DUMP { file: String },

    #[serde(alias = "get")]
    GET { uri: String },

    #[serde(alias = "delete")]
    DELETE { uri: String },

    #[serde(alias = "post")]
    POST { uri: String, body: Value },

    #[serde(alias = "put")]
    PUT { uri: String, body: Value },

    #[serde(alias = "patch")]
    PATCH { uri: String, body: Value },
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("Missing Command")]
    NoCommandFound,
    #[error("Missing Body")]
    MissingBody,
    #[error("Missing URI")]
    MissingUri,
    #[error("Invalid command: {0}")]
    BodyParseFailed(serde_json::Error),
    #[error("Invalid format")]
    InvalidFormat,
}
#[derive(Debug, Error)]
#[error("failed to deserialize {0}")]
pub struct DeserializeError(Box<dyn std::error::Error>);

#[derive(Debug, Error)]
#[error("failed to serialize {0}")]
pub struct SerializeError(Box<dyn std::error::Error>);

impl Command {
    pub fn try_new(string: &str) -> Result<Self, CommandParseError> {
        let (head, tail) = string.split_once('\n').unwrap_or((string, ""));
        let (verb, uri) = head.trim().split_once(' ').unwrap_or((head, ""));
        match verb.trim().to_lowercase().as_str() {
            "ping" => Ok(Self::PING),
            "get" => {
                if uri == "" {
                    Err(CommandParseError::MissingUri)
                } else {
                    Ok(Self::GET {
                        uri: uri.to_string(),
                    })
                }
            }
            "dump" => {
                if uri == "" {
                    Err(CommandParseError::MissingUri)
                } else {
                    Ok(Self::DUMP {
                        file: uri.to_string(),
                    })
                }
            }
            "delete" => {
                if uri == "" {
                    Err(CommandParseError::MissingUri)
                } else {
                    Ok(Self::DELETE {
                        uri: uri.to_string(),
                    })
                }
            }
            "post" => {
                let (head, _tail) = tail.split_once('\n').unwrap_or((tail, ""));
                let body = Self::parse_body(head)?;
                Ok(Self::POST {
                    uri: uri.to_string(),
                    body,
                })
            }
            "put" => {
                let (head, _tail) = tail.split_once('\n').unwrap_or((tail, ""));
                let body = Self::parse_body(head)?;
                Ok(Self::PUT {
                    uri: uri.to_string(),
                    body,
                })
            }
            "patch" => {
                let (head, _tail) = tail.split_once('\n').unwrap_or((tail, ""));
                let body = Self::parse_body(head)?;
                Ok(Self::PATCH {
                    uri: uri.to_string(),
                    body,
                })
            }
            _ => Err(CommandParseError::NoCommandFound),
        }
    }

    fn parse_body(string: &str) -> Result<Value, CommandParseError> {
        let (body, value) = string.trim().split_once(' ').unwrap_or((string, ""));
        if body.to_lowercase().as_str() != "body" {
            Err(CommandParseError::MissingBody)
        } else {
            serde_json::from_str(value).map_err(CommandParseError::BodyParseFailed)
        }
    }

    pub fn from_slice(input: &BytesMut) -> Result<Self, DeserializeError> {
        let mut decoder = Decoder::new(&input[..]).map_err(|e| DeserializeError(e.into()))?;
        let mut decompressed_data = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .map_err(|e| DeserializeError(e.into()))?;
        from_slice(&decompressed_data).map_err(|e| DeserializeError(e.into()))
    }

    pub fn to_vec(val: &Self) -> Result<Vec<u8>, SerializeError> {
        let vec = &to_vec(val).map_err(|e| SerializeError(e.into()))?;
        let mut encoder = Encoder::new(Vec::new(), 3).map_err(|e| SerializeError(e.into()))?;
        encoder
            .write_all(vec)
            .map_err(|e| SerializeError(e.into()))?;
        encoder.finish().map_err(|e| SerializeError(e.into()))
    }
}

impl FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Command::try_new(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    #[serde(alias = "pong")]
    PONG,

    #[serde(alias = "id")]
    ID(String),

    #[serde(alias = "object")]
    OBJECT(Value),

    #[serde(alias = "collection")]
    COLLECTION(Vec<Value>),

    #[serde(alias = "null")]
    NULL,

    #[serde(alias = "error")]
    ERROR(String),

    #[serde(alias = "ok")]
    OK
}

impl Response {
    pub fn from_slice(input: &BytesMut) -> Result<Self, DeserializeError> {
        let mut decoder = Decoder::new(&input[..]).map_err(|e| DeserializeError(e.into()))?;
        let mut decompressed_data = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .map_err(|e| DeserializeError(e.into()))?;
        from_slice(&decompressed_data).map_err(|e| DeserializeError(e.into()))
    }

    pub fn to_vec(val: &Self) -> Result<Vec<u8>, SerializeError> {
        let vec = &to_vec(val).map_err(|e| SerializeError(e.into()))?;
        let mut encoder = Encoder::new(Vec::new(), 3).map_err(|e| SerializeError(e.into()))?;
        encoder
            .write_all(vec)
            .map_err(|e| SerializeError(e.into()))?;
        encoder.finish().map_err(|e| SerializeError(e.into()))
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::NULL => write!(f, "{}", "null"),
            Response::PONG => write!(f, "{}", "pong"),
            Response::OK => write!(f,"{}","ok"),
            Response::OBJECT(value) => write!(f, "{}", print_value(value)),
            Response::COLLECTION(values) => {
                let mut res = "[".to_string();
                for (i, v) in values.iter().enumerate() {
                    res.push_str(&print_value(v));
                    if i < values.len() - 1 {
                        res.push_str(", ");
                    }
                }
                res.push_str("]");
                write!(f, "{}", res)
            }
            _ => {
                write!(f, "{}", serde_json::to_string(self).unwrap())
            }
        }
    }
}

fn print_value(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.clone()),
        Value::Array(a) => {
            let mut res = "[".to_string();
            for (i, v) in a.iter().enumerate() {
                res.push_str(&print_value(v));
                if i < a.len() - 1 {
                    res.push_str(", ");
                }
            }
            res.push_str("]");
            res
        }
        Value::Object(o) => {
            let mut res = "{".to_string();
            for (i, (k, v)) in o.iter().enumerate() {
                res.push_str(&format!("\"{}\": {}", k, print_value(v)));
                if i < o.len() - 1 {
                    res.push_str(", ");
                }
            }
            res.push_str("}");
            res
        }
    }
}
