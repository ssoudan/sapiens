//! Botrs library

/// Tools
pub mod tools;

pub(crate) mod context;

use std::rc::Rc;

use async_openai::types::Role;
use colored::Colorize;
use context::ChatHistory;
use llm_chain::tools::ToolCollection;
use llm_chain::traits::StepExt;
use llm_chain::Parameters;
use llm_chain_openai::chatgpt::{Executor, Model, Step};

use crate::tools::conclude::ConcludeTool;
use crate::tools::hue::room::RoomTool;
use crate::tools::hue::status::StatusTool;
use crate::tools::python::PythonTool;

fn create_system_prompt() -> String {
    "You are an automated agent named botGPT interacting with the WORLD".to_string()
}

const PREFIX: &str = r"You are botGPT, large language model assisting the WORLD. Use available tools to answer the question as best as you can.
You will proceed in a OODA loop made of the following steps:
- Observations: What do you know? What is your source? What don't you know? You might want to note down important information for later like relevant past Action results. 
- Orientation: Plan the intermediate objectives along the path to answer the original question. Make a list of current objectives. 
- Decision: Choose what to do first to answer the question. Why? What are the pros and cons of this decision?
- Action: Take a single Action consisting of exactly one tool invocation. The available Tools listed below. Use ConcludeTool when you have the final answer to the original question.

# Notes: 
- Use ConcludeTool with your conclusion (as text) once you have the final answer to the original question.
- Template expansion is not supported in Action. If you need to pass information from on action to another, you have to pass them manually.
- There are no APIs available to you. You have to use the Tools.
";

const TOOL_PREFIX: &str = r"
# The following are the ONLY Tools you can use for your Actions:
";

const FORMAT: &str = r"
# Format of your response

Please use the following format for your response - no need to be verbose:
====================
## Observations:
- ...
## Orientation:
- ...
## Decision:
- ...
## The ONLY Action: <Do not give multiple command. Only one per response.>
```yaml
command: <ToolName>
input:
  <... using the `input_format` for the Tool ...>
```
====================
";

const PROTO_EXCHANGE_1: &str = r"
## Original question: Sort in ascending order: [2, 3, 1, 4, 5]. 
";

const PROTO_EXCHANGE_2: &str = r#"
## Observations:
- The given list to sort is [2, 3, 1, 4, 5].
- I need to sort this list in ascending order.
## Orientation:
- SandboxedPythonTool can be used to sort the list.
## Decision:
- We can use the sorted() function of Python to sort the list.
## The ONLY Action:
```yaml
command: SandboxedPythonTool
input:
  code: |
    lst = [2, 3, 1, 4, 5]
    sorted_list = sorted(lst)
    print(f"The sorted list is {sorted_list}")
```
"#;

const PROTO_EXCHANGE_3: &str = r"
# Action result:
```yaml
status: 0
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
# Your turn
Original question: Sort the following list: [2, 3, 1, 4, 5]
Observations, Orientation, Decision, The ONLY Action?
";

const PROTO_EXCHANGE_4: &str = r"
## Observations:
- We needed to sort the list in ascending order.
- We have the result of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
## Orientation:
- I know the answer to the original question.
## Decision:
- Use the ConcludeTool to terminate the task with the sorted list.
## The ONLY Action:
```yaml
command: ConcludeTool
input:
  original_question: |
    Sort in ascending order: [2, 3, 1, 4, 5]
  conclusion: |
    The ascending sorted list is [1, 2, 3, 4, 5].
```
";

fn create_tool_description(tc: &ToolCollection) -> String {
    let prefix = TOOL_PREFIX.to_string();
    let tool_desc = tc.describe();
    prefix + &tool_desc
}

fn create_tool_warm_up(tc: &ToolCollection) -> String {
    let prefix = PREFIX.to_string();
    let tool_prompt = create_tool_description(tc);
    prefix + FORMAT + &tool_prompt
}

fn build_task_prompt(task: &str) -> String {
    format!(
        "# Your turn\nOriginal question: {}\nObservations, Orientation, Decision, The ONLY Action?",
        task
    )
}

/// Run a task with a set of tools
pub async fn something_with_rooms(
    bridge: Rc<huelib::bridge::Bridge>,
    task: &str,
    max_steps: usize,
) {
    let mut tool_collection = ToolCollection::new();

    tool_collection.add_tool(RoomTool::new(bridge.clone()));
    tool_collection.add_tool(ConcludeTool::new());
    tool_collection.add_tool(PythonTool::new());
    tool_collection.add_tool(StatusTool::new(bridge));

    let warm_up_prompt = create_tool_warm_up(&tool_collection);
    let system_prompt = create_system_prompt();

    let exec = Executor::new_default();

    // build the warm-up exchange with the user
    let prompt = [
        (Role::System, system_prompt),
        (Role::User, warm_up_prompt),
        (Role::Assistant, "Understood.".to_string()),
        (Role::User, PROTO_EXCHANGE_1.to_string()),
        (Role::Assistant, PROTO_EXCHANGE_2.to_string()),
        (Role::User, PROTO_EXCHANGE_3.to_string()),
        (Role::Assistant, PROTO_EXCHANGE_4.to_string()),
    ];

    let mut chat_history = ChatHistory::new(Model::ChatGPT3_5Turbo.to_string(), 256);

    chat_history.add_prompts(&prompt);

    // Now we are ready to start the task
    let task_prompt = build_task_prompt(task);

    chat_history
        .add_chitchat(Role::User, task_prompt.to_string())
        .expect("The task prompt is too long for the model");

    // Let's print the chat history so far - yellow for the system, green for the
    // user, blue for the assistant
    for message in chat_history.iter() {
        match message.role {
            Role::System => println!("{}", message.content.yellow()),
            Role::User => println!("{}", message.content.green()),
            Role::Assistant => println!("{}", message.content.blue()),
        }
        println!("=============")
    }

    // Build a tool description to inject it into the chat on error
    let tool_desc = create_tool_description(&tool_collection);

    for _ in 1..max_steps {
        let chain = Step::new(Model::ChatGPT3_5Turbo, &chat_history).to_chain();
        let res = chain.run(Parameters::new(), &exec).await.unwrap();
        // dbg!(&res);

        let message_text = res.choices.first().unwrap().message.content.clone();

        println!("{}", message_text.blue());

        let l = chat_history
            .add_chitchat(Role::Assistant, message_text.clone())
            .expect("The assistant response is too long for the model");
        println!(
            "============= {:>3} messages in the chat history =============",
            l
        );

        let resp = tool_collection.process_chat_input(&message_text);
        let l = match resp {
            Ok(x) => {
                let content = format!("# Action result: \n```yaml\n{}```\n{}", x, task_prompt);

                println!("{}", content.green());

                chat_history
                    .add_chitchat(Role::User, content.clone())
                    .expect("The user response is too long for the model")
            }
            Err(e) => {
                let content = format!(
                    "# Failed with:\n{}\n{}\nWhat was incorrect in previous response?\n{}",
                    e, tool_desc, task_prompt
                );
                println!("{}", content.red());

                chat_history
                    .add_chitchat(Role::User, content.clone())
                    .expect("The user response is too long for the model")
            }
        };
        println!(
            "============= {:>3} messages in the chat history =============",
            l
        );
    }
}
