use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuyTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub history: Vec<ChatCompletionMessageTemplate>,
    #[serde(default)]
    pub functions: Vec<ChatCompletionFunctionTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunctionTemplate {
    pub name: String,
    pub description: String,
    pub parameters: ChatCompletionFunctionParametersTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunctionParametersTemplate {
    #[serde(rename = "type")]
    kind: String,
    properties: HashMap<String, ChatCompletionFunctionPropertyTemplate>,
    required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunctionPropertyTemplate {
    #[serde(rename = "type")]
    pub kind: String,
    pub description: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatCompletionMessageTemplate {
    User(String),
    UserFromFile(String),
    System(String),
    Assistant(String),
}

impl GuyTemplate {
    pub fn from_yaml_file(path: &str) -> crate::error::Result<Self> {
        Ok(serde_yaml::from_str(&std::fs::read_to_string(path)?)?)
    }
}

impl Into<ChatCompletionFunction> for ChatCompletionFunctionTemplate {
    fn into(self) -> ChatCompletionFunction {
        ChatCompletionFunction {
            name: self.name,
            description: self.description,
            parameters: self.parameters.into(),
        }
    }
}

impl Into<ChatCompletionFunctionParameters> for ChatCompletionFunctionParametersTemplate {
    fn into(self) -> ChatCompletionFunctionParameters {
        ChatCompletionFunctionParameters {
            kind: self.kind,
            properties: self.properties.into_iter().map(|(k, v)| (k, v.into())).collect(),
            required: self.required,
        }
    }
}

impl  Into<ChatCompletionFunctionProperty> for ChatCompletionFunctionPropertyTemplate {
    fn into(self) -> ChatCompletionFunctionProperty {
        ChatCompletionFunctionProperty {
            kind: self.kind,
            description: self.description,
        }
    }
}
