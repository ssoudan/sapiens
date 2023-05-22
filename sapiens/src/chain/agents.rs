// FUTURE(ssoudan) more agent types
// ------
// Researcher: Observation + Direction to investigate
// Decider: Decision
// User: Propose Tasks - expect Conclusion
// Assistant: Observe, Orient, Decide, Act
// Reporter: from the env, the user, the tools,
// Introspecter: ?

use tracing::trace;

use crate::chain::{Agent, Context, Message, Outcome};
use crate::context::{ChatEntry, ChatHistory};
use crate::models::Role;
use crate::tools::toolbox::Toolbox;
use crate::tools::ToolUseError;
use crate::{context, prompt, SapiensConfig, WeakRuntimeObserver};

/// Error from the agent
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to add to the chat history
    #[error("Failed to add to the chat history: {0}")]
    ChatHistoryError(#[from] context::Error),
    /// Error from the model
    #[error("Error from the model: {0}")]
    ModelError(#[from] crate::models::Error),
}

// FUTURE(ssoudan) parameterize the prompt manager

/// An OODA agent
pub struct OODAAgent {
    prompt_manager: prompt::Manager,
    config: SapiensConfig,
    observer: WeakRuntimeObserver,
}

impl OODAAgent {
    /// Create a new [`OODAAgent`].
    pub async fn new(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Self {
        let prompt_manager = prompt::Manager::new(toolbox);
        Self {
            prompt_manager,
            config,
            observer,
        }
    }

    async fn convert_context_to_chat_history(
        &self,
        context: &Context,
    ) -> Result<ChatHistory, Error> {
        // Create a new chat history
        let max_token = { self.config.model.context_size().await };
        let mut chat_history = ChatHistory::new(
            self.config.clone(),
            max_token,
            self.config.min_tokens_for_completion,
        );

        // Add the prompts to the chat history
        self.prompt_manager
            .populate_chat_history(&mut chat_history)
            .await;

        // Convert the context to a chat history
        // - get the latest 'Task' from the context
        let task = context.get_latest_task().unwrap();
        let task = self.prompt_manager.build_task_prompt(&task);

        // - get the actions and (results|errors)
        for m in &context.messages {
            match m {
                Message::Action { content, .. } => {
                    // Add the action to the chat history as a message from the Assistant
                    chat_history
                        .add_chitchat(ChatEntry {
                            msg: content.to_string(),
                            role: Role::Assistant,
                        })
                        .await?;
                }
                Message::ActionResult {
                    invocation_count,
                    tool_name,
                    outcome,
                    ..
                } => {
                    let entry = match outcome {
                        Outcome::Success { result } => {
                            let msg = task.action_success_prompt(
                                tool_name.clone().unwrap_or("unknown".to_string()),
                                *invocation_count,
                                result,
                            );

                            // if the response is too long, we add an error message to the chat
                            // history instead
                            const MAX_RESPONSE_CHAR: usize = 2048;
                            if msg.len() > MAX_RESPONSE_CHAR {
                                let msg = format!("The response is too long ({}B). Max allowed is {}B. Ask for a shorter response or use SandboxedPython Tool to process the response the data.",
                                                         msg.len(),MAX_RESPONSE_CHAR);
                                let e = ToolUseError::InvocationFailed(msg);
                                let msg = task.action_failed_prompt(
                                    tool_name.clone().unwrap_or("unknown".to_string()),
                                    &e,
                                );

                                // add an error message to the chat history
                                ChatEntry {
                                    msg: msg.clone(),
                                    role: Role::User,
                                }
                            } else {
                                ChatEntry {
                                    msg,
                                    role: Role::User,
                                }
                            }
                        }
                        Outcome::NoValidInvocationsFound { e } => {
                            let msg = task.invalid_action_prompt(e);
                            ChatEntry {
                                msg,
                                role: Role::User,
                            }
                        }
                        Outcome::NoInvocationsFound { e } => {
                            let msg = task.invalid_action_prompt(e);
                            ChatEntry {
                                msg,
                                role: Role::User,
                            }
                        }
                        Outcome::ToolUseError { e } => {
                            let msg = task.action_failed_prompt(
                                tool_name.clone().unwrap_or("unknown".to_string()),
                                e,
                            );
                            ChatEntry {
                                msg,
                                role: Role::User,
                            }
                        }
                    };

                    // Add the result to the chat history
                    chat_history.add_chitchat(entry).await?;
                }
                _ => {
                    // Nothing
                }
            }
        }

        if chat_history.is_chitchat_empty() {
            // Add the recurring prompts to the chat history
            chat_history
                .add_chitchat(ChatEntry {
                    msg: task.to_prompt(),
                    role: Role::User,
                })
                .await?;
        }

        Ok(chat_history)
    }
}

#[async_trait::async_trait]
impl Agent for OODAAgent {
    type Error = Error;

    async fn act(&self, context: &Context) -> Result<Message, Error> {
        let chat_history = self.convert_context_to_chat_history(context).await?;

        // Query the model
        let input = chat_history.make_input();

        trace!(
            min_tokens = self.config.min_tokens_for_completion,
            max_tokens = self.config.max_tokens,
            "Querying model with {} entries",
            input.chat.len()
        );

        let res = self
            .config
            .model
            .query(input, self.config.max_tokens)
            .await?;

        trace!(res = ?res, "Got model response");

        // Show the message from the assistant
        if let Some(observer) = self.observer.upgrade() {
            observer
                .lock()
                .await
                .on_model_update(res.clone().into())
                .await;
        }

        // Return the response as an Action message
        Ok(Message::Action {
            content: res.msg,
            usage: res.usage,
        })
    }
}
