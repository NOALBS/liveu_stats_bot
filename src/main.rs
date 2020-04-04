mod config;
mod liveu;

use config::Config;
use liveu::Liveu;
use tokio::prelude::*;

#[tokio::main]
async fn main() {
    let config = match Config::new() {
        Ok(d) => d,
        Err(e) => panic!(e),
    };

    let api = match Liveu::authenticate(config.liveu).await {
        Ok(d) => d,
        Err(e) => panic!(e),
    };
    println!("Liveu: {:#?}", api);

    let inventories = match api.get_inventories().await {
        Ok(d) => d,
        Err(e) => panic!(e),
    };
    println!("Inventories: {:#?}", inventories.units.unwrap().len());
}
