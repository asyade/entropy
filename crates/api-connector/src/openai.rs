use crate::{keyring::KeyChain, prelude::*};
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenAIError {
    #[error("IO error: {}", _0)]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {}", _0)]
    JsonError(#[from] serde_json::Error),
    #[error("reqwest error: {}", _0)]
    HttpClientError(#[from] reqwest::Error),
    #[error("completion failed ({:?}): {} (http status: {})", code, message, status)]
    CompletionFailed {
        message: String,
        code: Option<String>,
        status: StatusCode,
    },
}

pub type Result<T> = std::result::Result<T, OpenAIError>;


#[derive(Clone, Debug)]
pub struct OpenAIConnector {
    profile: ChatGptProfile,
    client: Client,
    api_key: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct ChatGptProfile {
    #[serde(skip)]
    pub api_endpoint: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct ChatCompletionRequest<'a> {
    pub model: String,
    pub messages: &'a [ChatCompletionMessage],
    // todo: functions
    // pub functions: Option<&'a [ChatCompletionFunction]>,
    // pub function_call: Option<serde_json::Value>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub n: Option<u64>,
    pub stream: Option<bool>,
    pub stop: Option<String>,
    pub max_tokens: Option<u64>,
    // pub presence_penalty: Option<f64>,
    // pub frequency_penalty: Option<f64>,
    // pub logit_bias: Option<serde_json::Value>,
    // pub user: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<serde_json::Value>,
    pub usage: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatCompletionError {
    pub error: ChatCompletionErrorContent,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct  ChatCompletionErrorContent {
    pub message: String,
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatCompletionChoice {
    pub message: ChatCompletionMessage,
    pub index: u64,
    pub finish_reason: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatCompletionMessage {
    pub role: ChatCompletionRole,
    pub content: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunction {
    pub name: String,
    pub description: String,
    pub parameters: ChatCompletionFunctionParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunctionParameters {
    #[serde(rename = "type")]
    pub kind: String,
    pub properties: HashMap<String, ChatCompletionFunctionProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunctionProperty {
    #[serde(rename = "type")]
    pub kind: String,
    pub description: String,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatCompletionRole {
    System,
    User,
    Assistant,
    Function,
}

impl OpenAIConnector {
    pub fn new(keychain: &KeyChain) -> Self {
        Self {
            profile: ChatGptProfile::default(),
            client: Client::new(),
            api_key: keychain
                .get_api_key("OPENAI")
                .expect("API key not found in keychain: `OPENAI`"),
        }
    }

    pub async fn chat_completion_request<'a>(
        &self,
        request: ChatCompletionRequest<'a>,
    ) -> Result<ChatCompletionResponse> {
        let response = self
            .client
            .post(self.profile.api_endpoint.clone())
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;
        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error = response.json::<ChatCompletionError>().await;
            Err(OpenAIError::CompletionFailed {
                message: error.as_ref().map(|e| e.error.message.clone()).unwrap_or_else(|_| format!("Non success status code (failed to deserialize response): `{}`", status)),
                code: error.as_ref().map(|e| e.error.code.clone()).ok(),
                status,
            })
        }
    }
}

impl Default for ChatGptProfile {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
        }
    }
}

impl Default for ChatCompletionRequest<'static> {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            messages: &[],
            // functions: None,
            // function_call: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
        }
    }
}
