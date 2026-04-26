mod app;
mod post;
mod services;
mod state;
mod templates;

use {
    state::State,
    std::{io::Error, path::PathBuf},
    tracing_subscriber::EnvFilter,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = State {
        posts_path: PathBuf::new().join("posts"),
    };

    app::run(state).await?;

    Ok(())
}
