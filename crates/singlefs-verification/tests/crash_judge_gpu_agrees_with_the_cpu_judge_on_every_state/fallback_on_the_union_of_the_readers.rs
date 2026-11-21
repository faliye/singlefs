//! GPU 恢复内核判退看三样读者的并集（建上一版、只读挂载的打开、实例表），判退的两样读者从所选根那一段起的状态。

use super::*;

// ─── 里程碑三收尾（C755 的 GPU 一侧，F6）：GPU 恢复内核判退看三样读者的并集 ───

/// 去掉流里的几次写：写表按剩下的写重新编号，切段跟着改、空了的段不留，被判的根槽写跟着挪（它自己不许被去掉）。
fn drop_writes(flow: &mut RecordedFlow, dropped: &[usize]) {
    assert!(
        !dropped.contains(&flow.judged_root_index),
        "流 {}：被判的根槽写不去掉",
        flow.name
    );
    let index_after_the_drop =
        |index: usize| index - dropped.iter().filter(|gone| **gone < index).count();
    flow.segments = flow
        .segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .filter(|index| !dropped.contains(index))
                .map(|index| index_after_the_drop(*index))
                .collect::<Vec<usize>>()
        })
        .filter(|segment| !segment.is_empty())
        .collect();
    flow.judged_root_index = index_after_the_drop(flow.judged_root_index);
    let mut write_index = 0;
    flow.writes.retain(|_write| {
        let kept = !dropped.contains(&write_index);
        write_index += 1;
        kept
    });
}

/// 两次覆盖写那条流（新池新建文件 V⁻ txg 3 → 覆盖写成第二份内容 V txg 4 → 再覆盖写成第三份内容 R txg 5，同一个进程、实例 1）原样录下来，
/// 连同它的三版：R 的根槽落了而 R 那次系统配置轮换没落的状态上系统配置见证到 V、没见证 R。
pub(super) fn record_two_overwrites(tag: &str, name: &'static str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content());
    let flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(5),
                content: third_content(),
            },
        ],
    );
    assert_eq!(
        root_write_of_txg(&flow, JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.1),
        flow.judged_root_index,
        "流里最后一条根槽写是 R 的"
    );
    flow
}

/// R 那次发布在流里的写：V 的根槽写之后、R 的根槽写之前。
fn writes_of_the_judged_publish(flow: &RecordedFlow) -> std::ops::Range<usize> {
    root_write_of_txg(flow, CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.1) + 1..flow.judged_root_index
}

/// 两次覆盖写那条流上：R 写出的那个数据单元两份都不落盘（流里去掉那两次单元写：R 的根槽落了的状态上冷走读读不出它、经映射仍读不出——字节丢了），
/// R 那条记录第一个点名项的两条位置条目对调成按盘降序、重封（扫环照收；建上一版判记录的位置条目按盘升序，不收——内容拒收）。
/// 只读挂载的打开不读数据单元、不读记录，读得通；沿链读 R 的实例表读得通。
fn record_two_overwrites_whose_second_data_unit_is_lost_and_whose_second_record_the_rebuild_refuses(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites(tag, name);
    let data_unit_writes_of_the_judged_publish: Vec<usize> = writes_of_the_judged_publish(&flow)
        .filter(|index| {
            flow.writes[*index].kind == StepKind::UnitWrite
                && flow.writes[*index]
                    .bytes()
                    .is_some_and(|bytes| unit_class(bytes) == UNIT_CLASS_DATA)
        })
        .collect();
    assert_eq!(
        data_unit_writes_of_the_judged_publish.len(),
        2,
        "前提：R 只写了一个数据单元、两份"
    );
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let named_units = u32::from_le_bytes(bytes[12..16].try_into().expect("4"));
        assert!(named_units >= 1, "前提：R 的记录点名了单元");
        let first_location = JOURNAL_RECORD_HEADER_BYTES;
        let second_location = first_location + LOCATION_ENTRY_BYTES_IN_A_NAMED_UNIT;
        let device_of =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"));
        assert!(
            device_of(first_location) < device_of(second_location),
            "前提：写者按盘升序写点名项的位置条目"
        );
        let (first_part, second_part) = bytes.split_at_mut(second_location);
        first_part[first_location..second_location]
            .swap_with_slice(&mut second_part[..LOCATION_ENTRY_BYTES_IN_A_NAMED_UNIT]);
    });
    drop_writes(&mut flow, &data_unit_writes_of_the_judged_publish);
    flow
}

/// journal 记录点名项里一条位置条目的宽：盘 4 + 槽 6 + 整单元校验和 4。
const LOCATION_ENTRY_BYTES_IN_A_NAMED_UNIT: usize = 14;

/// 打包记录单元里第一条记录在单元里的偏移（码 3 头 136 字节）与 inode 记录里恒零的那一段的起点（偏移 108，`InodeRecord::parse` 判非零拒收）。
const FIRST_RECORD_OF_A_PACKED_UNIT: usize = 136;

const INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD: usize = 108;

/// 两次覆盖写那条流上：R 写出的 extent 树根两份都不落盘（冷走读与建上一版读 extent 树那一步读不出、经映射仍读不出——字节丢了，两样都停在那里；
/// extent 树在树表里排第一，冷走读读它之前不读别的树），R 的 inode 叶容器第一条记录偏移 108 那一段恒零区写成非零、重封到根
/// （只读挂载的打开不读 extent 树，读到这条记录拒收——内容拒收）。沿链读 R 的实例表读得通。
fn record_two_overwrites_whose_second_extent_root_is_lost_and_whose_second_inode_record_the_opening_refuses(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites(tag, name);
    let inode_leaf_of_the_judged_publish = last_unit_copies(
        &flow,
        "inode 叶容器",
        &|bytes| unit_class(bytes) == 3 && packed_record_type(bytes) == 2,
        false,
    );
    assert!(
        inode_leaf_of_the_judged_publish
            .iter()
            .all(|index| writes_of_the_judged_publish(&flow).contains(index)),
        "前提：流里最后那个 inode 叶容器是 R 写的"
    );
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &inode_leaf_of_the_judged_publish,
        &|bytes| {
            bytes[FIRST_RECORD_OF_A_PACKED_UNIT
                + INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD] = 1;
        },
    );
    let extent_root_of_the_judged_publish = last_unit_copies(
        &flow,
        "extent 树根",
        &|bytes| unit_class(bytes) == 2 && index_node_key_width(bytes) == 24,
        false,
    );
    assert!(
        extent_root_of_the_judged_publish
            .iter()
            .all(|index| writes_of_the_judged_publish(&flow).contains(index)),
        "前提：流里最后那个 extent 树节点是 R 写的"
    );
    drop_writes(&mut flow, &extent_root_of_the_judged_publish);
    flow
}

/// 数 CPU 看 journal 那一遍落在 R、结局读失败的状态（判退看并集的流上不退的那几个状态）。
fn states_staying_on_the_judged_publish() -> StateExamination {
    StateExamination::CountStatesWhere(consulted_recovery_lands_on(
        JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS,
        RECOVERY_WORD_CLASS_FAILED,
    ))
}

/// 一条判退看并集的流比完之后核两样：至少一个状态上 CPU 看 journal 那一遍落在 R、结局读失败（并集里有内容拒收，不退到 V；对拍时只交了
/// [`states_staying_on_the_judged_publish`] 那一样考察）；每个状态对拍字逐位相同。
fn assert_the_gpu_falls_back_on_the_union_like_the_cpu(
    flow: &RecordedFlow,
    comparison: &Comparison,
) {
    let states_staying_on_the_judged_publish = comparison.examined[0].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_FALL_BACK_UNION flow={} states={} states_staying_on_r={states_staying_on_the_judged_publish}",
        flow.name, comparison.states
    );
    assert!(
        states_staying_on_the_judged_publish > 0,
        "流 {}：R 的根槽落了而系统配置没见证 R 的状态上，三样读者的并集里有内容拒收，CPU 看 journal 那一遍不退、落在 R {JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS:?} 报读不出",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(comparison)
    );
}

/// 场景：两次覆盖写那条流，R 写出的数据单元两份都没落、R 的记录第一个点名项的位置条目按盘降序（重封过）
/// （[`record_two_overwrites_whose_second_data_unit_is_lost_and_whose_second_record_the_rebuild_refuses`]）的每个状态。
/// R 的根槽落了而系统配置没见证 R 的状态上，冷走读字节丢了、建上一版拒收 R 的记录、打开读得通：并集里有内容拒收，CPU 恢复不退、报 R 读不出。
/// 预期：GPU 恢复内核判退看同一个并集（`crash_judge_recovery.wgsl` 的 `selected_version_under_the_three_readers`），每个状态对拍字逐位相同。
/// 判别力：只看走读的 GPU 在这几个状态上退到 V、与 CPU 分家；`crates/mutations.tsv` 里「GPU 判退并集漏了建上一版那一样」那一行让这一条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_walk_lost_bytes_and_whose_rebuild_refused_content_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_walk_lost_bytes_and_whose_rebuild_refused_content_is_not_fallen_back_from";
    let flow = record_two_overwrites_whose_second_data_unit_is_lost_and_whose_second_record_the_rebuild_refuses(
        "crash-judge-gpu-fall-back-union-rebuild-refuses",
        "two-overwrites-second-data-unit-lost-and-second-record-refused-by-the-rebuild",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：两次覆盖写那条流，R 写出的 extent 树根两份都没落、R 的 inode 叶容器第一条记录恒零区写成非零（重封到根）
/// （[`record_two_overwrites_whose_second_extent_root_is_lost_and_whose_second_inode_record_the_opening_refuses`]）的每个状态。
/// R 的根槽落了而系统配置没见证 R 的状态上，冷走读与建上一版都停在 extent 树根上（字节丢了）、只读挂载的打开拒收那条 inode 记录：
/// 并集里有内容拒收，CPU 恢复不退、报 R 读不出。
/// 预期：GPU 恢复内核判退看同一个并集，每个状态对拍字逐位相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 判退并集漏了只读挂载打开那一样」那一行让这一条红（漏了打开，并集只剩字节丢了，GPU 退到 V）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_walk_lost_bytes_and_whose_opening_refused_content_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_walk_lost_bytes_and_whose_opening_refused_content_is_not_fallen_back_from";
    let flow = record_two_overwrites_whose_second_extent_root_is_lost_and_whose_second_inode_record_the_opening_refuses(
        "crash-judge-gpu-fall-back-union-opening-refuses",
        "two-overwrites-second-extent-root-lost-and-second-inode-record-refused-by-the-opening",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

// ─── 代码轮 m4-close-code-r3（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 8 条）：判退的两样读者（只读挂载的打开、
// 可写挂载的建上一版）补的叶序与分隔 key 那一判（I-9.12 第三句），各一条只有它拒收内容的流 ───

/// 两片 inode 叶那条流（新池新建文件 V⁻ txg 3 → 建 233 个空 inode，叶 [1..=233] 与 [234]，V txg 4）之后，同一个进程、实例 1 里把第一个文件
/// 覆盖写成第二份内容（R，txg 5），整条录下来；R 写出的 inode 树根（左片那条记录变了，根跟着重写）右片那条条目的分隔 key
/// 234 → [`SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_LOWERED`]、重封到根：记录照叶序严格升序、都不小于分隔 key 与容器号，读完右片判
/// 「分隔 key 大于前面那几片的最大记录号」不成立（内容拒收）。冷走读、只读挂载的打开、可写挂载的建上一版三样读者读全部叶，读到右片都拒。
fn record_two_inode_leaves_then_an_overwrite_whose_inode_root_lowers_the_separator_key_of_the_right_leaf(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    record_two_inode_leaves_then_an_overwrite_whose_inode_root_names_this_separator_key_of_the_right_leaf(
        tag,
        name,
        SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_LOWERED,
    )
}

/// 新池新建文件之外再建 233 个空 inode（两片 inode 叶），覆盖写成第二份内容（R，txg 5），整条录下来，什么都不改。
fn record_two_inode_leaves_then_an_overwrite(tag: &str, name: &'static str) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
    let flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(5),
                content: second_content(),
            },
        ],
    );
    assert_eq!(
        root_write_of_txg(&flow, JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.1),
        flow.judged_root_index,
        "流里最后一条根槽写是 R 的"
    );
    flow
}

/// 同 [`record_two_inode_leaves_then_an_overwrite_whose_inode_root_lowers_the_separator_key_of_the_right_leaf`]，右片那条条目的分隔 key 改成 `separator_key`。
fn record_two_inode_leaves_then_an_overwrite_whose_inode_root_names_this_separator_key_of_the_right_leaf(
    tag: &str,
    name: &'static str,
    separator_key: u64,
) -> RecordedFlow {
    let mut flow = record_two_inode_leaves_then_an_overwrite(tag, name);
    let inode_tree_root_over_two_leaf_containers = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
            && index_node_entry_count(bytes) == 2
            && read_u16_at(
                bytes,
                index_node_entries_start(INODE_TREE_KEY_BYTES)
                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
            ) == PACKED_RECORD_TYPE_INODE
    };
    let inode_root_of_the_judged_publish = last_unit_copies(
        &flow,
        name,
        &inode_tree_root_over_two_leaf_containers,
        false,
    );
    assert!(
        inode_root_of_the_judged_publish
            .iter()
            .all(|index| writes_of_the_judged_publish(&flow).contains(index)),
        "前提：流里最后那个两条条目的 inode 树根是 R 写的"
    );
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &inode_root_of_the_judged_publish,
        &|bytes| {
            let separator_of_the_right_leaf =
                first_entry_of_a_container_number_node() + usize::from(INODE_INTERNAL_ENTRY_BYTES);
            assert_eq!(
                read_u64_at(bytes, separator_of_the_right_leaf),
                INODE_RECORDS_PER_LEAF + 1,
                "改之前右片那条条目的分隔 key 是它的容器号 234"
            );
            bytes[separator_of_the_right_leaf..separator_of_the_right_leaf + 8]
                .copy_from_slice(&separator_key.to_le_bytes());
        },
    );
    flow
}

/// R 那次发布在流里写出的、`select` 认得的全部单元写（每个单元两份）。
fn unit_writes_of_the_judged_publish(
    flow: &RecordedFlow,
    select: &dyn Fn(&[u8]) -> bool,
) -> Vec<usize> {
    writes_of_the_judged_publish(flow)
        .filter(|index| {
            flow.writes[*index].kind == StepKind::UnitWrite
                && flow.writes[*index].bytes().is_some_and(select)
        })
        .collect()
}

/// [`record_two_inode_leaves_then_an_overwrite_whose_inode_root_lowers_the_separator_key_of_the_right_leaf`] 那条流上，R 写出的 extent 树根两份都不落盘
/// （流里去掉那两次单元写）：R 的根槽落了的状态上冷走读与建上一版读 extent 树那一步读不出、经映射仍读不出（字节丢了，两样都停在那里，
/// 都走不到 inode 树）；只读挂载的打开不读 extent 树，读到右片拒收（内容拒收）；沿链读 R 的实例表读得通。
fn record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_lowered_separator_key_only_the_opening_reaches(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow =
        record_two_inode_leaves_then_an_overwrite_whose_inode_root_lowers_the_separator_key_of_the_right_leaf(
            tag, name,
        );
    drop_the_extent_root_written_by_the_judged_publish(&mut flow);
    flow
}

/// 同 [`record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_lowered_separator_key_only_the_opening_reaches`]，右片那条条目的分隔 key 改高到
/// [`SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_RAISED`]：比左片最大号大（I-9.12 第三句不红）、比右片头一条记录号大（I-9.4 第一句红，内容拒收）。
/// 只读挂载的打开读到右片头一条就拒（`crash_judge_recovery.wgsl` 打开那一样的记录号下界那一判）。
fn record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_raised_separator_key_only_the_opening_reaches(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow =
        record_two_inode_leaves_then_an_overwrite_whose_inode_root_names_this_separator_key_of_the_right_leaf(
            tag,
            name,
            SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_RAISED,
        );
    drop_the_extent_root_written_by_the_judged_publish(&mut flow);
    flow
}

/// 流上 R 写出的 extent 树根两份都不落盘（流里去掉那两次单元写）。
fn drop_the_extent_root_written_by_the_judged_publish(flow: &mut RecordedFlow) {
    let extent_root_of_the_judged_publish = last_unit_copies(
        &*flow,
        "extent 树根",
        &|bytes| unit_class(bytes) == 2 && index_node_key_width(bytes) == 24,
        false,
    );
    assert!(
        extent_root_of_the_judged_publish
            .iter()
            .all(|index| writes_of_the_judged_publish(&*flow).contains(index)),
        "前提：流里最后那个 extent 树节点是 R 写的"
    );
    drop_writes(flow, &extent_root_of_the_judged_publish);
}

/// 中央映射树的节点（码 2、key 宽 27）与记账树的节点（码 2、key 宽 [`ACCOUNTING_KEY_BYTES`]）。
fn is_a_central_mapping_node(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2 && index_node_key_width(bytes) == 27
}

fn is_an_accounting_node(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2 && usize::from(index_node_key_width(bytes)) == ACCOUNTING_KEY_BYTES
}

/// [`record_two_inode_leaves_then_an_overwrite_whose_inode_root_lowers_the_separator_key_of_the_right_leaf`] 那条流上，R 写出的中央映射树节点与记账树节点
/// 都不落盘（流里去掉它们的单元写）。R 的根槽落了的状态上：只读挂载的打开头一步整棵读映射树、读不出（字节丢了）；冷走读读 extent 树之后读记账树，
/// 提示读不出、要经映射回退时映射树读不出（字节丢了），走不到 inode 树；可写挂载的建上一版按 extent 树、数据单元、inode 树、删除与容器树、
/// 分配记录树、记账树、映射树的次序读，提示都读得出的那几样读完，先走到 inode 树、读到右片拒收（内容拒收），走不到记账树与映射树；沿链读 R 的实例表读得通。
fn record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_lowered_separator_key_only_the_rebuild_reaches(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow =
        record_two_inode_leaves_then_an_overwrite_whose_inode_root_lowers_the_separator_key_of_the_right_leaf(
            tag, name,
        );
    drop_the_mapping_and_accounting_nodes_written_by_the_judged_publish(&mut flow);
    flow
}

/// 同 [`record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_lowered_separator_key_only_the_rebuild_reaches`]，右片那条条目的分隔 key 改高到
/// [`SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_RAISED`]：可写挂载的建上一版读到右片头一条就拒（I-9.4 第一句，`crash_judge_recovery.wgsl` 建上一版那一样的记录号下界那一判）。
fn record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_raised_separator_key_only_the_rebuild_reaches(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow =
        record_two_inode_leaves_then_an_overwrite_whose_inode_root_names_this_separator_key_of_the_right_leaf(
            tag,
            name,
            SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_RAISED,
        );
    drop_the_mapping_and_accounting_nodes_written_by_the_judged_publish(&mut flow);
    flow
}

/// 流上 R 写出的中央映射树节点与记账树节点都不落盘（流里去掉它们的单元写）。
fn drop_the_mapping_and_accounting_nodes_written_by_the_judged_publish(flow: &mut RecordedFlow) {
    let mapping_nodes_of_the_judged_publish =
        unit_writes_of_the_judged_publish(&*flow, &is_a_central_mapping_node);
    let accounting_nodes_of_the_judged_publish =
        unit_writes_of_the_judged_publish(&*flow, &is_an_accounting_node);
    assert!(
        mapping_nodes_of_the_judged_publish.len() >= 2
            && mapping_nodes_of_the_judged_publish.len().is_multiple_of(2),
        "前提：R 写了中央映射树节点、每个两份：{mapping_nodes_of_the_judged_publish:?}"
    );
    assert!(
        accounting_nodes_of_the_judged_publish.len() >= 2
            && accounting_nodes_of_the_judged_publish
                .len()
                .is_multiple_of(2),
        "前提：R 写了记账树节点、每个两份：{accounting_nodes_of_the_judged_publish:?}"
    );
    let mut dropped: Vec<usize> = mapping_nodes_of_the_judged_publish
        .into_iter()
        .chain(accounting_nodes_of_the_judged_publish)
        .collect();
    dropped.sort_unstable();
    drop_writes(flow, &dropped);
}

/// 映射条目的宽（key 27 + 两条位置条目 28）。
const MAPPING_ENTRY_BYTES_IN_THE_LEAF: usize = 55;

/// 映射 key 里码 2 尾段（出生序号 4 字节）的偏移：类标签 1 + 出生树 8 + 出生 txg 8 + 实例 4。
const MAPPING_KEY_BIRTH_SEQUENCE_OFFSET: usize = 21;

/// 映射 key 里出生树那一段（类标签之后 8 字节）。
const MAPPING_KEY_BIRTH_TREE: std::ops::Range<usize> = 1..9;

/// R 写出的映射树根兼叶上挑一条码 2 的 key，它后面那一条是别的树的（出生序号加一之后仍在它前面、不撞别的 key）：出生序号加一、重封到根；R 的数据单元两份都不落盘。
/// R 的根槽落了的状态上：冷走读读文件内容那一步字节丢了；只读挂载的打开不读数据单元，读通；可写挂载的建上一版数据单元读不出不算（经映射也读不出照抄），
/// 读完全部树，映射 key 集合对不上这一版进映射的单元（内容拒收）。并集里有内容拒收，CPU 恢复不退。
fn record_two_inode_leaves_whose_overwrite_lost_its_data_unit_and_whose_replaced_mapping_key_only_the_rebuild_reaches(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_inode_leaves_then_an_overwrite(tag, name);
    let mapping_root_of_the_judged_publish =
        last_unit_copies(&flow, name, &is_a_central_mapping_node, false);
    assert!(
        mapping_root_of_the_judged_publish
            .iter()
            .all(|index| writes_of_the_judged_publish(&flow).contains(index)),
        "前提：流里最后那个中央映射树节点是 R 写的"
    );
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &mapping_root_of_the_judged_publish,
        &|bytes| {
            let entries_start = index_node_entries_start(27);
            let entries = usize::from(index_node_entry_count(bytes));
            let key_at =
                |position: usize| entries_start + position * MAPPING_ENTRY_BYTES_IN_THE_LEAF;
            let position_of_the_key = (0..entries - 1)
                .find(|position| {
                    let key = key_at(*position);
                    let next_key = key_at(*position + 1);
                    bytes[key] == 2
                        && bytes[next_key] == 2
                        && bytes
                            [key + MAPPING_KEY_BIRTH_TREE.start..key + MAPPING_KEY_BIRTH_TREE.end]
                            != bytes[next_key + MAPPING_KEY_BIRTH_TREE.start
                                ..next_key + MAPPING_KEY_BIRTH_TREE.end]
                })
                .expect("前提：映射树根里有一条码 2 的 key，后面那一条是别的树的");
            let birth_sequence_at = key_at(position_of_the_key) + MAPPING_KEY_BIRTH_SEQUENCE_OFFSET;
            let birth_sequence = u32::from_le_bytes(
                bytes[birth_sequence_at..birth_sequence_at + 4]
                    .try_into()
                    .expect("四字节"),
            );
            bytes[birth_sequence_at..birth_sequence_at + 4]
                .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
        },
    );
    let data_unit_of_the_judged_publish =
        unit_writes_of_the_judged_publish(&flow, &|bytes| unit_class(bytes) == 1);
    assert_eq!(
        data_unit_of_the_judged_publish.len(),
        2,
        "前提：R 写了一个数据单元、两份：{data_unit_of_the_judged_publish:?}"
    );
    drop_writes(&mut flow, &data_unit_of_the_judged_publish);
    flow
}

/// 场景：两片 inode 叶之后覆盖写 R，R 的 inode 树根右片分隔 key 改低（重封到根）、R 的 extent 树根两份都没落
/// （[`record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_lowered_separator_key_only_the_opening_reaches`]）的每个状态。
/// R 的根槽落了而系统配置没见证 R 的状态上，冷走读与建上一版停在 extent 树根上（字节丢了）、只读挂载的打开读到右片拒收（叶序与分隔 key 那一判）：
/// 并集里有内容拒收，CPU 恢复不退、报 R 读不出。
/// 预期：GPU 恢复内核的打开那一样（`crash_judge_recovery.wgsl` 的 `open_reader_kind`）同判，每个状态对拍字逐位相同。
/// 判别力：这条流全量域约两千万态，变异行不点名这一条：`crates/mutations.tsv` 里「GPU 判退坏叶流：打开那一样不判叶序与分隔 key」那一行点名
/// [`the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from`]，
/// 同一条流上抓得到它的那几个状态都在那一条里。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_lowered_separator_key_only_the_opening_reaches(
        "crash-judge-gpu-fall-back-union-opening-refuses-the-separator-key",
        "two-inode-leaves-overwrite-extent-root-lost-and-separator-key-refused-by-the-opening",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：两片 inode 叶之后覆盖写 R，R 的 inode 树根右片分隔 key 改低（重封到根）、R 的中央映射树节点与记账树节点都没落
/// （[`record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_lowered_separator_key_only_the_rebuild_reaches`]）的每个状态。
/// R 的根槽落了而系统配置没见证 R 的状态上，只读挂载的打开停在映射树上、冷走读停在记账树上（都是字节丢了）、可写挂载的建上一版读到右片拒收
/// （叶序与分隔 key 那一判）：并集里有内容拒收，CPU 恢复不退、报 R 读不出。
/// 预期：GPU 恢复内核的建上一版那一样（`crash_judge_recovery.wgsl` 的 `rebuild_reader_kind`）同判，每个状态对拍字逐位相同。
/// 判别力：这条流全量域约两千万态，变异行不点名这一条：`crates/mutations.tsv` 里「GPU 判退坏叶流：建上一版那一样不判叶序与分隔 key」那一行点名
/// [`the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from`]，
/// 同一条流上抓得到它的那几个状态都在那一条里。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_lowered_separator_key_only_the_rebuild_reaches(
        "crash-judge-gpu-fall-back-union-rebuild-refuses-the-separator-key",
        "two-inode-leaves-overwrite-mapping-and-accounting-lost-and-separator-key-refused-by-the-rebuild",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 从被判的根槽写（R 的根槽写，[`RecordedFlow::judged_root_index`]）所在那一段起照 `limit` 展开，之前的段都不展开。
/// 之前的段在这些状态里整段持久（`crash` 枚举的持久集合：所在的段之前每一段整段持久，展不展开都一样），所以这些状态就是全量域上这条流序号最大的那几个；
/// 根槽写之前的长段（全量域约两千万态都在它们里头）不判：那些状态上 R 的根槽没落，择不到 R，读不到 R 写坏的那一处。
fn expansion_from_the_segment_of_the_root_of_the_judged_publish(
    flow: &RecordedFlow,
    limit: usize,
) -> impl Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync {
    let every_segment_up_to_the_limit = expansion_up_to(limit);
    let segment_of_the_root = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&flow.judged_root_index))
        .expect("切段罩住流里每次写，被判的根槽写落在其中一段里");
    move |segment_index: usize, segment: &[usize]| {
        if segment_index < segment_of_the_root {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            every_segment_up_to_the_limit(segment_index, segment)
        }
    }
}

/// 核 `expansion`（[`expansion_from_the_segment_of_the_root_of_the_judged_publish`]）判的那些状态放得进一块、持久集合与 `limit` 展开的整条流序号最大的那几个状态逐个相同：
/// 变异行点名的用例只判它们，判的就是全量域那一趟里的状态。
fn assert_the_states_under_the_expansion_are_the_last_states_of_the_flow(
    flow: &RecordedFlow,
    limit: usize,
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) {
    let tables_under =
        |expansion_of_the_tables: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion| {
            build_judge_tables_against_the_version_table(
                &JudgeFlow {
                    base: &flow.base,
                    writes: &flow.writes,
                    segments: &flow.segments,
                    judged_root_index: flow.judged_root_index,
                    versions: &flow.versions,
                    expansion: expansion_of_the_tables,
                },
                &[],
            )
            .expect("建得出表")
        };
    let tables_of_the_whole_flow = tables_under(&expansion_up_to(limit));
    let tables_under_the_expansion = tables_under(expansion);
    assert!(
        tables_under_the_expansion.state_count <= STATES_PER_PAIRING_BLOCK,
        "流 {}：从 R 的根槽写那一段起的状态放得进一块（{STATES_PER_PAIRING_BLOCK} 态），实际 {} 态",
        flow.name,
        tables_under_the_expansion.state_count
    );
    let states_before_them_in_the_whole_flow =
        tables_of_the_whole_flow.state_count - tables_under_the_expansion.state_count;
    // 迭代上界是上面断言压住的状态数；跨轮不带状态
    for ordinal in 0..tables_under_the_expansion.state_count {
        assert_eq!(
            tables_under_the_expansion.persisted_writes_of_state(ordinal),
            tables_of_the_whole_flow
                .persisted_writes_of_state(states_before_them_in_the_whole_flow + ordinal),
            "流 {}：这一展开的第 {ordinal} 个状态与整条流序号 {} 的状态持久集合相同",
            flow.name,
            states_before_them_in_the_whole_flow + ordinal
        );
    }
}

/// 同 [`compare_on_every_state_examining`]，只判从 R 的根槽写那一段起到流尾的状态（[`expansion_from_the_segment_of_the_root_of_the_judged_publish`]），
/// 展开不同，对拍输入摘要与块键都与全量域那一趟不同，各存各的。
fn compare_on_the_states_from_the_root_of_the_judged_publish_examining(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    limit: usize,
    examinations: &[StateExamination],
) -> Option<Comparison> {
    let expansion = expansion_from_the_segment_of_the_root_of_the_judged_publish(flow, limit);
    assert_the_states_under_the_expansion_are_the_last_states_of_the_flow(flow, limit, &expansion);
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        &[],
        &flow.segments,
        &expansion,
        ContainerCapacityKnobs::PRODUCT,
        examinations,
        &StatesOfThePairing::EveryStateOfTheFlow,
    )
}

/// 场景：与 [`the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from`]
/// 同一条流，只判从 R 的根槽写那一段起到流尾的状态（全量域上这条流序号最大的那几个，R 的根槽落了的状态都在其中）。
/// 预期：至少一个状态上 CPU 落在 R、报读不出，每个状态对拍字逐位相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 判退坏叶流：打开那一样不判叶序与分隔 key」那一行让这一条红（打开读通，并集只剩字节丢了，GPU 退到 V）；
/// 「GPU 判退坏叶流流尾那几态：之前的段也展开」「GPU 判退坏叶流流尾那几态：之后的段不展开」两行各让一条断言红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_lowered_separator_key_only_the_opening_reaches(
        "crash-judge-gpu-fall-back-union-opening-refuses-the-separator-key-from-the-root-of-r",
        "two-inode-leaves-overwrite-extent-root-lost-and-separator-key-refused-by-the-opening-from-the-root-of-r",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_the_states_from_the_root_of_the_judged_publish_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：与 [`the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from`]
/// 同一条流，只判从 R 的根槽写那一段起到流尾的状态（全量域上这条流序号最大的那几个，R 的根槽落了的状态都在其中）。
/// 预期：至少一个状态上 CPU 落在 R、报读不出，每个状态对拍字逐位相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 判退坏叶流：建上一版那一样不判叶序与分隔 key」那一行让这一条红（建上一版接着读到记账树、字节丢了，
/// 并集只剩字节丢了，GPU 退到 V）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_lowered_separator_key_only_the_rebuild_reaches(
        "crash-judge-gpu-fall-back-union-rebuild-refuses-the-separator-key-from-the-root-of-r",
        "two-inode-leaves-overwrite-mapping-and-accounting-lost-and-separator-key-refused-by-the-rebuild-from-the-root-of-r",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_the_states_from_the_root_of_the_judged_publish_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：同 [`the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_not_above_the_leaf_before_it_is_not_fallen_back_from`]，
/// 右片分隔 key 改高（[`record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_raised_separator_key_only_the_opening_reaches`]）的每个状态：只读挂载的打开拒在
/// I-9.4 第一句（记录号不小于分隔 key），叶序与分隔 key 那一判不红，并集里有内容拒收，CPU 恢复不退。
/// 判别力：这条流全量域约两千万态，变异行不点名这一条：`crates/mutations.tsv` 里「GPU 判退坏叶流：打开那一样不判记录号下界」那一行点名
/// [`the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from`]，
/// 同一条流上抓得到它的那几个状态都在那一条里。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_raised_separator_key_only_the_opening_reaches(
        "crash-judge-gpu-fall-back-union-opening-refuses-the-raised-separator-key",
        "two-inode-leaves-overwrite-extent-root-lost-and-raised-separator-key-refused-by-the-opening",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：与 [`the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from`]
/// 同一条流，只判从 R 的根槽写那一段起到流尾的状态（全量域上这条流序号最大的那几个，R 的根槽落了的状态都在其中）。
/// 预期：至少一个状态上 CPU 落在 R、报读不出，每个状态对拍字逐位相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 判退坏叶流：打开那一样不判记录号下界」那一行让这一条红（打开读通，并集只剩字节丢了，GPU 退到 V）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_extent_root_and_whose_raised_separator_key_only_the_opening_reaches(
        "crash-judge-gpu-fall-back-union-opening-refuses-the-raised-separator-key-from-the-root-of-r",
        "two-inode-leaves-overwrite-extent-root-lost-and-raised-separator-key-refused-by-the-opening-from-the-root-of-r",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_the_states_from_the_root_of_the_judged_publish_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：两片 inode 叶之后覆盖写 R，R 的 inode 树根右片分隔 key 改高（重封到根）、R 的中央映射树节点与记账树节点都没落
/// （[`record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_raised_separator_key_only_the_rebuild_reaches`]），只判从 R 的根槽写那一段起到流尾的状态。
/// 只读挂载的打开停在映射树上、冷走读停在记账树上（都是字节丢了）、可写挂载的建上一版拒在 I-9.4 第一句（内容拒收）：并集里有内容拒收，CPU 恢复不退。
/// 预期：至少一个状态上 CPU 落在 R、报读不出，每个状态对拍字逐位相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 判退坏叶流：建上一版那一样不判记录号下界」那一行让这一条红（建上一版接着读到记账树、字节丢了，并集只剩字节丢了，GPU 退到 V）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_mapping_and_accounting_trees_and_whose_raised_separator_key_only_the_rebuild_reaches(
        "crash-judge-gpu-fall-back-union-rebuild-refuses-the-raised-separator-key-from-the-root-of-r",
        "two-inode-leaves-overwrite-mapping-and-accounting-lost-and-raised-separator-key-refused-by-the-rebuild-from-the-root-of-r",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_the_states_from_the_root_of_the_judged_publish_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}

/// 场景：两片 inode 叶之后覆盖写 R，R 的映射树根一条 key 被换（重封到根）、R 的数据单元两份都没落
/// （[`record_two_inode_leaves_whose_overwrite_lost_its_data_unit_and_whose_replaced_mapping_key_only_the_rebuild_reaches`]），只判从 R 的根槽写那一段起到流尾的状态。
/// 冷走读字节丢了、只读挂载的打开读通、可写挂载的建上一版拒在映射 key 集合那一判（`recovery::central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots`，内容拒收）。
/// 预期：至少一个状态上 CPU 落在 R、报读不出，每个状态对拍字逐位相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 判退：建上一版那一样不判映射 key 集合」「GPU 判退：查映射那一处不记查不到」两行让这一条红（建上一版读通，并集只剩字节丢了，GPU 退到 V）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_replaced_mapping_key_is_not_fallen_back_from(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_the_judged_publish_that_a_selected_version_whose_rebuild_alone_reaches_a_replaced_mapping_key_is_not_fallen_back_from";
    let flow = record_two_inode_leaves_whose_overwrite_lost_its_data_unit_and_whose_replaced_mapping_key_only_the_rebuild_reaches(
        "crash-judge-gpu-fall-back-union-rebuild-refuses-the-replaced-mapping-key-from-the-root-of-r",
        "two-inode-leaves-overwrite-data-unit-lost-and-mapping-key-refused-by-the-rebuild-from-the-root-of-r",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_the_states_from_the_root_of_the_judged_publish_examining(
        cards,
        &flow,
        expand_up_to(),
        &[states_staying_on_the_judged_publish()],
    ) else {
        return;
    };
    assert_the_gpu_falls_back_on_the_union_like_the_cpu(&flow, &comparison);
}
