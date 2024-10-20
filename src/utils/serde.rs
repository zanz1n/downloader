use std::{
    fmt::Debug,
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Deref,
    path::{Path, PathBuf},
};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedFile(String);

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
pub struct ResolvedPath(String);

impl ResolvedPath {
    pub fn new(path: String) -> Result<Self, String> {
        let meta = fs::metadata(&path)
            .map_err(|err| format!("failed to open path `{path}`: {err}"))?;

        if !meta.is_dir() {
            return Err(format!("`{path}` is not a valid path"));
        }

        Ok(ResolvedPath(path))
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut new = PathBuf::from(&self.0);
        new.push(path);
        new
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

pub mod duration_secs {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[inline]
    pub fn serialize<S: Serializer>(
        duration: &Duration,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

pub mod base64 {
    use base64::{prelude::BASE64_STANDARD_NO_PAD as BASE64, Engine};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[inline]
    pub fn serialize<S: Serializer>(
        slice: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        BASE64.encode(slice).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        BASE64.decode(s).map_err(|err| {
            serde::de::Error::custom(format!(
                "failed to decode base64 string: {err}"
            ))
        })
    }
}
