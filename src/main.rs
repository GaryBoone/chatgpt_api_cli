use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use thousands::Separable;

mod api;
mod bot;
mod client;

// This project is a simple chatbot that uses the OpenAI's chat completions API and the new
// `gpt-3.5-turbo` model to generate responses to user input. The chat history is sent to the API
// with each request so that the API can respond within the context of the conversation.

// Setup:
// 1. Obtain an OpenAI API key from https://beta.openai.com/account/api-keys
// 2. Either:
//    - Set the environment variable OPENAI_API_KEY to the API key.
//    $ export OPENAI_API_KEY=<key>
//    OR:
//    - Create a file named `open_ai_auth_key.txt` in the project directory and put the API key in it.
// Run:
// 3. $ cargo run
//    Or to see full API requests and responses:
//    $ RUST_LOG=debug cargo run
// 4. Enter text. The complete chat history is sent to the API for context and the API's response is printed.
//    Enter `c` to clear the chat history and `q` to exit.

// The environment variable that contains the OpenAI API key. Use this or the file below.
// This environment variable will be checked first. If not defined, the code will read the file below.
const OPENAI_API_KEY_VAR: &str = "OPENAI_API_KEY";
// The file that contains the OpenAI API key. Use this or the environment variable above.
const OPENAI_API_KEY_FILE: &str = "open_ai_auth_key.txt";

// The prompt to display initially or when the user enters an empty line.
const PROMPT_HELP: &str = "Enter text. Enter `c` to clear the chat history and `q` to exit.";

// Obtain the OpenAI API key from the environment variable OPENAI_API_KEY_ENV. If not defined, read it
// from the file OPENAI_API_KEY_FILE.
fn auth_token() -> Result<String> {
    match env::var(OPENAI_API_KEY_VAR).context("OPENAI_API_KEY not set") {
        Ok(s) => return Ok(s),
        Err(_) => fs::read_to_string(OPENAI_API_KEY_FILE)
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

// Create a ChatGPT demo by collecting user input and sending it to the API. Print the API's
// response and provide controls for clearing the chat history and exiting the demo.
fn main() -> Result<()> {
    env_logger::init();

    let mut chat_bot = bot::ChatBot::new(auth_token()?);

    println!("> {}", PROMPT_HELP);
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        // Read a line of input from the user.
        let mut input_line = String::new();
        std::io::stdin().read_line(&mut input_line)?;
        let input_line = input_line.trim();

        // Handle user flow control.
        match input_line.trim() {
            "q" => {
                println!("  [Exiting]");
                break;
            }
            "c" => {
                println!("  [Clearing chat history]");
                chat_bot.clear();
                continue;
            }
            "" => {
                println!("> {}", PROMPT_HELP);
                continue;
            }
            _ => {}
        }

        // Send the message to the API and print the response.
        println!("  [Sending chat to gpt-3.5-turbo...]");
        let (text, tokens) = chat_bot.chat(input_line)?;
        println!(
            "GPT [{} tokens used for this context and prompt]: {}",
            tokens.separate_with_commas(),
            &text
        );
    }
    Ok(())
}
