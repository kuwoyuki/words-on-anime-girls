mod db;
mod reddit;
mod webhook;

use db::WordsOnAnimeGirls;
use reddit::{ListingError, RedditClient, RedditListingChildData};
use reqwest::Response;
use std::env;
use std::error;
use std::time::Duration;
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
    hook.fire().await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    WordsOnAnimeGirls::ensure_exists();
    // todo: config or sth
    let client_id = env::var("CLIENT_ID").expect("$CLIENT_ID is not set");
    let client_secret = env::var("CLIENT_SECRET").expect("$CLIENT_SECRET is not set");
    let hook_url = env::var("DISCORD_HOOK").expect("$DISCORD_HOOK is not set");
    let interval: u64 = env::var("INTERVAL")
        .expect("$INTERVAL is not set")
        .parse()
        .unwrap();

    let reddit_client = RedditClient::new(client_id.as_str(), client_secret.as_str()).await?;

    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval));

        loop {
            interval.tick().await;

            let mut db = WordsOnAnimeGirls::read();
            let listing: RedditListingChildData;
            if db.oldest_listing == "" {
                listing = find_oldest_listing(&reddit_client)
                    .await
                    .expect("failed to find the oldest listing");
                db.oldest_listing = listing.name.clone();
            } else {
                listing = get_newer(&reddit_client, db.last_listing.as_str())
                    .await
                    .expect("failed to fetch the new listing");
            }
            // spam channel
            send_anime_girl(hook_url.as_str(), &listing.url)
                .await
                .unwrap();
            // bump lst
            db.last_listing = listing.name;

            task::spawn_blocking(move || db.write()).await.unwrap();
        }
    });

    forever.await?;
    Ok(())
}
