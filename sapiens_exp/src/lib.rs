//! Sapiens CLI library

use serde::{Deserialize, Serialize};

/// Tools related to experimentation.
///
/// They are intended to be used for experimentation around the language models
/// abilities, chaining and prompting only.
pub(crate) mod tools;

/// Execution traces.
pub mod traces;

/// Setup toolboxes and stuffs.
pub mod setup;

/// Evaluate a trial.
pub mod evaluate;

// Factors we want to be able to explore:
// ----------------------------
// Tooling:
// - Tool description format/serialization
// - Tool description content: example vs schema
// - Number of available tools
// - Complexity of the tools
// - 'Framework' to use tools - chaining, availability of templating, etc.
//
// Language Model:
// - LM choice
// - Invocation parameters
// - Fine-tuning if available
//
// Prompting:
// - Initial prompt
// - Response format
// - Recurring prompts
// - Richness of the response - and errors
// - Chat history management

// Elementary tasks to assess:
// ----------------
// - Identify tools
// - Identify how to use a tool
// - Identify input/output relationship for a tool - pick the input to get a
//   specific output
// - Identify when to use a tool
// - Identify how to use the output of a tool
// - Use the tool effectively
// - Use the tool efficiently
// - Follow tool specific instructions
// - Use combinations of tools
//   - Identify the right combination of tools
//   - Right order of tools
//   - Right I/O parameters
// - Ability to adapt and change plans
// - Use alternative ways to achieve the same goal
// - Planning
// - Execution

// Performance dimensions:
// --------------------
// - Outcome (success/failure)
// - Number of steps to success
// - Pathological behaviors (loops, drifts, hallucinated results, etc.)
// - Efficiency of the solution
// - Number of 'tokens' used
// - Repeatability
// - Ability to adapt to changes or circumvent problems

// Types of tasks:
// --------------
// - Tasks with a single solution
// - Tasks with multiple solutions
// - Tasks with no solution
// - Tasks with a single solution but multiple ways to get there
// - Tasks with a single solution but multiple ways to get there with different
//   outcomes
// - Tasks with a single solution but multiple ways to get there with different
//   outcomes and different number of steps
// - Tasks with different number of steps
// - Tasks with different number of tools used
// - Tasks with constraints on the tools used
// - Tasks with constraints on the order of the tools used
// - Tasks with constraints on the number of tools used
// - Tasks with constraints on the number of steps

// Example of Task:
// ----------------
// - Make a cake for 8 people
// - Make a cake for 8 people with chocolate
// - Make a sandwich
// - Make an omelette
// - Make an omelette with no eggs
// - Make a sandwich with ham
// - Make a sandwich with ham and cheese
// - Make a lunch for 8 people
// - Make a lime pie
// - Make a meal with no electricity
// - Make a vegetarian sandwich

// Type of tools:
// --------------
// - Tools that can do 1 thing
// - Tools that can do multiple things
// - Tools that can use other tools
// - Tools that have constraints on their use
// - Tools that are terminal
// - Tools that are alternative to other tools
// - Tools that are complementary to other tools. They can or have to be used
//   together. In a specific order or not.
// - Tools that are exclusive to other tools. They cannot be used together.
// - Tools that do repeated things

// Tools
// -----
// - Oven
// - Fridge
// - Microwave
// - Toaster
// - Mixer
// - Knife
// - Trash
// - Grater
// - Bowl
// - Whisk
// - Pan
// - Pot
// - Squeezer

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Model to use
    pub model: String,
    /// Maximum number of steps to execute
    pub max_steps: usize,
}

impl From<&Config> for sapiens::Config {
    fn from(config: &Config) -> Self {
        Self {
            model: config.model.clone(),
            max_steps: config.max_steps,
            ..Default::default()
        }
    }
}
