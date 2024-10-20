use std::{error::Error, io::ErrorKind, path::Path, sync::Arc};

use auth::{repository::TokenRepository, routes::auth_routes};
use axum::{Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use config::{Args, Config};
use jsonwebtoken::Algorithm;
use server::layer_root_router;
use sqlx::{migrate, SqlitePool};
use storage::{
    manager::ObjectManager, repository::ObjectRepository, routes::file_routes,
};
use tokio::{runtime::Builder, select};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use user::repository::UserRepository;
use utils::{crypto::fetch_jwt_key_files, sys::shutdown_signal};

mod auth;
mod config;
mod errors;
mod server;
mod storage;
mod user;
mod utils;

async fn run_http(cfg: &Config) -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = ObjectManager::new(&cfg.storage);

    let sqlite_path = cfg.storage.state_dir.join("files.sqlite");
    touch_file(&sqlite_path)?;

    let db = SqlitePool::connect(&format!(
        "sqlite:{}",
        sqlite_path.to_string_lossy()
    ))
    .await?;
    migrate!().run(&db).await?;

    let obj_repo = ObjectRepository::new(db.clone());
    let user_repo = UserRepository::new(db, cfg.auth.password_hash_cost);

    let (enc_key, dec_key) =
        fetch_jwt_key_files(&cfg.auth.token_cert, &cfg.auth.token_key)
            .await
            .map_err(|e| format!("failed to get jwt key files: {e}"))?;

    let token_repo = TokenRepository::new(
        Algorithm::EdDSA,
        enc_key,
        dec_key,
        cfg.auth.token_duration,
        cfg.auth.token_duration,
        cfg.auth.secret_key.clone(),
    );

    let app = layer_root_router(
        Router::new()
            .nest("/api/file", file_routes(Router::new()))
            .nest("/api/auth", auth_routes(Router::new())),
    )
    .layer(Extension(obj_repo))
    .layer(Extension(Arc::new(manager)))
    .layer(Extension(user_repo))
    .layer(Extension(Arc::new(token_repo)));

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

fn touch_file(path: &Path) -> Result<(), String> {
    std::fs::File::open(path)
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                std::fs::File::create(path)
            } else {
                Err(err)
            }
        })
        .map(|_| ())
        .map_err(|err| format!("failed to open/create sqlite file: {err}"))
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
