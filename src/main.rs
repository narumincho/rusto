mod graph_generator;
mod notion;
mod notion_minecraft_db;
mod tau_t_shirt;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    graph_generator::generate_graph().await?;
    Ok(())
}
