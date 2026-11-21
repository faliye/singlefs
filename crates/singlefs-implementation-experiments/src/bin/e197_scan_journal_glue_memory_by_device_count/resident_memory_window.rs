//! 量窗：往 `/proc/self/clear_refs` 写 `5` 重置 `VmHWM`、读 `/proc/self/status` 的 `VmRSS` 与 `VmHWM`，算常驻页峰值 H。
use super::*;

// ============================================================================================
// 进程内的常驻页峰值 H（登记修订三「进程内那一条量法 H」）：只经 `std::fs` 写 `/proc/self/clear_refs`、读 `/proc/self/status`。
// 量窗开头写 `5` 把 `VmHWM` 重置成此刻的常驻量，读 `VmRSS` 当起点；交回之后读 `VmHWM`；H = (VmHWM − 起点) × 1024。
// 写不进、读不出都交 `None`，结果行写 `unreadable`（V11：那一次作废，不当 0）。
// ============================================================================================

const PROCESS_FILESYSTEM_SELF_STATUS_FILE: &str = "/proc/self/status";
const PROCESS_FILESYSTEM_SELF_CLEAR_REFERENCES_FILE: &str = "/proc/self/clear_refs";
const VM_RESIDENT_FIELD: &str = "VmRSS:";
const VM_HIGH_WATER_MARK_FIELD: &str = "VmHWM:";
const BYTES_PER_KIBIBYTE: i64 = 1024;

/// `/proc/self/status` 文本里某个字段（`VmRSS:`、`VmHWM:`）的 KiB 数；字段不在、不是 `kB` 单位、读不成数都交 `None`。
fn kibibytes_of_the_status_field(status_text: &str, field: &str) -> Option<u64> {
    let rest = status_text
        .lines()
        .find_map(|line| line.strip_prefix(field))?;
    let mut words = rest.split_whitespace();
    let kibibytes: u64 = words.next()?.parse().ok()?;
    (words.next()? == "kB").then_some(kibibytes)
}

fn signed_kibibytes(kibibytes: u64) -> i64 {
    i64::try_from(kibibytes).expect("KiB 数装得进 i64")
}

impl ResidentWindow {
    /// 开窗：往 `/proc/self/clear_refs` 写 `5`，随即读 `VmRSS`。写不进或读不出交 `None`。
    pub(super) fn open() -> Option<ResidentWindow> {
        ResidentWindow::open_with(
            PROCESS_FILESYSTEM_SELF_CLEAR_REFERENCES_FILE,
            PROCESS_FILESYSTEM_SELF_STATUS_FILE,
        )
    }

    fn open_with(clear_refs_file: &str, status_file: &str) -> Option<ResidentWindow> {
        std::fs::write(clear_refs_file, "5").ok()?;
        let status_text = std::fs::read_to_string(status_file).ok()?;
        Some(ResidentWindow {
            rss_start_kib: kibibytes_of_the_status_field(&status_text, VM_RESIDENT_FIELD)?,
        })
    }

    /// H：交回之后的 `VmHWM` 减起点，换成字节。
    pub(super) fn peak_over_the_start_bytes(&self, end: &ResidentWindowEnd) -> i64 {
        (signed_kibibytes(end.high_water_mark_kib) - signed_kibibytes(self.rss_start_kib))
            * BYTES_PER_KIBIBYTE
    }

    /// 某一刻的 `VmRSS`（KiB）减起点，换成字节：轨迹与期末值。
    pub(super) fn resident_over_the_start_bytes(&self, resident_kib: u64) -> i64 {
        (signed_kibibytes(resident_kib) - signed_kibibytes(self.rss_start_kib)) * BYTES_PER_KIBIBYTE
    }
}

impl ResidentWindowEnd {
    pub(super) fn read_now() -> Option<ResidentWindowEnd> {
        ResidentWindowEnd::read_from(PROCESS_FILESYSTEM_SELF_STATUS_FILE)
    }

    fn read_from(status_file: &str) -> Option<ResidentWindowEnd> {
        let status_text = std::fs::read_to_string(status_file).ok()?;
        Some(ResidentWindowEnd {
            high_water_mark_kib: kibibytes_of_the_status_field(
                &status_text,
                VM_HIGH_WATER_MARK_FIELD,
            )?,
            resident_kib: kibibytes_of_the_status_field(&status_text, VM_RESIDENT_FIELD)?,
        })
    }
}

/// 此刻的 `VmRSS`（KiB）；读不出交 `None`。池在每块盘最后一槽被读到时调它（轨迹）。
pub(super) fn vm_rss_kib_now() -> Option<u64> {
    let status_text = std::fs::read_to_string(PROCESS_FILESYSTEM_SELF_STATUS_FILE).ok()?;
    kibibytes_of_the_status_field(&status_text, VM_RESIDENT_FIELD)
}

#[cfg(test)]
mod tests;
