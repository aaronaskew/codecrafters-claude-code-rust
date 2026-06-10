use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use std::{collections::HashMap, env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

mod tools;

use tools::*;

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

    let tools = vec![
        Tool {
            kind: Kind::Function,
            function: ToolFunction {
                name: ToolName::Read,
                description: Tool::description(ToolName::Read),
                parameters: ToolParameters {
                    kind: Kind::Object,
                    properties: ToolParametersProperties {
                        file_path: Some(ToolParameterKind::FilePath {
                            kind: Kind::String,
                            description: "The path to the file to read".to_owned(),
                        }),
                        content: None,
                        command: None,
                    },
                    required: vec![ToolParameterKind::FilePath {
                        kind: Kind::String,
                        description: "The path to the file to read".to_owned(),
                    }],
                },
            },
        },
        Tool {
            kind: Kind::Function,
            function: ToolFunction {
                name: ToolName::Write,
                description: Tool::description(ToolName::Write),
                parameters: ToolParameters {
                    kind: Kind::Object,
                    properties: ToolParametersProperties {
                        file_path: Some(ToolParameterKind::FilePath {
                            kind: Kind::String,
                            description: "The path of the file to write to".to_owned(),
                        }),
                        content: Some(ToolParameterKind::Content {
                            kind: Kind::String,
                            description: "The content to write to the file".to_owned(),
                        }),
                        command: None,
                    },
                    required: vec![
                        ToolParameterKind::FilePath {
                            kind: Kind::String,
                            description: "The path of the file to write to".to_owned(),
                        },
                        ToolParameterKind::Content {
                            kind: Kind::String,
                            description: "The content to write to the file".to_owned(),
                        },
                    ],
                },
            },
        },
        Tool {
            kind: Kind::Function,
            function: ToolFunction {
                name: ToolName::Bash,
                description: Tool::description(ToolName::Bash),
                parameters: ToolParameters {
                    kind: Kind::Object,
                    properties: ToolParametersProperties {
                        file_path: None,
                        content: None,
                        command: Some(ToolParameterKind::Command {
                            kind: Kind::String,
                            description: "The command to execute".to_owned(),
                        }),
                    },
                    required: vec![ToolParameterKind::Command {
                        kind: Kind::String,
                        description: "The command to execute".to_owned(),
                    }],
                },
            },
        },
    ];

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": messages,
                "model": "anthropic/claude-haiku-4.5",
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
                    let content = match name {
                        "Read" if let Some(file_path) = arguments.get("file_path") => {
                            fs::read_to_string(file_path)?
                        }
                        "Write"
                            if let Some(file_path) = arguments.get("file_path")
                                && let Some(content) = arguments.get("content") =>
                        {
                            let mut file = File::options()
                                .create(true)
                                .truncate(true)
                                .write(true)
                                .open(file_path)?;
                            file.write_all(content.as_bytes())?;
                            content.clone()
                        }
                        "Bash" if let Some(command) = arguments.get("command") => {
                            let output: String = match Command::new("bash")
                                .arg("-c")
                                .arg(command)
                                .arg("2>&1") // Pipe stderr into stdout as both are returned by tool
                                .output()
                            {
                                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                                Err(err) => format!("Error: {}", err),
                            };

                            output
                        }
                        unknown_tool => panic!("unknown tool: {}", unknown_tool),
                    };

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
