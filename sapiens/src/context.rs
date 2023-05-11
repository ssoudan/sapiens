//! Maintain the context for the bot.
use std::fmt::{Debug, Formatter};

use crate::models::Role;
use crate::SapiensConfig;

/// A trait for formatting entries for the chat history
pub trait ChatEntryFormatter {
    /// Format the entry
    fn format(&self, entry: &ChatEntry) -> String;
}

/// An error that can occur when adding a prompt to the chat history
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The prompt is too long
    #[error("The prompt is too long")]
    PromptTooLong,
}

/// A history entry
#[derive(Debug, Clone)]
pub struct ChatEntry {
    /// The role
    pub role: Role,
    /// The message
    pub msg: String,
}

/// Maintain a chat history that can be truncated (from the head) to ensure
/// we have enough tokens to complete the task
///
/// The prompt is the part of the history that we want to stay at the top of the
/// history. The chitchat is the rest of the history.
///
/// Add the prompting messages to the history with [ChatHistory::add_prompts].
///
/// To ensure we have enough tokens to complete the task, we truncate the
/// chitchat history when new messages are added - with
/// [ChatHistory::add_chitchat].
#[derive(Clone)]
pub struct ChatHistory {
    /// Config - contains a ref to the model
    config: SapiensConfig,
    /// The maximum number of tokens we can have in the history for the model
    max_token: usize,
    /// The minimum number of tokens we need to complete the task
    min_token_for_completion: usize,
    /// The 'prompt' (aka messages we want to stay at the top of the history)
    prompt: Vec<ChatEntry>,
    /// Num token for the prompt
    prompt_num_tokens: usize,
    /// The other messages
    chitchat: Vec<ChatEntry>,
}

impl Debug for ChatHistory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatHistory")
            .field("max_token", &self.max_token)
            .field("min_token_for_completion", &self.min_token_for_completion)
            .field("prompt_num_tokens", &self.prompt_num_tokens)
            .finish()
    }
}

impl ChatHistory {
    /// Create a new chat history
    pub fn new(config: SapiensConfig, max_token: usize, min_token_for_completion: usize) -> Self {
        Self {
            config,
            max_token,
            min_token_for_completion,
            prompt: vec![],
            prompt_num_tokens: 0,
            chitchat: vec![],
        }
    }

    /// add a prompt to the history
    pub async fn add_prompts(&mut self, prompts: &[(Role, String)]) {
        for (role, content) in prompts {
            let msg = ChatEntry {
                role: role.clone(),
                msg: content.clone(),
            };
            self.prompt.push(msg);
        }

        // update the prompt_num_tokens
        self.prompt_num_tokens = self.config.model.num_tokens(&self.prompt).await;
    }

    /// add a message to the chitchat history, and prune the history if needed
    /// returns the number of messages in the chitchat history
    pub async fn add_chitchat(&mut self, entry: ChatEntry) -> Result<usize, Error> {
        self.chitchat.push(entry);

        // prune the history if needed
        self.purge().await
    }

    /// uses [tiktoken_rs::num_tokens_from_messages] prune
    /// the chitchat history starting from the head until we have enough
    /// tokens to complete the task
    async fn purge(&mut self) -> Result<usize, Error> {
        // FIXME(ssoudan) preserve the alternance of roles

        let token_budget = self.max_token.saturating_sub(self.prompt_num_tokens);

        if token_budget == 0 {
            // we can't even fit the prompt
            self.chitchat = vec![];
            return Err(Error::PromptTooLong);
        }

        // loop until we have enough available tokens to complete the task
        {
            while self.chitchat.len() > 1 {
                let num_tokens = self.config.model.num_tokens(&self.chitchat).await;
                if num_tokens <= token_budget - self.min_token_for_completion {
                    return Ok(self.chitchat.len());
                }
                self.chitchat.remove(0);
            }
        }

        Ok(self.chitchat.len())
    }

    /// iterate over the prompt and chitchat messages
    pub fn iter(&self) -> impl Iterator<Item = &ChatEntry> {
        self.prompt.iter().chain(self.chitchat.iter())
    }

    /// format the history using the given formatter
    pub fn format<T>(&self, formatter: &T) -> Vec<String>
    where
        T: ChatEntryFormatter + ?Sized,
    {
        self.iter()
            .map(|msg| formatter.format(msg))
            .collect::<Vec<_>>()
    }

    /// dump the history
    pub fn dump(&self) -> ChatHistoryDump {
        ChatHistoryDump {
            messages: self.iter().cloned().collect(),
        }
    }
}

impl From<&ChatHistory> for Vec<ChatEntry> {
    fn from(val: &ChatHistory) -> Self {
        val.iter().cloned().collect()
    }
}

/// A dump of the chat history
pub struct ChatHistoryDump {
    /// the messages
    pub messages: Vec<ChatEntry>,
}

impl ChatHistoryDump {
    /// format the history using the given formatter
    pub fn format<T>(&self, formatter: &T) -> Vec<String>
    where
        T: ChatEntryFormatter + ?Sized,
    {
        self.messages
            .iter()
            .map(|msg| formatter.format(msg))
            .collect::<Vec<_>>()
    }
}
