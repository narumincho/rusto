use std::collections::HashMap;

pub async fn post_handler(
    axum::extract::Json(payload): axum::extract::Json<crate::notion::NotionRequestBody>,
) -> &'static str {
    let data_source_id = &payload.data.parent.data_source_id;
    let pages = crate::notion::get_notion_pages_in_data_source(data_source_id).await;
    for page in pages {
        if page.user_id.is_empty() {
            let user_uuid = get_user_uuid_from_user_name(&page.user_name).await;
            crate::notion::update_page(
                &page.id,
                &crate::notion::UpdatePageParameter {
                    user_name: None,
                    user_id: Some(user_uuid),
                },
            )
            .await;
        }
    }
    "ok"
}

#[derive(serde::Deserialize, Debug)]
struct MojangUserResponse {
    id: String,
}

async fn get_user_uuid_from_user_name(user_name: &String) -> String {
    let response = reqwest::get(format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        user_name
    ))
    .await
    .unwrap();
    response.json::<MojangUserResponse>().await.unwrap().id
}
