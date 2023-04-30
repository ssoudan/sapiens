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
//! - The bowl contains the cereal
//! - The bowl contains the milk
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
    use rust_fsm::*;

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
            GetBowl => BowlNoCerealNoMilk,
            GetCereal => NoBowlCerealNoMilk,
            GetMilk => NoBowlNoCerealMilk,
        },

        BowlNoCerealNoMilk => {
            GetCereal => BowlCerealNoMilk,
            GetMilk => BowlNoCerealMilk,
            GetBowl => BowlNoCerealNoMilk, // TODO(ssoudan) should we allow this?
        },

        NoBowlCerealNoMilk => {
            GetBowl => BowlCerealNoMilk,
            GetMilk => NoBowlCerealMilk,
            GetCereal => NoBowlCerealNoMilk, // TODO(ssoudan) should we allow this?
        },

        NoBowlNoCerealMilk => {
            GetBowl => BowlNoCerealMilk,
            GetCereal => NoBowlNoCerealMilk,
            GetMilk => NoBowlNoCerealMilk, // TODO(ssoudan) should we allow this?
        },

        BowlCerealNoMilk => {
            GetBowl => BowlCerealNoMilk, // TODO(ssoudan) should we allow this?
            GetCereal => BowlCerealNoMilk, // TODO(ssoudan) should we allow this?
            GetMilk => BowlCerealMilk,
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
        println!("{:?}", machine.state());

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
}
