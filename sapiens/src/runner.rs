use std::fmt::Debug;

use tracing::debug;

use crate::context::{ChatEntry, ChatHistory};
use crate::openai::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Role};
use crate::prompt::Task;
use crate::tools::{TerminationMessage, ToolUseError, Toolbox};
use crate::{prompt, Client, Config, Error};

/// A chain - not yet specialized to a task
#[derive(Clone)]
pub struct Chain {
    toolbox: Toolbox,
    config: Config,
    prompt_manager: prompt::Manager,
    openai_client: Client,
    /// With the initial prompt
    chat_history: ChatHistory,
}

impl Debug for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chain")
            // .field("toolbox", &self.toolbox)
            .field("config", &self.config)
            // .field("openai_client", &self.openai_client)
            // .field("chat_history", &self.chat_history)
            .finish()
    }
}

impl Chain {
    /// Create a new chain
    pub async fn new(toolbox: Toolbox, config: Config, openai_client: Client) -> Self {
        let mut chat_history =
            ChatHistory::new(config.model.clone(), config.min_token_for_completion);

        // Add the prompts to the chat history
        let prompt_manager = prompt::Manager::new(toolbox.clone());
        prompt_manager
            .populate_chat_history(&mut chat_history)
            .await;

        Self {
            toolbox,
            config,
            openai_client,
            chat_history,
            prompt_manager,
        }
    }

    /// Start a task
    pub fn start_task(&self, task: String) -> Result<TaskChain, Error> {
        let task = self.prompt_manager.build_task_prompt(&task);

        let entry = ChatEntry {
            msg: task.to_string(),
            role: Role::User,
        };

        // clone and update
        let mut chain = self.clone();

        chain.chat_history.add_chitchat(entry)?;

        Ok(TaskChain { chain, task })
    }
}

/// A chain for a specific task
pub struct TaskChain {
    chain: Chain,
    task: Task,
}

impl Debug for TaskChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskChain")
            .field("chain", &self.chain)
            .field("task", &self.task)
            .finish()
    }
}

impl TaskChain {
    /// Query the model
    #[tracing::instrument(skip(self))]
    pub async fn query_model(&mut self) -> Result<ChatEntry, Error> {
        let input = self.prepare_chat_completion_request();

        debug!("Sending request to OpenAI");
        let res = self.chain.openai_client.chat().create(input).await?;
        debug!(usage = ?res.usage, "Got a response from OpenAI");

        let first = res.choices.first().ok_or(Error::NoResponseFromModel)?;

        let msg = first.message.content.clone();

        // Add the response to the chat history
        let entry = ChatEntry {
            role: Role::Assistant,
            msg,
        };

        self.add_to_chat_history(entry.clone())?;

        Ok(entry)
    }

    /// prepare the [`ChatCompletionRequest`] to be passed to OpenAI
    fn prepare_chat_completion_request(&self) -> CreateChatCompletionRequest {
        let messages: Vec<ChatCompletionRequestMessage> = (&self.chain.chat_history).into();
        CreateChatCompletionRequest {
            model: self.chain.config.model.clone(),
            messages,
            temperature: None,
            top_p: None,
            n: Some(1),
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }

    /// Add a chat entry to the chat history
    fn add_to_chat_history(&mut self, entry: ChatEntry) -> Result<usize, Error> {
        Ok(self.chain.chat_history.add_chitchat(entry)?)
    }

    /// Try to find the tool invocation from the chat message and invoke the
    /// corresponding tool.
    ///
    /// See [`crate::invoke_tool`] for more details.
    #[tracing::instrument(skip(self, data))]
    pub async fn invoke_tool(&self, data: &str) -> (String, Result<String, ToolUseError>) {
        let toolbox = self.chain.toolbox.clone();
        crate::tools::invoke_tool(toolbox, data).await
    }

    /// Generate a new prompt for the assistant based on the response from the
    /// Tool.
    ///
    /// If the response is too long, we add an error message to the chat history
    pub fn on_tool_success(
        &mut self,
        tool_name: String,
        response: String,
    ) -> Result<ChatEntry, Error> {
        let msg = self.task.action_success_prompt(&tool_name, response);

        // if the response is too long, we add an error message to the chat history
        // instead
        const MAX_RESPONSE_CHAR: usize = 2048;
        if msg.len() > MAX_RESPONSE_CHAR {
            let e = ToolUseError::InvocationFailed(format!(
                "The response is too long ({}B). Max allowed is {}B. Ask for a shorter response or use SandboxedPython Tool to process the response the data.",
                msg.len(),
                MAX_RESPONSE_CHAR
            ));
            let msg = self.task.action_failed_prompt(&tool_name, &e);

            // add an error message to the chat history
            self.add_to_chat_history(ChatEntry {
                msg: msg.clone(),
                role: Role::User,
            })?;

            return Err(Error::ActionResponseTooLong(msg));
        }

        let entry = ChatEntry {
            msg,
            role: Role::User,
        };
        self.add_to_chat_history(entry.clone())?;

        Ok(entry)
    }

    /// Generate a new prompt for the assistant based on the error from the
    /// Tool invocation.
    pub fn on_tool_failure(
        &mut self,
        tool_name: String,
        e: ToolUseError,
    ) -> Result<ChatEntry, Error> {
        let msg = self.task.action_failed_prompt(tool_name, &e);

        let entry = ChatEntry {
            msg,
            role: Role::User,
        };

        self.add_to_chat_history(entry.clone())?;

        Ok(entry)
    }

    /// Return the termination messages if the chain is terminated or `None`
    pub async fn is_terminal(&self) -> Option<Vec<TerminationMessage>> {
        let t = self.chain.toolbox.termination_messages().await;
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }

    /// Return the chat history
    pub fn chat_history(&self) -> &ChatHistory {
        &self.chain.chat_history
    }
}
