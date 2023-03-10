use anyhow::{anyhow, Result};

use crate::api;
use crate::client;

// Chat holds and manages the history of structured chat messages.
struct Chat {
    messages: Vec<api::Message>,
}

impl Chat {
    fn new() -> Self {
        Self { messages: vec![] }
    }

    // Add a text line from the user by structuring it as a chat message and storing it.
    fn add_user_text(&mut self, text: &str) {
        self.messages.push(api::Message {
            role: "user".to_string(),
            content: Some(text.to_string()),
        });
    }

    // Add an already-structured message to the chat history.
    fn add_message(&mut self, message: api::Message) {
        self.messages.push(message);
    }

    // Remove the context given to the chatbot with each request by clearing the chat history.
    fn clear(&mut self) {
        self.messages.clear();
    }
}

// ChatBot holds the chat history and the client that sends the chat history to the API.
pub struct ChatBot {
    chat: Chat,
    client: client::Client,
}

impl ChatBot {
    pub fn new(auth_token: String) -> Self {
        Self {
            chat: Chat::new(),
            client: client::Client::new(auth_token),
        }
    }

    // Add the user's text to the chat history and send the whole history to the API so that it can
    // respond within the context of the conversation. Return the model's response text and the
    // number of tokens used.
    pub fn chat(&mut self, text: &str) -> Result<(String, u32)> {
        self.chat.add_user_text(text);

        let (gpt_message, tokens) = self.client.send(&self.chat.messages)?;

        self.chat.add_message(gpt_message.clone());

        let text = gpt_message
            .content
            .clone()
            .ok_or(anyhow!("no content received"))?;
        Ok((text, tokens))
    }

    // Clear the chat history.
    pub fn clear(&mut self) {
        self.chat.clear();
    }
}
