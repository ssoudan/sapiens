use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::sync::Arc;

use sapiens::context::{ChatEntry, ChatHistoryDump};
use sapiens::models::Role;
use sapiens::{
    InvalidInvocationNotification, InvocationFailureNotification, InvocationSuccessNotification,
    ModelUpdateNotification, StepObserver, TerminationNotification,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::info;

use crate::tools;
use crate::tools::State;

/// Token usage
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// The number of tokens used for the prompt
    pub prompt_tokens: u32,
    /// The number of tokens used for the completion
    pub completion_tokens: u32,
    /// The total number of tokens used
    pub total_tokens: u32,
}

fn to_lines(s: impl AsRef<str>) -> Vec<String> {
    s.as_ref().split('\n').map(|s| s.to_string()).collect()
}

impl Add for Usage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            prompt_tokens: self.prompt_tokens + rhs.prompt_tokens,
            completion_tokens: self.completion_tokens + rhs.completion_tokens,
            total_tokens: self.total_tokens + rhs.total_tokens,
        }
    }
}

impl From<sapiens::models::Usage> for Usage {
    fn from(usage: sapiens::models::Usage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

/// The trace of an execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// The events in the trace
    pub(crate) events: Vec<EventAndState>,
}

/// The status of a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reason")]
pub enum CompletionStatus {
    /// `Conclude` tool was invoked
    Concluded {
        /// The conclusion
        conclusion: String,
    },
    /// `MaxSteps` was reached
    MaxStepsReached,
}

/// The result of a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InvocationResult {
    /// The tool invocation was successful
    Success {
        /// The output of the tool - split into lines
        output: Vec<String>,
    },
    /// The tool invocation failed
    Failure {
        /// The error message - split into lines
        error: Vec<String>,
    },
}

/// Prompt entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEntry {
    /// the role
    pub role: Role,
    /// The message
    pub msg: String,
}

impl From<&ChatEntry> for PromptEntry {
    fn from(entry: &ChatEntry) -> Self {
        Self {
            role: entry.role.clone(),
            msg: entry.msg.clone(),
        }
    }
}

/// An event in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// New task started
    Start {
        /// The task
        task: String,
    },
    /// Initial prompt
    Prompt {
        /// Initial prompt
        initial_prompt: Vec<PromptEntry>,
    },
    /// A tool was invoked
    ToolInvocationSucceeded {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool - split into lines
        assistant_message: Vec<String>,
        /// Result of the tool invocation
        result: InvocationResult,
        /// Token usage
        token_usage: Option<Usage>,
        /// Number of invocation blocks in the message
        available_invocation_count: usize,
        /// The input that was extracted from the message and passed to
        /// `tool_name` - split into lines
        extracted_input: Vec<String>,
    },
    /// Invalid tool invocation
    ToolInvocationFailed {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool - split into lines
        tool_input: Vec<String>,
        /// The error message - split into lines
        error: Vec<String>,
        /// Token usage
        token_usage: Option<Usage>,
        /// Number of invocation blocks in the message
        available_invocation_count: usize,
        /// The input that was extracted from the message and passed to
        /// `tool_name` - split into lines
        extracted_input: Vec<String>,
    },
    /// The chat was not updated after a tool invocation failed
    ToolInvocationFailedAndChatNotUpdated {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool - split into lines
        tool_input: Vec<String>,
        /// The error message - split into lines
        error: Vec<String>,
        /// Token usage
        token_usage: Option<Usage>,
        /// Number of invocation blocks in the message
        available_invocation_count: usize,
        /// The input that was extracted from the message and passed to
        /// `tool_name` - split into lines
        extracted_input: Vec<String>,
    },
    /// The task chain succeeded
    End(CompletionStatus),
    /// Invalid invocation
    InvalidInvocation {
        /// The input of the tool - split into lines
        tool_input: Vec<String>,
        /// The error message - split into lines
        error: Vec<String>,
        /// Token usage
        token_usage: Option<Usage>,
        /// Number of invocation blocks in the message
        available_invocation_count: usize,
    },
    /// Invalid invocation
    InvalidInvocationAndChatNotUpdated {
        /// The input of the tool - split into lines
        tool_input: Vec<String>,
        /// The error message - split into lines
        error: Vec<String>,
        /// Token usage
        token_usage: Option<Usage>,
        /// Number of invocation blocks in the message
        available_invocation_count: usize,
    },
}

/// An event and the state after the event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAndState {
    /// The event
    #[serde(flatten)]
    pub event: Event,
    /// The state after the event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl Event {
    /// Get the token usage
    pub fn tokens(&self) -> Option<Usage> {
        match &self {
            Event::Prompt { .. } => None,
            Event::Start { .. } => None,
            Event::ToolInvocationSucceeded { token_usage, .. } => token_usage.clone(),
            Event::ToolInvocationFailed { token_usage, .. } => token_usage.clone(),
            Event::ToolInvocationFailedAndChatNotUpdated { token_usage, .. } => token_usage.clone(),
            Event::InvalidInvocation { token_usage, .. } => token_usage.clone(),
            Event::InvalidInvocationAndChatNotUpdated { token_usage, .. } => token_usage.clone(),
            Event::End(_) => None,
        }
    }

    /// Wrap the event in an [`EventAndState`] with the given `state`
    fn into_event_and_state(self, state: Option<String>) -> EventAndState {
        EventAndState { event: self, state }
    }
}

/// Trace collecting observer
pub struct TraceObserver {
    /// The trace
    trace: Trace,
    /// Temporary store for the input
    /// of the tool invocation
    tool_input: Option<ModelUpdateNotification>,
    /// Termination event
    termination: Option<TerminationNotification>,
    /// Whether the trace is finalized
    finalized: bool,
    /// The shared state
    state: Option<Arc<Mutex<dyn State>>>,
}

impl Debug for TraceObserver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraceObserver")
            // .field("trace", &self.trace)
            // .field("tool_input", &self.tool_input)
            // .field("termination_messages", &self.termination_messages)
            .finish()
    }
}

impl Default for TraceObserver {
    fn default() -> Self {
        Self {
            trace: Trace { events: vec![] },
            tool_input: None,
            termination: None,
            finalized: false,
            state: None,
        }
    }
}

impl TraceObserver {
    /// Create a new trace observer
    pub fn new(shared_state: Arc<Mutex<dyn tools::State>>) -> Self {
        Self {
            trace: Trace { events: vec![] },
            tool_input: None,
            termination: None,
            finalized: false,
            state: Some(shared_state),
        }
    }

    async fn get_state(&self) -> Option<String> {
        if let Some(state) = self.state.as_ref() {
            let guard = state.lock().await;

            let state = guard.state();
            info!("Current state: {}", state);
            Some(state)
        } else {
            None
        }
    }

    /// Get the trace
    pub async fn trace(&mut self) -> Trace {
        if self.finalized {
            return self.trace.clone();
        }

        let state = self.get_state().await;

        // Add the final event
        match self.termination.take() {
            Some(termination) => {
                let conclusion = termination
                    .messages
                    .iter()
                    .map(|msg| msg.conclusion.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.trace.events.push(
                    Event::End(CompletionStatus::Concluded { conclusion })
                        .into_event_and_state(state),
                );
            }
            None => {
                self.trace.events.push(
                    Event::End(CompletionStatus::MaxStepsReached).into_event_and_state(state),
                );
            }
        }

        self.trace.clone()
    }
}

impl From<InvocationSuccessNotification> for Event {
    fn from(notification: InvocationSuccessNotification) -> Self {
        let InvocationSuccessNotification {
            tool_name,
            res,
            usage,
            assistant_message,
            available_invocation_count,
            extracted_input,
        } = notification;

        match res {
            // The invocation was successful
            Ok(res) => Event::ToolInvocationSucceeded {
                tool_name,
                assistant_message: to_lines(assistant_message.msg),
                result: InvocationResult::Success {
                    output: to_lines(res.msg),
                },
                token_usage: usage.map(Into::into),
                available_invocation_count,
                extracted_input: to_lines(extracted_input),
            },
            // The invocation was successful, but the output could not be
            // passed to the chat history
            Err(err) => Event::ToolInvocationSucceeded {
                tool_name,
                assistant_message: to_lines(assistant_message.msg),
                result: InvocationResult::Failure {
                    error: to_lines(format!("{}", err)),
                },
                token_usage: usage.map(Into::into),
                available_invocation_count,
                extracted_input: to_lines(extracted_input),
            },
        }
    }
}

impl From<InvocationFailureNotification> for Event {
    fn from(notification: InvocationFailureNotification) -> Self {
        let InvocationFailureNotification {
            tool_name,
            res,
            usage,
            assistant_message,
            available_invocation_count,
            extracted_input,
        } = notification;

        match res {
            // The invocation was unsuccessful
            Ok(res) => Event::ToolInvocationFailed {
                tool_name,
                tool_input: to_lines(assistant_message.msg),
                error: to_lines(res.msg),
                token_usage: usage.map(Into::into),
                available_invocation_count,
                extracted_input: to_lines(extracted_input),
            },
            // The invocation was unsuccessful, and the error message could
            // not be passed to the chat history
            Err(err) => Event::ToolInvocationFailedAndChatNotUpdated {
                tool_name,
                tool_input: to_lines(assistant_message.msg),
                error: to_lines(format!("{}", err)),
                token_usage: usage.map(Into::into),
                available_invocation_count,
                extracted_input: to_lines(extracted_input),
            },
        }
    }
}

impl From<InvalidInvocationNotification> for Event {
    fn from(notification: InvalidInvocationNotification) -> Self {
        let InvalidInvocationNotification {
            res,
            assistant_message,
            available_invocation_count,
            usage,
        } = notification;

        match res {
            // The invocation was invalid
            Ok(res) => Event::InvalidInvocation {
                tool_input: to_lines(assistant_message.msg),
                error: to_lines(res.msg),
                token_usage: usage.map(Into::into),
                available_invocation_count,
            },
            // No valid invocation found, and the error message could
            // not be passed to the chat history
            Err(err) => Event::InvalidInvocationAndChatNotUpdated {
                tool_input: to_lines(assistant_message.msg),
                error: to_lines(format!("{}", err)),
                token_usage: usage.map(Into::into),
                available_invocation_count,
            },
        }
    }
}

#[async_trait::async_trait]
impl StepObserver for TraceObserver {
    async fn on_task(&mut self, task: &str) {
        let state = self.get_state().await;

        self.trace.events.push(
            Event::Start {
                task: task.to_string(),
            }
            .into_event_and_state(state),
        );
    }

    async fn on_start(&mut self, chat_history: ChatHistoryDump) {
        let state = self.get_state().await;

        self.trace.events.push(
            Event::Prompt {
                initial_prompt: chat_history.messages.iter().map(|msg| msg.into()).collect(),
            }
            .into_event_and_state(state),
        );
    }

    async fn on_model_update(&mut self, model_update: ModelUpdateNotification) {
        self.tool_input = Some(model_update);
    }

    async fn on_invocation_success(&mut self, event: InvocationSuccessNotification) {
        let state = self.get_state().await;

        self.trace
            .events
            .push(Event::from(event).into_event_and_state(state));
    }

    async fn on_invalid_invocation(&mut self, event: InvalidInvocationNotification) {
        let state = self.get_state().await;

        self.trace
            .events
            .push(Event::from(event).into_event_and_state(state));
    }

    async fn on_invocation_failure(&mut self, event: InvocationFailureNotification) {
        let state = self.get_state().await;

        self.trace
            .events
            .push(Event::from(event).into_event_and_state(state));
    }

    async fn on_termination(&mut self, event: TerminationNotification) {
        self.termination = Some(event);
    }
}
