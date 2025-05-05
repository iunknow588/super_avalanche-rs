use std::io::{self, Error, ErrorKind};

use url::Url;

#[allow(clippy::type_complexity)]
/// 从URL字符串中提取scheme、host、port、path和chain alias。
///
/// # Errors
///
/// 如果URL解析失败，则返回错误。
pub fn extract_scheme_host_port_path_chain_alias(
    s: &str,
) -> io::Result<(
    Option<String>, // scheme
    String,         // host
    Option<u16>,    // port
    Option<String>, // URL path
    Option<String>, // chain alias
)> {
    if !s.starts_with("http://") && !s.starts_with("https://") {
        let (_, host, port, path, chain_alias) = parse_url(format!("http://{s}").as_str())?;
        return Ok((None, host, port, path, chain_alias));
    }
    parse_url(s)
}

#[allow(clippy::type_complexity)]
/// 解析URL字符串，提取scheme、host、port、path和chain alias。
///
/// # Errors
///
/// 如果URL解析失败，则返回错误。
fn parse_url(
    s: &str,
) -> io::Result<(
    Option<String>,
    String,
    Option<u16>,
    Option<String>,
    Option<String>,
)> {
    let url = Url::parse(s)
        .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("failed Url::parse '{e}'")))?;

    let host = if let Some(hs) = url.host_str() {
        hs.to_string()
    } else {
        return Err(Error::new(ErrorKind::InvalidInput, "no host found"));
    };

    let port = url.port();

    let (path, chain_alias) = if url.path().is_empty() || url.path() == "/" {
        (None, None)
    } else {
        // e.g., "/ext/bc/C/rpc"
        url.path_segments().map_or_else(
            || (Some(url.path().to_string()), None),
            |mut path_segments| {
                let _ext = path_segments.next();
                let _bc = path_segments.next();
                let chain_alias = path_segments.next();
                chain_alias.map_or_else(
                    || (Some(url.path().to_string()), None),
                    |ca| (Some(url.path().to_string()), Some(ca.to_string())),
                )
            },
        )
    };

    Ok((
        Some(url.scheme().to_string()),
        host,
        port,
        path,
        chain_alias,
    ))
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- utils::urls::test_extract_scheme_host_port_path_chain_alias --exact --show-output
#[test]
fn test_extract_scheme_host_port_path_chain_alias() {
    // 将测试拆分为多个函数，以降低复杂度
    test_basic_urls();
    test_chain_urls();
}

/// 测试基本URL解析
#[allow(dead_code)]
#[allow(clippy::cognitive_complexity)]
fn test_basic_urls() {
    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://localhost:9650").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "localhost");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("localhost:9650").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "localhost");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://abc:9650").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "abc");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("abc:9650").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "abc");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://127.0.0.1:9650").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("127.0.0.1:9650").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());
}

/// 测试带有链ID的URL解析
#[allow(dead_code)]
fn test_chain_urls() {
    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://127.0.0.1:9650/ext/bc/C/rpc").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert_eq!(path.unwrap(), "/ext/bc/C/rpc");
    assert_eq!(chain_alias.unwrap(), "C");

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("127.0.0.1:9650/ext/bc/C/rpc").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert_eq!(path.unwrap(), "/ext/bc/C/rpc");
    assert_eq!(chain_alias.unwrap(), "C");

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("1.2.3.4:1/ext/bc/abcde/rpc").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "1.2.3.4");
    assert_eq!(port.unwrap(), 1);
    assert_eq!(path.unwrap(), "/ext/bc/abcde/rpc");
    assert_eq!(chain_alias.unwrap(), "abcde");
}
