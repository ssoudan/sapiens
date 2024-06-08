//! Execution chains
//! - [x] OODA - Observe, Orient, Decide, Act
//!   - [x] in single step - See [`SingleStepOODAChain`]
//!   - [x] in several steps - See [`MultiStepOODAChain`]
//! - [ ] 2205.11916 - Zeroshot reasoners - "Let's think step by step" - 2022
//! - [ ] 2207.05608 - Inner monologue - Different types of feedbacks - 2022
//! - [ ] 2302.00083 - In context RALM - Jan 2023
//! - [ ] 2302.01560 - DEPS - Describe, explain, plan, select stages. Feb 2023
//! - [ ] 2210.03629 - `ReAct` - Reasoning + Action - Mar 2023
//! - [ ] 2303.11366 - Reflexion - heuristic + self-reflection - Mar 2023
//! - [ ] 2303.17071 - DERA - Distinct roles+responsibilities - Mar 2023
//! - [ ] 2305.10601 - Tree of Thoughts - May 2023
// TODO(ssoudan) + LLM self-consistency

// FUTURE(ssoudan) more chains

/// Agents
pub mod agents;
/// Schedulers are responsible for deciding which agent to run next.
pub mod schedulers;

#[cfg(test)]
mod tests;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::chains::agents::ooda::{multistep, one_step};
use crate::chains::schedulers::{MultiAgentScheduler, SingleAgentScheduler};
use crate::context::ContextDump;
use crate::models::Usage;
use crate::tools::toolbox::{invoke_tool, InvokeResult, Toolbox};
use crate::tools::{TerminationMessage, ToolUseError};
use crate::{invocation, SapiensConfig, WeakRuntimeObserver};

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
        e: invocation::Error,
    },
    /// No invocation was found
    NoInvocationsFound {
        /// The invocation error
        e: invocation::Error,
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
        /// Token usage
        usage: Option<Usage>,
    },
    /// A new orientation
    Orientation {
        /// The orientation
        content: String,
        /// Token usage
        usage: Option<Usage>,
    },
    /// A new decision
    Decision {
        /// The decision
        content: String,
        /// Token usage
        usage: Option<Usage>,
    },
    /// A new action
    Action {
        /// The action
        content: String,
        /// Token usage
        usage: Option<Usage>,
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
            Self::Task { content } => write!(f, "Task: {content}"),
            Self::Observation { content,.. } => write!(f, "Observation: {content}"),
            Self::Orientation { content ,..} => write!(f, "Orientation: {content}"),
            Self::Decision { content,.. } => write!(f, "Decision: {content}"),
            Self::Action { content ,..} => write!(f, "Action: {content}"),
            Self::ActionResult {
                invocation_count,
                tool_name,
                extracted_input,
                outcome,
            } => write!(
                f,
                "ActionResult: {invocation_count} invocations found, tool_name: {tool_name:?}, extracted_input: {extracted_input:?}, outcome: {outcome:?}",                                
            ),
        }
    }
}

impl From<InvokeResult> for Message {
    fn from(result: InvokeResult) -> Self {
        match result {
            InvokeResult::NoInvocationsFound { e } => Self::ActionResult {
                invocation_count: 0,
                tool_name: None,
                extracted_input: None,
                outcome: Outcome::NoInvocationsFound { e },
            },
            InvokeResult::NoValidInvocationsFound {
                e,
                invocation_count,
            } => Self::ActionResult {
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
            } => Self::ActionResult {
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
            } => Self::ActionResult {
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
    /// Create a new context
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Dump the context into a [`ContextDump`]
    #[must_use]
    pub fn dump(&self) -> ContextDump {
        ContextDump {
            messages: self.messages.clone(),
        }
    }

    /// Returns the latest task
    #[must_use]
    pub fn get_latest_task(&self) -> Option<String> {
        self.messages.iter().rev().find_map(|m| match m {
            Message::Task { content } => Some(content.clone()),
            _ => None,
        })
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
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
    AgentFailed(#[from] agents::Error),
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

/// A runtime for sapiens
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
        if let Message::Action { content, .. } = message {
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

/// a chain of steps to perform a task.
#[async_trait::async_trait]
pub trait Chain: Send + Sync {
    /// Dump the current context
    fn dump(&self) -> ContextDump;

    /// Execute a single step of the chain
    async fn step(&mut self) -> Result<Vec<TerminationMessage>, Error>;
}

/// A single-step OODA chain
pub struct SingleStepOODAChain {
    runtime: Runtime,
}

impl SingleStepOODAChain {
    /// Create a new [`SingleStepOODAChain`]
    pub async fn new(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Result<Self, Error> {
        let agent = one_step::Agent::new(config.clone(), toolbox.clone(), observer.clone());

        let scheduler =
            SingleAgentScheduler::new(config.max_steps, Box::new(agent), observer.clone());
        Ok(Self {
            runtime: Runtime::new(toolbox, Box::new(scheduler), observer).await?,
        })
    }

    /// Add a new task to the OODA chain
    #[must_use]
    pub fn with_task(mut self, task: String) -> Self {
        self.runtime
            .context
            .messages
            .push(Message::Task { content: task });

        self
    }
}

#[async_trait::async_trait]
impl Chain for SingleStepOODAChain {
    fn dump(&self) -> ContextDump {
        self.runtime.context.dump()
    }

    async fn step(&mut self) -> Result<Vec<TerminationMessage>, Error> {
        self.runtime.step().await
    }
}

/// Multistep OODA chain
pub struct MultiStepOODAChain {
    /// The runtime of the chain
    runtime: Runtime,
}

impl MultiStepOODAChain {
    /// Create a new [`MultiStepOODAChain`]
    pub async fn new(
        config: SapiensConfig,
        toolbox: Toolbox,
        observer: WeakRuntimeObserver,
    ) -> Result<Self, Error> {
        let agents = vec![
            multistep::Agent::new_observer(config.clone(), toolbox.clone(), observer.clone()),
            multistep::Agent::new_orienter(config.clone(), toolbox.clone(), observer.clone()),
            multistep::Agent::new_decider(config.clone(), toolbox.clone(), observer.clone()),
            multistep::Agent::new_actor(config.clone(), toolbox.clone(), observer.clone()),
        ];

        let agents = agents
            .into_iter()
            .map(|a| Box::new(a) as Box<dyn Agent<Error = agents::Error>>)
            .collect();

        let scheduler = MultiAgentScheduler::new(config.max_steps, agents, observer.clone());
        Ok(Self {
            runtime: Runtime::new(toolbox, Box::new(scheduler), observer).await?,
        })
    }

    /// Add a new task to the OODA chain
    #[must_use]
    pub fn with_task(mut self, task: String) -> Self {
        self.runtime
            .context
            .messages
            .push(Message::Task { content: task });

        self
    }
}

#[async_trait::async_trait]
impl Chain for MultiStepOODAChain {
    fn dump(&self) -> ContextDump {
        self.runtime.context.dump()
    }

    async fn step(&mut self) -> Result<Vec<TerminationMessage>, Error> {
        self.runtime.step().await
    }
}
