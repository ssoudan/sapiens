pub mod openai;
pub mod vertex_ai;

use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::context::ChatEntry;

/// A model reference
pub type ModelRef = Arc<Box<dyn Model>>;

/// Errors from the models
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The openai error
    #[error("Model invocation failed")]
    OpenAIError(#[from] openai::OpenAIError),
    /// No response from the model
    #[error("No response from the model")]
    NoResponseFromModel,
    /// The model is not supported
    #[error("Model not supported: {0}")]
    ModelNotSupported(String),
    /// Vertex AI error
    #[error("Vertex AI error: {0}")]
    VertexAIError(#[from] gcp_vertex_ai_generative_language::Error),
    /// Filtered output
    #[error("Filtered output")]
    Filtered,
}

/// Roles in the conversation
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The system
    System,
    /// The user
    #[default]
    User,
    /// The assistant
    Assistant,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}

// FUTURE(ssoudan) support pure completion API
// FUTURE(ssoudan) support ability to run multistep chains to come to response
// FUTURE(ssoudan) support local llam.cpp models

/// Something that can count the number of tokens in a chat entry
#[async_trait::async_trait]
pub trait ChatEntryTokenNumber {
    /// Count the number of tokens in the chat entries
    async fn num_tokens(&self, input: ChatInput) -> usize;

    /// Get the context size
    async fn context_size(&self) -> usize;
}

/// A chat input
#[derive(Debug, Clone)]
pub struct ChatInput {
    /// The context
    pub(crate) context: Vec<ChatEntry>,
    /// The examples
    pub(crate) examples: Vec<(ChatEntry, ChatEntry)>,
    /// The chat history
    pub(crate) chat: Vec<ChatEntry>,
}

/// A model
#[async_trait::async_trait]
pub trait Model: ChatEntryTokenNumber + Send + Sync {
    /// Query the model
    async fn query(
        &self,
        input: ChatInput,
        max_tokens: Option<usize>,
    ) -> Result<ModelResponse, Error>;
}

/// Response from a language model
#[derive(Clone)]
pub struct ModelResponse {
    // TODO(ssoudan) support getting multiple candidates
    /// The message
    pub msg: String,
    /// The usage
    pub usage: Option<Usage>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

impl Debug for ModelResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ModelResponse {{ ")?;
        write!(f, "msg: \n{}, \n", &self.msg)?;
        if let Some(usage) = &self.usage {
            writeln!(f, "usage: {:#?}, ", usage)?;
        }
        if let Some(finish_reason) = &self.finish_reason {
            writeln!(f, "finish_reason: {}, ", &finish_reason)?;
        }
        write!(f, "}}")
    }
}

/// Token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// The number of tokens used for the prompt
    pub prompt_tokens: u32,
    /// The number of tokens used for the completion
    pub completion_tokens: u32,
    /// The total number of tokens used
    pub total_tokens: u32,
}

/// Supported models
#[derive(Clone, Serialize, Deserialize, Default)]
pub enum SupportedModel {
    /// GPT 3.5 Turbo
    #[default]
    GPT3_5Turbo,
    /// GPT 3.5 Turbo 0613
    GPT3_5Turbo0613,
    /// GPT 3.5 Turbo 16k
    GPT3_5Turbo16k,
    /// Vicuna 7B 1.1
    Vicuna7B1_1,
    /// Vicuna 13B 1.1
    Vicuna13B1_1,
    /// GCP "chat-bison-001"
    ChatBison001,
}

impl Display for SupportedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedModel::GPT3_5Turbo => write!(f, "gpt-3.5-turbo"),
            SupportedModel::GPT3_5Turbo0613 => write!(f, "gpt-3.5-turbo-0613"),
            SupportedModel::GPT3_5Turbo16k => write!(f, "gpt-3.5-turbo-16k"),
            SupportedModel::Vicuna7B1_1 => write!(f, "vicuna-7b-1.1"),
            SupportedModel::Vicuna13B1_1 => write!(f, "vicuna-13b-1.1"),
            SupportedModel::ChatBison001 => write!(f, "chat-bison-001"),
        }
    }
}

impl Debug for SupportedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedModel::GPT3_5Turbo => write!(f, "gpt-3.5-turbo"),
            SupportedModel::GPT3_5Turbo0613 => write!(f, "gpt-3.5-turbo-0613"),
            SupportedModel::GPT3_5Turbo16k => write!(f, "gpt-3.5-turbo-16k"),
            SupportedModel::Vicuna7B1_1 => write!(f, "vicuna-7b-1.1"),
            SupportedModel::Vicuna13B1_1 => write!(f, "vicuna-13b-1.1"),
            SupportedModel::ChatBison001 => write!(f, "chat-bison-001"),
        }
    }
}

impl FromStr for SupportedModel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gpt-3.5-turbo" => Ok(Self::GPT3_5Turbo),
            "gpt-3.5-turbo-0613" => Ok(Self::GPT3_5Turbo0613),
            "gpt-3.5-turbo-16k" => Ok(Self::GPT3_5Turbo16k),
            "vicuna-7b-1.1" => Ok(Self::Vicuna7B1_1),
            "vicuna-13b-1.1" => Ok(Self::Vicuna13B1_1),
            "chat-bison-001" => Ok(Self::ChatBison001),
            _ => Err(Error::ModelNotSupported(s.to_string())),
        }
    }
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for SupportedModel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            SupportedModel::GPT3_5Turbo,
            SupportedModel::GPT3_5Turbo0613,
            SupportedModel::GPT3_5Turbo16k,
            SupportedModel::Vicuna7B1_1,
            SupportedModel::Vicuna13B1_1,
            SupportedModel::ChatBison001,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            SupportedModel::GPT3_5Turbo => Some(clap::builder::PossibleValue::new("gpt-3.5-turbo")),
            SupportedModel::GPT3_5Turbo0613 => {
                Some(clap::builder::PossibleValue::new("gpt-3.5-turbo-0613"))
            }
            SupportedModel::GPT3_5Turbo16k => {
                Some(clap::builder::PossibleValue::new("gpt-3.5-turbo-16k"))
            }
            SupportedModel::Vicuna7B1_1 => Some(clap::builder::PossibleValue::new("vicuna-7b-1.1")),
            SupportedModel::Vicuna13B1_1 => {
                Some(clap::builder::PossibleValue::new("vicuna-13b-1.1"))
            }
            SupportedModel::ChatBison001 => {
                Some(clap::builder::PossibleValue::new("chat-bison-001"))
            }
        }
    }
}
