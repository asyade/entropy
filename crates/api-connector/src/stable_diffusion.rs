use crate::{prelude::*, keyring::KeyChain};
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StableDiffusionError {
    #[error("IO error: {}", _0)]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {}", _0)]
    JsonError(#[from] serde_json::Error),
    #[error("reqwest error: {}", _0)]
    HttpClientError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, StableDiffusionError>;

pub struct StableDiffusionConnector {
    speech_to_text_profile: SpeechToTextProfile,
    client: Client,
    api_key: String,
}
#[derive(Debug, Serialize)]
pub struct SpeechToTextProfile {
    #[serde(skip)]
    pub api_endpoint: String,
    pub width: String,
    pub height: String,
    pub samples: String,
    pub num_inference_steps: String,
    pub safety_checker: String,
    pub enhance_prompt: String,
    pub guidance_scale: f64,
    pub multi_lingual: String,
    pub panorama: String,
    pub self_attention: String,
    pub upscale: String,
    pub embeddings_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateImageRequest<'a> {
    #[serde(flatten)]
    pub profile: &'a SpeechToTextProfile,
    pub key: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub seed: Option<u64>,
    pub webhook: Option<String>,
    pub track_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateImageResponse {
    pub status: String,
    pub id: u64,
    pub output: Vec<String>,
    pub meta: serde_json::Value,
}

impl StableDiffusionConnector {
    pub fn new(keychain: &KeyChain) -> Self {
        Self {
            speech_to_text_profile: SpeechToTextProfile::default(),
            client: Client::new(),
            api_key: keychain.get_api_key("STABLE_DIFFUSION").unwrap(),
        }
    }

    pub async fn generate_image(
        &self,
        prompt: String,
        negative_prompt: Option<String>,
        seed: Option<u64>,
    ) -> Result<GenerateImageResponse> {
        let response = self
            .client
            .post(self.speech_to_text_profile.api_endpoint.clone())
            .json(&GenerateImageRequest {
                profile: &self.speech_to_text_profile,
                key: self.api_key.clone(),
                prompt,
                negative_prompt,
                seed,
                webhook: None,
                track_id: None,
            })
            .send()
            .await?;
        Ok(response.json().await?)
    }
}

impl Default for SpeechToTextProfile {
    fn default() -> Self {
        Self {
            api_endpoint: "https://stablediffusionapi.com/api/v3/text2img".to_string(),
            width: "720".to_string(),
            height: "480".to_string(),
            samples: "1".to_string(),
            num_inference_steps: "100".to_string(),
            safety_checker: "no".to_string(),
            enhance_prompt: "no".to_string(),
            guidance_scale: 10.0,
            multi_lingual: "no".to_string(),
            panorama: "no".to_string(),
            self_attention: "no".to_string(),
            upscale: "no".to_string(),
            embeddings_model: None,
        }
    }
}
