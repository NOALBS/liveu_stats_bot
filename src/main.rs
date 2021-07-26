use anyhow::{Context, Result};
use liveu_stats_bot::{config::Config, liveu::Liveu, liveu_monitor::Monitor, twitch::Twitch};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Started liveu stats bot v{}", env!("CARGO_PKG_VERSION"));

    let config = match Config::load("config.json") {
        Ok(c) => c,
        Err(_) => Config::ask_for_settings().await?,
    };

    println!("Liveu: Authenticating...");
    let liveu = Liveu::authenticate(config.liveu.clone())
        .await
        .context("Failed to authenticate. Are your login details correct?")?;
    println!("Liveu: Authenticated");

    let liveu_boss_id = if let Some(boss_id) = &config.liveu.id {
        boss_id.to_owned()
    } else {
        let inventories = liveu
            .get_inventories()
            .await
            .context("Error getting inventories")?;
        let loc = Liveu::get_boss_id_location(&inventories);
        inventories.units[loc].id.to_owned()
    };

    println!("\nTwitch: Connecting...");
    let (twitch_client, twitch_join_handle) =
        Twitch::run(config.clone(), liveu.clone(), liveu_boss_id.to_owned());
    println!("Twitch: Connected");

    if config.liveu.monitor {
        println!("Liveu: Running liveu monitor");
        let monitor = Monitor {
            client: twitch_client.clone(),
            config: config.clone(),
            liveu: liveu.clone(),
            boss_id: liveu_boss_id.to_owned(),
        };
        monitor.run();
    }

    twitch_join_handle.await?;

    Ok(())
}
