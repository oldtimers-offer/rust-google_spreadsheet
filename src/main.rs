use google_sheets4::{
    hyper, hyper_rustls,
    oauth2::{ServiceAccountAuthenticator, ServiceAccountKey},
    Sheets,
};
use reqwest::Client;
use serde_json::json;

use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load the service account key
    let service_account_key: ServiceAccountKey = serde_json::from_reader(std::fs::File::open(
        r"A:\Education\Rust\rust-spreadsheet\key\key.json",
    )?)?;

    // Create the authenticator using the service account key
    let auth = ServiceAccountAuthenticator::builder(service_account_key)
        .build()
        .await?;

    // Create an HTTPS connector with native root certificates
    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("Failed to load native roots")
        .https_or_http()
        .enable_http1()
        .build();

    // Create the hyper client with the HTTPS connector
    let client = hyper::Client::builder().build(https_connector);

    // Initialize Google Sheets API hub
    let hub = Sheets::new(client, auth);

    // The ID of the spreadsheet and the range of cells to read (A1:B, assuming first two rows)
    let spreadsheet_id = "1Mv-_8o51nWvpas2QROn6KE_2x4yreFe4-QyYaHPlA3Y";
    let range = "Sheet1!A1:B";

    // Read data from the spreadsheet
    let result = hub
        .spreadsheets()
        .values_get(spreadsheet_id, range)
        .doit()
        .await?;

    // Extract the values from the response
    if let Some(values) = result.1.values {
        // Convert the values into JSON
        let blog_posts: Vec<_> = values
            .into_iter()
            .map(|row| {
                let title = row.get(0).map_or("".to_string(), |v| v.to_string());
                let description = row.get(1).map_or("".to_string(), |v| v.to_string());
                json!({
                    "title": title,
                    "description": description,
                })
            })
            .collect();

        // Convert to JSON format
        let json_data = json!(blog_posts);

        // Send the data to the local API
        let api_url = "http://localhost:3000/data"; // Adjust this URL to match your local API
        let http_client = Client::new();

        let response = http_client
            .post(api_url)
            .json(&json_data) // Send the JSON data
            .send()
            .await?;

        if response.status().is_success() {
            println!("Data successfully sent to the local API.");
        } else {
            println!(
                "Failed to send data to the local API. Status: {}",
                response.status()
            );
        }
    } else {
        println!("No data found.");
    }

    Ok(())
}
