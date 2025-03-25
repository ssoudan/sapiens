//! GCP Vertex AI Generative AI API

use core::fmt::Debug;
use std::sync::Arc;

use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::Ollama;
use tokio::sync::Mutex;
use tracing::debug;

use crate::models;
use crate::models::{
    ChatEntryTokenNumber, ChatInput, Error, ModelRef, ModelResponse, Role, SupportedModel,
};

/// Ollama runtime
#[derive(Clone)]
pub struct LanguageModel {
    model: SupportedModel,

    /// The ollama client
    client: Arc<Mutex<Ollama>>,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for LanguageModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageModel")
            .field("model", &self.model)
            .finish()
    }
}

/// Build an Ollama client
pub fn build(host: String, port: u16, model: SupportedModel) -> Result<ModelRef, Error> {
    let client = Ollama::new(host, port);

    let model = LanguageModel {
        model,
        client: Arc::new(Mutex::new(client)),
    };

    Ok(Arc::new(Box::new(model)))
}

impl LanguageModel {
    fn prepare_input(&self, input: &ChatInput) -> ChatMessageRequest {
        let mut messages = vec![];

        let context = input
            .context
            .iter()
            .map(|c| c.msg.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        messages.push(ChatMessage::system(context));

        for (user, bot) in &input.examples {
            messages.push(ChatMessage::user(user.msg.to_string()));
            messages.push(ChatMessage::assistant(bot.msg.to_string()));
        }

        for entry in &input.chat {
            match entry.role {
                Role::User => messages.push(ChatMessage::user(entry.msg.to_string())),
                Role::Assistant => messages.push(ChatMessage::assistant(entry.msg.to_string())),
                Role::System => messages.push(ChatMessage::system(entry.msg.to_string())),
                _ => {}
            }
        }

        let model_name = self.model.to_string();
        let model_name = model_name.strip_prefix("ollama-").unwrap();

        debug!("model_name: {}", model_name);

        ChatMessageRequest::new(model_name.to_string(), messages)
    }
}

#[async_trait::async_trait]
impl ChatEntryTokenNumber for LanguageModel {
    async fn num_tokens(&self, input: ChatInput) -> usize {
        let prompt = self.prepare_input(&input);

        let char_count = prompt
            .messages
            .iter()
            .map(|m| match m.role {
            ollama_rs::generation::chat::MessageRole::Assistant => 11,
            ollama_rs::generation::chat::MessageRole::System => 8,
            ollama_rs::generation::chat::MessageRole::Tool | ollama_rs::generation::chat::MessageRole::User => 6,
            } + m.content.chars().count())
            .sum::<usize>();

        char_count / 4 // FIXME(ssoudan) this is rough
    }

    async fn context_size(&self) -> usize {
        match self.model {
            SupportedModel::OllamaMixtral => 32768,
            SupportedModel::OllamaLlamaPro => 4096,
            SupportedModel::OllamaLlama3Instruct | SupportedModel::OllamaLlama370BInstruct => 8192,
            _ => {
                panic!("Unsupported model: {:?}", self.model);
            }
        }
    }
}

#[async_trait::async_trait]
impl models::Model for LanguageModel {
    async fn query(
        &self,
        input: ChatInput,
        _max_tokens: Option<usize>,
    ) -> Result<ModelResponse, Error> {
        let prompt = self.prepare_input(&input);

        let client = self.client.lock().await;
        let resp = client.send_chat_messages(prompt).await?;
        drop(client);

        Ok(ModelResponse {
            msg: resp.message.content,
            usage: None,
            finish_reason: None,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::context::ChatEntry;
//     use crate::models::Role;

//     #[tokio::test]
//     async fn test_bison_sizes() {
//         let model = build("http://257.1024.512.12".to_string(), 11434)
//             .await
//             .unwrap();

//         assert_eq!(model.context_size().await, 32768);

//         let input = ChatInput {
//             context: vec![ChatEntry {
//                 role: Role::System,
//                 msg: "A chat between a user and an assistant.".to_string(),
//             }],
//             examples: vec![],
//             chat: vec![
//                 ChatEntry {
//                     role: Role::User,
//                     msg: "Hello, my name is Marcel".to_string(),
//                 },
//                 ChatEntry {
//                     role: Role::Assistant,
//                     msg: "Hello, Marcel, how are you doing
// today?".to_string(),                 },
//                 ChatEntry {
//                     role: Role::User,
//                     msg: "I am doing great, thanks for asking".to_string(),
//                 },
//                 ChatEntry {
//                     role: Role::Assistant,
//                     msg: "That's great to hear!".to_string(),
//                 },
//             ],
//         };

//         let token_sz = model.num_tokens(input.clone()).await;

//         assert_eq!(token_sz, 50);

//         let resp = model.query(input, None).await.unwrap();

//         assert_ne!(resp.msg.len(), 0);
//     }
// }
