use saas_agent_error_loop::{onboard_tenant, InfraiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = InfraiClient::from_env()?;
    let tenant = std::env::args().nth(1).unwrap_or_else(|| "acme-eu".into());
    let state = onboard_tenant(&client, &tenant, true).await?;
    println!("tenant {tenant}: {state:?}");
    Ok(())
}

