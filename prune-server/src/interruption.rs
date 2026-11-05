//! 压测器收到 SIGTERM（task-server 的 cancel、systemd 停单元）或 SIGINT 时只置一个标记；
//! 各循环看到标记就停下、照常返回，压测库由守卫在返回路上删掉（成功、失败、被打断都删）。

use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

extern "C" fn record_interruption(_signal: libc::c_int) {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

/// 装 SIGTERM 与 SIGINT 的处理。
pub fn install_termination_handlers() -> Result<(), String> {
    for signal in [libc::SIGTERM, libc::SIGINT] {
        let handler = record_interruption as extern "C" fn(libc::c_int);
        // SAFETY: 处理函数只写一个原子变量，是异步信号安全的；signal 的两个参数都是合法值。
        let previous = unsafe { libc::signal(signal, handler as libc::sighandler_t) };
        if previous == libc::SIG_ERR {
            return Err(format!(
                "装不上信号 {signal} 的处理：{}",
                std::io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}

pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}
