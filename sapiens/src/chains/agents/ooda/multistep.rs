use std::fmt::{Debug, Formatter};

use tracing::{debug, trace};

use crate::chains::agents::{format_outcome, Error};
use crate::chains::{Context, Message};
use crate::context::{ChatEntry, ChatHistory};
use crate::models::Role;
use crate::tools::toolbox::Toolbox;
use crate::{chains, prompt, SapiensConfig, WeakRuntimeObserver};

const PREFIX: &str = r"You are part of a group of cooperating assistants named Sapiens. Use available tools to answer the question as best as you can.
You will collectively proceed iteratively using an OODA loop. Don't overstep your role.

- Action response will be provided. 
- Never produce the response of an Action. 
- Only use YAML for the Action.
- The loop will repeated until you have the answer to the original question. 
- No task is complete until the Conclude Tool is used to provide the answer. 
";

const TOOL_PREFIX: &str = r"
# The following are the ONLY Tools one can use for the Actions:
";

const OBSERVER_RESPONSE_FORMAT: &str = r"
# Format of your response

You must use the following format for your response. Comments are in bold and should be removed from your response.
====================
## Observations: 
**What do you know to be true? What do you you don't know? What are your sources? Note down important information for later.**
- <...>
====================
";

const ORIENTER_RESPONSE_FORMAT: &str = r"
# Format of your response

You must use the following format for your response. Comments are in bold and should be removed from your response.
====================
## Orientation: 
**Plan the intermediate objectives to answer complete the original task. Maintain a list of current objectives updated as you go.**
- <...>
```
====================
";

const DECIDER_RESPONSE_FORMAT: &str = r"
# Format of your response

You must use the following format for your response. Comments are in bold and should be removed from your response.
====================
## Decision: 
**Decide what to do first to answer the question. Why? How will you if it succeeds? How will you if it fails?**
- <...>
====================
";

const ACTOR_RESPONSE_FORMAT: &str = r"
# Format of your response

You must use the following format for your response. Comments are in bold and should be removed from your response.
====================
## The ONLY Action: 
**Take a single Action consisting of exactly one pair of `tool_name` and `parameters`. Never give more than one YAML. **
```yaml
tool_name: <ToolName>
parameters:
    <...>  
```
We will take further action based on the response.
====================

Notes: 
- Action has the following fields: `tool_name` and `parameters` ONLY.
- `parameters` uses the format specified for the Tool.
- One Action at a time. No more. No less.
";

const OBSERVER_PROTO_INITIAL_RESPONSE: &str = r"
## Observations:
- The given list to sort is [2, 3, 1, 4, 5].
- I need to sort this list in ascending order.
";

const OBSERVER_PROTO_SECOND_INPUT: &str = r#"
## Orientation:
- SandboxedPython can be used to sort the list.
- I need to provide only the `tool_name` and `parameters` fields for the SandboxedPython Tool.
- I expect the response of the Action to contains the field `stdout` with the sorted list and `stderr` empty.
- I need to use the Conclude Tool to terminate the task when I have the sorted list in plain text.
## Decision:
- We can use the sorted() function of Python to sort the list.
## The ONLY Action:
```yaml
tool_name: SandboxedPython
parameters:
  code: |
    lst = [2, 3, 1, 4, 5]
    sorted_list = sorted(lst)
    print(f"The sorted list is {sorted_list}")
```
We will take further action based on the response.
# Action SandboxedPython response:
```yaml
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
"#;

const OBSERVER_PROTO_SECOND_RESPONSE: &str = r"
## Observations:
- We needed to sort the list in ascending order.
- We have the response of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
";

const ORIENTER_PROTO_INITIAL_RESPONSE: &str = r"
## Orientation:
- SandboxedPython can be used to sort the list.
- I need to provide only the `tool_name` and `parameters` fields for the SandboxedPython Tool.
- I expect the response of the Action to contains the field `stdout` with the sorted list and `stderr` empty.
- I need to use the Conclude Tool to terminate the task when I have the sorted list in plain text.
";

const ORIENTER_PROTO_SECOND_INPUT: &str = r#"
## Decision:
- We can use the sorted() function of Python to sort the list.
## The ONLY Action:
```yaml
tool_name: SandboxedPython
parameters:
  code: |
    lst = [2, 3, 1, 4, 5]
    sorted_list = sorted(lst)
    print(f"The sorted list is {sorted_list}")
```
We will take further action based on the response.
# Action SandboxedPython response:
```yaml
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
## Observations:
- We needed to sort the list in ascending order.
- We have the response of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
"#;

const ORIENTER_PROTO_SECOND_RESPONSE: &str = r"
## Orientation:
- I know the answer to the original question.
- I need to provide the `tool_name` and `parameters` fields for the Conclude Tool.
";

const DECIDER_PROTO_INITIAL_RESPONSE: &str = r"
## Decision:
- We can use the sorted() function of Python to sort the list.
";

const DECIDER_PROTO_SECOND_INPUT: &str = r#"
## The ONLY Action:
```yaml
tool_name: SandboxedPython
parameters:
  code: |
    lst = [2, 3, 1, 4, 5]
    sorted_list = sorted(lst)
    print(f"The sorted list is {sorted_list}")
```
We will take further action based on the response.
# Action SandboxedPython response:
```yaml
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
## Observations:
- We needed to sort the list in ascending order.
- We have the response of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
## Orientation:
- I know the answer to the original question.
- I need to provide the `tool_name` and `parameters` fields for the Conclude Tool.
"#;

const DECIDER_PROTO_SECOND_RESPONSE: &str = r"
## Decision:
- Use the Conclude Tool to terminate the task with the sorted list.
";

const ACTOR_PROTO_INITIAL_RESPONSE: &str = r#"
## The ONLY Action:
```yaml
tool_name: SandboxedPython
parameters:
  code: |
    lst = [2, 3, 1, 4, 5]
    sorted_list = sorted(lst)
    print(f"The sorted list is {sorted_list}")
```
That's it for now. We will take further action based on the response.
"#;

const ACTOR_PROTO_SECOND_INPUT: &str = r"
# Action SandboxedPython response:
```yaml
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
## Observations:
- We needed to sort the list in ascending order.
- We have the response of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
## Orientation:
- I know the answer to the original question.
- I need to provide the `tool_name` and `parameters` fields for the Conclude Tool.
## Decision:
- Use the Conclude Tool to terminate the task with the sorted list.
";

const ACTOR_PROTO_SECOND_RESPONSE: &str = r"
## The ONLY Action:
```yaml
tool_name: Conclude
parameters:
  original_question: |
    Sort in ascending order: [2, 3, 1, 4, 5]
  conclusion: |
    The ascending sorted list is [1, 2, 3, 4, 5].
```
";

enum AgentRole {
    Observer { prompt_manager: prompt::Manager },
    Orienter { prompt_manager: prompt::Manager },
    Decider { prompt_manager: prompt::Manager },
    Actor { prompt_manager: prompt::Manager },
}

impl Debug for AgentRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observer { .. } => write!(f, "Observer"),
            Self::Orienter { .. } => write!(f, "Orienter"),
            Self::Decider { .. } => write!(f, "Decider"),
            Self::Actor { .. } => write!(f, "Actor"),
        }
    }
}

impl AgentRole {
    #[allow(clippy::too_many_lines)]
    async fn convert_context_to_chat_history(
        &self,
        mut chat_history: ChatHistory,
        context: &Context,
    ) -> Result<ChatHistory, Error> {
        // build the examples
        let examples = self.build_examples();

        let prompt_manager = match self {
            Self::Observer { prompt_manager }
            | Self::Orienter { prompt_manager }
            | Self::Decider { prompt_manager }
            | Self::Actor { prompt_manager } => prompt_manager,
        };

        // Add the prompts to the chat history
        prompt_manager
            .populate_chat_history(&mut chat_history, examples)
            .await;

        // Convert the context to a chat history
        // - get the latest 'Task' from the context
        let task = context.get_latest_task().unwrap();

        let task = prompt_manager.build_task_prompt(&task);

        // build the chat history from the context:
        // - group together Orientation, Decision, Action, ActionResult messages as a
        //   single chat entry from the User
        // - Observation messages become individual chat entries from the Assistant
        let mut user_msg = vec![];
        match self {
            Self::Observer { .. } => {
                for m in &context.messages {
                    match m {
                        Message::Observation { content, .. } => {
                            if !user_msg.is_empty() {
                                // Add the user message to the chat history as a message from the
                                // User
                                chat_history.add_chitchat(ChatEntry {
                                    msg: user_msg.join("\n"),
                                    role: Role::User,
                                });

                                user_msg.clear();
                            }

                            // Add the observation to the chat history as a message from the
                            // Observer
                            chat_history.add_chitchat(ChatEntry {
                                msg: content.clone(),
                                role: Role::Assistant,
                            });
                        }
                        Message::Orientation { content, .. }
                        | Message::Decision { content, .. }
                        | Message::Action { content, .. } => {
                            user_msg.push(content.clone());
                        }
                        Message::ActionResult {
                            invocation_count,
                            tool_name,
                            outcome,
                            ..
                        } => {
                            let entry =
                                format_outcome(&task, *invocation_count, tool_name, outcome);

                            user_msg.push(entry);
                        }
                        Message::Task { .. } => {
                            // Nothing
                        }
                    }
                }
            }

            Self::Orienter { .. } => {
                for m in &context.messages {
                    match m {
                        Message::Orientation { content, .. } => {
                            if !user_msg.is_empty() {
                                // Add the user message to the chat history as a message from the
                                // User
                                chat_history.add_chitchat(ChatEntry {
                                    msg: user_msg.join("\n"),
                                    role: Role::User,
                                });

                                user_msg.clear();
                            }

                            // Add the observation to the chat history as a message from the
                            // Observer
                            chat_history.add_chitchat(ChatEntry {
                                msg: content.clone(),
                                role: Role::Assistant,
                            });
                        }
                        Message::Observation { content, .. }
                        | Message::Decision { content, .. }
                        | Message::Action { content, .. } => {
                            user_msg.push(content.clone());
                        }
                        Message::ActionResult {
                            invocation_count,
                            tool_name,
                            outcome,
                            ..
                        } => {
                            let entry =
                                format_outcome(&task, *invocation_count, tool_name, outcome);

                            user_msg.push(entry);
                        }
                        Message::Task { .. } => {
                            // Nothing
                        }
                    }
                }
            }
            Self::Decider { .. } => {
                for m in &context.messages {
                    match m {
                        Message::Action { content, .. }
                        | Message::Observation { content, .. }
                        | Message::Orientation { content, .. } => {
                            user_msg.push(content.clone());
                        }
                        Message::Decision { content, .. } => {
                            if !user_msg.is_empty() {
                                // Add the user message to the chat history as a message from the
                                // User
                                chat_history.add_chitchat(ChatEntry {
                                    msg: user_msg.join("\n"),
                                    role: Role::User,
                                });

                                user_msg.clear();
                            }

                            chat_history.add_chitchat(ChatEntry {
                                msg: content.clone(),
                                role: Role::Assistant,
                            });
                        }

                        Message::ActionResult {
                            invocation_count,
                            tool_name,
                            outcome,
                            ..
                        } => {
                            let entry =
                                format_outcome(&task, *invocation_count, tool_name, outcome);

                            user_msg.push(entry);
                        }
                        Message::Task { .. } => {
                            // Nothing
                        }
                    }
                }
            }
            Self::Actor { .. } => {
                for m in &context.messages {
                    match m {
                        Message::Observation { content, .. }
                        | Message::Orientation { content, .. }
                        | Message::Decision { content, .. } => {
                            user_msg.push(content.clone());
                        }
                        Message::Action { content, .. } => {
                            if !user_msg.is_empty() {
                                // Add the user message to the chat history as a message from the
                                // User
                                chat_history.add_chitchat(ChatEntry {
                                    msg: user_msg.join("\n"),
                                    role: Role::User,
                                });

                                user_msg.clear();
                            }

                            chat_history.add_chitchat(ChatEntry {
                                msg: content.clone(),
                                role: Role::Assistant,
                            });
                        }
                        Message::ActionResult {
                            invocation_count,
                            tool_name,
                            outcome,
                            ..
                        } => {
                            let entry =
                                format_outcome(&task, *invocation_count, tool_name, outcome);

                            user_msg.push(entry);
                        }
                        Message::Task { .. } => {
                            // Nothing
                        }
                    }
                }
            }
        }

        if !user_msg.is_empty() {
            // Add the user message to the chat history as a message from the User
            chat_history.add_chitchat(ChatEntry {
                msg: user_msg.join("\n"),
                role: Role::User,
            });
        }

        if chat_history.is_chitchat_empty() {
            // Add the recurring prompts to the chat history
            chat_history.add_chitchat(ChatEntry {
                msg: task.to_prompt(),
                role: Role::User,
            });
        }

        // prune the history if needed
        chat_history.purge().await?;

        Ok(chat_history)
    }

    fn build_examples(&self) -> Vec<(String, String)> {
        match self {
            Self::Observer { prompt_manager } => {
                let warmup_task =
                    prompt_manager.build_task_prompt("Sort in ascending order: [2, 3, 1, 4, 5]");

                vec![
                    (
                        warmup_task.to_prompt(),
                        OBSERVER_PROTO_INITIAL_RESPONSE.trim().to_string(),
                    ),
                    (
                        (format!(
                            "{}{}",
                            OBSERVER_PROTO_SECOND_INPUT.trim(),
                            warmup_task.to_prompt()
                        ))
                        .trim()
                        .to_string(),
                        OBSERVER_PROTO_SECOND_RESPONSE.trim().to_string(),
                    ),
                ]
            }
            Self::Orienter { prompt_manager } => {
                let warmup_task =
                    prompt_manager.build_task_prompt("Sort in ascending order: [2, 3, 1, 4, 5]");

                vec![
                    (
                        warmup_task.to_prompt(),
                        ORIENTER_PROTO_INITIAL_RESPONSE.trim().to_string(),
                    ),
                    (
                        (format!(
                            "{}{}",
                            ORIENTER_PROTO_SECOND_INPUT.trim(),
                            warmup_task.to_prompt()
                        ))
                        .trim()
                        .to_string(),
                        ORIENTER_PROTO_SECOND_RESPONSE.trim().to_string(),
                    ),
                ]
            }
            Self::Decider { prompt_manager } => {
                let warmup_task =
                    prompt_manager.build_task_prompt("Sort in ascending order: [2, 3, 1, 4, 5]");

                vec![
                    (
                        warmup_task.to_prompt(),
                        DECIDER_PROTO_INITIAL_RESPONSE.trim().to_string(),
                    ),
                    (
                        (format!(
                            "{}{}",
                            DECIDER_PROTO_SECOND_INPUT.trim(),
                            warmup_task.to_prompt()
                        ))
                        .trim()
                        .to_string(),
                        DECIDER_PROTO_SECOND_RESPONSE.trim().to_string(),
                    ),
                ]
            }
            Self::Actor { prompt_manager } => {
                let warmup_task =
                    prompt_manager.build_task_prompt("Sort in ascending order: [2, 3, 1, 4, 5]");

                vec![
                    (
                        warmup_task.to_prompt(),
                        ACTOR_PROTO_INITIAL_RESPONSE.trim().to_string(),
                    ),
                    (
                        (format!(
                            "{}{}",
                            ACTOR_PROTO_SECOND_INPUT.trim(),
                            warmup_task.to_prompt()
                        ))
                        .trim()
                        .to_string(),
                        ACTOR_PROTO_SECOND_RESPONSE.trim().to_string(),
                    ),
                ]
            }
        }
    }
}

/// An agent
pub struct Agent {
    role: AgentRole,
    config: SapiensConfig,
    observer: WeakRuntimeObserver,
}

impl Agent {
    /// Create a new [`Agent`] with the role of an observer.
    #[must_use]
    pub fn new_observer(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Self {
        let system_prompt =
            "You are part of Sapiens agents and your role is to observe and report.".to_string();

        let prompt = "What are your observations?".to_string();

        let prompt_manager = prompt::Manager::new(
            toolbox,
            system_prompt,
            prompt,
            PREFIX.to_string(),
            TOOL_PREFIX.to_string(),
            OBSERVER_RESPONSE_FORMAT.to_string(),
        );

        Self {
            role: AgentRole::Observer { prompt_manager },
            config,
            observer,
        }
    }

    /// Create a new [`Agent`] with the role of an orienter.
    #[must_use]
    pub fn new_orienter(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Self {
        let system_prompt =
            "You are part of Sapiens agents and your role is to orient the other agents based on the observations."
                .to_string();

        let prompt = "What is your orientation?".to_string();

        let prompt_manager = prompt::Manager::new(
            toolbox,
            system_prompt,
            prompt,
            PREFIX.to_string(),
            TOOL_PREFIX.to_string(),
            ORIENTER_RESPONSE_FORMAT.to_string(),
        );

        Self {
            role: AgentRole::Orienter { prompt_manager },
            config,
            observer,
        }
    }

    /// Create a new [`Agent`] with the role of a decider.
    #[must_use]
    pub fn new_decider(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Self {
        let system_prompt =
            "You are part of Sapiens agents and your role is to decide what need to be done based on the observations and guidance you got."
                .to_string();

        let prompt = "What is your decision?".to_string();

        let prompt_manager = prompt::Manager::new(
            toolbox,
            system_prompt,
            prompt,
            PREFIX.to_string(),
            TOOL_PREFIX.to_string(),
            DECIDER_RESPONSE_FORMAT.to_string(),
        );

        Self {
            role: AgentRole::Decider { prompt_manager },
            config,
            observer,
        }
    }

    /// Create a new [`Agent`] with the role of an actor.
    #[must_use]
    pub fn new_actor(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Self {
        let system_prompt =
            "You are part of Sapiens agents and your role is to act on the world as it has been decided."
                .to_string();

        let prompt = "What is your action?".to_string();

        let prompt_manager = prompt::Manager::new(
            toolbox,
            system_prompt,
            prompt,
            PREFIX.to_string(),
            TOOL_PREFIX.to_string(),
            ACTOR_RESPONSE_FORMAT.to_string(),
        );

        Self {
            role: AgentRole::Actor { prompt_manager },
            config,
            observer,
        }
    }

    async fn convert_context_to_chat_history(
        &self,
        context: &Context,
    ) -> Result<ChatHistory, Error> {
        let max_token = self.config.model.context_size().await;

        // Create a new chat history
        let chat_history = ChatHistory::new(self.config.clone(), max_token);
        self.role
            .convert_context_to_chat_history(chat_history, context)
            .await
    }
}

#[async_trait::async_trait]
impl chains::Agent for Agent {
    type Error = Error;

    async fn act(&self, context: &Context) -> Result<Message, Error> {
        let chat_history = self.convert_context_to_chat_history(context).await?;

        // Query the model
        let input = chat_history.make_input();

        debug!(
            min_tokens = self.config.min_tokens_for_completion,
            max_tokens = self.config.max_tokens,
            role = ?self.role,
            "Querying model with {} entries",
            input.chat.len()
        );

        trace!("Querying model:\n{:#?}", input);

        let res = self
            .config
            .model
            .query(input, self.config.max_tokens)
            .await?;

        trace!("Got model response:\n{:#?}", res);

        // Show the message from the assistant
        if let Some(observer) = self.observer.upgrade() {
            observer
                .lock()
                .await
                .on_model_update(res.clone().into())
                .await;
        }

        // Return the response as a message
        match self.role {
            AgentRole::Observer { .. } => Ok(Message::Observation {
                content: res.msg,
                usage: res.usage,
            }),
            AgentRole::Orienter { .. } => Ok(Message::Orientation {
                content: res.msg,
                usage: res.usage,
            }),
            AgentRole::Decider { .. } => Ok(Message::Decision {
                content: res.msg,
                usage: res.usage,
            }),
            AgentRole::Actor { .. } => Ok(Message::Action {
                content: res.msg,
                usage: res.usage,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use indoc::indoc;
    use insta::assert_debug_snapshot;

    use super::*;
    use crate::chains::Outcome;
    use crate::void_observer;

    #[tokio::test]
    async fn observer_converts_context_to_chat_history() {
        let context = build_dummy_context();

        let toolbox = Toolbox::default();

        let observer = void_observer();
        let weak_observer = Arc::downgrade(&observer);
        let agent = Agent::new_observer(SapiensConfig::default(), toolbox, weak_observer);

        let chat_history = agent.convert_context_to_chat_history(&context).await;

        assert_debug_snapshot!(chat_history);
    }

    #[tokio::test]
    async fn orienter_converts_context_to_chat_history() {
        let mut context = build_dummy_context();

        context.add_message(Message::Observation {
            content: indoc! {r"
            ## Observations:
            - We needed to sort the list in ascending order.
            - We have the response of the Action.
            - We have the sorted list: [1, 2, 3, 4, 5].
            "
            }
            .trim()
            .to_string(),
            usage: None,
        });

        let toolbox = Toolbox::default();

        let observer = void_observer();
        let weak_observer = Arc::downgrade(&observer);
        let agent = Agent::new_orienter(SapiensConfig::default(), toolbox, weak_observer);

        let chat_history = agent.convert_context_to_chat_history(&context).await;

        assert_debug_snapshot!(chat_history);
    }

    #[tokio::test]
    async fn decider_converts_context_to_chat_history() {
        let mut context = build_dummy_context();

        context.add_message(Message::Observation {
            content: indoc! {r"
            ## Observations:
            - We needed to sort the list in ascending order.
            - We have the response of the Action.
            - We have the sorted list: [1, 2, 3, 4, 5].
            "
            }
            .trim()
            .to_string(),
            usage: None,
        });

        context.add_message(Message::Orientation {
            content: indoc! {r"
            ## Orientation:
            - I know the answer to the original question.
            - I need to provide the `tool_name` and `parameters` fields for the Conclude Tool.
            "
            }
            .trim()
            .to_string(),
            usage: None,
        });

        let toolbox = Toolbox::default();

        let observer = void_observer();
        let weak_observer = Arc::downgrade(&observer);
        let agent = Agent::new_decider(SapiensConfig::default(), toolbox, weak_observer);

        let chat_history = agent.convert_context_to_chat_history(&context).await;

        assert_debug_snapshot!(chat_history);
    }

    #[tokio::test]
    async fn actor_converts_context_to_chat_history() {
        let mut context = build_dummy_context();

        context.add_message(Message::Observation {
            content: indoc! {r"
            ## Observations:
            - We needed to sort the list in ascending order.
            - We have the response of the Action.
            - We have the sorted list: [1, 2, 3, 4, 5].
            "
            }
            .trim()
            .to_string(),
            usage: None,
        });

        context.add_message(Message::Orientation {
            content: indoc! {r"
            ## Orientation:
            - I know the answer to the original question.
            - I need to provide the `tool_name` and `parameters` fields for the Conclude Tool.
            "
            }
            .trim()
            .to_string(),
            usage: None,
        });

        context.add_message(Message::Decision {
            content: indoc! {r"
            ## Decision:
            - Use the Conclude Tool to terminate the task with the sorted list.
            "
            }
            .trim()
            .to_string(),
            usage: None,
        });

        let toolbox = Toolbox::default();

        let observer = void_observer();
        let weak_observer = Arc::downgrade(&observer);
        let agent = Agent::new_actor(SapiensConfig::default(), toolbox, weak_observer);

        let chat_history = agent.convert_context_to_chat_history(&context).await;

        assert_debug_snapshot!(chat_history);
    }

    fn build_dummy_context() -> Context {
        let mut context = Context::new();

        context.add_message(Message::Task {
            content: "Sort in ascending order: [2, 3, 1, 4, 5]".to_string(),
        });

        context.add_message(Message::Observation {
            content: indoc! {r"
            ## Observations:
            - The given list to sort is [2, 3, 1, 4, 5].
            - I need to sort this list in ascending order."
            }
            .trim()
            .to_string(),
            usage: None,
        });

        context.add_message(Message::Orientation {
            content: indoc! {r"
            ## Orientation:
            - SandboxedPython can be used to sort the list.
            - I need to provide only the `tool_name` and `parameters` fields for the SandboxedPython Tool.
            - I expect the response of the Action to contains the field `stdout` with the sorted list and `stderr` empty.
            - I need to use the Conclude Tool to terminate the task when I have the sorted list in plain text."
            }.trim().to_string(),
            usage: None,
        });

        context.add_message(Message::Decision {
            content: indoc! {r"
            ## Decision:
            - We can use the sorted() function of Python to sort the list."
            }
            .trim()
            .to_string(),
            usage: None,
        });

        context.add_message(Message::Action {
            content: indoc! {r#"
            ## The ONLY Action:
            ```yaml
            tool_name: SandboxedPython
            parameters:
              code: |
                lst = [2, 3, 1, 4, 5]
                sorted_list = sorted(lst)
                print(f"The sorted list is {sorted_list}")
            ```
            That's it for now. We will take further action based on the response.            
            "#
            }
            .trim()
            .to_string(),
            usage: None,
        });

        context.add_message(Message::ActionResult {
            invocation_count: 1,
            tool_name: Some("SandboxedPython".to_string()),
            extracted_input: Some(
                indoc! {r#"
            tool_name: SandboxedPython
            parameters:
              code: |
                lst = [2, 3, 1, 4, 5]
                sorted_list = sorted(lst)
                print(f"The sorted list is {sorted_list}")
            "#}
                .trim()
                .to_string(),
            ),
            outcome: Outcome::Success {
                result: indoc! {r"
                stdout: |
                  The sorted list is [1, 2, 3, 4, 5]
                stderr: ''
                "}
                .trim()
                .to_string(),
            },
        });
        context
    }
}
