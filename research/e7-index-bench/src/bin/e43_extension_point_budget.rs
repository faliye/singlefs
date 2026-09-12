//! E43：扩展点字节上限 —— D21（权威态与派生态的分界）已定项 2。
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
//! - `BASE_METADATA_TENTHS = 1580`：D21「单元元数据的记账」表，本工程 2 副本档 158.0 B
//!   （2026-09-02 起单元头按 D18（块里携带什么信息）已定项 7 的 91 字节初值计，原 55 字节假设作废）
//!   （单元头 55 + 指针头部 31 + 位置条目 ×2 共 22）
//! - `BASE_METADATA_RAID_TENTHS = 2020`：同表 4+2 条带档 202.0 B
//! - `ZFS_TENTHS = 1280` / `BTRFS_4_KIBIBYTE_UNIT_TENTHS = 825` / `BTRFS_128_KIBIBYTE_UNIT_TENTHS = 2073`：同表对照行
//! - `NODE_HEADER_BYTES = 64`、`POINTER_BYTES = 40`：E29（坏一个节点的爆炸半径）的扇出口径，
//!   本实验的 N=0 档必须逐字复现它的 49 / 100 / 408 / 1636
//! - `DEVICE_FIELD_BYTES = 1`、`PHYSICAL_BLOCK_ADDRESS_BYTES = 6`、`LENGTH_FIELD_BYTES = 2`：D21「本工程那 108 字节的去向」
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
const BASE_METADATA_TENTHS: u64 = 1580; // 本工程 2 副本档 158.0 B（单元头 105 + 父节点侧 53）
const BASE_METADATA_RAID_TENTHS: u64 = 2020; // 本工程 4+2 条带档 202.0 B（单元头 105 + 指针头部 31 + 位置条目 ×6 共 66）
const ZFS_TENTHS: u64 = 1280; // ZFS blkptr 128.0 B
const BTRFS_4_KIBIBYTE_UNIT_TENTHS: u64 = 825; // btrfs 4 KiB 单元 82.5 B
const BTRFS_128_KIBIBYTE_UNIT_TENTHS: u64 = 2073; // btrfs 128 KiB 单元 207.3 B

// ── E29 的节点几何口径 ──
const NODE_HEADER_BYTES: u64 = 64;
const POINTER_BYTES: u64 = 40;

// ── 位置条目字段（出处：D21「本工程那 108 字节的去向」）──
const DEVICE_FIELD_BYTES: u64 = 1;
const PHYSICAL_BLOCK_ADDRESS_BYTES: u64 = 6;
const LENGTH_FIELD_BYTES: u64 = 2;
const CHECKSUM_BYTES: u64 = 4; // 位置条目里的密文校验和

// ── 158 字节里，哪些住在单元自己身上 ──
// D4（校验和位置）已定「父节点存子节点校验和」⇒ **指针住在父节点里，不在被指的单元里**。
// ⇒ 单元自身的容器约束只扣得到单元头那 105 字节，扣不到指针那 53 字节。
const IN_UNIT_HEADER_BYTES: u64 = 105; // 单元头（kb 里 `format-const: DATA_UNIT_HEADER_BYTES`，D18 已定项 7），住在单元里
const OUT_OF_UNIT_POINTER_BYTES: u64 = 53; // 指针头部 31 + 位置条目 ×2 共 22，住在父节点里

/// 扩展点长在哪些单元上。D21 没区分，所以两档都跑。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Carrier {
    /// 每个单元都带（D20「单元不分元数据与数据」的字面读法）
    AllUnits,
    /// 只有数据单元带，索引节点不带
    DataOnly,
}

/// 占比，单位 0.001%。`metadata_tenths` 是 0.1 字节。
/// 四舍五入用整数做：加半个分母再整除。
fn share_in_thousandths_of_a_percent(metadata_tenths: u64, unit: u64) -> u64 {
    (metadata_tenths * 10_000 + unit / 2) / unit
}

/// 扇出。扩展点吃掉节点里本来能放指针的字节。
fn fanout(node_bytes: u64, extension_point_bytes: u64, carrier: Carrier) -> u64 {
    let effective_extension_point_bytes = match carrier {
        Carrier::AllUnits => extension_point_bytes,
        Carrier::DataOnly => 0, // 索引节点不带 ⇒ 扇出不受影响
    };
    (node_bytes.saturating_sub(NODE_HEADER_BYTES + effective_extension_point_bytes) / POINTER_BYTES).max(2)
}

/// 装下 `leaves` 个叶子需要的层数。
fn height(node_bytes: u64, extension_point_bytes: u64, carrier: Carrier, leaves: u64) -> u32 {
    let node_fanout = fanout(node_bytes, extension_point_bytes, carrier);
    let mut levels = 0u32;
    let mut leaves_reachable = 1u64;
    while leaves_reachable < leaves {
        leaves_reachable = leaves_reachable.saturating_mul(node_fanout);
        levels += 1;
    }
    levels
}

/// 让树高比 `extension_point_bytes = 0` 多一层的最小 `extension_point_bytes`。取不到（该档永远不涨）返回 None。
fn height_threshold(node_bytes: u64, carrier: Carrier, leaves: u64) -> Option<u64> {
    let base_height = height(node_bytes, 0, carrier, leaves);
    // 上界取到「节点里一个指针都放不下」为止
    for extension_point_bytes in 0..=node_bytes {
        if height(node_bytes, extension_point_bytes, carrier, leaves) > base_height {
            return Some(extension_point_bytes);
        }
    }
    None
}

/// 段一的上界：D21「128 KiB 单元上比 ZFS 省」在计入口径下容得下的最大 `N`。
/// 「省」= 严格小于。
fn maximum_extension_point_bytes_cheaper_than_zfs() -> u64 {
    let mut extension_point_bytes = 0u64;
    while BASE_METADATA_TENTHS + 10 * (extension_point_bytes + 1) < ZFS_TENTHS {
        extension_point_bytes += 1;
    }
    extension_point_bytes
}

/// D21「比 btrfs 省一半」在计入口径下成不成立：`本工程 × 2 ≤ btrfs`。
fn btrfs_half_holds(extension_point_bytes: u64) -> bool {
    (BASE_METADATA_TENTHS + 10 * extension_point_bytes) * 2 <= BTRFS_128_KIBIBYTE_UNIT_TENTHS
}

/// 下界之一：只放得下「指到哪」。整单元的校验/认证已覆盖扩展点（D21 硬约束 1），
/// 所以这一档不再单独带校验和。
fn minimum_extension_point_bytes_bare() -> u64 {
    DEVICE_FIELD_BYTES + PHYSICAL_BLOCK_ADDRESS_BYTES
}

/// 下界之二：还要让 checker 判得了「配额范围被记为已分配」（D21 硬约束 4）⇒ 必须带长度。
fn minimum_extension_point_bytes_with_quota_length() -> u64 {
    minimum_extension_point_bytes_bare() + LENGTH_FIELD_BYTES
}

/// 下界之三：若扩展点里的指向要与**核心位置条目同等**完整性，就得把密文校验和一起带上。
/// D21「108 字节的去向」写的位置条目是 dev 1 + 物理偏移 6 + 密文校验和 4 = 11。
fn minimum_extension_point_bytes_with_checksum() -> u64 {
    minimum_extension_point_bytes_bare() + CHECKSUM_BYTES
}

/// 下界之四：两样都要。
fn minimum_extension_point_bytes_full() -> u64 {
    minimum_extension_point_bytes_bare() + CHECKSUM_BYTES + LENGTH_FIELD_BYTES
}

/// **结构上界**：扩展点大到把单元的净荷挤成 0，就不是「浪费」，是「放不下」。
/// 判据形态符合 `.claude/rules/fs-design.md`「能不能把『用错了』变成『挂不上』」——
/// 这个数在 mkfs / 挂载时就判得了，超了直接拒绝，而不是运行时才发现。
fn maximum_extension_point_bytes_keeping_payload_positive(unit: u64) -> u64 {
    unit - IN_UNIT_HEADER_BYTES - 1 // 至少留 1 字节净荷
}

/// **结构上界（索引节点那一档）**：节点里至少还要放得下一个指针。
fn maximum_extension_point_bytes_keeping_one_pointer_in_node(node: u64) -> u64 {
    node - NODE_HEADER_BYTES - POINTER_BYTES
}

// ── 自证单元那一档（D20 推论三：根槽、journal 记录头）──
// 它们没有带校验和的父指针，原子宽度**等于运行时探测到的 `physical_block_size`**。
// ⇒ 扩展点在这一档的余量由**原子宽度**夹，不由「省不省」夹。
const JOURNAL_HEADER_BYTES: u64 = 78; // D23 已定项 4 逐字：头部字段合计 78 字节（tail_lsn 随已定项 3 去掉）
const ROOT_SLOT_CANDIDATE: u64 = 256; // D22 已定项 2 的候选槽宽

/// 一个自证单元的头部落进一个原子单元之后，还剩多少字节。
/// D23 已定项 4 逐字：84 字节头「占 512 扇区的 16%，其后还余 428 字节」。
fn self_witness_room(header_bytes: u64, atomic: u64) -> u64 {
    atomic - header_bytes
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
fn mount_verdict(extension_point_bytes: u64, unit: u64, node: u64, slot: u64, atomic: u64) -> &'static str {
    if extension_point_bytes > maximum_extension_point_bytes_keeping_payload_positive(unit) {
        return "reject_no_payload"; // 净荷被挤成 0
    }
    if extension_point_bytes > maximum_extension_point_bytes_keeping_one_pointer_in_node(node) {
        return "reject_no_pointer"; // 节点里放不下一个指针
    }
    if slots_per_atomic(slot, atomic) != 1 {
        return "reject_slot_shares_atomic"; // 撕裂隔离失效：一个原子单元里不止一个槽
    }
    if extension_point_bytes > self_witness_room(JOURNAL_HEADER_BYTES, atomic) {
        return "reject_self_witness_overflow"; // 自证单元的头顶不住一个原子宽度
    }
    "ok"
}

/// `LENGTH_FIELD_BYTES` 个字节按**字节**计量时表达得了的最大范围。
/// 2 字节 ⇒ 65535 < 131072：**装不下一个 128 KiB 单元**。
fn length_field_maximum_bytes() -> u64 {
    (1u64 << (8 * LENGTH_FIELD_BYTES)) - 1
}

fn carrier_tag(carrier: Carrier) -> &'static str {
    match carrier {
        Carrier::AllUnits => "all_units",
        Carrier::DataOnly => "data_only",
    }
}

const UNITS: [u64; 4] = [4096, 16384, 65536, 131072];
const NODES: [u64; 4] = [2048, 4096, 16384, 65536];
const EXTENSION_POINT_BYTES_SCANNED: [u64; 7] = [0, 16, 32, 64, 128, 256, 512];

fn main() {
    let mut emitter = Emitter::new();
    let leaves: u64 = 1 << 24; // 1600 万叶，与 E29 同口径

    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config base_meta_tenths={BASE_METADATA_TENTHS} base_meta_raid_tenths={BASE_METADATA_RAID_TENTHS} \
             zfs_tenths={ZFS_TENTHS} btrfs_4k_tenths={BTRFS_4_KIBIBYTE_UNIT_TENTHS} btrfs_128k_tenths={BTRFS_128_KIBIBYTE_UNIT_TENTHS} \
             node_hdr={NODE_HEADER_BYTES} ptr_bytes={POINTER_BYTES} leaves={leaves} \
             in_unit_hdr={IN_UNIT_HEADER_BYTES} out_of_unit_ptr={OUT_OF_UNIT_POINTER_BYTES}"
        ))
    );

    // ── 段一：空间对照。两种记法都发。 ──
    for unit_bytes in UNITS {
        for extension_point_bytes in EXTENSION_POINT_BYTES_SCANNED {
            let counted_tenths = BASE_METADATA_TENTHS + 10 * extension_point_bytes;
            let btrfs_tenths = match unit_bytes {
                4096 => format!("{}", BTRFS_4_KIBIBYTE_UNIT_TENTHS),
                131072 => format!("{}", BTRFS_128_KIBIBYTE_UNIT_TENTHS),
                _ => "NA".to_string(), // D21 只列了 4 KiB 与 128 KiB 两档，别处不许编
            };
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=space unit={unit_bytes} n={extension_point_bytes} counted_tenths={counted_tenths} counted_pct_milli={} \
                     uncounted_tenths={BASE_METADATA_TENTHS} uncounted_pct_milli={} \
                     raid_counted_tenths={} raid_counted_pct_milli={} \
                     zfs_tenths={ZFS_TENTHS} btrfs_tenths={btrfs_tenths} cheaper_than_zfs={} \
                     raid_cheaper_than_zfs={}",
                    share_in_thousandths_of_a_percent(counted_tenths, unit_bytes),
                    share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS, unit_bytes),
                    BASE_METADATA_RAID_TENTHS + 10 * extension_point_bytes,
                    share_in_thousandths_of_a_percent(BASE_METADATA_RAID_TENTHS + 10 * extension_point_bytes, unit_bytes),
                    u8::from(counted_tenths < ZFS_TENTHS),
                    u8::from(BASE_METADATA_RAID_TENTHS + 10 * extension_point_bytes < ZFS_TENTHS),
                ))
            );
        }
    }

    // ── 段二：扇出与树高，两档载体都跑。 ──
    for carrier in [Carrier::AllUnits, Carrier::DataOnly] {
        for node_bytes in NODES {
            for extension_point_bytes in EXTENSION_POINT_BYTES_SCANNED {
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=geom carrier={} node={node_bytes} n={extension_point_bytes} fanout={} height={} height_delta={}",
                        carrier_tag(carrier),
                        fanout(node_bytes, extension_point_bytes, carrier),
                        height(node_bytes, extension_point_bytes, carrier, leaves),
                        height(node_bytes, extension_point_bytes, carrier, leaves) - height(node_bytes, 0, carrier, leaves),
                    ))
                );
            }
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=geom_threshold carrier={} node={node_bytes} height_grows_at={}",
                    carrier_tag(carrier),
                    height_threshold(node_bytes, carrier, leaves)
                        .map(|threshold_bytes| threshold_bytes.to_string())
                        .unwrap_or_else(|| "never".into()),
                ))
            );
        }
    }

    // ── 段三与汇总 ──
    let lower_bound_bytes = minimum_extension_point_bytes_with_quota_length();
    let upper_bound_cheaper_than_zfs = maximum_extension_point_bytes_cheaper_than_zfs();
    let upper_bound_from_geometry = NODES
        .iter()
        .filter_map(|&node_bytes| height_threshold(node_bytes, Carrier::AllUnits, leaves))
        .min()
        .map(|threshold_bytes| threshold_bytes - 1);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=bounds n_min_bare={} n_min_quota={lower_bound_bytes} n_min_with_csum={} n_min_full={} \
             len_field_max_bytes={} len2_covers_128k={} \
             n_max_counted_zfs={upper_bound_cheaper_than_zfs} raid_cheaper_than_zfs_at_zero={} \
             n_max_payload_4k={} n_max_payload_128k={} n_max_node_2k={} n_max_node_64k={} \
             btrfs_half_holds_at_zero={} n_max_geom_all_units={} n_max_geom_data_only=unbounded",
            minimum_extension_point_bytes_bare(),
            minimum_extension_point_bytes_with_checksum(),
            minimum_extension_point_bytes_full(),
            length_field_maximum_bytes(),
            u8::from(length_field_maximum_bytes() >= 131072),
            u8::from(BASE_METADATA_RAID_TENTHS < ZFS_TENTHS),
            maximum_extension_point_bytes_keeping_payload_positive(4096),
            maximum_extension_point_bytes_keeping_payload_positive(131072),
            maximum_extension_point_bytes_keeping_one_pointer_in_node(2048),
            maximum_extension_point_bytes_keeping_one_pointer_in_node(65536),
            u8::from(btrfs_half_holds(0)),
            upper_bound_from_geometry.map(|upper_bound_bytes| upper_bound_bytes.to_string()).unwrap_or_else(|| "unbounded".into()),
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=interval counted_lo={lower_bound_bytes} counted_hi={upper_bound_cheaper_than_zfs} counted_contains_128={} \
             uncounted_lo={lower_bound_bytes} uncounted_hi={} uncounted_contains_128={}",
            u8::from(lower_bound_bytes <= 128 && 128 <= upper_bound_cheaper_than_zfs),
            upper_bound_from_geometry.map(|upper_bound_bytes| upper_bound_bytes.to_string()).unwrap_or_else(|| "unbounded".into()),
            u8::from(upper_bound_from_geometry.is_none_or(|upper_bound_bytes| lower_bound_bytes <= 128 && 128 <= upper_bound_bytes)),
        ))
    );

    // ── 自证单元档：扩展点在这一档的余量由原子宽度夹 ──
    // journal 记录头：它独占一个原子单元，余量 = 原子宽度 − 头长度。「一个原子单元几个槽」对它没有意义。
    for atomic in [512u64, 4096] {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=self_witness kind=journal_record hdr={JOURNAL_HEADER_BYTES} atomic={atomic} \
                 room={} slots_per_atomic=NA",
                self_witness_room(JOURNAL_HEADER_BYTES, atomic),
            ))
        );
    }
    // 根槽：它按槽轮换，余量受槽宽夹（槽内固定字段 kb 未写，这里给的是上界估计）；
    // 「一个原子单元几个槽」对它才是判据。
    for atomic in [512u64, 4096] {
        println!(
            "{}",
            emitter.emit_raw(&format!(
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
        emitter.emit_raw(&format!(
            "name=poscontrol_slot slot=512 atomic=512 slots_per_atomic={} \
             slot=256_atomic=512_slots={}",
            slots_per_atomic(512, 512),
            slots_per_atomic(256, 512),
        ))
    );
    // 若自证单元也带扩展点，最紧的那条上界
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=self_witness_bound n_max_journal_512={} n_max_root_slot_256={} \
             n_max_if_self_witness_carries={}",
            self_witness_room(JOURNAL_HEADER_BYTES, 512),
            ROOT_SLOT_CANDIDATE - 1,
            self_witness_room(JOURNAL_HEADER_BYTES, 512).min(ROOT_SLOT_CANDIDATE - 1),
        ))
    );

    // ── 挂载时求值：同一份声明在不同设备上的判定 ──
    // 关键在于同一个 (N, 槽宽) 在 512 与 4096 两种探测结果下会不会给出不同判定。
    for atomic in [512u64, 4096] {
        for slot in [256u64, 512, 4096] {
            for extension_point_bytes in [0u64, 128, 512] {
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=mount_eval atomic={atomic} slot={slot} n={extension_point_bytes} unit=4096 node=4096 verdict={}",
                        mount_verdict(extension_point_bytes, 4096, 4096, slot, atomic),
                    ))
                );
            }
        }
    }

    // ── 阳性对照：四个单元档、四个节点档各跑一次，不是只跑第一档 ──
    for unit_bytes in UNITS {
        let extension_point_bytes = unit_bytes / 2;
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=poscontrol_space unit={unit_bytes} n={extension_point_bytes} pct_milli_at_zero={} pct_milli_at_half={}",
                share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS, unit_bytes),
                share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS + 10 * extension_point_bytes, unit_bytes),
            ))
        );
    }
    for node_bytes in NODES {
        let extension_point_bytes = node_bytes / 2;
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=poscontrol_geom node={node_bytes} n={extension_point_bytes} fanout_at_zero={} fanout_at_half={} \
                 fanout_data_only_at_half={}",
                fanout(node_bytes, 0, Carrier::AllUnits),
                fanout(node_bytes, extension_point_bytes, Carrier::AllUnits),
                fanout(node_bytes, extension_point_bytes, Carrier::DataOnly),
            ))
        );
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const LEAF_COUNT: u64 = 1 << 24;

    // ────────── 绝对值断言：把模型钉在已入库的数上 ──────────
    // 这些不是臂间互比。只互比的话，四条臂一起错（例如把 BASE_META 打错）照样相等。

    /// 本工程那两行必须逐字复现 D21「单元元数据的记账」表。
    #[test]
    fn our_rows_reproduce_the_decision_21_table() {
        assert_eq!(share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS, 4096), 3857); // 158.0 B → 3.857%
        assert_eq!(share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS, 131072), 121); // 158.0 B → 0.121%
        assert_eq!(share_in_thousandths_of_a_percent(BASE_METADATA_RAID_TENTHS, 4096), 4932); // 202.0 B → 4.932%
        assert_eq!(share_in_thousandths_of_a_percent(BASE_METADATA_RAID_TENTHS, 131072), 154); // 202.0 B → 0.154%
    }

    /// 对照行也必须复现——它们是判据的输入，错了整个上界跟着错。
    #[test]
    fn reference_rows_reproduce_the_decision_21_table() {
        assert_eq!(share_in_thousandths_of_a_percent(ZFS_TENTHS, 4096), 3125); // ZFS 128.0 B → 3.125%
        assert_eq!(share_in_thousandths_of_a_percent(ZFS_TENTHS, 131072), 98); // ZFS 128.0 B → 0.098%
        assert_eq!(share_in_thousandths_of_a_percent(BTRFS_4_KIBIBYTE_UNIT_TENTHS, 4096), 2014); // btrfs 82.5 B → 2.014%
        assert_eq!(share_in_thousandths_of_a_percent(BTRFS_128_KIBIBYTE_UNIT_TENTHS, 131072), 158); // btrfs 207.3 B → 0.158%
    }

    /// N=0 的几何必须逐字复现 E29 的扇出与树高。
    #[test]
    fn zero_extension_point_reproduces_e30_geometry() {
        let got: Vec<u64> = NODES.iter().map(|&node_bytes| fanout(node_bytes, 0, Carrier::AllUnits)).collect();
        assert_eq!(got, vec![49, 100, 408, 1636]);
        let heights: Vec<u32> = NODES
            .iter()
            .map(|&node_bytes| height(node_bytes, 0, Carrier::AllUnits, LEAF_COUNT))
            .collect();
        assert_eq!(heights, vec![5, 4, 3, 3]);
    }

    /// 段一那条「比 ZFS 省」的计数：158 > 128，在 N = 0 就不成立 ⇒ 计数为 0。
    /// （上界 A 本来就已按「比谁省不是判据」删掉；这条钉死的是计算本身没有静默漂移。）
    #[test]
    fn counted_upper_bound_from_the_zfs_clause_is_zero_now() {
        assert_eq!(maximum_extension_point_bytes_cheaper_than_zfs(), 0);
        assert_eq!(BASE_METADATA_TENTHS, ZFS_TENTHS + 300); // 2 副本档比 ZFS 贵 30.0 字节
    }

    /// **D21 那半句按字面读在 N=0 就不成立**：158 × 2 = 316 > 207.3。
    /// 这条不是模型坏了，是被测的那句话该改写——所以它要有自己的断言，不能只在正文里提一句。
    #[test]
    fn the_btrfs_half_clause_already_fails_at_zero() {
        assert!(!btrfs_half_holds(0));
        // 实际比值 158 / 207.3 = 76.2%（千分之一为单位，向下取整）
        let ratio_permille =
            (BASE_METADATA_TENTHS * 1000 + BTRFS_128_KIBIBYTE_UNIT_TENTHS / 2) / BTRFS_128_KIBIBYTE_UNIT_TENTHS;
        assert_eq!(ratio_permille, 762);
    }

    /// **105 字节头之后两档都比 ZFS 贵**（2026-09-05 C113 定案加写序 10 与载荷 CRC 4 起）：
    /// 2 副本档 158.0、条带档 202.0，都超过 ZFS 的 128.0——
    /// 「比谁省」本来就不是判据（D21 2026-08-30 用户定案），这里只钉数字没有漂。
    #[test]
    fn both_rows_are_dearer_than_zfs_at_zero() {
        assert!(BASE_METADATA_RAID_TENTHS > ZFS_TENTHS);
        // 写成加法而不是减法：常量相减在变异下会**编译期溢出**，
        // 那时变异被记成「无效」而不是「被抓到」——一条本该会红的断言就静默失效了。
        assert_eq!(BASE_METADATA_RAID_TENTHS, ZFS_TENTHS + 740); // 条带档超出 74.0 字节
        assert_eq!(BASE_METADATA_TENTHS, ZFS_TENTHS + 300); // 2 副本档超出 30.0 字节
    }

    /// **158 字节里只有 105 住在单元里**——D4（校验和位置）已定父节点存子节点校验和，
    /// 指针那 53 字节住在父节点。⇒ 容器约束扣的是 105，不是 158。
    #[test]
    fn only_the_unit_header_lives_inside_the_unit() {
        assert_eq!(IN_UNIT_HEADER_BYTES + OUT_OF_UNIT_POINTER_BYTES, 158);
        assert_eq!(IN_UNIT_HEADER_BYTES * 10, 1050);
        assert_eq!(IN_UNIT_HEADER_BYTES + OUT_OF_UNIT_POINTER_BYTES, BASE_METADATA_TENTHS / 10);
    }

    /// **结构上界**：把净荷挤成 0 才是「放不下」。这是唯一一条能把「用错了」变成「挂不上」的上界。
    /// 独立算术：4096 − 105 − 1 = 3990；131072 − 105 − 1 = 130966；
    /// 索引节点 2048 − 64 − 40 = 1944；65536 − 64 − 40 = 65432。
    #[test]
    fn structural_upper_bounds_are_pinned() {
        assert_eq!(maximum_extension_point_bytes_keeping_payload_positive(4096), 3990);
        assert_eq!(maximum_extension_point_bytes_keeping_payload_positive(131072), 130966);
        assert_eq!(maximum_extension_point_bytes_keeping_one_pointer_in_node(2048), 1944);
        assert_eq!(maximum_extension_point_bytes_keeping_one_pointer_in_node(65536), 65432);
    }

    /// **自证单元那一档的余量由原子宽度夹**，绝对值钉在 D23 逐字写下的那个数上：
    /// 78 字节头落进 512 扇区之后余 434。
    #[test]
    fn self_witness_room_matches_the_decision_23_number() {
        assert_eq!(self_witness_room(JOURNAL_HEADER_BYTES, 512), 434);
        assert_eq!(self_witness_room(JOURNAL_HEADER_BYTES, 4096), 4018);
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
    /// 独立算术：min(512 − 78, 256 − 1) = min(434, 255) = 255。
    #[test]
    fn the_self_witness_bound_is_255_not_864() {
        let bound = self_witness_room(JOURNAL_HEADER_BYTES, 512).min(ROOT_SLOT_CANDIDATE - 1);
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
    ///
    /// ⚠️ **单元头 91 → 105 之后，4 KiB 那一格的两个上界换了次序**：净荷上界
    /// 4096 − 105 − 1 = 3990 比指针上界 4096 − 64 − 40 = 3992 紧 2 字节
    /// ⇒ `unit == node == 4096` 那一格再也够不到 `reject_no_pointer`，
    /// 要用「单元大于节点」的格子（128 KiB 单元 + 4 KiB 节点）才够得到。
    /// 次序本身钉成绝对值断言，免得下次改常量时它又静默翻回去。
    #[test]
    fn every_rejection_reason_is_reachable() {
        assert_eq!(maximum_extension_point_bytes_keeping_payload_positive(4096) + 2, maximum_extension_point_bytes_keeping_one_pointer_in_node(4096));
        assert_eq!(mount_verdict(5000, 4096, 4096, 512, 512), "reject_no_payload");
        assert_eq!(mount_verdict(4000, 4096, 4096, 512, 512), "reject_no_payload");
        assert_eq!(mount_verdict(4000, 131072, 4096, 512, 512), "reject_no_pointer");
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
        assert_eq!(minimum_extension_point_bytes_bare(), 7); // dev 1 + 物理偏移 6
        assert_eq!(minimum_extension_point_bytes_with_quota_length(), 9); // + 长度 2
        assert_eq!(minimum_extension_point_bytes_with_checksum(), 11); // + 密文校验和 4
        assert_eq!(minimum_extension_point_bytes_full(), 13); // 两样都要
    }

    /// **长度字段按字节计量时 2 个字节装不下一个 128 KiB 单元**：65535 < 131072。
    /// ⇒ 「长度 2 字节」这个借来的宽度对配额范围不成立，下界那一档还要再抬或改计量单位。
    #[test]
    fn two_byte_length_cannot_span_a_128_kib_unit() {
        assert_eq!(length_field_maximum_bytes(), 65535);
        assert!(length_field_maximum_bytes() < 131072);
    }

    /// 段二的上界，四个节点档各自钉死。独立算术（叶 2²⁴、指针 40、头 64）：
    /// 2 KiB 要 f ≤ 27（27⁵ = 14 348 907 < 2²⁴）⇒ extension_point_bytes ≥ 1984 − 1119 = 865；
    /// 4 KiB 要 f ≤ 63（63⁴ = 15 752 961 < 2²⁴）⇒ extension_point_bytes ≥ 4032 − 2559 = 1473；
    /// 16 KiB / 64 KiB 要 f ≤ 255（255³ = 16 581 375 < 2²⁴）⇒ 6081 / 55233。
    #[test]
    fn height_thresholds_are_pinned_per_node_size() {
        let got: Vec<u64> = NODES
            .iter()
            .map(|&node_bytes| height_threshold(node_bytes, Carrier::AllUnits, LEAF_COUNT).unwrap())
            .collect();
        assert_eq!(got, vec![865, 1473, 6081, 55233]);
    }

    /// 段二对 `DataOnly` 那一档**永远不涨**——索引节点不带扩展点。
    /// 两档结论不同 ⇒ D21 那个岔路是承重的，不是措辞问题。
    #[test]
    fn data_only_carrier_never_grows_the_tree() {
        for node_bytes in NODES {
            assert_eq!(height_threshold(node_bytes, Carrier::DataOnly, LEAF_COUNT), None);
            assert_eq!(
                fanout(node_bytes, 512, Carrier::DataOnly),
                fanout(node_bytes, 0, Carrier::DataOnly)
            );
        }
    }

    // ────────── 阳性对照：每一条臂都跑，不是只跑第一条 ──────────

    /// 四个单元档各自把 N 抬到单元的一半，占比必须显著上升，且等于独立算出的值。
    /// 测不出上升 ⇒ 模型根本没把扩展点算进去，整轮作废。
    #[test]
    fn positive_control_every_unit_arm() {
        for unit_bytes in UNITS {
            let base = share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS, unit_bytes);
            let half = share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS + 10 * (unit_bytes / 2), unit_bytes);
            assert!(half > base, "单元 {unit_bytes}：抬到一半之后占比没涨");
        }
        // 绝对值锚：4 KiB 单元、N = 2048 ⇒ (158 + 2048) / 4096 = 53.857%
        assert_eq!(share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS + 10 * 2048, 4096), 53857);
        // 128 KiB 单元、N = 65536 ⇒ (158 + 65536) / 131072 = 50.121%
        assert_eq!(share_in_thousandths_of_a_percent(BASE_METADATA_TENTHS + 10 * 65536, 131072), 50121);
    }

    /// 四个节点档各自把 N 抬到节点的一半，扇出必须下降，且等于独立算出的值。
    #[test]
    fn positive_control_every_node_arm() {
        for node_bytes in NODES {
            let base = fanout(node_bytes, 0, Carrier::AllUnits);
            let half = fanout(node_bytes, node_bytes / 2, Carrier::AllUnits);
            assert!(half < base, "节点 {node_bytes}：抬到一半之后扇出没降");
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
        for unit_bytes in UNITS {
            for extension_point_bytes in EXTENSION_POINT_BYTES_SCANNED {
                let metadata_tenths = BASE_METADATA_TENTHS + 10 * extension_point_bytes;
                let share_percent_via_float = (metadata_tenths as f64 / 10.0) / unit_bytes as f64 * 100.0;
                let via_float = (share_percent_via_float * 1000.0).round() as u64;
                assert_eq!(share_in_thousandths_of_a_percent(metadata_tenths, unit_bytes), via_float, "单元 {unit_bytes}、N {extension_point_bytes} 两条路径不一致");
            }
        }
    }

    /// 校验路径自己要能红：喂一个已知错的分母，两条路径必须**分道扬镳**。
    /// 抓不到 ⇒ 上面那条一致是回声，不是证据。
    #[test]
    fn the_cross_check_itself_can_go_red() {
        let metadata_tenths = BASE_METADATA_TENTHS;
        let wrong = share_in_thousandths_of_a_percent(metadata_tenths, 4096 * 2); // 把单元大小弄错一倍
        let right = ((metadata_tenths as f64 / 10.0) / 4096.0 * 100.0 * 1000.0).round() as u64;
        assert_ne!(wrong, right, "把分母弄错一倍之后两条路径居然还一致，这条校验是摆设");
    }
}
