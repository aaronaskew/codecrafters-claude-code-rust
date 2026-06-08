use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{collections::HashMap, env, process};
use tokio::fs;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let mut messages = vec![];

    messages.push(json!({
        "role": "user",
        "content": args.prompt
    }));

    loop {
        eprintln!(
            "\n\n***********\nmessages(len={}):\n\n{:#?}\n***********\n\n",
            messages.len(),
            messages
        );

        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": messages,
                "model": "anthropic/claude-haiku-4.5",
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "Read",
                        "description": "Read and return the contents of a file",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type":"string",
                                    "description": "The path to the file to read"
                                }
                            },
                            "required": ["file_path"]
                        }
                    }
                }]
            }))
            .await?;

        // eprintln!("response: {}", response);

        let message = response["choices"][0]["message"].clone();
        messages.push(message);

        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // eprintln!("Logs from your program will appear here!");

        if let Some(tool_calls) = response["choices"][0]["message"]["tool_calls"].as_array()
            && !tool_calls.is_empty()
        {
            eprintln!("response has {} tool_calls", tool_calls.len());

            for tool_call in tool_calls {
                if let Some(tool_call_id) = tool_call["id"].as_str()
                    && let Some(function) = tool_call["function"].as_object()
                    && let Some(name) = function["name"].as_str()
                    && let Some(arguments) = function["arguments"].as_str()
                    && let Ok(arguments) =
                        serde_json::from_str::<HashMap<String, String>>(arguments)
                {
                    let content = match name {
                        "Read" if let Some(file_path) = arguments.get("file_path") => {
                            eprintln!(
                                "tool_call: {}(file_path={}) tool_call_id={}",
                                name, file_path, tool_call_id
                            );

                            fs::read_to_string(file_path).await?
                        }
                        _ => panic!("unknown tool"),
                    };

                    let tool_call_result_message = json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": content,
                    });

                    eprintln!("toll_call response: {:?}", tool_call_result_message);

                    messages.push(tool_call_result_message);
                }
            }
        } else if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            println!("{}", content);
            break;
        }
    }

    Ok(())
}
