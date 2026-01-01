use std::{collections::HashMap, iter::Map};

#[derive(serde::Deserialize, Debug)]
pub struct NotionRequestBody {
    data: NotionPageObject,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPageObject {
    parent: NotionPageParent,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPageParent {
    data_source_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionDatabaseResponse {
    results: Vec<NotionPageObjectWithProperties>,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPageObjectWithProperties {
    properties: NotionPropertyMap,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPropertyMap {
    ユーザー名: NotionProperty,
    ユーザーID: NotionProperty,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionProperty {
    id: String,
}

pub async fn post_handler(
    axum::extract::Json(payload): axum::extract::Json<NotionRequestBody>,
) -> &'static str {
    let data_source_id = &payload.data.parent.data_source_id;
    println!("Received payload: {:?}", data_source_id);
    let url = reqwest::Url::parse_with_params(
        &format!(
            "https://api.notion.com/v1/data_sources/{}/query",
            data_source_id
        ),
        &[
            ("filter_properties[]", "ユーザー名"),
            ("filter_properties[]", "ユーザーID"),
        ],
    )
    .unwrap();
    let result = reqwest::Client::new()
        .post(url)
        .header("Notion-Version", "2025-09-03")
        .header(
            "Authorization",
            format!("Bearer {}", get_notion_api_key().await),
        )
        // TODO pagination
        .send()
        .await
        .unwrap()
        .json::<NotionDatabaseResponse>()
        .await
        .unwrap();
    for page in &result.results {
        println!("Page properties: {:?}", page.properties);
    }
    "ok"
}

async fn get_notion_api_key() -> String {
    match std::env::var("NOTION_KEY") {
        // in Cloud Run
        Ok(val) => val,
        // local dev
        Err(_) => std::fs::read_to_string("./notionApiKey.txt").unwrap(),
    }
}
