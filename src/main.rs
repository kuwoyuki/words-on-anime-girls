mod db;
mod reddit;
mod webhook;

use db::WordsOnAnimeGirls;
use reddit::{ListingError, RedditClient, RedditListingChildData};
use reqwest::Response;
use std::env;
use std::error;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::{task, time};

async fn find_oldest_listing(
    reddit_client: &RedditClient,
) -> Result<RedditListingChildData, ListingError> {
    let mut after: Option<String> = None;
    let mut oldest_listing: Option<RedditListingChildData> = None;

    loop {
        // TODO: expect err
        let listings = reddit_client
            .get_listings("100", after.as_deref(), None)
            .await?;

        oldest_listing = match listings.last() {
            None => break,
            Some(x) => Some(x.data.clone()),
        };
        after = Some(oldest_listing.as_ref().unwrap().name.to_owned());
    }
    return match oldest_listing {
        None => Err(ListingError::NoListing),
        Some(x) => Ok(x),
    };
}

async fn get_newer(
    reddit_client: &RedditClient,
    before: &str,
) -> Result<RedditListingChildData, ListingError> {
    let newer_listing = reddit_client.get_listings("1", None, Some(before)).await?;
    return match newer_listing.first() {
        None => Err(ListingError::NoListing),
        Some(x) => Ok(x.data.clone()),
    };
}

async fn send_anime_girl(hook_url: &str, img_url: &str) -> reqwest::Result<Response> {
    let mut hook = webhook::DiscordWebHook::new(hook_url, img_url);
    hook.set_username("Anime Girls");
    hook.set_avatar_url("https://awau.moe/qj372zX.jpg");
    hook.fire().await
}

#[derive(Debug)]
struct Msg {
    webhook_url: String,
    name: String,
}
#[derive(Debug)]
enum MsgTypes {
    Old(String, String),
    New(Msg),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    WordsOnAnimeGirls::ensure_exists();
    // todo: config or sth
    let client_id = env::var("CLIENT_ID").expect("$CLIENT_ID is not set");
    let client_secret = env::var("CLIENT_SECRET").expect("$CLIENT_SECRET is not set");

    let db_lock = Arc::new(RwLock::new(WordsOnAnimeGirls::read()));
    let c_lock = Arc::clone(&db_lock);
    let (tx, mut rx) = mpsc::channel(2);

    let reddit_client = RedditClient::new(&client_id, &client_secret).await?;

    let db = db_lock.read().unwrap();
    if db.oldest_listing == "" {
        let listing = find_oldest_listing(&reddit_client)
            .await
            .expect("failed to find the oldest listing, can't continue.");
        tx.send(MsgTypes::Old(listing.name, listing.url))
            .await
            .unwrap();
    }
    drop(db);

    task::spawn_blocking(move || {
        while let Some(lst) = rx.blocking_recv() {
            let mut db = db_lock.write().unwrap();
            match lst {
                MsgTypes::Old(name, url) => {
                    db.oldest_listing = name;
                    db.oldest_url = url;
                }
                MsgTypes::New(Msg { webhook_url, name }) => {
                    for srv in db.servers.iter_mut() {
                        if srv.webhook_url != webhook_url {
                            continue;
                        }
                        srv.last_listing = name.to_owned();
                    }
                }
            }
            db.write();
            println!("db written: {:?}", db);
        }
    });

    let db = c_lock.read().unwrap();
    let servers = db.servers.clone();
    drop(db);

    let handles: Vec<_> = servers
        .into_iter()
        .map(|srv| {
            let reddit_client = reddit_client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(srv.interval));
                loop {
                    interval.tick().await;

                    println!("{:?}", srv);
                    let db = WordsOnAnimeGirls::read();
                    // get latest srv
                    let srv = db
                        .servers
                        .iter()
                        .find(|&x| x.webhook_url == srv.webhook_url)
                        .unwrap();

                    let listing = if srv.last_listing == "" {
                        RedditListingChildData {
                            name: db.oldest_listing,
                            title: "".to_string(),
                            url: db.oldest_url,
                        }
                    } else {
                        match get_newer(&reddit_client, srv.last_listing.as_str()).await {
                            Ok(l) => l,
                            Err(e) => {
                                println!("get_newer: {}", e);
                                continue;
                            }
                        }
                    };
                    // spam channel
                    send_anime_girl(&srv.webhook_url, &listing.url)
                        .await
                        .unwrap();

                    tx.send(MsgTypes::New(Msg {
                        webhook_url: srv.webhook_url.to_owned(),
                        name: listing.name,
                    }))
                    .await
                    .unwrap();
                }
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }

    Ok(())
}
