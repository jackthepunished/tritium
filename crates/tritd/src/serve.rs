//! A small HTTP/1.1 server with SSE streaming.
//!
//! Hand-rolled on `std::net`, deliberately: batch-1 single-session serving on an
//! edge device does not justify an async runtime, and the eventual FPGA host
//! will be happier without one.
//!
//! Hand-rolled parsing is an attack surface, so the defaults are conservative:
//! loopback only, capped request line, headers and body, a read timeout, and
//! chunked transfer encoding refused rather than half-implemented.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use trit_core::sampler::SamplerParams;
use trit_core::tokenizer::Tokenizer;

use crate::stream::TokenStream;
use crate::Runtime;

const MAX_LINE: usize = 8 << 10;
const MAX_HEADERS: usize = 64;
const MAX_BODY: usize = 1 << 20;
const READ_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ServeOptions {
    pub addr: String,
    pub max_tokens: usize,
}

pub fn serve(rt: Runtime, opts: &ServeOptions) -> Result<()> {
    let listener = TcpListener::bind(&opts.addr)
        .with_context(|| format!("bind {}", opts.addr))?;
    let local = listener.local_addr()?;
    eprintln!(
        "tritd serving on http://{local}  backend={}  kernel={}",
        rt.backend_name, rt.kernel_name
    );
    eprintln!("  POST /v1/completions   GET /v1/models   GET /healthz   GET /metrics");

    // One model, one session at a time. Decoding is memory bound at batch 1, so
    // concurrent sessions would contend for the same bandwidth rather than add
    // throughput; serializing keeps latency predictable and the answer correct.
    let rt = Arc::new(Mutex::new(rt));
    let default_max = opts.max_tokens;

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept failed: {e}");
                continue;
            }
        };
        let rt = rt.clone();
        // A panic in one connection must not take the server down.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Err(e) = handle(stream, &rt, default_max) {
                eprintln!("connection error: {e:#}");
            }
        }));
    }
    Ok(())
}

struct Request {
    method: String,
    path: String,
    body: String,
}

fn read_request(stream: &TcpStream) -> Result<Request> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    let mut r = BufReader::new(stream);

    let mut line = String::new();
    r.by_ref().take(MAX_LINE as u64).read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    anyhow::ensure!(!method.is_empty(), "malformed request line");

    let mut content_length = 0usize;
    for _ in 0..MAX_HEADERS {
        let mut h = String::new();
        let n = r.by_ref().take(MAX_LINE as u64).read_line(&mut h)?;
        if n == 0 || h == "\r\n" || h == "\n" {
            break;
        }
        let lower = h.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
        anyhow::ensure!(
            !lower.starts_with("transfer-encoding:") || !lower.contains("chunked"),
            "chunked transfer encoding is not supported"
        );
    }
    anyhow::ensure!(content_length <= MAX_BODY, "request body too large ({content_length} bytes)");

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        r.read_exact(&mut body)?;
    }
    Ok(Request { method, path, body: String::from_utf8(body)? })
}

fn write_response(mut s: &TcpStream, status: &str, content_type: &str, body: &str) -> Result<()> {
    write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )?;
    s.flush()?;
    Ok(())
}

fn json_escape(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

fn handle(stream: TcpStream, rt: &Arc<Mutex<Runtime>>, default_max: usize) -> Result<()> {
    let req = match read_request(&stream) {
        Ok(r) => r,
        Err(e) => {
            let _ = write_response(&stream, "400 Bad Request", "text/plain", &format!("{e:#}\n"));
            return Ok(());
        }
    };

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/healthz") => write_response(&stream, "200 OK", "text/plain", "ok\n"),
        ("GET", "/v1/models") => {
            let g = rt.lock().unwrap();
            let body = format!(
                r#"{{"object":"list","data":[{{"id":"tritium","object":"model","backend":{},"kernel":{},"context":{}}}]}}"#,
                json_escape(&g.backend_name),
                json_escape(g.kernel_name),
                g.model.config().max_seq
            );
            write_response(&stream, "200 OK", "application/json", &body)
        }
        ("GET", "/metrics") => {
            let g = rt.lock().unwrap();
            let m = &g.model;
            let body = format!(
                "tritium_weight_bytes_per_token {}\n\
                 tritium_ternary_bytes {}\n\
                 tritium_lm_head_bytes {}\n\
                 tritium_context_max {}\n",
                m.weight_bytes(),
                m.ternary_bytes(),
                m.lm_head_bytes(),
                m.config().max_seq
            );
            write_response(&stream, "200 OK", "text/plain; version=0.0.4", &body)
        }
        ("POST", "/v1/completions") => completions(stream, rt, &req.body, default_max),
        _ => write_response(&stream, "404 Not Found", "text/plain", "not found\n"),
    }
}

#[derive(serde::Deserialize)]
struct CompletionRequest {
    prompt: String,
    #[serde(default)]
    max_tokens: Option<usize>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    top_k: Option<usize>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    stream: bool,
}

fn completions(
    mut stream: TcpStream,
    rt: &Arc<Mutex<Runtime>>,
    body: &str,
    default_max: usize,
) -> Result<()> {
    let req: CompletionRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            return write_response(&stream, "400 Bad Request", "text/plain", &format!("{e}\n"))
        }
    };

    let guard = rt.lock().unwrap();
    let params = SamplerParams {
        temperature: req.temperature.unwrap_or(0.0),
        top_p: req.top_p.unwrap_or(1.0),
        top_k: req.top_k.unwrap_or(0),
        repetition_penalty: 1.0,
        seed: req.seed.unwrap_or(0),
    };
    let ids = match guard.tokenizer.encode(&req.prompt, true) {
        Ok(i) => i,
        Err(e) => {
            drop(guard);
            return write_response(&stream, "400 Bad Request", "text/plain", &format!("{e:#}\n"));
        }
    };
    let max = req.max_tokens.unwrap_or(default_max);

    let mut ts = match TokenStream::new(guard.model.clone(), &guard.tokenizer, &params, &ids, max) {
        Ok(t) => t,
        Err(e) => {
            drop(guard);
            return write_response(&stream, "400 Bad Request", "text/plain", &format!("{e:#}\n"));
        }
    };

    if !req.stream {
        let mut text = String::new();
        for tok in ts.by_ref() {
            text.push_str(&tok?.text);
        }
        text.push_str(&ts.flush());
        let body = format!(
            r#"{{"object":"text_completion","choices":[{{"index":0,"text":{}}}]}}"#,
            json_escape(&text)
        );
        return write_response(&stream, "200 OK", "application/json", &body);
    }

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n\
         Connection: close\r\n\r\n"
    )?;
    stream.flush()?;
    for tok in ts.by_ref() {
        let t = tok?;
        if t.text.is_empty() {
            continue;
        }
        write!(
            stream,
            "data: {{\"choices\":[{{\"index\":0,\"text\":{}}}]}}\n\n",
            json_escape(&t.text)
        )?;
        // Flush per token: an SSE stream that arrives in one block at the end is
        // not a stream.
        stream.flush()?;
    }
    let tail = ts.flush();
    if !tail.is_empty() {
        write!(stream, "data: {{\"choices\":[{{\"index\":0,\"text\":{}}}]}}\n\n", json_escape(&tail))?;
    }
    write!(stream, "data: [DONE]\n\n")?;
    stream.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escaping_survives_quotes_and_newlines() {
        assert_eq!(json_escape("a\"b\nc"), "\"a\\\"b\\nc\"");
    }
}
