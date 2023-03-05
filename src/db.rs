use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::{BufWriter, Write};

const PATH: &str = "/var/lib/words_on_anime_girls/db.json";

#[derive(Clone, Serialize, Deserialize, Debug)]
// TODO: server_id key?
pub struct ServerData {
    pub webhook_url: String,
    pub last_listing: String,
    pub interval: u64,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WordsOnAnimeGirls {
    pub servers: Vec<ServerData>,
    pub oldest_listing: String,
    pub oldest_url: String,
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

    // pub fn update_last_listing(&self, webhook_url: String, name: String) -> Self {
    //     let mut db = self.clone();
    //     for srv in db.servers.iter_mut() {
    //         if srv.webhook_url != webhook_url {
    //             continue;
    //         }
    //         srv.last_listing = name;
    //         break;
    //     }
    //     db
    // }

    // pub fn update_old_listing(&self, name: String, url: String) -> Self {
    //     let mut db = self.clone();
    //     db.oldest_listing = name;
    //     db.oldest_url = url;
    //     db
    // }
}
