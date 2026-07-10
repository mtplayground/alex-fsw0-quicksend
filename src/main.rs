mod app;
mod config;
mod email;
mod models;
mod rate_limit;
mod routes;
mod validation;

use std::path::Path;
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
    ensure_frontend_assets(&config.frontend_dist_dir)?;
    let bind_address = config.bind_address();
    let listener = TcpListener::bind(bind_address).await?;

    println!("listening on http://{bind_address}");

    axum::serve(listener, build_router(config)).await?;

    Ok(())
}

fn ensure_frontend_assets(frontend_dist_dir: &str) -> Result<(), std::io::Error> {
    let index_path = Path::new(frontend_dist_dir).join("index.html");

    if index_path.is_file() {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "frontend assets are missing: expected {}. Run the frontend build or set FRONTEND_DIST_DIR.",
            index_path.display()
        ),
    ))
}
