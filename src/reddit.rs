use reqwest::{
    header::{HeaderMap, AUTHORIZATION, USER_AGENT},
    Client, Error,
};
use serde::Deserialize;
use std::error;

#[derive(Deserialize, Debug)]
struct RedditAuthResponse {
    access_token: String,
    // token_type: String,
    // expires_in: u32,
    // scope: String,
}
#[derive(Clone, Deserialize, Debug)]
pub struct RedditListingChildData {
    pub title: String,
    pub name: String,
    pub url: String,
}
#[derive(Deserialize, Debug)]
pub struct RedditListingChild {
    pub kind: String,
    pub data: RedditListingChildData,
}
#[derive(Deserialize, Debug)]
pub struct RedditListingData {
    // pub after: String,
    pub dist: u32,
    // pub modhash: String,
    // pub geo_filter: String,
    pub children: Vec<RedditListingChild>,
}
#[derive(Deserialize, Debug)]
pub struct RedditListings {
    pub kind: String,
    pub data: RedditListingData,
}

pub struct RedditClient {
    client: Client,
    // todo
}

impl RedditClient {
    pub async fn new(client_id: &str, client_secret: &str) -> Result<RedditClient, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "linux:no:v0.0.1".parse().unwrap());
        let mut client_headers = headers.clone();

        let params = [("grant_type", "client_credentials")];
        let client = reqwest::Client::new();
        let res = client
            .post("https://www.reddit.com/api/v1/access_token")
            .headers(headers)
            .basic_auth(client_id, Some(client_secret))
            .form(&params)
            .send()
            .await?
            .json::<RedditAuthResponse>()
            .await?;

        let bearer_token = "Bearer ".to_owned() + &res.access_token;
        client_headers.insert(AUTHORIZATION, bearer_token.parse().unwrap());
        Ok(RedditClient {
            client: Client::builder().default_headers(client_headers).build()?,
        })
    }

    pub async fn get_listings(
        &self,
        count: &str,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<Vec<RedditListingChild>, Box<dyn error::Error>> {
        let mut qs: Vec<(&str, &str)> = vec![("limit", count)];
        match after {
            Some(x) => qs.push(("after", x)),
            None => (),
        };
        match before {
            Some(x) => qs.push(("before", x)),
            None => (),
        };

        let RedditListings {
            data: RedditListingData { children, .. },
            ..
        } = self
            .client
            .get("https://oauth.reddit.com/r/wordsonanimegirls/new?raw_json=1")
            .query(&qs)
            .send()
            .await?
            .json::<RedditListings>()
            .await?;

        // let txt = resp.text().await?;
        // println!("{:?}", txt);
        // let xx = serde_json::from_str::<RedditListings>(&txt)?;
        Ok(children)
    }
}
