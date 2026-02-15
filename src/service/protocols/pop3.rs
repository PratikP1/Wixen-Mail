//! POP3 protocol client
//!
//! Handles POP3 protocol for receiving email.

use crate::common::Result;
use std::collections::{HashMap, HashSet};

/// POP3 client configuration
#[derive(Debug, Clone)]
pub struct Pop3Config {
    pub server: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
}

/// POP3 message metadata from LIST/UIDL style commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pop3MessageInfo {
    pub id: u32,
    pub size: usize,
    pub uidl: String,
}

/// POP3 full message payload
#[derive(Debug, Clone)]
pub struct Pop3Message {
    pub id: u32,
    pub info: Pop3MessageInfo,
    pub raw: String,
}

/// POP3 client
pub struct Pop3Client {
    config: Pop3Config,
}

impl Pop3Client {
    /// Create a new POP3 client
    pub fn new(config: Pop3Config) -> Result<Self> {
        Ok(Self { config })
    }

    /// Connect and authenticate (placeholder full command set)
    pub async fn connect(&self, _password: &str) -> Result<Pop3Session> {
        tracing::info!(
            "POP3 connection to {}:{} as {} (placeholder)",
            self.config.server,
            self.config.port,
            crate::common::logging::mask_email(&self.config.username)
        );
        Ok(Pop3Session::new(self.config.clone()))
    }
}

/// Active POP3 session
pub struct Pop3Session {
    #[allow(dead_code)]
    config: Pop3Config,
    connected: bool,
    deleted: HashSet<u32>,
    store: HashMap<u32, Pop3Message>,
}

impl Pop3Session {
    fn new(config: Pop3Config) -> Self {
        let mut store = HashMap::new();
        for id in 1..=3u32 {
            let raw = format!(
                "From: sender{}@example.com\r\nTo: {}\r\nSubject: POP3 Test Message {}\r\n\r\nMessage {} body.",
                id, config.username, id, id
            );
            let info = Pop3MessageInfo {
                id,
                size: raw.len(),
                uidl: format!("UIDL-{}", id),
            };
            store.insert(id, Pop3Message { id, info, raw });
        }
        Self {
            config,
            connected: true,
            deleted: HashSet::new(),
            store,
        }
    }

    fn ensure_connected(&self) -> Result<()> {
        if self.connected {
            Ok(())
        } else {
            Err(crate::common::Error::Protocol(
                "POP3 session is not connected".to_string(),
            ))
        }
    }

    /// STAT: returns number of undeleted messages and total octets.
    pub async fn stat(&self) -> Result<(usize, usize)> {
        self.ensure_connected()?;
        let mut count = 0usize;
        let mut size = 0usize;
        for msg in self.store.values() {
            if !self.deleted.contains(&msg.id) {
                count += 1;
                size += msg.info.size;
            }
        }
        Ok((count, size))
    }

    /// LIST: metadata for undeleted messages.
    pub async fn list(&self) -> Result<Vec<Pop3MessageInfo>> {
        self.ensure_connected()?;
        let mut result: Vec<_> = self
            .store
            .values()
            .filter(|m| !self.deleted.contains(&m.id))
            .map(|m| m.info.clone())
            .collect();
        result.sort_by_key(|m| m.id);
        Ok(result)
    }

    /// UIDL map for undeleted messages.
    pub async fn uidl(&self) -> Result<Vec<(u32, String)>> {
        let result = self
            .list()
            .await?
            .into_iter()
            .map(|m| (m.id, m.uidl))
            .collect::<Vec<_>>();
        Ok(result)
    }

    /// RETR: full raw message.
    pub async fn retr(&self, id: u32) -> Result<Pop3Message> {
        self.ensure_connected()?;
        if self.deleted.contains(&id) {
            return Err(crate::common::Error::Protocol(format!(
                "Message {} marked for deletion",
                id
            )));
        }
        self.store
            .get(&id)
            .cloned()
            .ok_or_else(|| crate::common::Error::Protocol(format!("Message {} not found", id)))
    }

    /// TOP: returns header and up to N body lines.
    pub async fn top(&self, id: u32, body_lines: usize) -> Result<String> {
        let message = self.retr(id).await?;
        let mut split = message.raw.splitn(2, "\r\n\r\n");
        let header = split.next().unwrap_or_default();
        let body = split.next().unwrap_or_default();
        let selected = body
            .split("\r\n")
            .take(body_lines)
            .collect::<Vec<_>>()
            .join("\r\n");
        Ok(format!("{}\r\n\r\n{}", header, selected))
    }

    /// DELE: marks message as deleted.
    pub async fn dele(&mut self, id: u32) -> Result<()> {
        self.ensure_connected()?;
        if !self.store.contains_key(&id) {
            return Err(crate::common::Error::Protocol(format!(
                "Message {} not found",
                id
            )));
        }
        self.deleted.insert(id);
        Ok(())
    }

    /// RSET: clears deletion marks.
    pub async fn rset(&mut self) -> Result<()> {
        self.ensure_connected()?;
        self.deleted.clear();
        Ok(())
    }

    /// NOOP
    pub async fn noop(&self) -> Result<()> {
        self.ensure_connected()?;
        Ok(())
    }

    /// QUIT: commits deletions and closes session.
    pub async fn quit(mut self) -> Result<()> {
        self.ensure_connected()?;
        for id in &self.deleted {
            self.store.remove(id);
        }
        self.connected = false;
        Ok(())
    }

    /// Convenience: retrieve all undeleted full messages.
    pub async fn retrieve_messages(&self) -> Result<Vec<Pop3Message>> {
        let mut result = Vec::new();
        for info in self.list().await? {
            result.push(self.retr(info.id).await?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop3_client_creation() {
        let config = Pop3Config {
            server: "pop3.example.com".to_string(),
            port: 995,
            use_tls: true,
            username: "test@example.com".to_string(),
        };
        let client = Pop3Client::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_pop3_stat_list_retr() {
        let config = Pop3Config {
            server: "pop3.example.com".to_string(),
            port: 995,
            use_tls: true,
            username: "test@example.com".to_string(),
        };
        let client = Pop3Client::new(config).unwrap();
        let session = client.connect("password").await.unwrap();
        let (count, bytes) = session.stat().await.unwrap();
        assert_eq!(count, 3);
        assert!(bytes > 0);
        let list = session.list().await.unwrap();
        assert_eq!(list.len(), 3);
        let msg = session.retr(list[0].id).await.unwrap();
        assert!(msg.raw.contains("Subject: POP3 Test Message"));
    }

    #[tokio::test]
    async fn test_pop3_dele_rset_uidl_top() {
        let config = Pop3Config {
            server: "pop3.example.com".to_string(),
            port: 995,
            use_tls: true,
            username: "test@example.com".to_string(),
        };
        let client = Pop3Client::new(config).unwrap();
        let mut session = client.connect("password").await.unwrap();
        let uidl = session.uidl().await.unwrap();
        assert_eq!(uidl.len(), 3);
        let top = session.top(1, 1).await.unwrap();
        assert!(top.contains("Subject: POP3 Test Message 1"));
        session.dele(1).await.unwrap();
        assert_eq!(session.list().await.unwrap().len(), 2);
        session.rset().await.unwrap();
        assert_eq!(session.list().await.unwrap().len(), 3);
    }
}
