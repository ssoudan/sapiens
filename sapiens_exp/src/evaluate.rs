use sapiens::tools::toolbox::Stats;
use serde::{Deserialize, Serialize};

use crate::traces::{CompletionStatus, Event, Trace, Usage};
use crate::Config;

/// The score of a trial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    /// The number of attempted tool invocations - Conclude Tool is not counted
    /// here.
    attempted_invocations: u32,
    /// The number of successful tool invocations - Conclude Tool is not counted
    /// here.
    successful_invocations: u32,
    /// The number of tokens
    tokens: Usage,
    /// Completion status
    completed: bool,
    /// Reached accepting state
    reached_accepting_state: bool,
    /// Termination message
    termination_message: Option<String>,
    /// Tool utilization statistics
    tool_stats: Stats,
    /// The final state name
    final_state_name: String,
}

/// A trial is a task execution with a given configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    /// The trace of the execution
    trace: Trace,
    /// The task
    task: String,
    /// The configuration,
    config: Config,
    /// The Analysis of the run
    analysis: Analysis,
    /// Date and time of the trial
    date: String,
}

impl Trial {
    /// Create a new trial
    pub fn build(
        config: Config,
        task: String,
        trace: Trace,
        tool_stats: Stats,
        reached_accepting_state: bool,
        final_state_name: String,
    ) -> Self {
        let analysis = Self::analyze(
            &trace,
            tool_stats,
            reached_accepting_state,
            final_state_name,
        );

        Self {
            trace,
            task,
            config,
            analysis,
            date: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn analyze(
        trace: &Trace,
        tool_stats: Stats,
        reached_accepting_state: bool,
        final_state_name: String,
    ) -> Analysis {
        let attempted_invocations = trace
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event.event,
                    Event::ToolInvocationSucceeded { .. }
                        | Event::ToolInvocationFailed { .. }
                        | Event::ToolInvocationFailedAndChatNotUpdated { .. }
                )
            })
            .count() as u32;

        let successful_invocations = trace
            .events
            .iter()
            .filter(|event| matches!(event.event, Event::ToolInvocationSucceeded { .. }))
            .count() as u32;

        let tokens = trace.events.iter().fold(Usage::default(), |acc, event| {
            acc + event.event.tokens().unwrap_or_default()
        });

        let termination_message = trace.events.iter().find_map(|event| {
            if let Event::End(CompletionStatus::Concluded { conclusion, .. }) = &event.event {
                Some(conclusion.clone())
            } else {
                None
            }
        });

        let completed = termination_message.is_some();

        Analysis {
            attempted_invocations,
            successful_invocations,
            tokens,
            completed,
            reached_accepting_state,
            final_state_name,
            termination_message,
            tool_stats,
        }
    }
}
