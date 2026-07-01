#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    rusto::map_override::server().await?;
    Ok(())
}
