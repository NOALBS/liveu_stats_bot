mod liveu;

use tokio::prelude::*;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let api = liveu::Liveu::authenticate("email", "password").await;
    println!("{:#?}", api);
}
