//! OpenAI models

use std::sync::Arc;

pub use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest};
pub use tiktoken_rs::async_openai::num_tokens_from_messages;
pub use tiktoken_rs::model::get_context_size;
use tracing::{debug, error};

use crate::context::ChatEntry;
use crate::models::{ChatEntryTokenNumber, Error, Model, ModelRef, ModelResponse, Role, Usage};
/// Build an OpenAI model
/// # Arguments
/// * `model_name` - The name of the model
/// * `api_key` - The OpenAI API key
/// * `api_base` - The OpenAI API base URL - defaults to https://api.openai.com/v1
/// * `temperature` - The OpenAI chat completion request temperature. min: 0,
///   max: 2, default: 1. The higher the temperature, the crazier the text.
pub async fn build(
    model_name: &str,
    api_key: &str,
    api_base: Option<String>,
    temperature: Option<f32>,
) -> ModelRef {
    let mut openai_client = async_openai::Client::new().with_api_key(api_key);

    if let Some(api_base) = api_base {
        openai_client = openai_client.with_api_base(api_base);
    }

    let model = OpenAI::new(model_name.to_string(), temperature, openai_client);

    Arc::new(Box::new(model))
}

/// OpenAI model
#[derive(Debug, Clone)]
pub struct OpenAI {
    /// The name of the model
    model: String,
    /// The OpenAI chat completion request temperature
    /// min: 0, max: 2, default: 1,
    /// The higher the temperature, the crazier the text.
    pub temperature: Option<f32>,
    /// The client
    client: async_openai::Client,
}

impl OpenAI {
    /// Create a new OpenAI model
    pub fn new(model: String, temperature: Option<f32>, client: async_openai::Client) -> Self {
        Self {
            model,
            temperature,
            client,
        }
    }
}

impl Default for OpenAI {
    fn default() -> Self {
        Self::new(
            "gpt-3.5-turbo".to_string(),
            Some(0.),
            async_openai::Client::default(),
        )
    }
}

#[async_trait::async_trait]
impl ChatEntryTokenNumber for OpenAI {
    async fn num_tokens(&self, entries: &[ChatEntry]) -> usize {
        let messages = entries.iter().map(|x| x.into()).collect::<Vec<_>>();

        num_tokens_from_messages(&self.model, &messages).expect("model not supported")
    }

    async fn context_size(&self) -> usize {
        get_context_size(&self.model)
    }
}

impl OpenAI {
    /// prepare the [`ChatCompletionRequest`] to be passed to OpenAI
    fn prepare_chat_completion_request(
        &self,
        entries: &[&ChatEntry],
        max_tokens: Option<usize>,
    ) -> CreateChatCompletionRequest {
        let messages: Vec<ChatCompletionRequestMessage> =
            entries.iter().map(|&x| x.into()).collect();
        let temperature = self.temperature;
        CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature,
            top_p: None,
            n: Some(1),
            stream: None,
            stop: None,
            max_tokens: max_tokens.map(|x| x as u16),
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }
}

#[async_trait::async_trait]
impl Model for OpenAI {
    async fn query(
        &self,
        entries: &[&ChatEntry],
        max_tokens: Option<usize>,
    ) -> Result<ModelResponse, Error> {
        let input = self.prepare_chat_completion_request(entries, max_tokens);

        debug!("Sending request to OpenAI");
        let res = self.client.chat().create(input).await;
        if let Err(e) = &res {
            error!(error = ?e, "Error from OpenAI");
        }
        let res = res?;
        debug!(usage = ?res.usage, "Got a response from OpenAI");

        let first = res.choices.first().ok_or(Error::NoResponseFromModel)?;

        let msg = first.message.content.clone();

        Ok(ModelResponse {
            msg,
            usage: res.usage.map(Into::into),
        })
    }
}

impl From<&ChatEntry> for ChatCompletionRequestMessage {
    fn from(value: &ChatEntry) -> Self {
        Self {
            role: value.role.clone().into(),
            content: value.msg.clone(),
            name: None,
        }
    }
}

impl From<Role> for async_openai::types::Role {
    fn from(value: Role) -> Self {
        match value {
            Role::User => Self::User,
            Role::System => Self::System,
            Role::Assistant => Self::Assistant,
        }
    }
}

impl From<&ChatCompletionRequestMessage> for ChatEntry {
    fn from(msg: &ChatCompletionRequestMessage) -> Self {
        Self {
            role: msg.role.clone().into(),
            msg: msg.content.clone(),
        }
    }
}

impl From<async_openai::types::Role> for Role {
    fn from(value: async_openai::types::Role) -> Self {
        match value {
            async_openai::types::Role::User => Self::User,
            async_openai::types::Role::System => Self::System,
            async_openai::types::Role::Assistant => Self::Assistant,
        }
    }
}

impl From<async_openai::types::Usage> for Usage {
    fn from(usage: async_openai::types::Usage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn can_connect() {
        // let api_base = "https://api.openai.com/v1".to_string();
        let api_base = "http://hector:8000/v1".to_string();
        let openai_client = async_openai::Client::new().with_api_base(api_base);

        let request = CreateChatCompletionRequest {
            model: "vicuna-7b-1.1".to_string(),
            messages: vec![ChatCompletionRequestMessage {
                role: async_openai::types::Role::User,
                content: "Hello, my name is Marcel".to_string(),
                name: None,
            }],
            temperature: Some(0.0),
            top_p: None,
            n: Some(1),
            stream: None,
            stop: None,
            max_tokens: Some(1024),
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        };

        let response = openai_client.chat().create(request).await;
        println!("{:#?}", response);

        let response = response.unwrap();
        println!("{:#?}", response);

        println!("{}", response.choices.first().unwrap().message.content);
    }
}
