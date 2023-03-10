# chatgpt_api_cli

This project is a chatbot that uses the `OpenAI`'s chat completions API and the new
`gpt-3.5-turbo` model to generate AI responses to user input. The chat history is sent to the API
with each request so that the API can respond within the context of the conversation.

The code shows how to use the `OpenAI` API to generate chat completions in Rust.
It's also a pretty convenient command-line interface for chatting with the model.

It's not quite the simplest possible ChatGPT chatbot, but it's close, just adding some Rust structure.

## Details

This demo:

- structures the `OpenAI` Rest API calls and fields into Rust structs
- includes a chat loop that appends responses so that the model can use the history
- provides logging that prints the full JSON requests and responses
- uses the `reqwest` crate for the HTTP calls
- serializes and deserializes API structures using `serde` for JSON
- adds error with context handling using `anyhow`

## Setup:

1. Obtain an `OpenAI` API key from https://beta.openai.com/account/api-keys
2. Either:

   - Set the environment variable `OPENAI_API_KEY` to the API key.

   ```shell
     $ export OPENAI_API_KEY=<key>
   ```

   OR:

   - Create a file named `open_ai_auth_key.txt` in the project directory and put the API key in it.

### Run:

1. Start via

   ```shell
   $ cargo run
   ```

   Or to see full API requests and responses:

   ```shell
   $ RUST_LOG=debug cargo run
   ```

2. Enter text at the '>' prompt.

- The complete chat history is sent to the API for context and the API's response is printed.
- Enter `c` to clear the chat history and `q` to exit.

### API documentation

The `OpenAI` chat completion documentation is here:

- [Chat Completion Guide](https://platform.openai.com/docs/guides/chat)
- [API Reference | Chat](https://platform.openai.com/docs/api-reference/chat)
