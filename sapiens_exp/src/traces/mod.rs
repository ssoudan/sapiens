use std::fmt::{Debug, Formatter};

use sapiens::{
    InvocationFailureNotification, InvocationSuccessNotification, ModelUpdateNotification,
    StepObserver, TerminationNotification,
};
use serde::{Deserialize, Serialize};

/// Token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// The number of tokens used for the prompt
    pub prompt_tokens: u32,
    /// The number of tokens used for the completion
    pub completion_tokens: u32,
    /// The total number of tokens used
    pub total_tokens: u32,
}

impl From<sapiens::runner::Usage> for Usage {
    fn from(usage: sapiens::runner::Usage) -> Self {
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
    events: Vec<Event>,
}

/// The status of a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionStatus {
    /// `Conclude` tool was invoked
    Concluded {
        /// The conclusion
        conclusion: String,
        /// Token usage
        usage: Option<Usage>,
    },
    /// `MaxSteps` was reached
    MaxStepsReached,
}

/// The result of a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvocationResult {
    /// The tool invocation was successful
    Success {
        /// The output of the tool
        output: String,
    },
    /// The tool invocation failed
    Failure {
        /// The error message
        error: String,
    },
}

/// An event in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Event {
    /// New task started
    Start {
        /// The task
        task: String,
    },
    /// A tool was invoked
    ToolInvocationSucceeded {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool
        tool_input: String,
        /// Result of the tool invocation
        result: InvocationResult,
        /// Token usage
        token_usage: Option<Usage>,
    },
    /// Invalid tool invocation
    ToolInvocationFailed {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool
        tool_input: String,
        /// The error message
        error: String,
        /// Token usage
        token_usage: Option<Usage>,
    },
    /// The task chain failed
    TaskChainFailed {
        /// The name of the tool
        tool_name: String,
        /// The input of the tool
        tool_input: String,
        /// The error message
        error: String,
        /// Token usage
        token_usage: Option<Usage>,
    },
    /// The task chain succeeded
    End(CompletionStatus),
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
        Self::new()
    }
}

impl TraceObserver {
    /// Create a new trace observer
    pub fn new() -> Self {
        Self {
            trace: Trace { events: vec![] },
            tool_input: None,
            termination: None,
            finalized: false,
        }
    }

    /// Get the trace
    pub fn trace(&mut self) -> Trace {
        if self.finalized {
            return self.trace.clone();
        }

        // Add the final event
        match self.termination.take() {
            Some(termination) => {
                let conclusion = termination
                    .messages
                    .iter()
                    .map(|msg| msg.conclusion.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.trace
                    .events
                    .push(Event::End(CompletionStatus::Concluded {
                        conclusion,
                        usage: termination.usage.map(Into::into),
                    }));
            }
            None => {
                self.trace
                    .events
                    .push(Event::End(CompletionStatus::MaxStepsReached));
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
            input,
        } = notification;

        match res {
            Ok(res) => Event::ToolInvocationSucceeded {
                tool_name,
                tool_input: input.msg,
                result: InvocationResult::Success { output: res.msg },
                token_usage: usage.map(Into::into),
            },
            Err(err) => Event::ToolInvocationSucceeded {
                tool_name,
                tool_input: input.msg,
                result: InvocationResult::Failure {
                    error: format!("{}", err),
                },
                token_usage: usage.map(Into::into),
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
            input,
        } = notification;

        match res {
            Ok(res) => Event::ToolInvocationFailed {
                tool_name,
                tool_input: input.msg,
                error: res.msg,
                token_usage: usage.map(Into::into),
            },
            Err(err) => Event::TaskChainFailed {
                tool_name,
                tool_input: input.msg,
                error: format!("{}", err),
                token_usage: usage.map(Into::into),
            },
        }
    }
}

#[async_trait::async_trait]
impl StepObserver for TraceObserver {
    async fn on_task(&mut self, task: &str) {
        self.trace.events.push(Event::Start {
            task: task.to_string(),
        });
    }

    async fn on_model_update(&mut self, model_update: ModelUpdateNotification) {
        self.tool_input = Some(model_update);
    }

    async fn on_invocation_success(&mut self, event: InvocationSuccessNotification) {
        self.trace.events.push(event.into());
    }

    async fn on_invocation_failure(&mut self, event: InvocationFailureNotification) {
        self.trace.events.push(event.into());
    }

    async fn on_termination(&mut self, event: TerminationNotification) {
        self.termination = Some(event);
    }
}
