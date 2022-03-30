// extern crate discord_lib;
use discord_lib::discord::run;
// use futures::executor::block_on;
// use reqwest::Client;
mod exmaple;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Running Program\n");
    let client = reqwest::Client::new();
    run(&client).await?;
    print!("Ending Program\n");

    let mut d : exmaple::Ex;

    Ok(())
}
