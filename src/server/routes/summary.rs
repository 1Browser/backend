use axum::response::sse::Event;
use axum::response::sse::Sse;
use axum::routing::post;
use axum::{Extension, Json, Router};
use futures::Stream;
use indoc::formatdoc;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::language_models::LLMError;
use langchain_rust::llm::{OpenAI, OpenAIConfig};
use langchain_rust::schemas::Message;
use serde::Deserialize;
use tokio_stream::StreamExt as _;
use tracing::error;

pub fn new_router(openai: OpenAI<OpenAIConfig>) -> Router {
    Router::new()
        .route("/", post(summary))
        .layer(Extension(openai))
}

#[derive(Debug, Deserialize)]
pub struct SummaryRequest {
    content: String,
}

pub async fn summary(Extension(openai): Extension<OpenAI<OpenAIConfig>>, Json(request): Json<SummaryRequest>) -> Sse<impl Stream<Item=Result<Event, LLMError>>> {
    let messages = vec![
        Message::new_system_message(formatdoc! {
            r#"
                Summarize the key points of the following text concisely.
                The summary should be in the same language as the original text.
                Use plain text without any special characters or formatting.
                Avoid repeating the text verbatim.
                Instead, synthesize the key ideas into a brief, coherent summary.
                Also, ignore any links or descriptions of images in the text.
                Content to summarize:

                {content}
            "#,
            content = request.content,
        }),
        Message::new_human_message(&request.content),
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
