/// Schedulers are responsible for deciding which agent to run next.
pub mod schedulers;

/// Agents
pub mod agents;

#[cfg(test)]
mod tests;

// TODO(ssoudan) more chains:
// - OODA - Observe, Orient, Decide, Act
//  - in one shot
//  - in several steps
// - 2201.11903 - Chain of thought prompting - 2022
// - 2205.11916 - Zeroshot reasoners - "Let's think step by step" - 2022
// - 2207.05608 - Inner monologue - Different types of feedbacks - 2022
// - 2302.01560 - DEPS - Describe, explain, plan, select stages. Feb 2023
// - 2210.03629 - ReAct - Reasoning + Action - Mar 2023
// - 2303.11366 - Reflexion - heuristic + self-reflection - Mar 2023
// - 2303.17071 - DERA - Distinct roles+responsibilities - Mar 2023

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::chain::agents::OODAAgent;
use crate::chain::schedulers::SingleAgentScheduler;
use crate::context::ContextDump;
use crate::tools::invocation::InvocationError;
use crate::tools::toolbox::{invoke_tool, InvokeResult, Toolbox};
use crate::tools::{TerminationMessage, ToolUseError};
use crate::{SapiensConfig, WeakRuntimeObserver};

/// Outcome of an invocation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Outcome {
    /// The invocation was successful
    Success {
        /// The result of the invocation
        result: String,
    },
    /// No valid invocation was found
    NoValidInvocationsFound {
        /// The invocation error
        e: InvocationError,
    },
    /// No invocation was found
    NoInvocationsFound {
        /// The invocation error
        e: InvocationError,
    },
    /// The invocation failed
    ToolUseError {
        /// The tool use error
        e: ToolUseError,
    },
}

/// A message that can be produced by an agent for another agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    /// A new task to be performed
    Task {
        /// The description of the task
        content: String,
    },
    /// A new observation
    Observation {
        /// The observation
        content: String,
    },
    /// A new orientation
    Orientation {
        /// The orientation
        content: String,
    },
    /// A new decision
    Decision {
        /// The decision
        content: String,
    },
    /// A new action
    Action {
        /// The action
        content: String,
    },
    /// A new result
    ActionResult {
        /// The number of invocations found in the message
        invocation_count: usize,
        /// The name of the tool that was invoked
        tool_name: Option<String>,
        /// The extracted input for the tool
        extracted_input: Option<String>,
        /// The outcome of the invocation
        outcome: Outcome,
    },
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Task { content } => write!(f, "Task: {}", content),
            Message::Observation { content } => write!(f, "Observation: {}", content),
            Message::Orientation { content } => write!(f, "Orientation: {}", content),
            Message::Decision { content } => write!(f, "Decision: {}", content),
            Message::Action { content } => write!(f, "Action: {}", content),
            Message::ActionResult {
                invocation_count,
                tool_name,
                extracted_input,
                outcome,
            } => write!(
                f,
                "ActionResult: {} invocations found, tool_name: {:?}, extracted_input: {:?}, outcome: {:?}",
                invocation_count,
                tool_name,
                extracted_input,
                outcome
            ),
        }
    }
}

impl From<InvokeResult> for Message {
    fn from(result: InvokeResult) -> Self {
        match result {
            InvokeResult::NoInvocationsFound { e } => Message::ActionResult {
                invocation_count: 0,
                tool_name: None,
                extracted_input: None,
                outcome: Outcome::NoInvocationsFound { e },
            },
            InvokeResult::NoValidInvocationsFound {
                e,
                invocation_count,
            } => Message::ActionResult {
                invocation_count,
                tool_name: None,
                extracted_input: None,
                outcome: Outcome::NoValidInvocationsFound { e },
            },
            InvokeResult::Success {
                invocation_count,
                tool_name,
                extracted_input,
                result,
            } => Message::ActionResult {
                invocation_count,
                tool_name: Some(tool_name),
                extracted_input: Some(extracted_input),
                outcome: Outcome::Success { result },
            },
            InvokeResult::Error {
                invocation_count,
                tool_name,
                e,
                ..
            } => Message::ActionResult {
                invocation_count,
                tool_name: Some(tool_name),
                extracted_input: None,
                outcome: Outcome::ToolUseError { e },
            },
        }
    }
}

/// The history of a [`Message`] produced by the [`Chain`].
#[derive(Clone, Default)]
pub struct Context {
    messages: Vec<Message>,
}

impl Context {
    /// Dump the context into a [`ContextDump`]
    pub fn dump(&self) -> ContextDump {
        ContextDump {
            messages: self.messages.clone(),
        }
    }
}

impl Context {
    /// Returns the latest task
    pub(crate) fn get_latest_task(&self) -> Option<String> {
        self.messages.iter().rev().find_map(|m| match m {
            Message::Task { content } => Some(content.clone()),
            _ => None,
        })
    }
}

/// An error that can occur during the creation or execution of a [`Chain`]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// No terminal tool in the toolbox
    #[error("No terminal tool")]
    NoTerminalTool,
    /// Max steps reached
    #[error("Max steps reached")]
    MaxStepsReached,
    /// Agent failed
    #[error("Agent failed: {0}")]
    AgentFailed(#[from] <OODAAgent as Agent>::Error),
}

/// An agent for sapiens
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// The error type of the agent
    type Error;

    /// Act on the given [`Context`] and return a [`Message`] or an error
    async fn act(&self, context: &Context) -> Result<Message, Self::Error>;
}

/// A scheduler for sapiens
#[async_trait::async_trait]
pub trait Scheduler: Send + Sync {
    /// Pick the next [`Agent`] to be called, call it and return the produced
    /// [`Message`]
    async fn schedule(&mut self, context: &Context) -> Result<Message, Error>;
}

/// A runtime of for sapiens
pub struct Runtime {
    context: Context,
    toolbox: Toolbox,
    scheduler: Box<dyn Scheduler>,
    observer: WeakRuntimeObserver,
}

/// The state of the runtime after it terminates
pub struct TerminalState {
    /// The messages produced by the runtime when it terminated
    pub messages: Vec<TerminationMessage>,
}

impl Runtime {
    /// Create a new [`Runtime`] with the given [`Toolbox`], [`Scheduler`] and
    /// [`WeakRuntimeObserver`].
    pub async fn new(
        toolbox: Toolbox,
        scheduler: Box<dyn Scheduler>,
        observer: WeakRuntimeObserver,
    ) -> Result<Self, Error> {
        if !toolbox.has_terminal_tools().await {
            return Err(Error::NoTerminalTool);
        }

        Ok(Self {
            context: Context::default(),
            toolbox,
            scheduler,
            observer,
        })
    }

    /// Run the runtime until it terminates.
    pub async fn run(&mut self) -> Result<TerminalState, Error> {
        loop {
            let messages = self.step().await?;
            if !messages.is_empty() {
                return Ok(TerminalState { messages });
            }
        }
    }

    /// Run one step of the runtime.
    pub async fn step(&mut self) -> Result<Vec<TerminationMessage>, Error> {
        let message = self.scheduler.schedule(&self.context).await?;

        self.context.messages.push(message.clone());

        if let Some(observer) = self.observer.upgrade() {
            observer
                .lock()
                .await
                .on_message(message.clone().into())
                .await;
        }

        // any action?
        if let Message::Action { content } = message {
            let res = invoke_tool(self.toolbox.clone(), &content).await;

            if let Some(observer) = self.observer.upgrade() {
                observer
                    .lock()
                    .await
                    .on_invocation_result(res.clone().into())
                    .await;
            }

            self.context.messages.push(res.into());
        }

        // are we done?
        Ok(self.toolbox.termination_messages().await)
    }
}

/// An OODA chain
pub struct OODAChain {
    runtime: Runtime,
}

impl OODAChain {
    /// Dump the current context
    pub fn dump(&self) -> ContextDump {
        self.runtime.context.dump()
    }
}

impl OODAChain {
    /// Create a new [`OODAChain`]
    pub async fn new(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Result<Self, Error> {
        let agent = OODAAgent::new(config.clone(), toolbox.clone(), observer.clone()).await;

        let scheduler =
            SingleAgentScheduler::new(config.max_steps, Box::new(agent), observer.clone());
        Ok(Self {
            runtime: Runtime::new(toolbox, Box::new(scheduler), observer).await?,
        })
    }

    /// Execute a single step of the OODA chain
    pub async fn step(&mut self) -> Result<Vec<TerminationMessage>, Error> {
        self.runtime.step().await
    }

    /// Add a new task to the OODA chain
    pub fn with_task(mut self, task: String) -> Self {
        self.runtime
            .context
            .messages
            .push(Message::Task { content: task });

        self
    }
}
