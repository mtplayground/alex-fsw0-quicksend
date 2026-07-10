mod app;
mod config;

use std::process::ExitCode;

use app::build_router;
use config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("server failed: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let bind_address = config.bind_address();
    let listener = TcpListener::bind(bind_address).await?;

    println!("listening on http://{bind_address}");

    axum::serve(listener, build_router(config)).await?;

    Ok(())
}
