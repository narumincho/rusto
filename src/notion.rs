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
    next_cursor: Option<String>,
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

#[derive(serde::Serialize, Debug)]
pub struct NotionQueryRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    start_cursor: Option<String>,
}

pub struct NotionQueryResponse {
    pub results: Vec<UserNameAndId>,
    pub next_cursor: Option<String>,
}

pub async fn get_notion_pages_in_data_source(
    data_source_id: &String,
    start_cursor: Option<String>,
) -> anyhow::Result<NotionQueryResponse> {
    let url = reqwest::Url::parse_with_params(
        &format!(
            "https://api.notion.com/v1/data_sources/{}/query",
            data_source_id
        ),
        &[
            ("filter_properties[]", "ユーザー名"),
            ("filter_properties[]", "ユーザーID"),
        ],
    )?;
    let response: NotionDatabaseResponse = reqwest::Client::new()
        .post(url)
        .header("Notion-Version", "2025-09-03")
        .header(
            "Authorization",
            format!("Bearer {}", get_notion_api_key().await?),
        )
        .json(&NotionQueryRequestBody { start_cursor })
        .send()
        .await?
        .json::<NotionDatabaseResponse>()
        .await?;
    Ok(NotionQueryResponse {
        results: response
            .results
            .into_iter()
            .map(|page| UserNameAndId {
                id: page.id,
                user_name: rich_text_to_string(&page.properties.ユーザー名.title),
                user_id: rich_text_to_string(&page.properties.ユーザーID.rich_text),
            })
            .collect(),
        next_cursor: response.next_cursor,
    })
}

async fn get_notion_api_key() -> anyhow::Result<String> {
    match std::env::var("NOTION_KEY") {
        // in Cloud Run
        Ok(val) => Ok(val),
        // local dev
        Err(_) => Ok(std::fs::read_to_string("./notionApiKey.txt")?),
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
    pub user_icon_url: Option<String>,
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

pub async fn update_page(page_id: &String, parameter: &UpdatePageParameter) -> anyhow::Result<()> {
    let update_url = reqwest::Url::parse(&format!("https://api.notion.com/v1/pages/{}", page_id))?;
    let update_body = UpdatePageRequestBody {
        id: page_id.clone(),
        icon: parameter
            .user_icon_url
            .as_ref()
            .map(|user_icon_url| UpdatePageIcon {
                type_: "external".to_string(),
                external: UpdatePageIconExternal {
                    url: user_icon_url.clone(),
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
            format!("Bearer {}", get_notion_api_key().await?),
        )
        .json(&update_body)
        .send()
        .await?;
    println!("response: {}", response.text().await?);
    Ok(())
}
