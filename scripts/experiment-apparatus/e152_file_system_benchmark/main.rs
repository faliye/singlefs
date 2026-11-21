//! E152（按里程碑对比六家文件系统的文件性能）：虚机里对一个配置跑一轮负载，结果行以 `E7RESULT` 打头；
//! 宿主上用 `summarize` 模式把多轮产物汇成中位、最小、最大与离散。
//!
//!   来宾：/bench <设备...> <配置> <轮次>        （vm-bench.sh 把设备路径排在参数前面）
//!   宿主：e152-file-system-benchmark summarize <产物>
//!
//! 负载、次序、判据与作废条款照跑前登记 `research/prompts/e152-preregistration.md`（含第十节的修订与第十一节的两盘镜像）逐条写；
//! 驱动在 `scripts/devices/e152-run.sh`，附加根在 `scripts/devices/e152-stage-root.sh`。
//! 这个二进制自己只判两件事：块层读数与 fio 读数对不对得上（登记第六节），以及同一个判定函数在对照上会不会红。

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

mod block_layer_counters_and_cell_verdicts;
mod command_running_and_failure_reports;
mod configurations_and_setup_scripts;
mod file_system_and_raw_device_rounds;
mod fio_jobs_and_persist_workload;
mod guest_entry;
mod summaries_of_runs;

use self::block_layer_counters_and_cell_verdicts::*;
use self::command_running_and_failure_reports::*;
use self::configurations_and_setup_scripts::*;
use self::file_system_and_raw_device_rounds::*;
use self::fio_jobs_and_persist_workload::*;
use self::guest_entry::*;
use self::summaries_of_runs::*;

const MOUNT_POINT: &str = "/mnt";
const LARGE_FILE_PATH: &str = "/mnt/large";
const METADATA_DIRECTORY: &str = "/mnt/metadata";
const SMALL_FILE_DIRECTORY: &str = "/mnt/small";
const PERSIST_DIRECTORY: &str = "/mnt/persist";
const ZFS_POOL_NAME: &str = "bench";
const MODULE_ORDER_DIRECTORY: &str = "/lib/modules/e152";
const SINGLEFS_DEVICE_BINARY: &str = "/usr/bin/new_pool_file_creation_on_device";
/// 登记第十一节：md 配置的 raid1 阵列。
const SOFTWARE_RAID_DEVICE: &str = "/dev/md0";
/// `/sys/block/<名>/stat` 的扇区恒按 512 字节计（内核 `Documentation/block/stat.rst`），与设备的逻辑块大小无关。
const SECTOR_BYTES: u64 = 512;
/// 登记第四节第 2 步：100 个新文件，每个 3000 字节。
const PERSIST_FILE_COUNT: usize = 100;
const PERSIST_FILE_BYTES: usize = 3000;
/// 登记第六节：块层字节 < 0.9 × fio 报的字节 ⇒ 那一格不算。
const DIRECT_CHECK_NUMERATOR: u64 = 9;
const DIRECT_CHECK_DENOMINATOR: u64 = 10;
/// 登记第七节：离散 ≤ 15% 记「稳定」。
const STABLE_SPREAD_PERCENT: f64 = 15.0;
const DMESG_TAIL_LINES: usize = 30;
/// 三个元数据作业作用在同一批文件上：fio 默认按作业名给文件起名，三个作业名字不同就会各找各的（登记第十节修订二）。
const METADATA_FILENAME_FORMAT: &str = "--filename_format=metadata.$filenum";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("summarize") {
        return run_summarize(&arguments[1..]);
    }
    run_guest(&arguments)
}
