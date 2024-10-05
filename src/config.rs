use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::utils::serde::{
    deserialize_socket_addr, ResolvedFile, ResolvedPath,
};

pub const DEFAULT_HTTP_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
pub const DEFAULT_TCP_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7777);
pub const DEFAULT_TEMP_DIR: &'static str = "/tmp/downloader";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
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

pub fn load(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = fs::read_to_string(path)?;

    if path.ends_with(".json") {
        serde_json::from_str(&file).map_err(Into::into)
    } else {
        toml::from_str(&file).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub net: NetConfig,
    pub ssl: SslConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetConfig {
    #[serde(default = "default_true")]
    pub enable_http: bool,
    #[serde(
        default = "default_http_addr",
        deserialize_with = "deserialize_socket_addr"
    )]
    pub http_addr: SocketAddr,

    #[serde(default = "default_false")]
    pub enable_tcp: bool,
    #[serde(
        default = "default_tcp_addr",
        deserialize_with = "deserialize_socket_addr"
    )]
    pub tpc_addr: SocketAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    #[serde(default = "default_true")]
    pub enable: bool,
    pub cert: Option<ResolvedFile>,
    pub key: Option<ResolvedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub state_dir: ResolvedPath,
    pub data_dir: ResolvedPath,
    #[serde(default = "default_temp_dir")]
    pub temp_dir: ResolvedPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token_cert: ResolvedFile,
    pub token_key: ResolvedFile,

    pub secret_key: String,
}

const fn default_false() -> bool {
    false
}

const fn default_true() -> bool {
    true
}

const fn default_http_addr() -> SocketAddr {
    DEFAULT_HTTP_ADDR
}

const fn default_tcp_addr() -> SocketAddr {
    DEFAULT_TCP_ADDR
}

fn default_temp_dir() -> ResolvedPath {
    ResolvedPath::new(DEFAULT_TEMP_DIR.into())
        .expect("failed to parse default temp path into ResolvedPath")
}
