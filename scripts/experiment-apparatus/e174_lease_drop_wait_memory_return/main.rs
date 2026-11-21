//! E174（借据丢掉时等待与显存回还）装置。跑前登记：research/prompts/e174-preregistration.md。
//!
//! 独立手写，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）：后端只用 Vulkan，
//! `required_limits` 取 `adapter.limits()`，`memory_hints` 取 `Performance`，读数用 `nvidia-smi -q -d MEMORY,PIDS`
//! 按进程号把 `Used GPU Memory` 加起来。这些设置抄自实现的取法，开跑前由执行员用登记第十一节那 7 条 grep 回比。
//!
//! 结构：借据的整套步骤（登记第五节「一笔借据在装置里怎么做」）写在 `run_one_lease`，对着 `CardHost` 这个 trait 写，
//! 有两份实现：`WgpuCardHost`（真卡）与 `SimulatedCardHost`（模型，单测与 `checks` 子命令用，不起卡）。
//! 登记第九节的变异都改在对着 trait 的那一份代码上，所以不起卡也能由单测抓到。
//!
//! 子命令：`run`（父进程：按清单起子进程、续跑；`--segment first|second --output 产物 [--skip-light] [--limit-jobs N] [--simulated]`
//! `--code-key-file <组成清单>`（必给）；一律经 `scripts/experiment-apparatus/device-code-key.py run` 起，
//! 例 `python3 scripts/experiment-apparatus/device-code-key.py run . --binary e174-lease-drop-wait-memory-return --feature e174-gpu
//! --entry e7_index_bench::run_child_on_the_card --entry e7_index_bench::ChildJob::child_arguments -- run …`，
//! `--simulated` 的那一趟入口换成 `e7_index_bench::run_child_on_the_model`；子进程照 device-code-key.py 放进环境变量
//! `DEVICE_CODE_KEY_DEVICE_BINARY` 的装置路径起，没有这个变量 `run` 退 3、一个子进程都不起）、
//! `child`（一个装置进程，由 `run` 起）、`judge`（读产物判，确定性：`--input 产物 [--input …] [--scope 名]`）、
//! `checks`（不起卡的自检，确定性）、`plan`（清单各阶段的格数与子进程数，确定性）、`simulate`（在模型卡上跑某一格几轮，试 `judge` 用；`--rounds` 不给就是登记的每格轮数）。
//! `run --simulated` 让每个子进程都跑在模型卡上，整条父子流水线（续跑、补轮、原文压缩、F1 停机）不起卡也走得通。
//! 续跑（登记修订 2 之五）：做完的一轮写进轮库的 `<轮键>.out` 那一条，轮键 = sha256（子进程入口与 `ChildJob::child_arguments` 两把代码键、
//! `{cell:?}` 与轮号、驱动与卡那一行）；同一条命令重起，轮键对得上的轮读回不再起；这一段的轮都齐了（或 F1 停机）由库拼出产物原子写到 `--output`
//! （已在而内容相同报「已经齐了」，不同退 2、一个字节不动）。产物路径、`--skip-light`、`--limit-jobs`、清单位置、日期、机器都不进轮键；
//! `codes/<代码键>.txt` 存组成清单，`index.tsv` 只当索引。轮库是 prune-server 里的那一份判定库（.claude/rules/verification.md「判定库只有一份」）：
//! 装置不开本地库目录，读写经 `device-code-key.py run` 开的两根管道转给服务（`crates/singlefs-device-library/src/lib.rs` 文件头；族与种类在 device-code-key.py 文件头「判定库的转交」），
//! 服务连不上照转回的退出码退（75），不退回本地文件。
//! 卡（`child` 子进程）只开服务给的：环境里有 `GPU_CARDS_BY_PRIORITY` 照它，没有而有领卡通道（服务交的五个变量）时到开设备前一刻才经
//! `task-server/task lease take` 领、子进程退出时还；从给这一台的卡里取头一张与这一格同型号的（`nvidia-smi` 报的 FB Total 对上登记第七节 B1 的数），
//! 照它的总线开卡、读数；没给卡、领卡没成、给的卡里没有这一型都让子进程退非 0（父进程记 child_failed），不自己枚举、不按写死的总线号开。
//! 父进程不开卡；轮键里「驱动与卡那一行」按型号取（`device_lines_by_card`）。

// admission: always 每次照此刻的仓与自己的常量现算，上一次跑出的产物不替这一次作保
// run-condition: command python3
// task-server-gpu-arguments: run child
use std::collections::BTreeMap;

// 判定库的口子（文件头「续跑」）：包 `singlefs-device-library`（crates/singlefs-device-library），scripts/experiment-apparatus/Cargo.toml 里按 path 依赖引
use singlefs_device_library::DeviceLibrary;

mod cards_methods_and_lease_records;
mod lease_runs_and_simulated_card_host;
mod real_card_host_and_child_roles;
mod round_data_and_verdict_judging;
mod rounds_and_parent_run;
mod segments_and_round_libraries;
mod simulated_runs_and_commands;

use self::cards_methods_and_lease_records::*;
use self::lease_runs_and_simulated_card_host::*;
use self::real_card_host_and_child_roles::*;
use self::round_data_and_verdict_judging::*;
use self::rounds_and_parent_run::*;
use self::segments_and_round_libraries::*;
use self::simulated_runs_and_commands::*;

// ============================== 常量（登记里的数，各只一处） ==============================

const MEBIBYTE: u64 = 1 << 20;
const GIBIBYTE: u64 = 1 << 30;
/// 登记第五节：S 大于 1 GiB 时拆成每张 1 GiB 的几张（NVIDIA Vulkan 存储缓冲绑定上限是 2 GiB − 1）。
const MAXIMUM_PIECE_BYTES: u64 = GIBIBYTE;
/// 登记第七节 B7。
const EXPECTED_DRIVER_VERSION: &str = "595.84";
const PROCESS_REPORT_ARGUMENTS: [&str; 3] = ["-q", "-d", "MEMORY,PIDS"];
const COMPUTE_APPLICATIONS_ARGUMENTS: [&str; 2] =
    ["--query-compute-apps=gpu_bus_id,pid,used_memory", "--format=csv,noheader,nounits"];
const LEASE_SIZES_MEBIBYTES: [u64; 5] = [1, 64, 512, 2048, 8192];
const WAIT_TIMEOUT_SECONDS: u64 = 30;
const SHORT_WINDOW_MICROSECONDS: u64 = 2_000_000;
const LONG_WINDOW_MICROSECONDS: u64 = 30_000_000;
const PASSES_PER_PASS_BY_PASS_LEASE: u32 = 8;
const LEASES_PER_PROCESS: u32 = 3;
const READS_PER_BASELINE: usize = 3;
/// 登记第六节：U_pre 或 U_dev 那三份的极差超过它，那一笔借据（U_dev 的话那一个子进程）作废。
const MAXIMUM_BASELINE_SPREAD_MEBIBYTES: u64 = 16;
/// 登记第六节：τ = 三份极差 + 2 MiB，2 MiB 是两份各自取整到 MiB 的读数相减最多差出不到 2 MiB。
const READING_ROUNDING_ALLOWANCE_MEBIBYTES: u64 = 2;
const ROUNDS_REQUIRED_PER_CELL: u32 = 5;
const MAXIMUM_REFILL_ROUNDS_PER_CELL: u32 = 3;
const COEXISTING_BUFFER_MEBIBYTES: u64 = 64;
const SHUFFLE_SEED: u64 = 174;
const CALIBRATION_BAND_LOWER_PERCENT: u64 = 80;
const CALIBRATION_BAND_UPPER_PERCENT: u64 = 125;
const CALIBRATION_CONSECUTIVE_HITS: usize = 3;
const CALIBRATION_ATTEMPT_LIMIT: usize = 40;
/// 登记第九节 K7、K9 的 0.8。
const COVERED_WORK_PERCENT: u64 = 80;
/// 登记第六节 Q1、Q1b、Q2、Q3 的门槛（微秒）。
const DONE_WAIT_ZERO_OVERHEAD_LIMIT_MICROSECONDS: u64 = 5_000;
const DONE_WAIT_EXPENSIVE_LIMIT_MICROSECONDS: u64 = 50_000;
const IN_FLIGHT_EXCESS_LIMIT_MICROSECONDS: u64 = 20_000;
const POLL_NO_WAIT_LIMIT_MICROSECONDS: u64 = 1_000;
const BACKGROUND_WORK_TARGET_MICROSECONDS: u64 = 400_000;
const SPIN_KERNEL_BINDING_BYTES: u64 = 256;
const HOLD_CONTROL_SIZES_MEBIBYTES: [u64; 2] = [512, 2048];
const COEXISTING_SIZES_MEBIBYTES: [u64; 2] = [1, 64];
const PASS_BY_PASS_SIZES_MEBIBYTES: [u64; 2] = [1, 64];
const LONG_WINDOW_SIZES_MEBIBYTES: [u64; 2] = [64, 2048];
const SHARED_DEVICE_SIZES_MEBIBYTES: [u64; 2] = [64, 2048];
/// 子进程非 0 退出时，把它 stderr 的末尾这么多个字符原样记进 `child_failed` 行（登记 V8：交回原文）。
const STDERR_TAIL_CHARACTERS: usize = 1500;

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let rest = arguments.get(1..).unwrap_or(&[]);
    let exit_code = match arguments.first().map(String::as_str) {
        Some("run") => exit_code_of(run_parent(rest)),
        Some("child") => exit_code_of(run_child(rest).map(|()| 0)),
        Some("judge") => exit_code_of(run_judge_command(rest).map(|()| 0)),
        Some("checks") => run_checks_command(),
        Some("plan") => exit_code_of(run_plan_command(rest).map(|()| 0)),
        Some("simulate") => exit_code_of(run_simulate_command(rest).map(|()| 0)),
        Some(_) | None => {
            eprintln!("用法：e174-lease-drop-wait-memory-return run|child|judge|checks|plan|simulate …（见文件头）");
            2
        }
    };
    std::process::exit(exit_code);
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // 源文件从编进来的清单目录直接点名（单元键认得这种写法，把这份文件算进键）；仓根是它往上四层，不经「清单目录/../..」这种出了包目录的路径
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e174_lease_drop_wait_memory_return/main.rs");
    let root = source.ancestors().nth(4).expect("源文件在 <仓根>/scripts/experiment-apparatus/e174_lease_drop_wait_memory_return/ 下").to_path_buf();
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests;
