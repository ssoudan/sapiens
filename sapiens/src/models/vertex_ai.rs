//! GCP Vertex AI Generative AI API

use core::fmt::Debug;
use std::sync::Arc;

use gcp_vertex_ai_generative_language::google::ai::generativelanguage::v1beta2::content_filter::BlockedReason;
use gcp_vertex_ai_generative_language::google::ai::generativelanguage::v1beta2::{
    CountMessageTokensRequest, Example, GenerateMessageRequest, GetModelRequest, Message,
    MessagePrompt,
};
use gcp_vertex_ai_generative_language::{Credentials, LanguageClient};
use tokio::sync::Mutex;
use tracing::warn;

use crate::models;
use crate::models::{
    ChatEntryTokenNumber, ChatInput, Error, ModelRef, ModelResponse, Role, SupportedModel,
};

/// GCP Vertex AI Generative Language Model
#[derive(Clone)]
pub struct LanguageModel {
    model: SupportedModel,

    /// the temperature
    pub temperature: Option<f32>,
    /// The GCP Vertex AI client
    client: Arc<Mutex<LanguageClient>>,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for LanguageModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageModel")
            .field("temperature", &self.temperature)
            .field("model", &self.model)
            .finish()
    }
}

/// Build a GCP Vertex AI Generative Language Model client
///
/// # Panics
///
/// Panics if the API key is not set
pub async fn build(api_key: String, temperature: Option<f32>) -> Result<ModelRef, Error> {
    let client = LanguageClient::new(Credentials::ApiKey(api_key))
        .await
        .unwrap();

    let model = LanguageModel {
        model: SupportedModel::ChatBison001,
        temperature,
        client: Arc::new(Mutex::new(client)),
    };

    Ok(Arc::new(Box::new(model)))
}

impl LanguageModel {
    fn prepare_input(input: &ChatInput) -> MessagePrompt {
        let context = input
            .context
            .iter()
            .map(|c| c.msg.clone())
            .collect::<Vec<String>>()
            .join("\n");

        let examples = input
            .examples
            .iter()
            .map(|(user, bot)| Example {
                input: Some(Message {
                    author: Role::User.to_string(),
                    content: user.msg.clone(),
                    citation_metadata: None,
                }),
                output: Some(Message {
                    author: Role::Assistant.to_string(),
                    content: bot.msg.clone(),
                    citation_metadata: None,
                }),
            })
            .collect();

        let messages = input
            .chat
            .iter()
            .map(|m| Message {
                author: m.role.to_string(),
                content: m.msg.clone(),
                citation_metadata: None,
            })
            .collect();

        MessagePrompt {
            context,
            examples,
            messages,
        }
    }
}

#[async_trait::async_trait]
impl ChatEntryTokenNumber for LanguageModel {
    async fn num_tokens(&self, input: ChatInput) -> usize {
        let prompt = Self::prepare_input(&input);

        let req = CountMessageTokensRequest {
            model: format!("models/{}", self.model),
            prompt: Some(prompt),
        };

        let mut client = self.client.lock().await;
        let resp = client
            .discuss_service
            .count_message_tokens(req)
            .await
            .unwrap();
        drop(client);

        resp.get_ref().token_count as usize
    }

    async fn context_size(&self) -> usize {
        let mut client = self.client.lock().await;

        let req = GetModelRequest {
            name: format!("models/{}", self.model),
        };

        client
            .model_service
            .get_model(req)
            .await
            .unwrap()
            .get_ref()
            .input_token_limit as usize
    }
}

#[async_trait::async_trait]
impl models::Model for LanguageModel {
    async fn query(
        &self,
        input: ChatInput,
        _max_tokens: Option<usize>,
    ) -> Result<ModelResponse, Error> {
        let prompt = Self::prepare_input(&input);

        let req = GenerateMessageRequest {
            model: format!("models/{}", self.model),
            prompt: Some(prompt),
            temperature: self.temperature,
            candidate_count: Some(1),
            top_p: None,
            top_k: None,
        };

        let mut client = self.client.lock().await;
        let resp = client
            .discuss_service
            .generate_message(req)
            .await
            .map_err(gcp_vertex_ai_generative_language::Error::from)?;
        drop(client);

        let resp = resp.get_ref();

        if resp.candidates.is_empty() {
            if !resp.filters.is_empty() {
                resp.filters.iter().for_each(|f| {
                    if let Some(message) = f.message.as_ref() {
                        warn!(
                            "Filter: {:?} - {}",
                            BlockedReason::try_from(f.reason).unwrap_or(BlockedReason::Unspecified),
                            message
                        );
                    } else {
                        warn!(
                            "Filter: {:?}",
                            BlockedReason::try_from(f.reason).unwrap_or(BlockedReason::Unspecified)
                        );
                    }
                });
                return Err(Error::Filtered);
            }

            return Err(Error::NoResponseFromModel);
        }

        Ok(ModelResponse {
            msg: resp.candidates[0].content.clone(),
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
//
//     #[tokio::test]
//     async fn test_bison_sizes() {
//         let api_key = std::env::var("GOOGLE_API_KEY").unwrap();
//
//         let model = build(api_key, None).await.unwrap();
//
//         assert_eq!(model.context_size().await, 4096);
//
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
//
//         let token_sz = model.num_tokens(input).await;
//
//         assert_eq!(token_sz, 67);
//     }
// }
