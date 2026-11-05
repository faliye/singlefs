//! TCP keepalive：服务与客户端两头的 TCP 连接都开，半开的连接（对端掉电、网线拔了、中间设备丢了状态）由内核在
//! 空闲 `idle` 之后每隔 `interval` 探一次、连探 `count` 次没回就关掉，读写随即报错，客户端照重连的路走。
//! 自证 half-open-connection-recovered 那一格读回内核里的设置核它真开了。

use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeepaliveSettings {
    pub idle: Duration,
    pub interval: Duration,
    pub count: u32,
}

impl Default for KeepaliveSettings {
    /// 空闲 10 秒起探、每 5 秒一次、连 3 次：半开的连接约 25 秒认出来。
    fn default() -> KeepaliveSettings {
        KeepaliveSettings {
            idle: Duration::from_secs(10),
            interval: Duration::from_secs(5),
            count: 3,
        }
    }
}

fn whole_seconds(duration: Duration, what: &str) -> Result<libc::c_int, String> {
    libc::c_int::try_from(duration.as_secs().max(1))
        .map_err(|_| format!("keepalive 的{what} {} 秒放不进 c_int", duration.as_secs()))
}

fn set_option(
    stream: &TcpStream,
    level: libc::c_int,
    name: libc::c_int,
    value: libc::c_int,
    what: &str,
) -> Result<(), String> {
    let size = libc::socklen_t::try_from(std::mem::size_of::<libc::c_int>())
        .expect("c_int 的字节数放得进 socklen_t");
    // SAFETY: 描述符来自活着的 TcpStream；value 是栈上的 c_int，指针与长度一致，内核只读它。
    let result = unsafe {
        libc::setsockopt(
            stream.as_raw_fd(),
            level,
            name,
            std::ptr::from_ref(&value).cast::<libc::c_void>(),
            size,
        )
    };
    if result != 0 {
        return Err(format!(
            "设不上 {what}：{}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn get_option(
    stream: &TcpStream,
    level: libc::c_int,
    name: libc::c_int,
    what: &str,
) -> Result<libc::c_int, String> {
    let mut value: libc::c_int = 0;
    let mut size = libc::socklen_t::try_from(std::mem::size_of::<libc::c_int>())
        .expect("c_int 的字节数放得进 socklen_t");
    // SAFETY: 描述符来自活着的 TcpStream；value 与 size 是栈上的变量，指针与长度一致，内核写回不超过 size 字节。
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            level,
            name,
            std::ptr::from_mut(&mut value).cast::<libc::c_void>(),
            &mut size,
        )
    };
    if result != 0 {
        return Err(format!(
            "读不出 {what}：{}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(value)
}

pub fn enable_keepalive(stream: &TcpStream, settings: KeepaliveSettings) -> Result<(), String> {
    set_option(
        stream,
        libc::SOL_SOCKET,
        libc::SO_KEEPALIVE,
        1,
        "SO_KEEPALIVE",
    )?;
    set_option(
        stream,
        libc::IPPROTO_TCP,
        libc::TCP_KEEPIDLE,
        whole_seconds(settings.idle, "空闲时长")?,
        "TCP_KEEPIDLE",
    )?;
    set_option(
        stream,
        libc::IPPROTO_TCP,
        libc::TCP_KEEPINTVL,
        whole_seconds(settings.interval, "探测间隔")?,
        "TCP_KEEPINTVL",
    )?;
    let count = libc::c_int::try_from(settings.count)
        .map_err(|_| format!("keepalive 的探测次数 {} 放不进 c_int", settings.count))?;
    set_option(
        stream,
        libc::IPPROTO_TCP,
        libc::TCP_KEEPCNT,
        count,
        "TCP_KEEPCNT",
    )
}

/// 从内核读回这条连接的 keepalive 设置；没开回 `None`。
pub fn read_keepalive(stream: &TcpStream) -> Result<Option<KeepaliveSettings>, String> {
    if get_option(stream, libc::SOL_SOCKET, libc::SO_KEEPALIVE, "SO_KEEPALIVE")? == 0 {
        return Ok(None);
    }
    let seconds = |value: libc::c_int| Duration::from_secs(u64::try_from(value).unwrap_or(0));
    let idle = seconds(get_option(
        stream,
        libc::IPPROTO_TCP,
        libc::TCP_KEEPIDLE,
        "TCP_KEEPIDLE",
    )?);
    let interval = seconds(get_option(
        stream,
        libc::IPPROTO_TCP,
        libc::TCP_KEEPINTVL,
        "TCP_KEEPINTVL",
    )?);
    let count = u32::try_from(get_option(
        stream,
        libc::IPPROTO_TCP,
        libc::TCP_KEEPCNT,
        "TCP_KEEPCNT",
    )?)
    .unwrap_or(0);
    Ok(Some(KeepaliveSettings {
        idle,
        interval,
        count,
    }))
}
