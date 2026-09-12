//! E140：单元含头逼出的跨单元页与读侧拷贝 —— D4 已定项 5「含头」在读路径上没记过的两层代价。
//!
//! ## 立项
//!
//! D4（校验和位置）已定项 5：单元恒 32768 字节，自描述头、声明长度、净荷、补齐全在这 32768 之内；
//! D18（块里携带什么信息）已定项 7 / 14 把码 1 头定成 105（含 nonce / MAC 预留位 133）。
//! 于是每单元净荷 P = 32768 − h 不是 4096 的整数倍，且净荷从偏移 h 起。后果两层：
//!
//! 1. 文件的 4 KiB 页有 s(P) = (4096 − gcd(4096, P)) / P 的比例跨在两个单元上，读它要读两个单元、验两次。
//! 2. 页缓存页 / O_DIRECT 用户缓冲拿不到零拷贝，每读一页多一次 memcpy；顺序读要把每单元净荷搬进连续缓冲。
//!
//! 格式内唯一的替代：净荷补齐到 28672（7 页），头与补齐占一个页，两层后果全消，
//! 代价是顺序读每用户字节多 32768/28672 − 1 = 14.3% 设备字节。
//!
//! ## 臂
//!
//! | 臂 | 净荷 | 页到单元 | 拷贝 |
//! |---|---|---|---|
//! | `padded` | 28672 | 页 k 在单元 k/7，从不跨 | 无，就地消费 |
//! | `h133` / `h105` | 32635 / 32663 | 页 k 从文件偏移 4096k 起，跨则读两个单元 | 随机臂 4 KiB、顺序臂每单元 P 字节 |
//! | `h133_nocopy` / `h105_nocopy` | 同上 | 同上 | 无（把跨单元与拷贝拆开） |
//! | `ctl_copy1m` | 28672 | 同 `padded` | 每次多搬 1 MiB（拷贝判别力对照） |
//!
//! 负载：`rand` QD=1、`randq` QD=16、`seq`（用户字节 512 MiB，I/O 固定 1 MiB）。
//! 判据与失败条款写在 `research/prompts/e140-preregistration.md`，装置写之前。

use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::time::Instant;

const O_DIRECT: i32 = 0o40000;
const ALIGN: usize = 4096;
/// D4 已定项 1 / 5：数据单元恒 32768 字节，含头。
const UNIT_BYTES: usize = 32768;
/// 用户按页消费数据；随机小读也是 4 KiB（D4 已定项 1 的原话）。
const PAGE_BYTES: usize = 4096;
/// D18 已定项 7 登记的码 1 头（format-const `DATA_UNIT_HEADER_BYTES`）。
const HEADER_REGISTERED_BYTES: usize = 105;
/// D18 已定项 14：含 nonce / MAC 预留位之后的码 1 头。
const HEADER_WITH_RESERVE_BYTES: usize = 133;
/// 替代形态：净荷补齐到 7 个整页，头与补齐占一个页。
const PADDED_PAYLOAD_BYTES: usize = 28672;
/// 顺序臂的 I/O 大小，与净荷布局无关：预读定它。
const SEQ_IO_BYTES: usize = 1024 * 1024;
/// 顺序臂要交给用户的字节数。
const SEQ_USER_BYTES: u64 = 512 * 1024 * 1024;
/// `randq` 臂的并发度。
const QUEUE_DEPTH: usize = 16;
/// 拷贝判别力对照每次多搬的字节数。
const CONTROL_COPY_BYTES: usize = 1024 * 1024;

/// O_DIRECT 要求缓冲、偏移、长度三样都对齐。
struct Aligned {
    pointer: *mut u8,
    length: usize,
    layout: Layout,
}
// 每个线程独占一个缓冲，不共享，跨线程移动是安全的。
unsafe impl Send for Aligned {}

impl Aligned {
    fn new(length: usize) -> Self {
        let layout = Layout::from_size_align(length, ALIGN).expect("对齐参数非法");
        // SAFETY: layout 非零长且对齐合法；分配失败下面立刻断言。
        let pointer = unsafe { alloc(layout) };
        assert!(!pointer.is_null(), "分配失败");
        // SAFETY: pointer 指向刚分配的 length 字节。
        unsafe { std::ptr::write_bytes(pointer, 0xA5, length) };
        Self { pointer, length, layout }
    }
    fn as_slice(&self) -> &[u8] {
        // SAFETY: pointer 在 Self 存活期间有效且长 length。
        unsafe { std::slice::from_raw_parts(self.pointer, self.length) }
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: 同上，且 &mut self 保证独占。
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length) }
    }
}

impl Drop for Aligned {
    fn drop(&mut self) {
        // SAFETY: 与 new 里同一个 layout。
        unsafe { dealloc(self.pointer, self.layout) }
    }
}

/// 确定性伪随机（xorshift64*）：同一个种子必须给出同一串页号。
fn next_random(state: &mut u64) -> u64 {
    let mut value = *state;
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    *state = value;
    value.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// 内核记的「这个进程让存储层取了多少字节」——本实验的校验路径，与程序记账不共享代码。
fn proc_read_bytes() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/io").ok()?;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("read_bytes:") {
            return value.trim().parse().ok();
        }
    }
    None
}

/// 含头 h 的单元装多少净荷。
fn payload_bytes(header_bytes: usize) -> usize {
    UNIT_BYTES - header_bytes
}

/// gcd(4096, P)：P 的 2 的幂因子，封顶 4096。
fn page_gcd(payload: usize) -> usize {
    PAGE_BYTES.min(1usize << payload.trailing_zeros().min(12))
}

/// 一整个周期里跨单元的页占比的闭式：(4096 − gcd(4096, P)) / P。
fn straddle_fraction(payload: usize) -> f64 {
    (PAGE_BYTES - page_gcd(payload)) as f64 / payload as f64
}

/// 页 k 的首字节落在哪个单元、净荷内偏移多少（净荷连续打包，文件偏移 4096k）。
fn locate_page(page_index: u64, payload: usize) -> (u64, usize) {
    let file_offset = page_index * PAGE_BYTES as u64;
    (file_offset / payload as u64, (file_offset % payload as u64) as usize)
}

/// 页 k 是否跨在两个单元上。
fn page_straddles(page_index: u64, payload: usize) -> bool {
    locate_page(page_index, payload).1 + PAGE_BYTES > payload
}

/// 穷举一段页号里跨单元的页数（与闭式互为校验）。
fn straddle_count(payload: usize, page_count: u64) -> u64 {
    (0..page_count).filter(|&k| page_straddles(k, payload)).count() as u64
}

/// 随机页读期望读几个单元。
fn expected_units_per_page(payload: usize) -> f64 {
    1.0 + straddle_fraction(payload)
}

/// 顺序读每用户字节要读多少设备字节。
fn device_bytes_per_user_byte(payload: usize) -> f64 {
    UNIT_BYTES as f64 / payload as f64
}

/// 装 n 个用户字节要几个单元。
fn units_for_user_bytes(user_bytes: u64, payload: usize) -> u64 {
    (user_bytes + payload as u64 - 1) / payload as u64
}

/// 两条臂一赢一输时的交叉点：随机页读占比 p* 使混合代价相等。
/// 代价按每用户字节的纳秒算：random 用 ns_per_op / 4096，sequential 用 1e9 / bytes_per_second。
/// 返回 None 表示同向（没有交叉点）。
fn crossover_random_share(
    random_ns_per_byte_padded: f64,
    random_ns_per_byte_header: f64,
    sequential_ns_per_byte_padded: f64,
    sequential_ns_per_byte_header: f64,
) -> Option<f64> {
    let random_gap = random_ns_per_byte_header - random_ns_per_byte_padded;
    let sequential_gap = sequential_ns_per_byte_padded - sequential_ns_per_byte_header;
    if random_gap <= 0.0 || sequential_gap <= 0.0 {
        return None;
    }
    // p·random_gap = (1−p)·sequential_gap
    Some(sequential_gap / (random_gap + sequential_gap))
}

fn open_file(path: &str, direct: bool) -> std::fs::File {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if direct {
        options.custom_flags(O_DIRECT);
    }
    options.open(path).unwrap_or_else(|error| {
        eprintln!("打不开 {path}：{error}");
        std::process::exit(3)
    })
}

/// 对整个单元算一遍 MAC，沿用 E6 / E58 的被测对象。
fn verify_unit(
    cipher: &Aes256Gcm,
    nonce: &Nonce<aes_gcm::aes::cipher::consts::U12>,
    unit: &mut [u8],
) -> u8 {
    let tag = cipher.encrypt_in_place_detached(nonce, b"", unit).expect("MAC 失败");
    tag[0]
}

#[derive(Clone, Copy)]
enum Layout2 {
    /// 净荷补齐到 7 个整页，页从单元偏移 4096 起。
    Padded,
    /// 含头 h，净荷从偏移 h 起、连续打包。
    Header { header_bytes: usize, copy_out: bool },
}

struct ArmResult {
    elapsed_ns: u64,
    verify_ns: u64,
    copy_ns: u64,
    device_bytes: u64,
    user_bytes: u64,
    operations: u64,
    straddled_pages: u64,
    sink: u64,
}

fn cipher_and_nonce() -> (Aes256Gcm, Nonce<aes_gcm::aes::cipher::consts::U12>) {
    let key = [0x42u8; 32];
    let cipher = Aes256Gcm::new_from_slice(&key).expect("密钥长度固定 32");
    (cipher, *Nonce::from_slice(&[0u8; 12]))
}

/// 随机页读：每次读页 k 所在的单元（跨则两个），验满，按臂决定拷不拷。
fn random_arm(
    path: &str,
    direct: bool,
    layout: Layout2,
    page_count: u64,
    operations: u64,
    seed: u64,
    control_copy: Option<usize>,
) -> ArmResult {
    let file = open_file(path, direct);
    let (cipher, nonce) = cipher_and_nonce();
    let mut buffer = Aligned::new(2 * UNIT_BYTES);
    let mut destination = Aligned::new(PAGE_BYTES);
    let mut control_source = control_copy.map(Aligned::new);
    let mut control_destination = control_copy.map(Aligned::new);
    let mut state = seed | 1;
    let mut sink = 0u64;
    let mut verify_ns = 0u64;
    let mut copy_ns = 0u64;
    let mut device_bytes = 0u64;
    let mut straddled_pages = 0u64;
    let started = Instant::now();
    for _ in 0..operations {
        let page_index = next_random(&mut state) % page_count;
        match layout {
            Layout2::Padded => {
                let unit_index = page_index / 7;
                let offset_in_unit = UNIT_BYTES - PADDED_PAYLOAD_BYTES + (page_index % 7) as usize * PAGE_BYTES;
                file.read_exact_at(&mut buffer.as_mut_slice()[..UNIT_BYTES], unit_index * UNIT_BYTES as u64)
                    .expect("读失败");
                device_bytes += UNIT_BYTES as u64;
                let verify_started = Instant::now();
                sink = sink.wrapping_add(verify_unit(&cipher, &nonce, &mut buffer.as_mut_slice()[..UNIT_BYTES]) as u64);
                verify_ns += verify_started.elapsed().as_nanos() as u64;
                sink = sink.wrapping_add(buffer.as_slice()[offset_in_unit] as u64);
            }
            Layout2::Header { header_bytes, copy_out } => {
                let payload = payload_bytes(header_bytes);
                let (unit_index, offset_in_payload) = locate_page(page_index, payload);
                let straddles = page_straddles(page_index, payload);
                let unit_count = if straddles { 2 } else { 1 };
                for extra in 0..unit_count {
                    let slot = extra * UNIT_BYTES;
                    file.read_exact_at(
                        &mut buffer.as_mut_slice()[slot..slot + UNIT_BYTES],
                        (unit_index + extra as u64) * UNIT_BYTES as u64,
                    )
                    .expect("读失败");
                }
                device_bytes += (unit_count * UNIT_BYTES) as u64;
                straddled_pages += straddles as u64;
                let verify_started = Instant::now();
                for extra in 0..unit_count {
                    let slot = extra * UNIT_BYTES;
                    sink = sink.wrapping_add(
                        verify_unit(&cipher, &nonce, &mut buffer.as_mut_slice()[slot..slot + UNIT_BYTES]) as u64,
                    );
                }
                verify_ns += verify_started.elapsed().as_nanos() as u64;
                let first_piece = PAGE_BYTES.min(payload - offset_in_payload);
                let source_start = header_bytes + offset_in_payload;
                if copy_out {
                    let copy_started = Instant::now();
                    let source = buffer.as_slice();
                    let target = destination.as_mut_slice();
                    target[..first_piece].copy_from_slice(&source[source_start..source_start + first_piece]);
                    if first_piece < PAGE_BYTES {
                        let second_start = UNIT_BYTES + header_bytes;
                        target[first_piece..].copy_from_slice(&source[second_start..second_start + PAGE_BYTES - first_piece]);
                    }
                    copy_ns += copy_started.elapsed().as_nanos() as u64;
                    sink = sink.wrapping_add(destination.as_slice()[0] as u64);
                } else {
                    sink = sink.wrapping_add(buffer.as_slice()[source_start] as u64);
                    if first_piece < PAGE_BYTES {
                        sink = sink.wrapping_add(buffer.as_slice()[UNIT_BYTES + header_bytes] as u64);
                    }
                }
            }
        }
        if let (Some(source), Some(target)) = (control_source.as_mut(), control_destination.as_mut()) {
            let copy_started = Instant::now();
            source.as_mut_slice()[0] = sink as u8;
            target.as_mut_slice().copy_from_slice(source.as_slice());
            copy_ns += copy_started.elapsed().as_nanos() as u64;
            sink = sink.wrapping_add(target.as_slice()[0] as u64);
        }
    }
    ArmResult {
        elapsed_ns: started.elapsed().as_nanos() as u64,
        verify_ns,
        copy_ns,
        device_bytes,
        user_bytes: operations * PAGE_BYTES as u64,
        operations,
        straddled_pages,
        sink,
    }
}

/// 同一个负载，QD=16：每个线程自己的 fd、缓冲、种子；挂钟按整个并发段算。
fn random_arm_queued(path: &str, direct: bool, layout: Layout2, page_count: u64, operations: u64, seed: u64) -> ArmResult {
    let per_thread = operations / QUEUE_DEPTH as u64;
    let started = Instant::now();
    let mut handles = Vec::with_capacity(QUEUE_DEPTH);
    for thread_index in 0..QUEUE_DEPTH as u64 {
        let path_owned = path.to_string();
        handles.push(std::thread::spawn(move || {
            random_arm(
                &path_owned,
                direct,
                layout,
                page_count,
                per_thread,
                seed ^ thread_index.wrapping_mul(0x9E37_79B9_7F4A_7C15),
                None,
            )
        }));
    }
    let mut total = ArmResult {
        elapsed_ns: 0,
        verify_ns: 0,
        copy_ns: 0,
        device_bytes: 0,
        user_bytes: 0,
        operations: 0,
        straddled_pages: 0,
        sink: 0,
    };
    for handle in handles {
        let part = handle.join().expect("线程 panic");
        total.verify_ns += part.verify_ns;
        total.copy_ns += part.copy_ns;
        total.device_bytes += part.device_bytes;
        total.user_bytes += part.user_bytes;
        total.operations += part.operations;
        total.straddled_pages += part.straddled_pages;
        total.sink = total.sink.wrapping_add(part.sink);
    }
    total.elapsed_ns = started.elapsed().as_nanos() as u64;
    total
}

/// 顺序读：I/O 固定 1 MiB，逐单元验；含头臂把每单元净荷搬进连续缓冲，补齐臂就地消费。
fn sequential_arm(path: &str, direct: bool, layout: Layout2) -> ArmResult {
    let file = open_file(path, direct);
    let (cipher, nonce) = cipher_and_nonce();
    let mut buffer = Aligned::new(SEQ_IO_BYTES);
    let mut destination = Aligned::new(SEQ_IO_BYTES);
    let units_per_io = SEQ_IO_BYTES / UNIT_BYTES;
    let (payload, header_bytes, copy_out) = match layout {
        Layout2::Padded => (PADDED_PAYLOAD_BYTES, UNIT_BYTES - PADDED_PAYLOAD_BYTES, false),
        Layout2::Header { header_bytes, copy_out } => (payload_bytes(header_bytes), header_bytes, copy_out),
    };
    let units = units_for_user_bytes(SEQ_USER_BYTES, payload);
    let io_count = (units + units_per_io as u64 - 1) / units_per_io as u64;
    let mut sink = 0u64;
    let mut verify_ns = 0u64;
    let mut copy_ns = 0u64;
    let started = Instant::now();
    for io_index in 0..io_count {
        file.read_exact_at(buffer.as_mut_slice(), io_index * SEQ_IO_BYTES as u64)
            .expect("顺序读失败");
        let verify_started = Instant::now();
        for unit in buffer.as_mut_slice().chunks_mut(UNIT_BYTES) {
            sink = sink.wrapping_add(verify_unit(&cipher, &nonce, unit) as u64);
        }
        verify_ns += verify_started.elapsed().as_nanos() as u64;
        if copy_out {
            let copy_started = Instant::now();
            let source = buffer.as_slice();
            let target = destination.as_mut_slice();
            for unit_index in 0..units_per_io {
                let source_start = unit_index * UNIT_BYTES + header_bytes;
                let target_start = unit_index * payload;
                target[target_start..target_start + payload]
                    .copy_from_slice(&source[source_start..source_start + payload]);
            }
            copy_ns += copy_started.elapsed().as_nanos() as u64;
            sink = sink.wrapping_add(destination.as_slice()[0] as u64);
        } else {
            sink = sink.wrapping_add(buffer.as_slice()[header_bytes] as u64);
        }
    }
    ArmResult {
        elapsed_ns: started.elapsed().as_nanos() as u64,
        verify_ns,
        copy_ns,
        device_bytes: io_count * SEQ_IO_BYTES as u64,
        user_bytes: units * payload as u64,
        operations: io_count,
        straddled_pages: 0,
        sink,
    }
}

/// 把测试区填满：全 0 的稀疏文件读起来可能根本不碰设备。
fn fill(path: &str, region: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(O_DIRECT)
        .open(path)
        .unwrap_or_else(|error| {
            eprintln!("建不了测试区 {path}：{error}");
            std::process::exit(3)
        });
    let current = file.seek(SeekFrom::End(0)).expect("取不到大小");
    if current >= region {
        return;
    }
    eprintln!("填充测试区 {} MiB …", region / (1024 * 1024));
    let mut buffer = Aligned::new(SEQ_IO_BYTES);
    for (index, byte) in buffer.as_mut_slice().iter_mut().enumerate() {
        *byte = (index as u8).wrapping_mul(31).wrapping_add(7);
    }
    file.seek(SeekFrom::Start(0)).expect("seek 失败");
    for _ in 0..(region / SEQ_IO_BYTES as u64) {
        file.write_all(buffer.as_slice()).expect("填充失败");
    }
    file.sync_all().expect("sync 失败");
}

fn emit_arm(emitter: &mut Emitter, name: &str, result: &ArmResult, read_before: Option<u64>, read_after: Option<u64>) {
    let proc_delta = match (read_before, read_after) {
        (Some(before), Some(after)) => format!("{}", after.saturating_sub(before)),
        _ => "NA".into(),
    };
    let proc_ratio = match (read_before, read_after) {
        (Some(before), Some(after)) if result.device_bytes > 0 => {
            format!("{:.4}", after.saturating_sub(before) as f64 / result.device_bytes as f64)
        }
        _ => "NA".into(),
    };
    let seconds = result.elapsed_ns as f64 / 1e9;
    let user_mib_per_second = if seconds > 0.0 {
        format!("{:.3}", result.user_bytes as f64 / (1024.0 * 1024.0) / seconds)
    } else {
        "NA".into()
    };
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name={name} ops={} dev_bytes={} user_bytes={} elapsed_ns={} verify_ns={} copy_ns={} \
             straddled={} units_per_op={:.5} ns_per_op={:.1} user_mib_per_s={user_mib_per_second} \
             proc_read_bytes={proc_delta} pr_over_devbytes={proc_ratio} sink={}",
            result.operations,
            result.device_bytes,
            result.user_bytes,
            result.elapsed_ns,
            result.verify_ns,
            result.copy_ns,
            result.straddled_pages,
            result.device_bytes as f64 / UNIT_BYTES as f64 / result.operations.max(1) as f64,
            result.elapsed_ns as f64 / result.operations.max(1) as f64,
            result.sink
        ))
    );
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e140-header-alignment <块设备或文件> [种子] [none|nodirect] [ops] [区域 MiB]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0x0140_1234);
    let mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    let direct = mode != "nodirect";
    let operations: u64 = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(8192);
    let region_mib: u64 = std::env::args().nth(5).and_then(|x| x.parse().ok()).unwrap_or(8192);
    let region = region_mib * 1024 * 1024;

    let is_device = std::fs::metadata(&path).map(|m| !m.is_file()).unwrap_or(false);
    if !is_device {
        fill(&path, region);
    }
    let size = open_file(&path, direct).seek(SeekFrom::End(0)).expect("取不到大小");
    if size == 0 {
        eprintln!("大小为 0 —— 判定不明，整轮作废");
        std::process::exit(5);
    }
    let region = region.min(size);
    let unit_total = region / UNIT_BYTES as u64;

    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config dev={path} size={size} region={region} units={unit_total} ops={operations} qd={QUEUE_DEPTH} \
             seq_io={SEQ_IO_BYTES} seq_user_bytes={SEQ_USER_BYTES} unit={UNIT_BYTES} page={PAGE_BYTES} \
             headers={HEADER_REGISTERED_BYTES}/{HEADER_WITH_RESERVE_BYTES} padded_payload={PADDED_PAYLOAD_BYTES} \
             control_copy={CONTROL_COPY_BYTES} seed={seed} mode={mode} o_direct={direct}"
        ))
    );

    // 页数按各自净荷算，最后一页不用（它可能伸出区域末尾）。
    let pages_padded = unit_total * 7 - 1;
    let arms: Vec<(&str, Layout2, u64)> = vec![
        ("padded", Layout2::Padded, pages_padded),
        (
            "h133",
            Layout2::Header { header_bytes: HEADER_WITH_RESERVE_BYTES, copy_out: true },
            unit_total * payload_bytes(HEADER_WITH_RESERVE_BYTES) as u64 / PAGE_BYTES as u64 - 1,
        ),
        (
            "h133_nocopy",
            Layout2::Header { header_bytes: HEADER_WITH_RESERVE_BYTES, copy_out: false },
            unit_total * payload_bytes(HEADER_WITH_RESERVE_BYTES) as u64 / PAGE_BYTES as u64 - 1,
        ),
        (
            "h105",
            Layout2::Header { header_bytes: HEADER_REGISTERED_BYTES, copy_out: true },
            unit_total * payload_bytes(HEADER_REGISTERED_BYTES) as u64 / PAGE_BYTES as u64 - 1,
        ),
        (
            "h105_nocopy",
            Layout2::Header { header_bytes: HEADER_REGISTERED_BYTES, copy_out: false },
            unit_total * payload_bytes(HEADER_REGISTERED_BYTES) as u64 / PAGE_BYTES as u64 - 1,
        ),
    ];

    let mut random_ns_per_byte = std::collections::HashMap::new();
    let mut sequential_ns_per_byte = std::collections::HashMap::new();
    for (arm_name, layout, page_count) in &arms {
        for load in ["rand", "randq", "seq"] {
            let before = proc_read_bytes();
            let result = match load {
                "rand" => random_arm(&path, direct, *layout, *page_count, operations, seed, None),
                "randq" => random_arm_queued(&path, direct, *layout, *page_count, operations, seed),
                _ => sequential_arm(&path, direct, *layout),
            };
            let after = proc_read_bytes();
            let name = format!("{load}_{arm_name}");
            emit_arm(&mut emitter, &name, &result, before, after);
            if result.operations > 0 && result.elapsed_ns > 0 {
                match load {
                    "rand" => {
                        random_ns_per_byte.insert(
                            arm_name.to_string(),
                            result.elapsed_ns as f64 / result.operations as f64 / PAGE_BYTES as f64,
                        );
                    }
                    "seq" => {
                        sequential_ns_per_byte
                            .insert(arm_name.to_string(), result.elapsed_ns as f64 / result.user_bytes as f64);
                    }
                    _ => {}
                }
            }
        }
    }

    // 拷贝判别力对照：padded 读加每次 memcpy 1 MiB。
    let before = proc_read_bytes();
    let control = random_arm(&path, direct, Layout2::Padded, pages_padded, operations, seed, Some(CONTROL_COPY_BYTES));
    let after = proc_read_bytes();
    emit_arm(&mut emitter, "rand_ctl_copy1m", &control, before, after);

    // 跑前写死的解析值，与实测落进同一份产物。
    for (label, payload) in [
        ("h133", payload_bytes(HEADER_WITH_RESERVE_BYTES)),
        ("h105", payload_bytes(HEADER_REGISTERED_BYTES)),
        ("padded", PADDED_PAYLOAD_BYTES),
    ] {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=model_{label} payload={payload} straddle_fraction={:.6} straddle_count_one_period={} expected_units_per_page={:.6} \
                 dev_bytes_per_user_byte={:.6} units_per_gib={}",
                straddle_fraction(payload),
                straddle_count(payload, payload as u64),
                expected_units_per_page(payload),
                device_bytes_per_user_byte(payload),
                units_for_user_bytes(1 << 30, payload)
            ))
        );
    }

    // 判据 1 / 2 与交叉点，由本轮实测算出（各轮各自算，跨轮同向才算数）。
    if let (Some(rp), Some(rh), Some(sp), Some(sh)) = (
        random_ns_per_byte.get("padded"),
        random_ns_per_byte.get("h133"),
        sequential_ns_per_byte.get("padded"),
        sequential_ns_per_byte.get("h133"),
    ) {
        let crossover = crossover_random_share(*rp, *rh, *sp, *sh)
            .map(|p| format!("{p:.6}"))
            .unwrap_or_else(|| "NA".into());
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=verdict r_rand_qd1={:.4} r_seq={:.4} random_ns_per_byte_padded={rp:.4} random_ns_per_byte_h133={rh:.4} \
                 seq_ns_per_byte_padded={sp:.4} seq_ns_per_byte_h133={sh:.4} crossover_random_share={crossover}",
                rh / rp,
                sh / sp
            ))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 绝对值断言 1：格式常量与替代形态的几何。它们一改，下面每条解析值都作废。
    #[test]
    fn constants_are_pinned() {
        assert_eq!(UNIT_BYTES, 32768, "D4 已定项 1 / 5");
        assert_eq!(PAGE_BYTES, 4096);
        assert_eq!(HEADER_REGISTERED_BYTES, 105, "D18 已定项 7 登记值");
        assert_eq!(HEADER_WITH_RESERVE_BYTES, 133, "D18 已定项 14：105 + 28 预留位");
        assert_eq!(PADDED_PAYLOAD_BYTES, 7 * 4096, "补齐臂：七个整页");
        assert_eq!(payload_bytes(HEADER_WITH_RESERVE_BYTES), 32635);
        assert_eq!(payload_bytes(HEADER_REGISTERED_BYTES), 32663);
        assert_eq!(SEQ_IO_BYTES, 1 << 20);
        assert_eq!(QUEUE_DEPTH, 16);
    }

    /// 绝对值断言 2：跨单元页占比的闭式，用一整个周期的穷举对它——两条路不共用代码。
    #[test]
    fn straddle_fraction_matches_exhaustive_count_over_one_period() {
        for payload in [32635usize, 32663] {
            let count = straddle_count(payload, payload as u64);
            assert_eq!(count, 4095, "净荷 {payload} 一个周期里应恰有 4095 页跨单元");
            let closed = straddle_fraction(payload) * payload as f64;
            assert!((closed - 4095.0).abs() < 1e-9, "闭式给 {closed}");
        }
        assert_eq!(straddle_count(28672, 28672), 0, "七个整页的净荷从不跨");
        assert_eq!(straddle_fraction(28672), 0.0);
    }

    /// 绝对值断言 3：h133 的跨单元占比 12.548%，期望单元数 1.12548。
    #[test]
    fn h133_straddle_share_is_one_eighth_ish() {
        let share = straddle_fraction(32635);
        assert!((share - 4095.0 / 32635.0).abs() < 1e-12);
        assert!((share - 0.125479).abs() < 1e-6, "实得 {share}");
        assert!((expected_units_per_page(32635) - 1.125479).abs() < 1e-6);
    }

    /// 绝对值断言 4：设备字节 / 用户字节——补齐臂恰 8/7，含头臂 1.004075。
    #[test]
    fn device_bytes_per_user_byte_is_pinned() {
        assert_eq!(device_bytes_per_user_byte(28672), 8.0 / 7.0);
        assert!((device_bytes_per_user_byte(32635) - 1.0040754).abs() < 1e-6);
        assert!(device_bytes_per_user_byte(28672) > device_bytes_per_user_byte(32635) * 1.13);
    }

    /// 页 7（文件偏移 28672）在 h133 下跨单元：前 3963 字节在单元 0，后 133 字节在单元 1。
    #[test]
    fn page_seven_straddles_units_zero_and_one_under_h133() {
        let (unit, offset) = locate_page(7, 32635);
        assert_eq!((unit, offset), (0, 28672));
        assert!(page_straddles(7, 32635));
        assert_eq!(32635 - offset, 3963, "第一段");
        assert_eq!(4096 - 3963, 133, "第二段");
        assert!(!page_straddles(6, 32635));
        assert_eq!(locate_page(8, 32635), (1, 133));
    }

    /// 补齐臂任何页都不跨：页到单元是整除。
    #[test]
    fn padded_pages_never_straddle() {
        for page in 0..20_000u64 {
            assert!(!page_straddles(page, 28672), "页 {page}");
        }
    }

    /// 一 GiB 文件各要多少单元：含头 32902、补齐 37450——差 4548 个单元，就是 14.3% 那笔账的绝对形态。
    #[test]
    fn units_for_one_gib_are_pinned() {
        assert_eq!(units_for_user_bytes(1 << 30, 32635), 32902);
        assert_eq!(units_for_user_bytes(1 << 30, 28672), 37450);
        assert_eq!(units_for_user_bytes(1 << 30, 28672) - units_for_user_bytes(1 << 30, 32635), 4548);
    }

    /// 交叉点：随机侧含头贵、顺序侧补齐贵时才有交叉；同向没有。
    #[test]
    fn crossover_exists_only_when_arms_split() {
        // 随机每字节 10 对 12（含头贵 2），顺序每字节 1.0 对 0.9（补齐贵 0.1）⇒ p* = 0.1/(2+0.1)
        let p = crossover_random_share(10.0, 12.0, 1.0, 0.9).expect("应有交叉点");
        assert!((p - 0.1 / 2.1).abs() < 1e-12, "{p}");
        assert!(crossover_random_share(10.0, 9.0, 1.0, 0.9).is_none(), "两格补齐都贵，没有交叉");
        assert!(crossover_random_share(10.0, 12.0, 0.9, 1.0).is_none(), "两格含头都贵，没有交叉");
    }

    /// 伪随机可复现且不退化。
    #[test]
    fn prng_is_deterministic() {
        let (mut a, mut b) = (777u64, 777u64);
        let xs: Vec<u64> = (0..8).map(|_| next_random(&mut a)).collect();
        let ys: Vec<u64> = (0..8).map(|_| next_random(&mut b)).collect();
        assert_eq!(xs, ys);
        assert!(xs.windows(2).all(|w| w[0] != w[1]));
    }
}
