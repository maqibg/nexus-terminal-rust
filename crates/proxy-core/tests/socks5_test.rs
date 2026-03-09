mod common;

use proxy_core::connect_via_proxy;

#[tokio::test]
async fn socks5_malicious_atyp_returns_error() {
    let (listener, addr) = common::bind_mock_server().await;
    // 协商响应: [0x05, 0x00] (无认证)
    // 连接响应: [0x05, 0x00, 0x00, 0xFF] (ATYP=0xFF 非法)
    let response = vec![0x05u8, 0x00, 0x05, 0x00, 0x00, 0xFF];
    common::serve_one_response(listener, response);

    let result = connect_via_proxy(
        "SOCKS5",
        "127.0.0.1",
        addr.port(),
        None,
        None,
        "example.com",
        80,
    )
    .await;

    assert!(result.is_err(), "非法 ATYP 应返回错误");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("不支持的地址类型"),
        "错误应说明 ATYP 非法: {msg}"
    );
}

#[tokio::test]
async fn socks5_domain_zero_length_no_panic() {
    let (listener, addr) = common::bind_mock_server().await;
    // 协商OK，连接响应域名长度=0: [VER=5, REP=0, RSV=0, ATYP=3, LEN=0, PORT=0x0050]
    let response = vec![0x05u8, 0x00, 0x05, 0x00, 0x00, 0x03, 0x00, 0x00, 0x50];
    common::serve_one_response(listener, response);

    // 不能 panic，可能因连接关闭返回 I/O 错误
    let _ = connect_via_proxy(
        "SOCKS5",
        "127.0.0.1",
        addr.port(),
        None,
        None,
        "example.com",
        80,
    )
    .await;
}

#[tokio::test]
async fn socks5_ipv4_atyp_success() {
    let (listener, addr) = common::bind_mock_server().await;
    // 协商: [0x05, 0x00], 连接响应 IPv4: [0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 80]
    let response = vec![0x05u8, 0x00, 0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 80];
    common::serve_one_response(listener, response);

    let result = connect_via_proxy(
        "SOCKS5",
        "127.0.0.1",
        addr.port(),
        None,
        None,
        "example.com",
        80,
    )
    .await;
    assert!(result.is_ok(), "IPv4 ATYP 应成功: {:?}", result.err());
}

#[tokio::test]
async fn socks5_ipv6_atyp_success() {
    let (listener, addr) = common::bind_mock_server().await;
    // 协商: [0x05, 0x00], 连接响应 IPv6: [0x05, 0x00, 0x00, 0x04, 16字节地址, 2字节端口]
    let mut response = vec![0x05u8, 0x00, 0x05, 0x00, 0x00, 0x04];
    response.extend_from_slice(&[0u8; 16]); // IPv6 地址
    response.extend_from_slice(&[0x00, 0x50]); // 端口 80
    common::serve_one_response(listener, response);

    let result = connect_via_proxy(
        "SOCKS5",
        "127.0.0.1",
        addr.port(),
        None,
        None,
        "example.com",
        80,
    )
    .await;
    assert!(result.is_ok(), "IPv6 ATYP 应成功: {:?}", result.err());
}
