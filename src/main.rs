use anyhow::{Context, Result};
use liveu_stats_botv2::{config::Config, liveu::Liveu, twitch::Twitch};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Started liveu stats bot");

    let config = match Config::load("config.json") {
        Ok(c) => c,
        Err(_) => Config::ask_for_settings()?,
    };

    println!("Liveu: Authenticating...");
    let liveu = Liveu::authenticate(config.liveu.clone())
        .await
        .context("Failed to authenticate. Are your login details correct?")?;
    println!("Liveu: Authenticated");

    let inventories = liveu
        .get_inventories()
        .await
        .context("Error getting inventories")?;
    let loc = liveu_stats_botv2::liveu::Liveu::get_boss_id_location(&inventories);
    let liveu_boss_id = inventories.units[loc].id.to_owned();

    println!("\nTwitch: Connecting...");
    let twitch_chat = Twitch::run(config, liveu, liveu_boss_id);
    println!("Twitch: Connected");

    twitch_chat.await?;

    Ok(())
}
