use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::context::{ChatEntry, ChatHistory};
use crate::models::Role;
use crate::tools::invocation::Error;
use crate::tools::toolbox::Toolbox;
use crate::tools::{ToolDescription, ToolUseError};

// FUTURE(ssoudan) prompt a-la: "below are a series of dialogues between..." for
// non-instruct models
/// Prompt manager
#[derive(Clone)]
pub(crate) struct Manager {
    toolbox: Toolbox,
    system_prompt: String,
    prompt: String,
    prefix: String,
    tool_prefix: String,
    response_format: String,
}

impl Manager {
    /// Create a new prompt manager
    #[must_use]
    pub(crate) const fn new(
        toolbox: Toolbox,
        system_prompt: String,
        prompt: String,
        prefix: String,
        tool_prefix: String,
        response_format: String,
    ) -> Self {
        Self {
            toolbox,
            system_prompt,
            prompt,
            prefix,
            tool_prefix,
            response_format,
        }
    }

    /// Create the prompt describing the tools
    async fn create_tool_description(&self) -> String {
        let prefix = self.tool_prefix.clone();

        let tool_desc = self.toolbox.describe().await;

        let mut tool_desc: Vec<ToolDescription> = tool_desc.into_values().collect();

        // sort by tool name
        tool_desc.sort_by(|a, b| a.name.cmp(&b.name));

        // yaml serialize the tool description
        let tool_desc = serde_yaml::to_string(&tool_desc).unwrap();

        prefix + &tool_desc
    }

    /// Create the prompt describing the tools and how to use them
    async fn create_tool_warm_up(&self) -> String {
        let tool_prompt = self.create_tool_description().await;

        format!("{}{}{}", self.prefix, self.response_format, tool_prompt)
    }

    /// Create the prompt for the task
    pub(crate) fn build_task_prompt(&self, task: &str) -> Task {
        let prompt = format!("# Your turn\nOriginal question: {}\n{}", task, self.prompt,);
        Task {
            task: task.to_string(),
            prompt,
        }
    }

    /// Create the 'system' prompt to describe the roles.
    fn create_system_prompt(&self) -> String {
        self.system_prompt.clone()
    }

    pub(crate) async fn populate_chat_history(
        &self,
        chat_history: &mut ChatHistory,
        examples: Vec<(String, String)>,
    ) {
        let warm_up_prompt = self.create_tool_warm_up().await;
        let system_prompt = self.create_system_prompt();

        chat_history.set_context(vec![
            ChatEntry {
                role: Role::System,
                msg: system_prompt.trim().to_string(),
            },
            ChatEntry {
                role: Role::User,
                msg: warm_up_prompt.trim().to_string(),
            },
        ]);

        for (prompt, response) in examples {
            chat_history.add_example(prompt, response);
        }
    }
}

/// Task-related prompts
///
/// Use [`Display`] to get the prompt.
pub struct Task {
    task: String,
    prompt: String,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task").field("task", &self.task).finish()
    }
}

impl Task {
    /// Create the prompt for the task
    #[must_use]
    pub fn to_prompt(&self) -> String {
        self.prompt.clone()
    }

    /// Create the prompt to react to an action failure
    pub(crate) fn action_failed_prompt(tool_name: impl AsRef<str>, e: &ToolUseError) -> String {
        format!(
            "# Action {} failed with:\n{:?}\nSomething was incorrect in previous response.",
            tool_name.as_ref(),
            e,
        )
    }

    /// Create the prompt to react to invalid action specification
    pub(crate) fn invalid_action_prompt(e: &Error) -> String {
        format!("# No valid Action found:\n{e:?}\nSomething was incorrect in previous response.")
    }

    /// Create the prompt to react to an action success
    pub(crate) fn action_success_prompt(
        tool_name: impl AsRef<str>,
        available_invocation_count: usize,
        result: impl AsRef<str>,
    ) -> String {
        if available_invocation_count == 1 {
            format!(
                "# Action {} response: \n```yaml\n{}```",
                tool_name.as_ref(),
                result.as_ref(),
            )
        } else {
            format!(
                "# Action {} response: \nYou must give only one Action at a time. There was {}. Only the first one was considered.\n```yaml\n{}```",
                tool_name.as_ref(),
                available_invocation_count,
                result.as_ref(),

            )
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_prompt())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn populate_chat_history() {
        use super::*;
        use crate::context::ChatHistory;
        use crate::Toolbox;

        let toolbox = Toolbox::default();
        let system_prompt =
            "You are an agent named Sapiens interacting with the WORLD. Listen to the WORLD!"
                .to_string();

        let prompt = "Do you have the answer? Use the Conclude Tool to terminate the task.\nObservations, Orientation, Decision, The ONLY Action?".to_string();

        let prefix = "Sapiens:".to_string();
        let tool_prefix = "Tool:".to_string();
        let response_format =
            "Something very long with Observations, Orientation, Decision, Action\n\n".to_string();

        let manager = Manager::new(
            toolbox,
            system_prompt,
            prompt,
            prefix,
            tool_prefix,
            response_format,
        );

        let config = crate::SapiensConfig::default();

        let max_token = config.model.context_size().await;
        let mut chat_history = ChatHistory::new(config.clone(), max_token);

        let examples = vec![];

        manager
            .populate_chat_history(&mut chat_history, examples)
            .await;

        // let prompts: Vec<ChatEntry> = chat_history.iter().cloned().collect();

        // println!("{:?}", prompts);
        let tokens = config.model.num_tokens(chat_history.make_input()).await;

        assert_eq!(tokens, 64);
    }
}
