pub mod openai;
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

// TODO(ssoudan)
// Support:
// - the token length estimation too
// - different prompting prefixes
// - configs
// - ability to run multistep chains to come to response

// TODO(ssoudan) support local llam.cpp models

/// Something that can count the number of tokens in a chat entry
#[async_trait::async_trait]
pub trait ChatEntryTokenNumber {
    /// Count the number of tokens in the chat entries
    async fn num_tokens(&self, entries: &[ChatEntry]) -> usize;

    /// Get the context size
    async fn context_size(&self) -> usize;
}

/// A model
#[async_trait::async_trait]
pub trait Model: ChatEntryTokenNumber + Send + Sync {
    /// Query the model
    async fn query(
        &self,
        entries: &[&ChatEntry],
        max_tokens: Option<usize>,
    ) -> Result<ModelResponse, Error>;
}

/// Response from a language model
#[derive(Debug, Clone)]
pub struct ModelResponse {
    /// The message
    pub msg: String,
    /// The usage
    pub usage: Option<Usage>,
}

/// Token usage
#[derive(Debug, Clone)]
pub struct Usage {
    /// The number of tokens used for the prompt
    pub prompt_tokens: u32,
    /// The number of tokens used for the completion
    pub completion_tokens: u32,
    /// The total number of tokens used
    pub total_tokens: u32,
}
