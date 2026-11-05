//! 服务的位置与地址。
//!
//! - TCP 地址：只读多机配置 `multi-host.env` 的 `PRUNE_SERVER_TCP_ADDRESS=<本机在有线直连段上的地址>:<端口>`，不读环境变量；配置里没有这一键由调用方报错，不补缺省。
//!   第二台的客户端连同一个地址。配置文件的位置照别的脚本：`SINGLEFS_MULTI_HOST_CONFIG` 设了照它，没设取编译时这个工程的上一层（仓根）的 `multi-host.env`
//!   （第二台在同一棵共享的树里跑，读的是同一份）。
//!   写法照模板 `multi-host.env.example` 文件头：一行一个 `KEY=值`，值不做 shell 展开，两头成对的引号去掉一层。
//! - socket：`${XDG_RUNTIME_DIR}/singlefs-prune-server/server.sock`（照 task-server：目录 700、socket 600，全路径至多 107 字节）。
//! - 状态目录：`~/.local/state/singlefs/prune-server`（照 task-server 的库放在 `~/.local/state/singlefs/` 下）；
//!   服务自己那一份库在它下面的 `library`（服务头一次起时建；布局在 `storage.rs` 文件头），
//!   故障日志是它下面的 `faults.log`，控制命令的停服务记录在它下面的 `stops/`。
use std::path::{Path, PathBuf};

pub const NETWORK_ADDRESS_KEY: &str = "PRUNE_SERVER_TCP_ADDRESS";
pub const MULTI_HOST_CONFIGURATION_VARIABLE: &str = "SINGLEFS_MULTI_HOST_CONFIG";
/// sockaddr_un 的 sun_path 是 108 字节，留一个给结尾的 0。
pub const MAXIMUM_SOCKET_PATH_BYTES: usize = 107;

pub fn default_multi_host_configuration_path() -> PathBuf {
    match std::env::var_os(MULTI_HOST_CONFIGURATION_VARIABLE) {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        Some(_) | None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("multi-host.env"),
    }
}

/// 取一个键的值；只读文件、不读环境变量：文件读不了是错，键不在回 `None`，由调用方报错、不补缺省。
pub fn read_configuration_value(
    configuration_path: &Path,
    key: &str,
) -> Result<Option<String>, String> {
    let text = std::fs::read_to_string(configuration_path)
        .map_err(|error| format!("读不了多机配置 {}：{error}", configuration_path.display()))?;
    Ok(configuration_value_in_text(&text, key))
}

fn configuration_value_in_text(text: &str, key: &str) -> Option<String> {
    let mut found = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((line_key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };
        if line_key.trim() == key {
            found = Some(strip_one_pair_of_quotes(raw_value.trim()).to_string());
        }
    }
    found
}

fn strip_one_pair_of_quotes(value: &str) -> &str {
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

pub fn default_socket_path() -> Result<PathBuf, String> {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(directory) if !directory.is_empty() => Ok(PathBuf::from(directory)
            .join("singlefs-prune-server")
            .join("server.sock")),
        Some(_) | None => Err("XDG_RUNTIME_DIR 没设，算不出缺省的 socket 路径".to_string()),
    }
}

fn home_directory(what: &str) -> Result<PathBuf, String> {
    match std::env::var_os("HOME") {
        Some(home) if !home.is_empty() => Ok(PathBuf::from(home)),
        Some(_) | None => Err(format!("HOME 没设，算不出缺省的{what}")),
    }
}

pub fn default_state_directory() -> Result<PathBuf, String> {
    Ok(home_directory("状态目录")?.join(".local/state/singlefs/prune-server"))
}

/// 状态目录里的库目录。
pub fn library_directory_in(state_directory: &Path) -> PathBuf {
    state_directory.join("library")
}

/// socket 全路径的字节数超了上限时回说明。
pub fn socket_path_problem(socket_path: &Path) -> Option<String> {
    let byte_count = socket_path.as_os_str().len();
    if byte_count > MAXIMUM_SOCKET_PATH_BYTES {
        return Some(format!(
            "socket 路径 {} 有 {byte_count} 字节，超过上限 {MAXIMUM_SOCKET_PATH_BYTES}",
            socket_path.display()
        ));
    }
    None
}
