use super::*;

// ============================================================================================
// E30：计数的全局分配器与 PC-堆。纯 std，不碰 `crates/`。
// ============================================================================================

/// 包 `std::alloc::System`，两份记账：全进程的一份（原子地记在用字节与峰值）和每条线程自己的一份（这条线程自己分出去、放掉的字节，线程本地）。
/// 峰值取「出现过的最大在用字节」，所以先占后放之后读它还是那个最大值。
///
/// 量窗（[`measure_in_a_window`]）读线程自己的那一份：发起量窗的那一条线程在窗里分、放的字节才算，别的线程（并行跑的别的用例）
/// 在窗里分或放多少都不进读数；全进程那一份只给只在单线程进程里才成立的读法（PC-堆 要它与线程那一份都落在登记区间里）。
pub struct CountingAllocator {
    in_use_bytes: AtomicU64,
    pub(super) peak_bytes: AtomicU64,
}

thread_local! {
    /// 这条线程自己的在用字节：它分出去的减它放掉的（有符号：线程可以放掉别的线程分的内存）。常量初始化、没有析构，
    /// 所以在分配器里读写它不会再分配。
    static THIS_THREAD_IN_USE_BYTES: Cell<i64> = const { Cell::new(0) };
    /// 这条线程自己的在用字节出现过的最大值（从上一次重置起）。
    static THIS_THREAD_PEAK_BYTES: Cell<i64> = const { Cell::new(0) };
}

impl CountingAllocator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_use_bytes: AtomicU64::new(0),
            peak_bytes: AtomicU64::new(0),
        }
    }

    fn record_growth(&self, bytes: u64) {
        let in_use_now = self
            .in_use_bytes
            .fetch_add(bytes, Ordering::Relaxed)
            .saturating_add(bytes);
        self.peak_bytes.fetch_max(in_use_now, Ordering::Relaxed);
        let grown_bytes = i64::try_from(bytes).unwrap_or(i64::MAX);
        let thread_in_use_now = THIS_THREAD_IN_USE_BYTES.with(|in_use| {
            let now = in_use.get().saturating_add(grown_bytes);
            in_use.set(now);
            now
        });
        THIS_THREAD_PEAK_BYTES.with(|peak| {
            if thread_in_use_now > peak.get() {
                peak.set(thread_in_use_now);
            }
        });
    }

    fn record_shrink(&self, bytes: u64) {
        self.in_use_bytes.fetch_sub(bytes, Ordering::Relaxed);
        let shrunk_bytes = i64::try_from(bytes).unwrap_or(i64::MAX);
        THIS_THREAD_IN_USE_BYTES
            .with(|in_use| in_use.set(in_use.get().saturating_sub(shrunk_bytes)));
    }

    /// 调用的这条线程自己的在用字节（它分出去的减它放掉的；放掉别的线程分的内存而低于 0 的，读 0）。
    #[must_use]
    pub fn thread_in_use(&self) -> u64 {
        u64::try_from(THIS_THREAD_IN_USE_BYTES.with(Cell::get)).unwrap_or(0)
    }

    /// 调用的这条线程自己的在用字节从上一次重置起出现过的最大值。
    #[must_use]
    pub fn thread_peak(&self) -> u64 {
        u64::try_from(THIS_THREAD_PEAK_BYTES.with(Cell::get)).unwrap_or(0)
    }

    /// 把这条线程的峰值重置成它此刻的在用字节：一个量窗从这里开始。
    pub fn reset_thread_peak_to_thread_in_use(&self) {
        let in_use_now = THIS_THREAD_IN_USE_BYTES.with(Cell::get);
        THIS_THREAD_PEAK_BYTES.with(|peak| peak.set(in_use_now));
    }

    #[must_use]
    pub fn in_use(&self) -> u64 {
        self.in_use_bytes.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn peak(&self) -> u64 {
        self.peak_bytes.load(Ordering::Relaxed)
    }

    /// 把峰值重置成此刻的在用字节：一个量窗从这里开始。
    pub fn reset_peak_to_in_use(&self) {
        self.peak_bytes
            .store(self.in_use_bytes.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

fn bytes_of_layout(layout: Layout) -> u64 {
    u64::try_from(layout.size()).unwrap_or(u64::MAX)
}

#[allow(
    unsafe_code,
    reason = "GlobalAlloc 是 unsafe trait，挂计数分配器只能写 unsafe impl；E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 量的是一次调用前后线程本地的 Layout 字节，VmHWM 是页粒度，锁文件与本机缓存里没有安全的计数库"
)]
// SAFETY: 每个方法原样转给 `System`，只在 `System` 交回成功之后改计数，不读不写被分配的内存。计数两份：全进程那一份是原子量
// （`AtomicU64`，几条线程同时分配也不丢计数）；线程本地那一份（`THIS_THREAD_IN_USE_BYTES`、`THIS_THREAD_PEAK_BYTES`）是常量初始化、
// 没有析构的 `Cell<i64>`，只由它所在的那条线程读写——在分配器里碰它不再分配（不递归进分配器），线程退出时也没有析构可跑，
// 不会碰上「线程本地量已销毁」而 panic。「碰它不再分配」靠平台用原生线程本地存储（这台的 x86_64 Linux 是）；退回按键存的
// 线程本地存储的平台上第一次碰它会分配。计数路径不 unwind：加减全用饱和或回绕的运算，没有会 panic 的那一步。
unsafe impl GlobalAlloc for CountingAllocator {
    #[allow(
        unsafe_code,
        reason = "GlobalAlloc::alloc 的签名要求 unsafe fn，转给 System 要调它的 unsafe 方法；E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 离不开计数分配器（见 impl 块上那一处）"
    )]
    // SAFETY: 调用方按 `GlobalAlloc::alloc` 的契约给 `layout`（尺寸不为零）；原样转给 `System`，契约不变。
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 调用方按 `GlobalAlloc::alloc` 的契约给 `layout`；原样转给 `System.alloc`，契约不变。
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            self.record_growth(bytes_of_layout(layout));
        }
        pointer
    }

    #[allow(
        unsafe_code,
        reason = "GlobalAlloc::alloc_zeroed 的签名要求 unsafe fn，转给 System 要调它的 unsafe 方法；E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 离不开计数分配器（见 impl 块上那一处）"
    )]
    // SAFETY: 调用方按 `GlobalAlloc::alloc_zeroed` 的契约给 `layout`（尺寸不为零）；原样转给 `System`，契约不变。
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 调用方按 `GlobalAlloc::alloc_zeroed` 的契约给 `layout`；原样转给 `System.alloc_zeroed`，契约不变。
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            self.record_growth(bytes_of_layout(layout));
        }
        pointer
    }

    #[allow(
        unsafe_code,
        reason = "GlobalAlloc::dealloc 的签名要求 unsafe fn，还给 System 要调它的 unsafe 方法；E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 离不开计数分配器（见 impl 块上那一处）"
    )]
    // SAFETY: 调用方保证 `pointer` 是用同一个 `layout` 从这个分配器分出来的；这个分配器分出的每一块都来自 `System`，原样还给 `System`。
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: 调用方保证 `pointer` 是用同一个 `layout` 从这个分配器分出来的；它来自 `System`，还给 `System.dealloc`。
        unsafe { System.dealloc(pointer, layout) };
        self.record_shrink(bytes_of_layout(layout));
    }

    #[allow(
        unsafe_code,
        reason = "GlobalAlloc::realloc 的签名要求 unsafe fn，转给 System 要调它的 unsafe 方法；E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 离不开计数分配器（见 impl 块上那一处）"
    )]
    // SAFETY: 调用方保证 `pointer`、`layout`、`new_size` 满足 `GlobalAlloc::realloc` 的契约（`pointer` 用同一个 `layout` 从这个分配器分出，
    // 它来自 `System`）；原样转给 `System`，契约不变。
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: 调用方保证 `pointer`、`layout`、`new_size` 满足 `GlobalAlloc::realloc` 的契约；原样转给 `System.realloc`。
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() {
            let old_bytes = bytes_of_layout(layout);
            let new_bytes = u64::try_from(new_size).unwrap_or(u64::MAX);
            if new_bytes >= old_bytes {
                self.record_growth(new_bytes - old_bytes);
            } else {
                self.record_shrink(old_bytes - new_bytes);
            }
        }
        new_pointer
    }
}

#[global_allocator]
pub(super) static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator::new();

/// PC-堆（登记 5.3）：量窗里分配并逐页写满一份 64 MiB 的 `Vec<u8>`，峰值多占必须落在 `[64 MiB, 64 MiB + 64 KiB]`。
pub(super) const HEAP_POSITIVE_CONTROL_BYTES: u64 = 64 << 20;
pub(super) const HEAP_POSITIVE_CONTROL_SLACK_BYTES: u64 = 65536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HeapPositiveControl {
    pub(super) peak_over_baseline_bytes: u64,
    pub(super) holds: bool,
}

pub(super) fn heap_positive_control(
    allocator: &CountingAllocator,
    upper_slack_bytes: u64,
) -> HeapPositiveControl {
    let process_baseline_bytes = allocator.in_use();
    let thread_baseline_bytes = allocator.thread_in_use();
    allocator.reset_peak_to_in_use();
    allocator.reset_thread_peak_to_thread_in_use();
    let mut buffer =
        vec![0u8; usize::try_from(HEAP_POSITIVE_CONTROL_BYTES).expect("64 MiB 装得进 usize")];
    buffer.fill(0xA5);
    std::hint::black_box(&buffer);
    let process_peak_over_baseline_bytes = allocator.peak().saturating_sub(process_baseline_bytes);
    let peak_over_baseline_bytes = allocator
        .thread_peak()
        .saturating_sub(thread_baseline_bytes);
    drop(buffer);
    HeapPositiveControl {
        peak_over_baseline_bytes,
        holds: heap_control_holds_on_both_readings(
            peak_over_baseline_bytes,
            process_peak_over_baseline_bytes,
            upper_slack_bytes,
        ),
    }
}

/// PC-堆 过的条件：量窗读的线程那一份和全进程那一份都落在登记区间里。PC-堆 在每个被测进程开头跑，那时进程只有这一条线程在动，
/// 两份必须相等地落在区间里；有别的线程在这一步里分了内存，全进程那一份会越出区间，这个进程的堆量就按 V30 作废。
fn heap_control_holds_on_both_readings(
    thread_peak_over_baseline_bytes: u64,
    process_peak_over_baseline_bytes: u64,
    upper_slack_bytes: u64,
) -> bool {
    heap_control_holds(thread_peak_over_baseline_bytes, upper_slack_bytes)
        && heap_control_holds(process_peak_over_baseline_bytes, upper_slack_bytes)
}

/// 峰值多占必须落在 `[64 MiB, 64 MiB + 上沿]`。
pub(super) fn heap_control_holds(peak_over_baseline_bytes: u64, upper_slack_bytes: u64) -> bool {
    peak_over_baseline_bytes >= HEAP_POSITIVE_CONTROL_BYTES
        && peak_over_baseline_bytes <= HEAP_POSITIVE_CONTROL_BYTES + upper_slack_bytes
}

#[cfg(test)]
mod tests;
