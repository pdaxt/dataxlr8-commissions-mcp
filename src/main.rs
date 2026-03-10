use anyhow::Result;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use tracing::info;

mod db;
mod tools;

#[cfg(test)]
mod tests;

use tools::CommissionsMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = dataxlr8_mcp_core::Config::from_env("dataxlr8-commissions-mcp")
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    dataxlr8_mcp_core::logging::init(&config.log_level);

    info!(
        server = config.server_name,
        "Starting DataXLR8 Commissions MCP server"
    );

    let database = dataxlr8_mcp_core::Database::connect(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    db::setup_schema(database.pool()).await?;

    let server = CommissionsMcpServer::new(database.clone());

    let transport = stdio();
    let service = server.serve(transport).await?;

    info!("Commissions MCP server connected via stdio");
    service.waiting().await?;

    database.close().await;
    info!("Commissions MCP server shut down");

    Ok(())
}
