use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{create_dir, File};
use std::io::copy;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct Emoji {
    ok: bool,
    emoji: HashMap<String, String>,
}

#[derive(Debug)]
struct EmojisError(String);

impl fmt::Display for EmojisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for EmojisError {}

const ENDPOINT: &str = "https://slack.com/api/emoji.list";
const SLACK_API_TOKEN: &str = "SLACK_API_TOKEN";
const DOWNLOAD_FOLDER: &str = "downloads";

fn get_emojis(token: String) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut resp = client
        .get(ENDPOINT)
        .header("Authorization", format!("Bearer {}", token))
        .send();

    match resp {
        Ok(ref mut r) => {
            let emoji: Emoji = r.json().expect("Error converting JSON");
            Ok(emoji.emoji)
        }
        Err(_) => Result::Err(Box::new(EmojisError(
            "Unable to download emoji list".into(),
        ))),
    }
}

fn download_emoji(name: &str, url: &str, path: &Path) -> Result<u64, std::io::Error> {
    let mut response = reqwest::get(url).expect("Unable to download emoji image");
    let extension = url.split('.').last().unwrap();
    let filename = format!("{}.{}", name, extension);
    let mut dest = File::create(path.join(filename)).unwrap();

    copy(&mut response, &mut dest)
}

fn main() {
    let token = env::var(SLACK_API_TOKEN).expect("Environment variable SLACK_API_TOKEN isn't set");
    let emojis_list = get_emojis(token).expect("Unable to download emoji list");

    let _ = create_dir(DOWNLOAD_FOLDER);
    let path = Path::new(DOWNLOAD_FOLDER);

    let count_of_emojis = emojis_list.len() as u64;
    let progress_bar = ProgressBar::new(count_of_emojis);

    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );

    for (name, url) in &emojis_list {
        progress_bar.inc(1);
        if url.starts_with("alias:") {
            continue;
        }
        match download_emoji(name, url, path) {
            Err(e) => println!("Unable to download {} due to {}", name, e),
            _ => progress_bar.set_message(name),
        }
    }

    progress_bar.finish_with_message("done");
}
