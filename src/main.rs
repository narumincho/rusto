mod graph_generator;
mod map_override;
mod notion;
mod notion_minecraft_db;
mod tau_t_shirt;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    map_override::server().await?;
    Ok(())
}
