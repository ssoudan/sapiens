//! Bake the acceptance test for the task in the tools
//! Scenario -> (Recipe, Config)
//!
//! Recipe -> (Task, Acceptance criteria)
//!
//! Acceptance criteria -> Tools
//!
//! run_until_completion: (Task, Config, Tools) -> (Trace, Stats, Acceptance
//! results)
//!
//! evaluate::Trial::analyze: (Trace, Stats, Acceptance results) -> Analysis
//!
//! evaluate::Trial::build: (Trace, State, Acceptance results, Task, Analysis)
//! -> Trial

use std::marker::PhantomData;
use std::ops::DerefMut;
use std::sync::Arc;

use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Scenario 0: Preparing a cereal bowl
pub mod scenario_0;

mod scenario_1;

/// The state of a scenario
pub trait State: Send + Sync {
    /// Reset the state to its initial state
    fn reset(&mut self);

    /// Check if the state has reached an accepting state
    fn has_reached_accepting_state(&self) -> bool;

    /// State name
    fn state(&self) -> String;
}

/// Something that updates the state of the system and produces an output
pub trait StateUpdater<S, O> {
    /// Update the state of the system and produce an output
    fn update(&self, state: &mut S) -> Result<O, ToolUseError>;
}

/// A generic tool that can be used to update the state of the system
pub struct GenericTool<I, S, O> {
    name: String,
    tool_description: ToolDescription,
    phantom_input: PhantomData<I>,
    phantom_output: PhantomData<O>,
    state: Arc<Mutex<S>>,
}

impl<I, S, O> GenericTool<I, S, O> {
    /// Create a new tool
    pub fn new(name: String, tool_description: ToolDescription, state: Arc<Mutex<S>>) -> Self {
        Self {
            name,
            tool_description,
            phantom_input: PhantomData,
            phantom_output: PhantomData,
            state,
        }
    }

    /// Get the name of the tool
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<I: Describe, S, O: Describe> GenericTool<I, S, O> {
    /// Create a new tool
    pub fn new_with_descriptions(name: String, description: String, state: Arc<Mutex<S>>) -> Self {
        Self::new(
            name.clone(),
            ToolDescription {
                name,
                description,
                input_format: I::describe(),
                output_format: O::describe(),
            },
            state,
        )
    }
}

impl<I, S, O> ProtoToolDescribe for GenericTool<I, S, O> {
    fn description(&self) -> ToolDescription {
        self.tool_description.clone()
    }
}

#[async_trait::async_trait]
impl<I, S, O> ProtoToolInvoke for GenericTool<I, S, O>
where
    I: Sync + for<'a> Deserialize<'a> + StateUpdater<S, O> + Send,
    S: Sync + Send,
    O: Sync + Serialize + Send,
{
    async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input: I = serde_yaml::from_value(input)?;

        let output = {
            // Lock
            let mut guard = self.state.lock().await;

            input.update(guard.deref_mut())?
        };

        Ok(serde_yaml::to_value(output)?)
    }
}

#[cfg(test)]
mod tests {
    use sapiens::tools::{Describe, FieldFormat, Format};
    use serde::{Deserialize, Serialize};

    use super::*;

    struct State {
        counter: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Output {
        counter: u32,
    }

    impl Describe for Output {
        fn describe() -> Format {
            vec![FieldFormat {
                name: "output".to_string(),
                r#type: "u32".to_string(),
                optional: false,
                description: "current value of the counter".to_string(),
            }]
            .into()
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Input {
        increment: u32,
    }

    impl Describe for Input {
        fn describe() -> Format {
            vec![FieldFormat {
                name: "increment".to_string(),
                r#type: "u32".to_string(),
                optional: false,
                description: "value to increment the counter by".to_string(),
            }]
            .into()
        }
    }

    impl StateUpdater<u32, Output> for Input {
        fn update(&self, state: &mut u32) -> Result<Output, ToolUseError> {
            *state += self.increment;
            Ok(Output { counter: *state })
        }
    }

    #[test]
    fn test_state_updater() {
        let mut state = State { counter: 0 };
        let input = Input { increment: 1 };
        let output = {
            let c = &mut state.counter;
            input.update(c)
        }
        .unwrap();
        assert_eq!(output.counter, 1);
        assert_eq!(state.counter, 1);

        let input = Input { increment: 2 };
        let output = input.update(&mut state.counter).unwrap();
        assert_eq!(output.counter, 3);
        assert_eq!(state.counter, 3);
    }

    #[tokio::test]
    async fn test_generic_tool() {
        let tool = GenericTool::<Input, u32, Output> {
            name: "Increment".to_string(),
            tool_description: ToolDescription::new(
                "Increment",
                "Increment the counter by the given value",
                Input::describe(),
                Output::describe(),
            ),
            phantom_input: PhantomData,
            phantom_output: PhantomData,
            state: Arc::new(Mutex::new(0)),
        };

        let input = serde_yaml::to_value(Input { increment: 1 }).unwrap();
        let output = tool.invoke(input).await.unwrap();
        let output: Output = serde_yaml::from_value(output).unwrap();
        assert_eq!(output.counter, 1);
    }
}
