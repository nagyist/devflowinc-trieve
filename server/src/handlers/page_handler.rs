use std::env;

use crate::{
    data::models::{DatasetConfiguration, Pool, SearchMethod, SortOptions, TypoOptions, UnifiedId},
    errors::ServiceError,
    get_env,
    operators::dataset_operator::get_dataset_by_id_query,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use minijinja::context;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::data::models::Templates;

use super::{
    auth_handler::LoggedUser,
    chunk_handler::{ChunkFilter, ScoringOptions},
};

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
pub enum PublicPageTheme {
    #[default]
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

// Duplicate of SearchChunksReqPayload but without "query"
#[derive(Serialize, Clone, Debug, ToSchema, Deserialize)]
#[schema(example = json!({
    "search_type": "semantic",
    "filters": {
        "should": [
            {
                "field": "metadata.key1",
                "match": ["value1", "value2"],
            }
        ],
        "must": [
            {
                "field": "num_value",
                "range": {
                    "gte": 0.0,
                    "lte": 1.0,
                    "gt": 0.0,
                    "lt": 1.0
                }
            }
        ],
        "must_not": [
            {
                "field": "metadata.key3",
                "match": ["value5", "value6"],
            }
        ]
    },
    "score_threshold": 0.5
}))]
pub struct PublicPageSearchOptions {
    /// Can be either "semantic", "fulltext", "hybrid, or "bm25". If specified as "hybrid", it will pull in one page of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page of the nearest cosine distant vectors. "fulltext" will pull in one page of full-text results based on SPLADE. "bm25" will get one page of results scored using BM25 with the terms OR'd together.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_type: Option<SearchMethod>,
    /// Page of chunks to fetch. Page is 1-indexed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    /// Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u64>,
    /// Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<ChunkFilter>,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_options: Option<SortOptions>,
    /// Scoring options provides ways to modify the sparse or dense vector created for the query in order to change how potential matches are scored. If not specified, this defaults to no modifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_options: Option<ScoringOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold for cosine distance metric. For Manhattan Distance, Euclidean Distance, and Dot Product, it will filter out scores above the threshold distance. This threshold applies before weight and bias modifications. If not specified, this defaults to no threshold. A threshold of 0 will default to no threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slim_chunks: Option<bool>,
    /// Set content_only to true to only returning the chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_only: Option<bool>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_quote_negated_terms: Option<bool>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_stop_words: Option<bool>,
    /// User ID is the id of the user who is making the request. This is used to track user interactions with the search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Typo options lets you specify different methods to handle typos in the search query. If not specified, this defaults to no typo handling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typo_options: Option<TypoOptions>,
    /// Enables autocomplete on the search modal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_autocomplete: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_queries: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<PublicPageTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_options: Option<PublicPageSearchOptions>,
    //pub openKeyCombination: { key?: string; label?: string; ctrl?: boolean }[],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_logo_img_src_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_queries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_ai_questions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_group_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_switching_modes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debounce_ms: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/public_page/{dataset_id}",
    context_path = "/api",
    tag = "Public",
    responses(
        (status = 200, description = "Public Page associated to the dataset"),
        (status = 400, description = "Service error relating to loading the public page", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the organization you want to fetch."),
    ),
)]
pub async fn public_page(
    dataset_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    templates: Templates<'_>,
    req: HttpRequest,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = dataset_id.into_inner();

    let dataset = get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id), pool).await?;

    let config = DatasetConfiguration::from_json(dataset.server_configuration);

    let base_server_url = get_env!(
        "BASE_SERVER_URL",
        "Server hostname for OpenID provider must be set"
    );

    let logged_in = req.extensions().get::<LoggedUser>().is_some();
    let dashboard_url =
        env::var("ADMIN_DASHBOARD_URL").unwrap_or("https://dashboard.trieve.ai".to_string());

    if config.PUBLIC_DATASET.enabled {
        let templ = templates.get_template("page.html").unwrap();
        let response_body = templ
            .render(context! {
                logged_in,
                dashboard_url,
                params => PublicPageParameters {
                    dataset_id: Some(dataset_id),
                    base_url: Some(base_server_url.to_string()),
                    api_key: Some(config.PUBLIC_DATASET.api_key.unwrap_or_default()),
                    ..config.PUBLIC_DATASET.extra_params.unwrap_or_default()
                }
            })
            .unwrap();

        Ok(HttpResponse::Ok().body(response_body))
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
