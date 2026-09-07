//! Cookie 持久化存储
//!
//! 支持从字符串解析 Cookie 并持久化到 JSON 文件，
//! 以及将 Cookie 注入到 Chromium 浏览器页面。

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;

/// Cookie 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieEntry {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: Option<String>,
}

/// 将 "name=value; Domain=xxx; Path=/" 格式字符串解析为 `CookieEntry`
#[must_use]
pub fn parse_cookie_str(s: &str) -> Option<CookieEntry> {
    let parts: Vec<&str> = s.split(';').map(str::trim).collect();
    let first_eq = parts.first()?;
    let (name, value) = first_eq.split_once('=')?;
    let mut domain = String::new();
    let mut path: Option<String> = None;
    for part in &parts[1..] {
        if let Some((k, v)) = part.split_once('=') {
            match k.trim().to_lowercase().as_str() {
                "domain" => domain = v.trim().to_string(),
                "path" => path = Some(v.trim().to_string()),
                _ => {}
            }
        }
    }
    Some(CookieEntry {
        name: name.trim().to_string(),
        value: value.trim().to_string(),
        domain,
        path,
    })
}

/// Cookie 存储 — 持久化到 JSON 文件
#[derive(Clone)]
pub struct CookieStore {
    path: PathBuf,
    cookies: Vec<CookieEntry>,
}

impl CookieStore {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        let cookies = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };
        Self { path, cookies }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.cookies)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    #[must_use]
    pub fn cookies(&self) -> &[CookieEntry] {
        &self.cookies
    }

    pub fn set_cookies(&mut self, entries: Vec<CookieEntry>) -> anyhow::Result<()> {
        self.cookies = entries;
        self.save()
    }
}

/// 线程安全的 Cookie 存储包装
pub struct SharedCookieStore {
    inner: Mutex<CookieStore>,
}

impl SharedCookieStore {
    #[must_use]
    pub const fn new(store: CookieStore) -> Self {
        Self {
            inner: Mutex::new(store),
        }
    }

    pub fn cookies(&self) -> Vec<CookieEntry> {
        self.inner.lock().cookies().to_vec()
    }

    pub fn set_cookies(&self, entries: Vec<CookieEntry>) -> anyhow::Result<()> {
        self.inner.lock().set_cookies(entries)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.inner.lock().save()
    }
}
