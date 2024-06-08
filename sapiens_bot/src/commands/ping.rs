use serenity::all::{CommandDataOption, CreateCommand};

pub(crate) fn run(_options: &[CommandDataOption]) -> String {
    "Hey, I'm alive!".to_string()
}

pub(crate) fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}
