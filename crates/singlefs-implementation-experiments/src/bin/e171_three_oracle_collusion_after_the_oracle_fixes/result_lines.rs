//! 结果行（`E7RESULT`、`E7TIMING`）与和流无关的核对。

use super::*;

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
pub(super) struct ResultLines {
    pub(super) emitted: u64,
    /// 收集模式（只在内联用例里用，修订 5 第 5.7 节 U8）：结果行收进这里、不打到标准输出；None 时照常打。
    pub(super) collected: Option<Vec<String>>,
}

impl ResultLines {
    pub(super) fn emit_result_line(&mut self, body: &str) {
        self.emitted += 1;
        match &mut self.collected {
            Some(collected) => collected.push(body.to_string()),
            None => println!("E7RESULT {body}"),
        }
    }
    pub(super) fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    pub(super) fn finish(&mut self) {
        self.emitted += 1;
        let body = format!("name=done emitted={}", self.emitted);
        match &mut self.collected {
            Some(collected) => collected.push(body),
            None => println!("E7RESULT {body}"),
        }
    }
}

/// 结果行里的值不许带空白（按空格切字段）。
pub(super) fn field_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// S1 里与流无关的几条：本地常量对验证包库的公开常量或现算值（实验包的造池取的也是这几样，回比不经它）。盘宽是库给新池新建文件那一类池定的
/// 那一档（`history::HistoryDeviceWidth::FourGibibytes`，库里写明与 `tests/common/mod.rs` 的文件镜像同宽）；mkfs 参数是
/// `known_bad_images::parameters_of_the_common_test_pool`；固定写时刻、第一个文件的字节数与内容是 `scenario` 那一份。
pub(super) fn stream_independent_checks() -> Vec<StopCheck> {
    let parameters = singlefs_verification::known_bad_images::parameters_of_the_common_test_pool();
    vec![
        StopCheck::new(
            "device_bytes",
            LOCAL_DEVICE_BYTES,
            singlefs_verification::history::HistoryDeviceWidth::FourGibibytes.device_bytes(),
        ),
        StopCheck::new(
            "physical_block_bytes",
            LOCAL_PHYSICAL_BLOCK_BYTES,
            parameters.geometry.physical_block_size,
        ),
        StopCheck::new(
            "minimum_input_output_bytes",
            LOCAL_MINIMUM_INPUT_OUTPUT_BYTES,
            parameters.geometry.minimum_input_output_bytes,
        ),
        StopCheck::new(
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_verification::memory_pool::SECTOR_BYTES,
        ),
        StopCheck::new(
            "filesystem_identifier",
            LOCAL_FILESYSTEM_IDENTIFIER,
            singlefs_verification::known_bad_images::E142_FILESYSTEM_IDENTIFIER,
        ),
        StopCheck::new(
            "fixed_write_time_seconds",
            LOCAL_FIXED_WRITE_TIME_SECONDS,
            singlefs_verification::scenario::FIXED_WRITE_TIME_SECONDS,
        ),
        StopCheck::new(
            "first_file_bytes",
            LOCAL_FIRST_FILE_BYTES,
            singlefs_verification::scenario::FIRST_FILE_BYTES,
        ),
        StopCheck::new(
            "first_file_content",
            true,
            local_first_file_content() == singlefs_verification::scenario::first_file_content(),
        ),
    ]
}

/// 中位取排好序之后下标 ⌊(n − 1)/2⌋ 那一个，p95 取下标 ⌈0.95·n⌉ − 1 那一个。
pub(super) fn timing_summary(mut nanoseconds: Vec<u128>) -> (u128, u128, u128, u128) {
    if nanoseconds.is_empty() {
        return (0, 0, 0, 0);
    }
    nanoseconds.sort_unstable();
    let count = nanoseconds.len();
    let total: u128 = nanoseconds.iter().sum();
    let ninety_fifth_percentile_index = (count * 95).div_ceil(100) - 1;
    (
        total,
        nanoseconds[(count - 1) / 2],
        nanoseconds[ninety_fifth_percentile_index],
        nanoseconds[count - 1],
    )
}

#[cfg(test)]
mod tests;
