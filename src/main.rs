mod config;
mod utils;

use std::{error::Error, future::Future};

use axum::Router;
use clap::Parser;
use config::Config;
use tokio::{net::TcpListener, runtime::Builder};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use utils::sys::shutdown_signal;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
    #[arg(short, long, default_value_t = false)]
    pub json_logs: bool,

    #[arg(
        short,
        long,
        default_value_t = String::from("/etc/downloader/config.toml"),
    )]
    pub config_path: String,
}

async fn run_http(
    cfg: &Config,
    signal: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = Router::new();

    let ln = TcpListener::bind(cfg.net.http_addr).await?;

    axum::serve(ln, app)
        .with_graceful_shutdown(signal)
        .await
        .map_err(Into::into)
}

async fn run(cfg: Config) -> Result<(), Box<dyn Error + Send + Sync>> {
    let signal = shutdown_signal()?;

    run_http(&cfg, signal).await?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    if args.debug {
        let builder =
            tracing_subscriber::fmt().with_max_level(LevelFilter::DEBUG);

        if args.json_logs {
            builder.json().init();
        } else {
            builder.compact().init();
        }
    } else {
        let builder = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env());

        if args.json_logs {
            builder.json().init();
        } else {
            builder.compact().init();
        }
    }

    let cfg = match config::load(&args.config_path) {
        Ok(v) => v,
        Err(err) => {
            fatal!(
                "Failed to open config file at `{}`: {}\n\
                Try specifying it the `--config-path` argument",
                args.config_path,
                err
            )
        }
    };

    tracing::debug!(config = ?cfg, "Loaded configuration");

    let tokio_result = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(run(cfg));

    if let Err(e) = tokio_result {
        fatal!("Unhandled error: {e}");
    }
}
