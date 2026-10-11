//! E58：校验和粒度的字节口径与时间口径代价 —— D4 已定项 1 那根从没量过的轴。
//!
//! ## 立项理由：已有的三个数量的都不是这根轴
//!
//! D4（校验和位置）已定项 1 问的是「校验和粒度与随机小读的张力」，而支撑它的三个实验
//! 量的都是旁轴：E6 是加密**吞吐**（CPU，单元 4K/16K/64K/256K），
//! E20 是**CPU 缓存**（节点大小 2K–64K），E7 是**固定内存预算下的设备 I/O**。
//! **端到端「读 4 KiB 用户数据要付多少」一次没测过**——D4 正文自己写着这句。
//! D25（目标负载优先级）也写着「字节口径至今未测」。本实验补的就是这一格。
//!
//! ## 被测量：读 4 KiB 用户数据的端到端代价，作为校验和粒度 G 的函数
//!
//! 内联校验和（D4 已定方向）意味着校验的最小单位是一个单元 G ⇒ 想读其中 4 KiB，
//! 必须**读满 G 并校验满 G**。G ∈ {4, 8, 16, 32, 64, 128} KiB。
//!
//! | 臂 | 负载 | G 影响什么 |
//! |---|---|---|
//! | `rand` | 随机 4 KiB 点读，QD=1（单线程） | 每次读满 G：设备时间 + 校验时间 |
//! | `randq` | 同上，QD=16（16 线程） | 同上，但瓶颈从延迟移到带宽 |
//! | `seq` | 顺序读 512 MiB，**I/O 固定 1 MiB**、只把校验按 G 分块 | 只剩每次校验的固定开销 |
//! | `meta` | 纯算术：每单元 53 字节父节点侧指针载荷（D21 口径 2 副本档拆开：指针头部 31 + 位置条目 22；单元头住单元内、读 G 已含，不计入——2026-09-02 修正 C89（单元头收口的遗留重算）点名的重复计账，原式误计 108） | 摊薄比 53/G |
//!
//! ⚠️ **`seq` 的 I/O 大小故意与 G 解耦**：真实顺序读的 I/O 大小由预读决定，不由校验粒度决定。
//! 让 I/O 跟着 G 变会把「大 I/O 更快」记到 G 头上——那是两个决策，不是一个。
//!
//! ## 跑前写死的解析预测（字节口径）
//!
//! 每 4 KiB 用户数据的字节代价，p = 随机小读在操作里的占比：
//!
//!   cost(p, G) = p·(G + 53) + (1−p)·(4096 + 53·4096/G)
//!
//! 随机侧线性涨（G/4096 倍读放大），顺序侧只买到 53/G 的元数据摊薄。
//! 两者相等的临界占比：p*/(1−p*) = 53·4096·(1/G₂ − 1/G₁)/(G₁ − G₂)。
//! **16 KiB 与 32 KiB 的临界值是 p* = 0.0404%**（单测钉死）——
//! 即字节口径下，只要随机小读多于千分之一，16 KiB 就赢 32 KiB。
//!
//! **时间口径没有预测**，因为它取决于每次 I/O 的固定开销占比，而那正是要量的东西：
//! NVMe 上一次 4 KiB 读与一次 32 KiB 读的**时间**差多少，字节比说了不算。
//!
//! ## 失败条款（跑前写死，跑完不许改）
//!
//! 1. **阳性对照，逐臂跑**：每个 (臂, G) 由**内核**记的 `/proc/self/io` `read_bytes` 增量
//!    必须等于 `ops × G`（容差 ±2%，留给文件系统自己的元数据读）。
//!    ⚠️ **程序自算的 `dev_bytes` 不构成观测**——它就是 `ops × G` 算出来的，
//!    拿它去对 `ops × G` 是自己和自己比。判别力全在 `read_bytes` 这一侧。
//!    任一格对不上 ⇒ 整轮作废（读的不是一个单元，后面所有比值无意义）。
//! 2. **判别力对照**：`rand` 臂上 G=4 KiB 与 G=128 KiB 的**有效用户带宽**必须差 ≥2×。
//!    差不到 ⇒ 这套度量分不出 32 倍读放大，整轮作废。
//! 3. **阴性对照**（`nodirect` 模式）：摘掉 O_DIRECT 之后
//!    `/proc/self/io` 的 `read_bytes` 增量必须塌到不足直连档的 10%，而程序自己数的
//!    `dev_bytes` 纹丝不动。塌不下去 ⇒ `read_bytes` 这条校验路径是回声，此前所有
//!    「两条路径一致」一并作废（evidence-discipline：校验路径本身也要证明它会红）。
//! 4. **读不到 ≠ 读到 0**：任何计时为 0、计数为 0、或 `read_bytes` 取不到 ⇒ 整轮作废，
//!    不许当成 0 参与判定。
//! 5. N=5 轮，判「通过」要 5 轮全通过。
//!
//! ## 反过来的结果我接不接受（跑前写下）
//!
//! - 32 KiB 在 D25 已定负载权重（seq 主 / rand 次）下总代价比 16 KiB **低 ≥5%**
//!   ⇒ 结论写「32 优于 16」，D8 已定项 2（节点大小 16 KiB）应重开，并走三方论证。
//! - 16 KiB **低 ≥5%** ⇒ 结论写「16 保持」，提问者的倾向不成立，如实写。
//! - 两者差 **<5%** ⇒ 结论写「这根轴分不出 16 与 32」，不许挑一个说更好，
//!   取值交给另一根轴（可恢复性 E1 / CPU 缓存 E20）。
//!
//! ## 它答不了什么
//!
//! 不碰崩溃一致性；不碰写路径；单盘无 RAID；校验用 AES-256-GCM（本机有 AES-NI），
//! 换算法要重跑；`rand` 的随机是全域均匀，没有热点分布；不建模预读命中。

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
/// 用户真正要的那一块。D4 已定项 1 的原话就是「随机小读」。
const USER_READ: u64 = 4096;
/// 被扫的校验和粒度。16 与 32 是本轮的争点，两边各留两档看曲线形状。
const GRAINS: [usize; 6] = [4096, 8192, 16384, 32768, 65536, 131072];
/// 每单元住在**父节点**里的指针载荷：指针头部 31 + 位置条目 ×2 共 22 = 53 字节
/// （D21（权威态与派生态的分界）2 副本档 108.0 的口径拆开）。单元自描述头
/// （91 字节初值，D18（块里携带什么信息）已定项 7）住在单元内、读 G 字节时已经读进来，
/// **不计入读放大**——原式误把 55 字节头也加在 G 外，2026-09-02 按
/// C89（单元头收口的遗留重算）修正。
const PTR_BYTES: u64 = 53;
/// `randq` 臂的并发度。
const QD: usize = 16;
/// `seq` 臂的 I/O 大小，**与 G 无关**：预读决定 I/O 大小，校验粒度不决定。
const SEQ_IO: usize = 1024 * 1024;

/// O_DIRECT 要求缓冲、偏移、长度三样都对齐。
struct Aligned {
    ptr: *mut u8,
    len: usize,
    layout: Layout,
}
// 每个线程独占一个缓冲，不共享 —— 跨线程移动是安全的。
unsafe impl Send for Aligned {}

impl Aligned {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, ALIGN).expect("对齐参数非法");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(ptr, 0xA5, len) };
        Self { ptr, len, layout }
    }
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for Aligned {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) }
    }
}

/// 确定性伪随机（xorshift64*）：同一个种子必须给出同一串偏移。
fn next_rand(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// 内核记的「这个进程让存储层取了多少字节」。**这是本实验的校验路径**——
/// 与程序自己数的 `dev_bytes` 互不共享代码，也不共享采样。
fn proc_read_bytes() -> Option<u64> {
    let t = std::fs::read_to_string("/proc/self/io").ok()?;
    for line in t.lines() {
        if let Some(v) = line.strip_prefix("read_bytes:") {
            return v.trim().parse().ok();
        }
    }
    None
}

/// 读放大：读 4 KiB 用户数据实际要读的倍数。
fn amp(g: usize) -> f64 {
    g as f64 / USER_READ as f64
}

/// 元数据摊薄比：每单元 53 字节父节点侧载荷除以单元大小。
fn meta_ratio(g: usize) -> f64 {
    PTR_BYTES as f64 / g as f64
}

/// 跑前写死的字节口径代价模型（每 4 KiB 用户数据）。
fn model_bytes(p: f64, g: usize) -> f64 {
    p * (g as f64 + PTR_BYTES as f64)
        + (1.0 - p) * (USER_READ as f64 + PTR_BYTES as f64 * USER_READ as f64 / g as f64)
}

/// 两个粒度在字节口径下代价相等的随机小读占比。
fn crossover_p(g1: usize, g2: usize) -> f64 {
    let num = PTR_BYTES as f64 * USER_READ as f64 * (1.0 / g2 as f64 - 1.0 / g1 as f64);
    let ratio = num / (g1 as f64 - g2 as f64);
    ratio / (1.0 + ratio)
}

fn open(path: &str, direct: bool) -> std::fs::File {
    let mut oo = OpenOptions::new();
    oo.read(true).write(true);
    if direct {
        oo.custom_flags(O_DIRECT);
    }
    oo.open(path).unwrap_or_else(|e| {
        eprintln!("打不开 {path}：{e}");
        std::process::exit(3)
    })
}

/// 一次校验：对整个单元算一遍 MAC。沿用 E6 的被测对象（AES-256-GCM，
/// `encrypt_in_place_detached`）——换 API 就换了被测对象。
fn verify(c: &Aes256Gcm, nonce: &Nonce<aes_gcm::aes::cipher::consts::U12>, buf: &mut [u8]) -> u8 {
    let t = c.encrypt_in_place_detached(nonce, b"", buf).expect("MAC 失败");
    t[0]
}

struct Arm {
    elapsed_ns: u64,
    verify_ns: u64,
    dev_bytes: u64,
    ops: u64,
    sink: u64,
}

/// 随机 4 KiB 点读：每次读满一个 G 单元并校验满 G。
fn rand_arm(path: &str, direct: bool, g: usize, units: u64, ops: u64, seed: u64) -> Arm {
    let f = open(path, direct);
    let key = [0x42u8; 32];
    let c = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut buf = Aligned::new(g);
    let mut st = seed | 1;
    let mut sink = 0u64;
    let mut verify_ns = 0u64;
    let t0 = Instant::now();
    for _ in 0..ops {
        let unit = next_rand(&mut st) % units;
        f.read_exact_at(&mut buf.as_mut_slice()[..g], unit * g as u64)
            .expect("读失败");
        let tv = Instant::now();
        sink = sink.wrapping_add(verify(&c, nonce, &mut buf.as_mut_slice()[..g]) as u64);
        verify_ns += tv.elapsed().as_nanos() as u64;
    }
    Arm {
        elapsed_ns: t0.elapsed().as_nanos() as u64,
        verify_ns,
        // ⚠️ **记账，不是观测**：它按定义等于 ops × G。观测那一侧是 `proc_read_bytes`。
        dev_bytes: ops * g as u64,
        ops,
        sink,
    }
}

/// 同一个负载，QD=16。每个线程自己的 fd、自己的缓冲、自己的种子。
fn randq_arm(path: &str, direct: bool, g: usize, units: u64, ops: u64, seed: u64) -> Arm {
    let per = ops / QD as u64;
    let t0 = Instant::now();
    let mut hs = Vec::with_capacity(QD);
    for tid in 0..QD as u64 {
        let p = path.to_string();
        hs.push(std::thread::spawn(move || {
            rand_arm(&p, direct, g, units, per, seed ^ (tid.wrapping_mul(0x9E37_79B9_7F4A_7C15)))
        }));
    }
    let mut acc = Arm { elapsed_ns: 0, verify_ns: 0, dev_bytes: 0, ops: 0, sink: 0 };
    for h in hs {
        let a = h.join().expect("线程 panic");
        acc.verify_ns += a.verify_ns;
        acc.dev_bytes += a.dev_bytes;
        acc.ops += a.ops;
        acc.sink = acc.sink.wrapping_add(a.sink);
    }
    // 挂钟按整个并发段算，不是各线程之和 —— 要的是聚合带宽。
    acc.elapsed_ns = t0.elapsed().as_nanos() as u64;
    acc
}

/// 顺序读：I/O 固定 1 MiB，校验按 G 分块。
fn seq_arm(path: &str, direct: bool, g: usize, total_bytes: u64) -> Arm {
    let f = open(path, direct);
    let key = [0x42u8; 32];
    let c = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut buf = Aligned::new(SEQ_IO);
    let ios = total_bytes / SEQ_IO as u64;
    let mut sink = 0u64;
    let mut verify_ns = 0u64;
    let t0 = Instant::now();
    for i in 0..ios {
        f.read_exact_at(buf.as_mut_slice(), i * SEQ_IO as u64)
            .expect("顺序读失败");
        let tv = Instant::now();
        for ch in buf.as_mut_slice().chunks_mut(g) {
            sink = sink.wrapping_add(verify(&c, nonce, ch) as u64);
        }
        verify_ns += tv.elapsed().as_nanos() as u64;
    }
    Arm {
        elapsed_ns: t0.elapsed().as_nanos() as u64,
        verify_ns,
        dev_bytes: ios * SEQ_IO as u64,
        ops: ios,
        sink,
    }
}

/// 把测试区填满。全 0 的稀疏文件读起来可能根本不碰设备。
fn fill(path: &str, region: u64) {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(O_DIRECT)
        .open(path)
        .unwrap_or_else(|e| {
            eprintln!("建不了测试区 {path}：{e}");
            std::process::exit(3)
        });
    let cur = f.seek(SeekFrom::End(0)).expect("取不到大小");
    if cur >= region {
        return;
    }
    eprintln!("填充测试区 {} MiB …", region / (1024 * 1024));
    let mut buf = Aligned::new(SEQ_IO);
    for (i, b) in buf.as_mut_slice().iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(31).wrapping_add(7);
    }
    f.seek(SeekFrom::Start(0)).expect("seek 失败");
    for _ in 0..(region / SEQ_IO as u64) {
        f.write_all(buf.as_slice()).expect("填充失败");
    }
    f.sync_all().expect("sync 失败");
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e58-csum-grain <块设备或文件> [种子] [none|nodirect] [ops] [区域 MiB]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0x5858_1234);
    let mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    let direct = mode != "nodirect";
    let ops: u64 = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(4096);
    let region_mb: u64 = std::env::args().nth(5).and_then(|x| x.parse().ok()).unwrap_or(8192);
    let region = region_mb * 1024 * 1024;

    // 块设备不填，普通文件才填。
    let is_dev = std::fs::metadata(&path).map(|m| !m.is_file()).unwrap_or(false);
    if !is_dev {
        fill(&path, region);
    }
    let size = open(&path, direct).seek(SeekFrom::End(0)).expect("取不到大小");
    if size == 0 {
        eprintln!("大小为 0 —— 判定不明，整轮作废");
        std::process::exit(5);
    }
    let region = region.min(size);

    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config dev={path} size={size} region={region} ops={ops} qd={QD} \
             seq_io={SEQ_IO} user_read={USER_READ} ptr_bytes={PTR_BYTES} seed={seed} \
             mode={mode} o_direct={direct} grains={:?}",
            GRAINS
        ))
    );

    for &g in GRAINS.iter() {
        let units = region / g as u64;
        for arm in ["rand", "randq", "seq"] {
            let r0 = proc_read_bytes();
            let a = match arm {
                "rand" => rand_arm(&path, direct, g, units, ops, seed),
                "randq" => randq_arm(&path, direct, g, units, ops, seed),
                _ => seq_arm(&path, direct, g, 512 * 1024 * 1024),
            };
            let r1 = proc_read_bytes();
            // 读不到 ≠ 读到 0：取不到就报 NA，不许填 0。
            let pr = match (r0, r1) {
                (Some(a), Some(b)) => format!("{}", b.saturating_sub(a)),
                _ => "NA".into(),
            };
            // 阳性对照就落在这一格：内核记的字节 ÷ 程序记账的字节，直连档必须 ≈1。
            let prr = match (r0, r1) {
                (Some(x), Some(y)) if a.dev_bytes > 0 => {
                    format!("{:.4}", y.saturating_sub(x) as f64 / a.dev_bytes as f64)
                }
                _ => "NA".into(),
            };
            let user_bytes = if arm == "seq" { a.dev_bytes } else { a.ops * USER_READ };
            let secs = a.elapsed_ns as f64 / 1e9;
            let user_mib = if secs > 0.0 {
                format!("{:.3}", user_bytes as f64 / (1024.0 * 1024.0) / secs)
            } else {
                "NA".into()
            };
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name={arm}_g{g} grain={g} ops={} dev_bytes={} user_bytes={user_bytes} \
                     elapsed_ns={} verify_ns={} ns_per_op={:.1} user_mib_per_s={user_mib} \
                     amp={:.4} proc_read_bytes={pr} pr_over_devbytes={prr} sink={}",
                    a.ops,
                    a.dev_bytes,
                    a.elapsed_ns,
                    a.verify_ns,
                    a.elapsed_ns as f64 / a.ops.max(1) as f64,
                    a.dev_bytes as f64 / user_bytes.max(1) as f64,
                    a.sink
                ))
            );
        }
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=meta_g{g} grain={g} ptr_bytes={PTR_BYTES} meta_ratio={:.6} \
                 read_amp_4k={:.4}",
                meta_ratio(g),
                amp(g)
            ))
        );
    }

    // 跑前写死的解析预测，与实测落进同一份产物：事后想改判据会跟这几行对不上。
    for p in [0.0f64, 0.00082330, 0.01, 0.1, 1.0] {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=model_cost p={p} g16384={:.4} g32768={:.4} ratio_32_over_16={:.6}",
                model_bytes(p, 16384),
                model_bytes(p, 32768),
                model_bytes(p, 32768) / model_bytes(p, 16384)
            ))
        );
    }

    // 字节口径的解析临界点，与实测放在同一份产物里，免得事后各引各的。
    for (a, b) in [(4096usize, 16384usize), (16384, 32768), (16384, 65536), (32768, 65536)] {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=model_p_star g1={a} g2={b} p_star={:.8}",
                crossover_p(a, b)
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言 1：粒度网格与两个格式常量。**
    /// 它们一改，下面每一条解析值都静默作废。
    #[test]
    fn constants_are_pinned() {
        assert_eq!(GRAINS, [4096, 8192, 16384, 32768, 65536, 131072]);
        assert_eq!(USER_READ, 4096, "D4 已定项 1 问的就是随机小读");
        assert_eq!(PTR_BYTES, 31 + 22, "父节点侧：指针头部 31 + 位置条目 ×2 共 22；单元头不计入");
        assert_eq!(SEQ_IO, 1048576, "顺序臂的 I/O 与 G 解耦，固定 1 MiB");
        assert_eq!(QD, 16);
    }

    /// **绝对值断言 2：读放大恰好是 G/4096，逐档钉死。**
    /// 这是「所有臂一起错」最容易溜过去的地方——臂间互比时它是常数因子。
    #[test]
    fn read_amplification_is_exactly_grain_over_four_k() {
        assert_eq!(amp(4096), 1.0);
        assert_eq!(amp(8192), 2.0);
        assert_eq!(amp(16384), 4.0);
        assert_eq!(amp(32768), 8.0);
        assert_eq!(amp(65536), 16.0);
        assert_eq!(amp(131072), 32.0);
    }

    /// **绝对值断言 3：元数据摊薄比 53/G。**
    /// 16 KiB 是 0.659%，32 KiB 是 0.330% —— 两者的差 0.33 个百分点，
    /// 正是「32 比 16 多买到的全部东西」在字节口径下的量。
    #[test]
    fn metadata_ratio_is_fifty_three_over_grain() {
        assert!((meta_ratio(16384) - 0.003235).abs() < 1e-6, "{}", meta_ratio(16384));
        assert!((meta_ratio(32768) - 0.001617).abs() < 1e-6, "{}", meta_ratio(32768));
        let gain = meta_ratio(16384) - meta_ratio(32768);
        assert!((gain - 0.001617).abs() < 1e-6, "32 比 16 只省 0.16 个百分点，实得 {gain}");
    }

    /// **绝对值断言 4：字节口径的临界占比 p* = 0.0823%。**
    /// 独立算术：p*/(1−p*) = 53·4096·(1/32768 − 1/16384)/(16384 − 32768)
    ///                     = 442368/(32768·16384) = 8.2418e-4。
    #[test]
    fn crossover_between_sixteen_and_thirty_two_is_four_ten_thousandths() {
        let p = crossover_p(16384, 32768);
        let hand = 217088.0 / (32768.0 * 16384.0); // 53·4096，独立手算，不走 crossover_p
        let expect = hand / (1.0 + hand);
        assert!((p - expect).abs() < 1e-12, "{p} vs {expect}");
        assert!((p - 0.00040419).abs() < 1e-7, "临界占比实得 {p}");
    }

    /// **临界点两侧各取一点，方向必须相反。** 只钉一个点看不出模型是不是常数。
    #[test]
    fn model_picks_sixteen_above_the_crossover_and_thirty_two_below() {
        let p = crossover_p(16384, 32768);
        assert!(model_bytes(p * 10.0, 16384) < model_bytes(p * 10.0, 32768), "占比高于临界时 16 应更省");
        assert!(model_bytes(p / 10.0, 32768) < model_bytes(p / 10.0, 16384), "占比低于临界时 32 应更省");
        // 临界点上两者必须相等（这才叫临界点）
        let d = (model_bytes(p, 16384) - model_bytes(p, 32768)).abs();
        assert!(d < 1e-6, "临界点上差 {d}");
    }

    /// **绝对值断言 5：纯随机负载下的字节代价，逐档钉死。**
    /// p=1 时每 4 KiB 用户数据要读 G+53 字节：16 KiB 档 16437，32 KiB 档 32821。
    #[test]
    fn pure_random_cost_is_grain_plus_pointer() {
        assert_eq!(model_bytes(1.0, 16384), 16437.0);
        assert_eq!(model_bytes(1.0, 32768), 32821.0);
        assert_eq!(model_bytes(1.0, 4096), 4149.0);
    }

    /// **绝对值断言 6：纯顺序负载下的字节代价。**
    /// p=0 时是 4096 + 53·4096/G：16 KiB 档 4109.25，32 KiB 档 4102.625。
    #[test]
    fn pure_sequential_cost_is_four_k_plus_amortized_pointer() {
        assert!((model_bytes(0.0, 16384) - 4109.25).abs() < 1e-9, "{}", model_bytes(0.0, 16384));
        assert!((model_bytes(0.0, 32768) - 4102.625).abs() < 1e-9, "{}", model_bytes(0.0, 32768));
    }

    /// 伪随机必须可复现，否则「同一种子同一串偏移」这句话是空的。
    #[test]
    fn prng_is_deterministic() {
        let (mut a, mut b) = (12345u64, 12345u64);
        let xs: Vec<u64> = (0..8).map(|_| next_rand(&mut a)).collect();
        let ys: Vec<u64> = (0..8).map(|_| next_rand(&mut b)).collect();
        assert_eq!(xs, ys);
        assert_eq!(xs.len(), 8);
        // 别退化成常数序列
        assert!(xs.windows(2).all(|w| w[0] != w[1]));
    }
}
