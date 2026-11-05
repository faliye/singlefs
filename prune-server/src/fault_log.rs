//! 故障日志：服务每出一个自身的错，往状态目录里的 `faults.log` 追加一行。客户端发错格式被拒的正常拒绝不记。
//!
//! 一行的写法（字段次序固定，`detail=` 在最后、可以带空格）：
//! `PRUNE_SERVER_FAULT time=<ISO 8601 写法的时刻，到毫秒，零时区> kind=<种类> server_pid=<服务进程号> connection=<连接号>
//!  transport=<unix|tcp> peer=<对端地址> host=<主机名> task=<任务号> attempt=<第几次> pid=<客户端进程号> detail=<说明>`；
//! 不知道是哪条连接的（内部 panic）连接那几段写 `-`。`prune-server/control watch` 盯这份文件里新增的行。
//!
//! 种类（`FaultKind`）：
//!   storage-read-failed       读库出错（成批点读、按前缀扫、去重查请求号、认领查结论）
//!   storage-write-failed      写库或同步 WAL 出错（这一组的回应都回存储出错）
//!   dispatch-bug              服务把帧分派错了地方（工作线程收到写、提交线程收到读这类，是服务的 bug）
//!   internal-panic            服务里有线程 panic（断言没过）
//!   stop-notice-undelivered   宣布停服务时有连接的「服务停了」限时内没写进套接字（写线程堵着）

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::connections::{announcement_fields, line_field_value, ConnectionRecord};

pub const FAULT_LINE_PREFIX: &str = "PRUNE_SERVER_FAULT";
/// 故障日志在状态目录里的文件名。
pub const FAULT_LOG_FILE_NAME: &str = "faults.log";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FaultKind {
    StorageReadFailed,
    StorageWriteFailed,
    DispatchBug,
    InternalPanic,
    /// 宣布停服务时有连接的「服务停了」限时内没写进套接字（那几条客户端只能靠重连上限退出）。
    StopNoticeUndelivered,
}

impl FaultKind {
    pub fn name(self) -> &'static str {
        match self {
            FaultKind::StorageReadFailed => "storage-read-failed",
            FaultKind::StorageWriteFailed => "storage-write-failed",
            FaultKind::DispatchBug => "dispatch-bug",
            FaultKind::InternalPanic => "internal-panic",
            FaultKind::StopNoticeUndelivered => "stop-notice-undelivered",
        }
    }
}

/// 自 1970 年起的天数换成（年，月，日），公历（Howard Hinnant 的 civil_from_days）。
fn civil_date_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let shifted = days_since_epoch + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (
        year,
        u32::try_from(month).expect("月份在 1 到 12 之间"),
        u32::try_from(day).expect("日在 1 到 31 之间"),
    )
}

/// 零时区的时刻，ISO 8601 写法、到毫秒、末尾带 Z。
pub fn timestamp_text(moment: SystemTime) -> String {
    let since_epoch = moment.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_seconds = i64::try_from(since_epoch.as_secs()).unwrap_or(i64::MAX);
    let (year, month, day) = civil_date_from_days(total_seconds.div_euclid(86_400));
    let second_of_day = total_seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        second_of_day / 3600,
        second_of_day % 3600 / 60,
        second_of_day % 60,
        since_epoch.subsec_millis()
    )
}

/// 说明里的换行换掉，一行一个故障。
fn single_line(text: &str) -> String {
    text.replace(['\n', '\r'], " / ")
}

/// 拼一行故障。
pub fn fault_line(
    moment: SystemTime,
    kind: FaultKind,
    connection: Option<&ConnectionRecord>,
    detail: &str,
) -> String {
    let connection_fields = match connection {
        Some(record) => format!(
            "connection={} transport={} peer={} {}",
            record.connection,
            record.transport.name(),
            line_field_value(&record.peer_address),
            announcement_fields(record.announcement().as_ref())
        ),
        None => format!(
            "connection=- transport=- peer=- {}",
            announcement_fields(None)
        ),
    };
    format!(
        "{FAULT_LINE_PREFIX} time={} kind={} server_pid={} {connection_fields} detail={}\n",
        timestamp_text(moment),
        kind.name(),
        std::process::id(),
        single_line(detail)
    )
}

/// 追加写的故障日志。弄坏开关 fault-line-not-written 下一行都不写。
pub struct FaultLog {
    path: PathBuf,
    file: Mutex<std::fs::File>,
    is_disabled: bool,
}

impl FaultLog {
    /// 打开（没有就建）状态目录里的故障日志，追加写。
    pub fn open(
        state_directory: &Path,
        break_switches: &BreakSwitches,
    ) -> Result<FaultLog, String> {
        let path = state_directory.join(FAULT_LOG_FILE_NAME);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| format!("打不开故障日志 {}：{error}", path.display()))?;
        Ok(FaultLog {
            path,
            file: Mutex::new(file),
            is_disabled: break_switches.is_enabled(BreakSwitch::FaultLineNotWritten),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 记一行；写不进去的打到标准错误（journalctl 里看得到），不拖垮正在处理的帧。
    pub fn record(&self, kind: FaultKind, connection: Option<&ConnectionRecord>, detail: &str) {
        let line = fault_line(SystemTime::now(), kind, connection, detail);
        eprint!("{line}");
        if self.is_disabled {
            return;
        }
        let mut file = self.file.lock().unwrap_or_else(PoisonError::into_inner);
        if let Err(error) = file.write_all(line.as_bytes()) {
            eprintln!("  ✗ 故障行写不进 {}：{error}", self.path.display());
        }
    }

    /// panic 时记一行：锁被正 panic 的那个线程拿着时不等（等了就死锁），只打到标准错误。
    fn record_from_panic(&self, detail: &str) {
        let line = fault_line(SystemTime::now(), FaultKind::InternalPanic, None, detail);
        eprint!("{line}");
        if self.is_disabled {
            return;
        }
        if let Ok(mut file) = self.file.try_lock() {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

/// 装 panic 钩子：任何线程 panic 都先记一行 internal-panic，再走原来的钩子（打 panic 信息）。
pub fn install_panic_recorder(fault_log: Arc<FaultLog>) {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_information| {
        let thread_name = std::thread::current()
            .name()
            .map_or_else(|| "没有名字".to_string(), str::to_string);
        fault_log.record_from_panic(&format!("线程「{thread_name}」panic：{panic_information}"));
        previous_hook(panic_information);
    }));
}
