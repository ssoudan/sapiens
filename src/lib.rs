//! Botrs library

/// Tools
pub mod tools;

use llm_chain::traits::StepExt;
use llm_chain::{Parameters, PromptTemplate};
use llm_chain_openai::chatgpt::{
    ChatPromptTemplate, Executor, MessagePromptTemplate, Model, Role, Step,
};
use llm_chain_tools::tools::ExitTool;
use llm_chain_tools::{create_tool_prompt_segment, ToolCollection};

use crate::tools::bash::BashTool;
use crate::tools::hue::room::RoomTool;

pub async fn something_with_rooms(bridge: huelib::bridge::Bridge) {
    let tool_collection = ToolCollection::new(vec![
        Box::new(RoomTool::new(bridge)),
        Box::new(ExitTool::new()),
    ]);
    let template =
        create_tool_prompt_segment(&tool_collection, "Please perform the following task: {}");
    let task = "List all the lights in the kitchen.";
    let prompt = template.format(&Parameters::new_with_text(task));

    println!("Prompt: {}", prompt);
    let exec = Executor::new_default();

    let mut chat = ChatPromptTemplate::new(vec![
        (
            Role::System,
            "You are an automated agent for performing tasks. Your output must always be YAML.",
        )
            .into(),
        (Role::User, &prompt).into(),
    ]);
    for _ in 1..5 {
        let chain = Step::new(Model::ChatGPT3_5Turbo, chat.clone()).to_chain();
        let res = chain.run(Parameters::new(), &exec).await.unwrap();
        let message_text = res.choices.first().unwrap().message.content.clone();
        println!("Assistant: {}", message_text);
        println!("=============");
        chat.add(MessagePromptTemplate::new(
            Role::Assistant,
            message_text.clone().into(),
        ));
        let resp = tool_collection.process_chat_input(&message_text);
        match resp {
            Ok(x) => {
                chat.add(MessagePromptTemplate::new(
                    Role::User,
                    format!("```yaml\n{}```\nProceed with your next command.", x).into(),
                ));
                println!("LLMCHAIN: {}\n", x)
            }
            Err(e) => {
                let pt = template.format(&Parameters::new_with_text(format!(
                    "Correct your output and perform the task - {}. Your task was: {}",
                    e, task
                )));
                let pt: PromptTemplate = pt.into();
                chat.add(MessagePromptTemplate::new(Role::User, pt));
                println!("Error: {}", e)
            }
        }
    }
}

pub async fn something_with_bash() {
    let tool_collection =
        ToolCollection::new(vec![Box::new(BashTool::new()), Box::new(ExitTool::new())]);
    let template =
        create_tool_prompt_segment(&tool_collection, "Please perform the following task: {}");
    let task = "Figure out my IP address";
    let prompt = template.format(&Parameters::new_with_text(task));

    println!("Prompt: {}", prompt);
    let exec = Executor::new_default();

    let mut chat = ChatPromptTemplate::new(vec![
        (
            Role::System,
            "You are an automated agent for performing tasks. Your output must always be YAML.",
        )
            .into(),
        (Role::User, &prompt).into(),
    ]);
    for _ in 1..5 {
        let chain = Step::new(Model::ChatGPT3_5Turbo, chat.clone()).to_chain();
        let res = chain.run(Parameters::new(), &exec).await.unwrap();
        let message_text = res.choices.first().unwrap().message.content.clone();
        println!("Assistant: {}", message_text);
        println!("=============");
        chat.add(MessagePromptTemplate::new(
            Role::Assistant,
            message_text.clone().into(),
        ));
        let resp = tool_collection.process_chat_input(&message_text);
        match resp {
            Ok(x) => {
                chat.add(MessagePromptTemplate::new(
                    Role::User,
                    format!("```yaml\n{}```\nProceed with your next command.", x).into(),
                ));
                println!("LLMCHAIN: {}\n", x)
            }
            Err(e) => {
                let pt = template.format(&Parameters::new_with_text(format!(
                    "Correct your output and perform the task - {}. Your task was: {}",
                    e, task
                )));
                let pt: PromptTemplate = pt.into();
                chat.add(MessagePromptTemplate::new(Role::User, pt));
                println!("Error: {}", e)
            }
        }
    }
}
