//! 块层读数与差值、软件 RAID 的同步状态、fio 读数的解析、直读写校验与单元的判定。

use super::*;

/// `/sys/block/<名>/stat` 里这一轮要的五个计数（内核 `Documentation/block/stat.rst` 的字段序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BlockLayerCounters {
    pub(super) read_requests: u64,
    read_sectors: u64,
    pub(super) write_requests: u64,
    write_sectors: u64,
    pub(super) flush_requests: u64,
}

/// 两次读数之差，扇区换成字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BlockLayerDelta {
    pub(super) read_requests: u64,
    pub(super) read_bytes: u64,
    pub(super) write_requests: u64,
    pub(super) write_bytes: u64,
    pub(super) flush_requests: u64,
}

/// FLUSH 在下标 15（5.5 起才有），下标 14 是 discard 耗时；少于 16 个字段就拒绝，不拿 0 顶上（读不到 ≠ 读到 0）。
fn parse_block_layer_counters(text: &str) -> Option<BlockLayerCounters> {
    let fields = text
        .split_whitespace()
        .map(|field| field.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    Some(BlockLayerCounters {
        read_requests: *fields.first()?,
        read_sectors: *fields.get(2)?,
        write_requests: *fields.get(4)?,
        write_sectors: *fields.get(6)?,
        flush_requests: *fields.get(15)?,
    })
}

impl BlockLayerCounters {
    pub(super) fn delta_since(self, earlier: Self) -> BlockLayerDelta {
        let difference = |later_value: u64, earlier_value: u64| {
            later_value
                .checked_sub(earlier_value)
                .expect("块层计数只增不减；变小说明两次读的不是同一块盘")
        };
        BlockLayerDelta {
            read_requests: difference(self.read_requests, earlier.read_requests),
            read_bytes: difference(self.read_sectors, earlier.read_sectors) * SECTOR_BYTES,
            write_requests: difference(self.write_requests, earlier.write_requests),
            write_bytes: difference(self.write_sectors, earlier.write_sectors) * SECTOR_BYTES,
            flush_requests: difference(self.flush_requests, earlier.flush_requests),
        }
    }
}

impl BlockLayerDelta {
    pub(super) fn describe(&self) -> String {
        format!(
            "block_read_bytes={} block_read_requests={} block_write_bytes={} block_write_requests={} block_flush_requests={}",
            self.read_bytes, self.read_requests, self.write_bytes, self.write_requests, self.flush_requests
        )
    }
}

pub(super) fn device_name(device_path: &str) -> Result<&str, GuestFailure> {
    Path::new(device_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| GuestFailure::new("device_name", format!("{device_path} 取不出设备名")))
}

pub(super) fn read_block_layer_counters(device_path: &str) -> Result<BlockLayerCounters, GuestFailure> {
    let path = format!("/sys/block/{}/stat", device_name(device_path)?);
    let text = std::fs::read_to_string(&path).map_err(|error| GuestFailure::new("block_counters", format!("{path}：{error}")))?;
    parse_block_layer_counters(&text)
        .ok_or_else(|| GuestFailure::new("block_counters", format!("{path} 解析不出 16 个字段：{text}")))
}

impl BlockLayerCounters {
    /// 登记第十一节：两块成员盘的计数逐项相加。
    fn plus(self, other: Self) -> Self {
        Self {
            read_requests: self.read_requests + other.read_requests,
            read_sectors: self.read_sectors + other.read_sectors,
            write_requests: self.write_requests + other.write_requests,
            write_sectors: self.write_sectors + other.write_sectors,
            flush_requests: self.flush_requests + other.flush_requests,
        }
    }
}

/// 所有成员盘的计数相加：镜像下读会分到两块盘上，只看一块盘，第六节的校验会把读误判成被缓存顶替（登记第十一节）；
/// 单盘配置只有一块盘，相加等于不变。md 配置传进来的是成员盘，不数 /dev/md0（数了就重复）。
pub(super) fn read_summed_block_layer_counters(device_paths: &[String]) -> Result<BlockLayerCounters, GuestFailure> {
    let mut summed = BlockLayerCounters { read_requests: 0, read_sectors: 0, write_requests: 0, write_sectors: 0, flush_requests: 0 };
    for device_path in device_paths {
        summed = summed.plus(read_block_layer_counters(device_path)?);
    }
    Ok(summed)
}

/// 登记第十一节的作废条款：`/proc/mdstat` 里出现 resync 或 recovery，说明阵列在后台同步，那一轮作废。
fn software_raid_is_synchronizing(mdstat_text: &str) -> bool {
    mdstat_text.contains("resync") || mdstat_text.contains("recovery")
}

pub(super) fn check_software_raid_state(moment: &str, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let text = std::fs::read_to_string("/proc/mdstat").map_err(|error| GuestFailure::new("software_raid_state", error.to_string()))?;
    let synchronizing = software_raid_is_synchronizing(&text);
    emitter.emit(
        "software_raid_state",
        &format!("moment={moment} synchronizing={synchronizing} mdstat={}", single_token(&text)),
    );
    if synchronizing {
        return Err(GuestFailure::new("software_raid_synchronizing", single_token(&text)));
    }
    Ok(())
}

pub(super) fn read_device_bytes(device_path: &str) -> Result<u64, GuestFailure> {
    let path = format!("/sys/block/{}/size", device_name(device_path)?);
    let text = std::fs::read_to_string(&path).map_err(|error| GuestFailure::new("device_size", format!("{path}：{error}")))?;
    let sectors = text
        .trim()
        .parse::<u64>()
        .map_err(|error| GuestFailure::new("device_size", format!("{path}：{error}")))?;
    Ok(sectors * SECTOR_BYTES)
}

/// fio 3.36 的 terse 第 3 版一行，只取这一轮要的几个字段（字段号以同一版 fio 的 JSON 输出对过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FioJobReport {
    pub(super) error_code: u64,
    pub(super) read_kibibytes: u64,
    pub(super) read_bandwidth_kibibytes_per_second: u64,
    pub(super) read_operations_per_second: u64,
    pub(super) read_completion_latency_percentile_99_microseconds: Option<u64>,
    pub(super) write_kibibytes: Option<u64>,
    pub(super) write_bandwidth_kibibytes_per_second: Option<u64>,
    pub(super) write_operations_per_second: Option<u64>,
    pub(super) write_completion_latency_percentile_99_microseconds: Option<u64>,
}

/// 字段号从 1 数：5 error、6 读 KiB、7 读带宽、8 读每秒操作数、30 读完成延迟 p99、47 写 KiB、48 写带宽、49 写每秒操作数、71 写 p99。
/// 没有盘的引擎（filecreate 这类）只有 121 个字段，写那一侧照样在前 71 个里。
pub(super) fn parse_fio_terse_version_3(line: &str) -> Option<FioJobReport> {
    let fields: Vec<&str> = line.trim_end().split(';').collect();
    if fields.first() != Some(&"3") {
        return None;
    }
    let number = |position_from_one: usize| -> Option<u64> { fields.get(position_from_one - 1)?.parse::<u64>().ok() };
    let percentile_99 = |position_from_one: usize| -> Option<u64> {
        fields
            .get(position_from_one - 1)?
            .strip_prefix("99.000000%=")?
            .parse::<u64>()
            .ok()
    };
    Some(FioJobReport {
        error_code: number(5)?,
        read_kibibytes: number(6)?,
        read_bandwidth_kibibytes_per_second: number(7)?,
        read_operations_per_second: number(8)?,
        read_completion_latency_percentile_99_microseconds: percentile_99(30),
        write_kibibytes: number(47),
        write_bandwidth_kibibytes_per_second: number(48),
        write_operations_per_second: number(49),
        write_completion_latency_percentile_99_microseconds: percentile_99(71),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DirectInputOutputCheck {
    DirectRead,
    DirectWrite,
    NotChecked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CellVerdict {
    Counted,
    CacheSubstituted,
    WritesAbsorbed,
    NotChecked,
    FioError,
}

impl CellVerdict {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Counted => "counted",
            Self::CacheSubstituted => "cache_substituted",
            Self::WritesAbsorbed => "writes_absorbed",
            Self::NotChecked => "not_checked",
            Self::FioError => "fio_error",
        }
    }
}

/// 登记第六节：块层字节 < 0.9 × fio 报的字节 ⇒ 那一格不算。对照的第二遍也走这个函数，它必须判「被缓存顶替」。
pub(super) fn judge_direct_input_output(check: DirectInputOutputCheck, fio_bytes: u64, block_bytes: u64) -> CellVerdict {
    let below_threshold = block_bytes * DIRECT_CHECK_DENOMINATOR < fio_bytes * DIRECT_CHECK_NUMERATOR;
    match check {
        DirectInputOutputCheck::DirectRead if below_threshold => CellVerdict::CacheSubstituted,
        DirectInputOutputCheck::DirectWrite if below_threshold => CellVerdict::WritesAbsorbed,
        DirectInputOutputCheck::DirectRead | DirectInputOutputCheck::DirectWrite => CellVerdict::Counted,
        DirectInputOutputCheck::NotChecked => CellVerdict::NotChecked,
    }
}

#[cfg(test)]
#[path = "block_layer_counters_and_cell_verdicts_tests.rs"]
mod tests;
