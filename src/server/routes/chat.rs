use axum::response::sse::Event;
use axum::response::sse::Sse;
use axum::routing::post;
use axum::{Extension, Json, Router};
use chrono::Utc;
use futures::{stream, Stream};
use futures::stream::StreamExt as _;
use http::StatusCode;
use langchain_rust::language_models::LLMError;
use langchain_rust::llm::{OpenAI, OpenAIConfig};
use reqwest::get;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use indoc::formatdoc;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::schemas::Message;
use crate::database::models::chat::Chat;
use tracing::error;

pub fn new_router(openai: OpenAI<OpenAIConfig>) -> Router {
    Router::new()
        .route("/", post(chat))
        .layer(Extension(openai))
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    user_id: String,
    article_url: String,
    message: String,
}

pub async fn chat(
    Extension(openai): Extension<OpenAI<OpenAIConfig>>,
    Extension(pg_pool): Extension<PgPool>,
    Json(mut request): Json<ChatRequest>
)  -> Sse<impl Stream<Item=Result<Event, LLMError>>> {
    // Find the history of the chat with the user and url
    let user_id = Uuid::parse_str(&request.user_id).expect("Invalid UUID");

    let chat_history = match get_or_create_chat_history(&pg_pool, &user_id, &request.article_url, &request.message).await {
        Ok(history) => history,
        Err(_) => {
            let error_stream = stream::once(async {
                Ok(Event::default().data("Failed to get or create chat history."))
            });
            return Sse::new(error_stream); // Return a compatible stream for the error case
        },
    };


    // Copy code // TODO: Delete
    let messages = vec![
        Message::new_system_message(formatdoc! {
            r#"
                Summarize the key points of the following text concisely.
                The summary should be in the same language as the original text.
                Use plain text without any special characters or formatting.
                Avoid repeating the text verbatim.
                Instead, synthesize the key ideas into a brief, coherent summary.
                Also, ignore any URLs or descriptions of images in the text, especially those in markdown format.
                Content to summarize:

                {content}
            "#,
            content = request.message,
        }),
        Message::new_human_message(&request.message),
    ];

    let openai = openai.with_model("gpt-4-turbo");

    let stream = openai.stream(&messages)
        .await
        .unwrap();

    let stream = stream.map(|result| {
        match result {
            Ok(stream_data) => Ok(Event::default().data(stream_data.content)),
            Err(error) => {
                error!(?error);

                Err(error)
            },
        }
    });

    Sse::new(stream)
}

pub async fn get_or_create_chat_history(
    pg_pool: &PgPool,
    user_id: &Uuid,
    article_url: &str,
    message: &str
) -> sqlx::Result<Chat> {
    let mut chat_history = Chat::get(pg_pool, user_id, article_url).await;

    if chat_history.is_err() {
        let article_content = get_article_content(article_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch or read article content: {}", e))?;


        let new_chat = Chat {
            user_id: *user_id,
            article_url: article_url.to_string(),
            article_content,
            message: message.to_string(),
            created_at: Utc::now(),
        };

        Chat::save(pg_pool, *user_id, article_url, &new_chat.article_content, message).await
            .map_err(|e| anyhow::anyhow!("Failed to save chat history: {}", e))?;

        chat_history = Ok(new_chat);
    }

    chat_history
}

pub async fn get_article_content(article_url: &str) -> Result<String, reqwest::Error> {
    let response = get(&format!("https://r.jina.ai/{}", article_url)).await?;
    let content = response.text().await?;
    Ok(content)
}

fn generate_prompt(chat_history: &Chat, message: &str) -> String {
    formatdoc! {"
        Article Content:
        {}

        User: {}

        Assistant: Let me analyze the article and respond to your message.
    ", chat_history.article_content, message}
}