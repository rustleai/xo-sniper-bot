use crate::{
    agent::AgentBuilder,
    embeddings::{self},
    extractor::ExtractorBuilder,
    Embed,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{completion::CompletionModel, embedding::EmbeddingModel};

// ================================================================
// Google Gemini Client
// ================================================================
const GEMINI_API_BASE_URL: &str = "https://generativelanguage.googleapis.com";

#[derive(Clone)]
pub struct Client {
    base_url: String,
    api_key: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, GEMINI_API_BASE_URL)
    }
    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        reqwest::header::CONTENT_TYPE,
                        "application/json".parse().unwrap(),
                    );
                    headers
                })
                .build()
                .expect("Gemini reqwest client should build"),
        }
    }

    /// Create a new Google Gemini client from the `GEMINI_API_KEY` environment variable.
    /// Panics if the environment variable is not set.
    pub fn from_env() -> Self {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
        Self::new(&api_key)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}?key={}", self.base_url, path, self.api_key).replace("//", "/");

        tracing::debug!("POST {}", url);
        self.http_client.post(url)
    }

    /// Create an embedding model with the given name.
    /// Note: default embedding dimension of 0 will be used if model is not known.
    /// If this is the case, it's better to use function `embedding_model_with_ndims`
    ///
    /// # Example
    /// ```
    /// use lumy::providers::gemini::{Client, self};
    ///
    /// // Initialize the Google Gemini client
    /// let gemini = Client::new("your-google-gemini-api-key");
    ///
    /// let embedding_model = gemini.embedding_model(gemini::embedding::EMBEDDING_GECKO_001);
    /// ```
    pub fn embedding_model(&self, model: &str) -> EmbeddingModel {
        EmbeddingModel::new(self.clone(), model, None)
    }

    /// Create an embedding model with the given name and the number of dimensions in the embedding generated by the model.
    ///
    /// # Example
    /// ```
    /// use lumy::providers::gemini::{Client, self};
    ///
    /// // Initialize the Google Gemini client
    /// let gemini = Client::new("your-google-gemini-api-key");
    ///
    /// let embedding_model = gemini.embedding_model_with_ndims("model-unknown-to-lumy", 1024);
    /// ```
    pub fn embedding_model_with_ndims(&self, model: &str, ndims: usize) -> EmbeddingModel {
        EmbeddingModel::new(self.clone(), model, Some(ndims))
    }

    /// Create an embedding builder with the given embedding model.
    ///
    /// # Example
    /// ```
    /// use lumy::providers::gemini::{Client, self};
    ///
    /// // Initialize the Google Gemini client
    /// let gemini = Client::new("your-google-gemini-api-key");
    ///
    /// let embeddings = gemini.embeddings(gemini::embedding::EMBEDDING_GECKO_001)
    ///     .simple_document("doc0", "Hello, world!")
    ///     .simple_document("doc1", "Goodbye, world!")
    ///     .build()
    ///     .await
    ///     .expect("Failed to embed documents");
    /// ```
    pub fn embeddings<D: Embed>(
        &self,
        model: &str,
    ) -> embeddings::EmbeddingsBuilder<EmbeddingModel, D> {
        embeddings::EmbeddingsBuilder::new(self.embedding_model(model))
    }

    /// Create a completion model with the given name.
    /// Gemini-specific parameters can be set using the [GenerationConfig](crate::providers::gemini::completion::gemini_api_types::GenerationConfig) struct.
    /// [Gemini API Reference](https://ai.google.dev/api/generate-content#generationconfig)
    pub fn completion_model(&self, model: &str) -> CompletionModel {
        CompletionModel::new(self.clone(), model)
    }

    /// Create an agent builder with the given completion model.
    /// Gemini-specific parameters can be set using the [GenerationConfig](crate::providers::gemini::completion::gemini_api_types::GenerationConfig) struct.
    /// [Gemini API Reference](https://ai.google.dev/api/generate-content#generationconfig)
    /// # Example
    /// ```
    /// use lumy::providers::gemini::{Client, self};
    ///
    /// // Initialize the Google Gemini client
    /// let gemini = Client::new("your-google-gemini-api-key");
    ///
    /// let agent = gemini.agent(gemini::completion::GEMINI_1_5_PRO)
    ///    .preamble("You are comedian AI with a mission to make people laugh.")
    ///    .temperature(0.0)
    ///    .build();
    /// ```
    pub fn agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }

    /// Create an extractor builder with the given completion model.
    pub fn extractor<T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync>(
        &self,
        model: &str,
    ) -> ExtractorBuilder<T, CompletionModel> {
        ExtractorBuilder::new(self.completion_model(model))
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}
