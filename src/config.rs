use read_input::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Json error: {0}")]
    Json(#[from] serde_json::error::Error),

    #[error("Error writing file: {0}")]
    Write(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub liveu: Liveu,
    pub twitch: Twitch,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Liveu {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Twitch {
    pub bot_username: String,
    pub bot_oauth: String,
    pub channel: String,
    pub commands: Vec<String>,
    pub command_cooldown: u16,
}

// FIXME: Ask if input is correct
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        match fs::read_to_string("config.json") {
            Ok(d) => Ok(serde_json::from_str::<Config>(&d)?),
            Err(_) => {
                println!("Please enter your Liveu details below");
                let liveu = Liveu {
                    email: input().msg("Email: ").get(),
                    password: input().msg("Password: ").get(), // FIXME: Change password input?
                };

                println!("\nPlease enter your Twitch details below");
                let twitch = Twitch {
                    bot_username: input().msg("Bot username: ").get(),
                    bot_oauth: input().msg("Bot oauth: ").get(),
                    channel: input().msg("Channel name: ").get(),
                    commands: vec![
                        // Use default commands?
                        "!lustats".to_string(),
                        "!liveustats".to_string(),
                        "!lus".to_string(),
                    ],
                    command_cooldown: input()
                        .msg("Command cooldown (seconds): ")
                        .err("Please enter a number")
                        .get(),
                };

                let config = Config { liveu, twitch };
                fs::write("config.json", serde_json::to_string_pretty(&config)?)?;

                print!("\x1B[2J");
                println!("\nSaved settings to config.json");

                Ok(config)
            }
        }
    }
}
