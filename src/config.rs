use error::Error;
use read_input::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::{error, liveu};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Liveu {
    pub email: String,
    pub password: String,
    pub monitor: bool,
    pub battery_notification: Vec<u8>,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Twitch {
    pub bot_username: String,
    pub bot_oauth: String,
    pub channel: String,
    pub commands: Vec<String>,
    pub battery_command: Vec<String>,
    pub start_command: String,
    pub stop_command: String,
    pub restart_command: String,
    pub admin_users: Option<Vec<String>>,
    pub command_cooldown: u16,
    pub mod_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rtmp {
    pub url: String,
    pub application: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub liveu: Liveu,
    pub twitch: Twitch,
    pub rtmp: Option<Rtmp>,
    pub custom_port_names: Option<CustomUnitNames>,
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
    pub async fn ask_for_settings() -> Result<Self, Error> {
        println!("Please enter your Liveu details below");
        let mut liveu = Liveu {
            email: input().msg("Email: ").get(),
            password: input().msg("Password: ").get(), // FIXME: Change password input?
            monitor: input_to_bool(&input()
            .msg("\nDo you want to receive automatic chat messages about\nthe status of your battery or modems (y/n): ")
            .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
            .err("Please enter y or n: ")
            .get()),
            id: None,
            battery_notification: [99, 50, 10, 5, 1].to_vec(),
        };

        let lauth = liveu::Liveu::authenticate(liveu.clone()).await?;
        let inventories = lauth.get_inventories().await?;

        if inventories.units.len() > 1 {
            let option = input_to_bool(
                &input()
                    .msg("Do you want to save a default unit to use in the config (y/n): ")
                    .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
                    .err("Please enter y or n: ")
                    .get(),
            );

            if option {
                let loc = liveu::Liveu::get_boss_id_location(&inventories);
                liveu.id = Some(inventories.units[loc].id.to_owned());
            }
        }

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
            battery_command: vec![
                "!battery".to_string(),
                "!liveubattery".to_string(),
                "!lub".to_string(),
            ],
            start_command: "!lustart".to_string(),
            stop_command: "!lustop".to_string(),
            restart_command: "!lurestart".to_string(),
            admin_users: None,
            command_cooldown: input()
                .msg("Command cooldown (seconds): ")
                .err("Please enter a number")
                .get(),
            mod_only: input_to_bool(
                &input()
                    .msg("Only allow mods to access the commands (y/n): ")
                    .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
                    .err("Please enter y or n: ")
                    .get(),
            ),
        };

        if let Some(oauth) = twitch.bot_oauth.strip_prefix("oauth:") {
            twitch.bot_oauth = oauth.to_string();
        }

        let q: String = input()
            .msg("\nAre you using nginx and would you like to display its bitrate as well (y/n): ")
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

        let q: String = input()
            .msg("\nWould you like to use a custom name for each port? (y/n): ")
            .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
            .err("Please enter y or n: ")
            .get();

        let mut custom_unit_names = None;

        if q == "y" {
            println!("Press enter to keep using the default value");

            let mut un = CustomUnitNames::default();

            un.ethernet = input().msg("Ethernet: ").default(un.ethernet).get();
            un.wifi = input().msg("WiFi: ").default(un.wifi).get();
            un.usb1 = input().msg("USB1: ").default(un.usb1).get();
            un.usb2 = input().msg("USB2: ").default(un.usb2).get();

            custom_unit_names = Some(un);
        }

        let config = Config {
            liveu,
            twitch,
            rtmp,
            custom_port_names: custom_unit_names,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomUnitNames {
    pub ethernet: String,
    pub wifi: String,
    pub usb1: String,
    pub usb2: String,
}

impl Default for CustomUnitNames {
    fn default() -> Self {
        CustomUnitNames {
            ethernet: "ETH".to_string(),
            wifi: "WiFi".to_string(),
            usb1: "USB1".to_string(),
            usb2: "USB2".to_string(),
        }
    }
}

/// Converts y or n to bool.
fn input_to_bool(confirm: &str) -> bool {
    if confirm == "y" {
        return true;
    }

    false
}
