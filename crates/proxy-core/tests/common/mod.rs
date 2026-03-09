use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// 绑定随机本地端口，返回监听器和其地址。
pub async fn bind_mock_server() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind mock TCP server");
    let addr = listener.local_addr().expect("failed to get local addr");
    (listener, addr)
}

/// 接受一个连接，向客户端发送 `response_bytes`，然后优雅关闭。
///
/// 优雅关闭：写完后发送 FIN（而非 RST），等客户端读完数据再退出。
/// 这样客户端的 read_exact 不会因为 OS error 10054（WSAEConnReset）而失败。
pub fn serve_one_response(
    listener: TcpListener,
    response_bytes: Vec<u8>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let _ = socket.write_all(&response_bytes).await;
            // 半关闭写端（发 FIN），让客户端能读完我们发出的数据
            let _ = socket.shutdown().await;
            // 排空客户端剩余发送，避免对方收到 RST
            let mut drain = [0u8; 256];
            loop {
                match socket.read(&mut drain).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        }
    })
}
