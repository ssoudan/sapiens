//! Sapiens library
//!
//! *Sapiens uses tools to interact with the world.*
//!
//! An experiment with handing over the tools to the machine.
//!
//! # Overview
//! This library is the core of Sapiens. It contains the logic for the
//! interaction between the user, the language model and the tools.
//!
//! # More information
//! See https://github.com/ssoudan/sapiens/tree/main/sapiens_cli for an example of usage or
//! https://github.com/ssoudan/sapiens/tree/main/sapiens_bot for a Discord bot.
//!
//! https://github.com/ssoudan/sapiens/tree/main/sapiens_exp is a framework to run experiments and collect traces
//! of the interactions between the language model and the tools to accomplish a
//! task.
//!
//! A collection of tools is defined in https://github.com/ssoudan/sapiens/tree/main/sapiens_tools.
pub mod context;

/// Prompt generation logic
pub mod prompt;

/// Toolbox for sapiens
pub mod tools;

/// Language models
pub mod models;

pub mod chains;

use std::fmt::Debug;
use std::str::FromStr;
use std::sync::{Arc, Weak};

use clap::builder::PossibleValue;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::chains::{Chain, Message, MultiStepOODAChain, SingleStepOODAChain};
use crate::context::{ChatEntry, ContextDump};
use crate::models::openai::OpenAI;
use crate::models::{ModelRef, ModelResponse, Role, Usage};
use crate::tools::invocation::InvocationError;
use crate::tools::toolbox::{InvokeResult, Toolbox};
use crate::tools::{TerminationMessage, ToolUseError};

/// The error type for the bot
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to add to the chat history
    #[error("Failed to add to the chat history: {0}")]
    ChatHistoryError(#[from] context::Error),
    /// Model evaluation error
    #[error("Model evaluation error: {0}")]
    ModelEvaluationError(#[from] models::Error),
    /// Reached the maximum number of steps
    #[error("Maximal number of steps reached")]
    MaxStepsReached,
    /// The response is too long
    #[error("The response is too long: {0}")]
    ActionResponseTooLong(String),
    /// Error in the chain
    #[error("Chain error: {0}")]
    ChainError(#[from] chains::Error),
}

/// Type of chain to use
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    /// OODA single step chain
    #[default]
    SingleStepOODA,
    /// OODA multi step chain
    MultiStepOODA,
}

impl FromStr for ChainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single-step-ooda" => Ok(ChainType::SingleStepOODA),
            "multi-step-ooda" => Ok(ChainType::MultiStepOODA),
            _ => Err(format!("Unknown chain type: {}", s)),
        }
    }
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for ChainType {
    fn value_variants<'a>() -> &'a [Self] {
        &[ChainType::SingleStepOODA, ChainType::MultiStepOODA]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            ChainType::SingleStepOODA => Some(PossibleValue::new("single-step-ooda")),
            ChainType::MultiStepOODA => Some(PossibleValue::new("multi-step-ooda")),
        }
    }
}

/// Configuration for the bot
#[derive(Clone)]
pub struct SapiensConfig {
    /// The model to use
    pub model: ModelRef,
    /// The maximum number of steps
    pub max_steps: usize,
    /// The type of chain to use
    pub chain_type: ChainType,
    /// The minimum number of tokens that need to be available for completion
    pub min_tokens_for_completion: usize,
    /// Maximum number of tokens for the model to generate
    pub max_tokens: Option<usize>,
}

impl Debug for SapiensConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("max_steps", &self.max_steps)
            .field("chain_type", &self.chain_type)
            .field("min_tokens_for_completion", &self.min_tokens_for_completion)
            .field("max_tokens", &self.max_tokens)
            .finish()
    }
}

impl Default for SapiensConfig {
    fn default() -> Self {
        Self {
            model: Arc::new(Box::<OpenAI>::default()),
            max_steps: 10,
            chain_type: ChainType::SingleStepOODA,
            min_tokens_for_completion: 256,
            max_tokens: None,
        }
    }
}

/// An update from the model
#[derive(Debug, Clone)]
pub struct ModelNotification {
    /// The message from the model
    pub chat_entry: ChatEntry,
    /// The number of tokens used by the model
    pub usage: Option<Usage>,
}

impl From<ModelResponse> for ModelNotification {
    fn from(res: ModelResponse) -> Self {
        Self {
            chat_entry: ChatEntry {
                role: Role::Assistant,
                msg: res.msg,
            },
            usage: res.usage,
        }
    }
}

/// A message from a scheduler
#[derive(Debug, Clone)]
pub struct MessageNotification {
    /// The message from the scheduler
    pub message: Message,
}

impl From<Message> for MessageNotification {
    fn from(message: Message) -> Self {
        Self { message }
    }
}

/// Notification of the result of a tool invocation
pub enum InvocationResultNotification {
    /// Invocation success notification
    InvocationSuccess(InvocationSuccessNotification),
    /// Invocation failure notification
    InvocationFailure(InvocationFailureNotification),
    /// Invalid invocation notification
    InvalidInvocation(InvalidInvocationNotification),
}

impl From<InvokeResult> for InvocationResultNotification {
    fn from(res: InvokeResult) -> Self {
        match res {
            InvokeResult::NoInvocationsFound { e } => {
                InvocationResultNotification::InvalidInvocation(InvalidInvocationNotification {
                    e,
                    invocation_count: 0,
                })
            }
            InvokeResult::NoValidInvocationsFound {
                e,
                invocation_count,
            } => InvocationResultNotification::InvalidInvocation(InvalidInvocationNotification {
                e,
                invocation_count,
            }),
            InvokeResult::Success {
                invocation_count,
                tool_name,
                extracted_input,
                result,
            } => InvocationResultNotification::InvocationSuccess(InvocationSuccessNotification {
                invocation_count,
                tool_name,
                extracted_input,
                result,
            }),
            InvokeResult::Error {
                invocation_count,
                tool_name,
                extracted_input,
                e,
            } => InvocationResultNotification::InvocationFailure(InvocationFailureNotification {
                invocation_count,
                tool_name,
                extracted_input,
                e,
            }),
        }
    }
}

/// Invocation success notification
pub struct InvocationSuccessNotification {
    /// The number of invocation blocks in the message
    pub invocation_count: usize,
    /// The tool name
    pub tool_name: String,
    /// The input that was extracted from the message and passed to `tool_name`
    pub extracted_input: String,
    /// The result
    pub result: String,
}

/// Invocation failure notification
pub struct InvocationFailureNotification {
    /// Number of invocation  blocks in the message
    pub invocation_count: usize,
    /// The tool name
    pub tool_name: String,
    /// The input that was extracted from the message and passed to `tool_name`
    pub extracted_input: String,
    /// The result
    pub e: ToolUseError,
}

/// Invalid invocation notification
pub struct InvalidInvocationNotification {
    /// The result
    pub e: InvocationError,
    /// Number of invocation blocks in the message
    pub invocation_count: usize,
}

/// Termination notification
pub struct TerminationNotification {
    /// The messages
    pub messages: Vec<TerminationMessage>,
}

/// Observer for the step progresses
#[async_trait::async_trait]
pub trait RuntimeObserver: Send {
    /// Called when the task is submitted
    async fn on_task(&mut self, _task: &str) {}

    /// Called on start
    async fn on_start(&mut self, _context: ContextDump) {}

    /// Called when the model returns something
    async fn on_model_update(&mut self, _event: ModelNotification) {}

    /// Called when the scheduler has selected a message
    async fn on_message(&mut self, _event: MessageNotification) {}

    /// Called when the tool invocation was successful
    async fn on_invocation_result(&mut self, _event: InvocationResultNotification) {}

    /// Called when the task is done
    async fn on_termination(&mut self, _event: TerminationNotification) {}
}

/// Wrap an observer into the a [`StrongRuntimeObserver<O>`] = [`Arc<Mutex<O>>`]
///
/// Use [`Arc::downgrade`] to get a [`Weak<Mutex<dyn RuntimeObserver>>`] and
/// pass it to [`run_to_the_end`] for example.
pub fn wrap_observer<O: RuntimeObserver + 'static>(observer: O) -> StrongRuntimeObserver<O> {
    Arc::new(Mutex::new(observer))
}

/// A strong reference to the observer
pub type StrongRuntimeObserver<O> = Arc<Mutex<O>>;

/// A weak reference to the observer
pub type WeakRuntimeObserver = Weak<Mutex<dyn RuntimeObserver>>;

/// A void observer
pub struct VoidTaskProgressUpdateObserver;

#[cfg(test)]
pub(crate) fn void_observer() -> StrongRuntimeObserver<VoidTaskProgressUpdateObserver> {
    wrap_observer(VoidTaskProgressUpdateObserver)
}

#[async_trait::async_trait]
impl RuntimeObserver for VoidTaskProgressUpdateObserver {}

/// A step in the task
pub struct Step {
    task_chain: Box<dyn Chain>,
    observer: WeakRuntimeObserver,
}

impl Step {
    /// Run the task for a single step
    async fn step(mut self) -> Result<TaskState, Error> {
        let termination_messages = self.task_chain.step().await?;

        // check if the task is done
        if !termination_messages.is_empty() {
            if let Some(observer) = self.observer.upgrade() {
                observer
                    .lock()
                    .await
                    .on_termination(TerminationNotification {
                        messages: termination_messages.clone(),
                    })
                    .await;
            }

            return Ok(TaskState::Stop {
                stop: Stop {
                    termination_messages,
                },
            });
        }

        Ok(TaskState::Step { step: self })
    }
}

/// The task is done
pub struct Stop {
    /// The termination messages
    pub termination_messages: Vec<TerminationMessage>,
}

/// The state machine of a task
pub enum TaskState {
    /// The task is not done yet
    Step {
        /// The actual step task
        step: Step,
    },
    /// The task is done
    Stop {
        /// the actual stopped task
        stop: Stop,
    },
}

impl TaskState {
    /// Create a new [`TaskState`] for a `task`.
    pub async fn new(config: SapiensConfig, toolbox: Toolbox, task: String) -> Result<Self, Error> {
        let observer = wrap_observer(VoidTaskProgressUpdateObserver {});
        let observer = Arc::downgrade(&observer);

        TaskState::with_observer(config, toolbox, task, observer).await
    }

    /// Create a new [`TaskState`] for a `task`.
    ///
    /// The `observer` will be called when the task starts and when a step is
    /// completed - either successfully or not. The `observer` will be called
    /// with the latest chat history element. It is also called on error.
    pub async fn with_observer(
        config: SapiensConfig,
        toolbox: Toolbox,
        task: String,
        observer: WeakRuntimeObserver,
    ) -> Result<Self, Error> {
        if let Some(observer) = observer.upgrade() {
            observer.lock().await.on_task(&task).await;
        }

        let task_chain = match config.chain_type {
            ChainType::SingleStepOODA => {
                let chain = SingleStepOODAChain::new(config, toolbox, observer.clone())
                    .await?
                    .with_task(task);
                Box::new(chain) as Box<dyn Chain>
            }
            ChainType::MultiStepOODA => {
                let chain = MultiStepOODAChain::new(config, toolbox, observer.clone())
                    .await?
                    .with_task(task);
                Box::new(chain) as Box<dyn Chain>
            }
        };

        // call the observer
        if let Some(observer) = observer.upgrade() {
            observer.lock().await.on_start(task_chain.dump()).await;
        }

        Ok(TaskState::Step {
            step: Step {
                task_chain,
                observer,
            },
        })
    }

    /// Run the task until it is done
    pub async fn run(mut self) -> Result<Stop, Error> {
        loop {
            match self {
                TaskState::Step { step } => {
                    self = step.step().await?;
                }
                TaskState::Stop { stop } => {
                    return Ok(stop);
                }
            }
        }
    }

    /// Run the task for a single step
    pub async fn step(self) -> Result<Self, Error> {
        match self {
            TaskState::Step { step } => step.step().await,
            TaskState::Stop { stop } => Ok(TaskState::Stop { stop }),
        }
    }

    /// is the task done?
    pub fn is_done(&self) -> Option<Vec<TerminationMessage>> {
        match self {
            TaskState::Step { step: _ } => None,
            TaskState::Stop { stop } => Some(stop.termination_messages.clone()),
        }
    }
}

/// Run until the task is done or the maximum number of steps is reached
///
/// See [`TaskState::new`], [`TaskState::step`] and [`TaskState::run`] for
/// more flexible ways to run a task
#[tracing::instrument(skip(toolbox, observer, config))]
pub async fn run_to_the_end(
    config: SapiensConfig,
    toolbox: Toolbox,
    task: String,
    observer: WeakRuntimeObserver,
) -> Result<Vec<TerminationMessage>, Error> {
    let task_state = TaskState::with_observer(config, toolbox, task, observer).await?;

    let stop = task_state.run().await?;

    Ok(stop.termination_messages)
}
