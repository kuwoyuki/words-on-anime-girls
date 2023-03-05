use reqwest::Response;
use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone)]
struct DiscordWebHookPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    tts: bool,
}

#[derive(Debug)]
pub struct DiscordWebHook {
    webhook_url: String,
    client: reqwest::Client,
    payload: DiscordWebHookPayload,
}

struct DefaultLength {
    content: usize,
    username: usize,
}

static MAX_LEN: DefaultLength = DefaultLength {
    content: 2000,
    username: 256,
};

impl DiscordWebHook {
    pub fn new(webhook_url: &str, content: &str) -> DiscordWebHook {
        let mut tmp_content = content.to_owned();
        tmp_content.truncate(MAX_LEN.content);

        let mut payload = DiscordWebHookPayload::default();
        payload.content = Some(tmp_content);

        return DiscordWebHook {
            webhook_url: webhook_url.to_owned(),
            client: reqwest::Client::new(),
            payload: payload,
        };
    }

    pub fn get_url(&self) -> &str {
        return &self.webhook_url;
    }

    pub async fn fire(&self) -> reqwest::Result<Response> {
        self.client
            .post(self.get_url())
            .json(&self.payload)
            .send()
            .await
    }

    pub fn set_avatar_url(&mut self, avatar_url: &str) {
        self.payload.avatar_url = Some(avatar_url.to_owned());
    }

    pub fn set_username(&mut self, username: &str) {
        let mut tmp_username = username.to_owned();
        tmp_username.truncate(MAX_LEN.username);
        self.payload.username = Some(tmp_username);
    }
}
