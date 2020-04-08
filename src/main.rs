#![deny(dead_code, unused_imports, clippy::clone_on_copy)]

mod config;
mod liveu;
mod twitch;

use config::Config;
use liveu::Liveu;

use anyhow::{Context, Result};
use read_input::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;

    println!("Liveu: Authenticating...");
    let api = Liveu::authenticate(config.liveu)
        .await
        .context("Failed to authenticate. Are your login details correct?")?;
    println!("Liveu: Authenticated");
    //let keep_me_alive = api.refresh_token();
    api.refresh_token();

    let inventories = api
        .get_inventories()
        .await
        .context("Error getting inventories")?;
    let boss_id: usize = get_id(&inventories);

    //let _ = keep_me_alive.await;

    println!("\nTwitch: Connecting...");
    let mut client = twitch::Twitch::connect(config.twitch).await?;
    println!("Twitch: Connected");

    let channel = client.config.channel.to_owned();
    let cooldown = client.config.command_cooldown.to_owned() as u64;
    let is_timeout = Arc::new(AtomicBool::new(false));

    while let Some(msg) = &client.read.recv().await {
        if client.config.commands.contains(msg) && !is_timeout.load(Ordering::Acquire) {
            let t = is_timeout.clone();

            tokio::spawn(async move {
                t.store(true, Ordering::Release);
                tokio::time::delay_for(std::time::Duration::from_secs(cooldown)).await;
                t.store(false, Ordering::Release);
            });

            let interfaces: Vec<liveu::Interface> = api
                .get_unit(&inventories.units[boss_id].id)
                .await
                .context("Error getting unit")?
                .into_iter()
                .filter(|x| x.connected)
                .map(|mut x| {
                    match x.port.as_ref() {
                        "eth0" => {
                            x.port = "Ethernet".to_string();
                        }
                        "wlan0" => {
                            x.port = "WiFi".to_string();
                        }
                        "2" => {
                            x.port = "USB1".to_string();
                        }
                        "3" => {
                            x.port = "USB2".to_string();
                        }
                        _ => {}
                    }
                    x
                })
                .collect();

            //println!("{:#?}", interfaces);
            if interfaces.is_empty() {
                client.send_message(&channel, "LiveU Offline :(").await?;
                continue;
            }

            let mut message: String = "(MODEMS) ".to_string();
            let mut total = 0;

            for (pos, interface) in interfaces.iter().enumerate() {
                let separator = if pos == interfaces.len() - 1 {
                    ""
                } else {
                    ", "
                };

                message = format!(
                    "{}{}: {} Kbps{}",
                    message, interface.port, interface.uplink_kbps, separator
                )
                .to_owned();
                total += interface.uplink_kbps;
            }

            if total == 0 {
                client
                    .send_message(&channel, "LiveU Online and Ready")
                    .await?;
                continue;
            }

            client.send_message(&channel, &message).await?;
            client
                .send_message(
                    &channel,
                    &format!("(TOTAL BITRATE) LiveU to LRT: {} Kbps", total),
                )
                .await?;
        }
    }

    Ok(())
}

fn get_id(inventories: &liveu::Inventories) -> usize {
    let size = inventories.units.len();

    if size == 0 {
        panic!("No units found!");
    }

    if size > 1 {
        println!("Found {} units!\n", size);

        for (pos, unit) in inventories.units.iter().enumerate() {
            println!("({}) {}", pos + 1, unit.reg_code);
        }

        let inp = input()
            .msg("\nPlease enter which one you want to use: ")
            .inside_err(
                1..=size,
                format!("Please enter a number between 1 and {}: ", size),
            )
            .err("That does not look like a number. Please try again:")
            .get();

        return inp - 1;
    }

    0
}
