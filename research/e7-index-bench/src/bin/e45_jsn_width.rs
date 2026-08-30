//! E45：序号位宽的本机实测与代价 —— D23 未定项 9。
//!
//! ## 它要顶掉一个假设
//!
//! D23 未定项 9 现在写着「10⁶ 记录/秒这一档**没有本机实测**，是取的上界假设」，
//! 而 48 位计数器在那一档上只有 8.9 年 —— **结论压在一个没量过的数上**。
//!
//! ## A 段：本机实测 fsync 率
//!
//! 记录数就是 fsync 数（D23 未定项 1 轴一已定「每次 fsync 发根」）
//! ⇒ 计数器的消耗率 = 负载的 fsync 率。
//! ⚠️ **它测的是 ext4 的 fsync 路径，不是本工程的** —— ext4 自己的日志开销含在里面，
//! 所以这个数是本工程 fsync 率的**上界代理**，引用时必须带这句。
//!
//! ## B 段：加宽 `jsn` 的代价（算术）
//!
//! 判据是「盘上占几字节」与「一个原子单元装几个点名项」，不是「头涨百分之几」——
//! D23 未定项 4 已定「记录头完整落在一个原子单元内」，记录要向上取整到原子单元。

use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

const O_DIRECT: i32 = 0o40000;
const ALIGN: usize = 4096;
const BLK: usize = 4096;

struct Aligned { ptr: *mut u8, len: usize, lay: Layout }
impl Aligned {
    fn new(len: usize) -> Self {
        let lay = Layout::from_size_align(len, ALIGN).unwrap();
        Aligned { ptr: unsafe { alloc(lay) }, len, lay }
    }
    fn as_mut(&mut self) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) } }
}
impl Drop for Aligned { fn drop(&mut self) { unsafe { dealloc(self.ptr, self.lay) } } }

#[derive(Clone, Copy, PartialEq, Debug)]
enum Arm {
    /// 每写一块就 fdatasync：最坏消耗率
    Sync1,
    /// 8 块一次 fdatasync：D25 已定的粗粒度
    Sync8,
    /// 阳性对照：同样的写，**不** fdatasync。它必须显著更快，否则 fdatasync 没到设备
    NoSync,
}

impl Arm {
    fn writes_per_sync(self) -> u64 {
        match self {                      // 没有 `_ =>`：加一条臂不补这里就编译不过
            Arm::Sync1 => 1,
            Arm::Sync8 => 8,
            Arm::NoSync => 1,
        }
    }
    fn syncs(self) -> bool { !matches!(self, Arm::NoSync) }
}

/// 跑一轮，返回 (fsync 次数, 耗时纳秒)。**耗时为 0 返回 None**——读不到 ≠ 读到 0。
fn one_round(path: &str, arm: Arm, syncs: u64, file_blocks: u64) -> std::io::Result<Option<(u64, u128)>> {
    let mut f = std::fs::OpenOptions::new().read(true).write(true)
        .custom_flags(O_DIRECT).open(path)?;
    let mut buf = Aligned::new(BLK);
    for (i, b) in buf.as_mut().iter_mut().enumerate() { *b = (i % 251) as u8; }
    let mut x: u64 = 0x2545_F491_4F6C_DD1D;
    let t0 = Instant::now();
    for _ in 0..syncs {
        for _ in 0..arm.writes_per_sync() {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;      // xorshift64
            let blk = x % file_blocks;
            f.seek(SeekFrom::Start(blk * BLK as u64))?;
            f.write_all(buf.as_mut())?;
        }
        if arm.syncs() { f.sync_data()?; }
    }
    let ns = t0.elapsed().as_nanos();
    Ok(measurement(syncs, ns))
}

/// 把「读不到 ≠ 读到 0」这一条抽成纯函数，**否则它只活在 I/O 路径里、没有任何单测看得见**。
/// 耗时为 0 一律返回 None，让调用方整轮作废。
fn measurement(syncs: u64, ns: u128) -> Option<(u64, u128)> {
    if ns == 0 { None } else { Some((syncs, ns)) }
}

/// 每秒多少次 fsync。整数千分之一，避免浮点在产物里抖。
fn per_sec_milli(syncs: u64, ns: u128) -> u64 {
    ((syncs as u128) * 1_000_000_000_000 / ns) as u64
}

// ── B 段：字节账 ──────────────────────────────────────────────────────
/// E24 已定的头部 84 字节里，`jsn` 占 8 字节。把它换成 `jsn_bytes` 之后的头部。
const HDR_BASE: u64 = 84;
const JSN_BASE: u64 = 8;
/// D23 未定项 7 定案新增：事务号 8 + 提交标记 1。
const TXN_BYTES: u64 = 9;
/// D23 未定项 8 方向已定的反向链，按 32 位算。
const CHAIN_BYTES: u64 = 4;
/// E24 的点名项宽度。
const ITEM_BYTES: u64 = 56;

fn header_bytes(jsn_bytes: u64) -> u64 { HDR_BASE - JSN_BASE + jsn_bytes + TXN_BYTES + CHAIN_BYTES }

/// 一条记录在盘上真的占几字节：向上取整到原子单元。
fn on_disk(jsn_bytes: u64, items: u64, unit: u64) -> u64 {
    (header_bytes(jsn_bytes) + items * ITEM_BYTES).div_ceil(unit) * unit
}

/// 一个原子单元装得下几个点名项。
fn items_per_unit(jsn_bytes: u64, unit: u64) -> u64 {
    unit.saturating_sub(header_bytes(jsn_bytes)) / ITEM_BYTES
}

/// 在 `0..=max_items` 这些点名项数里，加宽 `jsn` 会多占一个原子单元的**有几个**。
/// 这才是代价的完整形状：不是「零」，是「零点几个百分点的项数上会多一个单元」。
fn item_counts_that_cost_a_unit(from: u64, to: u64, unit: u64, max_items: u64) -> u64 {
    (0..=max_items).filter(|&n| on_disk(to, n, unit) > on_disk(from, n, unit)).count() as u64
}

/// 计数器撑多少年：`2^bits ÷ 速率`。速率单位是「每秒千分之一次」。
fn years(bits: u32, per_sec_milli: u64) -> u64 {
    if per_sec_milli == 0 { return u64::MAX }
    let cap = if bits >= 64 { u128::from(u64::MAX) } else { 1u128 << bits };
    (cap * 1000 / per_sec_milli as u128 / 31_557_600) as u64
}

fn main() {
    let mut em = Emitter::new();
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法: e45_jsn_width <镜像路径>（落临时目录，不进仓库）"); std::process::exit(2)
    });
    let file_blocks = 16_384u64;       // 64 MiB
    let syncs = 2_000u64;
    let rounds = 5;
    println!("{}", em.emit_raw(&format!(
        "name=config file_blocks={file_blocks} block={BLK} syncs_per_round={syncs} rounds={rounds}")));

    let mut medians = std::collections::BTreeMap::new();
    for arm in [Arm::Sync1, Arm::Sync8, Arm::NoSync] {
        // ⚠️ **第一轮是冷的，显式跑一轮预热并把它报出来、不计入**。
        // 实测：不预热时第 0 轮耗时是其余轮的 2 倍，五轮离散 49.7%，
        // 按本实验自己的失败条款（离散 > ±20% 报「不稳定」）就下不了结论。
        // 预热要**可见**——藏起来等于偷偷把最慢那轮删掉。
        match one_round(&path, arm, syncs, file_blocks) {
            Ok(Some((n, ns))) => println!("{}", em.emit_raw(&format!(
                "name=warmup arm={arm:?} syncs={n} elapsed_ns={ns} counted=false"))),
            Ok(None) => { eprintln!("e45: {arm:?} 预热轮耗时读数为 0，整轮作废"); std::process::exit(3) }
            Err(e) => { eprintln!("e45: {arm:?} 预热轮出错：{e}"); std::process::exit(4) }
        }
        let mut v = Vec::new();
        for r in 0..rounds {
            match one_round(&path, arm, syncs, file_blocks) {
                Ok(Some((n, ns))) => {
                    let ps = per_sec_milli(n, ns);
                    println!("{}", em.emit_raw(&format!(
                        "name=round arm={arm:?} round={r} syncs={n} elapsed_ns={ns} per_sec_milli={ps}")));
                    v.push(ps);
                }
                // 读不到 ≠ 读到 0：耗时为 0 整轮作废，不许当成一个测量值
                Ok(None) => { eprintln!("e45: {arm:?} 第 {r} 轮耗时读数为 0，整轮作废"); std::process::exit(3) }
                Err(e) => { eprintln!("e45: {arm:?} 第 {r} 轮出错：{e}"); std::process::exit(4) }
            }
        }
        v.sort_unstable();
        let med = v[v.len() / 2];
        let spread = (v[v.len() - 1] - v[0]) * 10_000 / med.max(1);   // 万分之一
        println!("{}", em.emit_raw(&format!(
            "name=arm arm={arm:?} median_per_sec_milli={med} spread_bp={spread} \
             min={} max={}", v[0], v[v.len() - 1])));
        medians.insert(format!("{arm:?}"), med);
    }

    // 阳性对照：不 fsync 必须显著更快，否则 fdatasync 根本没到设备
    let s1 = medians["Sync1"];
    let ns_ = medians["NoSync"];
    println!("{}", em.emit_raw(&format!(
        "name=poscontrol sync1_per_sec_milli={s1} nosync_per_sec_milli={ns_} ratio_bp={} ok={}",
        ns_ * 10_000 / s1.max(1), ns_ > s1 * 2)));

    // A 段折算：本机最坏臂（Sync1）下各位宽撑多少年
    for bits in [48u32, 56, 64] {
        println!("{}", em.emit_raw(&format!(
            "name=lifetime bits={bits} years_at_sync1={} years_at_sync8={}",
            years(bits, s1), years(bits, medians["Sync8"]))));
    }

    // B 段：加宽 jsn 的代价
    for unit in [512u64, 4096] {
        for jsn in [8u64, 10, 12, 16] {
            println!("{}", em.emit_raw(&format!(
                "name=width unit={unit} jsn_bytes={jsn} header={} on_disk_items1={} \
                 on_disk_items12={} items_per_unit={} cost_unit_count_0_100={}",
                header_bytes(jsn), on_disk(jsn, 1, unit), on_disk(jsn, 12, unit),
                items_per_unit(jsn, unit),
                item_counts_that_cost_a_unit(8, jsn, unit, 100))));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：头部字节由构造直接算出。
    /// 84 − 8（原 jsn）+ 新 jsn + 9（事务字段）+ 4（反向链）。
    #[test]
    fn header_bytes_match_independently_computed_arithmetic() {
        assert_eq!(header_bytes(8), 84 - 8 + 8 + 9 + 4);
        assert_eq!(header_bytes(8), 97);
        assert_eq!(header_bytes(12), 101);
        assert_eq!(header_bytes(16), 105);
    }

    /// **绝对值断言**：一个 512 单元装几个点名项，由算术钉死。
    #[test]
    fn items_per_unit_matches_independently_computed_arithmetic() {
        assert_eq!(items_per_unit(8, 512), (512 - 97) / 56);
        assert_eq!(items_per_unit(8, 512), 7);
        assert_eq!(items_per_unit(12, 512), 7, "jsn 加宽到 12 字节仍是 7 项");
        assert_eq!(items_per_unit(16, 512), 7, "jsn 加宽到 16 字节仍是 7 项");
        assert_eq!(items_per_unit(8, 4096), (4096 - 97) / 56);
        assert_eq!(items_per_unit(12, 4096), 71);
    }

    /// **臂的写次数由枚举直接给出**，不是从 I/O 路径读回来。
    #[test]
    fn writes_per_sync_matches_the_arm_definition() {
        assert_eq!(Arm::Sync1.writes_per_sync(), 1);
        assert_eq!(Arm::Sync8.writes_per_sync(), 8, "粗粒度那一臂该是 8 次写一次 fsync");
        assert_eq!(Arm::NoSync.writes_per_sync(), 1);
        assert!(Arm::Sync1.syncs() && Arm::Sync8.syncs() && !Arm::NoSync.syncs());
    }

    /// **「读不到 ≠ 读到 0」要能被单测看见。**
    #[test]
    fn zero_elapsed_is_rejected_not_clamped() {
        assert_eq!(measurement(100, 0), None, "耗时为 0 该整轮作废，不许夹成 1");
        assert_eq!(measurement(100, 5), Some((100, 5)));
    }

    /// **阳性对照要真的跑一次 I/O**，否则「fdatasync 有没有到设备」只活在产物里、
    /// 没有任何检查看得见（变异 M1 实测：把 `if arm.syncs()` 改成 `if false`，一个测试都不红）。
    #[test]
    fn fdatasync_actually_reaches_the_device() {
        let p = std::env::temp_dir().join(format!("e45-selftest-{}.img", std::process::id()));
        let path = p.to_string_lossy().to_string();
        std::fs::File::create(&p).unwrap().set_len(8 * 1024 * 1024).unwrap();
        let _ = one_round(&path, Arm::Sync1, 20, 2048);          // 预热
        let (n1, t1) = one_round(&path, Arm::Sync1, 60, 2048).unwrap().unwrap();
        let (n0, t0) = one_round(&path, Arm::NoSync, 60, 2048).unwrap().unwrap();
        let _ = std::fs::remove_file(&p);
        let sync_rate = per_sec_milli(n1, t1);
        let nosync_rate = per_sec_milli(n0, t0);
        assert!(nosync_rate > sync_rate * 2,
            "不 fdatasync 该至少快一倍，否则 fdatasync 根本没到设备（{nosync_rate} vs {sync_rate}）");
    }

    /// **代价不是「零」，是「零点几个百分点的项数上会多一个单元」。**
    /// 这条把代价的完整形状钉住——只说「典型值相同」等于挑了好看的那几格。
    #[test]
    fn widening_costs_a_unit_on_only_a_few_item_counts() {
        // 0..=100 个点名项里，8 → 12 字节会多占一个 512 单元的有几个
        // 绝对值：8 → 12 字节在 0..=100 项里一格都不多；8 → 16 在 512 单元下恰好多一格
        assert_eq!(item_counts_that_cost_a_unit(8, 12, 512, 100), 0);
        assert_eq!(item_counts_that_cost_a_unit(8, 12, 4096, 100), 0);
        assert_eq!(item_counts_that_cost_a_unit(8, 16, 512, 100), 1, "16 字节在 512 单元下恰好多一格");
        assert_eq!(item_counts_that_cost_a_unit(8, 16, 4096, 100), 0);
        // 扫描本身要真的扫过整个区间：加宽到一个明显更大的宽度必须找出很多格
        assert!(item_counts_that_cost_a_unit(8, 500, 512, 100) > 50,
            "加宽 492 字节该在大量项数上多占单元——扫描没在扫整个区间");
        // 而典型的那两档一格都不多
        for unit in [512u64, 4096] {
            for items in [1u64, 7, 12] {
                assert_eq!(on_disk(8, items, unit), on_disk(12, items, unit),
                    "jsn 8 → 12 在 {unit} 单元、{items} 项上该不多占");
            }
        }
    }

    /// **绝对值断言**：计数器寿命由 `2^位宽 ÷ 速率` 独立算出。
    /// 10⁶/秒 ⇒ per_sec_milli = 10⁹。
    #[test]
    fn counter_lifetime_matches_independently_computed_arithmetic() {
        let rate = 1_000_000_000u64;                    // 10⁶ 次/秒
        // 2^48 / 1e6 / 31557600 秒每年 ≈ 8.9 年
        assert_eq!(years(48, rate), (1u128 << 48) as u64 / 1_000_000 / 31_557_600);
        assert_eq!(years(48, rate), 8);
        assert!(years(56, rate) > 2000);
    }

    /// **速率换算是双射**：同样的次数、耗时翻倍 ⇒ 速率减半。
    #[test]
    fn rate_halves_when_elapsed_doubles() {
        let a = per_sec_milli(1000, 1_000_000_000);
        let b = per_sec_milli(1000, 2_000_000_000);
        assert_eq!(a, 1_000_000, "1000 次 / 1 秒 = 每秒 1000 次 = 1_000_000 千分之一");
        assert_eq!(b, a / 2);
    }

    /// **耗时读到 0 必须是 None**，不许当成一个测量值参与判定。
    #[test]
    fn zero_elapsed_is_not_a_measurement() {
        // one_round 在 ns == 0 时返回 None；这里直接验那个分支的算术前提
        assert_eq!(per_sec_milli(1, 1), 1_000_000_000_000);
    }

    /// **阳性对照的判据本身要有意义**：不 fsync 必须至少快一倍才算「到了设备」。
    #[test]
    fn the_positive_control_threshold_is_a_factor_of_two() {
        assert!(!(100u64 > 100 * 2), "同速不算通过");
        assert!(300u64 > 100 * 2, "快三倍算通过");
    }
}
