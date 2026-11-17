use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use serde_json::Value;

use crate::dto::BlenderResponse;

pub struct BlenderClient {
    stream: TcpStream,
}

impl BlenderClient {
    pub fn connect(host: &str, port: u16, timeout: Duration) -> Result<Self> {
        let addrs: Vec<_> = (host, port)
            .to_socket_addrs()
            .with_context(|| format!("cannot resolve {host}:{port}"))?
            .collect();

        // "localhost" may resolve to ::1 first while the addon only listens on
        // 127.0.0.1, so try every address before giving up.
        let mut last_err = None;
        for addr in &addrs {
            match TcpStream::connect_timeout(addr, timeout) {
                Ok(stream) => {
                    stream.set_read_timeout(Some(timeout))?;
                    stream.set_write_timeout(Some(timeout))?;
                    stream.set_nodelay(true)?;
                    return Ok(Self { stream });
                }
                Err(e) => last_err = Some(e),
            }
        }
        let err = last_err.map_or_else(|| anyhow!("no addresses found"), Into::into);
        Err(err.context(format!(
            "cannot connect to Blender at {host}:{port} (is the addon's server running?)"
        )))
    }

    /// Send a request and return its `result` payload.
    ///
    /// `request` is usually a [`BlenderCommand`]; any value that serializes to
    /// `{"type": ..., "params": ...}` works, which is how untyped commands are sent.
    pub fn send(&mut self, request: &impl Serialize) -> Result<Value> {
        self.stream.write_all(&serde_json::to_vec(request)?)?;

        match self.read_response()? {
            BlenderResponse::Success { result } => {
                // Some handlers report failure as {"status":"success","result":{"error":...}}.
                if let Some(err) = result.get("error").and_then(Value::as_str) {
                    bail!("Blender returned an error: {err}");
                }
                Ok(result)
            }
            BlenderResponse::Error { message } => {
                // execute_code packs {"exception_type","message","traceback"}
                // into the message as a JSON string; show the traceback.
                if let Ok(detail) = serde_json::from_str::<Value>(&message)
                    && let Some(tb) = detail.get("traceback").and_then(Value::as_str)
                {
                    bail!("Python error in Blender:\n{}", tb.trim_end());
                }
                bail!("Blender returned an error: {message}")
            }
        }
    }

    /// Read until the bytes received parse as one complete reply. The addon
    /// sends no delimiter, so "incomplete JSON" means "keep reading".
    fn read_response(&mut self) -> Result<BlenderResponse> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let n = match self.stream.read(&mut chunk) {
                Ok(n) => n,
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                    bail!("timed out waiting for Blender to respond")
                }
                Err(e) => return Err(e.into()),
            };
            if n == 0 {
                bail!("connection closed by Blender before a full response arrived");
            }
            buf.extend_from_slice(&chunk[..n]);
            match serde_json::from_slice::<BlenderResponse>(&buf) {
                Ok(v) => return Ok(v),
                Err(e) if e.is_eof() => continue, // incomplete, keep reading
                // Bad JSON, or a reply that is not {"status":"success"|"error",...}.
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("unexpected response from Blender: {}", String::from_utf8_lossy(&buf))
                    });
                }
            }
        }
    }
}
