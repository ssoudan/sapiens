use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::sync::Arc;

use sapiens::chains::Message;
use sapiens::context::{ChatEntry, ContextDump};
use sapiens::models::Role;
use sapiens::{
    InvalidInvocationNotification, InvocationFailureNotification, InvocationResultNotification,
    InvocationSuccessNotification, MessageNotification, ModelNotification, RuntimeObserver,
    TerminationNotification,
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

impl From<&sapiens::models::Usage> for Usage {
    fn from(usage: &sapiens::models::Usage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
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
    /// A tool was successfully invoked
    ToolInvocationSucceeded {
        /// The name of the tool
        tool_name: String,
        /// Number of invocation blocks in the message
        invocation_count: usize,
        /// The input that was extracted from the message and passed to
        /// `tool_name` - split into lines
        extracted_input: Vec<String>,
        /// The output of the tool - split into lines
        result: Vec<String>,
    },
    /// Invoked tool failed
    ToolInvocationFailed {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool - split into lines
        extracted_input: Vec<String>,
        /// Number of invocation blocks in the message
        invocation_count: usize,
        /// The error message - split into lines
        error: Vec<String>,
    },
    /// Message
    Message {
        /// The message
        message: Message,
    },
    /// The task chain succeeded
    End(CompletionStatus),
    /// Invalid invocation
    InvalidInvocation {
        /// The error message - split into lines
        error: Vec<String>,
        /// Number of invocation blocks in the message
        invocation_count: usize,
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
            Event::End(_) => None,
            Event::ToolInvocationSucceeded { .. } => None,
            Event::ToolInvocationFailed { .. } => None,
            Event::InvalidInvocation { .. } => None,
            Event::Message { message, .. } => match message {
                Message::Task { .. } => None,
                Message::Observation { usage, .. } => usage.as_ref().map(|x| x.into()),
                Message::Orientation { usage, .. } => usage.as_ref().map(|x| x.into()),
                Message::Decision { usage, .. } => usage.as_ref().map(|x| x.into()),
                Message::Action { usage, .. } => usage.as_ref().map(|x| x.into()),
                Message::ActionResult { .. } => None,
            },
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
    tool_input: Option<ModelNotification>,
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
            invocation_count,
            tool_name,
            extracted_input,
            result,
        } = notification;

        Event::ToolInvocationSucceeded {
            tool_name,
            invocation_count,
            extracted_input: to_lines(extracted_input),
            result: to_lines(result),
        }
    }
}

impl From<InvocationFailureNotification> for Event {
    fn from(notification: InvocationFailureNotification) -> Self {
        let InvocationFailureNotification {
            invocation_count,
            tool_name,
            extracted_input,
            e,
        } = notification;

        Event::ToolInvocationFailed {
            tool_name,
            invocation_count,
            extracted_input: to_lines(extracted_input),
            error: to_lines(format!("{}", e)),
        }
    }
}

impl From<InvalidInvocationNotification> for Event {
    fn from(notification: InvalidInvocationNotification) -> Self {
        let InvalidInvocationNotification {
            e,
            invocation_count,
        } = notification;

        Event::InvalidInvocation {
            invocation_count,
            error: to_lines(format!("{}", e)),
        }
    }
}

impl From<MessageNotification> for Event {
    fn from(notification: MessageNotification) -> Self {
        Event::Message {
            message: notification.message,
        }
    }
}

impl From<InvocationResultNotification> for Event {
    fn from(notification: InvocationResultNotification) -> Self {
        match notification {
            InvocationResultNotification::InvocationSuccess(x) => x.into(),
            InvocationResultNotification::InvocationFailure(x) => x.into(),
            InvocationResultNotification::InvalidInvocation(x) => x.into(),
        }
    }
}

#[async_trait::async_trait]
impl RuntimeObserver for TraceObserver {
    async fn on_task(&mut self, task: &str) {
        let state = self.get_state().await;

        self.trace.events.push(
            Event::Start {
                task: task.to_string(),
            }
            .into_event_and_state(state),
        );
    }

    async fn on_message(&mut self, event: MessageNotification) {
        let state = self.get_state().await;

        self.trace
            .events
            .push(Event::from(event).into_event_and_state(state));
    }

    async fn on_start(&mut self, _context: ContextDump) {}

    async fn on_model_update(&mut self, model_update: ModelNotification) {
        self.tool_input = Some(model_update);
    }

    async fn on_invocation_result(&mut self, event: InvocationResultNotification) {
        let state = self.get_state().await;

        self.trace
            .events
            .push(Event::from(event).into_event_and_state(state));
    }

    async fn on_termination(&mut self, event: TerminationNotification) {
        self.termination = Some(event);
    }
}
