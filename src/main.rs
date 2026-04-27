mod app;
mod post;
mod services;
mod state;
mod templates;

use {
    state::State,
    std::{env, io::Error, path::PathBuf},
    tracing_subscriber::EnvFilter,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let posts_path = env::var("POSTS_PATH").unwrap();
    let addr = env::var("ADDRESS").unwrap();
    let port: u16 = env::var("PORT").unwrap().parse().unwrap();

    let state = State {
        posts_path: PathBuf::from(posts_path),
    };

    app::run(state, addr, port).await?;

    Ok(())
}
