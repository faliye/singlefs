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

/// 准入模块（`research/scripts/admission.py`）用来说「登记或调用有错」的退出码；准入模块被信号杀掉、
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

/// 装置开跑之前问准入模块这一趟该不该跑（门禁与实验共用 `research/scripts/admission.py`，
/// 登记表是 `.claude/gate.d/stage-inputs.tsv`）。放行就返回要打在产物最前面的几行；不放行就照准入模块的
/// 退出码退出进程。手敲命令、`cargo run`、`replay.sh` 走的都是装置 `main` 里这一个入口，绕不过去。
/// 说明由准入模块直接打到 stderr。仓库根在编译时写进二进制（`CARGO_MANIFEST_DIR` 往上两级）。
pub fn admit_experiment_run_or_exit(admission_key: &str) -> Vec<String> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let admission_script = repository_root.join("research").join("scripts").join("admission.py");
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
            eprintln!("     → 怎么办：装好 python3、确认仓库里有 research/scripts/admission.py；仓库挪了目录就重新编一次装置");
            std::process::exit(ADMISSION_REGISTRATION_ERROR_EXIT_CODE);
        }
    };
    let admission_stdout = String::from_utf8_lossy(&admission_output.stdout);
    match admission_decision_of(admission_output.status.code(), &admission_stdout) {
        AdmissionDecision::Admitted { header_lines } => header_lines,
        AdmissionDecision::Refused { exit_code } => {
            if exit_code == ADMISSION_REGISTRATION_ERROR_EXIT_CODE && admission_output.status.success() {
                eprintln!("  ✗ 准入模块退 0 却没给出输入指纹行（stdout：{admission_stdout:?}）");
                eprintln!("     → 怎么办：python3 research/scripts/admission.py experiment <仓库根> {admission_key} 单跑一次看它打了什么，修准入模块");
            }
            std::process::exit(exit_code)
        }
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
