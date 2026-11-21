//! 服务进程的身份与「在跑」那一行，下一个进程起来时照它认上一个进程还在不在。
//!
//! 「在跑」那一行（`main.rs` 写进状态目录的 `serving`，[`serving_line`]）记这个进程的身份：进程号、进程起来的时刻
//! （`/proc/<进程号>/stat` 第 22 项，开机以来的时钟滴答）、这一次开机的号（`/proc/sys/kernel/random/boot_id`），与认领的持有者
//! （认领与写结论那个判定库客户端的身份，十六进制）。持有者是每个进程起来时新取的（`main.rs` 起来先删上一个进程的追加日志），
//! 上一个进程留下的认领下一个进程认不回，只能放：下一个进程起来、写自己那一行之前读上一个进程那一行
//! （[`PreviousServiceProcess::of_the_serving_line`]），判它还在不在（[`ServiceProcessIdentity::liveness`]）。
//! 开机号对得上、`/proc` 里那个进程号在、不是僵尸、起来的时刻相同，才是还活着：它还在判、写回时要核自己的认领，放了它的块就被别的持有者
//! 取走重判、它写回时被拒，所以不放。别的（重开过机、进程号不在、在而起来的时刻不同即进程号让别的进程用了、已是僵尸）都是没了，
//! 以它的持有者放掉它留下的认领（`crate::library_pending_pool` 的 `release_the_claims_left_by_the_previous_process`）。
//! 认不出那一行、判不了活没活的不放，打一行说明，认领到租期过了自己失效。

use std::collections::BTreeMap;
use std::io::ErrorKind;

use singlefs_verification::hexadecimal::hexadecimal_text;

/// 「在跑」那一行的开头；`control` 的 `SERVING_LINE_PREFIX` 与它逐字相同。
pub const SERVING_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_SERVING";
/// 那一行里进程号那一项的键（`control` 照它认进程号）。
const PROCESS_NUMBER_KEY: &str = "pid";
/// 那一行里进程起来的时刻那一项的键。
const PROCESS_START_KEY: &str = "process_start_ticks";
/// 那一行里开机号那一项的键。
const BOOT_KEY: &str = "boot";
/// 那一行里认领的持有者那一项的键。
const HOLDER_KEY: &str = "holder";
/// 这一次开机的号。
const BOOT_IDENTITY_FILE: &str = "/proc/sys/kernel/random/boot_id";
/// `/proc/<进程号>/stat` 里命令名那一项（括号括着、可能带空格与括号）之后，状态是头一项（stat 的第 3 项）。
const PROCESS_STATUS_STATE_INDEX_AFTER_THE_COMMAND: usize = 0;
/// 同上，进程起来的时刻是 stat 的第 22 项，在命令名之后从 0 数第 19 项。
const PROCESS_STATUS_START_TICKS_INDEX_AFTER_THE_COMMAND: usize = 19;
/// 读 `/proc/<进程号>/stat` 时那个进程恰好没了，内核回的错号（ESRCH）。
const NO_SUCH_PROCESS_ERROR_NUMBER: i32 = 3;
/// stat 里进程已结束的两种状态：僵尸（等收尸）、已死。
const ENDED_PROCESS_STATES: [char; 2] = ['Z', 'X'];

/// 一个服务进程的身份：进程号、起来的时刻（开机以来的时钟滴答）、开机号。三样一起才认得出同一个进程（进程号会被别的进程再用）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceProcessIdentity {
    pub process_number: u32,
    pub start_ticks: u64,
    pub boot: String,
}

/// 那个进程此刻还在不在。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessLiveness {
    Running,
    Gone,
}

/// `/proc/<进程号>/stat` 里要的两项。
pub(crate) struct ProcessStatusFields {
    state: char,
    pub(crate) start_ticks: u64,
}

/// 读 `/proc/<进程号>/stat`；进程号不在（或读的时候恰好没了）交 `None`。取进程启动刻的只有这一处（`crate::card_lease` 也调它）。
pub(crate) fn process_status_fields_of(
    process_number: u32,
) -> Result<Option<ProcessStatusFields>, String> {
    let path = format!("/proc/{process_number}/stat");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error)
            if error.kind() == ErrorKind::NotFound
                || error.raw_os_error() == Some(NO_SUCH_PROCESS_ERROR_NUMBER) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(format!("{path} 读不了：{error}")),
    };
    let after_the_command = text
        .rfind(')')
        .map(|position| &text[position + 1..])
        .ok_or_else(|| format!("{path} 里找不到命令名那一项的右括号：{text}"))?;
    let fields: Vec<&str> = after_the_command.split_whitespace().collect();
    let state = fields
        .get(PROCESS_STATUS_STATE_INDEX_AFTER_THE_COMMAND)
        .and_then(|field| field.chars().next())
        .ok_or_else(|| format!("{path} 里没有状态那一项：{text}"))?;
    let start_ticks = fields
        .get(PROCESS_STATUS_START_TICKS_INDEX_AFTER_THE_COMMAND)
        .and_then(|field| field.parse::<u64>().ok())
        .ok_or_else(|| format!("{path} 里起来的时刻那一项认不出：{text}"))?;
    Ok(Some(ProcessStatusFields { state, start_ticks }))
}

/// 这一次开机的号。
fn boot_of_this_machine() -> Result<String, String> {
    std::fs::read_to_string(BOOT_IDENTITY_FILE)
        .map(|text| text.trim().to_string())
        .map_err(|error| format!("{BOOT_IDENTITY_FILE} 读不了：{error}"))
}

impl ServiceProcessIdentity {
    /// 这个进程的身份。
    ///
    /// # Errors
    /// `/proc` 读不了、认不出。
    pub fn of_this_process() -> Result<Self, String> {
        let process_number = std::process::id();
        let boot = boot_of_this_machine()?;
        match process_status_fields_of(process_number)? {
            Some(status_fields) => Ok(Self {
                process_number,
                start_ticks: status_fields.start_ticks,
                boot,
            }),
            None => Err(format!(
                "/proc/{process_number}/stat 不在（这个进程自己的）"
            )),
        }
    }

    /// 这个身份的进程此刻还在不在（文件头）。
    ///
    /// # Errors
    /// 开机号或 `/proc/<进程号>/stat` 读不了、认不出（判不了，调用方不放它的认领）。
    pub fn liveness(&self) -> Result<ProcessLiveness, String> {
        if boot_of_this_machine()? != self.boot {
            return Ok(ProcessLiveness::Gone);
        }
        Ok(match process_status_fields_of(self.process_number)? {
            None => ProcessLiveness::Gone,
            Some(status_fields)
                if ENDED_PROCESS_STATES.contains(&status_fields.state)
                    || status_fields.start_ticks != self.start_ticks =>
            {
                ProcessLiveness::Gone
            }
            Some(_) => ProcessLiveness::Running,
        })
    }
}

/// 「在跑」那一行：这个进程的身份、编进来的限时、从判定库取活的标记与认领的持有者（十六进制）。
#[must_use]
pub fn serving_line(
    process: &ServiceProcessIdentity,
    oldest_pending_wait_limit_milliseconds: u128,
    holder: &[u8],
) -> String {
    format!(
        "{SERVING_LINE_PREFIX} {PROCESS_NUMBER_KEY}={} {PROCESS_START_KEY}={} {BOOT_KEY}={} \
         oldest_pending_wait_limit_milliseconds={oldest_pending_wait_limit_milliseconds} intake=library {HOLDER_KEY}={}",
        process.process_number,
        process.start_ticks,
        process.boot,
        hexadecimal_text(holder)
    )
}

/// 上一个服务进程：「在跑」那一行里记的身份与认领的持有者。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviousServiceProcess {
    pub process: ServiceProcessIdentity,
    pub holder: Vec<u8>,
}

/// 小写或大写十六进制文本的字节（位数是奇数、有别的字符交 `None`）。
fn bytes_of_the_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.bytes().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|start| u8::from_str_radix(&text[start..start + 2], 16).ok())
        .collect()
}

impl PreviousServiceProcess {
    /// 从「在跑」那一行认出上一个进程（[`serving_line`] 写的那几项）。
    ///
    /// # Errors
    /// 不是那一行的开头、缺项、某一项认不出、持有者是空的：交回哪一样。
    pub fn of_the_serving_line(line: &str) -> Result<Self, String> {
        let mut words = line.split_whitespace();
        if words.next() != Some(SERVING_LINE_PREFIX) {
            return Err(format!("不是以 {SERVING_LINE_PREFIX} 起头的一行"));
        }
        let fields: BTreeMap<&str, &str> = words.filter_map(|word| word.split_once('=')).collect();
        let field = |key: &str| {
            fields
                .get(key)
                .copied()
                .ok_or_else(|| format!("那一行没有 {key}="))
        };
        let process_number = field(PROCESS_NUMBER_KEY)?
            .parse::<u32>()
            .map_err(|error| format!("{PROCESS_NUMBER_KEY}= 认不出：{error}"))?;
        let start_ticks = field(PROCESS_START_KEY)?
            .parse::<u64>()
            .map_err(|error| format!("{PROCESS_START_KEY}= 认不出：{error}"))?;
        let boot = field(BOOT_KEY)?.to_string();
        let holder = bytes_of_the_hexadecimal(field(HOLDER_KEY)?)
            .filter(|holder| !holder.is_empty())
            .ok_or_else(|| format!("{HOLDER_KEY}= 不是非空的十六进制"))?;
        Ok(Self {
            process: ServiceProcessIdentity {
                process_number,
                start_ticks,
                boot,
            },
            holder,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 「在跑」那一行逐字的样子：`control` 照这几项的键读，改了这里要一起改 `control`。
    #[test]
    fn the_serving_line_is_written_word_for_word() {
        let process = ServiceProcessIdentity {
            process_number: 12,
            start_ticks: 34,
            boot: "b".to_string(),
        };
        let holder = [0x5a, 0x00, 0xff];
        assert_eq!(
            serving_line(&process, 500, &holder),
            "GPU_VERIFICATION_SERVER_SERVING pid=12 process_start_ticks=34 boot=b \
             oldest_pending_wait_limit_milliseconds=500 intake=library holder=5a00ff"
        );
    }

    /// 写出来的「在跑」那一行认得回同一个身份与持有者；这个进程自己的身份判成还活着，起来的时刻对不上的判成没了。
    #[test]
    fn the_serving_line_reads_back_and_only_the_same_start_counts_as_running() {
        let this_process = ServiceProcessIdentity::of_this_process().expect("读得到这个进程的身份");
        let holder = vec![0x5a, 0x00, 0xff];
        let line = serving_line(&this_process, 500, &holder);
        let previous = PreviousServiceProcess::of_the_serving_line(&line).expect("认得回");
        assert_eq!(
            previous,
            PreviousServiceProcess {
                process: this_process.clone(),
                holder,
            }
        );
        assert_eq!(
            this_process.liveness().expect("判得了"),
            ProcessLiveness::Running
        );
        let reused_number = ServiceProcessIdentity {
            process_number: this_process.process_number,
            start_ticks: this_process.start_ticks + 1,
            boot: this_process.boot.clone(),
        };
        assert_eq!(
            reused_number.liveness().expect("判得了"),
            ProcessLiveness::Gone,
            "进程号在、起来的时刻不同：进程号让别的进程用了，上一个进程没了"
        );
        let other_boot = ServiceProcessIdentity {
            process_number: this_process.process_number,
            start_ticks: this_process.start_ticks,
            boot: format!("{}-other-boot", this_process.boot),
        };
        assert_eq!(
            other_boot.liveness().expect("判得了"),
            ProcessLiveness::Gone,
            "开机号不同：重开过机"
        );
    }

    /// 缺项、持有者不是十六进制、开头不对的那一行认不出（调用方照「认不出不放」办）。
    #[test]
    fn a_serving_line_missing_a_field_is_not_recognized() {
        for line in [
            "GPU_VERIFICATION_SERVER_SERVING pid=12 oldest_pending_wait_limit_milliseconds=500 intake=library holder=ab",
            "GPU_VERIFICATION_SERVER_SERVING pid=12 process_start_ticks=34 boot=b holder=xyz",
            "SOMETHING_ELSE pid=12 process_start_ticks=34 boot=b holder=ab",
        ] {
            assert!(
                PreviousServiceProcess::of_the_serving_line(line).is_err(),
                "该认不出：{line}"
            );
        }
    }
}
