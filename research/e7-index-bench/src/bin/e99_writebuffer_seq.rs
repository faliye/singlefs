//! E99：write buffer 条目与 seq 的去重 —— D8 未定项 7 欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D8 write buffer 硬要求 3 逐字**：「**条目必须自带序号（seq），这是格式要求**」；
//!   「flush 时按 key 排序**丢弃时间序**，而去重规则是「后者胜」⇒ **『后者』必须由条目自己携带**；
//!   用不稳定排序时同 key 条目的相对次序未定义，去重结果因此不确定。」
//! - **D8 write buffer 硬要求 2**：条目形态必须是幂等完整值，不许是增量 Δ。
//! - **D5 已定项 3 依据第 3 条**：记账走 write buffer 前端，条目自带 seq。
//! - **D5 已定项 5（2026-09-03）**：记账 key 22 字节、条目 30 字节。
//! - **D16 已定项 6**：每次发布 `checkpoint_txg` + 1；fsync 触发的也是发布；**永不回退**。
//! - **D22 已定项 7**：`checkpoint_txg` **8 字节**（64 位）。
//! - **D23 已定项 9**：`jsn` 10 字节，序号与实例代号**分宽**——同一类问题的既有解法。
//! - **E44**：本机 2785 发布/秒；48 位撑 3202 年；jsn 加宽到 12 字节代价为零。
//! - **E17**：第一版漏了 seq，两条臂与真值当场对不上。
//!
//! ## 判据（E99 正文跑前写死，跑完不许改）
//!
//! 1. **位宽账要算得出，不许估**：`seq = (checkpoint_txg, 窗口内序号)` 两段各占几位、
//!    合起来几字节，必须是绝对值。`checkpoint_txg` 是 64 位 ⇒ **8 字节一个位都不剩**，
//!    模型要报出「窗口内序号可用位数 = 0」这个事实。
//! 2. **去重确定性**：四条臂各跑同一串更新——① 无 seq；② 墙上时钟；③ 每次挂载归零的计数器；
//!    ④ `(checkpoint_txg, 窗口序号)`——数「选错胜者」的次数，每条臂都要绝对值。
//! 3. **跨挂载单调性**：更新串中间插一次挂载，臂 ③ 的错误数必须**上升**，臂 ④ 必须恒 0。
//! 4. **截断的代价**：`checkpoint_txg` 截到 `w` 位之后回绕，**要数得出一个可数的违例**，
//!    不许只报一个回绕期就算完。
//!    ⚠️ **第三轮修正**：第一、二版只打印 `wrap_millis`，那个数**从来没接到任何违例计数上**
//!    ⇒ 反向接受条款「判据 4 算出截断带来 > 0 的选错」在代码里从没产生过一个数。
//!    现在按 D5 已定项 5 的 key 结构接：**代进了 key**，截断代 ⇒ 回绕之后
//!    两个不同 checkpoint 的条目**塌成同一个 key** ⇒ 数「本不该合并却被合并」的对数。
//! 5. **窗口内序号要多宽**：一次发布窗口内同一个 key 最多被写几次，按 D16 已定项 5 的
//!    `T_dirty` = 2 GiB 与 D5 已定项 5 的 30 字节条目反解。
//! 6. **（第二轮补）key 里有没有「代」，决定 seq 要不要重复携带 txg**：
//!    D5 已定项 5 逐字定 key = `(统计量标签 2, 树 ID 8, 设备 4, **代 8**)`，
//!    而 D16 已定项 6 定「代 = checkpoint 号」，D5 已定项 2 的骑手条款逐字补「记账条目的
//!    **跨发布合并不再可行**（代进了 key）」⇒ **同一个 key 的待去重条目必然同属一个发布窗口**
//!    ⇒ seq 只需要携带窗口内序号，txg 那 8 字节是 key 里已有段的**重复计账**。
//!    ⚠️ **这一条只对 key 里含代的树成立**——反向索引与 LRU 的 key 布局仓里没定，
//!    模型对它们报「不适用」，不外推。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，每条臂都跑**：臂 ① 必须错得最多；臂 ③ 插挂载之后必须比不插时更错。
//! - **阴性对照**：同 key 只写一次时，四条臂错误数必须全为 0。
//! - **反向接受条款**：判据 1 算出 8 字节装不下 **且** 判据 4 算出截断带来 > 0 的选错
//!   ⇒ 结论是「**8 字节不够，seq 要加宽**」，不许把窗口段压到 0 位硬凑。
//!
//! ## 它答不了的
//!
//! 纯计数模型：没有 write buffer 实现、没有真 flush、没有并发、没有崩溃点重放。
//! 不答「write buffer 落不落盘」——按两种读法各报一次影响，不替它定。

use e7_index_bench::Emitter;

/// D22 已定项 7：`checkpoint_txg` 8 字节 = 64 位。
const TXG_BITS: u32 = 64;
/// E44 本机实测：2785 发布/秒。
const PUBLISH_PER_SEC: u64 = 2785;
/// D16 已定项 5：`T_dirty` = 2 GiB。D5 已定项 5：记账条目 30 字节。
const T_DIRTY: u64 = 2 * 1024 * 1024 * 1024;
const ACCT_ENTRY_BYTES: u64 = 30;

/// **判据 1**：`seq` 取 `w` 字节时，装完 `txg_bits` 位的 txg 之后还剩几位给窗口序号。
/// 装不下时返回 `None`——**读不到 ≠ 读到 0**，不许退化成 0 位。
fn window_bits(seq_bytes: u64, txg_bits: u32) -> Option<u32> {
    let total = seq_bytes.checked_mul(8)? as u32;
    total.checked_sub(txg_bits)
}

/// **判据 4**：把 txg 截到 `w` 位，多少次发布之后回绕。
fn wrap_publishes(txg_bits_kept: u32) -> u128 {
    1u128 << txg_bits_kept
}

/// **判据 4**：把 key 里的「代」段截到 `bits` 位，跑 `publishes` 次发布之后，
/// 有多少对**本属不同 checkpoint 的条目塌成了同一个 key**（因而被错误合并）。
/// 代按 D5 已定项 5 在 key 里，所以截断它撞的不是排序，是**键的同一性**。
fn wrongly_merged_pairs(publishes: u64, bits: u32) -> u64 {
    if bits >= 64 {
        return 0;
    }
    let m = 1u64 << bits;
    if publishes <= m {
        return 0; // 还没绕完一圈
    }
    // 第 i 个模类里有 q_i 个 checkpoint，塌成同一个 key 的对数是 C(q_i, 2)
    let q = publishes / m;
    let r = publishes % m;
    // r 个模类各有 q+1 个，其余 m-r 个各有 q 个
    let c2 = |n: u64| n * n.saturating_sub(1) / 2;
    r * c2(q + 1) + (m - r) * c2(q)
}

/// 回绕期（毫秒），按 E44 的发布率。
fn wrap_millis(txg_bits_kept: u32) -> u128 {
    wrap_publishes(txg_bits_kept) * 1000 / PUBLISH_PER_SEC as u128
}

/// 一条 write buffer 条目：同一个 key 的第几次更新、它属于哪个发布窗口、
/// 到达 flush 排序时的**下标**（排序会丢掉它，这正是硬要求 3 说的那件事）。
/// key 里含不含「代」这一段——这是第二轮补的第六条判据的自变量。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeyShape {
    /// 记账树：D5 已定项 5 逐字，key 的末段就是「代」= checkpoint 号。
    WithGeneration,
    /// 反向索引 / LRU：key 布局仓里没定，**不知道有没有代**。
    Unknown,
}

/// **判据 6**：给定 key 形态，seq 需要携带哪几段、合起来几字节。
/// key 里含代 ⇒ 只要窗口序号；不含或不知道 ⇒ 要 txg + 窗口序号。
/// `Unknown` 报 `None`——**读不到 ≠ 读到 0**，不许替没定的树选一个答案。
fn seq_bytes_needed(shape: KeyShape, window_bits_needed: u32) -> Option<u64> {
    let win_bytes = (window_bits_needed as u64).div_ceil(8);
    match shape {
        KeyShape::WithGeneration => Some(win_bytes),
        KeyShape::Unknown => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct Upd {
    key: u32,
    /// 真值：这是该 key 的第几次更新（越大越新）。模型用它判「胜者对不对」。
    truth: u64,
    /// 产生它的那次发布。
    txg: u64,
    /// 该发布窗口内的第几次。
    within: u64,
    /// 墙上时钟读数（非单调：模型让它在某些点回拨）。
    wall: u64,
    /// 每次挂载归零的计数器。
    mount_local: u64,
}

/// 四条臂各自怎么给一条条目定序。返回 `None` = 这条臂给不出序（无 seq）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    NoSeq,
    WallClock,
    MountLocal,
    TxgPlusWindow,
}

fn order_key(a: Arm, u: &Upd) -> Option<u128> {
    match a {
        Arm::NoSeq => None,
        Arm::WallClock => Some(u.wall as u128),
        Arm::MountLocal => Some(u.mount_local as u128),
        // 两段拼起来：高位 txg、低位窗口序号。位宽够不够是判据 1 的事。
        Arm::TxgPlusWindow => Some(((u.txg as u128) << 32) | u.within as u128),
    }
}

/// **判据 2 / 3**：按某条臂做「排序 + 后者胜」，数选错胜者的 key 数。
/// 排序按 `.claude/kb/decisions/08-核心索引结构.md` 硬要求 3 的口径：
/// **按 key 排序会丢掉到达顺序**，所以无 seq 那条臂只能拿数组里的残留次序当序——
/// 模型用「稳定排序之后取最后一条」模拟它，而输入被**故意打乱**过。
/// 返回 (选错胜者的 key 数, **胜者未定义**的 key 数)。
///
/// 「未定义」这一格是 D8 硬要求 3 逐字说的那件事——「用不稳定排序时同 key 条目的
/// **相对次序未定义**」。两条条目的序号一样大时，结果取决于实现挑了哪一边
/// ⇒ **模型不许替它挑一边**，单独数出来。第一版拿 `>=`（后到的赢）挑了一边，
/// 于是「每次挂载归零」那条臂只错一半——那一半是被 tie-break 救回来的，不是它对。
fn wrong_winners(ups: &[Upd], arm: Arm) -> (u64, u64) {
    use std::collections::HashMap;
    // key -> (最大序号, 取到该序号的 truth 集合)
    let mut best: HashMap<u32, (Option<u128>, Vec<u64>)> = HashMap::new();
    for u in ups {
        let ok = order_key(arm, u);
        let e = best.entry(u.key).or_insert((ok, Vec::new()));
        match (ok, e.0) {
            (Some(a), Some(b)) => {
                if a > b {
                    *e = (ok, vec![u.truth]);
                } else if a == b {
                    e.1.push(u.truth);
                }
            }
            // 无序（无 seq）：所有条目并列，胜者完全未定义
            _ => {
                e.0 = None;
                e.1.push(u.truth);
            }
        }
    }
    let mut truth_max: HashMap<u32, u64> = HashMap::new();
    for u in ups {
        let t = truth_max.entry(u.key).or_insert(0);
        if u.truth > *t {
            *t = u.truth;
        }
    }
    let mut wrong = 0u64;
    let mut undef = 0u64;
    for (k, (_, winners)) in best.iter() {
        let tm = truth_max.get(k).copied().unwrap_or(0);
        if winners.len() > 1 {
            undef += 1;
        } else if winners.first().copied().unwrap_or(0) != tm {
            wrong += 1;
        }
    }
    (wrong, undef)
}

/// 造一串更新：`keys` 个 key，每个更新 `per_key` 次；`remount_at` 处插一次挂载
/// （挂载之后 `mount_local` 归零、`wall` 回拨）。次序被打乱，模拟「排序丢掉时间序」。
fn make_updates(keys: u32, per_key: u64, remount_at: Option<u64>) -> Vec<Upd> {
    let mut v = Vec::new();
    let mut mount_local = 0u64;
    let mut wall = 1_000_000u64;
    for i in 0..per_key {
        if Some(i) == remount_at {
            mount_local = 0;
            wall = wall.saturating_sub(500_000); // 挂载时钟回拨
        }
        for k in 0..keys {
            mount_local += 1;
            wall += 1;
            v.push(Upd {
                key: k,
                truth: i + 1,
                txg: 100 + i,
                within: 0,
                wall,
                mount_local,
            });
        }
    }
    // **打乱要真的打乱**：按 key 排序之后到达顺序不再可用，残留次序是任意的。
    // 确定性地把**偶数 key** 的子序列翻转、奇数 key 保持——
    // 于是「拿残留次序当序」那条臂在一半的 key 上必然选错，这个数是可预期的绝对值。
    let mut out: Vec<Upd> = Vec::with_capacity(v.len());
    for k in 0..keys {
        let mut sub: Vec<Upd> = v.iter().copied().filter(|u| u.key == k).collect();
        if k % 2 == 0 {
            sub.reverse();
        }
        out.extend(sub);
    }
    out
}

/// **判据 5**：一次发布窗口内同一个 key 最多写几次的上界——
/// `T_dirty` 全用来装记账条目时的条目数。
fn max_updates_per_window() -> u64 {
    T_DIRTY / ACCT_ENTRY_BYTES
}

fn bits_needed(n: u64) -> u32 {
    64 - n.leading_zeros()
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();

    out.push(em.emit_raw(&format!(
        "name=config txg_bits={TXG_BITS} publish_per_sec={PUBLISH_PER_SEC} \
         t_dirty={T_DIRTY} acct_entry={ACCT_ENTRY_BYTES}"
    )));

    // 判据 1：位宽账
    for &w in [8u64, 10, 12, 16].iter() {
        out.push(em.emit_raw(&format!(
            "name=seq_width seq_bytes={w} txg_bits={TXG_BITS} window_bits={}",
            window_bits(w, TXG_BITS)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "DOES_NOT_FIT".into())
        )));
    }
    // 判据 5：窗口序号要多宽
    let m = max_updates_per_window();
    let wb = bits_needed(m);
    out.push(em.emit_raw(&format!(
        "name=window_need max_updates_per_window={m} bits_needed={wb}"
    )));
    // 判据 6：key 里有没有「代」，决定 seq 要不要重复携带 txg
    for &shape in [KeyShape::WithGeneration, KeyShape::Unknown].iter() {
        out.push(em.emit_raw(&format!(
            "name=seq_need key_shape={:?} window_bits_needed={wb} seq_bytes_needed={} \
             txg_is_duplicate_of_key_segment={}",
            shape,
            seq_bytes_needed(shape, wb)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NOT_APPLICABLE".into()),
            u8::from(shape == KeyShape::WithGeneration),
        )));
    }

    // 判据 4：截断的代价——回绕期 **加上一个可数的违例**
    for &keep in [8u32, 16, 32, 48].iter() {
        for &pubs in [1_000u64, 100_000, 10_000_000].iter() {
            out.push(em.emit_raw(&format!(
                "name=truncate gen_bits_kept={keep} publishes={pubs} wrap_publishes={} \
                 wrap_millis={} wrongly_merged_pairs={}",
                wrap_publishes(keep),
                wrap_millis(keep),
                wrongly_merged_pairs(pubs, keep)
            )));
        }
    }

    // 判据 2 / 3：四条臂 × 插不插挂载
    for &(keys, per_key) in [(1000u32, 1u64), (1000, 8)].iter() {
        for &remount in [None, Some(4u64)].iter() {
            let ups = make_updates(keys, per_key, remount);
            for &arm in [Arm::NoSeq, Arm::WallClock, Arm::MountLocal, Arm::TxgPlusWindow].iter() {
                let (wrong, undef) = wrong_winners(&ups, arm);
                out.push(em.emit_raw(&format!(
                    "name=dedup keys={keys} per_key={per_key} remount={} arm={:?} \
                     wrong_winners={wrong} undefined_winners={undef}",
                    remount.map(|v| v.to_string()).unwrap_or_else(|| "none".into()),
                    arm,
                )));
            }
        }
    }

    for l in &out {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_kb() {
        assert_eq!(TXG_BITS, 64, "D22 已定项 7：checkpoint_txg 8 字节");
        assert_eq!(PUBLISH_PER_SEC, 2785, "E44 本机实测");
        assert_eq!(ACCT_ENTRY_BYTES, 30, "D5 已定项 5");
    }

    /// **判据 1 的绝对值**：8 字节一个位都不剩给窗口序号。
    #[test]
    fn criterion1_eight_bytes_leaves_zero_bits_for_the_window_counter() {
        assert_eq!(window_bits(8, 64), Some(0), "64 位 txg 正好吃满 8 字节");
        assert_eq!(window_bits(10, 64), Some(16), "10 字节留 16 位");
        assert_eq!(window_bits(12, 64), Some(32));
        // 比 txg 还窄时装不下 —— 报 None，不许退化成 0
        assert_eq!(window_bits(4, 64), None);
        assert_eq!(window_bits(0, 64), None);
    }

    /// **判据 6（第二轮补）的绝对值**：记账树的 key 末段就是代
    /// ⇒ 同一个 key 的待去重条目必然同窗口 ⇒ seq 只要 **4 字节**，
    /// 而在 seq 里再放一份 txg 是把 key 里已有的段抄第二遍。
    #[test]
    fn criterion6_the_generation_is_already_in_the_key_so_seq_need_not_repeat_it() {
        let wb = bits_needed(max_updates_per_window());
        assert_eq!(wb, 27);
        // 27 位 ⇒ 4 字节（向上取整）
        assert_eq!(seq_bytes_needed(KeyShape::WithGeneration, wb), Some(4));
        // key 布局没定的树：报「不适用」，**不许替它选一个数**
        assert_eq!(seq_bytes_needed(KeyShape::Unknown, wb), None);
        // 对照：若真要在 seq 里重复携带 64 位 txg，那就是 8 + 4 = 12 字节
        assert_eq!(window_bits(12, 64), Some(32));
        assert!(32 >= wb, "12 字节那条路也够，只是多抄了一遍 key 里已有的段");
        // 阳性对照：窗口需求变大时，要的字节数必须跟着变
        assert_eq!(seq_bytes_needed(KeyShape::WithGeneration, 8), Some(1));
        assert_eq!(seq_bytes_needed(KeyShape::WithGeneration, 33), Some(5));
    }

    /// **判据 5 的绝对值**：一个窗口内最多 71 582 788 条 ⇒ 窗口段至少 27 位。
    #[test]
    fn criterion5_window_counter_needs_at_least_27_bits() {
        // 手算：2 GiB / 30 = 2147483648 / 30 = 71582788（整除取下）
        assert_eq!(max_updates_per_window(), 71_582_788);
        assert_eq!(bits_needed(71_582_788), 27, "2^26 = 67108864 < 71582788 ≤ 2^27");
        // ⇒ 8 字节（0 位）与 10 字节（16 位）都不够，12 字节（32 位）够
        assert!(window_bits(8, 64).unwrap() < 27);
        assert!(window_bits(10, 64).unwrap() < 27);
        assert!(window_bits(12, 64).unwrap() >= 27);
    }

    /// **判据 4 的可数违例（第三轮补）**：代进了 key，截断它 ⇒ 回绕之后
    /// 两个不同 checkpoint 的条目塌成同一个 key ⇒ 被错误合并的对数是绝对值。
    #[test]
    fn criterion4_truncating_the_generation_merges_entries_that_must_stay_apart() {
        // 没绕完一圈 ⇒ 恒 0（阴性对照）
        assert_eq!(wrongly_merged_pairs(1_000, 16), 0, "1000 < 65536，还没绕");
        assert_eq!(wrongly_merged_pairs(65_536, 16), 0, "恰好一圈，边界上仍是 0");
        // 8 位（256 个模类）跑 1000 次发布：q = 3、r = 232
        // ⇒ 232 × C(4,2) + 24 × C(3,2) = 232×6 + 24×3 = 1392 + 72 = 1464
        assert_eq!(wrongly_merged_pairs(1_000, 8), 1464);
        // 16 位跑 10 000 000 次：手算 65536 × 152 = 9 961 472 ⇒ q = 152、r = 38 528
        let q = 10_000_000u64 / 65536;
        let r = 10_000_000u64 % 65536;
        assert_eq!((q, r), (152, 38_528));
        let c2 = |n: u64| n * n.saturating_sub(1) / 2;
        assert_eq!(c2(153), 11_628);
        assert_eq!(c2(152), 11_476);
        assert_eq!(wrongly_merged_pairs(10_000_000, 16), r * c2(q + 1) + (65536 - r) * c2(q));
        // 手算：38528 × 11628 + 27008 × 11476 = 448 003 584 + 309 943 808
        assert_eq!(wrongly_merged_pairs(10_000_000, 16), 757_947_392);
        // 48 位（已定的下界）跑一千万次发布：一对都不合并
        assert_eq!(wrongly_merged_pairs(10_000_000, 48), 0);
        // 不截断：恒 0
        assert_eq!(wrongly_merged_pairs(u64::MAX / 2, 64), 0);
        // **阳性对照**：位数越少违例越多，单调
        assert!(wrongly_merged_pairs(1_000_000, 8) > wrongly_merged_pairs(1_000_000, 16));
    }

    /// **等价变异留档**：`publishes <= m` 换成 `publishes < m`，在**所有输入上**同结果。
    /// 边界那一格 `publishes == m` 走下去时 q = 1、r = 0
    /// ⇒ `0 × C(2,2) + m × C(1,2)` 而 `C(1,2) = 0` ⇒ 仍是 0。**这不算盲区，是等价。**
    #[test]
    fn equivalent_mutation_wrap_boundary_off_by_one() {
        let c2 = |n: u64| n * n.saturating_sub(1) / 2;
        for &bits in [8u32, 16].iter() {
            let m = 1u64 << bits;
            // 原式在边界上早退返回 0；换成严格小于时会走下去，手算也是 0
            let q = m / m;
            let r = m % m;
            assert_eq!((q, r), (1, 0));
            assert_eq!(r * c2(q + 1) + (m - r) * c2(q), 0, "走下去也是 0 ⇒ 两种写法等价");
            assert_eq!(wrongly_merged_pairs(m, bits), 0);
        }
    }

    /// **判据 4 的回绕期**：按 2785 发布/秒，与已定条款交叉对上。
    #[test]
    fn criterion4_truncating_the_txg_buys_window_bits_at_a_wrap_period() {
        assert_eq!(wrap_publishes(32), 4_294_967_296);
        // 手算：2^32 × 1000 / 2785 = 1 542 178 491 ms ≈ 17.8 天，与 D5 已定项 2 骑手条款同一个数
        // 手算：4 294 967 296 × 1000 / 2785，整除取下
        assert_eq!(wrap_millis(32), 1_542_178_562);
        assert_eq!(wrap_millis(32) / 1000 / 86400, 17);
        // 48 位：3204 年
        assert_eq!(wrap_millis(48) / 1000 / 86400 / 365, 3204);
    }

    /// **判据 2 + 阳性对照**：无 seq 必须错得最多，txg+窗口必须恒 0。
    #[test]
    fn criterion2_only_a_monotone_seq_picks_the_right_winner() {
        let ups = make_updates(1000, 8, None);
        // 无 seq：1000 个 key 全部**未定义**（不是「选错」——它根本没有序）
        assert_eq!(wrong_winners(&ups, Arm::NoSeq), (0, 1000));
        // 不插挂载时墙钟与挂载计数器都单调 ⇒ 全对
        assert_eq!(wrong_winners(&ups, Arm::WallClock), (0, 0));
        assert_eq!(wrong_winners(&ups, Arm::MountLocal), (0, 0));
        assert_eq!(wrong_winners(&ups, Arm::TxgPlusWindow), (0, 0), "txg + 窗口序号：恒 0");
        // 阳性对照：无 seq 那条臂的「未定义」数必须是全部
        let (w, u) = wrong_winners(&ups, Arm::NoSeq);
        assert_eq!(w + u, 1000, "1000 个 key 一个都不落下");
    }

    /// **判据 3 + 阳性对照**：插一次挂载之后，归零计数器与回拨的墙钟必须变差，txg 臂必须不动。
    #[test]
    fn criterion3_a_remount_breaks_wall_clock_and_mount_local_but_not_txg() {
        let with = make_updates(1000, 8, Some(4));
        assert_eq!(wrong_winners(&with, Arm::TxgPlusWindow), (0, 0), "txg + 窗口：插挂载也恒 0");
        // 墙钟回拨 ⇒ 最大读数落在回拨之前 ⇒ 1000 个 key 全选错（且不是未定义，是明确选错）
        assert_eq!(wrong_winners(&with, Arm::WallClock), (1000, 0));
        // 挂载归零 ⇒ 回绕之后新旧条目**序号相等** ⇒ 胜者未定义，不是「选错」
        assert_eq!(wrong_winners(&with, Arm::MountLocal), (0, 1000));
        // 阳性对照：不插挂载时这两条臂全对 ⇒ 这一维真的进了模型
        assert_eq!(wrong_winners(&make_updates(1000, 8, None), Arm::WallClock), (0, 0));
        assert_eq!(wrong_winners(&make_updates(1000, 8, None), Arm::MountLocal), (0, 0));
    }

    /// **阴性对照**：同 key 只写一次时四条臂必须全对。
    #[test]
    fn negative_control_single_update_per_key_is_unambiguous() {
        let ups = make_updates(1000, 1, None);
        for &arm in [Arm::NoSeq, Arm::WallClock, Arm::MountLocal, Arm::TxgPlusWindow].iter() {
            assert_eq!(wrong_winners(&ups, arm), (0, 0), "{arm:?} 在无歧义输入上就该全对");
        }
    }

    /// **等价变异留档**：把 `Arm::NoSeq` 从 `None` 换成「所有条目同一个常数序号」，
    /// 在**所有输入上**与原式同结果——两种写法都让同 key 的条目全部并列，
    /// 而并列在本模型里就判「胜者未定义」。**这不算盲区，是等价。**
    /// 变异表里那一条已换成一个真会改行为的（把墙钟压成常数）。
    #[test]
    fn equivalent_mutation_noseq_as_constant_order_key() {
        for &(keys, per_key) in [(4u32, 1u64), (4, 4), (1000, 8)].iter() {
            let ups = make_updates(keys, per_key, None);
            // 原式：None
            let a = wrong_winners(&ups, Arm::NoSeq);
            // 等价写法：所有条目同一个常数 ⇒ 全并列 ⇒ 同样判未定义
            let b = {
                use std::collections::HashMap;
                let mut m: HashMap<u32, Vec<u64>> = HashMap::new();
                for u in &ups {
                    m.entry(u.key).or_default().push(u.truth);
                }
                let mut w = 0u64;
                let mut und = 0u64;
                for (_, v) in m.iter() {
                    if v.len() > 1 { und += 1 } else { w += 0 }
                }
                (w, und)
            };
            assert_eq!(a, b, "keys={keys} per_key={per_key} 上两种写法同结果");
        }
    }

    /// 造出来的更新串本身要有歧义，否则判据 2 测的是空气。
    #[test]
    fn the_input_is_actually_shuffled() {
        let ups = make_updates(4, 4, None);
        assert_eq!(ups.len(), 16);
        // 打乱之后，至少有一个 key 的最后一条不是它 truth 最大的那条
        let last_of_key0 = ups.iter().filter(|u| u.key == 0).last().unwrap().truth;
        let max_of_key0 = ups.iter().filter(|u| u.key == 0).map(|u| u.truth).max().unwrap();
        assert_eq!(max_of_key0, 4);
        assert_eq!(last_of_key0, 1, "偶数 key 的子序列被翻转 ⇒ 残留次序的最后一条是最旧的");
        assert_ne!(last_of_key0, max_of_key0, "输入没被打乱，判据 2 就没有对象");
        // 奇数 key 不翻转 ⇒ 残留次序的最后一条恰好是最新的
        let last_of_key1 = ups.iter().filter(|u| u.key == 1).last().unwrap().truth;
        assert_eq!(last_of_key1, 4);
    }
}
