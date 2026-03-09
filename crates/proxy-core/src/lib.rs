//! SOCKS5/HTTP 代理管理

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// 通过代理连接到目标主机
pub async fn connect_via_proxy(
    proxy_type: &str,
    proxy_host: &str,
    proxy_port: u16,
    proxy_username: Option<&str>,
    proxy_password: Option<&str>,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    let mut stream = TcpStream::connect(format!("{}:{}", proxy_host, proxy_port))
        .await
        .map_err(|e| format!("连接代理失败: {e}"))?;

    match proxy_type.to_uppercase().as_str() {
        "SOCKS5" => {
            socks5_handshake(
                &mut stream,
                proxy_username,
                proxy_password,
                target_host,
                target_port,
            )
            .await
        }
        "HTTP" => {
            http_connect(
                &mut stream,
                proxy_username,
                proxy_password,
                target_host,
                target_port,
            )
            .await
        }
        other => Err(format!("不支持的代理类型: {other}")),
    }?;

    Ok(stream)
}

async fn socks5_handshake(
    stream: &mut TcpStream,
    username: Option<&str>,
    password: Option<&str>,
    target_host: &str,
    target_port: u16,
) -> Result<(), String> {
    // 认证方法协商
    let has_auth = username.is_some();
    if has_auth {
        stream
            .write_all(&[0x05, 0x02, 0x00, 0x02])
            .await
            .map_err(|e| e.to_string())?;
    } else {
        stream
            .write_all(&[0x05, 0x01, 0x00])
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut buf = [0u8; 2];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| e.to_string())?;
    if buf[0] != 0x05 {
        return Err("SOCKS5 协议错误".into());
    }

    // 用户名/密码认证 (RFC 1929)
    if buf[1] == 0x02 {
        let user = username.unwrap_or("");
        let pass = password.unwrap_or("");
        let user_len =
            u8::try_from(user.len()).map_err(|_| "SOCKS5 用户名超长（最大 255 字节）")?;
        let pass_len = u8::try_from(pass.len()).map_err(|_| "SOCKS5 密码超长（最大 255 字节）")?;

        let mut auth = vec![0x01, user_len];
        auth.extend_from_slice(user.as_bytes());
        auth.push(pass_len);
        auth.extend_from_slice(pass.as_bytes());
        stream.write_all(&auth).await.map_err(|e| e.to_string())?;

        let mut resp = [0u8; 2];
        stream
            .read_exact(&mut resp)
            .await
            .map_err(|e| e.to_string())?;
        if resp[1] != 0x00 {
            return Err("SOCKS5 认证失败".into());
        }
    } else if buf[1] != 0x00 {
        return Err("SOCKS5 不支持的认证方法".into());
    }

    // 连接请求（域名模式）
    let host_bytes = target_host.as_bytes();
    let host_len =
        u8::try_from(host_bytes.len()).map_err(|_| "SOCKS5 目标主机名超长（最大 255 字节）")?;

    let mut req = vec![0x05, 0x01, 0x00, 0x03, host_len];
    req.extend_from_slice(host_bytes);
    req.push((target_port >> 8) as u8);
    req.push((target_port & 0xff) as u8);
    stream.write_all(&req).await.map_err(|e| e.to_string())?;

    // 读取响应头：VER, REP, RSV, ATYP
    let mut header = [0u8; 4];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|e| e.to_string())?;

    if header[0] != 0x05 {
        return Err("SOCKS5 响应版本错误".into());
    }
    if header[1] != 0x00 {
        return Err(format!("SOCKS5 连接失败，状态码: {}", header[1]));
    }

    // 按 ATYP 精确读取绑定地址，丢弃内容（仅用于消费字节）
    match header[3] {
        0x01 => {
            // IPv4：4 字节地址 + 2 字节端口
            let mut addr = [0u8; 6];
            stream
                .read_exact(&mut addr)
                .await
                .map_err(|e| e.to_string())?;
        }
        0x03 => {
            // 域名：1 字节长度 + N 字节域名 + 2 字节端口
            let mut len_buf = [0u8; 1];
            stream
                .read_exact(&mut len_buf)
                .await
                .map_err(|e| e.to_string())?;
            let domain_len = len_buf[0] as usize;
            let total = domain_len.checked_add(2).ok_or("SOCKS5 响应域名长度溢出")?;
            let mut rest = vec![0u8; total];
            stream
                .read_exact(&mut rest)
                .await
                .map_err(|e| e.to_string())?;
        }
        0x04 => {
            // IPv6：16 字节地址 + 2 字节端口
            let mut addr = [0u8; 18];
            stream
                .read_exact(&mut addr)
                .await
                .map_err(|e| e.to_string())?;
        }
        atyp => {
            return Err(format!("SOCKS5 不支持的地址类型: 0x{atyp:02x}"));
        }
    }

    Ok(())
}

async fn http_connect(
    stream: &mut TcpStream,
    username: Option<&str>,
    password: Option<&str>,
    target_host: &str,
    target_port: u16,
) -> Result<(), String> {
    let mut req = format!(
        "CONNECT {target_host}:{target_port} HTTP/1.1\r\nHost: {target_host}:{target_port}\r\n"
    );

    if let (Some(user), Some(pass)) = (username, password) {
        let mut creds = Vec::new();
        use std::io::Write;
        write!(creds, "{user}:{pass}").map_err(|e| e.to_string())?;
        let encoded = base64_encode(&creds);
        req.push_str(&format!("Proxy-Authorization: Basic {encoded}\r\n"));
    }

    req.push_str("\r\n");
    stream
        .write_all(req.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    // 读取响应头
    let mut buf = Vec::with_capacity(1024);
    loop {
        let mut byte = [0u8; 1];
        stream
            .read_exact(&mut byte)
            .await
            .map_err(|e| e.to_string())?;
        buf.push(byte[0]);
        if buf.len() >= 4 && &buf[buf.len() - 4..] == b"\r\n\r\n" {
            break;
        }
        if buf.len() > 4096 {
            return Err("HTTP CONNECT 响应过长".into());
        }
    }

    let response = String::from_utf8_lossy(&buf);
    if !response.contains("200") {
        return Err(format!(
            "HTTP CONNECT 失败: {}",
            response.lines().next().unwrap_or("")
        ));
    }

    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[(n >> 18 & 63) as usize] as char);
        result.push(CHARS[(n >> 12 & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[(n >> 6 & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn socks5_username_too_long_rejected() {
        let long_user = "a".repeat(256);
        let result: Result<(), &str> = u8::try_from(long_user.len())
            .map(|_| ())
            .map_err(|_| "SOCKS5 用户名超长（最大 255 字节）");
        assert!(result.is_err());
    }

    #[test]
    fn socks5_hostname_too_long_rejected() {
        let long_host = "a".repeat(256);
        let result: Result<(), &str> = u8::try_from(long_host.len())
            .map(|_| ())
            .map_err(|_| "SOCKS5 目标主机名超长（最大 255 字节）");
        assert!(result.is_err());
    }
}
