use std::fmt::Debug;

use tracing::trace;

use crate::context::{ChatEntry, ChatHistory, ChatHistoryDump};
use crate::models::{ModelResponse, Role};
use crate::prompt::Task;
use crate::tools::invocation::InvocationError;
use crate::tools::toolbox::{InvokeResult, Toolbox};
use crate::tools::{TerminationMessage, ToolUseError};
use crate::{prompt, Error, SapiensConfig};

/// A chain - not yet specialized to a task
#[derive(Clone)]
pub struct Chain {
    toolbox: Toolbox,
    config: SapiensConfig,
    prompt_manager: prompt::Manager,
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
    pub async fn new(toolbox: Toolbox, config: SapiensConfig) -> Self {
        let max_token = { config.model.context_size().await };
        let mut chat_history =
            ChatHistory::new(config.clone(), max_token, config.min_tokens_for_completion);

        // Add the prompts to the chat history
        let prompt_manager = prompt::Manager::new(toolbox.clone());
        prompt_manager
            .populate_chat_history(&mut chat_history)
            .await;

        Self {
            toolbox,
            config,
            chat_history,
            prompt_manager,
        }
    }

    /// Start a task
    pub async fn start_task(&self, task: String) -> Result<TaskChain, Error> {
        let task = self.prompt_manager.build_task_prompt(&task);

        let entry = ChatEntry {
            msg: task.to_string(),
            role: Role::User,
        };

        // clone and update
        let mut chain = self.clone();

        chain.chat_history.add_chitchat(entry).await?;

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
    ///
    /// Does not update the chat history
    #[tracing::instrument(skip(self))]
    pub async fn query_model(&mut self) -> Result<ModelResponse, Error> {
        // TODO(ssoudan) support different way of getting to a ModelResponse

        // - 2201.11903 - Chain of thought prompting - 2022
        // - 2205.11916 - Zeroshot reasoners - "Let's think step by step" - 2022
        // - 2207.05608 - Inner monologue - Different types of feedbacks - 2022
        // - 2302.01560 - DEPS - Describe, explain, plan, select stages. Feb 2023
        // - 2210.03629 - ReAct - Reasoning + Action - Mar 2023
        // - 2303.11366 - Reflexion - heuristic + self-reflection - Mar 2023
        // - 2303.17071 - DERA - Distinct roles+responsibilities - Mar 2023

        // TODO(ssoudan) should the chat_history be more structured? SARSA-like?
        // More roles, more type of information, more metadata, etc.?

        let input = self.chain.chat_history.make_input();

        trace!(min_tokens = self.chain.config.min_tokens_for_completion, max_tokens = self.chain.config.max_tokens, input = ?input, "Querying model with {} entries", input.chat.len());
        let res = self
            .chain
            .config
            .model
            .query(input, self.chain.config.max_tokens)
            .await?;
        trace!(res = ?res, "Got model response");
        Ok(res)
    }

    /// Add a chat entry to the chat history
    async fn add_to_chat_history(&mut self, entry: ChatEntry) -> Result<usize, Error> {
        Ok(self.chain.chat_history.add_chitchat(entry).await?)
    }

    /// Try to find the tool invocation from the chat message and invoke the
    /// corresponding tool.
    ///
    /// See [`crate::invoke_tool`] for more details.
    #[tracing::instrument(skip(self, data))]
    pub async fn invoke_tool(&self, data: &str) -> InvokeResult {
        let toolbox = self.chain.toolbox.clone();
        crate::tools::toolbox::invoke_tool(toolbox, data).await
    }

    /// Generate a new prompt for the assistant based on the response from the
    /// Tool.
    ///
    /// If the response is too long, we add an error message to the chat history
    pub async fn on_tool_success(
        &mut self,
        tool_name: &str,
        available_invocation_count: usize,
        query: ChatEntry,
        result: String,
    ) -> Result<ChatEntry, Error> {
        // add the query to the chat history
        self.add_to_chat_history(query).await?;

        // add the response to the chat history
        let msg = self
            .task
            .action_success_prompt(tool_name, available_invocation_count, result);

        // if the response is too long, we add an error message to the chat history
        // instead
        const MAX_RESPONSE_CHAR: usize = 2048;
        if msg.len() > MAX_RESPONSE_CHAR {
            let e = ToolUseError::InvocationFailed(format!(
                "The response is too long ({}B). Max allowed is {}B. Ask for a shorter response or use SandboxedPython Tool to process the response the data.",
                msg.len(),
                MAX_RESPONSE_CHAR
            ));
            let msg = self.task.action_failed_prompt(tool_name, &e);

            // add an error message to the chat history
            self.add_to_chat_history(ChatEntry {
                msg: msg.clone(),
                role: Role::User,
            })
            .await?;

            return Err(Error::ActionResponseTooLong(msg));
        }

        let entry = ChatEntry {
            msg,
            role: Role::User,
        };
        self.add_to_chat_history(entry.clone()).await?;

        Ok(entry)
    }

    /// Generate a new prompt for the assistant based on the error from the
    /// Tool invocation.
    pub async fn on_tool_failure(
        &mut self,
        tool_name: &String,
        query: ChatEntry,
        e: ToolUseError,
    ) -> Result<ChatEntry, Error> {
        // add the query to the chat history
        self.add_to_chat_history(query).await?;

        // add the error message to the chat history
        let msg = self.task.action_failed_prompt(tool_name, &e);

        let entry = ChatEntry {
            msg,
            role: Role::User,
        };

        self.add_to_chat_history(entry.clone()).await?;

        Ok(entry)
    }

    /// Generate a new prompt for the assistant based on the invocation parsing.
    pub async fn on_invocation_failure(
        &mut self,
        query: ChatEntry,
        e: InvocationError,
    ) -> Result<ChatEntry, Error> {
        // add the query to the chat history
        self.add_to_chat_history(query).await?;

        // add the error message to the chat history
        let msg = self.task.invalid_action_prompt(&e);

        let entry = ChatEntry {
            msg,
            role: Role::User,
        };

        self.add_to_chat_history(entry.clone()).await?;

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
    pub fn chat_history(&self) -> ChatHistoryDump {
        self.chain.chat_history.dump()
    }
}
