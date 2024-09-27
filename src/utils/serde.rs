use std::{
    fmt::Debug,
    fs::{self},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Deref,
};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedFile(pub String);

impl ResolvedFile {
    pub fn new(path: String) -> Result<Self, String> {
        let meta = fs::metadata(&path)
            .map_err(|err| format!("failed to open file `{path}`: {err}"))?;

        if !meta.is_file() {
            return Err(format!("`{path}` is not a valid file"));
        }

        Ok(ResolvedFile(path))
    }
}

impl Deref for ResolvedFile {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ResolvedFile {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ResolvedFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let path_buf = String::deserialize(deserializer)?;
        Self::new(path_buf).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPath(pub String);

impl ResolvedPath {
    pub fn new(path: String) -> Result<Self, String> {
        let meta = fs::metadata(&path)
            .map_err(|err| format!("failed to open path `{path}`: {err}"))?;

        if !meta.is_dir() {
            return Err(format!("`{path}` is not a valid path"));
        }

        Ok(ResolvedPath(path))
    }
}

impl Deref for ResolvedPath {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ResolvedPath {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ResolvedPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        let meta = fs::metadata(&path).map_err(|err| {
            serde::de::Error::custom(format!(
                "failed to open path `{path}`: {err}"
            ))
        })?;

        if !meta.is_dir() {
            return Err(serde::de::Error::custom(format!(
                "`{path}` is not a valid path"
            )));
        }

        Ok(ResolvedPath(path))
    }
}

struct NumberSocketAddrVisitor;

#[inline]
fn visit_any_n<T, E>(v: T) -> Result<SocketAddr, E>
where
    T: TryInto<u16>,
    E: serde::de::Error,
{
    let v = v.try_into().map_err(|_| {
        serde::de::Error::custom(format!(
            "must be a string-formated socket address or a number"
        ))
    })?;

    Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), v))
}

impl<'de> Visitor<'de> for NumberSocketAddrVisitor {
    type Value = SocketAddr;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("a socket address or a number")
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_any_n(v)
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_any_n(v)
    }

    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_any_n(v)
    }
}

pub fn deserialize_socket_addr<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SocketAddr, D::Error> {
    deserializer.deserialize_any(NumberSocketAddrVisitor)
}
