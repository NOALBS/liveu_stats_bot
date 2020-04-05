mod config;
mod liveu;

use config::Config;
use liveu::Liveu;

use read_input::prelude::*;

#[tokio::main]
async fn main() {
    let config = match Config::new() {
        Ok(d) => d,
        Err(e) => panic!(e),
    };

    let api = match Liveu::authenticate(config.liveu).await {
        Ok(d) => d,
        Err(e) => panic!("Error authenticating: {}", e),
    };

    let keep_me_alive = api.refresh_token();

    let inventories = match api.get_inventories().await {
        Ok(d) => d,
        Err(e) => panic!("Error get inventories: {}", e),
    };

    let size = inventories.units.len();
    let id: usize = {
        if size == 1 {
            0
        } else if size > 1 {
            println!("Found {} units!\n", size);

            for (pos, unit) in inventories.units.iter().enumerate() {
                println!("({}) {}", pos + 1, unit.reg_code);
            }

            let inp = input()
                .msg("\nPlease enter which one you want to use: ")
                .inside_err(1..=size, format!("That does not look like a correct number. Please enter a number from 1 to {}. Please try again:", size))
                .err("That does not look like a number. Please try again:")
                .get();

            inp - 1
        } else {
            panic!("No units found!");
        }
    };

    let stats = match api.get_unit(&inventories.units[id].id).await {
        Ok(d) => d,
        Err(e) => panic!("Error get unit: {}", e),
    };
    println!("Stats: {:#?}", stats);

    let _ = keep_me_alive.await;
}
