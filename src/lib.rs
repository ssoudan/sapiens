//! Botrs library

/// Tools
pub mod tools;

use colored::Colorize;
use llm_chain::tools::ToolCollection;
use llm_chain::traits::StepExt;
use llm_chain::{Parameters, PromptTemplate};
use llm_chain_openai::chatgpt::{
    ChatPromptTemplate, Executor, MessagePromptTemplate, Model, Role, Step,
};

use crate::tools::conclude::ConcludeTool;
use crate::tools::hue::room::RoomTool;
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

// const SOMETHING: &str = r"
// Please use the following format for your response - no need to be verbose:
// ====================
// Observations:
// - ...
// Orientation:
// - ...
// Decision:
// - ...
// The only Action: Do not give multiple command. Only one per response.
// ```yaml
// command: <ToolName>
// input:
//   <... using the `input_format` for the Tool ...>
// ```
// ====================
// ";

// TODO(ssoudan) use the chat history for the 'illustrative' exchange

const FORMAT: &str = r"
# The exchange between you (botGPT) and the WORLD will look like this - Note YAML is only used for the Action:
--------
[WORLD]: Question: ...
[botGPT]: 
## Observations:
- ...
## Orientation:
- ...
## Decision:
- ...
## The ONLY Action: 
```yaml
command: <ToolName>
input:
  <... using the `input_format` for the Tool ...>
```
[WORLD]: Action result: 
```yaml
...
```
Original question: ...
Observations, Orientation, Decision, The Action?
[botGPT]: 
## Observations:
- ...
## Orientation:
- ...
## Decision:
- ...
## The ONLY Action: 
```yaml
command: <ToolName>
input:
  <... using the `input_format` for the Tool ...>
```
--------

Your response only needs to include the Observations, Orientation, Decision and Action. The rest will be filled in automatically.
";

fn create_tool_description(tc: &ToolCollection) -> String {
    let prefix = TOOL_PREFIX.to_string();
    let tool_desc = tc.describe();
    prefix + &tool_desc
}

fn create_tool_prompt_segment(tc: &ToolCollection, prompt: &str) -> PromptTemplate {
    let prefix = PREFIX.to_string();
    let tool_prompt = create_tool_description(tc);
    (prefix + FORMAT + &tool_prompt + "\n---\n" + prompt).into()
}

fn recurring_prompt(task: &str) -> String {
    format!(
        "# Your turn\nOriginal question: {}\nObservations, Orientation, Decision, The ONLY Action?",
        task
    )
}

/// Run a task with a set of tools
pub async fn something_with_rooms(bridge: huelib::bridge::Bridge, task: &str, max_steps: usize) {
    let mut tool_collection = ToolCollection::new();

    tool_collection.add_tool(RoomTool::new(bridge));
    tool_collection.add_tool(ConcludeTool::new());
    tool_collection.add_tool(PythonTool::new());

    let prompt = recurring_prompt(task);
    let template = create_tool_prompt_segment(&tool_collection, &prompt);

    let prompt = template.format(&Parameters::new());

    println!("{}", prompt.blue());
    let exec = Executor::new_default();

    let system_prompt = create_system_prompt();
    let mut chat = ChatPromptTemplate::new(vec![
        (Role::System, system_prompt).into(),
        (Role::User, &prompt).into(),
    ]);

    let recurring_prompt = recurring_prompt(task);
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
