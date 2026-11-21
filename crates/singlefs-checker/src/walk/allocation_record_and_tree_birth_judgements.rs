//! 分配记录与树的诞生：I-5.5（从 mkfs 水位建树那一版的 t1 落点）、释放代与树表诞生 txg、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）、
//! I-5.4（分配记录罩住的槽互不相交）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 与 I-5.7（由记录施加出来的一版的分配记录树根出生在这一版）。

use super::*;

/// 走读到的一版里 I-5.5 要的四样：称呼（判红时写进第一处违例）、`checkpoint_txg`、它的树表指针与中央映射根指针
/// （根环里的根取根记录偏移 36 起与 256 起的各 86 字节，由记录施加出来的那一版取记录新根段里的那两条）。
pub(super) struct VersionToJudgeTheFirstDataUnitOf<'record> {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    pub(super) tree_table_pointer: &'record [u8],
    pub(super) mapping_root_pointer: &'record [u8],
}

/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：走读到的每一版（回退候选集里每条根，含由记录施加出来的那几版）各自重读一遍树表，
/// ① 树表非空，且种类 1（extent 树）条目的树 ID 等于 mkfs 种下的水位 65536；② 树表里每条条目的诞生 txg（条目偏移 108）都等于这一版的
/// `checkpoint_txg`——两样都成立就是从 mkfs 水位第一次建树的那一版：它的 extent 树引用的数据单元里，每块盘上落点槽号最小的那一个
/// 要等于这一版的分配记录在那块盘上算出的落点（[`first_data_unit_slot_the_allocation_records_leave_on_a_device`]：从单元区起始槽号 U 起，
/// 起点最小、32768 对齐、两槽都空、不在开放段里的那一对；U 是所选系统配置偏移 417）。不写死 U + 4：mkfs 的实例表（U、U + 1）被 COW 走、
/// 过了回收门槛之后，实现照 D3（空间分配） 已定项 8 第 1 条把数据放在 U（ I-5.5 按分配记录现算，判决
/// `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2）。这一版没有数据单元时不判，不满足 ①② 的版本不判。
/// 只读、不判别的不变量：主走读按 `visited_units` 只走一遍共享子树，共享的 extent 树只算进第一条走到它的版本，这一条要的是
/// 「这一版自己引用了哪些数据单元」（与 [`references_of_root`] 同一个缘故）。
///
/// 下面几格这一版判不了，报部分没判（[`InvariantVerdict::PartiallyJudged`]
/// 第二节 Y3）：没有违例时整条报部分没判，逐版列出没判的那一版（实例、txg）与为什么，形态同 I-9.28 树表没读全的那几条根：
/// 树表读不出、条目宽不到字段表的 200 字节（认不出这一版是不是这一类）；认出来的那一版 extent 树读不全（数据单元数不全）、
/// 分配记录树读不全（落点算不出）；种类 1 条目不止一条（① 读哪一条）、种类 3（分配记录树）条目不是恰好一条（落点按哪一棵算）——
/// 这两格的树表本身由 I-9.16 ③ ④ 判红，这里只记没判。
/// 这一版有数据单元、而池里某块盘上一个都没有：那块盘上没有「最小的那一个」，该盘跳过、不记（多盘布局下合法；GPU 判器本来就跳过）。
pub(super) fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
    reader: &dyn ImageReader,
    versions: &[VersionToJudgeTheFirstDataUnitOf<'_>],
    unit_area_start_slot: u64,
    judgements: &mut Judgements,
) {
    let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;
    let devices = reader.devices();
    let mut cache = IndexNodeCache::new();
    // 没判的那几版：「（版本称呼，含实例与 txg）：为什么」，逐版一条，走完一起报（`Judgements::partially_judged` 只记第一次给的说明）。
    let mut versions_not_judged: Vec<String> = Vec::new();
    // 认出来的从 mkfs 水位第一次建树的那几版（不论有没有数据单元）：一版都没认出来时报专门的不适用理由。
    let mut versions_built_from_the_mkfs_watermark = 0u64;
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            // core 照全零的树表指针读盘 0 槽 0、读不出，与树表读不出同一处置。
            versions_not_judged.push(format!(
                "{}：树表指针全零（core 照它读盘 0 槽 0、读不出），认不出它是不是从 mkfs 水位第一次建树的那一版",
                version.name
            ));
            continue;
        }
        let Some(tree_table) = read_index_node_without_judging(
            reader,
            &tree_table_pointer,
            PositionOfASecondaryRead::ThePointerItself,
            &mut cache,
        ) else {
            versions_not_judged.push(format!(
                "{}：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版",
                version.name
            ));
            continue;
        };
        if tree_table.entries.is_empty() {
            continue;
        }
        if tree_table
            .entries
            .iter()
            .any(|entry| entry.len() < tree_table_entry_bytes())
        {
            versions_not_judged.push(format!(
                "{}：树表条目宽不到字段表的 200 字节，认不出它是不是从 mkfs 水位第一次建树的那一版",
                version.name
            ));
            continue;
        }
        let extent_entries: Vec<&Vec<u8>> = tree_table
            .entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_EXTENT)
            .collect();
        let extent_entry = match extent_entries.as_slice() {
            [] => continue,
            [only_extent_entry] => *only_extent_entry,
            [_, _, ..] => {
                versions_not_judged.push(format!(
                    "{}：树表里种类 1（extent 树）条目有 {} 条，认法 ① 读不出哪一条（由 I-9.16 ③ ④ 判）",
                    version.name,
                    extent_entries.len()
                ));
                continue;
            }
        };
        let is_built_from_the_mkfs_watermark = read_u64(extent_entry, 0)
            == TREE_IDENTIFIER_WATERMARK_AT_MKFS
            && tree_table.entries.iter().all(|entry| {
                read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET) == version.checkpoint_txg
            });
        if !is_built_from_the_mkfs_watermark {
            continue;
        }
        versions_built_from_the_mkfs_watermark += 1;
        let extent_root = parse_node_pointer(&extent_entry[14..100]);
        if extent_root.all_zero {
            continue;
        }
        // 这一版自己的中央映射：extent 树、分配记录树的节点与数据单元按它认位置（读不出时是空表，照提示）。
        let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
            reader,
            &parse_node_pointer(version.mapping_root_pointer),
            &mut cache,
        );
        let Some((_, data_pointers)) =
            extent_tree_without_judging(reader, &extent_root, &central_mapping, &mut cache)
        else {
            versions_not_judged.push(format!(
                "{}（从 mkfs 水位第一次建树的那一版）：extent 树读不全，它引用的数据单元数不全",
                version.name
            ));
            continue;
        };
        // 落点按映射认过的位置取（D19（块指针的结构与宽度预算） 已定项 5：位置条目只是提示），同主走读记引用集合的那一处。
        let mut smallest_data_unit_slot_on_each_device: BTreeMap<u32, u64> = BTreeMap::new();
        for location in data_pointers
            .iter()
            .filter(|pointer| !pointer.all_zero)
            .flat_map(|pointer| {
                central_mapping
                    .locate(
                        pointer,
                        PositionAuthority::TheCentralMapping(MappingKeyUnitClass::DataUnit),
                    )
                    .locations
            })
        {
            smallest_data_unit_slot_on_each_device
                .entry(location.device)
                .and_modify(|smallest_slot| *smallest_slot = (*smallest_slot).min(location.slot))
                .or_insert(location.slot);
        }
        if smallest_data_unit_slot_on_each_device.is_empty() {
            continue;
        }
        let allocation_entries: Vec<&Vec<u8>> = tree_table
            .entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ALLOCATION)
            .collect();
        let [allocation_entry] = allocation_entries.as_slice() else {
            versions_not_judged.push(format!(
                "{}（从 mkfs 水位第一次建树的那一版）：树表里种类 3（分配记录树）条目有 {} 条，落点按哪一棵算不出（由 I-9.16 ③ ④ 判）",
                version.name,
                allocation_entries.len()
            ));
            continue;
        };
        let Some((_, allocation_records)) = allocation_record_tree_without_judging(
            reader,
            &parse_node_pointer(&allocation_entry[14..100]),
            &central_mapping,
            &mut cache,
        ) else {
            versions_not_judged.push(format!(
                "{}（从 mkfs 水位第一次建树的那一版）：分配记录树读不全，落点算不出",
                version.name
            ));
            continue;
        };
        for device in &devices {
            // 这一版在这块盘上一个数据单元都没有：没有「最小的那一个」，该盘跳过、不记。
            if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device) {
                let expected_slot = first_data_unit_slot_the_allocation_records_leave_on_a_device(
                    &allocation_records,
                    *device,
                    version.checkpoint_txg,
                    unit_area_start_slot,
                    open_cluster_segment_named_by_the_tree_table_on_a_device(
                        &tree_table_pointer,
                        *device,
                    ),
                );
                judgements.judge(invariant, smallest_slot == expected_slot, || {
                    format!(
                        "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是这一版的分配记录从单元区起始槽号 {unit_area_start_slot} 起算出的落点 {expected_slot}",
                        version.name
                    )
                });
            }
        }
    }
    if !versions_not_judged.is_empty() {
        judgements.partially_judged(
            invariant,
            format!(
                "判到的那几版落点都对得上；这几版没判：{}",
                versions_not_judged.join("；")
            ),
        );
    }
    // 一版都没认出来时的专门理由（「四支豁免」那一行）：不然落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开。认出来了而一版都没判到（认出来的那几版在池里的盘上都没有数据单元）
    // 另写一句。
    // `not_applicable` 不改已经评估过的：认出来的版本里有一版判到了，这一句不出现。
    if versions_built_from_the_mkfs_watermark == 0 {
        judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);
    } else {
        judgements.not_applicable(
            invariant,
            VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT,
        );
    }
}

/// I-5.5 一次都没判、因为走读到的版本里没有一版满足认法 ①② 时的不适用理由。
pub const NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK: &str = "回退候选集里每条根与由记录施加出来的那几版，没有一版是从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条条目的诞生 txg 等于这一版的 txg）：一版都不判";

/// I-5.5 一次都没判、因为认出来的从 mkfs 水位第一次建树的那几版在池里的盘上都没有数据单元时的不适用理由（那一行「V 没有数据单元时不判」）：
/// extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上（判法逐盘只看池里有的盘），三样都走到这一句
pub const VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT: &str = "认出了从 mkfs 水位第一次建树的那一版，但认出来的那几版在池里的盘上都没有数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上）：一版都不判";

/// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenClusterSegment {
    first_slot: u64,
}

impl OpenClusterSegment {
    fn holds_slot(self, slot: u64) -> bool {
        self.first_slot <= slot && slot < self.first_slot + CLUSTER_SEGMENT_SLOTS
    }
}

/// I-5.5 拿哪一段当这一版在这块盘上的开放段：这一版树表单元在这块盘上那一份所在的段（树表单元最末从开放段 bump，
/// D3（空间分配） 已定项 10 ⑤）。树表指针在这块盘上没有位置条目时交回 `None`（这块盘上不排除哪一段）。
fn open_cluster_segment_named_by_the_tree_table_on_a_device(
    tree_table_pointer: &PointerView,
    device: u32,
) -> Option<OpenClusterSegment> {
    tree_table_pointer
        .locations
        .iter()
        .find(|location| location.device == device)
        .map(|location| OpenClusterSegment {
            first_slot: location.slot - location.slot % CLUSTER_SEGMENT_SLOTS,
        })
}

/// I-5.5 的判据（按这一版的分配记录现算，D3（空间分配） 已定项 8 第 1 条、已定项 10 ② ③）：
/// 从单元区起始槽号 U 起，这一版在 `device` 上起点最小、起点 32768 对齐（槽号是 [`DATA_UNIT_SLOTS`] 的整数倍；U 是 64 的整数倍，
/// 读系统配置时判过）、两槽都空、不在开放段里的那一对槽的起点。「空」= 没有被「已释放、或分配代早于这一版」的分配记录罩着。
/// 不看盘尾：比每一个被占的槽都靠后的那一对总是空的，交回的落点越过盘尾时与任何真落点都不等，照样判红。
/// 迭代上界：跳过开放段至多一次（候选只增不减），其余每一轮跳过的那一对里至少有一个被占的槽，合计不超过被占槽数 + 1 轮。
///
/// 下面几样 checker 在这一版的盘面上复算不出，只能近似，都照实写进 `crates/singlefs-checker/invariants/07-分配器.md` I-5.5 那一行：
/// - 回收门槛：已释放的记录一律当占着。实现按「释放代 ≤ max(F_生效, 环里最旧有效根)」回收（D3（空间分配） 已定项 7 清扫准入），
///   这一版根里的 F、txg 与根环槽数盘上有，被抛弃的根与挂载那一刻读不出的槽盘上没有，门槛复算不出；记录在被重新分配之前一直写着已释放。
///   ⇒ 写死从 U + 4 起扫的坏实现在「U、U + 1 已回收、还没被重新分配」的那几版上判成立，抓不到。
/// - 这一版自己分配的记录（未释放、分配代等于这一版的 txg）当空：它们就是这一版取落点时还空着的槽（数据单元与提交内生块都在里面）。
/// - 开放段盘上没有字段：拿这一版树表单元所在的段当它（[`open_cluster_segment_named_by_the_tree_table_on_a_device`]）；
///   这次挂载更早开过、已经用尽的段认不出（段用尽才换段，用尽的段里剩不下起点对齐的空槽对）。
/// - 分配准入合取（D3（空间分配） 已定项 7）里隔离位（影子账）、抬 F 扣住位、保护期内的 R 盘上都没有：没有分配记录罩着的槽一律当空。
fn first_data_unit_slot_the_allocation_records_leave_on_a_device(
    allocation_records: &[AllocationRecordView],
    device: u32,
    version_checkpoint_txg: u64,
    unit_area_start_slot: u64,
    open_cluster_segment: Option<OpenClusterSegment>,
) -> u64 {
    let occupied_slots: BTreeSet<u64> = allocation_records
        .iter()
        .filter(|record| {
            record.device == device
                && (record.is_released || record.generation < version_checkpoint_txg)
        })
        .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
        .collect();
    let mut candidate = unit_area_start_slot;
    loop {
        if let Some(segment) = open_cluster_segment.filter(|segment| segment.holds_slot(candidate))
        {
            candidate = segment.first_slot + CLUSTER_SEGMENT_SLOTS;
            continue;
        }
        if (candidate..candidate + DATA_UNIT_SLOTS).all(|slot| !occupied_slots.contains(&slot)) {
            return candidate;
        }
        candidate += DATA_UNIT_SLOTS;
    }
}

/// 候选集里的一条根：它在 `roots` 里的下标、它的 checkpoint_txg 与这一遍取到的引用集合。
struct ScannedCandidateRoot {
    root_index: usize,
    pub(super) checkpoint_txg: u64,
    pub(super) references: RootReferences,
}

/// 一棵树的树表条目在某一条有效根那里的样子（I-9.14 按树 ID 归并这些）。
struct TreeTableEntrySighting {
    root_checkpoint_txg: u64,
    /// 这条根的树表单元落在哪 (设备身份, 起点槽)：两条根共用一个树表单元时，它们的条目不是两处独立的证据。
    pub(super) tree_table_placement: (u32, u64),
    pub(super) birth_txg: u64,
}

/// I-3.9（释放代落在停止引用它的那一格区间里）：对最新根那棵账里每条带已释放标志的分配记录，在候选集里取还引用这个落点的
/// 最新一条有效根（L）与最早不再引用它的有效根（T，L 之后的第一条），释放代必须落在 `(L, T]` 里；每条有效根都还引用它的
/// （L 之后没有根），它不许带已释放标志。候选集里一条根都没引用过它的（已经回收、还没被复用：见证它释放的根已不在候选集里）
/// 跳过不判，全是这一类时整条报不适用——三处都照 `crates/singlefs-checker/invariants/05-空间记账.md` I-3.9 那一行。
///
/// 为什么是区间：根环里的 txg 可以有洞——一次发布的根槽从没写过、而它的 journal 记录在挂载时被施加（D23（journal 的角色与格式）：
/// 记录自带新根段），那次发布做的释放就写着一个环里没有根的 txg。`crash_enumeration_fixed_script_stream.rs` 的残留记录那条流
/// 正是这个形状（txg 5 的根槽从没写过、jsn 5 被施加、释放代 5、环里最早不再引用那个落点的根是 txg 6），按「等于 T」判，
/// 那条流上 12 个合法状态判红。L 与 T 之间没有洞时 `(L, T]` 只有 T 一个值，就是「等于最早不再引用它的那条根的 txg」；
/// C374（释放代与树表诞生 txg 只有验收断言盯着） 点名的判别力（发布 B 那次的释放代从 4 写成 3）照样红。
fn judge_release_generations(
    scanned: &[ScannedCandidateRoot],
    newest_position: usize,
    judgements: &mut Judgements,
) {
    let released: Vec<&AllocationRecordView> = scanned[newest_position]
        .references
        .allocation_records
        .iter()
        .filter(|record| record.is_released)
        .collect();
    if released.is_empty() {
        judgements.not_applicable("I-3.9", "最新的根下面没有带已释放标志的分配记录");
        return;
    }
    for root in scanned {
        assert_eq!(
            root.references.depth,
            ReferenceScanDepth::EveryReferencedPlacement,
            "最新根那棵账里有已释放记录时，每条候选根都按「走到叶」扫过（`judge_release_generation_and_tree_table_birth` 定的深度）"
        );
    }
    // 守卫按记录收窄（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行）：比 L 晚的候选根有一条走不完，
    // 这个落点可能藏在它读不出的节点下面（L 算小、T 算早），这一条跳过；L 及更早的根走不完不碍事——L 取的是还引用它的最大 txg。
    let mut skipped_behind_an_incomplete_later_root = 0u64;
    let mut witnessed_records = 0u64;
    for record in released {
        let placement = (record.device, record.slot);
        let Some(last_referencing_txg) = scanned
            .iter()
            .filter(|root| root.references.placements.contains(&placement))
            .map(|root| root.checkpoint_txg)
            .max()
        else {
            // 候选集里一条根都没引用过它：见证它释放的那条根已经不在候选集里（F_生效 抬过释放代之后落点被回收，
            // 盘上那条记录仍写着已释放，D16（发布语义） 已定项 1 / D3（空间分配） 已定项 7）。这个镜像上找不到那条根，这一条判不了。
            continue;
        };
        let some_later_root_is_incomplete = scanned
            .iter()
            .any(|root| root.checkpoint_txg > last_referencing_txg && !root.references.is_complete);
        if some_later_root_is_incomplete {
            skipped_behind_an_incomplete_later_root += 1;
            continue;
        }
        witnessed_records += 1;
        let first_root_without_it = scanned
            .iter()
            .map(|root| root.checkpoint_txg)
            .filter(|txg| *txg > last_referencing_txg)
            .min();
        let in_the_witnessed_interval = record.generation > last_referencing_txg
            && first_root_without_it.is_some_and(|txg| record.generation <= txg);
        judgements.judge("I-3.9", in_the_witnessed_interval, || {
            match first_root_without_it {
                Some(txg) => format!(
                    "盘 {} 槽 {} 的分配记录释放代 {} 不在 ({last_referencing_txg}, {txg}] 里：还引用它的最新一条有效根是 txg {last_referencing_txg}，最早不再引用它的是 txg {txg}",
                    record.device, record.slot, record.generation
                ),
                None => format!(
                    "盘 {} 槽 {} 的分配记录带已释放标志（释放代 {}），而候选集里每一条有效根都还引用它",
                    record.device, record.slot, record.generation
                ),
            }
        });
    }
    if skipped_behind_an_incomplete_later_root > 0 {
        judgements.part_of_the_range_left_unjudged(
            "I-3.9",
            format!("有 {skipped_behind_an_incomplete_later_root} 条带已释放标志的记录落在比还引用它的最后一条根晚、又走不完的候选根后面（单元读不出，或走到条目格式没有条款的形状）：那几条的 L 与 T 定不下"),
        );
    }
    if witnessed_records == 0 {
        judgements.not_applicable(
            "I-3.9",
            "带已释放标志的记录一条都没有候选根引用过：见证它们释放的根已不在候选集里",
        );
    }
}

/// I-9.14（树表条目的诞生 txg 跨根不变）：按树 ID 归并候选集里每条根的树表条目，诞生 txg 必须相同。
/// 只在这棵树的条目出现在**两个不同的树表单元**里时才判——几条根共用一个树表单元（暖机那几代都指着 mkfs 种的第 0 版）时，
/// 拿同一份字节比自己是恒真的判定，那正是 C374（释放代与树表诞生 txg 只有验收断言盯着） 点的那一类；这时报「不适用」，不报成立。
///
/// ⚠️ **「跨根」的射程只到同一条时间线**（C511（回退到无文件那一版之后诞生代怎么接）：收窄）。
/// 收窄不在这个函数里，在**入参**上：`scanned` 是回退候选集——按最新那条根指着的实例表判仍然有效的那一组根
/// （D23（journal 的角色与格式） 已定项 14：(i, T) 有效 ⟺ 实例表里没有实例 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti），
/// 筛在 `check_pool_image` 里 `abandoned` 那一条上。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的那条旧线上，
/// 一进不来，所以这里比的恒是同一条线上的根。
/// **为什么收窄**：同一条线上的根之间才有「这棵树只在第一次出现时设诞生 txg」这层关系可比；被崩溃恢复切掉的旧线上的根不与现行线比。
/// 管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），不切时间线、树表条目照抄，不走到这一格。
/// 跨线（含被崩溃恢复抛弃的旧线）的诞生 txg 一致归 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）
/// （[`judge_tree_identifier_birth_txg_across_every_readable_root`]）：这一条不再有「只有被切掉的那条线记得不同，不许红」那一格
/// （判别力第 ③ 条与它的正向用例删掉；那份镜像今天红 I-9.28、I-9.14 照旧成立）。
/// 没有被抛弃时间线的那一格由 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite` 盯着。
/// I-9.14 部分没判时一条候选根的树表为什么没进比较（C701）；读全了是 `None`。
fn why_the_tree_table_of_a_candidate_root_is_not_read_whole(
    reading: &TreeTableReading,
) -> Option<&'static str> {
    match reading {
        TreeTableReading::EveryEntryRead => None,
        TreeTableReading::PointerAllZero => Some("树表指针全零"),
        TreeTableReading::Unreadable => Some("树表读不出"),
        TreeTableReading::SomeEntryShorterThanTheFieldTable => Some("树表有条目比字段表短"),
    }
}

fn judge_tree_table_birth_txg(scanned: &[ScannedCandidateRoot], judgements: &mut Judgements) {
    let mut by_tree: BTreeMap<u64, Vec<TreeTableEntrySighting>> = BTreeMap::new();
    // 树表没读全的候选根（C701（别的判法静默跳过读不出的部分））：它们的条目没进比较，
    // 读不出的那一份可能正是对不上的那一份，逐条列出、报部分没判，不静默少一份证据（与 I-9.28 同一个报法）。
    let mut roots_whose_tree_table_is_not_read_whole: Vec<String> = Vec::new();
    for root in scanned {
        let why_not_read_whole = why_the_tree_table_of_a_candidate_root_is_not_read_whole(
            &root.references.tree_table_reading,
        );
        if let Some(why) = why_not_read_whole {
            roots_whose_tree_table_is_not_read_whole
                .push(format!("候选根 txg {}：{why}", root.checkpoint_txg));
        }
        let Some(tree_table_placement) = root.references.tree_table_placement else {
            continue;
        };
        for (tree, birth_txg) in &root.references.tree_table_birth_txg {
            by_tree
                .entry(*tree)
                .or_default()
                .push(TreeTableEntrySighting {
                    root_checkpoint_txg: root.checkpoint_txg,
                    tree_table_placement,
                    birth_txg: *birth_txg,
                });
        }
    }
    let mut compared_trees = 0u64;
    for (tree, sightings) in &by_tree {
        let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings
            .iter()
            .map(|sighting| sighting.tree_table_placement)
            .collect();
        if distinct_tree_tables.len() < 2 {
            continue;
        }
        compared_trees += 1;
        let first_birth_txg = sightings[0].birth_txg;
        let birth_txg_is_the_same_across_roots = sightings
            .iter()
            .all(|sighting| sighting.birth_txg == first_birth_txg);
        judgements.judge("I-9.14", birth_txg_is_the_same_across_roots, || {
            let seen: Vec<String> = sightings
                .iter()
                .map(|sighting| {
                    format!(
                        "根 txg {} 记 {}",
                        sighting.root_checkpoint_txg, sighting.birth_txg
                    )
                })
                .collect();
            format!("树 {tree} 的树表条目诞生 txg 跨根不一：{}", seen.join("、"))
        });
    }
    if !roots_whose_tree_table_is_not_read_whole.is_empty() {
        judgements.partially_judged(
            "I-9.14",
            format!(
                "这几条候选根的树表条目没进比较：{}",
                roots_whose_tree_table_is_not_read_whole.join("；")
            ),
        );
    }
    if compared_trees == 0 {
        judgements.not_applicable(
            "I-9.14",
            "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
        );
    }
}

/// 一棵树的树表条目在根环里某一条读得出的根那里的样子（I-9.28 按树 ID 归并这些；不分时间线，所以带上实例代号）。
struct TreeTableEntrySightingInTheRing {
    root_instance: u32,
    root_checkpoint_txg: u64,
    /// 这条根的树表单元落在哪 (设备身份, 起点槽)：两条根共用一个树表单元时，它们的条目不是两处独立的证据。
    pub(super) tree_table_placement: (u32, u64),
    pub(super) birth_txg: u64,
}

/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）：根环里每一条读得出且自证合法的根（`roots`，
/// 不按实例表判有效与否、不按 F 与窗口滤，含被崩溃恢复抛弃的旧线上的根）各读一遍树表，按树 ID（条目偏移 0）归并，诞生 txg 必须相同。
/// 判法 checker 自己写（D13（验证路线） 已定项 5），与 I-9.14 的 [`judge_tree_table_birth_txg`] 分开：那一条只比回退候选集。
///
/// 四种结局：
/// - 有一棵树在两处记的诞生 txg 不一 ⇒ 违例；
/// - 都对得上，而有读得出、自证过的根树表指针非零、树表却读不出或有条目比字段表短 ⇒ 部分没判（[`InvariantVerdict::PartiallyJudged`]），
///   逐条列出这些根（盘号、实例、txg、为什么没判）：它们的条目没进比较，读不出的那一份可能正是对不上的那一份；
/// - 都没有，也没有一棵树的条目出现在**两个不同的树表单元**里 ⇒ 不适用（几条根共用一个树表单元时拿同一份字节比自己是恒真判定，
///   C374（释放代与树表诞生 txg 只有验收断言盯着） 那一类），不报成立；
/// - 其余 ⇒ 成立。
///
/// 树表指针全零的根照 core 的根记录读者读盘 0 槽 0、读不出，与树表读不出一样报部分没判。
pub(super) fn judge_tree_identifier_birth_txg_across_every_readable_root(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut sightings_by_tree: BTreeMap<u64, Vec<TreeTableEntrySightingInTheRing>> =
        BTreeMap::new();
    let mut roots_whose_tree_table_was_not_read_in_full: Vec<String> = Vec::new();
    for (region, _, root) in roots {
        let references = references_of_root(
            reader,
            &root.record_bytes,
            ReferenceScanDepth::TreeTableOnly,
            cache,
        );
        let why_not_judged = match references.tree_table_reading {
            TreeTableReading::EveryEntryRead => None,
            TreeTableReading::PointerAllZero => {
                Some("树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）")
            }
            TreeTableReading::Unreadable => Some("树表读不出"),
            TreeTableReading::SomeEntryShorterThanTheFieldTable => Some("树表有条目比字段表短"),
        };
        if let Some(why_not_judged) = why_not_judged {
            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
            let device = geometry.region_devices[usize::try_from(*region).expect(
                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
            )];
            roots_whose_tree_table_was_not_read_in_full.push(format!(
                "盘 {device} 上的根（实例 {}、txg {}）：{why_not_judged}",
                root.instance, root.checkpoint_txg
            ));
        }
        let Some(tree_table_placement) = references.tree_table_placement else {
            continue;
        };
        for (tree, birth_txg) in &references.tree_table_birth_txg {
            sightings_by_tree
                .entry(*tree)
                .or_default()
                .push(TreeTableEntrySightingInTheRing {
                    root_instance: root.instance,
                    root_checkpoint_txg: root.checkpoint_txg,
                    tree_table_placement,
                    birth_txg: *birth_txg,
                });
        }
    }
    let mut compared_trees = 0u64;
    for (tree, sightings) in &sightings_by_tree {
        let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings
            .iter()
            .map(|sighting| sighting.tree_table_placement)
            .collect();
        if distinct_tree_tables.len() < 2 {
            continue;
        }
        compared_trees += 1;
        let first_birth_txg = sightings[0].birth_txg;
        let birth_txg_is_the_same_in_every_readable_root = sightings
            .iter()
            .all(|sighting| sighting.birth_txg == first_birth_txg);
        judgements.judge(
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            birth_txg_is_the_same_in_every_readable_root,
            || {
                let seen: Vec<String> = sightings
                    .iter()
                    .map(|sighting| {
                        format!(
                            "根（实例 {}、txg {}）记 {}",
                            sighting.root_instance,
                            sighting.root_checkpoint_txg,
                            sighting.birth_txg
                        )
                    })
                    .collect();
                format!(
                    "树 {tree} 的树表条目诞生 txg 在根环全部读得出的根里不一：{}",
                    seen.join("、")
                )
            },
        );
    }
    if !roots_whose_tree_table_was_not_read_in_full.is_empty() {
        judgements.partially_judged(
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            format!(
                "比到的 {compared_trees} 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较：{}",
                roots_whose_tree_table_was_not_read_in_full.join("；")
            ),
        );
    } else if compared_trees == 0 {
        judgements.not_applicable(
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
        );
    }
}

/// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
/// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（随 C374 立条时定的口径）。
pub(super) fn judge_release_generation_and_tree_table_birth(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    newest_index: usize,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    if candidate_indexes.len() < 2 {
        judgements.not_applicable(
            "I-3.9",
            "回退候选集里只有一条根：没有第二条根能见证「不再引用这个落点」",
        );
        judgements.not_applicable(
            "I-9.14",
            "回退候选集里只有一条根：树表条目没有第二条根的那一份可比",
        );
        return;
    }
    // 最新根那棵账定这一遍走多深：它一条已释放记录都没有时 I-3.9 判不了，别的根只要树表（I-9.14 那一半）。
    let newest = references_of_root(
        reader,
        &roots[newest_index].2.record_bytes,
        ReferenceScanDepth::EveryReferencedPlacement,
        cache,
    );
    let depth = if newest
        .allocation_records
        .iter()
        .any(|record| record.is_released)
    {
        ReferenceScanDepth::EveryReferencedPlacement
    } else {
        ReferenceScanDepth::TreeTableOnly
    };
    let mut scanned: Vec<ScannedCandidateRoot> = Vec::with_capacity(candidate_indexes.len());
    for index in candidate_indexes.iter().copied() {
        scanned.push(ScannedCandidateRoot {
            root_index: index,
            checkpoint_txg: roots[index].2.checkpoint_txg,
            references: if index == newest_index {
                newest.clone()
            } else {
                references_of_root(reader, &roots[index].2.record_bytes, depth, cache)
            },
        });
    }
    scanned.sort_unstable_by_key(|root| root.checkpoint_txg);
    let newest_position = scanned
        .iter()
        .position(|root| root.root_index == newest_index)
        .expect("最新根在回退候选集里（`check_pool_image` 无条件把它放进去）");
    judge_release_generations(&scanned, newest_position, judgements);
    judge_tree_table_birth_txg(&scanned, judgements);
}

/// I-5.4 按版本那一层判的一版：称呼（判红与部分没判时点名，说清是候选根、由记录施加出来的那一版、还是下一次挂载先施加的那一版）、
/// 树表指针与「树表 0 条那一版的分配记录树根」（根环里的根取根记录偏移 36 与 342 起的 86 字节，
/// 另两类取记录新根段偏移 0 与 274 起的那两条：D23（journal 的角色与格式） 已定项 4 的字段表，施加一条记录 = 把所选根的这几个字段换成记录里的），
/// 与这一版的中央映射根指针（根记录偏移 256 起，记录新根段偏移 86 起）：分配记录树的节点不在豁免三类里，按这一版的映射认位置。
pub(super) struct VersionToJudgeTheAllocationRecordsOf<'record> {
    pub(super) name: String,
    pub(super) tree_table_pointer: &'record [u8],
    pub(super) allocation_record_tree_root_held_outside_the_tree_table: &'record [u8],
    pub(super) mapping_root_pointer: &'record [u8],
}

/// I-5.4（分配记录罩住的槽互不相交）：候选集里每条有效根的分配记录树，同一块盘上任意两条记录（不论已分配、已释放、释放代是否已低于
/// 回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交（`crates/singlefs-checker/invariants/07-分配器.md` I-5.4 那一行）。I-5.1（物理范围不重叠） 管的是
/// 树里的引用，判不出记录之间的重叠：引用互不重叠而两条分配记录罩住同一个槽的镜像，下一次可写挂载重建分配器时 panic。
/// 几条根指着同一个分配记录树根时（照抄上一版的根）那棵树只判一次；每个 (树, 盘) 判一格，那块盘上只有一条记录也算判过。
/// 树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读下来，同一块盘上跨叶的记录一起比。
/// 走不到记录的根不判：树表或分配记录树读不出、位置对不上（由 I-2.1 / I-1.1 / I-7.2 说话）。
///
/// 同一条下另判一格「一条根只有一棵分配记录树」：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节
/// （偏移 342）必须全零——那一版的账住树表种类 3 那一条，根记录这一项只有树表 0 条、写过行的那一版写非零
/// （D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行；
/// 第二节 Y1 `allocation_record_tree_root` 那一行）。非零时这一版有两棵分配记录树，「这条根的分配记录树」不止一棵。
/// 不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
/// 树表指针非零而树表读不出的根，这一格判不了（认不出带不带树），它那一版自己的分配记录树（树表种类 3 那一条指着的那一棵）也走不到：
/// 不计成立，I-5.4 报部分没判、逐版列出
/// 树表读得出、0 条而根记录那一项非零的根（写过行的那一版，C512（树表 0 条的一版上被换下的实例表记在哪没有条款）），它那一版的分配记录树就是
/// 根记录直接持有的那一棵：那棵树走不到叶时同样不计成立，I-5.4 报部分没判、逐版列出（F1 的同形）。
/// 这一格成立的那一次评估只在**这一版自己的**分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4（判决
/// `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行，按版本那一层是
/// `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 的零轮收严）：没走到的那几版不计，I-5.4 报部分没判、逐版列出没计的那几版
/// ——否则带树一版上这一格成立一次就压过不适用或别的版本没判到的那一半，这一版分配记录树两份清零的镜像上 I-5.4 报成立。
/// 不成立照报违例（[`Judgements::partially_judged`] 的契约：判出违例照报违例）。
///
/// 按版本那一层判的不只是候选根：由记录施加出来的那几版（[`versions_applied_only_by_records`]）与下一次挂载先施加的那一版
/// （[`version_the_next_mount_applies_first`]）也各判一版，树表指针与「树表 0 条那一版的分配记录树根」取记录新根段里的，判法与候选根同一套
/// （[`VersionToJudgeTheAllocationRecordsOf`]；
/// 「由记录施加出来的版本与下一次挂载先施加的那一版」那一行的改法 RV：这两类版本都进评估，后一类上两条记录罩住同一个槽，
/// 正是这一条要拦的「下一次可写挂载重建分配器时」那件事）。读不出的照上面几格报部分没判，点名的称呼说清是哪一类版本。
pub(super) fn judge_allocation_records_disjoint(
    reader: &dyn ImageReader,
    versions: &[VersionToJudgeTheAllocationRecordsOf<'_>],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    // 走到了叶的分配记录树根（按第一条位置条目认，与 `seen_allocation_nodes` 同一个键）：别的版本照抄同一个根时，
    // 它那一版的树也算走到了叶（那棵树只判一次）；先走的那一版读不出的，照抄它的那一版同样算没走到。
    let mut allocation_roots_walked_to_their_leaves: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    // 带树一版「根记录里树表 0 条那一版的分配记录树根全零」那一格：不成立当场照报违例；成立的那几版按走到的次序记下哪一版、
    // 这一版自己的分配记录树走没走到叶，走完全部候选根之后才逐版定计不计进评估（判决
    // `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「I-5.4『成立要至少一棵树走到叶』」那一行的零轮收严：
    // 按版本记，别的版本的树走到了叶不替这一版作保）。
    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<(String, bool)> =
        Vec::new();
    // 树表指针非零而树表读不出的那几版：认不出带不带树，「根记录里分配记录树根全零」那一格与这一版自己的分配记录树都没判。
    let mut versions_whose_tree_table_does_not_read_back: Vec<String> = Vec::new();
    // 树表指针全零的那几版：core 照它读盘 0 槽 0、读不出，与树表读不出同一处置。
    let mut versions_whose_tree_table_pointer_is_all_zero: Vec<String> = Vec::new();
    // 树表读得出、0 条，分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））而那棵树走不到叶的那几版：
    // 这一版自己的分配记录树没判（「I-5.4 按版本：坏盘」
    // 那一行改法 F1 的同形）。
    let mut versions_whose_root_record_allocation_record_tree_does_not_read_back: Vec<String> =
        Vec::new();
    for version in versions {
        let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;
        let mut this_version_holds_its_allocation_record_tree_in_the_root_record = false;
        // 这一版的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录（或记录新根段）直接持有的那一棵
        // （C512（树表 0 条的一版上被换下的实例表记在哪没有条款），代码审阅 6c 第 9 条：原来这一棵不读）。带文件的一版后者恒全零。
        let mut allocation_roots = vec![parse_node_pointer(
            version.allocation_record_tree_root_held_outside_the_tree_table,
        )];
        let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
            reader,
            &parse_node_pointer(version.mapping_root_pointer),
            cache,
        );
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            versions_whose_tree_table_pointer_is_all_zero.push(version.name.clone());
        } else {
            let tree_table_read_back = read_index_node_without_judging(
                reader,
                &tree_table_pointer,
                PositionOfASecondaryRead::ThePointerItself,
                cache,
            );
            if tree_table_read_back.is_none() {
                versions_whose_tree_table_does_not_read_back.push(version.name.clone());
            }
            if let Some(tree_table) = tree_table_read_back {
                this_version_holds_its_allocation_record_tree_in_the_root_record =
                    tree_table.entries.is_empty() && !allocation_roots[0].all_zero;
                if !tree_table.entries.is_empty() {
                    if allocation_roots[0].all_zero {
                        this_version_carries_trees_and_a_zero_root_record_allocation_tree_root =
                            true;
                    } else {
                        judgements.judge("I-5.4", false, || {
                            format!(
                                "{}的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
                                version.name,
                                tree_table.entries.len()
                            )
                        });
                    }
                }
                allocation_roots.extend(
                    tree_table
                        .entries
                        .iter()
                        .filter(|entry| {
                            entry.len() >= tree_table_entry_bytes()
                                && read_u16(entry, 10) == TREE_KIND_ALLOCATION
                        })
                        .map(|entry| parse_node_pointer(&entry[14..100])),
                );
            }
        }
        let mut this_version_allocation_record_tree_walked_to_its_leaves = false;
        for allocation_root in allocation_roots {
            if allocation_root.all_zero {
                continue;
            }
            // 「同一棵树」按映射认过的落点认：同一条过期提示在搬迁前后两版里指的是同一个单元，认过之后落在同一处。
            let located_allocation_root = central_mapping.locate(
                &allocation_root,
                PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
            );
            let first_location = &located_allocation_root.locations[0];
            let allocation_root_key = (
                first_location.device,
                first_location.slot,
                first_location.checksum,
            );
            if !seen_allocation_nodes.insert(allocation_root_key) {
                if allocation_roots_walked_to_their_leaves.contains(&allocation_root_key) {
                    this_version_allocation_record_tree_walked_to_its_leaves = true;
                }
                continue;
            }
            let Some((_, records)) = allocation_record_tree_without_judging(
                reader,
                &allocation_root,
                &central_mapping,
                cache,
            ) else {
                continue;
            };
            judge_allocation_record_ranges(&records, &version.name, judgements);
            allocation_roots_walked_to_their_leaves.insert(allocation_root_key);
            this_version_allocation_record_tree_walked_to_its_leaves = true;
        }
        if this_version_carries_trees_and_a_zero_root_record_allocation_tree_root {
            versions_where_the_root_record_allocation_tree_root_is_zero.push((
                version.name.clone(),
                this_version_allocation_record_tree_walked_to_its_leaves,
            ));
        }
        if this_version_holds_its_allocation_record_tree_in_the_root_record
            && !this_version_allocation_record_tree_walked_to_its_leaves
        {
            versions_whose_root_record_allocation_record_tree_does_not_read_back
                .push(version.name.clone());
        }
    }
    let some_allocation_record_tree_walked_to_its_leaves =
        !allocation_roots_walked_to_their_leaves.is_empty();
    let mut versions_not_counted: Vec<&str> = Vec::new();
    for (version, this_version_allocation_record_tree_walked_to_its_leaves) in
        &versions_where_the_root_record_allocation_tree_root_is_zero
    {
        if *this_version_allocation_record_tree_walked_to_its_leaves {
            judgements.judge("I-5.4", true, String::new);
        } else {
            versions_not_counted.push(version);
        }
    }
    if !some_allocation_record_tree_walked_to_its_leaves {
        judgements.not_applicable(
            "I-5.4",
            "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上",
        );
    }
    // `partially_judged` 只留第一次给的说明：两种没判到的版本合成一句报。
    let mut unjudged_parts: Vec<String> = Vec::new();
    if !versions_not_counted.is_empty() {
        unjudged_parts.push(if some_allocation_record_tree_walked_to_its_leaves {
            format!(
                "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：{}",
                versions_not_counted.join("、")
            )
        } else {
            format!(
                "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：{}",
                versions_not_counted.join("、")
            )
        });
    }
    if !versions_whose_tree_table_does_not_read_back.is_empty() {
        unjudged_parts.push(format!(
            "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{}",
            versions_whose_tree_table_does_not_read_back.join("、")
        ));
    }
    if !versions_whose_tree_table_pointer_is_all_zero.is_empty() {
        unjudged_parts.push(format!(
            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{}",
            versions_whose_tree_table_pointer_is_all_zero.join("、")
        ));
    }
    if !versions_whose_root_record_allocation_record_tree_does_not_read_back.is_empty() {
        unjudged_parts.push(format!(
            "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：{}",
            versions_whose_root_record_allocation_record_tree_does_not_read_back.join("、")
        ));
    }
    if !unjudged_parts.is_empty() {
        judgements.partially_judged("I-5.4", unjudged_parts.join("；"));
    }
}

/// 一个起点槽上的单元头能不能用、能用时头里的诞生代号（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的偏移表：
/// 码 1 诞生代号 75、明文头末尾 105；码 2 诞生代号 52 + 2k、明文头末尾 86 + 2k；码 3 诞生代号 73、明文头末尾 107；
/// 头校验和三类都在 10）。「能用」= magic 对、类标签是 1 / 2 / 3、头校验和过——头读不出、类标签不认、头校验和不过的
/// 这一格没有对象（I-3.10 那一行射程 ③：读不出本身由 I-1.1 与 I-2.1 管）。
pub(super) fn birth_txg_in_a_usable_unit_header(header: &[u8]) -> Option<u64> {
    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
        return None;
    }
    let (plain_header_end, birth_txg_offset) = match header[6] {
        1 => (105, 75),
        2 => {
            let key_span = 2 * usize::from(header[51]);
            (86 + key_span, 52 + key_span)
        }
        3 => (107, 73),
        _outside_the_three_unit_classes => return None,
    };
    checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET)
        .then(|| read_u64(header, birth_txg_offset))
}

/// 下一次挂载的恢复会先施加的那一版的树表指针与中央映射根指针（改法 E，`research/prompts/m2-wave3-code-r1-main-verification.md`
/// 第三节 Y4 与第四节第 2 条，被攻过零轮）：那一版由 [`version_the_next_mount_applies_first`] 认，两条指针取它新根段里的
/// （D23（journal 的角色与格式） 已定项 4 的字段表）。I-3.10 经这里读，I-5.4 按版本那一层读同一个函数认出的同一版。
pub(super) fn tree_table_and_mapping_root_pointers_of_the_version_the_next_mount_applies_first(
    version_the_next_mount_applies_first: Option<&VersionTheNextMountAppliesFirst>,
) -> Option<(PointerView, PointerView)> {
    version_the_next_mount_applies_first.map(|version| {
        (
            parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER]),
            parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER]),
        )
    })
}

/// 下一次挂载的恢复会先施加的那一版：重放施加的第一次发布，根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
/// 那一类已被施加、实例表里有行）。
pub(super) struct VersionTheNextMountAppliesFirst {
    pub(super) instance: u32,
    pub(super) checkpoint_txg: u64,
    pub(super) new_root_segment: Vec<u8>,
}

/// 下一次挂载的恢复会先施加的那一版：它的 (实例代号, txg) 与那次发布末条记录的整段新根段。I-3.10 只取树表指针与中央映射根指针
/// （[`tree_table_and_mapping_root_pointers_of_the_version_the_next_mount_applies_first`]），I-5.4 按版本那一层还要「树表 0 条那一版的分配记录树根」；
/// 两条都经这一个函数认这一版，同一个条件。
///
/// 认法（改法 R1 与第三节第 1 条）：所选根取最新根
/// （与 I-4.1 同一个择法），照 I-4.1 那一份重放（[`replay_the_next_mount_would_do`]）施加的**第一次**发布、取它末条那一份新根段
/// （已定项 15：core 施加末条那一份，`crates/singlefs-core/src/recovery.rs` 拼新根那一段）。core 的施加单位是一次发布：
/// 从链首起连续走到末条才整次施加，末条没落、同一次发布前面某条没落（断号）、反向链对不上、序号或 txg 接不上、末条之后同一次发布还有记录、
/// 点名单元验不过的那次发布都不施加；这里用同一份重放，那次发布同样不认。
/// 重放一次发布都不施加、锚点认不定、所选根 txg 没有后继时没有这一版。前缀停在后面某次发布的点名单元上时，停之前整次收下的发布
/// core 照施加，第一次那一版照认（I-4.1 对这种镜像报不适用，那是 I-4.1 自己的射程）。这一版的 txg 比最新根大，根环里没有它的根。
///
/// **只接这一步**：恢复连着施加几次发布时，第二次及以后那几版的分配记录这里读不到；读它只多判 I-3.10 一格，
/// 它的分配记录与它罩住的单元是同一次发布写出来的，不论施加与否两个数都同源。
pub(super) fn version_the_next_mount_applies_first(
    reader: &dyn ImageReader,
    records_the_recovery_reader_reads_by_device: &[(u32, JournalRecordsOfADevice)],
    newest_root: &crate::RootView,
    journal_ring_bytes: u64,
) -> Option<VersionTheNextMountAppliesFirst> {
    let records_the_replay_reads =
        records_as_the_recovery_reads_them(records_the_recovery_reader_reads_by_device);
    let last_record_of_the_first_publish_applied = match replay_the_next_mount_would_do(
        reader,
        &records_the_replay_reads,
        newest_root,
        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),
    ) {
        ReplayTheNextMountWouldDo::AnchorUndecided
        | ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor
        | ReplayTheNextMountWouldDo::NothingApplied => None,
        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
            last_record_of_the_first_publish_applied_before_it,
        } => last_record_of_the_first_publish_applied_before_it,
        ReplayTheNextMountWouldDo::Applied {
            last_record_of_the_first_publish,
            publishes: _,
            records: _,
            last_record_counter: _,
            last_record: _,
        } => Some(last_record_of_the_first_publish),
    }?;
    Some(VersionTheNextMountAppliesFirst {
        instance: last_record_of_the_first_publish_applied.instance,
        checkpoint_txg: last_record_of_the_first_publish_applied.checkpoint_txg,
        new_root_segment: last_record_of_the_first_publish_applied
            .new_root_segment
            .clone(),
    })
}

/// I-3.10 要读的一棵分配记录树：它的根指针，与它那一版的中央映射（分配记录树的节点不在豁免三类里，按那一版的映射认位置；
/// 几版共用同一份映射根时各读一份，读不出时是空表，照提示读）。
pub(super) struct AllocationRecordTreeOfAVersion {
    pub(super) root_pointer: PointerView,
    pub(super) central_mapping: std::rc::Rc<CentralMappingLocationsOfAVersion>,
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）读哪几片分配记录：候选集里每条根的分配记录树
/// （树表里种类 3 那一条指着的根），树表 0 条那一版的分配记录树（根记录直接持有，C512（树表 0 条的一版上被换下的实例表记在哪没有条款）），
/// 由记录施加出来的那几版的分配记录树（它们的树表在记录的新根段里），以及下一次挂载会先施加的那一版的分配记录树
/// （[`tree_table_and_mapping_root_pointers_of_the_version_the_next_mount_applies_first`]）。每一棵带上它那一版的中央映射。
/// 只读不判：判定归 I-2.1 / I-1.1 那几条的走读。
pub(super) fn allocation_record_trees_of_the_candidate_versions(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    versions_applied_only_by_records: &[VersionAppliedOnlyByRecords],
    tree_table_and_mapping_root_pointers_of_the_version_the_next_mount_applies_first: Option<(
        PointerView,
        PointerView,
    )>,
    cache: &mut IndexNodeCache,
) -> Vec<AllocationRecordTreeOfAVersion> {
    // 每一版：树表指针、中央映射根指针、根记录直接持有的分配记录树根（记录施加出来的两类没有这一项）。
    let mut versions: Vec<(PointerView, PointerView, Option<PointerView>)> = Vec::new();
    for index in candidate_indexes.iter().copied() {
        let record = &roots[index].2.record_bytes;
        versions.push((
            parse_node_pointer(&record[36..122]),
            parse_node_pointer(&record[ROOT_RECORD_MAPPING_ROOT_POINTER]),
            Some(parse_node_pointer(&record[342..428])),
        ));
    }
    for version in versions_applied_only_by_records {
        let (tree_table_pointer, mapping_root_pointer) =
            version.tree_table_and_mapping_root_pointers();
        versions.push((
            parse_node_pointer(tree_table_pointer),
            parse_node_pointer(mapping_root_pointer),
            None,
        ));
    }
    versions.extend(
        tree_table_and_mapping_root_pointers_of_the_version_the_next_mount_applies_first.map(
            |(tree_table_pointer, mapping_root_pointer)| {
                (tree_table_pointer, mapping_root_pointer, None)
            },
        ),
    );
    let mut allocation_record_trees: Vec<AllocationRecordTreeOfAVersion> = Vec::new();
    for (tree_table_pointer, mapping_root_pointer, root_held_by_the_root_record) in versions {
        let central_mapping =
            std::rc::Rc::new(CentralMappingLocationsOfAVersion::read_without_judging(
                reader,
                &mapping_root_pointer,
                cache,
            ));
        let mut root_pointers: Vec<PointerView> =
            root_held_by_the_root_record.into_iter().collect();
        if !tree_table_pointer.all_zero {
            if let Some(tree_table) = read_index_node_without_judging(
                reader,
                &tree_table_pointer,
                PositionOfASecondaryRead::ThePointerItself,
                cache,
            ) {
                for entry in &tree_table.entries {
                    if entry.len() < tree_table_entry_bytes()
                        || read_u16(entry, 10) != TREE_KIND_ALLOCATION
                    {
                        continue;
                    }
                    root_pointers.push(parse_node_pointer(&entry[14..100]));
                }
            }
        }
        allocation_record_trees.extend(
            root_pointers
                .into_iter()
                .filter(|root_pointer| !root_pointer.all_zero)
                .map(|root_pointer| AllocationRecordTreeOfAVersion {
                    root_pointer,
                    central_mapping: std::rc::Rc::clone(&central_mapping),
                }),
        );
    }
    allocation_record_trees
}

/// I-3.10 读不出一棵分配记录树（节点按那一版的映射与提示都读不出、层级或位置对不上）时报的射程缺口：那棵树里的记录一条都没判，
/// 别的树判到了也不报成立（[`Judgements::part_of_the_range_not_judged`]）。
pub const ALLOCATION_GENERATIONS_OF_AN_UNREADABLE_ALLOCATION_RECORD_TREE_ARE_NOT_JUDGED: &str = "候选集里有一棵分配记录树读不全（节点按那一版的中央映射与提示都读不出，或层级、位置对不上；读不出本身归 I-1.1 与 I-2.1）：那棵树里未释放记录的分配代一条都没判";

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）：任一**未带已释放标志**的分配记录，它的分配代等于它罩住的那个单元头里的
/// 诞生代号——记录与单元是同一次发布写出来的，两个数同源（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.10 那一行）。射程照那一行：
/// ① 只管未释放的记录（已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里））；
/// ② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头——记录的 key 就是起点槽，按它读；
/// ③ 那个槽上读不出可用的单元头时这一格没有对象，不判（[`birth_txg_in_a_usable_unit_header`]）。
/// 它拦的是：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着。
/// 几条根指着同一个分配记录树节点时（照抄上一版的根）那个节点只读一次（按那一版的映射认过的落点认）；同一条记录（盘、槽、代）出现在几片里只判一次。
/// 一棵树读不全时记射程里判不了（[`ALLOCATION_GENERATIONS_OF_AN_UNREADABLE_ALLOCATION_RECORD_TREE_ARE_NOT_JUDGED`]），不跳过了事：
/// 跳过的话别的树判到了就报成立，那棵树里的记录无痕漏判。
/// 一格都没判到时整条报不适用并带理由，不报成立。
pub(super) fn judge_allocation_generations_against_unit_births(
    reader: &dyn ImageReader,
    allocation_record_trees: &[AllocationRecordTreeOfAVersion],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    let mut examined_records: BTreeSet<(u32, u64, u64)> = BTreeSet::new();
    let mut judged_records = 0u64;
    for allocation_record_tree in allocation_record_trees {
        let located_root = allocation_record_tree.central_mapping.locate(
            &allocation_record_tree.root_pointer,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        );
        let first_location = &located_root.locations[0];
        if !seen_allocation_nodes.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
        )) {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(
            reader,
            &allocation_record_tree.root_pointer,
            &allocation_record_tree.central_mapping,
            cache,
        ) else {
            judgements.part_of_the_range_left_unjudged(
                "I-3.10",
                ALLOCATION_GENERATIONS_OF_AN_UNREADABLE_ALLOCATION_RECORD_TREE_ARE_NOT_JUDGED
                    .to_string(),
            );
            continue;
        };
        for record in records {
            if record.is_released
                || !examined_records.insert((record.device, record.slot, record.generation))
            {
                continue;
            }
            let Some(birth_txg) = reader
                .read(
                    record.device,
                    record.slot * SLOT_BYTES,
                    UNIT_HEADER_SCAN_BYTES,
                )
                .as_deref()
                .and_then(birth_txg_in_a_usable_unit_header)
            else {
                continue;
            };
            judged_records += 1;
            judgements.judge_naming_the_slots(
                "I-3.10",
                record.generation == birth_txg,
                || vec![(record.device, record.slot)],
                || {
                    format!(
                        "盘 {} 槽 {} 的分配记录（未释放、跨 {} 槽）分配代 {}，它罩住的单元头里的诞生代号是 {birth_txg}",
                        record.device, record.slot, record.span_slots, record.generation
                    )
                },
            );
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            "I-3.10",
            if examined_records.is_empty() {
                "候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）"
            } else {
                "未释放的分配记录罩住的起点槽上一个可用的单元头都读不出：读不出本身归 I-1.1 与 I-2.1"
            },
        );
    }
}

/// 一次发布的记录有没有点名单元（I-5.7 的前件，取自记录自述的点名项：同一 (实例, txg) 的记录里有一条点名项非空就算）。
pub(super) fn publish_names_some_unit(
    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    instance: u32,
    checkpoint_txg: u64,
) -> bool {
    records_as_the_recovery_reads_them
        .range((instance, u64::MIN)..=(instance, u64::MAX))
        .any(|(_, record)| {
            record.checkpoint_txg == checkpoint_txg && !record.named_entries.is_empty()
        })
}

/// I-5.7 判的一版：由记录施加出来、根槽从没落盘的那一版，或下一次挂载先施加的那一版。
pub(super) struct VersionToJudgeTheAllocationRecordTreeRootBirthOf<'record> {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    /// 新根段里「树表 0 条那一版的分配记录树根指针」那 86 字节。
    pub(super) allocation_record_tree_root_held_outside_the_tree_table: &'record [u8],
    /// 这次发布的记录点名项非空（[`publish_names_some_unit`]）。
    pub(super) publish_names_some_unit: bool,
}

/// [`ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION`]（I-5.7，C706）：
/// 每一版点名项非空、新根段里那条指针不全零时，
/// 指针的出生 txg 等于这一版的 txg。写者依据：树表 0 条那一版上写过单元的发布每次重建分配记录树（`crates/singlefs-core/src/transaction.rs`
/// 的 `prepare_the_row_publish_on_a_version_without_file`），零单元发布照抄上一版那条指针、点名项为空。
/// 前件取自记录、不取自被比的那条指针，指针整条换成一棵合法的旧树时前件照样成立。
pub(super) fn judge_allocation_record_tree_roots_born_with_the_versions_applied_only_by_records(
    versions: &[VersionToJudgeTheAllocationRecordTreeRootBirthOf<'_>],
    judgements: &mut Judgements,
) {
    let invariant =
        ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION;
    for version in versions {
        let pointer =
            parse_node_pointer(version.allocation_record_tree_root_held_outside_the_tree_table);
        if pointer.all_zero || !version.publish_names_some_unit {
            continue;
        }
        judgements.judge(
            invariant,
            pointer.birth_txg == version.checkpoint_txg,
            || {
                format!(
                "{}：记录点名项非空，新根段里分配记录树根指针的出生 txg 是 {}，不是这一版的 txg {}",
                version.name, pointer.birth_txg, version.checkpoint_txg
            )
            },
        );
    }
    judgements.not_applicable(
        invariant,
        NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE,
    );
}

#[cfg(test)]
mod tests;
