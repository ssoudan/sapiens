//! # Scenario 0
//! ----------
//! *"Make a cereal bowl" with "multiple ways to get to a single solution"*
//!
//! ## Sub-task
//! - Get bowl
//! - Get cereal
//! - Get milk
//! - Add cereal to bowl
//! - Add milk to bowl
//! - Serve cereal
//!
//! ## Tools
//! - Get <x>
//! - Add <x> to <y>
//! - Serve <x>
//!
//! See [`build`].
//!
//! ## With tools
//! - Get <bowl>
//! - Get <cereal>
//! - Get <milk>
//! - Add <cereal> to <bowl>
//! - Add <milk> to <bowl>
//! - Serve <bowl>
//!
//! ## Acceptance criteria
//! - The bowl contains the cereal &&
//! - The bowl contains the milk &&
//! - The bowl is served
//!
//! See [`InternalState::has_reached_accepting_state`]
//!
//! ### State machine:
//! See [`CerealBowlRecipe`]

use std::sync::Arc;

use rust_fsm::*;
use sapiens::tools::toolbox::Toolbox;
use sapiens::tools::{Describe, ToolUseError};
use sapiens_derive::Describe;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::tools;
use crate::tools::{GenericTool, StateUpdater};

state_machine! {
    derive(Debug)
    repr_c(false)
    CerealBowlRecipe(NoBowlNoCerealNoMilk)

    // procurement
    NoBowlNoCerealNoMilk => {
        GetBowl => BowlNoCerealNoMilk [BowlFound],
        GetCereal => NoBowlCerealNoMilk [CerealFound],
        GetMilk => NoBowlNoCerealMilk [MilkFound],
    },

    BowlNoCerealNoMilk => {
        GetCereal => BowlCerealNoMilk [CerealFound],
        GetMilk => BowlNoCerealMilk [MilkFound],
        GetBowl => BowlNoCerealNoMilk [BowlFound],
    },

    NoBowlCerealNoMilk => {
        GetBowl => BowlCerealNoMilk [BowlFound],
        GetMilk => NoBowlCerealMilk [MilkFound],
        GetCereal => NoBowlCerealNoMilk [CerealFound],
    },

    NoBowlNoCerealMilk => {
        GetBowl => BowlNoCerealMilk [BowlFound],
        GetCereal => NoBowlNoCerealMilk [CerealFound],
        GetMilk => NoBowlNoCerealMilk [MilkFound],
    },

    BowlCerealNoMilk => {
        GetBowl => BowlCerealNoMilk [BowlFound],
        GetCereal => BowlCerealNoMilk [CerealFound],
        GetMilk => BowlCerealMilk [MilkFound],
    },

    BowlNoCerealMilk => {
        GetBowl => BowlNoCerealMilk [BowlFound],
        GetCereal => BowlCerealMilk [CerealFound],
        GetMilk => BowlNoCerealMilk [MilkFound],
    },

    NoBowlCerealMilk => {
        GetBowl => BowlCerealMilk [BowlFound],
        GetCereal => NoBowlCerealMilk [CerealFound],
        GetMilk => NoBowlCerealMilk [MilkFound],
    },

    // half way
    NoMilkBowlWithCereal => {
        GetMilk => BowlWithCereal [MilkFound],
    },

    NoCerealBowlWithMilk => {
        GetCereal => BowlWithMilk [CerealFound],
    },

    // mixing
    BowlCerealNoMilk => {
        AddCerealToBowl => NoMilkBowlWithCereal [CerealAdded],
    },

    BowlNoCerealMilk => {
        AddMilkToBowl => NoCerealBowlWithMilk [MilkAdded],
    },

    BowlCerealMilk => {
        AddCerealToBowl => BowlWithCereal [CerealAdded],
        AddMilkToBowl => BowlWithMilk [MilkAdded],
    },
    BowlWithCereal(AddMilkToBowl) => BowlWithCerealAndMilk [MilkAdded],
    BowlWithMilk(AddCerealToBowl) => BowlWithCerealAndMilk [CerealAdded],

    // serving
    BowlWithCerealAndMilk(ServeBowl) => Served [Accepted],
}

/// The state of the scenario 0
struct InternalState {
    fsm: StateMachine<CerealBowlRecipe>,
}

impl InternalState {
    fn new() -> Self {
        Self {
            fsm: StateMachine::new(),
        }
    }

    /// Get the current state
    #[allow(dead_code)]
    fn state(&self) -> &CerealBowlRecipeState {
        self.fsm.state()
    }
}

impl tools::State for InternalState {
    fn reset(&mut self) {
        self.fsm = StateMachine::new();
    }

    fn has_reached_accepting_state(&self) -> bool {
        matches!(self.fsm.state(), CerealBowlRecipeState::Served)
    }

    fn state(&self) -> String {
        format!("{:?}", self.fsm.state())
    }
}

impl Default for InternalState {
    fn default() -> Self {
        Self::new()
    }
}

/// What sits in the closet
#[derive(Debug, Serialize, Deserialize, Clone)]
enum ClosetObject {
    /// Bowl
    Bowl,
    /// Cereal
    Cereal,
    /// Milk
    Milk,
}

#[derive(Debug, Describe, Serialize, Deserialize, Clone)]
/// The input of a closet
struct ClosetInput {
    /// what to get. Value can be: Bowl, Cereal, or Milk. Only one at a time.
    get: ClosetObject,
}

impl StateUpdater<InternalState, ClosetOutput> for ClosetInput {
    fn update(&self, state: &mut InternalState) -> Result<ClosetOutput, ToolUseError> {
        match &self.get {
            ClosetObject::Bowl => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::GetBowl)
                    .ok()
                    .flatten();
                Ok(x.into())
            }
            ClosetObject::Cereal => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::GetCereal)
                    .ok()
                    .flatten();
                Ok(x.into())
            }
            ClosetObject::Milk => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::GetMilk)
                    .ok()
                    .flatten();
                Ok(x.into())
            }
        }
    }
}

/// The output of a closet
#[derive(Debug, Serialize, Deserialize, Clone, Describe)]
struct ClosetOutput {
    /// was the object found?
    found: bool,
    /// what was found? (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<ClosetObject>,
}

impl From<Option<CerealBowlRecipeOutput>> for ClosetOutput {
    fn from(output: Option<CerealBowlRecipeOutput>) -> Self {
        match output {
            Some(CerealBowlRecipeOutput::CerealFound) => ClosetOutput {
                found: true,
                object: Some(ClosetObject::Cereal),
            },
            Some(CerealBowlRecipeOutput::MilkFound) => ClosetOutput {
                found: true,
                object: Some(ClosetObject::Milk),
            },
            Some(CerealBowlRecipeOutput::BowlFound) => ClosetOutput {
                found: true,
                object: Some(ClosetObject::Bowl),
            },
            _ => ClosetOutput {
                found: false,
                object: None,
            },
        }
    }
}

/// What to poor in
#[derive(Debug, Serialize, Deserialize, Clone)]
enum Container {
    /// Bowl
    Bowl,
}

/// What to pour
#[derive(Debug, Serialize, Deserialize, Clone)]
enum Pourable {
    /// Cereal
    Cereal,
    /// Milk
    Milk,
}

#[derive(Debug, Describe, Serialize, Deserialize, Clone)]
/// The input of a Mixing action
struct MixingInput {
    /// What to pour in. Value can be: Bowl. You must have it first.
    container: Container,
    /// What to pour. Value can be: Cereal or Milk. You must have it first.
    pourable: Pourable,
}

impl StateUpdater<InternalState, MixingOutput> for MixingInput {
    fn update(&self, state: &mut InternalState) -> Result<MixingOutput, ToolUseError> {
        match (&self.container, &self.pourable) {
            (Container::Bowl, Pourable::Cereal) => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::AddCerealToBowl)
                    .ok()
                    .flatten();
                Ok(x.into())
            }
            (Container::Bowl, Pourable::Milk) => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::AddMilkToBowl)
                    .ok()
                    .flatten();
                Ok(x.into())
            }
        }
    }
}

/// The output of a mixing action
#[derive(Debug, Serialize, Deserialize, Clone, Describe)]
struct MixingOutput {
    /// was it poured?
    added: bool,
    /// what was poured? (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<Pourable>,
}

impl From<Option<CerealBowlRecipeOutput>> for MixingOutput {
    fn from(output: Option<CerealBowlRecipeOutput>) -> Self {
        match output {
            Some(CerealBowlRecipeOutput::MilkAdded) => MixingOutput {
                added: true,
                object: Some(Pourable::Milk),
            },
            Some(CerealBowlRecipeOutput::CerealAdded) => MixingOutput {
                added: true,
                object: Some(Pourable::Cereal),
            },
            _ => MixingOutput {
                added: false,
                object: None,
            },
        }
    }
}

/// What to serve
#[derive(Debug, Serialize, Deserialize, Clone)]
enum Serveable {
    /// Bowl
    Bowl,
}

#[derive(Debug, Describe, Serialize, Deserialize, Clone)]
/// The input of a serving action
struct ServingInput {
    /// what to serve. Value can be: Bowl.
    serveable: Serveable,
}

impl StateUpdater<InternalState, ServingOutput> for ServingInput {
    fn update(&self, state: &mut InternalState) -> Result<ServingOutput, ToolUseError> {
        match &self.serveable {
            Serveable::Bowl => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::ServeBowl)
                    .ok()
                    .flatten();
                Ok(x.into())
            }
        }
    }
}

/// The output of a serving action
#[derive(Debug, Serialize, Deserialize, Clone, Describe)]
struct ServingOutput {
    /// was it accepted? if not, maybe the bowl was not ready to be served.
    accepted: bool,
}

impl From<Option<CerealBowlRecipeOutput>> for ServingOutput {
    fn from(output: Option<CerealBowlRecipeOutput>) -> Self {
        match output {
            Some(CerealBowlRecipeOutput::Accepted) => ServingOutput { accepted: true },
            _ => ServingOutput { accepted: false },
        }
    }
}

/// Scenario 0
///
/// This scenario is a simple one. It is about making a bowl of cereal.
///
/// There are 3 tools: closet, mixing and serving.
/// The closet is where you can find the bowl, the cereal and the milk.
/// The mixing is where you can mix the cereal and the milk in the bowl.
/// The serving is where you can serve the bowl.
/// The goal is to make a bowl of cereal and serve it.
pub async fn build(mut toolbox: Toolbox) -> (Toolbox, Arc<Mutex<dyn tools::State>>) {
    let state = InternalState::new();
    let shared_state = Arc::new(Mutex::new(state));

    let closet: GenericTool<ClosetInput, InternalState, ClosetOutput> =
        GenericTool::new_with_descriptions(
            "Closet".to_string(),
            "Place where to find stuffs.".to_string(),
            shared_state.clone(),
        );

    let mixing: GenericTool<MixingInput, InternalState, MixingOutput> =
        GenericTool::new_with_descriptions(
            "Mixing".to_string(),
            "When you need to mix things in a container. You have to have them before using this tool.".to_string(),
            shared_state.clone(),
        );

    let serving: GenericTool<ServingInput, InternalState, ServingOutput> =
        GenericTool::new_with_descriptions(
            "Serving".to_string(),
            "When the meal is ready, use this to serve it.".to_string(),
            shared_state.clone(),
        );

    toolbox.add_tool(closet).await;
    toolbox.add_tool(mixing).await;
    toolbox.add_tool(serving).await;

    (toolbox, shared_state)
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    use sapiens::tools::{ProtoToolInvoke, ToolDescription};

    use super::*;
    use crate::tools::State;

    #[test]
    fn cereal_bowl() {
        let mut machine: StateMachine<CerealBowlRecipe> = StateMachine::new();
        println!("{:?}", machine.state());

        let _x = machine.consume(&CerealBowlRecipeInput::GetBowl).unwrap();
        println!("{:?}", machine.state());
        let _x = machine.consume(&CerealBowlRecipeInput::GetCereal).unwrap();
        println!("{:?}", machine.state());
        let _x = machine.consume(&CerealBowlRecipeInput::GetMilk).unwrap();
        let x = machine.state();
        println!("{:?}", x);

        let _x = machine
            .consume(&CerealBowlRecipeInput::AddCerealToBowl)
            .unwrap();
        println!("{:?}", machine.state());
        let _x = machine
            .consume(&CerealBowlRecipeInput::AddMilkToBowl)
            .unwrap();
        println!("{:?}", machine.state());
        let _x = machine.consume(&CerealBowlRecipeInput::ServeBowl).unwrap();
        println!("{:?}", machine.state());
    }

    #[tokio::test]
    async fn test_with_tools() {
        let shared_state = InternalState::default();

        let shared_state = Arc::new(Mutex::new(shared_state));

        let closet: GenericTool<ClosetInput, InternalState, ClosetOutput> =
            GenericTool::new_with_descriptions(
                "Closet".to_string(),
                "place where to find stuffs".to_string(),
                shared_state.clone(),
            );

        // first get the bowl
        let input = ClosetInput {
            get: ClosetObject::Bowl,
        };

        let input = serde_yaml::to_value(input).unwrap();
        let output = closet.invoke(input).await.unwrap();
        let output = serde_yaml::from_value::<ClosetOutput>(output).unwrap();
        println!("{:?}", output);
        {
            let guard = shared_state.lock().await;
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::BowlNoCerealNoMilk) {
                panic!("KO");
            }
        }

        // now get the cereal
        let input = ClosetInput {
            get: ClosetObject::Cereal,
        };

        let input = serde_yaml::to_value(input).unwrap();
        let output = closet.invoke(input).await.unwrap();
        let output = serde_yaml::from_value::<ClosetOutput>(output).unwrap();
        println!("{:?}", output);
        {
            let guard = shared_state.lock().await;
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::BowlCerealNoMilk) {
                panic!("KO");
            }
        }

        // now get the milk
        let input = ClosetInput {
            get: ClosetObject::Milk,
        };

        let input = serde_yaml::to_value(input).unwrap();
        let output = closet.invoke(input).await.unwrap();
        let output = serde_yaml::from_value::<ClosetOutput>(output).unwrap();
        println!("{:?}", output);
        {
            let guard = shared_state.lock().await;
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::BowlCerealMilk) {
                panic!("KO");
            }
        }

        // add a mixing tool
        let mixing: GenericTool<MixingInput, InternalState, MixingOutput> =
            GenericTool::new_with_descriptions(
                "Mixing".to_string(),
                "mixing stuffs".to_string(),
                shared_state.clone(),
            );

        // now mix the cereal
        let input = MixingInput {
            container: Container::Bowl,
            pourable: Pourable::Cereal,
        };

        let input = serde_yaml::to_value(input).unwrap();
        let output = mixing.invoke(input).await.unwrap();
        let output = serde_yaml::from_value::<MixingOutput>(output).unwrap();
        println!("{:?}", output);
        {
            let guard = shared_state.lock().await;
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::BowlWithCereal) {
                panic!("KO");
            }
        }

        // now mix the milk
        let input = MixingInput {
            container: Container::Bowl,
            pourable: Pourable::Milk,
        };

        let input = serde_yaml::to_value(input).unwrap();
        let output = mixing.invoke(input).await.unwrap();
        let output = serde_yaml::from_value::<MixingOutput>(output).unwrap();
        println!("{:?}", output);
        {
            let guard = shared_state.lock().await;
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::BowlWithCerealAndMilk) {
                panic!("KO");
            }
        }

        // add a serving tool
        let serving: GenericTool<ServingInput, InternalState, ServingOutput> =
            GenericTool::new_with_descriptions(
                "Serving".to_string(),
                "serving stuffs".to_string(),
                shared_state.clone(),
            );

        // now serve the bowl
        let input = ServingInput {
            serveable: Serveable::Bowl,
        };

        let input = serde_yaml::to_value(input).unwrap();
        let output = serving.invoke(input).await.unwrap();
        let output = serde_yaml::from_value::<ServingOutput>(output).unwrap();
        println!("{:?}", output);
        {
            let mut guard = shared_state.lock().await;
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::Served) {
                panic!("KO");
            }

            assert!(
                guard.has_reached_accepting_state(),
                "Should be in accepting state"
            );

            // reset the state
            guard.reset();

            let state = guard.state();
            assert!(
                matches!(state, CerealBowlRecipeState::NoBowlNoCerealNoMilk),
                "Should be in initial state"
            );
        }

        // and we are done!
    }

    #[tokio::test]
    async fn test_with_toolbox() {
        let toolbox = Toolbox::default();

        let (toolbox, shared_state) = build(toolbox).await;

        {
            let guard = shared_state.lock().await;
            assert!(!guard.has_reached_accepting_state());
        }

        let description = toolbox.describe().await;

        let mut description: Vec<ToolDescription> = description.into_values().collect();

        description.sort_by(|a, b| a.name.cmp(&b.name));

        assert_yaml_snapshot!(description);
    }
}
