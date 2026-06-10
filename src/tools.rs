use serde::{Serialize, Serializer};

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
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
        kind: Kind,
        description: String,
    },
    #[serde(untagged)]
    Content {
        #[serde(rename = "type")]
        kind: Kind,
        description: String,
    },
    #[serde(untagged)]
    Command {
        #[serde(rename = "type")]
        kind: Kind,
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
    pub kind: Kind,
    pub properties: ToolParametersProperties,
    #[serde(serialize_with = "parameters_kind_name_only")]
    pub required: Vec<ToolParameterKind>,
}

#[derive(Serialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub kind: Kind,
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
            kind: Kind::String,
            description: "The path to the file to read".to_owned(),
        };

        let read_tool = Tool {
            kind: Kind::Function,
            function: ToolFunction {
                name: name.clone(),
                description: Tool::description(name),
                parameters: ToolParameters {
                    kind: Kind::Object,
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
