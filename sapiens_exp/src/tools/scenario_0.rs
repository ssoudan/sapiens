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
//! State machine:
//! - (no_bowl,no_cereal,no_milk)
//! - (no_bowl,cereal,no_milk)
//! - (no_bowl,cereal,milk)
//! - (bowl,no_cereal,milk)
//! - (bowl,cereal,no_milk)
//! - (bowl,no_cereal,no_milk)
//! - (bowl,cereal,no_milk)
//! - (bowl,cereal,milk)
//! - (served)

use std::sync::Arc;

use rust_fsm::*;
use sapiens::tools::toolbox::Toolbox;
use sapiens::tools::{Describe, ToolUseError};
use sapiens_derive::Describe;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::tools::{GenericTool, StateUpdater};

state_machine! {
    derive(Debug)
    repr_c(false)
    CerealBowlRecipe(NoBowlNoCerealNoMilk)

    // procurement
    NoBowlNoCerealNoMilk => {
        GetBowl => BowlNoCerealNoMilk [Found],
        GetCereal => NoBowlCerealNoMilk [Found],
        GetMilk => NoBowlNoCerealMilk [Found],
    },

    BowlNoCerealNoMilk => {
        GetCereal => BowlCerealNoMilk [Found],
        GetMilk => BowlNoCerealMilk [Found],
        GetBowl => BowlNoCerealNoMilk,
    },

    NoBowlCerealNoMilk => {
        GetBowl => BowlCerealNoMilk [Found],
        GetMilk => NoBowlCerealMilk [Found],
        GetCereal => NoBowlCerealNoMilk,
    },

    NoBowlNoCerealMilk => {
        GetBowl => BowlNoCerealMilk [Found],
        GetCereal => NoBowlNoCerealMilk [Found],
        GetMilk => NoBowlNoCerealMilk,
    },

    BowlCerealNoMilk => {
        GetBowl => BowlCerealNoMilk,
        GetCereal => BowlCerealNoMilk,
        GetMilk => BowlCerealMilk [Found],
    },

    BowlNoCerealMilk => {
        GetBowl => BowlNoCerealMilk,
        GetCereal => BowlCerealMilk [Found],
        GetMilk => BowlNoCerealMilk,
    },

    NoBowlCerealMilk => {
        GetBowl => BowlCerealMilk [Found],
        GetCereal => NoBowlCerealMilk,
        GetMilk => NoBowlCerealMilk,
    },

    // mixing
    BowlCerealMilk => {
        AddCerealToBowl => BowlWithCereal [Success],
        AddMilkToBowl => BowlWithMilk [Success],
    },
    BowlWithCereal(AddMilkToBowl) => BowlWithCerealAndMilk [Success],
    BowlWithMilk(AddCerealToBowl) => BowlWithCerealAndMilk [Success],

    // serving
    BowlWithCerealAndMilk(ServeBowl) => Served [Success],
}

/// The state of the scenario 0
pub struct State {
    fsm: StateMachine<CerealBowlRecipe>,
}

impl State {
    fn new() -> Self {
        Self {
            fsm: StateMachine::new(),
        }
    }

    /// is the scenario finished?
    pub fn has_reached_accepting_state(&self) -> bool {
        matches!(self.fsm.state(), CerealBowlRecipeState::Served)
    }

    /// Reset the state machine
    pub fn reset(&mut self) {
        self.fsm = StateMachine::new();
    }

    /// Get the current state
    #[allow(dead_code)]
    fn state(&self) -> &CerealBowlRecipeState {
        self.fsm.state()
    }
}

impl Default for State {
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
    /// what to get: Bowl, Cereal, Milk
    get: ClosetObject,
}

impl StateUpdater<State, ClosetOutput> for ClosetInput {
    fn update(&self, state: &mut State) -> Result<ClosetOutput, ToolUseError> {
        match &self.get {
            ClosetObject::Bowl => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::GetBowl)
                    .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?;
                Ok(x.into())
            }
            ClosetObject::Cereal => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::GetCereal)
                    .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?;
                Ok(x.into())
            }
            ClosetObject::Milk => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::GetMilk)
                    .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?;
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
}

impl From<Option<CerealBowlRecipeOutput>> for ClosetOutput {
    fn from(output: Option<CerealBowlRecipeOutput>) -> Self {
        match output {
            Some(CerealBowlRecipeOutput::Found) => ClosetOutput { found: true },
            _ => ClosetOutput { found: false },
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
/// The input of a mixing action
struct MixingInput {
    /// what to pour in. Value can be: Bowl.
    container: Container,
    /// what to pour. Value can be: Cereal or Milk.
    pourable: Pourable,
}

impl StateUpdater<State, MixingOutput> for MixingInput {
    fn update(&self, state: &mut State) -> Result<MixingOutput, ToolUseError> {
        match (&self.container, &self.pourable) {
            (Container::Bowl, Pourable::Cereal) => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::AddCerealToBowl)
                    .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?;
                Ok(x.into())
            }
            (Container::Bowl, Pourable::Milk) => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::AddMilkToBowl)
                    .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?;
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
}

impl From<Option<CerealBowlRecipeOutput>> for MixingOutput {
    fn from(output: Option<CerealBowlRecipeOutput>) -> Self {
        match output {
            Some(CerealBowlRecipeOutput::Success) => MixingOutput { added: true },
            _ => MixingOutput { added: false },
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
    /// what to serve. Value can be: bowl.
    serveable: Serveable,
}

impl StateUpdater<State, ServingOutput> for ServingInput {
    fn update(&self, state: &mut State) -> Result<ServingOutput, ToolUseError> {
        match &self.serveable {
            Serveable::Bowl => {
                let x = state
                    .fsm
                    .consume(&CerealBowlRecipeInput::ServeBowl)
                    .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?;
                Ok(x.into())
            }
        }
    }
}

/// The output of a serving action
#[derive(Debug, Serialize, Deserialize, Clone, Describe)]
struct ServingOutput {
    /// was it served?
    served: bool,
}

impl From<Option<CerealBowlRecipeOutput>> for ServingOutput {
    fn from(output: Option<CerealBowlRecipeOutput>) -> Self {
        match output {
            Some(CerealBowlRecipeOutput::Success) => ServingOutput { served: true },
            _ => ServingOutput { served: false },
        }
    }
}

/// Scenario 0
pub async fn build(mut toolbox: Toolbox) -> (Toolbox, Arc<Mutex<State>>) {
    let state = State::new();
    let shared_state = Arc::new(Mutex::new(state));

    let closet: GenericTool<ClosetInput, State, ClosetOutput> = GenericTool::new_with_descriptions(
        "closet".to_string(),
        "place where to find stuffs".to_string(),
        shared_state.clone(),
    );

    let mixing: GenericTool<MixingInput, State, MixingOutput> = GenericTool::new_with_descriptions(
        "mixing".to_string(),
        "when you need to mix things in a container".to_string(),
        shared_state.clone(),
    );

    let serving: GenericTool<ServingInput, State, ServingOutput> =
        GenericTool::new_with_descriptions(
            "serving".to_string(),
            "when the meal is ready to be served".to_string(),
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
        let shared_state = State::default();

        let shared_state = Arc::new(Mutex::new(shared_state));

        let closet: GenericTool<ClosetInput, State, ClosetOutput> =
            GenericTool::new_with_descriptions(
                "closet".to_string(),
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
        let mixing: GenericTool<MixingInput, State, MixingOutput> =
            GenericTool::new_with_descriptions(
                "mixing".to_string(),
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
        let serving: GenericTool<ServingInput, State, ServingOutput> =
            GenericTool::new_with_descriptions(
                "serving".to_string(),
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
            let state = guard.state();
            assert!(matches!(state, CerealBowlRecipeState::NoBowlNoCerealNoMilk));
        }

        let description = toolbox.describe().await;

        let mut description: Vec<ToolDescription> = description.into_values().collect();

        description.sort_by(|a, b| a.name.cmp(&b.name));

        assert_yaml_snapshot!(description);
    }
}
