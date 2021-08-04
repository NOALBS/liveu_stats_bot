use error::Error;
use read_input::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::{error, liveu};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Liveu {
    pub email: String,
    pub password: String,
    pub id: Option<String>,
    pub monitor: Monitor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub battery: bool,
    pub battery_notification: Vec<u8>,
    pub modems: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Twitch {
    pub bot_username: String,
    pub bot_oauth: String,
    pub channel: String,
    pub admin_users: Option<Vec<String>>,
    pub mod_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Commands {
    pub command_cooldown: u16,
    pub stats: Vec<String>,
    pub battery: Vec<String>,
    pub start: String,
    pub stop: String,
    pub restart: String,
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
    pub commands: Commands,
    pub rtmp: Option<Rtmp>,
    pub custom_port_names: Option<CustomUnitNames>,
}

impl Config {
    /// Loads the config
    pub fn load<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let file = fs::read_to_string(path)?;
        let mut config = serde_json::from_str::<Config>(&file)?;
        Self::lowercase_settings(&mut config);

        println!("{:#?}", config);
        Ok(config)
    }

    /// Lowercase settings which should always be lowercase
    pub fn lowercase_settings(config: &mut Config) {
        let Twitch {
            bot_username,
            bot_oauth,
            channel,
            admin_users,
            ..
        } = &mut config.twitch;

        *channel = channel.to_lowercase();
        *bot_oauth = bot_oauth.to_lowercase();
        *bot_username = bot_username.to_lowercase();

        if let Some(admin_users) = admin_users {
            for user in admin_users {
                *user = user.to_lowercase();
            }
        }
    }

    /// Asks the user to enter settings and save it to disk
    pub async fn ask_for_settings() -> Result<Self, Error> {
        println!("Please enter your Liveu details below");

        let email = input().msg("Email: ").get();
        let password = input().msg("Password: ").get(); // FIXME: Change password input?
        let monitor_enabled = input_to_bool(&input()
            .msg("\nDo you want to receive automatic chat messages about\nthe status of your battery or modems (Y/n): ")
            .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
            .err("Please enter y or n: ")
            .default("y".to_string())
            .get());

        let monitor = Monitor {
            battery: monitor_enabled,
            battery_notification: [99, 50, 10, 5, 1].to_vec(),
            modems: monitor_enabled,
        };

        let mut liveu = Liveu {
            email,
            password,
            id: None,
            monitor,
        };

        let lauth = liveu::Liveu::authenticate(liveu.clone()).await?;
        let inventories = lauth.get_inventories().await?;

        if inventories.units.len() > 1 {
            let option = input_to_bool(
                &input()
                    .msg("Do you want to save a default unit to use in the config (y/N): ")
                    .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
                    .err("Please enter y or n: ")
                    .default("n".to_string())
                    .get(),
            );

            if option {
                let loc = liveu::Liveu::get_boss_id_location(&inventories);
                liveu.id = Some(inventories.units[loc].id.to_owned());
            }
        }

        println!("\nPlease enter your Twitch details below");
        let twitch = Twitch {
            bot_username: input().msg("Bot username: ").get(),
            bot_oauth: input()
                .msg("(You can generate an Oauth here: https://twitchapps.com/tmi/)\nBot oauth: ")
                .get(),
            channel: input().msg("Channel name: ").get(),
            admin_users: None,
            mod_only: input_to_bool(
                &input()
                    .msg("Only allow mods to access the commands (Y/n): ")
                    .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
                    .err("Please enter y or n: ")
                    .default("y".to_string())
                    .get(),
            ),
        };

        let commands = Commands {
            command_cooldown: input()
                .msg("Command cooldown (default 5 seconds): ")
                .err("Please enter a number")
                .default(5)
                .get(),
            stats: vec![
                "!lustats".to_string(),
                "!liveustats".to_string(),
                "!lus".to_string(),
            ],
            battery: vec![
                "!battery".to_string(),
                "!liveubattery".to_string(),
                "!lub".to_string(),
            ],
            start: "!lustart".to_string(),
            stop: "!lustop".to_string(),
            restart: "!lurestart".to_string(),
        };

        let q: String = input()
            .msg("\nAre you using nginx and would you like to display its bitrate as well (y/N): ")
            .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
            .err("Please enter y or n: ")
            .default("n".to_string())
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
            .msg("\nWould you like to use a custom name for each port? (y/N): ")
            .add_test(|x: &String| x.to_lowercase() == "y" || x.to_lowercase() == "n")
            .err("Please enter y or n: ")
            .default("n".to_string())
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

        let mut config = Config {
            liveu,
            twitch,
            commands,
            rtmp,
            custom_port_names: custom_unit_names,
        };
        fs::write(CONFIG_FILE_NAME, serde_json::to_string_pretty(&config)?)?;

        // FIXME: Does not work on windows
        print!("\x1B[2J");

        let mut path = std::env::current_dir()?;
        path.push(CONFIG_FILE_NAME);
        println!(
            "Saved settings to {} in {}",
            CONFIG_FILE_NAME,
            path.display()
        );

        Self::lowercase_settings(&mut config);

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
