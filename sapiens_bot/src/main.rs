//! Discord bot for the Sapiens.
mod commands;
mod runner;

use std::env;

use dotenvy::dotenv_override;
use pyo3::PyResult;
use serenity::async_trait;
use serenity::futures::channel::mpsc;
use serenity::futures::{SinkExt, StreamExt};
use serenity::http::CacheHttp;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use tokio::spawn;
use tracing::info;

use crate::runner::{JobUpdate, NewJob};

struct Handler {
    guild_id: GuildId,
    tx: RwLock<mpsc::Sender<NewJob>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            info!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                info!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        let channel = new_message.channel_id.to_channel(&ctx.http).await.unwrap();

        info!("Received a message on {}: {:#?}", channel, new_message);

        // if message is not from me
        if new_message.author.id == ctx.cache().unwrap().current_user_id() {
            info!("Message is from me, ignoring");
            return;
        }

        if new_message.content.starts_with("DO: ") {
            let max_steps = 12;
            let task = new_message.content[4..].to_string();

            // TODO(ssoudan) option to hide the warmup prompts
            // TODO(ssoudan) ask to continue after max_steps

            if task.is_empty() {
                info!("Empty task, ignoring");

                new_message
                    .channel_id
                    .say(&ctx.http, "Please provide a task: 'DO: <task>'")
                    .await
                    .unwrap();

                return;
            }

            let (tx, mut rx) = mpsc::channel::<JobUpdate>(20);

            // Send the job to the runner
            self.tx
                .write()
                .await
                .send(NewJob::new(task, max_steps, tx))
                .await
                .unwrap();

            // create a thread to display the job updates
            let thread = new_message
                .channel_id
                .create_private_thread(&ctx.http, |thread| {
                    thread.name("test").auto_archive_duration(1440)
                })
                .await
                .unwrap();

            info!("Created thread: {:#?}", thread);

            thread.id.join_thread(&ctx.http).await.unwrap();
            info!("Joined thread: {:#?}", thread);

            // add the user who called the command to the thread
            thread
                .id
                .add_thread_member(&ctx.http, new_message.author.id)
                .await
                .unwrap();

            info!("Added member to thread: {:#?}", thread);

            // send a welcome message
            thread
                .send_message(&ctx.http, |message| {
                    message
                        .content("hihi")
                        .allowed_mentions(|mentions| mentions.replied_user(true))
                })
                .await
                .unwrap();

            // TODO(ssoudan) how to display tipping animation?

            // wait for job updates and post
            while let Some(job_update) = rx.next().await {
                info!("Received job update: {:#?}", job_update);

                let msgs = match job_update {
                    JobUpdate::Vec(v) => Some(v),
                    JobUpdate::FailedToStart(e) => Some(e),
                    JobUpdate::ToolError(e) => Some(e),
                    JobUpdate::Over => None,
                };

                if let Some(msgs) = msgs {
                    for txt in msgs {
                        thread
                            .send_message(&ctx.http, |message| {
                                message
                                    .content(txt)
                                    .allowed_mentions(|mentions| mentions.replied_user(true))
                            })
                            .await
                            .unwrap();
                    }
                }
            }

            // Say goodbye
            thread
                .send_message(&ctx.http, |message| {
                    message
                        .content("byebye")
                        .allowed_mentions(|mentions| mentions.replied_user(true))
                })
                .await
                .unwrap();

            return;
        }

        let old_messages: Vec<Message> = new_message
            .channel_id
            .messages(&ctx.http, |messages| {
                messages.before(new_message.id).limit(100)
            })
            .await
            .unwrap()
            .into_iter()
            .collect();

        info!(
            "Old Messages: {:#?}",
            old_messages
                .iter()
                .map(|m| m.content.clone())
                .collect::<Vec<_>>()
        );

        new_message.channel_id.say(&ctx.http, "oui!").await.unwrap();
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Create new commands for this guild
        let commands = GuildId::set_application_commands(&self.guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::ping::register(command))
        })
        .await
        .unwrap();

        info!(
            "I now have the following guild slash commands: {:#?}",
            commands.iter().map(|c| c.name.clone()).collect::<Vec<_>>()
        );
    }
}

// #[tokio::main(flavor = "current_thread")]

#[pyo3_asyncio::tokio::main]
async fn main() -> PyResult<()> {
    let _ = dotenv_override();

    // TODO(ssoudan) graceful shutdown
    // TODO(ssoudan) build the chat history from the channel history

    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();

    let guild_id = GuildId(
        env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .parse()
            .expect("GUILD_ID must be an integer"),
    );

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create Sapiens bot
    let (tx, rx) = mpsc::channel(100);

    // Got to be created before the envs are removed
    let mut runner = runner::Runner::new(rx).await;

    // Remove all environment variables from the environment
    for (key, _) in env::vars() {
        env::remove_var(key);
    }
    assert!(env::vars().next().is_none(), "Environment is not empty");
    ////////////////////////////////////////////////
    // No more environment variables at this point
    ////////////////////////////////////////////////

    // Build the message handler
    let event_handler = Handler {
        guild_id,
        tx: RwLock::new(tx),
    };

    // Build our client.
    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(token, intents)
        .event_handler(event_handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    spawn(async move {
        if let Err(why) = client.start().await {
            info!("Client error: {:?}", why);
        }
    });

    runner.run().await;

    Ok(())
}
