use error::Error;
use read_input::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::error;

const CONFIG_FILE_NAME: &str = "config.json";

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rtmp {
    pub url: String,
    pub application: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub liveu: Liveu,
    pub twitch: Twitch,
    pub rtmp: Option<Rtmp>,
}

impl Config {
    /// Loads the config
    pub fn load<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let config = fs::read_to_string(path)?;
        Ok(serde_json::from_str::<Config>(&config)?)
    }

    /// Asks the user to enter settings and save it to disk
    pub fn ask_for_settings() -> Result<Self, Error> {
        println!("Please enter your Liveu details below");
        let liveu = Liveu {
            email: input().msg("Email: ").get(),
            password: input().msg("Password: ").get(), // FIXME: Change password input?
        };

        println!("\nPlease enter your Twitch details below");
        let mut twitch = Twitch {
            bot_username: input().msg("Bot username: ").get(),
            bot_oauth: input()
                .msg("(You can generate an Oauth here: https://twitchapps.com/tmi/)\nBot oauth: ")
                .get(),
            channel: input().msg("Channel name: ").get(),
            commands: vec![
                "!lustats".to_string(),
                "!liveustats".to_string(),
                "!lus".to_string(),
            ],
            command_cooldown: input()
                .msg("Command cooldown (seconds): ")
                .err("Please enter a number")
                .get(),
        };

        if let Some(oauth) = twitch.bot_oauth.strip_prefix("oauth:") {
            twitch.bot_oauth = oauth.to_string();
        }

        let q: String = input()
            .msg("Are you using nginx and would you like to display its bitrate as well (y/n): ")
            .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
            .err("Please enter y or n: ")
            .get();

        let mut rtmp = None;

        if q == "y" {
            rtmp = Some(Rtmp {
                url: input().msg("Please enter the stats page URL: ").get(),
                application: input().msg("Application name: ").get(),
                key: input().msg("Stream key: ").get(),
            });
        }

        let config = Config {
            liveu,
            twitch,
            rtmp,
        };
        fs::write(CONFIG_FILE_NAME, serde_json::to_string_pretty(&config)?)?;

        print!("\x1B[2J");

        let mut path = std::env::current_dir()?;
        path.push(CONFIG_FILE_NAME);
        println!(
            "Saved settings to {} in {}",
            CONFIG_FILE_NAME,
            path.display()
        );

        Ok(config)
    }
}
