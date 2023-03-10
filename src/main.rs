use anyhow::{anyhow, Context, Result};
use log::info;
use std::env;
use std::fs;
use std::io::Write;
use thousands::Separable;

mod chat_api;

// This is a basic chatbot that uses newly announced `gpt-3.5-turbo` model via the OpenAI API. The
// code shows how to use the OpenAI API to generate chat completions in Rust.
//
// It's not quite the simplest possible chatbot, but it's close, just adding some Rust structure.
// This demo:
// • uses the `reqwest` crate for the HTTP calls
// • serializes and deserializes API structures using `serde` for JSON
// • adds error with context handling using `anyhow`
// • structures the OpenAI Rest API calls and fields into Rust structs
// • includes a chat loop that appends responses so that the model can use the history
// • provides logging that prints the full JSON requests and responses.
//   (Run with `RUST_LOG=info cargo run` to see the log output.)

const MODEL: &str = "gpt-3.5-turbo";
const URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_API_KEY_VAR: &str = "OPENAI_API_KEY";
const OPENAI_API_KEY_FILE: &str = "open_ai_auth_key.txt";
const PROMPT: &str = "Enter text. Enter `c` to clear the chat history and `q` to exit.";

// Obtain the OpenAI API key from the environment variable OPENAI_API_KEY_ENV. If not defined, read it
// from the file OPENAI_API_KEY_FILE
fn auth_token() -> Result<String> {
    match env::var(OPENAI_API_KEY_VAR).context("OPENAI_API_KEY not set") {
        Ok(s) => return Ok(s),
        Err(e) => fs::read_to_string(OPENAI_API_KEY_FILE)
            .context(format!(
                "error reading auth token file: {}",
                OPENAI_API_KEY_FILE
            ))
            .map(|s| s.trim().to_string())
            .context(format!(
                "couldn't find OpenAI authentication key in environment variable (${}) or file",
                OPENAI_API_KEY_VAR
            )),
    }
}

// Chat holds and manages the history of structured chat messages.
struct Chat {
    messages: Vec<chat_api::Message>,
}

impl Chat {
    fn new() -> Self {
        Self { messages: vec![] }
    }

    fn add_user_text(&mut self, text: &str) {
        self.messages.push(chat_api::Message {
            role: "user".to_string(),
            content: Some(text.to_string()),
        });
    }

    fn add_message(&mut self, message: chat_api::Message) {
        self.messages.push(message);
    }

    fn clear(&mut self) {
        self.messages.clear();
    }
}

struct ChatBot {
    auth_token: String,
    chat: Chat,
    client: reqwest::blocking::Client,
}

impl ChatBot {
    fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            chat: Chat::new(),
            client: reqwest::blocking::Client::new(),
        }
    }

    fn chat(&mut self, text: &str) -> Result<(u32, String)> {
        self.chat.add_user_text(text);

        let request = chat_api::ChatRequest {
            model: MODEL.to_string(),
            messages: self.chat.messages.clone(),
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
        if resp.status().is_server_error() {
            return Err(anyhow!("server error ({})", resp.status()));
        } else if !resp.status().is_success() {
            return Err(anyhow!(
                "unsuccessful server response (code: {:?})",
                resp.status()
            ));
        }

        // Extract and deserialize the response message.
        let text = resp.text()?;
        let r: chat_api::ChatResponse = serde_json::from_str(&text)?;
        let gpt_message = &r.choices.first().context("no first choice")?.message;

        // Add the message to the chat history so that it can be sent to the API, providing
        // additional context for the next user message.
        self.chat.add_message(gpt_message.clone());

        let tokens = r.usage.total_tokens;
        // Return just the text of the message.
        let text = gpt_message
            .content
            .clone()
            .ok_or(anyhow!("no content received"))?;
        Ok((tokens, text))
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let auth_token = auth_token()?;

    println!("> {}", PROMPT);

    let mut chat_bot = ChatBot::new(auth_token);
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut input_line = String::new();
        std::io::stdin().read_line(&mut input_line)?;
        let input_line = input_line.trim();

        match input_line.trim() {
            "q" => {
                println!("  [Exiting]");
                break;
            }
            "c" => {
                println!("  [Clearing chat history]");
                chat_bot.chat.clear();
                continue;
            }
            "" => {
                println!("> {}", PROMPT);
                continue;
            }
            _ => {}
        }

        println!("  [Sending chat to gpt-3.5-turbo...]");
        let (tokens, text) = chat_bot.chat(input_line)?;
        println!(
            "GPT [{} tokens used for this context and prompt]: {}",
            tokens.separate_with_commas(),
            &text
        );
    }
    Ok(())
}
