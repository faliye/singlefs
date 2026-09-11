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
