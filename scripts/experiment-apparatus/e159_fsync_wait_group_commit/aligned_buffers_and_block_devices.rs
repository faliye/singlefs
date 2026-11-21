//! 对齐缓冲、随机取整的底层、步标记，与真设备、录制设备、睡眠设备三种块设备。

use super::*;

// ============================================================
// 对齐缓冲与 O_DIRECT（模式照 e53_ring_failure.rs 的 AlignedBuffer / O_DIRECT 写法）
// ============================================================

pub(super) struct AlignedBuffer {
    pointer: *mut u8,
    length_in_bytes: usize,
    allocation_layout: Layout,
}
impl AlignedBuffer {
    pub(super) fn new_with_seed(length_in_bytes: usize, seed: u64) -> Self {
        let allocation_layout = Layout::from_size_align(length_in_bytes, ALIGNMENT_BYTES).unwrap();
        // SAFETY: `layout` 由 `Layout::from_size_align` 校验过；`alloc` 要求布局大小非零，调用处传的都是 O_DIRECT 用的正的整块长度。
        let pointer = unsafe { alloc(allocation_layout) };
        let mut state = seed;
        let mut written = 0usize;
        // SAFETY: `pointer` 是刚 `alloc` 出的 `length_in_bytes` 字节；下面的循环在任何读之前按种子写满每个字节，之后才把缓冲交出去。
        let slice = unsafe { std::slice::from_raw_parts_mut(pointer, length_in_bytes) };
        while written < length_in_bytes {
            let word = splitmix64(&mut state).to_le_bytes();
            let take = word.len().min(length_in_bytes - written);
            slice[written..written + take].copy_from_slice(&word[..take]);
            written += take;
        }
        AlignedBuffer { pointer, length_in_bytes, allocation_layout }
    }
    pub(super) fn as_ref(&self) -> &[u8] {
        // SAFETY: `pointer` 指向构造时 `alloc` 出的 `length_in_bytes` 字节，内存未初始化，只当 I/O 缓冲整块写入或交给系统调用读入，Rust 里不按值读未写的字节；切片的借用与 `&self` 一样长，缓冲在 `Drop` 之前不会释放。
        unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) }
    }
    /// O_DIRECT 的读必须把数据读进一块按 `ALIGNMENT_BYTES` 对齐的缓冲区，
    /// 不能读进普通的 `Vec`（`RealBlockDevice::read_at` 的 M9/S1 之外那个真实 bug：
    /// 冒烟跑在虚机里跑到 `read_at` 时炸出 EINVAL，
    /// 因为当时读进的是 `vec![0u8; length]`——那不是对齐缓冲区）。
    fn as_mut(&mut self) -> &mut [u8] {
        // SAFETY: `pointer` 指向构造时 `alloc` 出的 `length_in_bytes` 字节，内存未初始化，只当 I/O 缓冲整块写入或交给系统调用读入，Rust 里不按值读未写的字节；独占借用 `&mut self`，同一时刻没有别的引用指向这块内存，缓冲在 `Drop` 之前不会释放。
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) }
    }
}
impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        // SAFETY: `pointer` 与布局正是构造时 `alloc` 用的指针与布局，`drop` 只跑一次，之后没有别的引用。
        unsafe { dealloc(self.pointer, self.allocation_layout) }
    }
}
// SAFETY: `pointer` 指向这个缓冲独占的分配，没有别的所有者；整份所有权移到别的线程不会产生别名。
unsafe impl Send for AlignedBuffer {}
// SAFETY: 共享引用只给出 `as_ref` 的只读切片，缓冲内容在构造之后只有 `&mut self` 的方法才改，所以多个线程同时持 `&AlignedBuffer` 读不会冲突。
unsafe impl Sync for AlignedBuffer {}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// [0, 1) 上的均匀变量，用来做「整数部分 + 以小数部分为概率再加 1」的随机取整（第五节 5.3）。
pub(super) fn uniform_unit_interval(state: &mut u64) -> f64 {
    (splitmix64(state) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}


// ============================================================
// 块设备抽象：真设备（O_DIRECT）与录制假设备（单测用，不碰 I/O）
// ============================================================

/// 一步操作的记号，供 S1（次序对拍）与 M1/M2/M9/M10 的单测直接断言。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum StepMark {
    Write { kind: UnitKind },
    Barrier,
    Read { kind: UnitKind },
}

pub(super) trait BlockDevice: Send + Sync {
    fn write_at(&self, offset: u64, buffer: &[u8], kind: UnitKind);
    fn read_at(&self, offset: u64, length: usize, kind: UnitKind) -> Vec<u8>;
    fn barrier(&self);
}

pub(super) struct RealBlockDevice {
    file: File,
}
impl RealBlockDevice {
    pub(super) fn open(path: &str) -> std::io::Result<RealBlockDevice> {
        let file = OpenOptions::new().read(true).write(true).custom_flags(O_DIRECT).open(path)?;
        Ok(RealBlockDevice { file })
    }
}
impl BlockDevice for RealBlockDevice {
    fn write_at(&self, offset: u64, buffer: &[u8], _kind: UnitKind) {
        self.file.write_at(buffer, offset).expect("write_at 失败：设备或对齐有问题");
    }
    fn read_at(&self, offset: u64, length: usize, _kind: UnitKind) -> Vec<u8> {
        let mut buffer = AlignedBuffer::new_with_seed(length, 0);
        self.file.read_exact_at(buffer.as_mut(), offset).expect("read_at 失败：设备或对齐有问题");
        buffer.as_ref().to_vec()
    }
    fn barrier(&self) {
        self.file.sync_data().expect("sync_data 失败");
    }
}

/// 单测用：不碰真 I/O，只把每一步记下来，供 S1、M1、M2、M9、M10 断言用（`mutation-sampling.md`
/// 第四类要求单测取样点覆盖实验会跑到的取值——这份记录只覆盖控制流与调用顺序，不覆盖真实时延，
/// 时延相关的 M3/M4/M6 由 `SleepingBlockDevice`（见下）在真线程真时钟上跑，不需要真设备）。
pub(super) struct RecordingBlockDevice {
    log: Mutex<Vec<(Instant, StepMark)>>,
}
impl RecordingBlockDevice {
    pub(super) fn new() -> RecordingBlockDevice {
        RecordingBlockDevice { log: Mutex::new(Vec::new()) }
    }
    pub(super) fn drain(&self) -> Vec<(Instant, StepMark)> {
        std::mem::take(&mut self.log.lock().unwrap())
    }
}
impl BlockDevice for RecordingBlockDevice {
    fn write_at(&self, _offset: u64, _buffer: &[u8], kind: UnitKind) {
        self.log.lock().unwrap().push((Instant::now(), StepMark::Write { kind }));
    }
    fn read_at(&self, _offset: u64, length: usize, kind: UnitKind) -> Vec<u8> {
        self.log.lock().unwrap().push((Instant::now(), StepMark::Read { kind }));
        vec![0u8; length]
    }
    fn barrier(&self) {
        self.log.lock().unwrap().push((Instant::now(), StepMark::Barrier));
    }
}

/// 单测用：不碰真设备，但用真线程 + 真 `Instant`；每次 `barrier()` 睡一段固定时长模拟持久化延迟，
/// 让 PC1/PC2/PC3 那一类「结构对不对」的检查能在宿主上跑，不必起虚机（M3/M4/M5/M6 的结构半用它测；
/// 真实设备上的量级仍旧只能在虚机里量，见跑前登记第九节）。
pub(super) struct SleepingBlockDevice {
    pub(super) barrier_delay: Duration,
}
impl BlockDevice for SleepingBlockDevice {
    fn write_at(&self, _offset: u64, _buffer: &[u8], _kind: UnitKind) {}
    fn read_at(&self, _offset: u64, length: usize, _kind: UnitKind) -> Vec<u8> {
        vec![0u8; length]
    }
    fn barrier(&self) {
        std::thread::sleep(self.barrier_delay);
    }
}
