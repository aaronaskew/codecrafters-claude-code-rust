use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    process::Command,
};

use serde::{Serialize, Serializer};

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum JsonValueKind {
    Function,
    Object,
    String,
}

#[derive(Serialize, Clone)]
pub enum ToolName {
    Read,
    Write,
    Bash,
}

#[derive(Serialize)]
pub struct ToolFunction {
    pub name: ToolName,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ToolParameterKind {
    #[serde(untagged)]
    FilePath {
        #[serde(rename = "type")]
        kind: JsonValueKind,
        description: String,
    },
    #[serde(untagged)]
    Content {
        #[serde(rename = "type")]
        kind: JsonValueKind,
        description: String,
    },
    #[serde(untagged)]
    Command {
        #[serde(rename = "type")]
        kind: JsonValueKind,
        description: String,
    },
}

#[derive(Serialize)]
pub struct ToolParametersProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<ToolParameterKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolParameterKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<ToolParameterKind>,
}

#[derive(Serialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub kind: JsonValueKind,
    pub properties: ToolParametersProperties,
    #[serde(serialize_with = "parameters_kind_name_only")]
    pub required: Vec<ToolParameterKind>,
}

impl ToolParameters {
    fn new(name: ToolName) -> Self {
        let properties = match name {
            ToolName::Read => ToolParametersProperties {
                file_path: Some(ToolParameterKind::FilePath {
                    kind: JsonValueKind::String,
                    description: "The path to the file to read".to_owned(),
                }),
                content: None,
                command: None,
            },
            ToolName::Write => ToolParametersProperties {
                file_path: Some(ToolParameterKind::FilePath {
                    kind: JsonValueKind::String,
                    description: "The path to the file to write to".to_owned(),
                }),
                content: Some(ToolParameterKind::Content {
                    kind: JsonValueKind::String,
                    description: "The content to write to the file".to_owned(),
                }),
                command: None,
            },
            ToolName::Bash => ToolParametersProperties {
                file_path: Some(ToolParameterKind::Command {
                    kind: JsonValueKind::String,
                    description: "The command to execute".to_owned(),
                }),
                content: None,
                command: None,
            },
        };

        let required = {
            let mut required = vec![];

            if let Some(file_path) = properties.file_path.clone() {
                required.push(file_path);
            }

            if let Some(content) = properties.content.clone() {
                required.push(content);
            }

            if let Some(command) = properties.command.clone() {
                required.push(command);
            }

            required
        };

        Self {
            kind: JsonValueKind::Object,
            properties,
            required,
        }
    }
}

#[derive(Serialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub kind: JsonValueKind,
    pub function: ToolFunction,
}

impl Tool {
    pub fn description(name: ToolName) -> String {
        match name {
            ToolName::Read => "Read and return the contents of a file".to_owned(),
            ToolName::Write => "Write content to a file".to_owned(),
            ToolName::Bash => "Execute a shell command".to_owned(),
        }
    }

    pub fn new(name: ToolName) -> Self {
        Self {
            kind: JsonValueKind::Function,
            function: ToolFunction {
                name: name.clone(),
                description: Tool::description(name.clone()),
                parameters: ToolParameters::new(name),
            },
        }
    }

    /// Call the given tool by `name` and return the output of when the tool is run.
    pub fn call(
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match name {
            "Read" if let Some(file_path) = arguments.get("file_path") => {
                Ok(fs::read_to_string(file_path)?)
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
                Ok(content.clone())
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

                Ok(output)
            }
            unknown_tool => panic!("unknown tool: {}", unknown_tool),
        }
    }
}

pub fn parameters_kind_name_only<S>(
    tool_parameter_kinds: &[ToolParameterKind],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let kind_names = tool_parameter_kinds
        .iter()
        .map(|k| match k {
            ToolParameterKind::FilePath { .. } => "file_path",
            ToolParameterKind::Content { .. } => "content",
            ToolParameterKind::Command { .. } => "command",
        })
        .collect::<Vec<_>>();

    kind_names.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn read_tool() {
        let name = ToolName::Read;
        let file_path = ToolParameterKind::FilePath {
            kind: JsonValueKind::String,
            description: "The path to the file to read".to_owned(),
        };

        let read_tool = Tool {
            kind: JsonValueKind::Function,
            function: ToolFunction {
                name: name.clone(),
                description: Tool::description(name),
                parameters: ToolParameters {
                    kind: JsonValueKind::Object,
                    properties: ToolParametersProperties {
                        file_path: Some(file_path.clone()),
                        content: None,
                        command: None,
                    },
                    required: vec![file_path],
                },
            },
        };

        let read_json = json!(
        {
            "type": "function",
            "function": {
                "name": "Read",
                "description": "Read and return the contents of a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The path to the file to read"
                        }
                    },
                    "required": ["file_path"],
                }
            }
        });

        println!(
            "json text:\n\n{}",
            serde_json::to_string_pretty(&read_json).unwrap()
        );

        println!(
            "derived:\n\n{}",
            serde_json::to_string_pretty(&read_tool).unwrap()
        );

        assert_eq!(
            read_json,
            serde_json::from_str::<Value>(&serde_json::to_string(&read_tool).unwrap()).unwrap()
        );
    }
}
