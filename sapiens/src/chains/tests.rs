use std::sync::Arc;

use indoc::indoc;
use serde_yaml::Value;
use tokio::sync::Mutex;

use super::*;
use crate::tools::{FieldFormat, Format, TerminalTool, Tool, ToolDescription};
use crate::void_observer;

struct SimpleAgent {}

impl From<()> for Error {
    fn from(_value: ()) -> Self {
        panic!("This should not happen")
    }
}

#[async_trait::async_trait]
impl Agent for SimpleAgent {
    type Error = ();

    async fn act(&self, _context: &Context) -> Result<Message, ()> {
        Ok(Message::Observation {
            content: "Hello".to_string(),
            usage: None,
        })
    }
}

#[derive(Default)]
struct ConcludeTool {
    done: Mutex<Option<String>>,
}

#[async_trait::async_trait]
impl Tool for ConcludeTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ConcludeTool".to_string(),
            description: "A tool to conclude a task".to_string(),
            parameters: Format {
                fields: vec![FieldFormat {
                    name: "conclusion".to_string(),
                    r#type: "str".to_string(),
                    optional: false,
                    description: "The conclusion".to_string(),
                }],
            },
            responses_content: Format::default(),
        }
    }

    async fn invoke(&self, _input: Value) -> Result<Value, ToolUseError> {
        let mut done = self.done.lock().await;

        if done.is_some() {
            return Err(ToolUseError::InvocationFailed(
                "This task is already done.".to_string(),
            ));
        }

        // set done
        *done = Some("Done".to_string());

        Ok(Value::Null)
    }
}

#[async_trait::async_trait]
impl TerminalTool for ConcludeTool {
    async fn is_done(&self) -> bool {
        let done = self.done.lock().await;
        done.is_some()
    }

    async fn take_done(&self) -> Option<TerminationMessage> {
        {
            let mut done = self.done.lock().await;
            done.take().map(|input| TerminationMessage {
                conclusion: input,
                original_question: "tbd".to_string(),
            })
        }
    }
}

#[tokio::test]
async fn observes_too_much() {
    let toolbox = {
        let mut toolbox = Toolbox::default();
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        toolbox
    };
    let observer = void_observer();
    let observer = Arc::downgrade(&observer);

    let scheduler = Box::new(schedulers::SingleAgentScheduler::new(
        10,
        Box::new(SimpleAgent {}),
        observer.clone(),
    ));
    let mut runtime = Runtime::new(toolbox, scheduler, observer).await.unwrap();

    let terminal_state = runtime.run().await;

    runtime
        .context
        .messages
        .iter()
        .for_each(|m| println!("{:?}", m));

    assert!(terminal_state.is_err());
}

struct NoAsSimpleAgent {}

#[async_trait::async_trait]
impl Agent for NoAsSimpleAgent {
    type Error = ();

    async fn act(&self, context: &Context) -> Result<Message, ()> {
        if context.messages.is_empty() {
            Ok(Message::Observation {
                content: "Hello".to_string(),
                usage: None,
            })
        } else {
            Ok(Message::Action {
                content: indoc! {r#"
                ```yaml
                tool_name: ConcludeTool
                parameters:
                    conclusion: "Done"
                ```
                "#
                }
                .to_string(),
                usage: None,
            })
        }
    }
}

#[tokio::test]
async fn observes_and_conclude() {
    let toolbox = {
        let mut toolbox = Toolbox::default();
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        toolbox
    };

    let observer = void_observer();
    let observer = Arc::downgrade(&observer);

    let scheduler = Box::new(schedulers::SingleAgentScheduler::new(
        10,
        Box::new(NoAsSimpleAgent {}),
        observer.clone(),
    ));
    let mut runtime = Runtime::new(toolbox, scheduler, observer).await.unwrap();

    let terminal_state = runtime.run().await;

    runtime
        .context
        .messages
        .iter()
        .for_each(|m| println!("{:?}", m));

    assert!(terminal_state.is_ok());

    let terminal_state = terminal_state.unwrap();
    assert_eq!(terminal_state.messages.len(), 1);

    let message = &terminal_state.messages[0];
    assert_eq!(message.conclusion, "Done");
}
