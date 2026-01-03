use image::codecs::jpeg::PixelDensity;

pub struct UserNameAndId {
    pub id: String,
    pub user_name: String,
    pub user_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionRequestBody {
    pub data: NotionPageObject,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPageObject {
    pub parent: NotionPageParent,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPageParent {
    pub data_source_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionDatabaseResponse {
    results: Vec<NotionPageObjectWithProperties>,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPageObjectWithProperties {
    id: String,
    properties: NotionPropertyMap,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionPropertyMap {
    ユーザー名: NotionTitleProperty,
    ユーザーID: NotionRichTextProperty,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NotionTitleProperty {
    title: Vec<NotionRichTextItem>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NotionRichTextProperty {
    rich_text: Vec<NotionRichTextItem>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NotionRichTextItem {
    text: NotionRichTextItemText,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NotionRichTextItemText {
    content: String,
}

pub async fn get_notion_pages_in_data_source(data_source_id: &String) -> Vec<UserNameAndId> {
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
    let response: NotionDatabaseResponse = reqwest::Client::new()
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
    response
        .results
        .into_iter()
        .map(|page| UserNameAndId {
            id: page.id,
            user_name: rich_text_to_string(&page.properties.ユーザー名.title),
            user_id: rich_text_to_string(&page.properties.ユーザーID.rich_text),
        })
        .collect()
}

async fn get_notion_api_key() -> String {
    match std::env::var("NOTION_KEY") {
        // in Cloud Run
        Ok(val) => val,
        // local dev
        Err(_) => std::fs::read_to_string("./notionApiKey.txt").unwrap(),
    }
}

fn rich_text_to_string(rich_text: &Vec<NotionRichTextItem>) -> String {
    rich_text
        .iter()
        .map(|item| item.text.content.clone())
        .collect::<Vec<String>>()
        .join("")
}

pub struct UpdatePageParameter {
    pub user_id: Option<String>,
    pub user_name: Option<String>,
}

#[derive(serde::Serialize)]
struct UpdatePageRequestBody {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<UpdatePageIcon>,
    properties: UpdatePropertyParameter,
}

#[derive(serde::Serialize)]
struct UpdatePageIcon {
    #[serde(rename = "type")]
    type_: String,
    external: UpdatePageIconExternal,
}

#[derive(serde::Serialize)]
struct UpdatePageIconExternal {
    url: String,
}

#[derive(serde::Serialize)]
struct UpdatePropertyParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    ユーザー名: Option<NotionTitleProperty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ユーザーID: Option<NotionRichTextProperty>,
}

pub async fn update_page(page_id: &String, parameter: &UpdatePageParameter) {
    let update_url =
        reqwest::Url::parse(&format!("https://api.notion.com/v1/pages/{}", page_id)).unwrap();
    let update_body = UpdatePageRequestBody {
        id: page_id.clone(),
        icon: parameter.user_id.as_ref().map(|user_id| UpdatePageIcon {
            type_: "external".to_string(),
            external: UpdatePageIconExternal {
                url: format!("https://mc-heads.net/avatar/{}", user_id),
            },
        }),
        properties: UpdatePropertyParameter {
            ユーザー名: parameter
                .user_name
                .as_ref()
                .map(|name| NotionTitleProperty {
                    title: vec![NotionRichTextItem {
                        text: NotionRichTextItemText {
                            content: name.clone(),
                        },
                    }],
                }),
            ユーザーID: parameter.user_id.as_ref().map(|id| NotionRichTextProperty {
                rich_text: vec![NotionRichTextItem {
                    text: NotionRichTextItemText {
                        content: id.clone(),
                    },
                }],
            }),
        },
    };
    let response = reqwest::Client::new()
        .patch(update_url)
        .header("Notion-Version", "2025-09-03")
        .header(
            "Authorization",
            format!("Bearer {}", get_notion_api_key().await),
        )
        .json(&update_body)
        .send()
        .await
        .unwrap();
    println!("response: {}", response.text().await.unwrap());
}
