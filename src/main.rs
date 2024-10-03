use std::{error::Error, sync::Arc};

use axum::{routing, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use config::Config;
use server::layer_router;
use sqlx::{migrate, SqlitePool};
use storage::{manager::ObjectManager, repository::ObjectRepository, routes};
use tokio::{runtime::Builder, select};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use utils::sys::shutdown_signal;

mod config;
mod errors;
mod server;
mod storage;
mod utils;

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

async fn run_http(cfg: &Config) -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = ObjectManager::new(&cfg.storage);

    let sqlite_uri =
        format!("sqlite:{:?}", cfg.storage.state_dir.join("files.sqlite"));

    let db = SqlitePool::connect(&sqlite_uri).await?;
    migrate!().run(&db).await?;

    let repository = ObjectRepository::new(db);

    let app = layer_router(
        Router::new()
            .route("/file/:id", routing::get(routes::get_file))
            .route("/file", routing::post(routes::post_file))
            .route("/file/:id", routing::delete(routes::delete_file))
            .route("/file/:id", routing::put(routes::update_file))
            .route(
                "/file-multipart",
                routing::post(routes::post_file_multipart),
            )
            .route(
                "/file-multipart/:id",
                routing::put(routes::update_file_multipart),
            )
            .layer(Extension(repository))
            .layer(Extension(Arc::new(manager))),
    );

    let tls_cfg = load_tls_config(&cfg.ssl).await;

    tracing::info!(
        addr = %cfg.net.http_addr,
        tls_enabled = tls_cfg.is_some(),
        "listening for http connections",
    );

    if let Some(tls_cfg) = tls_cfg {
        axum_server::bind_rustls(cfg.net.http_addr, tls_cfg)
            .serve(app.into_make_service())
            .await?;
    } else {
        axum_server::bind(cfg.net.http_addr)
            .serve(app.into_make_service())
            .await?;
    }

    Ok(())
}

async fn run(cfg: Config) -> Result<(), Box<dyn Error + Send + Sync>> {
    let signal = shutdown_signal()?;

    select! {
        _ = signal => {}
        res = run_http(&cfg) => {
            if let Err(err) = res {
                return Err(err);
            }
        }
    }

    tracing::info!("closed http server");

    Ok(())
}

async fn load_tls_config(cfg: &config::SslConfig) -> Option<RustlsConfig> {
    if !cfg.enable {
        return None;
    }

    if cfg.cert.is_none() {
        tracing::error!("TLS is enable but certificate file was not provided");
    }
    if cfg.key.is_none() {
        tracing::error!("TLS is enable but key file was not provided");
    }

    RustlsConfig::from_pem_file(
        cfg.cert.as_ref()?.as_str(),
        cfg.key.as_ref()?.as_str(),
    )
    .await
    .map_err(|error| tracing::error!(%error, "failed to load TLS pem files"))
    .ok()
}

fn main() {
    let args = Args::parse();

    if args.debug {
        let builder =
            tracing_subscriber::fmt().with_max_level(LevelFilter::DEBUG);

        if args.json_logs {
            builder.json().init();
        } else {
            builder.init();
        }
    } else {
        let builder = tracing_subscriber::fmt().with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        );

        if args.json_logs {
            builder.json().init();
        } else {
            builder.init();
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

    tracing::debug!(config = ?cfg, "loaded configuration");

    let tokio_result = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(run(cfg));

    if let Err(e) = tokio_result {
        fatal!("Unhandled error: {e}");
    }
}
