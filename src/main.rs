use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::{create_dir, File};
use std::io::copy;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct Emoji {
    ok: bool,
    emoji: HashMap<String, String>,
}

const ENDPOINT: &str = "https://slack.com/api/emoji.list";
const SLACK_API_TOKEN: &str = "SLACK_API_TOKEN";

fn get_emojis(token: String) -> Result<HashMap<String, String>, Box<std::error::Error>> {
    let client = reqwest::Client::new();

    let mut resp = client
        .get(ENDPOINT)
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    let emoji: Emoji = resp.json().expect("Error!");

    Ok(emoji.emoji)
}

fn download_emoji(name: &str, url: &str, path: &Path) -> Result<u64, std::io::Error> {
    let mut response = reqwest::get(url).unwrap();

    let extension = url.split('.').last().unwrap();
    let filename = format!("{}.{}", name, extension);

    let mut dest = File::create(path.join(filename)).unwrap();

    copy(&mut response, &mut dest)
}

fn main() {
    let token = env::var(SLACK_API_TOKEN).unwrap();

    let emojis_list = get_emojis(token).expect("Unable to download emoji list");
    let _ = create_dir("downloads");
    let path = Path::new("downloads");
    for (name, url) in &emojis_list {
        if url.starts_with("alias:") {
            continue;
        }
        match download_emoji(name, url, path) {
            Err(e) => println!("Unable to download {} due to {}", name, e),
            _ => println!("Downloaded {}", name),
        }
    }
}
