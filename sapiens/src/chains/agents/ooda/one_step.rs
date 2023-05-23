use tracing::{debug, trace};

use crate::chains::agents::{format_outcome, Error};
use crate::chains::{Context, Message};
use crate::context::{ChatEntry, ChatHistory};
use crate::models::Role;
use crate::tools::toolbox::Toolbox;
use crate::{chains, prompt, SapiensConfig, WeakRuntimeObserver};

/// An OODA agent
pub struct Agent {
    prompt_manager: prompt::Manager,
    config: SapiensConfig,
    observer: WeakRuntimeObserver,
}

// FUTURE(ssoudan) parameterize the prompt manager

const PREFIX: &str = r"You are Sapiens, a large language model assisting the WORLD. Use available tools to answer the question as best as you can.
You will proceed iteratively using an OODA loop.

- Action result will be provided to you. 
- Never produce the result of an Action. 
- The loop will repeated until you have the answer to the original question. 
- No task is complete until the Conclude Tool is used to provide the answer.
- You cannot use jinja2 templating in your response. Be concise. 
";

const TOOL_PREFIX: &str = r"
# The following are the ONLY Tools you can use for your Actions:
";

const RESPONSE_FORMAT: &str = r"
# Format of your response

You must use the following format for your response. Comments are in bold and should be removed from your response.
====================
## Observations: 
**What do you know to be true? What do you you don't know? What are your sources? Note down important information for later.**
- <...>
## Orientation: 
**Plan the intermediate objectives to answer the original question. Maintain a list of current objectives updated as you go.**
- <...>
## Decision: 
**Decide what to do first to answer the question. Why? How will you if it succeeds? How will you if it fails?**
- <...>
## The ONLY Action: 
**Take a single Action consisting of exactly one pair of `tool_name` and `parameters`. Never give more than one YAML. **
```yaml
tool_name: <ToolName>
parameters:
    <...>  
```
We will take further action based on the result.
====================

Notes: 
- Action has the following fields: `tool_name` and `parameters` ONLY.
- `parameters` uses the format specified for the Tool.
- `result_fields` is the format you can expect of the result of the Action. You can use this to orient yourself but never use it in your response.
- One Action at a time. No more. No less.
";

const PROTO_EXCHANGE_2: &str = r#"
## Observations:
- The given list to sort is [2, 3, 1, 4, 5].
- I need to sort this list in ascending order.
## Orientation:
- SandboxedPython can be used to sort the list.
- I need to provide only the `tool_name` and `parameters` fields for the SandboxedPython Tool.
- I expect the result of the Action to contains the field `stdout` with the sorted list and `stderr` empty.
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
We will take further action based on the result.
"#;

const PROTO_EXCHANGE_3: &str = r"
# Action SandboxedPython result:
```yaml
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
";

const PROTO_EXCHANGE_4: &str = r"
## Observations:
- We needed to sort the list in ascending order.
- We have the result of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
## Orientation:
- I know the answer to the original question.
- I need to provide the `tool_name` and `parameters` fields for the Conclude Tool.
## Decision:
- Use the Conclude Tool to terminate the task with the sorted list.
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

impl Agent {
    /// Create a new [`Agent`].
    pub async fn new(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Self {
        let system_prompt =
            "You are an agent named Sapiens interacting with the WORLD. Listen to the WORLD!"
                .to_string();

        let prompt = "Do you have the answer? Use the Conclude Tool to terminate the task.\nObservations, Orientation, Decision, The ONLY Action?".to_string();

        let prompt_manager = prompt::Manager::new(
            toolbox,
            system_prompt,
            prompt,
            PREFIX.to_string(),
            TOOL_PREFIX.to_string(),
            RESPONSE_FORMAT.to_string(),
        );
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
        let mut chat_history = ChatHistory::new(self.config.clone(), max_token);

        let warmup_task = self
            .prompt_manager
            .build_task_prompt("Sort in ascending order: [2, 3, 1, 4, 5]");

        let examples = vec![
            (warmup_task.to_prompt(), PROTO_EXCHANGE_2.trim().to_string()),
            (
                (format!("{}{}", PROTO_EXCHANGE_3, warmup_task.to_prompt()))
                    .trim()
                    .to_string(),
                PROTO_EXCHANGE_4.trim().to_string(),
            ),
        ];

        // Add the prompts to the chat history
        self.prompt_manager
            .populate_chat_history(&mut chat_history, examples)
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
                        .await;
                }
                Message::ActionResult {
                    invocation_count,
                    tool_name,
                    outcome,
                    ..
                } => {
                    let entry = format_outcome(&task, invocation_count, tool_name, outcome);

                    // add an error message to the chat history
                    let entry = ChatEntry {
                        msg: entry,
                        role: Role::User,
                    };

                    // Add the result to the chat history
                    chat_history.add_chitchat(entry).await;
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
                .await;
        }

        // prune the history if needed
        chat_history.purge().await?;

        Ok(chat_history)
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use indoc::indoc;
    use insta::assert_debug_snapshot;

    use super::*;
    use crate::chains::Outcome;
    use crate::void_observer;

    #[tokio::test]
    async fn it_converts_context_to_chat_history() {
        let mut context = Context::new();

        context.add_message(Message::Task {
            content: "Sort in ascending order: [2, 3, 1, 4, 5]".to_string(),
        });

        context.add_message(Message::Action {
            content: indoc! {r#"
            ## Observations:
            - The given list to sort is [2, 3, 1, 4, 5].
            - I need to sort this list in ascending order.
            ## Orientation:
            - SandboxedPython can be used to sort the list.
            - I need to provide only the `tool_name` and `parameters` fields for the SandboxedPython Tool.
            - I expect the result of the Action to contains the field `stdout` with the sorted list and `stderr` empty.
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
            We will take further action based on the result.            
            "#
            }.to_string(),
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
                .to_string(),
            ),
            outcome: Outcome::Success {
                result: indoc! {r#"
                stdout: |
                  The sorted list is [1, 2, 3, 4, 5]
                stderr: ''
                "#}
                .to_string(),
            },
        });

        let toolbox = Toolbox::default();

        let observer = void_observer();
        let weak_observer = Arc::downgrade(&observer);
        let agent = Agent::new(Default::default(), toolbox, weak_observer).await;

        let chat_history = agent.convert_context_to_chat_history(&context).await;

        assert_debug_snapshot!(chat_history);
    }
}
