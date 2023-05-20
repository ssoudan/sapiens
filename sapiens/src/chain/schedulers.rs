use super::*;
use crate::chain;

/// A simple scheduler that can be used to schedule agents
///
/// It has only one agent and a maximum number of steps.
/// Since it has only one agent, it will always schedule the same agent.
pub struct SingleAgentScheduler<E> {
    remaining_steps: usize,
    agent: Box<dyn Agent<Error = E>>,
    #[allow(dead_code)]
    observer: WeakRuntimeObserver,
}

impl<E> SingleAgentScheduler<E> {
    /// Create a new scheduler with a maximum number of steps and an agent
    pub fn new(
        max_steps: usize,
        agent: Box<dyn Agent<Error = E>>,
        observer: WeakRuntimeObserver,
    ) -> Self {
        Self {
            remaining_steps: max_steps,
            agent,
            observer,
        }
    }
}

#[async_trait::async_trait]
impl<E> Scheduler for SingleAgentScheduler<E>
where
    chain::Error: From<E>,
{
    async fn schedule(&mut self, context: &Context) -> Result<Message, Error> {
        if self.remaining_steps == 0 {
            return Err(Error::MaxStepsReached);
        }
        self.remaining_steps -= 1;

        let agent = self.agent.as_ref();

        let message = agent.act(context).await?;

        Ok(message)
    }
}
