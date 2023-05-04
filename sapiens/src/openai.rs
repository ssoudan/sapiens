pub use async_openai::error::OpenAIError;
pub use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Role};
pub use async_openai::Client;

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn can_connect() {
//         // let api_base = "https://api.openai.com/v1".to_string();
//         let api_base = "http://localhost:8000/v1".to_string();
//         let openai_client = Client::new().with_api_base(api_base);
//
//         let request = CreateChatCompletionRequest {
//             model: "vicuna-7b-1.1".to_string(),
//             messages: vec![ChatCompletionRequestMessage {
//                 role: Role::User,
//                 content: "Hello, my name is Marcel".to_string(),
//                 name: None,
//             }],
//             temperature: Some(0.0),
//             top_p: None,
//             n: Some(1),
//             stream: None,
//             stop: None,
//             max_tokens: Some(1024),
//             presence_penalty: None,
//             frequency_penalty: None,
//             logit_bias: None,
//             user: None,
//         };
//
//     let response = openai_client.chat().create(request).await;
//     assert!(response.is_ok());
//
//     let response = response.unwrap();
//     println!("{:#?}", response);
//
//     println!("{}", response.choices.first().unwrap().message.content);
// }
// }
