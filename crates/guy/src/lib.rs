use crate::prelude::*;
use crate::template::*;

pub mod error;
pub mod prelude;
pub mod template;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guy {
    pub name: Option<String>,
    pub description: Option<String>,
    pub history: Vec<ChatCompletionMessage>,
    pub functions: Vec<ChatCompletionFunction>,
}

impl Guy {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            history: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub async fn load_template(&mut self, template: GuyTemplate) -> Result<()> {
        for message in template.history {
            match message {
                ChatCompletionMessageTemplate::User(content) => {
                    self.push_message(content, ChatCompletionRole::User);
                },
                ChatCompletionMessageTemplate::System(content) => {
                    self.push_message(content, ChatCompletionRole::System);
                },
                ChatCompletionMessageTemplate::Assistant(content) => {
                    self.push_message(content, ChatCompletionRole::Assistant);
                },
                ChatCompletionMessageTemplate::UserFromFile(path) => {
                    let content = tokio::fs::read_to_string(path).await?;
                    self.push_message(content, ChatCompletionRole::User);
                },
            }
        }
        self.functions = template.functions.into_iter().map(|e| e.into()).collect();
        Ok(())
    }

    pub fn push_message(&mut self, content: String, role: ChatCompletionRole) {
        let completion = ChatCompletionMessage { content, role };
        self.history.push(completion);
    }

    pub async fn completion(&mut self, connector: &OpenAIConnector) -> Result<ChatCompletionResponse> {
        let request = ChatCompletionRequest {
            messages: &self.history[..],
            ..Default::default()
        };
        let response = connector.chat_completion_request(request).await.unwrap();
        self.history.push(response.choices[0].message.clone());
        Ok(response)
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn test_guy() {
        dotenv::dotenv().ok();
        let template = GuyTemplate::from_yaml_file("../../data/guys/crypto.yaml").unwrap();
        let mut guy = Guy::new();
        guy.load_template(template).await.unwrap();
        
        dbg!(&guy);
    }
}