use std::error::Error;

use reqwest;
use serde_derive::{Deserialize, Serialize};
extern crate dotenv;
use dotenv::dotenv;
use std::env;
use meilisearch_sdk::*;


#[derive(Debug, Deserialize, Serialize)]
struct Record {
    instructor: String,
    song: String,
    artist: String,
    code: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let MEILISEARCH_URL = env::var("MEILISEARCH_URL").expect("MEILISEARCH_URL must be set");
    let MEILISEARCH_API_KEY = env::var("MEILISEARCH_API_KEY").expect("MEILISEARCH_API_KEY must be set");
    // Use the database_url to set up your database connection

    // Create a client (without sending any request so that can't fail)
    let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));

    let songs = match client
        .get_index("songs")
        .await {
            Ok(index) => index,
            Err(error) => {
                println!("No index found, creating one");
                client.create_index("songs", Some("code")).await?;
                client.get_index("songs").await?
            }
        };

    // An index is where the documents are stored.

    for record in get_google_sheet().await? {
        println!("{:?}", record);
        songs.add_documents(&[record], Some("code")).await?;
    }
    Ok(())
}

async fn get_google_sheet() -> Result<impl Iterator<Item = Record>, Box<dyn Error>> {
    //Result<Vec<Vec<String>>, Error> {
    // VRD public spreadsheet. Adding "/export?format=csv" to the end of the URL will return the sheet as a CSV.
    // let data = reqwest::get("https://docs.google.com/spreadsheets/d/14Nh9M1r__S-BHS00j6hi0otF4A63LXhoX9ISFZv7nrs/export?format=csv").await?.bytes_stream().await?;
    let data = reqwest::get("https://docs.google.com/spreadsheets/d/14Nh9M1r__S-BHS00j6hi0otF4A63LXhoX9ISFZv7nrs/export?format=csv").await?.text().await?;
    // async fn get_google_sheet() -> Result<Vec<Record>, Box<dyn Error>> {
    //     let url = "https://docs.google.com/spreadsheets/d/14Nh9M1r__S-BHS00j6hi0otF4A63LXhoX9ISFZv7nrs/export?format=csv";
    //     let data = reqwest::get(url).await?.text().await?;

    // We need to skip the first "10" lines because the First record looks like 10 lines to data.lines()

    let data_without_first_line = data.lines().skip(10).collect::<Vec<_>>().join("\n");

    // println!("{:?}", data_without_first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .escape(Some(b'\\'))
        .from_reader(data_without_first_line.as_bytes());

    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records.into_iter())
}
