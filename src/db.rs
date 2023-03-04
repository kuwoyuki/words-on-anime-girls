use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::{BufWriter, Write};

const PATH: &str = "/var/lib/words_on_anime_girls/db.json";

#[derive(Serialize, Deserialize, Debug)]
// TODO: server_id key?
pub struct WordsOnAnimeGirls {
    pub last_listing: String,
    pub oldest_listing: String,
}

impl WordsOnAnimeGirls {
    pub fn ensure_exists() -> () {
        let exists = std::path::Path::new(&PATH).exists();
        if !exists {
            fs::write(&PATH, r#"{"last_listing":"","oldest_listing":""}"#)
                .expect("Unable to create db.json");
        }
    }
    pub fn read() -> WordsOnAnimeGirls {
        let data = fs::read_to_string(&PATH).unwrap();
        serde_json::from_str::<WordsOnAnimeGirls>(&data).unwrap()
    }

    pub fn write(&self) -> () {
        let file = fs::File::create(&PATH).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, self).unwrap();
        writer.flush().unwrap();
    }
}
