use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{collections::HashMap, env, process};

use crate::tool::{Tool, ToolName};

mod tool;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load .env if running locally
    let _ = dotenvy::dotenv();

    // Set model if running locally
    let model = if let Ok(local_model) = env::var("LOCAL_MODEL") {
        local_model.clone()
    } else {
        "anthropic/claude-haiku-4.5".to_owned()
    };

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

    let tools = vec![
        Tool::new(ToolName::Read),
        Tool::new(ToolName::Write),
        Tool::new(ToolName::Bash),
    ];

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": messages,
                "model": model,
                "tools": tools
            }))
            .await?;

        let message = response["choices"][0]["message"].clone();
        messages.push(message);

        if let Some(tool_calls) = response["choices"][0]["message"]["tool_calls"].as_array()
            && !tool_calls.is_empty()
        {
            for tool_call in tool_calls {
                if let Some(tool_call_id) = tool_call["id"].as_str()
                    && let Some(function) = tool_call["function"].as_object()
                    && let Some(name) = function["name"].as_str()
                    && let Some(arguments) = function["arguments"].as_str()
                    && let Ok(arguments) =
                        serde_json::from_str::<HashMap<String, String>>(arguments)
                {
                    let content = Tool::call(name, arguments)?;

                    let tool_call_result_message = json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": content,
                    });

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
