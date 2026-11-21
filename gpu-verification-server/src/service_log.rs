//! 服务进程的日志：验证包逐次派活打的两种行不交给日志，服务自己按块、按卡打汇总行（`crate::serving_from_the_library` 文件头「日志」）。
//!
//! 验证包里的 GPU 判器每次派活往标准错误打 `GPU_LIFECYCLE`（每一遍提交前后、等待前后各一行）与 `CRASH_JUDGE_ACCOUNTING_SLICE`、
//! `CRASH_JUDGE_ACCOUNTING_SLICES`（每片一行、每次派活一行），满速时每秒几千行；用户级单元的日志由 journald 按整个用户管理器限流，
//! 超过就整窗丢，连别的行一起丢。
//! 所以在服务进程里滤：起来头一件事把标准错误（fd 2）换成一条管道的写端，一条线程读管道，逐次派活的两种行只计数（[`DISPATCH_LINES_DROPPED`]）、
//! 不转，别的行原样转给原来的标准错误（[`route_the_standard_error_through_the_filter`]）；判法是 [`filter_the_service_log`]。
//!
//! 换 fd 2 只用标准库（这个包不另加依赖：加了改仓根 `Cargo.lock`，验证包的构建脚本读它）：先开管道、核 `/proc/self/fd/<写端>` 开得了，
//! 再关掉 fd 2、从那个路径再开一份，这时最小的空号就是 2。这一步要在起任何线程之前做（`main` 的头一步），之后不再动。代价：
//! - fd 2 带 close-on-exec：这个进程起的子进程若照继承标准错误，它的标准错误是关着的（验证包起的 `cargo -V`、`rustc -vV` 用 `output()` 收，不受影响）；
//! - 进程退出时管道里还没转出去的行会丢：`main` 收尾时关掉 fd 2、等过滤线程读完（[`RoutedStandardError::finish`]，至多等 [`FILTER_DRAIN_LIMIT`]）；
//!   panic 的那一句由 panic 钩子直接写原来的标准错误、不经管道；被信号打死时管道里的行照丢；
//! - 服务日志里看不到逐次派活的时刻（卡住时最后进入的阶段也看不到）：要逐次派活的现场，在测试进程里跑（测试与门禁的日志照旧有这两种行）。
//!
//! 退出码：关掉 fd 2 之后再开不出来（核过开得了，只在核与开之间管道没了时走到）时往原来的标准错误写一句、退 1。

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::fd::{AsFd, AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

use crate::break_switches::{BreakSwitch, BreakSwitches};

/// 验证包逐次派活打的两种行的开头（`CRASH_JUDGE_ACCOUNTING_SLICE` 也罩住 `CRASH_JUDGE_ACCOUNTING_SLICES`）。
pub const DISPATCH_LINE_PREFIXES: [&str; 2] = ["GPU_LIFECYCLE ", "CRASH_JUDGE_ACCOUNTING_SLICE"];
/// 这个进程里滤掉了几行（按卡汇总那一段的行里报，`crate::serving_from_the_library`）。
pub static DISPATCH_LINES_DROPPED: AtomicU64 = AtomicU64::new(0);
/// 收尾时等过滤线程把管道里的行转完的上限：管道至多几十 KB，转给 journald 不到一秒。
pub const FILTER_DRAIN_LIMIT: Duration = Duration::from_secs(2);
/// 关掉 fd 2 之后再开不出来时的退出码（启动失败）。
const STANDARD_ERROR_LOST_EXIT: i32 = 1;
const STANDARD_ERROR_DESCRIPTOR: i32 = 2;
/// 关掉 fd 2 之前要开着的两个号：它们开着，再开的那一份才落在 2 上。
const STANDARD_INPUT_AND_OUTPUT_DESCRIPTORS: [i32; 2] = [0, 1];

/// 这一行是不是验证包逐次派活打的那两种之一（行首逐字比）。
#[must_use]
pub fn is_a_dispatch_line(line: &[u8]) -> bool {
    DISPATCH_LINE_PREFIXES
        .iter()
        .any(|prefix| line.starts_with(prefix.as_bytes()))
}

/// 读 `input` 到尽头：逐次派活的行只在 `dropped` 上加一、不转；别的行原样（连行尾）写进 `forward`。
/// 写不进 `forward` 的不管、接着读：不读，管道满了，往标准错误写的线程全堵住。弄坏开关 dispatch-lines-forwarded 下逐次派活的行也照转。
///
/// # Errors
/// 读 `input` 失败。
pub fn filter_the_service_log(
    mut input: impl BufRead,
    forward: &mut impl Write,
    dropped: &AtomicU64,
    break_switches: &BreakSwitches,
) -> std::io::Result<()> {
    let forwards_the_dispatch_lines = break_switches.is_on(BreakSwitch::DispatchLinesForwarded);
    let mut line = Vec::new();
    // 迭代上界：input 的行数；读到尽头（写端全关了）返回
    loop {
        line.clear();
        if input.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }
        if is_a_dispatch_line(&line) && !forwards_the_dispatch_lines {
            dropped.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        let _ = forward.write_all(&line);
    }
}

/// 换过的标准错误：收尾时等过滤线程读完。
pub struct RoutedStandardError {
    filter_finished: Receiver<()>,
}

impl RoutedStandardError {
    /// 关掉 fd 2（管道最后一个写端），等过滤线程把剩下的行转完，至多 [`FILTER_DRAIN_LIMIT`]。之后这个进程不再往标准错误写。
    /// 只在 `main` 返回之前、别的线程都收了之后调。
    pub fn finish(self) {
        // SAFETY: fd 2 是 route_the_standard_error_through_the_filter 开的那份管道写端，交出去时 into_raw_fd 了，没有别的 OwnedFd 拿着它；
        // 调用方保证此刻只剩主线程、关掉之后没人再写它。
        drop(unsafe { OwnedFd::from_raw_fd(STANDARD_ERROR_DESCRIPTOR) });
        let _ = self.filter_finished.recv_timeout(FILTER_DRAIN_LIMIT);
    }
}

/// 把标准错误换成过滤线程读的管道（文件头）。必须在起任何线程之前调。
///
/// # Errors
/// 标准输入或标准输出关着、复制不了标准错误、开不了管道或管道的写端、起不了过滤线程：fd 2 还没动或已换好，交回原因。
/// 关掉 fd 2 之后再开不出来不交回：往原来的标准错误写一句、退 1（文件头「退出码」）。
pub fn route_the_standard_error_through_the_filter() -> Result<RoutedStandardError, String> {
    for descriptor in STANDARD_INPUT_AND_OUTPUT_DESCRIPTORS {
        let path = format!("/proc/self/fd/{descriptor}");
        if !Path::new(&path).exists() {
            return Err(format!(
                "{path} 不在（标准输入或标准输出关着）：关掉 fd 2 之后再开的那一份落不到 2 上"
            ));
        }
    }
    let original = std::io::stderr()
        .as_fd()
        .try_clone_to_owned()
        .map_err(|error| format!("复制不了标准错误：{error}"))?;
    let original = Arc::new(File::from(original));
    let (reader, writer) = std::io::pipe().map_err(|error| format!("开不了管道：{error}"))?;
    let writer_path = format!("/proc/self/fd/{}", writer.as_raw_fd());
    drop(
        OpenOptions::new()
            .write(true)
            .open(&writer_path)
            .map_err(|error| format!("管道的写端 {writer_path} 开不了：{error}"))?,
    );
    // SAFETY: fd 2 是起这个进程的那一方给的标准错误，这个进程里没有 OwnedFd 拿着它（标准库的 Stderr 照号写、不关它）；
    // 调用方保证还没有别的线程：关掉到下一句开出来之间没人写它，也没人开别的文件抢走 2 这个号。
    drop(unsafe { OwnedFd::from_raw_fd(STANDARD_ERROR_DESCRIPTOR) });
    let reopened = match OpenOptions::new().write(true).open(&writer_path) {
        Ok(reopened) if reopened.as_raw_fd() == STANDARD_ERROR_DESCRIPTOR => reopened,
        Ok(reopened) => {
            let _ = writeln!(
                &*original,
                "  ✗ 换标准错误：关掉 fd 2 之后再开 {writer_path} 落在 fd {} 上，不是 2；服务不起",
                reopened.as_raw_fd()
            );
            std::process::exit(STANDARD_ERROR_LOST_EXIT);
        }
        Err(error) => {
            let _ = writeln!(
                &*original,
                "  ✗ 换标准错误：关掉 fd 2 之后 {writer_path} 开不了（{error}）；服务不起"
            );
            std::process::exit(STANDARD_ERROR_LOST_EXIT);
        }
    };
    // fd 2 从此归这个进程的标准错误：不由哪个 OwnedFd 关（收尾时 RoutedStandardError::finish 关）
    let _ = reopened.into_raw_fd();
    drop(writer);
    let original_for_the_panics = Arc::clone(&original);
    std::panic::set_hook(Box::new(move |information| {
        let thread = std::thread::current();
        let _ = writeln!(
            &*original_for_the_panics,
            "thread '{}' {information}",
            thread.name().unwrap_or("<unnamed>")
        );
    }));
    let (finished, filter_finished) = channel();
    std::thread::Builder::new()
        .name("service-log-filter".to_string())
        .spawn(move || {
            let mut forward = &*original;
            let _ = filter_the_service_log(
                BufReader::new(reader),
                &mut forward,
                &DISPATCH_LINES_DROPPED,
                &BreakSwitches::none(),
            );
            let _ = finished.send(());
        })
        .map_err(|error| format!("起不了过滤日志的线程：{error}"))?;
    Ok(RoutedStandardError { filter_finished })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 逐次派活的三种行首（`GPU_LIFECYCLE`、`CRASH_JUDGE_ACCOUNTING_SLICE`、`CRASH_JUDGE_ACCOUNTING_SLICES`）一行不转、计了数，
    /// 别的行（含服务自己打的认领行、没有行尾的末一行）原样、按原次序转。
    #[test]
    fn dispatch_lines_are_counted_and_every_other_line_is_forwarded_verbatim() {
        let input = "GPU_LIFECYCLE unix_microseconds=1 elapsed_microseconds=2 pid=3 thread=ThreadId(4) test=None phase=dispatch_begin card=0\n\
                     GPU_VERIFICATION_SERVER_BLOCK_CLAIMED stream=aa block=1\n\
                     CRASH_JUDGE_ACCOUNTING_SLICE card=\"x\" first_state=0 states=4 round=CountingAsRecorded slice=0 low=0 high=4 seconds=0.001\n\
                     CRASH_JUDGE_ACCOUNTING_SLICES card=\"x\" first_state=0 states=4\n\
                     a line without its end";
        let mut forwarded = Vec::new();
        let dropped = AtomicU64::new(0);
        filter_the_service_log(
            input.as_bytes(),
            &mut forwarded,
            &dropped,
            &BreakSwitches::none(),
        )
        .expect("内存里读写不会失败");
        assert_eq!(
            String::from_utf8(forwarded).expect("转出来的是 UTF-8"),
            "GPU_VERIFICATION_SERVER_BLOCK_CLAIMED stream=aa block=1\n\
             a line without its end"
        );
        assert_eq!(dropped.load(Ordering::Relaxed), 3);
    }
}
