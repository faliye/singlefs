//! E7 索引 harness 的度量口径。
//!
//! I/O 在 `main.rs`，本模块只放**可单独验证的算术**——
//! 口径算错会静默污染所有实验数字，而算术是能被单测钉死的那部分。

/// 一轮 I/O 的原始观测。时间用纳秒，避免毫秒取整把快的那档抹平。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub operation_count: u64,
    pub bytes_per_operation: u64,
    pub elapsed_nanoseconds: u64,
}

impl Sample {
    pub fn total_bytes(&self) -> u64 {
        self.operation_count.saturating_mul(self.bytes_per_operation)
    }

    /// MiB/s。耗时为 0 时返回 None——**读不到 ≠ 读到 0**：
    /// 除零如果悄悄返回 0 或 inf，会被当成一个真实测量值参与判定。
    pub fn mebibytes_per_second(&self) -> Option<f64> {
        if self.elapsed_nanoseconds == 0 {
            return None;
        }
        let elapsed_seconds = self.elapsed_nanoseconds as f64 / 1e9;
        Some((self.total_bytes() as f64 / (1024.0 * 1024.0)) / elapsed_seconds)
    }

    /// 每秒操作数。同样在耗时为 0 时返回 None。
    pub fn iops(&self) -> Option<f64> {
        if self.elapsed_nanoseconds == 0 {
            return None;
        }
        Some(self.operation_count as f64 / (self.elapsed_nanoseconds as f64 / 1e9))
    }
}

/// 一条给宿主解析的结果行。前缀是 harness 的抓取锚点，改它要同时改 vm-bench.sh。
pub fn result_line(name: &str, sample: &Sample) -> String {
    let mebibytes_per_second_text = sample
        .mebibytes_per_second()
        .map(|mebibytes_per_second| format!("{mebibytes_per_second:.3}"))
        .unwrap_or_else(|| "NA".into());
    let iops = sample
        .iops()
        .map(|operations_per_second| format!("{operations_per_second:.1}"))
        .unwrap_or_else(|| "NA".into());
    format!(
        "E7RESULT name={name} ops={} bytes_per_op={} elapsed_ns={} total_bytes={} mib_per_s={mebibytes_per_second_text} iops={iops}",
        sample.operation_count,
        sample.bytes_per_operation,
        sample.elapsed_nanoseconds,
        sample.total_bytes()
    )
}

/// 结果行发射器。**它存在的唯一理由是让「漏了一行」变成可检出的事实**——
/// 控制台会被 BIOS 转义序列、内核日志、串口噪声污染，
/// 宿主那边只要有一行没被抓到，实验结果就静默少了一项而没人知道。
/// 收尾行带上累计条数，宿主比对条数对不上就整轮作废。
#[derive(Default)]
pub struct Emitter {
    emitted: u64,
}

impl Emitter {
    pub fn new() -> Self {
        Self { emitted: 0 }
    }

    /// 发一条结果，返回该打印的整行。
    pub fn emit(&mut self, name: &str, sample: &Sample) -> String {
        self.emitted += 1;
        result_line(name, sample)
    }

    /// 发一条自由格式的结果（非 Sample 形态，例如设备大小）。
    pub fn emit_raw(&mut self, body: &str) -> String {
        self.emitted += 1;
        format!("E7RESULT {body}")
    }

    /// 收尾行。`emitted` 计入自身，所以宿主该抓到的总行数就等于这个数。
    pub fn finish(&mut self) -> String {
        self.emitted += 1;
        format!("E7RESULT name=done emitted={}", self.emitted)
    }
}

/// 准入模块（`scripts/gate/admission.py`）用来说「登记或调用有错」的退出码；准入模块被信号杀掉、
/// 放行却没给指纹行时装置也照这个码退出。
const ADMISSION_REGISTRATION_ERROR_EXIT_CODE: i32 = 2;

/// 产物头那几行的前缀：`replay.sh` 逐字节比对之前删掉它们，`name=done` 的条数不数它们。
const ADMISSION_HEADER_PREFIX: &str = "E7INPUT ";

/// 准入模块答完之后装置这一侧怎么办。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionDecision {
    /// 放行：这几行原样打在产物最前面（输入指纹，强制重跑时还有理由）。
    Admitted { header_lines: Vec<String> },
    /// 不放行：照这个码退出，一行结果都不打。77 输入自上次产物以来没变，3 前提没齐，2 登记有错。
    Refused { exit_code: i32 },
}

/// 把准入模块的退出码与 stdout 翻成判定。放行却没有指纹行、或 stdout 里混进别的行，按登记错处理：
/// 放行的产物必须带指纹，不然下一次准入判不了「输入没变」。被信号杀掉（没有退出码）同样不许当成放行。
pub fn admission_decision_of(exit_code: Option<i32>, admission_stdout: &str) -> AdmissionDecision {
    match exit_code {
        Some(0) => {
            let header_lines: Vec<String> = admission_stdout.lines().map(String::from).collect();
            let carries_fingerprint = header_lines
                .iter()
                .any(|header_line| header_line.starts_with("E7INPUT name=input_fingerprint "));
            let only_header_lines = header_lines
                .iter()
                .all(|header_line| header_line.starts_with(ADMISSION_HEADER_PREFIX));
            if carries_fingerprint && only_header_lines {
                AdmissionDecision::Admitted { header_lines }
            } else {
                AdmissionDecision::Refused { exit_code: ADMISSION_REGISTRATION_ERROR_EXIT_CODE }
            }
        }
        Some(refusing_exit_code) => AdmissionDecision::Refused { exit_code: refusing_exit_code },
        None => AdmissionDecision::Refused { exit_code: ADMISSION_REGISTRATION_ERROR_EXIT_CODE },
    }
}

/// 装置开跑之前问准入模块这一趟该不该跑（门禁与实验共用 `scripts/gate/admission.py`，
/// 登记表是 `.claude/gate.d/stage-inputs.tsv`）。放行就返回要打在产物最前面的几行；不放行就照准入模块的
/// 退出码退出进程。手敲命令、`cargo run`、`replay.sh` 走的都是装置 `main` 里这一个入口，绕不过去。
/// 说明由准入模块直接打到 stderr。仓库根在编译时写进二进制（`CARGO_MANIFEST_DIR` 往上两级）。
pub fn admit_experiment_run_or_exit(admission_key: &str) -> Vec<String> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let admission_script = repository_root.join("scripts").join("gate").join("admission.py");
    let admission_output = match std::process::Command::new("python3")
        .arg(&admission_script)
        .arg("experiment")
        .arg(&repository_root)
        .arg(admission_key)
        .stderr(std::process::Stdio::inherit())
        .output()
    {
        Ok(admission_output) => admission_output,
        Err(error) => {
            eprintln!("  ✗ 准入模块起不来（python3 {}）：{error}", admission_script.display());
            eprintln!("     → 怎么办：装好 python3、确认仓库里有 scripts/gate/admission.py；仓库挪了目录就重新编一次装置");
            std::process::exit(ADMISSION_REGISTRATION_ERROR_EXIT_CODE);
        }
    };
    let admission_stdout = String::from_utf8_lossy(&admission_output.stdout);
    match admission_decision_of(admission_output.status.code(), &admission_stdout) {
        AdmissionDecision::Admitted { header_lines } => header_lines,
        AdmissionDecision::Refused { exit_code } => {
            if exit_code == ADMISSION_REGISTRATION_ERROR_EXIT_CODE && admission_output.status.success() {
                eprintln!("  ✗ 准入模块退 0 却没给出输入指纹行（stdout：{admission_stdout:?}）");
                eprintln!("     → 怎么办：python3 scripts/gate/admission.py experiment <仓库根> {admission_key} 单跑一次看它打了什么，修准入模块");
            }
            std::process::exit(exit_code)
        }
    }
}

/// 「本次未跑」那一行的记号：别的实验的产物、拿来当期望值的留存产物读不到时打它（写法照 crates 那一侧
/// `NOT_RUN_<原因> test=<用例> reason=<一句>` 的约定，`singlefs_harness::gpu_memory_ledger::gpu_not_run_line`）。
/// `research/` 可以整个删掉（`.claude/rules/directory-layout.md`「research/：可以整个删掉的研究目录」），
/// 读它的装置与单测一律在运行时读、读不到报这一行，不用 `include_str!`（产物不在就编不过），也不 panic。
/// 路径由调用处写成字面的 `concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<文件名>")` 交进来：函数单元键
/// （`scripts/gate/lib_function_units.py`）折得出这种写法，文件在时把它的内容算进键（与 `include_str!` 一样），不在时这一条拒算、每趟现判，
/// 不会把一次「本次未跑」当成判过的绿复用下去；在这里按文件名拼路径就折不出，每一条都拒算。
pub const ARCHIVED_PRODUCT_NOT_RUN_MARKER: &str = "NOT_RUN_ARCHIVED_PRODUCT";

/// 装置 `main` 读不到别的实验的产物时的退出码：「本次未跑」，不是通过，也不是红（`replay.sh` 把这一行记本次未跑）。
pub const NOT_RUN_EXIT_CODE: i32 = 77;

/// 读不到那一份时那一行：记号、用例名、读不到的是 `research/results/` 下的哪一份、怎么取回。
pub fn archived_product_not_run_line(test_name: &str, path: &str, error: &str) -> String {
    let file_name = std::path::Path::new(path).file_name().map_or(path.into(), |name| name.to_string_lossy());
    format!(
        "{ARCHIVED_PRODUCT_NOT_RUN_MARKER} test={test_name} reason=research/results/{file_name} 读不到（{error}），这一项本次未跑；要跑先从 git 历史取回：git log --all --diff-filter=D --name-only -- \"*{file_name}\""
    )
}

/// 单测里报本次未跑：把那一行写进进程的标准输出。写的是 `std::io::stdout()` 本身，不经 `println!`：
/// libtest 只截 `print!` 一族，这样不带 `--nocapture` 的 `cargo test` 输出里也看得见这一行。
pub fn report_not_run_line_on_stdout(line: &str) {
    use std::io::Write as _;
    writeln!(std::io::stdout().lock(), "{line}").expect("写不出本次未跑那一行");
}

/// 读那一份（路径照 [`ARCHIVED_PRODUCT_NOT_RUN_MARKER`] 那一段写成字面的 `concat!`）；读不到就报一行本次未跑
/// （[`report_not_run_line_on_stdout`]）、交 `None`，调用方照常返回。
pub fn read_archived_product_or_report_not_run(test_name: &str, path: &str) -> Option<String> {
    read_archived_product_or_write_not_run_line(test_name, path, &mut std::io::stdout().lock())
}

/// [`read_archived_product_or_report_not_run`] 的本体：本次未跑那一行写进 `not_run_output`。自检用例交一块内存缓冲，
/// 不往真的标准输出打：门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格把输出里的每一行记号当成一条本次未跑。
fn read_archived_product_or_write_not_run_line(test_name: &str, path: &str, not_run_output: &mut dyn std::io::Write) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(error) => {
            writeln!(not_run_output, "{}", archived_product_not_run_line(test_name, path, &error.to_string())).expect("写不出本次未跑那一行");
            None
        }
    }
}

/// 装置 `main` 用：按次序读这几份（路径写法同上），全读得到就按次序交出；读不到的每一份各往 stderr 打一行本次未跑，
/// 然后退 [`NOT_RUN_EXIT_CODE`]，一行结果都不打。
pub fn read_archived_products_or_exit_not_run(device_name: &str, paths: &[&str]) -> Vec<String> {
    let mut texts = Vec::with_capacity(paths.len());
    let mut missing = 0usize;
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(text) => texts.push(text),
            Err(error) => {
                eprintln!("{}", archived_product_not_run_line(device_name, path, &error.to_string()));
                missing += 1;
            }
        }
    }
    if missing > 0 {
        std::process::exit(NOT_RUN_EXIT_CODE);
    }
    texts
}

/// E175 的装置源码：research 工作区不编它，`replay.sh` 的 `driver_e175` 把它拷进仓副本的 `crates/singlefs-harness/tests/` 编、按名字起一条用例。
/// 这里钉住驱动依赖它的那几样（用例名、标了 `#[ignore]`、驱动设的几个环境变量），源码改了名字驱动就起不来，这一条先红。
#[cfg(test)]
mod e175_device_source_tests {
    const DEVICE_SOURCE: &str = include_str!("e175_product_path_inode_watermark_regression/device-source.rs");

    #[test]
    fn the_e175_device_source_carries_the_ignored_case_and_the_variables_the_replay_driver_uses() {
        let case = "fn run_one_family_on_one_arm_into_the_result_file()";
        let case_at = DEVICE_SOURCE.find(case).expect("装置源码里要有 driver_e175 按 --exact 起的那条用例");
        assert!(DEVICE_SOURCE[..case_at].trim_end().rsplit('\n').take(3).any(|line| line.trim_start().starts_with("#[ignore")), "那条用例要标 #[ignore]：driver_e175 带 --ignored 起它");
        for variable in ["E175_ARM", "E175_FAMILY", "E175_RESULT_FILE", "E175_WORKER_THREADS", "E175_TIME_BUDGET_SECONDS"] {
            assert!(DEVICE_SOURCE.contains(&format!("\"{variable}\"")), "装置源码要读 driver_e175 设的 {variable}");
        }
    }
}

#[cfg(test)]
mod archived_product_tests {
    use super::*;

    #[test]
    fn a_missing_archived_product_is_reported_not_run_instead_of_read() {
        let path = std::env::temp_dir().join(format!("no-such-product-for-the-not-run-test-{}.out", std::process::id()));
        let path = path.to_str().expect("临时目录的路径是 UTF-8");
        assert!(!std::path::Path::new(path).exists(), "这一条要的是一份不在的产物");
        let mut not_run_output = Vec::new();
        assert_eq!(read_archived_product_or_write_not_run_line("lib::archived_product_tests", path, &mut not_run_output), None);
        let written = String::from_utf8(not_run_output).expect("本次未跑那一行是 UTF-8");
        assert!(written.starts_with("NOT_RUN_ARCHIVED_PRODUCT test=lib::archived_product_tests reason=research/results/no-such-product-for-the-not-run-test-") && written.ends_with('\n'),
                "读不到时写出一整行本次未跑：{written}");
        let line = archived_product_not_run_line("lib::archived_product_tests", path, "No such file or directory (os error 2)");
        assert!(line.starts_with("NOT_RUN_ARCHIVED_PRODUCT test=lib::archived_product_tests reason=research/results/no-such-product-for-the-not-run-test-"), "{line}");
        assert!(line.contains("本次未跑") && !line.contains(&*std::env::temp_dir().to_string_lossy()), "那一行只写文件名：{line}");
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;

    #[test]
    fn exit_code_zero_with_a_fingerprint_line_admits_and_keeps_every_header_line_in_order() {
        let admission_stdout = "E7INPUT name=input_fingerprint key=E142 sha256=ab files=145\nE7INPUT name=forced_rerun key=E142 overrode_unchanged=true reason=复核\n";
        assert_eq!(
            admission_decision_of(Some(0), admission_stdout),
            AdmissionDecision::Admitted {
                header_lines: vec![
                    "E7INPUT name=input_fingerprint key=E142 sha256=ab files=145".to_string(),
                    "E7INPUT name=forced_rerun key=E142 overrode_unchanged=true reason=复核".to_string(),
                ]
            },
            "放行时两行产物头原样、按次序交回"
        );
    }

    #[test]
    fn exit_code_zero_without_a_fingerprint_line_is_refused_as_a_registration_error() {
        assert_eq!(admission_decision_of(Some(0), ""), AdmissionDecision::Refused { exit_code: 2 }, "放行却没有指纹行不许放行");
        assert_eq!(
            admission_decision_of(Some(0), "E7INPUT name=forced_rerun key=E142 reason=复核\n"),
            AdmissionDecision::Refused { exit_code: 2 },
            "只有理由行、没有指纹行不许放行"
        );
    }

    #[test]
    fn exit_code_zero_with_a_stray_non_header_line_is_refused_as_a_registration_error() {
        assert_eq!(
            admission_decision_of(Some(0), "E7INPUT name=input_fingerprint key=E142 sha256=ab files=1\n放行 E142\n"),
            AdmissionDecision::Refused { exit_code: 2 },
            "stdout 混进不是 E7INPUT 的行会被写进产物，不许放行"
        );
    }

    #[test]
    fn refusing_exit_codes_pass_through_unchanged_and_a_signal_death_is_a_registration_error() {
        assert_eq!(admission_decision_of(Some(77), ""), AdmissionDecision::Refused { exit_code: 77 }, "输入没变照 77 退出");
        assert_eq!(admission_decision_of(Some(3), ""), AdmissionDecision::Refused { exit_code: 3 }, "前提没齐照 3 退出");
        assert_eq!(admission_decision_of(Some(1), ""), AdmissionDecision::Refused { exit_code: 1 }, "准入模块抛异常退 1 照样不放行");
        assert_eq!(admission_decision_of(None, ""), AdmissionDecision::Refused { exit_code: 2 }, "被信号杀掉不许当成放行");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_bytes_is_operation_count_times_bytes_per_operation() {
        let sample = Sample {
            operation_count: 256,
            bytes_per_operation: 4096,
            elapsed_nanoseconds: 1,
        };
        assert_eq!(sample.total_bytes(), 1_048_576);
    }

    #[test]
    fn one_mebibyte_in_one_second_is_one_mebibyte_per_second() {
        let sample = Sample {
            operation_count: 1,
            bytes_per_operation: 1024 * 1024,
            elapsed_nanoseconds: 1_000_000_000,
        };
        assert_eq!(sample.mebibytes_per_second(), Some(1.0));
        assert_eq!(sample.iops(), Some(1.0));
    }

    /// 耗时为 0 必须报 None，不许退化成 0 或 inf ——
    /// 一个悄悄变成 0 的吞吐会被当成「测到了，很慢」，而真相是「没测到」。
    #[test]
    fn zero_elapsed_is_not_a_measurement() {
        let sample = Sample {
            operation_count: 10,
            bytes_per_operation: 4096,
            elapsed_nanoseconds: 0,
        };
        assert_eq!(sample.mebibytes_per_second(), None);
        assert_eq!(sample.iops(), None);
        assert!(result_line("x", &sample).contains("mib_per_s=NA"));
        assert!(result_line("x", &sample).contains("iops=NA"));
    }

    /// 结果行必须带 harness 认的锚点前缀，否则宿主一条都抓不到。
    #[test]
    fn result_line_carries_the_anchor_prefix() {
        let sample = Sample {
            operation_count: 2,
            bytes_per_operation: 512,
            elapsed_nanoseconds: 1_000_000,
        };
        let line = result_line("seq_write", &sample);
        assert!(line.starts_with("E7RESULT "), "锚点前缀丢了：{line}");
        assert!(line.contains("name=seq_write"));
        assert!(line.contains("total_bytes=1024"));
    }

    /// 收尾行报的条数必须**含它自己**，否则宿主的比对会永远差一。
    #[test]
    fn done_count_includes_itself() {
        let sample = Sample {
            operation_count: 1,
            bytes_per_operation: 1,
            elapsed_nanoseconds: 1,
        };
        let mut emitter = Emitter::new();
        let _ = emitter.emit_raw("name=device_size bytes=1");
        let _ = emitter.emit("a", &sample);
        let _ = emitter.emit("b", &sample);
        assert_eq!(emitter.finish(), "E7RESULT name=done emitted=4");
    }

    /// 每一条发出去的行都必须带锚点，收尾行也不例外。
    #[test]
    fn every_emitted_line_carries_the_anchor() {
        let sample = Sample {
            operation_count: 1,
            bytes_per_operation: 1,
            elapsed_nanoseconds: 1,
        };
        let mut emitter = Emitter::new();
        for line in [emitter.emit_raw("name=x"), emitter.emit("y", &sample), emitter.finish()] {
            assert!(line.starts_with("E7RESULT "), "锚点丢了：{line}");
        }
    }

    /// ops 极大时不许 panic —— 溢出要饱和，不要在实验跑到一半炸掉。
    #[test]
    fn huge_operation_count_saturates_instead_of_panicking() {
        let sample = Sample {
            operation_count: u64::MAX,
            bytes_per_operation: 4096,
            elapsed_nanoseconds: 1_000_000_000,
        };
        assert_eq!(sample.total_bytes(), u64::MAX);
    }
}

/// E12 用：把「删除判定」这条路径上的设备 I/O 数出来。
/// 挂钟受虚机与宿主影响，**I/O 计数是结构性的量**，所以两者都报。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct IoCounters {
    pub reads: u64,
    pub writes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl IoCounters {
    /// 每次操作平均触发多少次设备 I/O。ops 为 0 时返回 None——
    /// 除零悄悄返回 0 会被当成「测到了，很省」，而真相是「没测」。
    pub fn io_per_op(&self, operation_count: u64) -> Option<f64> {
        if operation_count == 0 {
            return None;
        }
        Some((self.reads + self.writes) as f64 / operation_count as f64)
    }
}

/// 计数器所在的页号。每页 `per_page` 个计数器。
pub fn page_of(block: u64, per_page: u64) -> u64 {
    assert!(per_page > 0, "每页计数器数必须为正");
    block / per_page
}

/// 定容 LRU。`touch` 返回被逐出的页号（若有）。
/// 它是本实验判别力的来源：**缓存边界不生效，整个实验就测不出东西**。
pub struct Lru {
    capacity_in_pages: usize,
    order: std::collections::VecDeque<u64>,
    dirty: std::collections::HashSet<u64>,
}

impl Lru {
    pub fn new(capacity_in_pages: usize) -> Self {
        assert!(capacity_in_pages > 0, "缓存容量必须为正");
        Self { capacity_in_pages, order: std::collections::VecDeque::with_capacity(capacity_in_pages), dirty: Default::default() }
    }
    pub fn contains(&self, page: u64) -> bool {
        self.order.contains(&page)
    }
    pub fn resident_page_count(&self) -> usize {
        self.order.len()
    }
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
    pub fn is_dirty(&self, page: u64) -> bool {
        self.dirty.contains(&page)
    }
    pub fn mark_dirty(&mut self, page: u64) {
        self.dirty.insert(page);
    }
    /// 访问一页：已在缓存则提到最近端并返回 None；不在则插入，
    /// 满了就逐出最久未用的那一页并返回它。
    pub fn touch(&mut self, page: u64) -> Option<u64> {
        if let Some(position_in_recency_order) = self.order.iter().position(|&cached_page| cached_page == page) {
            self.order.remove(position_in_recency_order);
            self.order.push_back(page);
            return None;
        }
        let evicted = if self.order.len() >= self.capacity_in_pages { self.order.pop_front() } else { None };
        self.order.push_back(page);
        evicted
    }
    pub fn take_dirty(&mut self, page: u64) -> bool {
        self.dirty.remove(&page)
    }
    pub fn drain_dirty(&mut self) -> Vec<u64> {
        let mut dirty_pages: Vec<u64> = self.dirty.drain().collect();
        dirty_pages.sort_unstable();
        dirty_pages
    }
}

#[cfg(test)]
mod e12_tests {
    use super::*;

    #[test]
    fn io_per_op_counts_both_directions() {
        let counters = IoCounters { reads: 3, writes: 1, bytes_read: 0, bytes_written: 0 };
        assert_eq!(counters.io_per_op(2), Some(2.0));
    }

    /// 零操作不是「零 I/O」，是「没测」。
    #[test]
    fn zero_operations_is_not_a_measurement() {
        let counters = IoCounters { reads: 5, writes: 5, bytes_read: 0, bytes_written: 0 };
        assert_eq!(counters.io_per_op(0), None);
    }

    #[test]
    fn page_index_is_block_over_per_page() {
        assert_eq!(page_of(0, 512), 0);
        assert_eq!(page_of(511, 512), 0);
        assert_eq!(page_of(512, 512), 1);
    }

    /// LRU 必须真的逐出——不逐出的话「工作集超过缓存」这个自变量就是假的，
    /// 整个 E12 会测出「引用计数免费」这个错误结论。
    #[test]
    fn lru_evicts_least_recently_used() {
        let mut cache = Lru::new(2);
        assert_eq!(cache.touch(1), None);
        assert_eq!(cache.touch(2), None);
        assert_eq!(cache.touch(1), None); // 1 变成最近使用
        assert_eq!(cache.touch(3), Some(2), "该逐出的是最久未用的 2");
        assert!(cache.contains(1) && cache.contains(3) && !cache.contains(2));
        assert_eq!(cache.resident_page_count(), 2);
    }

    /// 容量以内不许逐出，否则会虚报随机读。
    #[test]
    fn lru_does_not_evict_within_capacity() {
        let mut cache = Lru::new(4);
        for page in 0..4 {
            assert_eq!(cache.touch(page), None);
        }
        assert_eq!(cache.resident_page_count(), 4);
    }

    #[test]
    fn dirty_pages_are_tracked_and_drained_once() {
        let mut cache = Lru::new(4);
        cache.touch(7);
        cache.mark_dirty(7);
        assert!(cache.is_dirty(7));
        assert_eq!(cache.drain_dirty(), vec![7]);
        assert!(!cache.is_dirty(7), "drain 之后不该还是脏的");
    }
}
