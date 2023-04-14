//! Botrs library

/// Tools
pub mod tools;

use std::rc::Rc;

use colored::Colorize;
use llm_chain::tools::ToolCollection;
use llm_chain::traits::StepExt;
use llm_chain::{Parameters, PromptTemplate};
use llm_chain_openai::chatgpt::{
    ChatPromptTemplate, Executor, MessagePromptTemplate, Model, Role, Step,
};

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
- Template expansion is not supported in Action. If you need to pass information from on action to another, you have to pass them manually.
- Use ConcludeTool with your conclusion (as text) once you have the final answer to the original question.
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
Original question: Sort [2, 3, 1, 4, 5]
Observations, Orientation, Decision, The ONLY Action?
";

const PROTO_EXCHANGE_4: &str = r"
# Observations:
- We needed to sort the list in ascending order.
- We have the result of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
# Orientation:
- I know the answer to the original question.
# Decision:
- Use the ConcludeTool to terminate the task with the sorted list.
# The ONLY Action:
```yaml
command: ConcludeTool
input:
  conclusion: |
    The ascending sorted list is [1, 2, 3, 4, 5].
```
";

fn create_tool_description(tc: &ToolCollection) -> String {
    let prefix = TOOL_PREFIX.to_string();
    let tool_desc = tc.describe();
    prefix + &tool_desc
}

fn create_tool_warm_up_template(tc: &ToolCollection) -> PromptTemplate {
    let prefix = PREFIX.to_string();
    let tool_prompt = create_tool_description(tc);
    (prefix + FORMAT + &tool_prompt).into()
}

fn recurring_prompt(task: &str) -> String {
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

    let warm_up_template = create_tool_warm_up_template(&tool_collection);
    let warm_up_prompt = warm_up_template.format(&Parameters::new());

    let exec = Executor::new_default();

    // build the warm up exchange with the user
    let exchange = [
        (Role::User, PROTO_EXCHANGE_1),
        (Role::Assistant, PROTO_EXCHANGE_2),
        (Role::User, PROTO_EXCHANGE_3),
        (Role::Assistant, PROTO_EXCHANGE_4),
    ];

    let system_prompt = create_system_prompt();
    let mut chat = ChatPromptTemplate::new(vec![
        (Role::System, system_prompt).into(),
        (Role::User, &warm_up_prompt).into(),
        (Role::Assistant, "Understood.").into(),
    ]);

    for (role, message) in exchange.into_iter() {
        chat.add(MessagePromptTemplate::new(
            role,
            PromptTemplate::static_string(message.to_string()),
        ));
    }

    let recurring_prompt = recurring_prompt(task);

    // Now we are ready to start the task
    chat.add(MessagePromptTemplate::new(
        Role::User,
        PromptTemplate::static_string(recurring_prompt.clone()),
    ));

    // Let's print the chat history so far - yellow for the system, green for the
    // user, blue for the assistant
    for message in chat.format(&Parameters::new()).iter() {
        match message.role {
            Role::System => println!("{}", message.content.yellow()),
            Role::User => println!("{}", message.content.green()),
            Role::Assistant => println!("{}", message.content.blue()),
        }
        println!("=============")
    }

    let tool_desc = create_tool_description(&tool_collection);

    for _ in 1..max_steps {
        let chain = Step::new(Model::ChatGPT3_5Turbo, chat.clone()).to_chain();
        let res = chain.run(Parameters::new(), &exec).await.unwrap();
        // dbg!(&res);

        let message_text = res.choices.first().unwrap().message.content.clone();

        println!("{}", message_text.green());
        println!("=============");

        chat.add(MessagePromptTemplate::new(
            Role::Assistant,
            PromptTemplate::static_string(message_text.clone()),
        ));

        let resp = tool_collection.process_chat_input(&message_text);
        match resp {
            Ok(x) => {
                let content = format!("# Action result: \n```yaml\n{}```\n{}", x, recurring_prompt);

                println!("{}", content.blue());

                let msg = MessagePromptTemplate::new(
                    Role::User,
                    PromptTemplate::static_string(content.clone()),
                );

                chat.add(msg);
            }
            Err(e) => {
                let content = format!(
                    "# Failed with:\n{}\n{}\nWhat was incorrect in previous response?\n{}",
                    e, tool_desc, recurring_prompt
                );
                println!("{}", content.red());

                chat.add(MessagePromptTemplate::new(
                    Role::User,
                    PromptTemplate::static_string(content.clone()),
                ));
            }
        }
        println!("=============");
    }
}
