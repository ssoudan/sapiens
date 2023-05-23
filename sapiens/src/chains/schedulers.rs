use super::*;
use crate::chains;

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
    chains::Error: From<E>,
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

/// Scheduler that schedules multiple agents in a fixed order
pub struct MultiAgentScheduler<E> {
    remaining_steps: usize,
    next_agent: usize,
    agents: Vec<Box<dyn Agent<Error = E>>>,
    #[allow(dead_code)]
    observer: WeakRuntimeObserver,
}

impl<E> MultiAgentScheduler<E> {
    /// Create a new scheduler with a maximum number of steps and a list of
    /// agents
    pub fn new(
        max_steps: usize,
        agents: Vec<Box<dyn Agent<Error = E>>>,
        observer: WeakRuntimeObserver,
    ) -> Self {
        Self {
            remaining_steps: max_steps,
            next_agent: 0,
            agents,
            observer,
        }
    }
}

#[async_trait::async_trait]
impl<E> Scheduler for MultiAgentScheduler<E>
where
    chains::Error: From<E>,
{
    async fn schedule(&mut self, context: &Context) -> Result<Message, Error> {
        if self.remaining_steps == 0 {
            return Err(Error::MaxStepsReached);
        }
        self.remaining_steps -= 1;

        if self.next_agent >= self.agents.len() {
            self.next_agent = 0;
        }

        let agent = self.agents.get_mut(self.next_agent).unwrap();
        self.next_agent += 1;

        let message = agent.act(context).await?;

        Ok(message)
    }
}
