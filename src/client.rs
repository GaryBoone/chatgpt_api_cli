use crate::api;
use anyhow::{anyhow, Context, Result};
use log::info;

const MODEL: &str = "gpt-3.5-turbo";
const URL: &str = "https://api.openai.com/v1/chat/completions";

// Define a client that handles the HTTP requests and responses to and from the OpenAI API.
pub struct Client {
    auth_token: String,
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            client: reqwest::blocking::Client::new(),
        }
    }

    // Send the chat history to the API. Return the reply message and the number of tokens used in
    // the response. Log the full request and response.
    pub fn send(&self, messages: &Vec<api::Message>) -> Result<(api::Message, u32)> {
        let request = api::ChatRequest {
            model: MODEL.to_string(),
            messages: messages.clone(),
            temperature: Some(0.7),
            ..Default::default()
        };

        info!("Request: {:#?}", &request);

        let res = self
            .client
            .post(URL)
            .bearer_auth(&self.auth_token)
            .header("Content-Type", "application/json")
            .json(&request)
            .send();

        let resp = match res {
            Ok(resp) => resp,
            Err(e) => {
                // This is an error with the reqwest library or the network, not the API.
                return Err(anyhow!("error sending request: {}", e));
            }
        };

        info!("Response: {:#?}", &resp);

        // Check for server errors.
        if !resp.status().is_success() {
            return Err(anyhow!(
                "unsuccessful API request (code: {})",
                resp.status()
            ));
        }

        // Extract and deserialize the model's message.
        let text = resp.text()?;
        let r: api::ChatResponse = serde_json::from_str(&text)?;
        let reply = r
            .choices
            .first()
            .context("no first choice")?
            .message
            .clone();
        Ok((reply, r.usage.total_tokens))
    }
}
