//! 里程碑「第二个事务」固定脚本的推演模型（只用 std）。
//! 把 `.claude/kb/milestone/02-second-txn.md`「固定脚本（预想）」那一段逐次发布展开，
//! 按 D16（发布语义） 已定项 1、D16 已定项 8 / 已定项 9、D23（journal 的角色与格式） 已定项 14、
//! D18（块里携带什么信息） 已定项 11、D22（单元原子性怎么合成） 已定项 16 的根槽落点公式判每一步走不走得通。
//! 攻方腿第一轮（2026-09-16）用，不入库、不改仓里任何已有文件。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PublishKind {
    /// 改过用户可见状态的发布（D16 已定项 1「4」只数这一种）
    UserVisible,
    /// 暖机推的空发布（D16 已定项 8）
    WarmUpEmpty,
    /// 抬回退下界 F 产生的空发布（D16 已定项 1）
    RaiseFloorEmpty,
    /// 回退那次发布：回退行 + 中间实例行 + 这次发布的单元 + 回退根同一次（D23 已定项 14）
    RollbackPublish,
    /// 第一次之后的可写挂载写实例表行那次发布（D18 已定项 11）
    WriteRowPublish,
}

#[derive(Clone, Debug)]
struct Publish {
    checkpoint_txg: u64,
    instance_number: u32,
    kind: PublishKind,
    /// 这次发布写出的单元数（每盘），×2 才是录制流里的写请求数（D2 已定项 6 的 w ≥ 2）
    units_per_device: usize,
    /// 这次发布之前是否刚做过取号（取号两写自成一段，D23 已定项 16 的屏障切在它之后）
    preceded_by_instance_acquisition: bool,
    note: &'static str,
}

/// D22（单元原子性怎么合成） 已定项 16：区域 = txg mod 3、槽 = (txg div 3) mod 8。
fn region_of(checkpoint_txg: u64) -> u64 {
    checkpoint_txg % 3
}
fn slot_of(checkpoint_txg: u64) -> u64 {
    (checkpoint_txg / 3) % 8
}
/// D2（RAID 条带策略） 已定项 7：根环三个区域的设备归属第一版写死 0 / 1 / 0。
fn device_of_region(region: u64) -> u32 {
    match region {
        0 => 0,
        1 => 1,
        2 => 0,
        other => panic!("区域只有 0 / 1 / 2，拿到 {other}"),
    }
}
fn device_of_txg(checkpoint_txg: u64) -> u32 {
    device_of_region(region_of(checkpoint_txg))
}

/// D16 已定项 8 ⚠️：新实例写成的根覆盖两块盘之前不让 fsync 返回，做法是连推空发布。
/// 第一个带根的发布是 `first_txg`（首次挂载是暖机第一次，之后的挂载是写行那次、回退那次）。
/// 返回还要补几次空发布，以及补完之后的下一个可用 txg。
fn warm_up_empty_publishes(first_txg: u64, first_root_counts: bool) -> (Vec<u64>, u64) {
    let mut covered: Vec<u32> = Vec::new();
    let mut next = first_txg;
    if first_root_counts {
        covered.push(device_of_txg(first_txg));
        next = first_txg + 1;
    }
    let mut pushed = Vec::new();
    while !(covered.contains(&0) && covered.contains(&1)) {
        covered.push(device_of_txg(next));
        pushed.push(next);
        next += 1;
        assert!(pushed.len() <= 3, "D16 已定项 8 的几何上界 R = 3 被越过");
    }
    (pushed, next)
}

fn build_script() -> Vec<Publish> {
    let mut script: Vec<Publish> = Vec::new();
    // ---- 实例 1：mkfs 之后第一次可写挂载 ----
    // 写行区间 [max(所选根的实例, 1), 新实例) = [1, 1) 是空的 ⇒ 没有写行那次发布
    // （first-txn-layout.md 八「第一次之后的可写挂载（写行）」那一行逐字：实例 0 不写）。
    let (warm1, next) = warm_up_empty_publishes(1, false);
    for txg in &warm1 {
        script.push(Publish {
            checkpoint_txg: *txg,
            instance_number: 1,
            kind: PublishKind::WarmUpEmpty,
            units_per_device: 0, // 树表 0 条 ⇒ 零单元（D16 已定项 9 逐字）
            preceded_by_instance_acquisition: *txg == warm1[0],
            note: "暖机空发布（实例 1）",
        });
    }
    assert_eq!(next, 3, "FIRST_TRANSACTION_TXG 应当是 3");
    // A：第一个事务
    script.push(Publish { checkpoint_txg: 3, instance_number: 1, kind: PublishKind::UserVisible,
        units_per_device: 8, preceded_by_instance_acquisition: false, note: "A 第一个事务" });
    // B：覆盖写 + 释放第一个数据单元（步 1 / 步 2）
    script.push(Publish { checkpoint_txg: 4, instance_number: 1, kind: PublishKind::UserVisible,
        units_per_device: 8, preceded_by_instance_acquisition: false, note: "B 覆盖写 + 释放" });
    // ---- 实例 2：关闭重开（步 3）----
    script.push(Publish { checkpoint_txg: 5, instance_number: 2, kind: PublishKind::WriteRowPublish,
        units_per_device: 8, preceded_by_instance_acquisition: true, note: "写行那次发布" });
    let (warm2, next2) = warm_up_empty_publishes(5, true);
    for txg in &warm2 {
        script.push(Publish { checkpoint_txg: *txg, instance_number: 2, kind: PublishKind::WarmUpEmpty,
            units_per_device: C_MAX_UNITS, preceded_by_instance_acquisition: false,
            note: "暖机空发布（实例 2，记账树已存在 ⇒ 写 c_max 单元）" });
    }
    // C：再覆盖一次
    script.push(Publish { checkpoint_txg: next2, instance_number: 2, kind: PublishKind::UserVisible,
        units_per_device: 8, preceded_by_instance_acquisition: false, note: "C 覆盖写" });
    // ---- 实例 3：管理员回退到 A 的根（步 4）----
    let max_txg_in_ring = next2;
    let rollback_txg = max_txg_in_ring + 1;
    script.push(Publish { checkpoint_txg: rollback_txg, instance_number: 3, kind: PublishKind::RollbackPublish,
        units_per_device: 8, preceded_by_instance_acquisition: true, note: "D 回退行 + 中间实例行 + 回退根" });
    let (warm3, next3) = warm_up_empty_publishes(rollback_txg, true);
    for txg in &warm3 {
        script.push(Publish { checkpoint_txg: *txg, instance_number: 3, kind: PublishKind::WarmUpEmpty,
            units_per_device: C_MAX_UNITS, preceded_by_instance_acquisition: false,
            note: "暖机空发布（实例 3）" });
    }
    // ---- 步 5：释放那次 + 攒根 3 次 + 抬 F 空发布 + 生效补一次 + E ----
    let release_txg = next3;
    for (index, txg) in (release_txg..release_txg + 4).enumerate() {
        script.push(Publish { checkpoint_txg: txg, instance_number: 3, kind: PublishKind::UserVisible,
            units_per_device: 8, preceded_by_instance_acquisition: false,
            note: if index == 0 { "覆盖写：释放 50180，释放代 = 这次 txg" } else { "攒根用的覆盖写" } });
    }
    let raise_txg = release_txg + 4;
    script.push(Publish { checkpoint_txg: raise_txg, instance_number: 3, kind: PublishKind::RaiseFloorEmpty,
        units_per_device: C_MAX_UNITS, preceded_by_instance_acquisition: false, note: "抬 F 的空发布" });
    script.push(Publish { checkpoint_txg: raise_txg + 1, instance_number: 3, kind: PublishKind::RaiseFloorEmpty,
        units_per_device: C_MAX_UNITS, preceded_by_instance_acquisition: false,
        note: "补一次，让带新 F 的根覆盖两块盘（D16 已定项 1 的生效口径）" });
    script.push(Publish { checkpoint_txg: raise_txg + 2, instance_number: 3, kind: PublishKind::UserVisible,
        units_per_device: 8, preceded_by_instance_acquisition: false, note: "E 复用 50180" });
    script
}

/// 一次空发布写几个单元：D28（挂载期承诺量） 已定项 4 现算的 c_max。
/// 第一个事务那种池规模下 E148（提交固定点按两棵记录树重算） 报 4 块，池规模 9 块；
/// 这个脚本的池只有一个文件，取 5 当保守下沿，另在层 0 那一节按 9 再算一遍。
const C_MAX_UNITS: usize = 5;

/// D13（验证路线） 已定项 4 的闭式，与 crates/singlefs-harness/src/crash.rs 的
/// `closed_form_state_count` 逐字同型：1 + Σ(2^|段| − 1)。
fn closed_form(segments: &[usize]) -> u128 {
    1 + segments.iter().map(|n| (1u128 << n) - 1).sum::<u128>()
}

/// 把一条发布序列切成录制流的段（first-txn-layout.md 八的形状 + 末尾那条 ⚠️：
/// 发布与下一次发布之间没有屏障 ⇒ 上一次发布的超级块槽写与这次的单元写落在同一段）。
/// 返回每一段的写请求数。
fn segments_of(script: &[Publish], empty_publish_writes_units: bool) -> Vec<usize> {
    let mut segments: Vec<usize> = Vec::new();
    let mut carried_superblock_writes = 0usize; // 上一次发布留下的超级块槽写
    for publish in script {
        let mut units = publish.units_per_device * 2;
        if !empty_publish_writes_units
            && matches!(publish.kind, PublishKind::WarmUpEmpty | PublishKind::RaiseFloorEmpty)
        {
            units = 0; // 里程碑步 0 的模型：空发布同型 2+1+2
        }
        if publish.preceded_by_instance_acquisition {
            // [上一次的超级块 + 取号超级块 × 2] 屏障
            segments.push(carried_superblock_writes + 2);
            carried_superblock_writes = 0;
        }
        if units == 0 {
            // 零单元的空发布开头有屏障（登记表「空发布（暖机）」那一行逐字）
            if carried_superblock_writes > 0 {
                segments.push(carried_superblock_writes);
                carried_superblock_writes = 0;
            }
        } else {
            segments.push(carried_superblock_writes + units);
            carried_superblock_writes = 0;
        }
        segments.push(2); // [记录 × 2 盘] 屏障
        segments.push(1); // [根 FUA]
        carried_superblock_writes = 2; // [超级块槽 × 2]，与下一段合并
    }
    if carried_superblock_writes > 0 {
        segments.push(carried_superblock_writes);
    }
    segments
}

fn main() {
    let script = build_script();
    println!("== 一、固定脚本逐次发布（D22 已定项 16：区域 txg mod 3、槽 (txg div 3) mod 8；D2 已定项 7：区域 0/1/2 归盘 0/1/0）");
    println!("{:>4} {:>4} {:>3} {:>4} {:>4} {:>5}  {}", "txg", "实例", "区域", "槽", "盘", "单元", "这是什么");
    for publish in &script {
        println!(
            "{:>4} {:>4} {:>3} {:>4} {:>4} {:>5}  {}",
            publish.checkpoint_txg,
            publish.instance_number,
            region_of(publish.checkpoint_txg),
            slot_of(publish.checkpoint_txg),
            device_of_txg(publish.checkpoint_txg),
            publish.units_per_device,
            publish.note
        );
    }

    println!("\n== 二、暖机次数（D16 已定项 8：本实例写成的根覆盖两块盘之前不让 fsync 返回）");
    for (label, first_txg, counts) in [
        ("实例 1（首次挂载，第一个带根的发布就是暖机第一次）", 1u64, false),
        ("实例 2（写行那次发布自己的根也算一块盘）", 5, true),
        ("实例 3（回退那次发布自己的根也算一块盘）", 9, true),
    ] {
        let (pushed, next) = warm_up_empty_publishes(first_txg, counts);
        println!("  {label}：要补 {} 次空发布 {:?}，之后第一个可用 txg = {next}", pushed.len(), pushed);
    }
}
