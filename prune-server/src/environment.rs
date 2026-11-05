//! 压测器的环境诊断：起止 `/proc/loadavg`、压测库所在块设备的忙度（`/proc/diskstats` 的 io_ticks 差值）。
//! loadavg 1 分钟值在起跑前或跑完时超过阈值，这一趟标 `environment-dirty`，不判达标没达标（同机别的重活压着盘与 CPU 时，
//! 量到的吞吐说明不了服务扛不扛得住：压测 2064 那一趟 loadavg 到过 209）。

use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::break_switches::{BreakSwitch, BreakSwitches};

/// /proc/diskstats 每行：主号、次号、名字之后的第 10 个计数是 io_ticks（设备忙着的毫秒数）。
const INPUT_OUTPUT_TICKS_FIELD_INDEX: usize = 12;

/// 解 `/proc/loadavg` 的前三个数（1、5、15 分钟）。
pub fn parse_load_averages(text: &str) -> Option<[f64; 3]> {
    let mut fields = text.split_whitespace();
    let mut averages = [0.0; 3];
    for slot in &mut averages {
        *slot = fields.next()?.parse().ok()?;
    }
    Some(averages)
}

pub fn read_load_averages() -> Option<[f64; 3]> {
    parse_load_averages(&std::fs::read_to_string("/proc/loadavg").ok()?)
}

/// 缺省阈值：本机 CPU 核数的一半。
pub fn default_load_threshold() -> f64 {
    let cores = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    cores as f64 / 2.0
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvironmentJudgment {
    Clean,
    Dirty,
}

/// 起跑前与跑完的 1 分钟 loadavg 有一个超过阈值就是不干净；读不到 loadavg 的也算不干净（判不了不当干净）。
pub fn judge_environment(
    load_at_start: Option<f64>,
    load_at_end: Option<f64>,
    threshold: f64,
    break_switches: &BreakSwitches,
) -> EnvironmentJudgment {
    if break_switches.is_enabled(BreakSwitch::BenchLoadThresholdIgnored) {
        return EnvironmentJudgment::Clean;
    }
    match (load_at_start, load_at_end) {
        (Some(start), Some(end)) if start <= threshold && end <= threshold => {
            EnvironmentJudgment::Clean
        }
        (Some(_) | None, Some(_) | None) => EnvironmentJudgment::Dirty,
    }
}

/// 块设备：名字与主次号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockDevice {
    pub name: String,
    pub major: u32,
    pub minor: u32,
}

fn split_device_number(device: u64) -> (u32, u32) {
    // glibc 的 dev_t 编码（gnu_dev_major / gnu_dev_minor）。
    let major = ((device >> 8) & 0xfff) | ((device >> 32) & !0xfff);
    let minor = (device & 0xff) | ((device >> 12) & !0xff);
    (
        u32::try_from(major).unwrap_or(u32::MAX),
        u32::try_from(minor).unwrap_or(u32::MAX),
    )
}

/// 在 diskstats 文本里找主次号对得上的那一行，回（名字，io_ticks）。
pub fn parse_input_output_ticks(diskstats: &str, major: u32, minor: u32) -> Option<(String, u64)> {
    diskstats.lines().find_map(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let line_major: u32 = fields.first()?.parse().ok()?;
        let line_minor: u32 = fields.get(1)?.parse().ok()?;
        if line_major != major || line_minor != minor {
            return None;
        }
        let ticks: u64 = fields.get(INPUT_OUTPUT_TICKS_FIELD_INDEX)?.parse().ok()?;
        Some(((*fields.get(2)?).to_string(), ticks))
    })
}

/// 路径所在挂载点的来源设备（`/proc/self/mountinfo` 里最长的那个挂载点前缀），给 btrfs 这类 st_dev 是匿名号的文件系统用。
fn mount_source_device(path: &Path) -> Option<(u32, u32)> {
    let canonical = std::fs::canonicalize(path).ok()?;
    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    let mut best: Option<(usize, String)> = None;
    for line in mountinfo.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let mount_point = fields.get(4)?;
        let separator = fields.iter().position(|field| *field == "-")?;
        let source = fields.get(separator + 2)?;
        if canonical.starts_with(mount_point)
            && best
                .as_ref()
                .is_none_or(|(length, _)| mount_point.len() > *length)
        {
            best = Some((mount_point.len(), (*source).to_string()));
        }
    }
    let (_, source) = best?;
    let metadata = std::fs::metadata(&source).ok()?;
    Some(split_device_number(metadata.rdev()))
}

/// 路径所在的块设备；tmpfs 这类没有块设备的回 `None`。
pub fn block_device_for_path(path: &Path) -> Option<BlockDevice> {
    let diskstats = std::fs::read_to_string("/proc/diskstats").ok()?;
    let (major, minor) = split_device_number(std::fs::metadata(path).ok()?.dev());
    let (major, minor) = if parse_input_output_ticks(&diskstats, major, minor).is_some() {
        (major, minor)
    } else {
        mount_source_device(path)?
    };
    let (name, _) = parse_input_output_ticks(&diskstats, major, minor)?;
    Some(BlockDevice { name, major, minor })
}

pub fn read_input_output_ticks(device: &BlockDevice) -> Option<u64> {
    let diskstats = std::fs::read_to_string("/proc/diskstats").ok()?;
    parse_input_output_ticks(&diskstats, device.major, device.minor).map(|(_, ticks)| ticks)
}

/// 忙度百分比：io_ticks 差值除以墙钟毫秒数。
pub fn busy_percent(ticks_at_start: u64, ticks_at_end: u64, wall_milliseconds: f64) -> f64 {
    if wall_milliseconds <= 0.0 {
        return 0.0;
    }
    (ticks_at_end.saturating_sub(ticks_at_start) as f64 / wall_milliseconds * 100.0).min(100.0)
}
