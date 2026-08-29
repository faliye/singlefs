//! E12：生命周期判定的两条路，量设备 I/O。
//!
//! 判据与失败条款见 kb/experiments.md E12。**必须真的碰设备**——
//! 纯内存微基准会把这题测成「没有差别」，而差别恰恰全在 I/O 上。

use e7_index_bench::{page_of, Emitter, IoCounters, Lru};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

/// 块层自己数的 I/O。**这是校验用的独立路径**——内核数的，与本程序的计数器
/// 不共享任何代码、任何采样、任何工具（`singlefs-ai-sop/rules/evidence-discipline.md`）。
/// 两者对不上，说明本程序的模型是错的：O_DIRECT 没真的绕过缓存、
/// 内核合并了请求、或者有预读。
#[derive(Debug, Default, Clone, Copy)]
struct BlkStat {
    read_ios: u64,
    read_sectors: u64,
    write_ios: u64,
    write_sectors: u64,
}

fn blkstat(dev_path: &str) -> Option<BlkStat> {
    let name = dev_path.rsplit('/').next()?;
    let txt = std::fs::read_to_string(format!("/sys/block/{name}/stat")).ok()?;
    let f: Vec<u64> = txt.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    // 字段序：读完成数 读合并 读扇区 读耗时 写完成数 写合并 写扇区 写耗时 ...
    if f.len() < 8 {
        return None;
    }
    Some(BlkStat { read_ios: f[0], read_sectors: f[2], write_ios: f[4], write_sectors: f[6] })
}

fn blk_delta(a: Option<BlkStat>, b: Option<BlkStat>) -> Option<BlkStat> {
    let (a, b) = (a?, b?);
    Some(BlkStat {
        read_ios: b.read_ios.saturating_sub(a.read_ios),
        read_sectors: b.read_sectors.saturating_sub(a.read_sectors),
        write_ios: b.write_ios.saturating_sub(a.write_ios),
        write_sectors: b.write_sectors.saturating_sub(a.write_sectors),
    })
}

/// 把「我数的」与「块层数的」并排打出来。读不到块层读数就明说读不到，
/// **绝不静默当成 0**（`rules/test-discipline.md`：读不到 ≠ 读到 0）。
fn fmt_blk(d: Option<BlkStat>) -> String {
    match d {
        Some(b) => format!(
            "blk_reads={} blk_writes={} blk_read_kib={} blk_write_kib={}",
            b.read_ios, b.write_ios, b.read_sectors / 2, b.write_sectors / 2
        ),
        None => "blk_reads=NA blk_writes=NA blk_read_kib=NA blk_write_kib=NA".to_string(),
    }
}

const O_DIRECT: i32 = 0o40000;
const PAGE: usize = 4096;
const COUNTERS_PER_PAGE: u64 = (PAGE / 8) as u64; // 每个计数器 8 字节
const DEADLIST_REC: usize = 16; // deadlist 一条定长记录
const DEADLIST_BUF: usize = 1024 * 1024;

struct Aligned {
    ptr: *mut u8,
    len: usize,
    layout: Layout,
}
impl Aligned {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, PAGE).expect("对齐参数非法");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(ptr, 0, len) };
        Self { ptr, len, layout }
    }
    fn s(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
    fn m(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
impl Drop for Aligned {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) }
    }
}

fn next_rand(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

struct Dev {
    f: std::fs::File,
    io: IoCounters,
}
impl Dev {
    fn read_page(&mut self, page: u64, buf: &mut Aligned) {
        self.f.seek(SeekFrom::Start(page * PAGE as u64)).expect("seek");
        self.f.read_exact(buf.m()).expect("读页失败");
        self.io.reads += 1;
        self.io.bytes_read += PAGE as u64;
    }
    fn write_page(&mut self, page: u64, buf: &Aligned) {
        self.f.seek(SeekFrom::Start(page * PAGE as u64)).expect("seek");
        self.f.write_all(buf.s()).expect("写页失败");
        self.io.writes += 1;
        self.io.bytes_written += PAGE as u64;
    }
    fn append(&mut self, off: u64, buf: &Aligned, len: usize) {
        self.f.seek(SeekFrom::Start(off)).expect("seek");
        self.f.write_all(&buf.s()[..len]).expect("顺序写失败");
        self.io.writes += 1;
        self.io.bytes_written += len as u64;
    }
}

/// 臂 A：D5 时间序。比较两个标量（都在手上），追加一条定长记录，缓冲满了顺序刷。
/// D5 的删除判定：**块的 birth 比最近一个快照还新 ⇒ 直接释放，连 deadlist 都不写。**
///
/// 这就是本实验要量的那个机制。抽成函数是为了让单测走**生产路径**——
/// 测试里再写一遍判据，就测不到 `arm_d5` 里实际用的那一份
/// （E21 的分块逻辑踩过这个坑，变异测试当场证明）。
fn is_direct_release(birth: u64, prev_snap_txg: u64) -> bool {
    birth > prev_snap_txg
}

fn arm_d5(dev: &mut Dev, ops: u64, seed: u64, deadlist_base: u64) -> IoCounters {
    dev.io = IoCounters::default();
    let mut buf = Aligned::new(DEADLIST_BUF);
    let mut used = 0usize;
    let mut off = deadlist_base;
    let mut st = seed;
    let prev_snap_txg: u64 = 1_000_000; // 常驻内存的一个标量
    for _ in 0..ops {
        let birth = next_rand(&mut st) % 2_000_000; // 指针里已有的字段，零次额外 I/O
        if is_direct_release(birth, prev_snap_txg) {
            continue; // 直接释放：连 deadlist 都不写
        }
        buf.m()[used..used + 8].copy_from_slice(&birth.to_le_bytes());
        used += DEADLIST_REC;
        if used + DEADLIST_REC > DEADLIST_BUF {
            dev.append(off, &buf, used);
            off += used as u64;
            used = 0;
        }
    }
    if used > 0 {
        // O_DIRECT 的长度也必须块对齐——向上取整到整页（尾部是零填充，不影响计量口径）
        dev.append(off, &buf, used.next_multiple_of(PAGE));
    }
    dev.io
}

/// 臂 B：引用计数。定位计数器页 → 缺页则 O_DIRECT 读 → 减一 → 标脏 → 逐出时回写。
fn arm_refcount(
    dev: &mut Dev,
    ops: u64,
    seed: u64,
    counters: u64,
    cache_pages: usize,
    sequential: bool,
) -> IoCounters {
    dev.io = IoCounters::default();
    let mut lru = Lru::new(cache_pages);
    let mut page_buf = Aligned::new(PAGE);
    let mut st = seed;
    for i in 0..ops {
        // sequential 故障：把随机访问换成顺序，命中率应当大幅上升
        let blk = if sequential { i % counters } else { next_rand(&mut st) % counters };
        let pg = page_of(blk, COUNTERS_PER_PAGE);
        if !lru.contains(pg) {
            if let Some(ev) = lru.touch(pg) {
                if lru.take_dirty(ev) {
                    dev.write_page(ev, &page_buf); // 回写被逐出的脏页
                }
            }
            dev.read_page(pg, &mut page_buf);
        } else {
            lru.touch(pg);
        }
        // 减一：读-改-写那个「改」
        let idx = ((blk % COUNTERS_PER_PAGE) * 8) as usize;
        let v = u64::from_le_bytes(page_buf.s()[idx..idx + 8].try_into().unwrap());
        page_buf.m()[idx..idx + 8].copy_from_slice(&v.saturating_sub(1).to_le_bytes());
        lru.mark_dirty(pg);
    }
    for pg in lru.drain_dirty() {
        dev.write_page(pg, &page_buf);
    }
    dev.io
}

/// 臂 C：D5 + 稀疏旁表。先探内存位图；未被 clone 走 A，被 clone 走 B。
/// `clone_permille` 控制被 clone 的比例。
fn arm_hybrid(
    dev: &mut Dev,
    ops: u64,
    seed: u64,
    counters: u64,
    cache_pages: usize,
    clone_permille: u64,
    deadlist_base: u64,
) -> (IoCounters, u64) {
    dev.io = IoCounters::default();
    // 稀疏旁表的探测结构：每块 1 bit，常驻内存
    let bitmap: Vec<u64> = {
        let words = (counters as usize).div_ceil(64);
        let mut b = vec![0u64; words];
        let mut s2 = seed ^ 0xDEAD_BEEF;
        for i in 0..counters {
            if next_rand(&mut s2) % 1000 < clone_permille {
                b[(i / 64) as usize] |= 1 << (i % 64);
            }
        }
        b
    };
    let is_cloned = |blk: u64| bitmap[(blk / 64) as usize] & (1 << (blk % 64)) != 0;

    let mut lru = Lru::new(cache_pages);
    let mut page_buf = Aligned::new(PAGE);
    let dl = Aligned::new(DEADLIST_BUF);
    let mut used = 0usize;
    let mut off = deadlist_base;
    let mut st = seed;
    let mut cloned_hits = 0u64;
    for _ in 0..ops {
        let blk = next_rand(&mut st) % counters;
        if !is_cloned(blk) {
            // 未被 clone：探测零设备 I/O，走 D5 路径
            used += DEADLIST_REC;
            if used + DEADLIST_REC > DEADLIST_BUF {
                dev.append(off, &dl, used);
                off += used as u64;
                used = 0;
            }
            continue;
        }
        cloned_hits += 1;
        let pg = page_of(blk, COUNTERS_PER_PAGE);
        if !lru.contains(pg) {
            if let Some(ev) = lru.touch(pg) {
                if lru.take_dirty(ev) {
                    dev.write_page(ev, &page_buf);
                }
            }
            dev.read_page(pg, &mut page_buf);
        } else {
            lru.touch(pg);
        }
        let idx = ((blk % COUNTERS_PER_PAGE) * 8) as usize;
        let v = u64::from_le_bytes(page_buf.s()[idx..idx + 8].try_into().unwrap());
        page_buf.m()[idx..idx + 8].copy_from_slice(&v.saturating_sub(1).to_le_bytes());
        lru.mark_dirty(pg);
    }
    if used > 0 {
        dev.append(off, &dl, used.next_multiple_of(PAGE));
    }
    for pg in lru.drain_dirty() {
        dev.write_page(pg, &page_buf);
    }
    (dev.io, cloned_hits)
}

fn main() {
    let dev_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e12-lifecycle <块设备> [种子]");
        std::process::exit(2)
    });
    // 第三个参数是**故障注入模式**：用来验证这套测量本身有没有判别力。
    // 不注入故障就无法分辨「比值 1.00 = 一切正确」与「比值 1.00 = 校验路径是摆设」。
    let fault = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    let use_direct = fault != "nodirect";
    let mut oo = OpenOptions::new();
    oo.read(true).write(true);
    if use_direct {
        oo.custom_flags(O_DIRECT);
    }
    let f = oo
        .open(&dev_path)
        .unwrap_or_else(|e| {
            eprintln!("打不开 {dev_path}（O_DIRECT）：{e}");
            std::process::exit(3)
        });
    let mut dev = Dev { f, io: IoCounters::default() };
    let size = dev.f.seek(SeekFrom::End(0)).expect("取不到设备大小");
    if size == 0 {
        eprintln!("设备大小为 0 —— 判定不明，整轮作废");
        std::process::exit(5);
    }

    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!("name=device_size bytes={size}")));

    // 档位（表大小）永远按基准缓存 256 页定义，**不随故障注入变化**——
    // 否则放大缓存时表也跟着放大，比值不变，什么也没测到（第一版就是这么错的）。
    let base_cache_pages: usize = 256;
    // bigcache 故障：只放大 LRU 容量，让 8× 档的整张表装得下，缺页率应当归零
    let cache_pages: usize = if fault == "bigcache" { 2048 } else { base_cache_pages };
    let ops: u64 = 200_000;
    // 种子从命令行来：每轮换一个，验证结论不是某一个序列的巧合。
    // 同一轮内三臂共用同一个种子——否则比的不是同一件事。
    let seed: u64 = std::env::args()
        .nth(2)
        .and_then(|x| x.parse().ok())
        .unwrap_or(0x5156_1234_ABCD_0001);
    let deadlist_base = size / 2; // 落在设备后半段，避开计数器表
    let cache_counters = base_cache_pages as u64 * COUNTERS_PER_PAGE; // 档位基准，不随故障变
    let blk_ok = blkstat(&dev_path).is_some();

    // 读不到块层读数就明说，绝不静默当成 0：校验路径没了要让人看见
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config cache_pages={cache_pages} cache_counters={cache_counters} ops={ops} counters_per_page={COUNTERS_PER_PAGE} seed={seed} blkstat_available={blk_ok} fault={fault} o_direct={use_direct}"
        ))
    );
    if !blk_ok {
        eprintln!("读不到 /sys/block/*/stat —— 校验路径缺失，本轮结果不许当成已校验");
    }

    for (label, mult) in [("fits", 1u64), ("x2", 2), ("x8", 8)] {
        let counters = cache_counters * mult;
        if (counters / COUNTERS_PER_PAGE + 1) * PAGE as u64 > size / 2 {
            eprintln!("设备太小，装不下 {label} 档的计数器表");
            std::process::exit(6);
        }

        let s0 = blkstat(&dev_path);
        let t = Instant::now();
        let a = arm_d5(&mut dev, ops, seed, deadlist_base);
        let a_ns = t.elapsed().as_nanos() as u64;
        let a_blk = fmt_blk(blk_delta(s0, blkstat(&dev_path)));
        println!("{}", em.emit_raw(&format!(
            "name=d5 ws={label} counters={counters} reads={} writes={} bytes_w={} elapsed_ns={a_ns} io_per_op={:.6} {a_blk}",
            a.reads, a.writes, a.bytes_written, a.io_per_op(ops).unwrap()
        )));

        let s0 = blkstat(&dev_path);
        let t = Instant::now();
        let b = arm_refcount(&mut dev, ops, seed, counters, cache_pages, fault == "sequential");
        let b_ns = t.elapsed().as_nanos() as u64;
        let b_blk = fmt_blk(blk_delta(s0, blkstat(&dev_path)));
        println!("{}", em.emit_raw(&format!(
            "name=refcount ws={label} counters={counters} reads={} writes={} elapsed_ns={b_ns} io_per_op={:.6} {b_blk}",
            b.reads, b.writes, b.io_per_op(ops).unwrap()
        )));

        let s0 = blkstat(&dev_path);
        let t = Instant::now();
        let (c, hits) = arm_hybrid(&mut dev, ops, seed, counters, cache_pages, 10, deadlist_base);
        let c_ns = t.elapsed().as_nanos() as u64;
        let c_blk = fmt_blk(blk_delta(s0, blkstat(&dev_path)));
        println!("{}", em.emit_raw(&format!(
            "name=hybrid ws={label} counters={counters} clone_permille=10 cloned_hits={hits} reads={} writes={} elapsed_ns={c_ns} io_per_op={:.6} {c_blk}",
            c.reads, c.writes, c.io_per_op(ops).unwrap()
        )));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **D5 的删除判定：birth 比最近快照新 ⇒ 直接释放。**
    /// 边界必须是严格大于——等于最近快照代号的块**仍被那个快照引用**，不能直接释放。
    /// 判反了会让实验量到一个不存在的收益。
    #[test]
    fn direct_release_boundary_is_strictly_greater() {
        assert!(is_direct_release(1_000_001, 1_000_000), "更新的块应当直接释放");
        assert!(!is_direct_release(1_000_000, 1_000_000), "等于最近快照代号的块仍被引用，不许直接释放");
        assert!(!is_direct_release(999_999, 1_000_000), "更老的块必须进 deadlist");
    }

    /// 随机数必须真的散开。它决定「多大比例的块走直接释放」，
    /// 退化成常数的话两条臂的 I/O 比就是假的。
    #[test]
    fn next_rand_is_not_degenerate() {
        let mut st = 7u64;
        let v: Vec<u64> = (0..1000).map(|_| next_rand(&mut st) % 2_000_000).collect();
        let uniq: std::collections::HashSet<_> = v.iter().collect();
        assert!(uniq.len() > 900, "1000 次只取到 {} 个不同值，没散开", uniq.len());
        // 且直接释放的比例应当接近一半（阈值取在 2_000_000 的中点 1_000_000）
        let direct = v.iter().filter(|&&b| is_direct_release(b, 1_000_000)).count();
        assert!((400..=600).contains(&direct), "直接释放比例 {direct}/1000 偏得太远，负载不是设想的那个");
    }

    /// **`O_DIRECT` 要求缓冲按页对齐。** 不对齐的话读写会 `EINVAL`，
    /// 而那会表现为「I/O 计数为 0」——看起来像「很省」，实际是根本没跑。
    #[test]
    fn aligned_buffer_is_page_aligned() {
        for len in [PAGE, PAGE * 4, DEADLIST_BUF] {
            let a = Aligned::new(len);
            assert_eq!(a.ptr as usize % PAGE, 0, "长度 {len} 的缓冲没有按页对齐");
            assert_eq!(a.len, len);
        }
    }

    /// 块层计数的差值必须逐字段相减；少减一个字段会让校验路径悄悄失效。
    #[test]
    fn blk_delta_subtracts_every_field() {
        let a = BlkStat { read_ios: 1, read_sectors: 3, write_ios: 2, write_sectors: 4 };
        let b = BlkStat { read_ios: 11, read_sectors: 33, write_ios: 22, write_sectors: 44 };
        let d = blk_delta(Some(a), Some(b)).expect("两侧都有读数时应当有差值");
        assert_eq!((d.read_ios, d.write_ios, d.read_sectors, d.write_sectors), (10, 20, 30, 40));
    }

    /// **读不到 ≠ 读到 0**：任一侧缺读数时差值必须是 None，不许当成 0。
    #[test]
    fn missing_blkstat_is_not_zero() {
        let a = BlkStat { read_ios: 1, read_sectors: 3, write_ios: 2, write_sectors: 4 };
        assert!(blk_delta(None, Some(a)).is_none());
        assert!(blk_delta(Some(a), None).is_none());
        assert!(blk_delta(None, None).is_none());
    }
}
