//! E43：扩展点字节上限 —— D21（权威态与派生态的分界）未定项 2。
//!
//! ## 它答什么
//!
//! D21 已定「扩展点大小按线在超级块里声明，上限取固定字节数」，
//! 但**上限取多少未定案**——正文那个 128 是举例。
//!
//! 本实验不问「128 好不好」（那样只可能产出支持性证据），而是**三段夹一个区间**：
//!
//! | 段 | 给哪一侧 | 判据 |
//! |---|---|---|
//! | 空间对照 | 上界 | N 使 D21 正文「128 KiB 单元上比 ZFS 省、比 btrfs 省一半」不再成立 |
//! | 扇出与树高 | 上界 | N 不得使任一节点档的树高比 N=0 时多一层 |
//! | 放得下一个合法指向 | **下界** | 小于它 ⇒ 扩展点放不下任何指向 ⇒ 该功能等于不存在 |
//!
//! ## 两种记法都要报
//!
//! 扩展点的字节算不算「为管理一个单元而多付的字节」，D21 的对照表没说。
//! **计入**⇒ 段一给出上界；**不计入**（记为开线方净荷）⇒ 段一不给上界，约束落到段二。
//! 挑一种报等于替 D21 定了口径，所以两种都算、都发。
//!
//! ## 两档几何都要跑
//!
//! D21 正文只说「单元预留一段」，**没说索引节点这类单元带不带扩展点**。
//! 按 D20（承重面：单元的原子性与自包含）「单元不分元数据与数据」的字面读法是都带。
//! ⇒ `Carrier::AllUnits` 与 `Carrier::DataOnly` 两档都跑；结论不同就把这个岔路退回 D21。
//!
//! ## 常量的出处（2026-08-29 逐字现查 kb 正文）
//!
//! - `BASE_META_TENTHS = 1080`：D21「单元元数据的记账」表，本工程 2 副本档 108.0 B
//!   （单元头 55 + 指针头部 31 + 位置条目 ×2 共 22）
//! - `BASE_META_RAID_TENTHS = 1520`：同表 4+2 条带档 152.0 B
//! - `ZFS_TENTHS = 1280` / `BTRFS_4K_TENTHS = 825` / `BTRFS_128K_TENTHS = 2073`：同表对照行
//! - `NODE_HDR = 64`、`PTR_BYTES = 40`：E29（坏一个节点的爆炸半径）的扇出口径，
//!   本实验的 N=0 档必须逐字复现它的 49 / 100 / 408 / 1636
//! - `DEV_BYTES = 1`、`PBA_BYTES = 6`、`LEN_BYTES = 2`：D21「本工程那 108 字节的去向」
//!   逐字写的位置条目字段（dev 1 + 物理偏移 6）与指针头部的 extent 偏移 2。
//!   ⚠️ **出处是 D21 的字节分解，不是 D19 的字段表**——D19 已定「位置条目带 dev」，
//!   但它没有逐字写这两个字段的宽度。
//!   ⚠️ **那 6 个字节的计量单位 kb 里没写**（字节还是块）：按字节读上限是 256 TiB，
//!   按 4 KiB 块读是 2⁶² 字节。本实验的下界不依赖这个读法（两种读法下宽度都是 6），
//!   但**要往上抬这个宽度时会依赖**，见本实验的已知局限。
//!
//! **全整数运算**：占比用 0.001% 为单位的整数，不走浮点——
//! 复跑要逐字节一致，而浮点格式化是最常见的那种「今天不一样了」。

use e7_index_bench::Emitter;

// ── D21 对照表的常量（单位：0.1 字节，避免浮点）──
const BASE_META_TENTHS: u64 = 1080; // 本工程 2 副本档 108.0 B
const BASE_META_RAID_TENTHS: u64 = 1520; // 本工程 4+2 条带档 152.0 B
const ZFS_TENTHS: u64 = 1280; // ZFS blkptr 128.0 B
const BTRFS_4K_TENTHS: u64 = 825; // btrfs 4 KiB 单元 82.5 B
const BTRFS_128K_TENTHS: u64 = 2073; // btrfs 128 KiB 单元 207.3 B

// ── E29 的节点几何口径 ──
const NODE_HDR: u64 = 64;
const PTR_BYTES: u64 = 40;

// ── 位置条目字段（出处：D21「本工程那 108 字节的去向」）──
const DEV_BYTES: u64 = 1;
const PBA_BYTES: u64 = 6;
const LEN_BYTES: u64 = 2;
const CSUM_BYTES: u64 = 4; // 位置条目里的密文校验和

// ── 108 字节里，哪些住在单元自己身上 ──
// D4（校验和位置）已定「父节点存子节点校验和」⇒ **指针住在父节点里，不在被指的单元里**。
// ⇒ 单元自身的容器约束只扣得到单元头那 55 字节，扣不到指针那 53 字节。
const IN_UNIT_HDR: u64 = 55; // 单元头（自描述），住在单元里
const OUT_OF_UNIT_PTR: u64 = 53; // 指针头部 31 + 位置条目 ×2 共 22，住在父节点里

/// 扩展点长在哪些单元上。D21 没区分，所以两档都跑。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Carrier {
    /// 每个单元都带（D20「单元不分元数据与数据」的字面读法）
    AllUnits,
    /// 只有数据单元带，索引节点不带
    DataOnly,
}

/// 占比，单位 0.001%。`meta_tenths` 是 0.1 字节。
/// 四舍五入用整数做：加半个分母再整除。
fn pct_milli(meta_tenths: u64, unit: u64) -> u64 {
    (meta_tenths * 10_000 + unit / 2) / unit
}

/// 扇出。扩展点吃掉节点里本来能放指针的字节。
fn fanout(node_bytes: u64, ext: u64, carrier: Carrier) -> u64 {
    let ext = match carrier {
        Carrier::AllUnits => ext,
        Carrier::DataOnly => 0, // 索引节点不带 ⇒ 扇出不受影响
    };
    (node_bytes.saturating_sub(NODE_HDR + ext) / PTR_BYTES).max(2)
}

/// 装下 `leaves` 个叶子需要的层数。
fn height(node_bytes: u64, ext: u64, carrier: Carrier, leaves: u64) -> u32 {
    let f = fanout(node_bytes, ext, carrier);
    let mut h = 0u32;
    let mut cap = 1u64;
    while cap < leaves {
        cap = cap.saturating_mul(f);
        h += 1;
    }
    h
}

/// 让树高比 `ext=0` 多一层的最小 `ext`。取不到（该档永远不涨）返回 None。
fn height_threshold(node_bytes: u64, carrier: Carrier, leaves: u64) -> Option<u64> {
    let base = height(node_bytes, 0, carrier, leaves);
    // 上界取到「节点里一个指针都放不下」为止
    for ext in 0..=node_bytes {
        if height(node_bytes, ext, carrier, leaves) > base {
            return Some(ext);
        }
    }
    None
}

/// 段一的上界：D21「128 KiB 单元上比 ZFS 省」在计入口径下容得下的最大 `N`。
/// 「省」= 严格小于。
fn n_max_cheaper_than_zfs() -> u64 {
    let mut n = 0u64;
    while BASE_META_TENTHS + 10 * (n + 1) < ZFS_TENTHS {
        n += 1;
    }
    n
}

/// D21「比 btrfs 省一半」在计入口径下成不成立：`本工程 × 2 ≤ btrfs`。
fn btrfs_half_holds(n: u64) -> bool {
    (BASE_META_TENTHS + 10 * n) * 2 <= BTRFS_128K_TENTHS
}

/// 下界之一：只放得下「指到哪」。整单元的校验/认证已覆盖扩展点（D21 硬约束 1），
/// 所以这一档不再单独带校验和。
fn n_min_bare() -> u64 {
    DEV_BYTES + PBA_BYTES
}

/// 下界之二：还要让 checker 判得了「配额范围被记为已分配」（D21 硬约束 4）⇒ 必须带长度。
fn n_min_quota() -> u64 {
    n_min_bare() + LEN_BYTES
}

/// 下界之三：若扩展点里的指向要与**核心位置条目同等**完整性，就得把密文校验和一起带上。
/// D21「108 字节的去向」写的位置条目是 dev 1 + 物理偏移 6 + 密文校验和 4 = 11。
fn n_min_with_csum() -> u64 {
    n_min_bare() + CSUM_BYTES
}

/// 下界之四：两样都要。
fn n_min_full() -> u64 {
    n_min_bare() + CSUM_BYTES + LEN_BYTES
}

/// **结构上界**：扩展点大到把单元的净荷挤成 0，就不是「浪费」，是「放不下」。
/// 判据形态符合 `.claude/rules/fs-design.md`「能不能把『用错了』变成『挂不上』」——
/// 这个数在 mkfs / 挂载时就判得了，超了直接拒绝，而不是运行时才发现。
fn n_max_payload_positive(unit: u64) -> u64 {
    unit - IN_UNIT_HDR - 1 // 至少留 1 字节净荷
}

/// **结构上界（索引节点那一档）**：节点里至少还要放得下一个指针。
fn n_max_node_keeps_one_ptr(node: u64) -> u64 {
    node - NODE_HDR - PTR_BYTES
}

// ── 自证单元那一档（D20 推论三：根槽、journal 记录头）──
// 它们没有带校验和的父指针，原子宽度**等于运行时探测到的 `physical_block_size`**。
// ⇒ 扩展点在这一档的余量由**原子宽度**夹，不由「省不省」夹。
const JOURNAL_HDR: u64 = 84; // D23 已定项 4 逐字：头部字段合计 84 字节
const ROOT_SLOT_CANDIDATE: u64 = 256; // D22 未定项 2 的候选槽宽

/// 一个自证单元的头部落进一个原子单元之后，还剩多少字节。
/// D23 已定项 4 逐字：84 字节头「占 512 扇区的 16%，其后还余 428 字节」。
fn self_witness_room(hdr: u64, atomic: u64) -> u64 {
    atomic - hdr
}

/// 一个原子单元里挤进几个槽。E34 主张一：槽宽 256、原子宽度 512 ⇒ **2 个**，
/// 于是写一个槽可能撕裂邻槽。撕裂隔离要求这个数为 1。
fn slots_per_atomic(slot: u64, atomic: u64) -> u64 {
    (atomic / slot).max(1)
}

/// 挂载时求值一次的几何判定。**返回「挂得上 / 挂不上」，不返回「慢一点」**——
/// 这是 `.claude/rules/fs-design.md`「能不能把『用错了』变成『挂不上』」在本项上的形态。
/// 输入全部是**操作期间不会变的量**：超级块里声明的 N、格式里的单元与节点大小、
/// 运行时探测到的 `physical_block_size`。⇒ 挂载时算一次，运行时只查不算。
fn mount_verdict(n: u64, unit: u64, node: u64, slot: u64, atomic: u64) -> &'static str {
    if n > n_max_payload_positive(unit) {
        return "reject_no_payload"; // 净荷被挤成 0
    }
    if n > n_max_node_keeps_one_ptr(node) {
        return "reject_no_pointer"; // 节点里放不下一个指针
    }
    if slots_per_atomic(slot, atomic) != 1 {
        return "reject_slot_shares_atomic"; // 撕裂隔离失效：一个原子单元里不止一个槽
    }
    if n > self_witness_room(JOURNAL_HDR, atomic) {
        return "reject_self_witness_overflow"; // 自证单元的头顶不住一个原子宽度
    }
    "ok"
}

/// `LEN_BYTES` 个字节按**字节**计量时表达得了的最大范围。
/// 2 字节 ⇒ 65535 < 131072：**装不下一个 128 KiB 单元**。
fn len_field_max_bytes() -> u64 {
    (1u64 << (8 * LEN_BYTES)) - 1
}

fn carrier_tag(c: Carrier) -> &'static str {
    match c {
        Carrier::AllUnits => "all_units",
        Carrier::DataOnly => "data_only",
    }
}

const UNITS: [u64; 4] = [4096, 16384, 65536, 131072];
const NODES: [u64; 4] = [2048, 4096, 16384, 65536];
const NS: [u64; 7] = [0, 16, 32, 64, 128, 256, 512];

fn main() {
    let mut em = Emitter::new();
    let leaves: u64 = 1 << 24; // 1600 万叶，与 E29 同口径

    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config base_meta_tenths={BASE_META_TENTHS} base_meta_raid_tenths={BASE_META_RAID_TENTHS} \
             zfs_tenths={ZFS_TENTHS} btrfs_4k_tenths={BTRFS_4K_TENTHS} btrfs_128k_tenths={BTRFS_128K_TENTHS} \
             node_hdr={NODE_HDR} ptr_bytes={PTR_BYTES} leaves={leaves} \
             in_unit_hdr={IN_UNIT_HDR} out_of_unit_ptr={OUT_OF_UNIT_PTR}"
        ))
    );

    // ── 段一：空间对照。两种记法都发。 ──
    for u in UNITS {
        for n in NS {
            let counted = BASE_META_TENTHS + 10 * n;
            let btrfs = match u {
                4096 => format!("{}", BTRFS_4K_TENTHS),
                131072 => format!("{}", BTRFS_128K_TENTHS),
                _ => "NA".to_string(), // D21 只列了 4 KiB 与 128 KiB 两档，别处不许编
            };
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=space unit={u} n={n} counted_tenths={counted} counted_pct_milli={} \
                     uncounted_tenths={BASE_META_TENTHS} uncounted_pct_milli={} \
                     raid_counted_tenths={} raid_counted_pct_milli={} \
                     zfs_tenths={ZFS_TENTHS} btrfs_tenths={btrfs} cheaper_than_zfs={} \
                     raid_cheaper_than_zfs={}",
                    pct_milli(counted, u),
                    pct_milli(BASE_META_TENTHS, u),
                    BASE_META_RAID_TENTHS + 10 * n,
                    pct_milli(BASE_META_RAID_TENTHS + 10 * n, u),
                    u8::from(counted < ZFS_TENTHS),
                    u8::from(BASE_META_RAID_TENTHS + 10 * n < ZFS_TENTHS),
                ))
            );
        }
    }

    // ── 段二：扇出与树高，两档载体都跑。 ──
    for c in [Carrier::AllUnits, Carrier::DataOnly] {
        for nb in NODES {
            for n in NS {
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=geom carrier={} node={nb} n={n} fanout={} height={} height_delta={}",
                        carrier_tag(c),
                        fanout(nb, n, c),
                        height(nb, n, c, leaves),
                        height(nb, n, c, leaves) - height(nb, 0, c, leaves),
                    ))
                );
            }
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=geom_threshold carrier={} node={nb} height_grows_at={}",
                    carrier_tag(c),
                    height_threshold(nb, c, leaves)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "never".into()),
                ))
            );
        }
    }

    // ── 段三与汇总 ──
    let lo = n_min_quota();
    let hi_zfs = n_max_cheaper_than_zfs();
    let hi_geom = NODES
        .iter()
        .filter_map(|&nb| height_threshold(nb, Carrier::AllUnits, leaves))
        .min()
        .map(|t| t - 1);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=bounds n_min_bare={} n_min_quota={lo} n_min_with_csum={} n_min_full={} \
             len_field_max_bytes={} len2_covers_128k={} \
             n_max_counted_zfs={hi_zfs} raid_cheaper_than_zfs_at_zero={} \
             n_max_payload_4k={} n_max_payload_128k={} n_max_node_2k={} n_max_node_64k={} \
             btrfs_half_holds_at_zero={} n_max_geom_all_units={} n_max_geom_data_only=unbounded",
            n_min_bare(),
            n_min_with_csum(),
            n_min_full(),
            len_field_max_bytes(),
            u8::from(len_field_max_bytes() >= 131072),
            u8::from(BASE_META_RAID_TENTHS < ZFS_TENTHS),
            n_max_payload_positive(4096),
            n_max_payload_positive(131072),
            n_max_node_keeps_one_ptr(2048),
            n_max_node_keeps_one_ptr(65536),
            u8::from(btrfs_half_holds(0)),
            hi_geom.map(|v| v.to_string()).unwrap_or_else(|| "unbounded".into()),
        ))
    );
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=interval counted_lo={lo} counted_hi={hi_zfs} counted_contains_128={} \
             uncounted_lo={lo} uncounted_hi={} uncounted_contains_128={}",
            u8::from(lo <= 128 && 128 <= hi_zfs),
            hi_geom.map(|v| v.to_string()).unwrap_or_else(|| "unbounded".into()),
            u8::from(hi_geom.is_none_or(|h| lo <= 128 && 128 <= h)),
        ))
    );

    // ── 自证单元档：扩展点在这一档的余量由原子宽度夹 ──
    // journal 记录头：它独占一个原子单元，余量 = 原子宽度 − 头长度。「一个原子单元几个槽」对它没有意义。
    for atomic in [512u64, 4096] {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=self_witness kind=journal_record hdr={JOURNAL_HDR} atomic={atomic} \
                 room={} slots_per_atomic=NA",
                self_witness_room(JOURNAL_HDR, atomic),
            ))
        );
    }
    // 根槽：它按槽轮换，余量受槽宽夹（槽内固定字段 kb 未写，这里给的是上界估计）；
    // 「一个原子单元几个槽」对它才是判据。
    for atomic in [512u64, 4096] {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=self_witness kind=root_slot slot={ROOT_SLOT_CANDIDATE} atomic={atomic} \
                 room_upper={} slots_per_atomic={}",
                ROOT_SLOT_CANDIDATE - 1,
                slots_per_atomic(ROOT_SLOT_CANDIDATE, atomic),
            ))
        );
    }
    // 阳性对照：把槽宽抬到原子宽度，撕裂隔离必须恢复成 1
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_slot slot=512 atomic=512 slots_per_atomic={} \
             slot=256_atomic=512_slots={}",
            slots_per_atomic(512, 512),
            slots_per_atomic(256, 512),
        ))
    );
    // 若自证单元也带扩展点，最紧的那条上界
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=self_witness_bound n_max_journal_512={} n_max_root_slot_256={} \
             n_max_if_self_witness_carries={}",
            self_witness_room(JOURNAL_HDR, 512),
            ROOT_SLOT_CANDIDATE - 1,
            self_witness_room(JOURNAL_HDR, 512).min(ROOT_SLOT_CANDIDATE - 1),
        ))
    );

    // ── 挂载时求值：同一份声明在不同设备上的判定 ──
    // 关键在于同一个 (N, 槽宽) 在 512 与 4096 两种探测结果下会不会给出不同判定。
    for atomic in [512u64, 4096] {
        for slot in [256u64, 512, 4096] {
            for n in [0u64, 128, 512] {
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=mount_eval atomic={atomic} slot={slot} n={n} unit=4096 node=4096 verdict={}",
                        mount_verdict(n, 4096, 4096, slot, atomic),
                    ))
                );
            }
        }
    }

    // ── 阳性对照：四个单元档、四个节点档各跑一次，不是只跑第一档 ──
    for u in UNITS {
        let n = u / 2;
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=poscontrol_space unit={u} n={n} pct_milli_at_zero={} pct_milli_at_half={}",
                pct_milli(BASE_META_TENTHS, u),
                pct_milli(BASE_META_TENTHS + 10 * n, u),
            ))
        );
    }
    for nb in NODES {
        let n = nb / 2;
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=poscontrol_geom node={nb} n={n} fanout_at_zero={} fanout_at_half={} \
                 fanout_data_only_at_half={}",
                fanout(nb, 0, Carrier::AllUnits),
                fanout(nb, n, Carrier::AllUnits),
                fanout(nb, n, Carrier::DataOnly),
            ))
        );
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const L: u64 = 1 << 24;

    // ────────── 绝对值断言：把模型钉在已入库的数上 ──────────
    // 这些不是臂间互比。只互比的话，四条臂一起错（例如把 BASE_META 打错）照样相等。

    /// 本工程那两行必须逐字复现 D21「单元元数据的记账」表。
    #[test]
    fn our_rows_reproduce_the_d21_table() {
        assert_eq!(pct_milli(BASE_META_TENTHS, 4096), 2637); // 108.0 B → 2.637%
        assert_eq!(pct_milli(BASE_META_TENTHS, 131072), 82); // 108.0 B → 0.082%
        assert_eq!(pct_milli(BASE_META_RAID_TENTHS, 4096), 3711); // 152.0 B → 3.711%
        assert_eq!(pct_milli(BASE_META_RAID_TENTHS, 131072), 116); // 152.0 B → 0.116%
    }

    /// 对照行也必须复现——它们是判据的输入，错了整个上界跟着错。
    #[test]
    fn reference_rows_reproduce_the_d21_table() {
        assert_eq!(pct_milli(ZFS_TENTHS, 4096), 3125); // ZFS 128.0 B → 3.125%
        assert_eq!(pct_milli(ZFS_TENTHS, 131072), 98); // ZFS 128.0 B → 0.098%
        assert_eq!(pct_milli(BTRFS_4K_TENTHS, 4096), 2014); // btrfs 82.5 B → 2.014%
        assert_eq!(pct_milli(BTRFS_128K_TENTHS, 131072), 158); // btrfs 207.3 B → 0.158%
    }

    /// N=0 的几何必须逐字复现 E29 的扇出与树高。
    #[test]
    fn zero_ext_reproduces_e30_geometry() {
        let got: Vec<u64> = NODES.iter().map(|&nb| fanout(nb, 0, Carrier::AllUnits)).collect();
        assert_eq!(got, vec![49, 100, 408, 1636]);
        let h: Vec<u32> = NODES
            .iter()
            .map(|&nb| height(nb, 0, Carrier::AllUnits, L))
            .collect();
        assert_eq!(h, vec![5, 4, 3, 3]);
    }

    /// 段一的上界：计入口径下「比 ZFS 省」只容得下 19 字节。
    /// 独立算术：108 + N < 128 ⇒ N < 20 ⇒ N_max = 19。
    #[test]
    fn counted_upper_bound_from_the_zfs_clause_is_19() {
        assert_eq!(n_max_cheaper_than_zfs(), 19);
        assert!(BASE_META_TENTHS + 10 * 19 < ZFS_TENTHS);
        assert!(BASE_META_TENTHS + 10 * 20 >= ZFS_TENTHS);
    }

    /// **D21 那半句按字面读在 N=0 就不成立**：108 × 2 = 216 > 207.3。
    /// 这条不是模型坏了，是被测的那句话该改写——所以它要有自己的断言，不能只在正文里提一句。
    #[test]
    fn the_btrfs_half_clause_already_fails_at_zero() {
        assert!(!btrfs_half_holds(0));
        // 实际比值 108 / 207.3 = 52.1%，不是 50%（千分之一为单位，四舍五入）
        let ratio_permille =
            (BASE_META_TENTHS * 1000 + BTRFS_128K_TENTHS / 2) / BTRFS_128K_TENTHS;
        assert_eq!(ratio_permille, 521);
    }

    /// **「比 ZFS 省」在 D21 自己的表里就只对 2 副本档成立**：
    /// 4+2 条带档 152.0 B 在 N=0 就已经超过 ZFS 的 128.0 B。
    /// ⇒ 计入口径下的上界 A 要按最费的那一档算，而那一档的上界是**负的**（该句对它从不成立）。
    #[test]
    fn the_stripe_row_is_already_dearer_than_zfs_at_zero() {
        assert!(BASE_META_RAID_TENTHS > ZFS_TENTHS);
        // 写成加法而不是减法：常量相减在变异下会**编译期溢出**，
        // 那时变异被记成「无效」而不是「被抓到」——一条本该会红的断言就静默失效了。
        assert_eq!(BASE_META_RAID_TENTHS, ZFS_TENTHS + 240); // 超出 24.0 字节
        // ⇒ 上界 A 对条带档在 N=0 就不存在，它只对 2 副本档成立
        assert_eq!(BASE_META_TENTHS + 200, ZFS_TENTHS); // 2 副本档还有 20.0 字节余量
    }

    /// **108 字节里只有 55 住在单元里**——D4（校验和位置）已定父节点存子节点校验和，
    /// 指针那 53 字节住在父节点。⇒ 容器约束扣的是 55，不是 108。
    #[test]
    fn only_the_unit_header_lives_inside_the_unit() {
        assert_eq!(IN_UNIT_HDR + OUT_OF_UNIT_PTR, 108);
        assert_eq!(IN_UNIT_HDR * 10, 550);
        assert_eq!(IN_UNIT_HDR + OUT_OF_UNIT_PTR, BASE_META_TENTHS / 10);
    }

    /// **结构上界**：把净荷挤成 0 才是「放不下」。这是唯一一条能把「用错了」变成「挂不上」的上界。
    /// 独立算术：4096 − 55 − 1 = 4040；131072 − 55 − 1 = 131016；
    /// 索引节点 2048 − 64 − 40 = 1944；65536 − 64 − 40 = 65432。
    #[test]
    fn structural_upper_bounds_are_pinned() {
        assert_eq!(n_max_payload_positive(4096), 4040);
        assert_eq!(n_max_payload_positive(131072), 131016);
        assert_eq!(n_max_node_keeps_one_ptr(2048), 1944);
        assert_eq!(n_max_node_keeps_one_ptr(65536), 65432);
    }

    /// **自证单元那一档的余量由原子宽度夹**，绝对值钉在 D23 逐字写下的那个数上：
    /// 84 字节头落进 512 扇区之后余 428。
    #[test]
    fn self_witness_room_matches_the_d23_number() {
        assert_eq!(self_witness_room(JOURNAL_HDR, 512), 428);
        assert_eq!(self_witness_room(JOURNAL_HDR, 4096), 4012);
    }

    /// **撕裂隔离**：E34 主张一——槽宽 256、原子宽度 512 ⇒ 一个原子单元里挤 2 个槽。
    /// 阳性对照：槽宽抬到 512 ⇒ 降到 1。测不出这个差别说明模型没在按原子单元算。
    #[test]
    fn two_256_byte_slots_share_one_512_byte_atomic_unit() {
        assert_eq!(slots_per_atomic(256, 512), 2);
        assert_eq!(slots_per_atomic(512, 512), 1); // 阳性对照
        assert_eq!(slots_per_atomic(256, 4096), 16);
    }

    /// **若自证单元也带扩展点，上界是 255，不是 864。**
    /// 独立算术：min(512 − 84, 256 − 1) = min(428, 255) = 255。
    #[test]
    fn the_self_witness_bound_is_255_not_864() {
        let bound = self_witness_room(JOURNAL_HDR, 512).min(ROOT_SLOT_CANDIDATE - 1);
        assert_eq!(bound, 255);
        assert!(bound < 864); // 比索引节点那条紧 3.4 倍
    }

    /// **挂载时判定是可移植性的分水岭**：同一份声明（槽宽 512）在 512 字节原子宽度的设备上
    /// 挂得上，在 4Kn 设备上**挂不上**——因为 8 个槽会挤进一个原子单元，撕裂隔离失效。
    /// ⇒ 「挂载时算一次」的代价是：**镜像的可挂载性依赖它当前插在哪台机器上**。
    #[test]
    fn the_same_declaration_can_mount_here_and_be_rejected_there() {
        assert_eq!(mount_verdict(128, 4096, 4096, 512, 512), "ok");
        assert_eq!(
            mount_verdict(128, 4096, 4096, 512, 4096),
            "reject_slot_shares_atomic"
        );
        // 候选槽宽 256 在两种设备上都挂不上
        assert_eq!(
            mount_verdict(0, 4096, 4096, 256, 512),
            "reject_slot_shares_atomic"
        );
    }

    /// **每条拒绝理由都要够得到**——一条永远返回不了的分支等于没写。
    #[test]
    fn every_rejection_reason_is_reachable() {
        assert_eq!(mount_verdict(5000, 4096, 4096, 512, 512), "reject_no_payload");
        assert_eq!(mount_verdict(4020, 4096, 4096, 512, 512), "reject_no_pointer");
        assert_eq!(
            mount_verdict(0, 4096, 4096, 256, 512),
            "reject_slot_shares_atomic"
        );
        assert_eq!(
            mount_verdict(500, 8192, 8192, 512, 512),
            "reject_self_witness_overflow"
        );
        assert_eq!(mount_verdict(128, 4096, 4096, 512, 512), "ok");
    }

    /// 下界是一个**区间**，不是一个数：四档各自钉死。
    /// 取哪一档取决于两个 kb 里没写的口径——扩展点里的指向要不要带密文校验和、
    /// 配额范围的长度以什么为单位。
    #[test]
    fn lower_bound_is_a_range_from_seven_to_thirteen() {
        assert_eq!(n_min_bare(), 7); // dev 1 + 物理偏移 6
        assert_eq!(n_min_quota(), 9); // + 长度 2
        assert_eq!(n_min_with_csum(), 11); // + 密文校验和 4
        assert_eq!(n_min_full(), 13); // 两样都要
    }

    /// **长度字段按字节计量时 2 个字节装不下一个 128 KiB 单元**：65535 < 131072。
    /// ⇒ 「长度 2 字节」这个借来的宽度对配额范围不成立，下界那一档还要再抬或改计量单位。
    #[test]
    fn a_two_byte_length_cannot_span_a_128_kib_unit() {
        assert_eq!(len_field_max_bytes(), 65535);
        assert!(len_field_max_bytes() < 131072);
    }

    /// 段二的上界，四个节点档各自钉死。独立算术（叶 2²⁴、指针 40、头 64）：
    /// 2 KiB 要 f ≤ 27（27⁵ = 14 348 907 < 2²⁴）⇒ ext ≥ 1984 − 1119 = 865；
    /// 4 KiB 要 f ≤ 63（63⁴ = 15 752 961 < 2²⁴）⇒ ext ≥ 4032 − 2559 = 1473；
    /// 16 KiB / 64 KiB 要 f ≤ 255（255³ = 16 581 375 < 2²⁴）⇒ 6081 / 55233。
    #[test]
    fn height_thresholds_are_pinned_per_node_size() {
        let got: Vec<u64> = NODES
            .iter()
            .map(|&nb| height_threshold(nb, Carrier::AllUnits, L).unwrap())
            .collect();
        assert_eq!(got, vec![865, 1473, 6081, 55233]);
    }

    /// 段二对 `DataOnly` 那一档**永远不涨**——索引节点不带扩展点。
    /// 两档结论不同 ⇒ D21 那个岔路是承重的，不是措辞问题。
    #[test]
    fn data_only_carrier_never_grows_the_tree() {
        for nb in NODES {
            assert_eq!(height_threshold(nb, Carrier::DataOnly, L), None);
            assert_eq!(
                fanout(nb, 512, Carrier::DataOnly),
                fanout(nb, 0, Carrier::DataOnly)
            );
        }
    }

    // ────────── 阳性对照：每一条臂都跑，不是只跑第一条 ──────────

    /// 四个单元档各自把 N 抬到单元的一半，占比必须显著上升，且等于独立算出的值。
    /// 测不出上升 ⇒ 模型根本没把扩展点算进去，整轮作废。
    #[test]
    fn positive_control_every_unit_arm() {
        for u in UNITS {
            let base = pct_milli(BASE_META_TENTHS, u);
            let half = pct_milli(BASE_META_TENTHS + 10 * (u / 2), u);
            assert!(half > base, "单元 {u}：抬到一半之后占比没涨");
        }
        // 绝对值锚：4 KiB 单元、N = 2048 ⇒ (108 + 2048) / 4096 = 52.637%
        assert_eq!(pct_milli(BASE_META_TENTHS + 10 * 2048, 4096), 52637);
        // 128 KiB 单元、N = 65536 ⇒ (108 + 65536) / 131072 = 50.082%
        assert_eq!(pct_milli(BASE_META_TENTHS + 10 * 65536, 131072), 50082);
    }

    /// 四个节点档各自把 N 抬到节点的一半，扇出必须下降，且等于独立算出的值。
    #[test]
    fn positive_control_every_node_arm() {
        for nb in NODES {
            let base = fanout(nb, 0, Carrier::AllUnits);
            let half = fanout(nb, nb / 2, Carrier::AllUnits);
            assert!(half < base, "节点 {nb}：抬到一半之后扇出没降");
        }
        // 绝对值锚：4 KiB 节点、N = 2048 ⇒ (4096 − 64 − 2048) / 40 = 49
        assert_eq!(fanout(4096, 2048, Carrier::AllUnits), 49);
        // 64 KiB 节点、N = 32768 ⇒ (65536 − 64 − 32768) / 40 = 817
        assert_eq!(fanout(65536, 32768, Carrier::AllUnits), 817);
    }

    // ────────── 校验路径：换一条独立算法复现同一个判断 ──────────

    /// 占比的整数路径与浮点路径必须一致。两条路径不共享代码：
    /// 一条走「加半个分母再整除」，一条走 f64 除法 + `round`。
    #[test]
    fn integer_and_float_paths_agree_on_every_cell() {
        for u in UNITS {
            for n in NS {
                let meta = BASE_META_TENTHS + 10 * n;
                let f = (meta as f64 / 10.0) / u as f64 * 100.0;
                let via_float = (f * 1000.0).round() as u64;
                assert_eq!(pct_milli(meta, u), via_float, "单元 {u}、N {n} 两条路径不一致");
            }
        }
    }

    /// 校验路径自己要能红：喂一个已知错的分母，两条路径必须**分道扬镳**。
    /// 抓不到 ⇒ 上面那条一致是回声，不是证据。
    #[test]
    fn the_cross_check_itself_can_go_red() {
        let meta = BASE_META_TENTHS;
        let wrong = pct_milli(meta, 4096 * 2); // 把单元大小弄错一倍
        let right = ((meta as f64 / 10.0) / 4096.0 * 100.0 * 1000.0).round() as u64;
        assert_ne!(wrong, right, "把分母弄错一倍之后两条路径居然还一致，这条校验是摆设");
    }
}
