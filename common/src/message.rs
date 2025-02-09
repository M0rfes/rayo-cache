use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] 
pub enum Command {
    #[serde(alias = "PING")]
    Ping,
    #[serde(alias = "SET")]
    Set(String,Value),
}

#[derive(Debug, Error)]
#[error("Invalid command: {0}")]
pub struct CommandParseError(String);

impl FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Convert the input to lowercase and trim any surrounding whitespace.
        let ss = s.trim().to_lowercase();
        let command:Vec<&str> = ss.split(" ").collect();
        match &command[..]  {
            ["ping"] => Ok(Command::Ping),
            ["set",key,value] => {
                let Ok(v)= serde_json::from_str(value) else {
                    return  Err(CommandParseError(value.to_string()));
                };
                Ok(Command::Set(key.to_string(),v))
            },
            _ => Err(CommandParseError(s.to_string())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] 
pub enum Response {
     #[serde(alias = "PONG")]
    Pong,

    #[serde(alias = "OK")]
    Ok,
}