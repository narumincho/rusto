const DATA_SOURCE_ID: &str = "2da33257-b9cf-8030-9be8-000b4a60ce28";

pub async fn update_minecraft_db() -> &'static str {
    let mut cursor = None;

    loop {
        let pages_and_cursor =
            match crate::notion::get_notion_pages_in_data_source(DATA_SOURCE_ID, cursor).await {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Error getting notion pages: {}", e);
                    return "error";
                }
            };
        cursor = pages_and_cursor.next_cursor;
        for page in pages_and_cursor.results {
            println!("Processing page: {}", page.user_name);
            if page.user_id.is_empty() {
                let user_uuid_option = match get_user_uuid_from_user_name(&page.user_name).await {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("Error getting uuid for user {}: {}", page.user_name, e);
                        None
                    }
                };
                if let Err(e) = crate::notion::update_page(
                    &page.id,
                    &crate::notion::UpdatePageParameter {
                        user_name: None,
                        user_id: Some(match user_uuid_option {
                            Some(uuid) => uuid.to_string(),
                            None => "不明なユーザー名です".to_string(),
                        }),
                        user_icon_url: user_uuid_option
                            .map(|uuid| format!("https://mc-heads.net/avatar/{}", uuid)),
                    },
                )
                .await
                {
                    eprintln!("Error updating page {}: {}", page.id, e);
                }
            }
        }
        if cursor.is_none() {
            return "ok";
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct MojangUserResponse {
    id: Option<uuid::Uuid>,
}

async fn get_user_uuid_from_user_name(user_name: &String) -> anyhow::Result<Option<uuid::Uuid>> {
    let response = reqwest::get(format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        user_name
    ))
    .await?;
    if response.status() == reqwest::StatusCode::NO_CONTENT
        || response.status() == reqwest::StatusCode::NOT_FOUND
    {
        return Ok(None);
    }
    let response_body = response.json::<MojangUserResponse>().await?;
    Ok(response_body.id)
}
