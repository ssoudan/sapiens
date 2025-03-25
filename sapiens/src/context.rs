//! Maintain the context for the bot.
use std::fmt::{Debug, Formatter};

use tracing::trace;

use crate::chains::Message;
use crate::models::{ChatInput, Role};
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
#[derive(Clone)]
pub struct ChatEntry {
    /// The role
    pub role: Role,
    /// The message
    pub msg: String,
}

impl Debug for ChatEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.role, self.msg)
    }
}

/// Maintain a chat history that can be truncated (from the head) to ensure
/// we have enough tokens to complete the task
///
/// The prompt is the part of the history that we want to stay at the top of the
/// history. The chitchat is the rest of the history.
///
/// Add the prompting messages to the history with [`ChatHistory::add_prompts`].
///
/// To ensure we have enough tokens to complete the task, we truncate the
/// chitchat history when new messages are added - with
/// [`ChatHistory::add_chitchat`].
#[derive(Clone)]
pub struct ChatHistory {
    /// Config - contains a ref to the model
    config: SapiensConfig,
    /// The maximum number of tokens we can have in the input for the model
    max_token: usize,
    /// The 'context' - first messages.
    context: Vec<ChatEntry>,
    /// The examples
    examples: Vec<(ChatEntry, ChatEntry)>,
    /// The other messages
    chitchat: Vec<ChatEntry>,
}

impl Debug for ChatHistory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatHistory")
            .field("config", &self.config)
            .field("max_token", &self.max_token)
            .field("context", &self.context)
            .field("examples", &self.examples)
            .field("chitchat", &self.chitchat)
            .finish()
    }
}

impl ChatHistory {
    /// Create a new chat history
    #[must_use]
    pub const fn new(config: SapiensConfig, max_token: usize) -> Self {
        Self {
            config,
            max_token,
            context: vec![],
            examples: vec![],
            chitchat: vec![],
        }
    }

    /// Set the context msg
    pub fn set_context(&mut self, context: Vec<ChatEntry>) {
        self.context = context;
    }

    /// add a prompt to the history
    pub fn add_example(&mut self, user: String, bot: String) {
        let msg_user = ChatEntry {
            role: Role::User,
            msg: user,
        };

        let msg_bot = ChatEntry {
            role: Role::Assistant,
            msg: bot,
        };

        self.examples.push((msg_user, msg_bot));
    }

    /// add a message to the chitchat history, and prune the history if needed
    /// returns the number of messages in the chitchat history
    pub fn add_chitchat(&mut self, entry: ChatEntry) {
        // ensure we don't have two consecutive messages from the same role
        if let Some(last) = self.chitchat.last() {
            if last.role == entry.role {
                self.chitchat.pop();
            }
        }

        self.chitchat.push(entry);
    }

    /// Prepare the input for the model
    pub(crate) fn make_input(&self) -> ChatInput {
        ChatInput {
            context: self.context.clone(),
            examples: self.examples.clone(),
            chat: self.chitchat.clone(),
        }
    }

    /// Is the chitchat history empty?
    pub(crate) const fn is_chitchat_empty(&self) -> bool {
        self.chitchat.is_empty()
    }

    /// uses [`tiktoken_rs::num_tokens_from_messages`] prune
    /// the chitchat history starting from the head until we have enough
    /// tokens to complete the task
    pub async fn purge(&mut self) -> Result<usize, Error> {
        if self.chitchat.is_empty() {
            return Ok(0);
        }

        trace!(
            max_token = self.max_token,
            min_tokens_for_completion = self.config.min_tokens_for_completion,
            "purging history"
        );

        // start by pruning the examples
        while !self.examples.is_empty() {
            let input = self.make_input();
            let num_tokens = self.config.model.num_tokens(input).await;
            trace!(
                max_token = self.max_token,
                min_tokens_for_completion = self.config.min_tokens_for_completion,
                len = self.examples.len(),
                num_tokens,
                "purging history - examples"
            );

            if num_tokens <= self.max_token - self.config.min_tokens_for_completion {
                return Ok(self.chitchat.len());
            }
            // remove oldest message
            self.examples.remove(0);
        }

        // loop until we have enough available tokens to complete the task
        while self.chitchat.len() > 1 {
            let input = self.make_input();
            let num_tokens = self.config.model.num_tokens(input).await;
            trace!(
                max_token = self.max_token,
                min_tokens_for_completion = self.config.min_tokens_for_completion,
                len = self.chitchat.len(),
                num_tokens,
                "purging history - loop"
            );

            if num_tokens <= self.max_token - self.config.min_tokens_for_completion {
                return Ok(self.chitchat.len());
            }

            // remove oldest message
            self.chitchat.remove(0);
        }

        let input = self.make_input();
        let num_tokens = self.config.model.num_tokens(input).await;

        if num_tokens <= self.max_token - self.config.min_tokens_for_completion {
            return Ok(self.chitchat.len());
        }

        Err(Error::PromptTooLong)
    }

    /// iterate over the prompt and chitchat messages
    pub fn iter(&self) -> impl Iterator<Item = &ChatEntry> {
        self.context
            .iter()
            .chain(self.examples.iter().flat_map(|(a, b)| vec![a, b]))
            .chain(self.chitchat.iter())
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
}

impl From<&ChatHistory> for Vec<ChatEntry> {
    fn from(val: &ChatHistory) -> Self {
        val.iter().cloned().collect()
    }
}

/// A dump of the chat history
#[allow(clippy::module_name_repetitions)]
pub struct ContextDump {
    /// the messages
    pub messages: Vec<Message>,
}

/// A formatter for a chat entry
pub trait MessageFormatter {
    /// format the message
    fn format(&self, msg: &Message) -> String;
}

impl ContextDump {
    /// format the history using the given formatter
    pub fn format<T>(&self, formatter: &T) -> Vec<String>
    where
        T: MessageFormatter + ?Sized,
    {
        self.messages
            .iter()
            .map(|msg| formatter.format(msg))
            .collect::<Vec<_>>()
    }
}
