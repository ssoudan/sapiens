//! OpenAI models

use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::Arc;

pub use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
pub use tiktoken_rs::async_openai::num_tokens_from_messages;
pub use tiktoken_rs::model::get_context_size;
use tracing::{debug, error};

use crate::context::ChatEntry;
use crate::models::{
    ChatEntryTokenNumber, ChatInput, Error, Model, ModelRef, ModelResponse, Role, Usage,
};
/// Build an OpenAI model
/// # Arguments
/// * `model_name` - The model to use
/// * `api_key` - The OpenAI API key
/// * `api_base` - The OpenAI API base URL - defaults to https://api.openai.com/v1
/// * `temperature` - The OpenAI chat completion request temperature. min: 0,
///   max: 2, default: 1. The higher the temperature, the crazier the text.
pub async fn build(
    model: SupportedModel,
    api_key: Option<String>,
    api_base: Option<String>,
    temperature: Option<f32>,
) -> Result<ModelRef, Error> {
    let mut openai_client = async_openai::Client::new();

    if let Some(api_key) = api_key {
        openai_client = openai_client.with_api_key(api_key);
    }

    if let Some(api_base) = api_base {
        openai_client = openai_client.with_api_base(api_base);
    }

    let model = OpenAI::new(model, temperature, openai_client);

    Ok(Arc::new(Box::new(model)))
}

/// OpenAI model
#[derive(Debug, Clone)]
pub struct OpenAI {
    /// The model
    model: SupportedModel,
    /// The OpenAI chat completion request temperature
    /// min: 0, max: 2, default: 1,
    /// The higher the temperature, the crazier the text.
    pub temperature: Option<f32>,
    /// The client
    client: async_openai::Client,
}

/// Supported models
#[derive(Clone, Serialize, Deserialize, Default)]
pub enum SupportedModel {
    /// GPT 3.5 Turbo
    #[default]
    GPT3_5Turbo,
    /// Vicuna 7B 1.1
    Vicuna7B1_1,
    /// Vicuna 13B 1.1
    Vicuna13B1_1,
}

impl Display for SupportedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedModel::GPT3_5Turbo => write!(f, "gpt-3.5-turbo"),
            SupportedModel::Vicuna7B1_1 => write!(f, "vicuna-7b-1.1"),
            SupportedModel::Vicuna13B1_1 => write!(f, "vicuna-13b-1.1"),
        }
    }
}

impl Debug for SupportedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedModel::GPT3_5Turbo => write!(f, "gpt-3.5-turbo"),
            SupportedModel::Vicuna7B1_1 => write!(f, "vicuna-7b-1.1"),
            SupportedModel::Vicuna13B1_1 => write!(f, "vicuna-13b-1.1"),
        }
    }
}

impl FromStr for SupportedModel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gpt-3.5-turbo" => Ok(Self::GPT3_5Turbo),
            "vicuna-7b-1.1" => Ok(Self::Vicuna7B1_1),
            "vicuna-13b-1.1" => Ok(Self::Vicuna13B1_1),
            _ => Err(Error::ModelNotSupported(s.to_string())),
        }
    }
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for SupportedModel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            SupportedModel::GPT3_5Turbo,
            SupportedModel::Vicuna7B1_1,
            SupportedModel::Vicuna13B1_1,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            SupportedModel::GPT3_5Turbo => Some(clap::builder::PossibleValue::new("gpt-3.5-turbo")),
            SupportedModel::Vicuna7B1_1 => Some(clap::builder::PossibleValue::new("vicuna-7b-1.1")),
            SupportedModel::Vicuna13B1_1 => {
                Some(clap::builder::PossibleValue::new("vicuna-13b-1.1"))
            }
        }
    }
}

impl OpenAI {
    /// Create a new OpenAI model
    pub fn new(
        model: SupportedModel,
        temperature: Option<f32>,
        client: async_openai::Client,
    ) -> Self {
        Self {
            model,
            temperature,
            client,
        }
    }
}

impl Default for OpenAI {
    fn default() -> Self {
        Self {
            model: SupportedModel::GPT3_5Turbo,
            temperature: Some(0.),
            client: Default::default(),
        }
    }
}

const LLAMA_TOKENIZER_JSON: &str = include_str!("tokenizer.json");

lazy_static! {
    static ref LLAMA_TOKENIZER: tokenizers::Tokenizer =
        tokenizers::Tokenizer::from_str(LLAMA_TOKENIZER_JSON).unwrap();
}

#[async_trait::async_trait]
impl ChatEntryTokenNumber for OpenAI {
    async fn num_tokens(&self, input: ChatInput) -> usize {
        let req = self.prepare_chat_completion_request(input, None);

        match &self.model {
            SupportedModel::GPT3_5Turbo => {
                let messages = req.messages;
                num_tokens_from_messages(&self.model.to_string(), messages.as_slice())
                    .expect("model not supported")
            }
            SupportedModel::Vicuna7B1_1 | SupportedModel::Vicuna13B1_1 => {
                // See https://github.com/lm-sys/FastChat/blob/667c584ad437b4655f29ca99d480d96833470860/fastchat/conversation.py#LL62C24-L62C24
                let seps = vec![" ", "</s>"];

                let chat = req
                    .messages
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        // assumes the first is a 'System' message
                        let sep = seps[(i + 1) % seps.len()];
                        match x.role {
                            async_openai::types::Role::System => {
                                if x.content.is_empty() {
                                    String::new()
                                } else {
                                    format!("{}\n", x.content)
                                }
                            }
                            async_openai::types::Role::User => {
                                if x.content.is_empty() {
                                    format!("{}:\n", x.role.to_string().to_ascii_uppercase())
                                } else {
                                    format!(
                                        "{}: {}\n",
                                        x.role.to_string().to_ascii_uppercase(),
                                        x.content
                                    )
                                }
                            }
                            async_openai::types::Role::Assistant => {
                                if x.content.is_empty() {
                                    format!("{}:\n", x.role.to_string().to_ascii_uppercase())
                                } else {
                                    format!(
                                        "{}: {}{}\n",
                                        x.role.to_string().to_ascii_uppercase(),
                                        x.content,
                                        sep
                                    )
                                }
                            }
                        }
                    })
                    .collect::<String>();

                let encoding = LLAMA_TOKENIZER.encode(chat, false).unwrap();

                encoding.get_ids().len() + 1

                // FUTURE(ssoudan) compare with the number of tokens from the
                // response
            }
        }
    }

    async fn context_size(&self) -> usize {
        match &self.model {
            SupportedModel::GPT3_5Turbo => get_context_size(&self.model.to_string()),
            SupportedModel::Vicuna7B1_1 | SupportedModel::Vicuna13B1_1 => 2048,
        }
    }
}

impl OpenAI {
    /// prepare the [`ChatCompletionRequest`] to be passed to OpenAI
    fn prepare_chat_completion_request(
        &self,
        input: ChatInput,
        max_tokens: Option<usize>,
    ) -> CreateChatCompletionRequest {
        let mut messages = vec![];

        for m in input.context {
            messages.push(ChatCompletionRequestMessage {
                role: m.role.into(),
                content: m.msg,
                name: None,
            });
        }
        messages.push(ChatCompletionRequestMessage {
            role: Role::Assistant.into(),
            content: "Understood.".to_string(),
            name: None,
        });

        for example in input.examples {
            messages.push(ChatCompletionRequestMessage {
                role: example.role.into(),
                content: example.msg,
                name: None,
            });
        }

        for message in input.chat {
            messages.push(ChatCompletionRequestMessage {
                role: message.role.into(),
                content: message.msg,
                name: None,
            });
        }

        let temperature = self.temperature;
        CreateChatCompletionRequest {
            model: self.model.to_string(),
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
        input: ChatInput,
        max_tokens: Option<usize>,
    ) -> Result<ModelResponse, Error> {
        let input = self.prepare_chat_completion_request(input, max_tokens);

        debug!("Sending request to the model");
        let res = self.client.chat().create(input).await;
        if let Err(e) = &res {
            error!(error = ?e, "Error from the model");
        }
        let res = res?;
        debug!(usage = ?res.usage, "Got a response from the model");

        let first = res.choices.first().ok_or(Error::NoResponseFromModel)?;

        let msg = first.message.content.clone();

        Ok(ModelResponse {
            msg,
            usage: res.usage.map(Into::into),
            finish_reason: first.finish_reason.clone(),
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

    // #[tokio::test]
    // async fn can_connect() {
    //     // let api_base = "https://api.openai.com/v1".to_string();
    //     let api_base = "http://hector:8000/v1".to_string();
    //     let openai_client = async_openai::Client::new().with_api_base(api_base);
    //
    //     let request = CreateChatCompletionRequest {
    //         model: "vicuna-7b-1.1".to_string(),
    //         messages: vec![ChatCompletionRequestMessage {
    //             role: async_openai::types::Role::User,
    //             content: "Hello, my name is Marcel".to_string(),
    //             name: None,
    //         }],
    //         temperature: Some(0.0),
    //         top_p: None,
    //         n: Some(1),
    //         stream: None,
    //         stop: None,
    //         max_tokens: Some(1024),
    //         presence_penalty: None,
    //         frequency_penalty: None,
    //         logit_bias: None,
    //         user: None,
    //     };
    //
    //     let response = openai_client.chat().create(request).await;
    //     println!("{:#?}", response);
    //
    //     let response = response.unwrap();
    //     println!("{:#?}", response);
    //
    //     println!("{}", response.choices.first().unwrap().message.content);
    // }

    #[tokio::test]
    async fn test_vicuna_sizes() {
        let model = build(SupportedModel::Vicuna7B1_1, None, None, None)
            .await
            .unwrap();

        assert_eq!(model.context_size().await, 2048);

        let input = ChatInput {
            context: vec![
                ChatEntry {
                    role: Role::System,
                    msg: "A chat between a user and an assistant.".to_string(),
                },
                ChatEntry {
                    role: Role::User,
                    msg: "Hello, my name is Marcel".to_string(),
                },
            ],
            examples: vec![],
            chat: vec![
                ChatEntry {
                    role: Role::Assistant,
                    msg: "Hello, Marcel, how are you doing today?".to_string(),
                },
                ChatEntry {
                    role: Role::User,
                    msg: "I am doing great, thanks for asking".to_string(),
                },
                ChatEntry {
                    role: Role::Assistant,
                    msg: "That's great to hear!".to_string(),
                },
            ],
        };

        let token_sz = model.num_tokens(input).await;

        assert_eq!(token_sz, 70);
    }

    #[tokio::test]
    async fn test_gpt3_sizes() {
        let model = build(SupportedModel::GPT3_5Turbo, None, None, None)
            .await
            .unwrap();

        assert_eq!(model.context_size().await, 4096);

        let input = ChatInput {
            context: vec![
                ChatEntry {
                    role: Role::System,
                    msg: "A chat between a user and an assistant.".to_string(),
                },
                ChatEntry {
                    role: Role::User,
                    msg: "Hello, my name is Marcel".to_string(),
                },
            ],
            examples: vec![],
            chat: vec![
                ChatEntry {
                    role: Role::Assistant,
                    msg: "Hello, Marcel, how are you doing today?".to_string(),
                },
                ChatEntry {
                    role: Role::User,
                    msg: "I am doing great, thanks for asking".to_string(),
                },
                ChatEntry {
                    role: Role::Assistant,
                    msg: "That's great to hear!".to_string(),
                },
            ],
        };

        let token_sz = model.num_tokens(input).await;

        assert_eq!(token_sz, 75);
    }
}
