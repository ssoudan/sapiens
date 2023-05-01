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

// # Scenario 0
// pub fn build() {
//     // TODO(ssoudan) implement
// }

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use rust_fsm::*;
    use sapiens::tools::{Describe, ProtoToolInvoke};
    use sapiens_derive::Describe;
    use serde::{Deserialize, Serialize};

    use crate::tools::{GenericTool, StateUpdater};

    // state_machine! {
    //     derive(Debug)
    //     repr_c(true)
    //     CircuitBreaker(Closed)
    //
    //     Closed(Unsuccessful) => Open [SetupTimer],
    //     Open(TimerTriggered) => HalfOpen,
    //     HalfOpen => {
    //         Successful => Closed,
    //         Unsuccessful => Open [SetupTimer],
    //     }
    // }
    //
    // #[test]
    // fn something() {
    //     let mut machine: StateMachine<CircuitBreaker> = StateMachine::new();
    //
    //     let x = machine.consume(&CircuitBreakerInput::Unsuccessful).unwrap();
    //
    //     if let Some(CircuitBreakerOutput::SetupTimer) = x {
    //         println!("SetupTimer");
    //         return;
    //     }
    //     panic!("Unexpected");
    // }

    state_machine! {
        derive(Debug)
        repr_c(false)
        CerealBowlRecipe(NoBowlNoCerealNoMilk)

        // TODO(ssoudan) emit reward?

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
            AddCerealToBowl => BowlWithCereal,
            AddMilkToBowl => BowlWithMilk,
        },
        BowlWithCereal(AddMilkToBowl) => BowlWithCerealAndMilk,
        BowlWithMilk(AddCerealToBowl) => BowlWithCerealAndMilk,

        // serving
        BowlWithCerealAndMilk(ServeBowl) => Served,
    }

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

    struct State {
        fsm: StateMachine<CerealBowlRecipe>,
    }

    impl State {
        fn new() -> Self {
            Self {
                fsm: StateMachine::new(),
            }
        }

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
        /// what to get: bowl, cereal, milk
        get: ClosetObject,
    }

    impl StateUpdater<State, ClosetOutput> for ClosetInput {
        fn update(&self, state: &mut State) -> ClosetOutput {
            match &self.get {
                ClosetObject::Bowl => {
                    let x = state.fsm.consume(&CerealBowlRecipeInput::GetBowl).unwrap();
                    x.into()
                }
                ClosetObject::Cereal => {
                    let x = state
                        .fsm
                        .consume(&CerealBowlRecipeInput::GetCereal)
                        .unwrap();
                    x.into()
                }
                ClosetObject::Milk => {
                    let x = state.fsm.consume(&CerealBowlRecipeInput::GetMilk).unwrap();
                    x.into()
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
                None => ClosetOutput { found: false },
                Some(CerealBowlRecipeOutput::Found) => ClosetOutput { found: true },
            }
        }
    }

    #[tokio::test]
    async fn test_with_tool() {
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
            let guard = shared_state.lock().unwrap();
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
            let guard = shared_state.lock().unwrap();
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
            let guard = shared_state.lock().unwrap();
            let state = guard.state();
            if !matches!(state, CerealBowlRecipeState::BowlCerealMilk) {
                panic!("KO");
            }
        }
    }
}
