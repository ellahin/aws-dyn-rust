use dotenvy::dotenv;

use reqwest::Client;

use std::env;

use serde_json;

use common_data::UpdateRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Cannot load .env file");

    let key = match env::var("KEY") {
        Ok(e) => e,
        Err(_) => panic!("cannot load var KEY"),
    };

    let secret = match env::var("SECRET") {
        Ok(e) => e,
        Err(_) => panic!("cannot load var SECRET"),
    };

    let url = match env::var("URL") {
        Ok(e) => e,
        Err(_) => panic!("cannot load var URL"),
    };
    let client = Client::new();

    let request = UpdateRequest {
        key: key,
        secret: secret,
    };

    match client
        .post(url)
        .body(serde_json::to_string(&request).unwrap())
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    Ok(())
}
