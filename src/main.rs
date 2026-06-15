mod notion;
mod notion_minecraft_db;
mod tau_t_shirt;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tau_t_shirt::main()
}
