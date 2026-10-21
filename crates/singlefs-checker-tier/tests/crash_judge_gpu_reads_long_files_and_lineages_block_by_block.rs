//! checker 档模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器按块分批读的三样（用户 2026-10-19 定「按块分批读，不设上限」）：恢复内核读第一个文件的数据指针（`crash_judge_common.wgsl` 的
//! `R_DATA`，一块 `DATA_POINTERS_PER_BLOCK` 条，读到块末尾从下段根往下找下一片叶），恢复内核与走读内核读中央映射的条目（`R_MAPPING_LEAVES`：
//! 一片映射叶一行、只记叶的字节基，条目逐片从叶的字节里读，不抄进一张定长的表），扫描内核读一版的谱系记录（`crash_judge_checker_scan.wgsl`，
//! 一片一片地从谱系容器的字节里读，不再抄进一张 256 条的表；C779）。
//! 长文件那几条流：新池新建文件之后一次顺序写几百到上千个单元（几块数据指针、下段几片叶、中央映射几片叶），被判的就是这次顺序写；
//! 恢复落到这一版的状态要读完全部单元。整份对拍的那几条单元数压在池级 checker 两个内核按单元、按映射条目记的那几张表之内
//! （`crash_judge_checker_common.wgsl` 的 `C_MAX_IN_VERSION`（一个单元两个落点）、`C_MAX_DATA_OBJECTS`、`C_MAX_VERSION_MAPPING_ROWS`，
//! 扫描内核的 `G_MAX_PLACEMENTS`），映射条目不过 [`MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS`] 条。
//! 映射条目过千的那一条（[`DATA_UNITS_OF_THE_FILE_WHOSE_MAPPING_PASSES_A_THOUSAND_ENTRIES`] 个单元，映射里 key 最大的那个数据单元两条位置提示改成别的槽、重封到根，
//! 恢复要经映射读它，它的映射条目排在第 1024 条之后）：池级 checker 两个内核在上面那几张表上判不了，只比恢复内核写的那一半。
//! 映射叶那条（[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元，映射里 key 最大的那个数据单元的映射条目两条位置条目对调、重封到根，那条条目不在第一片映射叶里）：
//! 两边都红 I-2.5（映射条目的位置条目按设备身份升序）。
//! 长文件旁边那条（长文件之后建一个 inode、按 inode 号写它 [`DATA_UNITS_OF_THE_FILE_WRITTEN_BY_INODE_NUMBER_BESIDE_THE_LONG_FILE`] 个单元，被判的是这次写）：
//! 被判那一版 extent 上段叶两条条目，恢复内核读完第一个文件的下段，再从 `R_UPPER`（草稿区里紧跟 `R_DATA` 那一块）取那个文件的那一条。
//! 这条流里长文件之后的几版共用长文件那棵子树：走读内核把先走过它的那一版记下的落点并进后走的那一版（`crash_judge_checker.wgsl` 的 `w_merge_below`），
//! 一次展开过的落点（两盘各一份）多于 1024 个、少于那张表的条数（`crash_judge_checker_common.wgsl` 的 `C_MAX_MERGE_EXPANDED`，`C_MAX_BELOW + 1`）。
//! 谱系那两条流：新池新建文件、可写挂载、建一批空 inode（inode 树多一片叶容器）之后连着 [`ROLLBACKS_BEFORE_THE_JUDGED_ONE`] 次管理员回退
//! （每次回退到它前一版，两种 inode 树一版一版地轮换），被判的是再下一次回退，它那一版谱系树有第二片容器（产品容量一片 1359 条），
//! 它自己那条谱系记录在第二片里；被判那一版的谱系父版与它同一种树、它前一版是另一种，谱系读漏了第二片，I-9.35 的父版就取错、比出那片叶容器。
//! 产品容量那一条两边都不红；另一条前面那些回退把谱系容器压到一片
//! [`RECORDS_OF_THE_FIRST_CONTAINER_OF_THE_SELF_PROOF`] 条（只供测试的开关，`DeletionSwitches::lineage_container_capacity`），被判那次照产品容量追加，
//! 非最右那片不满，CPU 池级 checker 按产品入口红 I-9.36，GPU 要红在同样的状态（判别力自证，C779 那一行的验收）。
//! 谱系记录改坏的三条（谱系那种流，回退次数少，[`ROLLBACKS_BEFORE_THE_JUDGED_ONE_OF_A_SHORT_LINEAGE`] 次；被判那一版的谱系容器两份副本改完重封到根）：
//! 叶序里两条对调（接得成链，扫描内核走「叶序不是升序就两两比」那一支、比不出同一个 T_rb）、两条同一个 T_rb、一条 T_old 等于 T_rb（后两条接不成链，
//! `walk.rs` 的 `LineageRecordsInconsistent`：被判那一版落了的状态上最新根 I-9.30 判不到，CPU 报不适用；谱系记录次序与内容本身没有一条不变量判红）。
//! 每条整份对拍的流每个状态 GPU 都判得了、对拍字（`crash_judge_gpu::CrossCheckWords`）与 CPU 逐状态相同。
//! 判别力：恢复内核读到块末尾不找下一块（只读第一块）、恢复内核把头一块之外的指针也往 `R_DATA` 写（压到 `R_UPPER`）、
//! 走读内核并进先走那一版的落点时展开过的单元改回 1024 条（满了记判不了）、按散列索引找孩子时同一个父单元的几行接链漏掉中间几行、
//! 挂父单元那张表的索引建好之后新来的父单元不进桶表、展开过的单元去重只比盘、恢复内核查映射与走读内核走映射条目只读第一片叶、扫描内核只读谱系第一片容器、
//! 谱系叶序不是升序就当接不成链、两两比比不出同一个 T_rb、不核 T_old < T_rb，这几条红（`crates/mutations.tsv` 里本文件那几行）。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, file_content, geometry, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::{
    IMPLEMENTED_INVARIANTS,
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, verdict_bits, CrossCheckWords, POOL_CHECKER_UNDECIDABLE_WORD,
    RECOVERY_OUTCOME_CODE_FILE_READ, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, GpuCardPool,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::pointer::{DataPointer, LocationEntry};
use singlefs_core::transaction::{
    publish_new_inodes, publish_sequential_write_of_an_inode, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{
    data_unit_payload_capacity, PACKED_TYPE_LINEAGE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED,
};
use singlefs_core::unlink::DeletionSwitches;
use singlefs_format::{
    INODE_LEAF_RECORDS, LINEAGE_CONTAINER_RECORDS, LINEAGE_RECORD_BYTES, MAPPING_ENTRY_BYTES,
    MAPPING_KEY_BYTES, PACKED_UNIT_RECORDS_OFFSET,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;
use std::collections::{BTreeMap, BTreeSet};

/// 这份测试文件的仓内路径：CPU 那一侧取样过库的设点文件。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs";
/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// 恢复内核一块数据指针装几条（`crash_judge_common.wgsl` 的 `DATA_POINTERS_PER_BLOCK`）：长文件要比它多出几块。
const DATA_POINTERS_THE_RECOVERY_KERNEL_HOLDS_IN_ONE_BLOCK: usize = 256;
/// 一片 extent 下段叶装几个单元（`crash_judge_common.wgsl` 的 `EXTENT_LOWER_LEAF_DATA_UNITS`）：长文件的下段要有几片叶、根是内部节点。
const DATA_UNITS_OF_AN_EXTENT_LOWER_LEAF: usize = 144;
/// 长文件那次顺序写的单元数：三块半数据指针、七片下段叶、几片映射叶；加上 extent、分配记录、记账那几棵树的节点，映射条目不过
/// [`MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS`] 条。
const DATA_UNITS_OF_THE_LONG_FILE: usize = 900;
const _: () = assert!(
    DATA_UNITS_OF_THE_LONG_FILE > 3 * DATA_POINTERS_THE_RECOVERY_KERNEL_HOLDS_IN_ONE_BLOCK
        && DATA_UNITS_OF_THE_LONG_FILE > 2 * DATA_UNITS_OF_AN_EXTENT_LOWER_LEAF,
    "长文件要比三块数据指针、两片下段叶都长"
);
/// 池级 checker 两个内核一版中央映射那张表的行数（`crash_judge_checker_common.wgsl` 的 `C_MAX_VERSION_MAPPING_ROWS`）：整份对拍的长文件那一版的
/// 映射条目要装得下；映射条目过千那一条要比它多。
const MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS: usize = 1024;
/// 映射条目过千那条流的单元数：一个单元一条映射条目，映射条目多于
/// [`MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS`] 条、几片映射叶。
const DATA_UNITS_OF_THE_FILE_WHOSE_MAPPING_PASSES_A_THOUSAND_ENTRIES: usize = 1100;
/// 长文件旁边那条流里按 inode 号写的那个文件的单元数：下段一片叶、上段叶里一条读下段的条目
/// （同 `crash_judge_gpu_judges_files_written_by_inode_number_like_the_cpu_judge.rs` 那条两文件流的 inode 2）。
const DATA_UNITS_OF_THE_FILE_WRITTEN_BY_INODE_NUMBER_BESIDE_THE_LONG_FILE: usize = 3;
/// 长文件旁边那条流建几个空 inode：一个，按 inode 号写的就是它。
const INODES_CREATED_BESIDE_THE_LONG_FILE: u64 = 1;
// 恢复内核记 extent 上段叶条目那一段（`crash_judge_common.wgsl` 的 `R_UPPER`，每条 2 字）在草稿区里紧跟 `R_DATA` 那一块数据指针（每条 2 字）：
// 第一个文件头一块之外第 k 条指针要是也往 `R_DATA` 写，落在上段第 k 条叶条目上。上段叶按 inode 号升序，第一个文件那一条在前、按 inode 号写的那个文件那一条紧跟其后，
// 长文件越过一块的指针要盖到后面那一条。
const _: () = assert!(
    DATA_UNITS_OF_THE_LONG_FILE > DATA_POINTERS_THE_RECOVERY_KERNEL_HOLDS_IN_ONE_BLOCK + 1,
    "长文件越过一块数据指针的那几条要盖到上段叶里第一个文件那一条之后的那一条"
);
/// I-2.5（位置条目按设备身份严格升序）的编号：映射叶那条把一条映射条目的两条位置条目对调。
const LOCATION_ENTRIES_IN_ASCENDING_DEVICE_ORDER: &str = "I-2.5";
/// 谱系记录改坏那三条被判那次之前回退几次：偶数（被判那一版落在少一片 inode 叶容器的那种树上，同 [`ROLLBACKS_BEFORE_THE_JUDGED_ONE`]），
/// 谱系一片容器，被判那一版五条记录。
const ROLLBACKS_BEFORE_THE_JUDGED_ONE_OF_A_SHORT_LINEAGE: u64 = 4;
const _: () = assert!(
    ROLLBACKS_BEFORE_THE_JUDGED_ONE_OF_A_SHORT_LINEAGE.is_multiple_of(2),
    "两种 inode 树一版一版地轮换：偶数次回退之后现行那一版是多一片叶容器的那种"
);
/// 自证那条流前面那些回退把谱系容器压到一片几条：比产品容量少一条，被判那次照产品容量追加时非最右那片不满。
const RECORDS_OF_THE_FIRST_CONTAINER_OF_THE_SELF_PROOF: u64 = LINEAGE_CONTAINER_RECORDS - 1;
/// 被判那次之前回退几次：比一片谱系容器的产品容量多一次（产品容量那条流第二片容器在被判那一版之前就开了），次数是偶数，
/// 被判那一版落在少一片 inode 叶容器的那种树上（[`record_the_lineage_flow`]）。
const ROLLBACKS_BEFORE_THE_JUDGED_ONE: u64 = LINEAGE_CONTAINER_RECORDS + 1;
const _: () = assert!(
    ROLLBACKS_BEFORE_THE_JUDGED_ONE.is_multiple_of(2),
    "两种 inode 树一版一版地轮换：偶数次回退之后现行那一版是多一片叶容器的那种"
);
/// 一次建几个空 inode：多于一片 inode 叶容器装的条数（`INODE_LEAF_RECORDS`），inode 树多一片叶容器（建完之后现数核过）。
const NEW_INODES_FOR_A_SECOND_INODE_LEAF_CONTAINER: u64 = 300;
const _: () = assert!(
    NEW_INODES_FOR_A_SECOND_INODE_LEAF_CONTAINER > INODE_LEAF_RECORDS,
    "建的空 inode 一片叶容器装不下"
);
/// 扫描内核抄谱系记录的那张表原来装几条（C779）：两条谱系流被判那一版的记录要多于它。
const LINEAGE_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_IT_READ_BLOCK_BY_BLOCK: usize = 256;

/// 一条流：起点镜像（被判那次之前的录制流施加上去）、被判那次的写表与段、模型认的每一版与被判那一版的 txg。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    judged_txg: CheckpointTxg,
}

/// 被判那次的写从 `operations_before_the_judged_publish` 起：之前的录制流施加成起点镜像，之后的切成写表与段。
fn recorded_flow_from_the_judged_publish_on(
    pool: &BuiltPool,
    operations_before_the_judged_publish: usize,
    versions: Vec<PublishedVersion>,
) -> RecordedFlow {
    let judged_txg = pool.output.root.checkpoint_txg;
    let operations = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_judged_publish]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_judged_publish..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("被判那次发布有根槽写");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        judged_txg,
    }
}

/// 长文件的内容：`data_units` 个满载的数据单元，字节随位置换。
fn content_of_the_long_file(data_units: usize) -> Vec<u8> {
    (0..data_units * data_unit_payload_capacity())
        .map(|index| u8::try_from((index * 7 + index / 251) % 253).expect("小于 256"))
        .collect()
}

/// 新池新建文件（txg 3），被判的是接着那一次顺序写（txg 4，`data_units` 个单元）；连同那次顺序写的发布结果（数据指针与映射 key）交回。
fn record_a_long_file_flow(tag: &str, data_units: usize) -> (RecordedFlow, TransactionOutput) {
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let operations_before_the_judged_publish = pool.stream.operation_count();
    let content = content_of_the_long_file(data_units);
    let output =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(
        output.data_pointers.len(),
        data_units,
        "被判那一版第一个文件的单元数"
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: output.root.checkpoint_txg,
        content,
    });
    let flow = recorded_flow_from_the_judged_publish_on(
        &pool,
        operations_before_the_judged_publish,
        versions,
    );
    (flow, output)
}

/// 整份对拍的长文件那条流（[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元）：映射条目装得进池级 checker 两个内核一版中央映射那张表。
fn record_the_long_file_flow(tag: &str) -> (RecordedFlow, TransactionOutput) {
    let (flow, output) = record_a_long_file_flow(tag, DATA_UNITS_OF_THE_LONG_FILE);
    assert!(
        output.mapping_keys.len() <= MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS,
        "被判那一版的映射条目 {} 条，装得进池级 checker 两个内核一版中央映射那张表（{MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS} 条）",
        output.mapping_keys.len()
    );
    (flow, output)
}

/// 按 inode 号写的那个文件的内容：`data_units` 个满载的数据单元，字节随位置换、与长文件的不同。
fn content_of_the_file_written_by_inode_number(data_units: usize) -> Vec<u8> {
    (0..data_units * data_unit_payload_capacity())
        .map(|index| u8::try_from((index * 11 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 新池新建文件（txg 3）、顺序写成长文件（txg 4，[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元）、建一个空 inode（txg 5）施加成起点镜像；
/// 被判的是按 inode 号把那个 inode 整份写成 [`DATA_UNITS_OF_THE_FILE_WRITTEN_BY_INODE_NUMBER_BESIDE_THE_LONG_FILE`] 个单元的那次发布（txg 6）。
/// 第一个文件的数据单元与下段照抄，被判那一版 extent 上段叶两条条目：第一个文件那一条在前，那个 inode 那一条在后。
/// 起点镜像那一版上段叶只有第一个文件那一条（建出来的 inode 长度 0，extent 树不动）。
fn record_the_flow_of_a_file_written_by_inode_number_beside_the_long_file(
    tag: &str,
) -> RecordedFlow {
    let publish_parameters = parameters();
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let content_of_the_first_file = content_of_the_long_file(DATA_UNITS_OF_THE_LONG_FILE);
    let long_file_output = sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_the_first_file,
    );
    assert_eq!(
        long_file_output.data_pointers.len(),
        DATA_UNITS_OF_THE_LONG_FILE,
        "长文件那一版第一个文件的单元数"
    );
    versions.push(PublishedVersion {
        instance: long_file_output.root.instance,
        checkpoint_txg: long_file_output.root.checkpoint_txg,
        content: content_of_the_first_file.clone(),
    });
    // 建出来的 inode 号从水位起发：比第一个文件的号大，上段叶里排在第一个文件那一条之后
    let inode_written_by_inode_number = InodeNumber(pool.output.inode_number_watermark());
    assert!(
        inode_written_by_inode_number.0 > FIRST_INODE_NUMBER,
        "新建的 inode {} 排在第一个文件之后",
        inode_written_by_inode_number.0
    );
    let output_with_the_new_inode = {
        let previous_of_the_new_inode = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous_of_the_new_inode,
            INODES_CREATED_BESIDE_THE_LONG_FILE,
            FIXED_WRITE_TIME_SECONDS,
            previous_of_the_new_inode.root.instance,
        )
        .expect("建一个空 inode")
    };
    versions.push(PublishedVersion {
        instance: output_with_the_new_inode.root.instance,
        checkpoint_txg: output_with_the_new_inode.root.checkpoint_txg,
        content: content_of_the_first_file.clone(),
    });
    pool.output = output_with_the_new_inode;
    let operations_before_the_judged_publish = pool.stream.operation_count();
    let judged_output = {
        let previous_of_the_judged_publish = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write_of_an_inode(
            &mut writer,
            &mut pool.allocator,
            &previous_of_the_judged_publish,
            inode_written_by_inode_number,
            &content_of_the_file_written_by_inode_number(
                DATA_UNITS_OF_THE_FILE_WRITTEN_BY_INODE_NUMBER_BESIDE_THE_LONG_FILE,
            ),
            FIXED_WRITE_TIME_SECONDS,
            previous_of_the_judged_publish.root.instance,
        )
        .unwrap_or_else(|refusal| {
            panic!(
                "按 inode 号写 inode {}：{refusal:?}",
                inode_written_by_inode_number.0
            )
        })
    };
    assert_eq!(
        judged_output.data_pointers.len(),
        DATA_UNITS_OF_THE_LONG_FILE,
        "被判那一版第一个文件的单元照抄"
    );
    assert_eq!(
        judged_output
            .file_other_than_the_first(inode_written_by_inode_number)
            .map(|file| file.data_pointers.len()),
        Some(DATA_UNITS_OF_THE_FILE_WRITTEN_BY_INODE_NUMBER_BESIDE_THE_LONG_FILE),
        "被判那一版按 inode 号写的那个文件的单元数"
    );
    assert!(
        judged_output.mapping_keys.len() <= MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS,
        "被判那一版的映射条目 {} 条，装得进池级 checker 两个内核一版中央映射那张表（{MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS} 条）",
        judged_output.mapping_keys.len()
    );
    versions.push(PublishedVersion {
        instance: judged_output.root.instance,
        checkpoint_txg: judged_output.root.checkpoint_txg,
        content: content_of_the_first_file,
    });
    pool.output = judged_output;
    recorded_flow_from_the_judged_publish_on(&pool, operations_before_the_judged_publish, versions)
}

/// 一片谱系容器装几条的开关：产品容量，或前面那些回退压小的那一档。
fn deletion_switches_with_the_lineage_container_capacity(
    lineage_container_capacity: LineageContainerCapacity,
) -> DeletionSwitches {
    let product = DeletionSwitches::PRODUCT;
    DeletionSwitches {
        batch_size: product.batch_size,
        tombstone_container_capacity: product.tombstone_container_capacity,
        lineage_container_capacity,
        retirement_container_capacity: product.retirement_container_capacity,
        container_tree_node_entries: product.container_tree_node_entries,
    }
}

/// 挂着的时候回退到现行那一版的前一版（同一个实例），记下回退那次发布那一版（内容与前一版相同，都是第一个文件那一版的内容）。
fn roll_back_to_the_version_before_and_note_it(
    pool: &mut BuiltPool,
    versions: &mut Vec<PublishedVersion>,
) {
    let current = pool.output.root;
    let target = RollbackTarget {
        instance: current.instance,
        checkpoint_txg: CheckpointTxg(current.checkpoint_txg.0 - 1),
    };
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
    .unwrap_or_else(|error| {
        panic!(
            "从 txg {} 回退到 txg {}：{error:?}",
            current.checkpoint_txg.0, target.checkpoint_txg.0
        )
    });
    assert_eq!(
        pool.output.root.checkpoint_txg.0,
        current.checkpoint_txg.0 + 1,
        "回退是一次向前发布、txg 加一"
    );
    versions.push(PublishedVersion {
        instance: pool.output.root.instance,
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    });
}

/// 这一版谱系树每片容器的记录数，按叶序。
fn records_of_each_lineage_container(pool: &BuiltPool) -> Vec<usize> {
    pool.output
        .lineage_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect()
}

/// 新池新建文件、重开可写挂载、建 [`NEW_INODES_FOR_A_SECOND_INODE_LEAF_CONTAINER`] 个空 inode（inode 树多一片叶容器），之后连着回退
/// [`ROLLBACKS_BEFORE_THE_JUDGED_ONE`] 次（每次回退到前一版：两种 inode 树一版一版地轮换；一片谱系容器按 `capacity_before_the_judged_rollback` 装），
/// 被判的是再下一次回退（照产品容量追加，谱系第二片容器里的那一条）。被判那一版是少一片叶容器的那种树，它的谱系父版（T_old，回退的目标）
/// 与它同一种树、它前一版是另一种：谱系读漏了第二片容器里那一条时父版取成前一版，I-9.35 两边比出那片叶容器、判红（CPU 照谱系取父版、不红）。
fn record_the_lineage_flow(
    tag: &str,
    capacity_before_the_judged_rollback: LineageContainerCapacity,
) -> RecordedFlow {
    let (flow, pool) = record_rollbacks_after_new_inodes(
        tag,
        capacity_before_the_judged_rollback,
        ROLLBACKS_BEFORE_THE_JUDGED_ONE,
    );
    let records = records_of_each_lineage_container(&pool);
    assert_eq!(records.len(), 2, "被判那一版谱系树有两片容器：{records:?}");
    assert!(
        records.iter().sum::<usize>() > LINEAGE_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_IT_READ_BLOCK_BY_BLOCK,
        "被判那一版的谱系记录多于扫描内核原来那张表的 {LINEAGE_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_IT_READ_BLOCK_BY_BLOCK} 条：{records:?}"
    );
    flow
}

/// 谱系那种流的录制（[`record_the_lineage_flow`] 的写法）：被判那次之前回退 `rollbacks_before_the_judged_one` 次（偶数），被判的是再下一次回退。
/// 连同录完的池交回（数谱系容器用）。
fn record_rollbacks_after_new_inodes(
    tag: &str,
    capacity_before_the_judged_rollback: LineageContainerCapacity,
    rollbacks_before_the_judged_one: u64,
) -> (RecordedFlow, BuiltPool) {
    assert!(
        rollbacks_before_the_judged_one.is_multiple_of(2),
        "偶数次回退之后现行那一版是多一片叶容器的那种"
    );
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let txg_of_the_first_file = pool.output.root.checkpoint_txg;
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("新池新建文件之后重开，现行那一版带文件");
    // 挂载那几次发布：新实例、内容照第一个文件那一版
    for txg in txg_of_the_first_file.0 + 1..=pool.output.root.checkpoint_txg.0 {
        versions.push(PublishedVersion {
            instance: pool.output.root.instance,
            checkpoint_txg: CheckpointTxg(txg),
            content: file_content(),
        });
    }
    let inode_leaf_containers_before_the_new_inodes = pool.output.inode_leaf_containers.len();
    publish_new_inodes_and_note_the_version(&mut pool, &mut versions);
    assert!(
        pool.output.inode_leaf_containers.len() > inode_leaf_containers_before_the_new_inodes,
        "建了 {NEW_INODES_FOR_A_SECOND_INODE_LEAF_CONTAINER} 个空 inode 之后 inode 树的叶容器多了（之前 {inode_leaf_containers_before_the_new_inodes} 片、之后 {} 片）",
        pool.output.inode_leaf_containers.len()
    );
    pool.allocator
        .set_deletion_switches(deletion_switches_with_the_lineage_container_capacity(
            capacity_before_the_judged_rollback,
        ));
    // 迭代上界：rollbacks_before_the_judged_one 次回退，每次一条谱系记录
    for _rollback in 0..rollbacks_before_the_judged_one {
        roll_back_to_the_version_before_and_note_it(&mut pool, &mut versions);
    }
    pool.allocator
        .set_deletion_switches(deletion_switches_with_the_lineage_container_capacity(
            LineageContainerCapacity::FromTheContainerFormat,
        ));
    let inode_leaf_containers_of_the_version_before_the_judged_one =
        pool.output.inode_leaf_containers.len();
    let operations_before_the_judged_publish = pool.stream.operation_count();
    roll_back_to_the_version_before_and_note_it(&mut pool, &mut versions);
    assert!(
        pool.output.inode_leaf_containers.len() < inode_leaf_containers_of_the_version_before_the_judged_one,
        "被判那一版是少一片叶容器的那种树（{} 片），它前一版是多一片的那种（{inode_leaf_containers_of_the_version_before_the_judged_one} 片）",
        pool.output.inode_leaf_containers.len()
    );
    let flow = recorded_flow_from_the_judged_publish_on(
        &pool,
        operations_before_the_judged_publish,
        versions,
    );
    (flow, pool)
}

/// 挂着的时候建 [`NEW_INODES_FOR_A_SECOND_INODE_LEAF_CONTAINER`] 个空 inode（号从记账水位起、现行实例），记下这一版（第一个文件不变）。
fn publish_new_inodes_and_note_the_version(
    pool: &mut BuiltPool,
    versions: &mut Vec<PublishedVersion>,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        NEW_INODES_FOR_A_SECOND_INODE_LEAF_CONTAINER,
        FIXED_WRITE_TIME_SECONDS,
        previous.root.instance,
    )
    .expect("建空 inode");
    versions.push(PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content: file_content(),
    });
    pool.output = output;
}

/// 一个状态 CPU 那一侧判出的：对拍字、红项、两遍恢复有没有一遍落到被判那一版、被判那次的根槽写落没落。
struct CpuJudgementOfAState {
    cross_check_words: CrossCheckWords,
    red_items: Vec<String>,
    a_recovery_lands_on_the_judged_version: bool,
    judged_root_persisted: bool,
}

/// 每个状态两边各判一遍：GPU 结论字与 CPU 那一侧，按状态序号。卡借不到（等满了、或没有独显）交 `None`。
fn judge_every_state_on_both_sides(
    cards: &'static GpuCardPool,
    test_name: &str,
    flow: &RecordedFlow,
) -> Option<(Vec<[u32; VERDICT_WORDS]>, Vec<CpuJudgementOfAState>)> {
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let judged_txg = flow.judged_txg;
    let mut cpu: Vec<CpuJudgementOfAState> = Vec::new();
    let item_name = format!("reads_block_by_block::{test_name}::cpu_side");
    let judged_on_the_cpu = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            let lands_on_the_judged_version =
                |root: Option<(InstanceGeneration, CheckpointTxg)>| {
                    root.is_some_and(|(_, txg)| txg == judged_txg)
                };
            cpu.push(CpuJudgementOfAState {
                cross_check_words: cross_check_words_of_cpu_judgement(judgement),
                red_items: judgement.red_items(),
                a_recovery_lands_on_the_judged_version: lands_on_the_judged_version(
                    judgement.consulted.effective_root,
                ) || lands_on_the_judged_version(
                    judgement.ignored.effective_root,
                ),
                judged_root_persisted: judgement.root_persisted,
            });
        },
    )
    .judged()?;
    assert_eq!(judged_on_the_cpu, state_count, "CPU 判了每个状态");
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    Some((gpu, cpu))
}

fn gpu_could_not_judge(gpu_words: &[u32; VERDICT_WORDS]) -> bool {
    gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
        || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
}

/// 每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同；不是的状态列出来（前几个）。
fn assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
    flow_name: &str,
    gpu: &[[u32; VERDICT_WORDS]],
    cpu: &[CpuJudgementOfAState],
) {
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (gpu_could_not_judge(gpu_words)
                || !cross_check_words_agree_in_the_cross_check(
                    &gpu_cross_check_words,
                    &cpu_judgement.cross_check_words,
                ))
            .then(|| {
                format!(
                    "状态 {ordinal}：GPU 判不了 {}（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）；GPU {gpu_cross_check_words:?}；CPU {:?}",
                    gpu_could_not_judge(gpu_words),
                    gpu_words[1],
                    gpu_words[POOL_CHECKER_UNDECIDABLE_WORD],
                    cpu_judgement.cross_check_words,
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {flow_name}：{} 个状态 GPU 判不了或与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 流本身合法：CPU 判器每个状态都不红。
fn assert_the_cpu_judge_finds_every_state_green(flow_name: &str, cpu: &[CpuJudgementOfAState]) {
    let red: Vec<String> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| !judgement.red_items.is_empty())
        .take(8)
        .map(|(ordinal, judgement)| format!("状态 {ordinal}：{:?}", judgement.red_items))
        .collect();
    assert!(
        red.is_empty(),
        "流 {flow_name} 合法，CPU 判器每个状态都不该红（前几个）：\n{}",
        red.join("\n")
    );
}

/// 要有被判那次的根槽写落了、恢复落到那一版的状态（GPU 读的是被判那一版）。
fn assert_some_state_recovers_to_the_judged_version(flow_name: &str, cpu: &[CpuJudgementOfAState]) {
    assert!(
        cpu.iter().any(|judgement| {
            judgement.a_recovery_lands_on_the_judged_version && judgement.judged_root_persisted
        }),
        "流 {flow_name}：要有被判那次的根槽写落了、恢复落到那一版的状态"
    );
}

/// 第一个文件 900 个数据单元（三块半数据指针、下段七片叶）：恢复落到那一版的状态 GPU 恢复内核读完全部单元，每个状态判得了、对拍字与 CPU 逐状态相同。
/// 恢复内核读到块末尾不找下一块时，那几个状态第 256 个单元起读的还是第一块的指针，单元的锚点偏移对不上，走读失败、与 CPU 对不上，这条红；
/// 改回一张 256 条、满了记判不了的表时那几个状态判不了，同样红。
#[test]
fn the_gpu_judge_reads_every_data_unit_of_a_file_longer_than_one_block_of_data_pointers_like_the_cpu_judge(
) {
    const TEST_NAME: &str = "the_gpu_judge_reads_every_data_unit_of_a_file_longer_than_one_block_of_data_pointers_like_the_cpu_judge";
    const FLOW_NAME: &str = "long-file-past-one-block-of-data-pointers";
    let (flow, _output) = record_the_long_file_flow("crash-judge-gpu-long-file");
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    assert_some_state_recovers_to_the_judged_version(FLOW_NAME, &cpu);
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 长文件（[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元，越过一块数据指针）旁边另一个按 inode 号写出的文件，被判的是写那个文件的那次发布：
/// 被判那一版 extent 上段叶两条条目，恢复内核按次序先读第一个文件的下段，再从上段那一段（`crash_judge_common.wgsl` 的 `R_UPPER`）
/// 取那个文件的那条叶条目读它的下段。恢复落到那一版的状态 GPU 收下两个文件、读完第一个文件的全部单元，每个状态判得了、对拍字与 CPU 逐状态相同。
/// `R_UPPER` 在草稿区里紧跟 `R_DATA` 那一块：读第一个文件的下段时头一块之外的指针也往 `R_DATA` 写，就压到 `R_UPPER` 里还没读的那条叶条目，
/// 读回的是第一个文件的一条数据指针（标签那一字节落在指针全 0 的 nonce 里，读成标签 0），那个文件收成一个单元都没有、恢复记 UnitMalformed，
/// 而 CPU 读得出，这条红。长文件独自那条流上段叶只有第一个文件那一条，压到的是已经读过的那一条，那条流测不出这一处。
/// 长文件之后的几版共用长文件那棵子树，池级 checker 走读内核把它并进后走的那一版时一次展开过的落点多于 1024 个：展开过的单元那张表改回 1024 条时
/// 这条流上有状态 GPU 判不了（站点 5），这条也红。并进时去重与找孩子按散列索引找（展开过的单元、挂父单元的行都过 64 条，桶表建过）：
/// 漏并进几个孩子落点，后几版的引用集合少了它们，I-3.5 把它们当成后一版新出现的，GPU 红而 CPU 不红，这条也红。
#[test]
fn the_gpu_judge_reads_a_file_written_by_inode_number_after_a_file_longer_than_one_block_of_data_pointers_like_the_cpu_judge(
) {
    const TEST_NAME: &str = "the_gpu_judge_reads_a_file_written_by_inode_number_after_a_file_longer_than_one_block_of_data_pointers_like_the_cpu_judge";
    const FLOW_NAME: &str = "file-written-by-inode-number-beside-a-long-file";
    let flow = record_the_flow_of_a_file_written_by_inode_number_beside_the_long_file(
        "crash-judge-gpu-file-by-inode-number-beside-a-long-file",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    assert_some_state_recovers_to_the_judged_version(FLOW_NAME, &cpu);
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 产品容量下谱系树有第二片容器的那一版（C779）：回退 1359 次装满第一片，再回退一次开第二片，被判的是再下一次回退（它那条记录在第二片里）。
/// 每个状态 GPU 判得了、对拍字与 CPU 逐状态相同，两边都不红（流是合法的）。扫描内核只读第一片容器时，被判那一版的谱系父版取成它前一版，
/// I-9.35 比出前一版多的那片 inode 叶容器、GPU 红而 CPU 不红；改回一张 256 条、满了记判不了的表时每个状态判不了。两样都让这条红。
#[test]
fn the_gpu_judge_judges_a_version_whose_lineage_reaches_a_second_container_at_the_product_capacity_like_the_cpu_judge(
) {
    const TEST_NAME: &str = "the_gpu_judge_judges_a_version_whose_lineage_reaches_a_second_container_at_the_product_capacity_like_the_cpu_judge";
    const FLOW_NAME: &str = "lineage-past-one-container-at-the-product-capacity";
    let flow = record_the_lineage_flow(
        "crash-judge-gpu-lineage-product",
        LineageContainerCapacity::FromTheContainerFormat,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    assert_some_state_recovers_to_the_judged_version(FLOW_NAME, &cpu);
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 判别力自证（C779 那一行的验收「把非最右那片谱系容器改成不满，GPU 必须红 I-9.36」）：前面那些回退把谱系容器压到一片 1358 条
/// （第 1359 条开了第二片），被判那次照产品容量追加进第二片；按产品入口判，非最右那片不满。被判那次的根槽写落了、恢复落到那一版的状态 CPU 池级 checker 红 I-9.36，
/// GPU 逐状态与 CPU 相同，也就红在同样的状态上。
#[test]
fn the_gpu_judge_reddens_like_the_cpu_judge_when_a_lineage_container_short_of_the_product_capacity_is_not_the_rightmost(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_like_the_cpu_judge_when_a_lineage_container_short_of_the_product_capacity_is_not_the_rightmost";
    const FLOW_NAME: &str = "lineage-with-a-short-container-before-the-rightmost";
    let capped = LineageRecordsPerContainerCappedForTests::new(
        RECORDS_OF_THE_FIRST_CONTAINER_OF_THE_SELF_PROOF,
    )
    .expect("比产品容量少一条，在 1 到 1359 之间");
    let flow = record_the_lineage_flow(
        "crash-judge-gpu-lineage-short-container",
        LineageContainerCapacity::CappedForTests(capped),
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    let invariant_index = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|identifier| {
            *identifier
                == NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
        })
        .expect("I-9.36 在已实现的不变量里");
    let reddening_states: Vec<usize> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| {
            judgement.cross_check_words.pool_checker_violated_bits & (1u128 << invariant_index) != 0
        })
        .map(|(ordinal, _)| ordinal)
        .collect();
    assert!(
        !reddening_states.is_empty(),
        "流 {FLOW_NAME}：CPU 池级 checker 要在非最右那片不满的状态上红 I-9.36"
    );
    assert!(
        reddening_states.iter().all(|ordinal| {
            cross_check_words_of_gpu_verdict(&gpu[*ordinal]).pool_checker_violated_bits
                & (1u128 << invariant_index)
                != 0
        }),
        "流 {FLOW_NAME}：CPU 红 I-9.36 的状态 GPU 也红（{} 个状态）",
        reddening_states.len()
    );
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 一个状态 CPU 与 GPU 对拍字里恢复内核写的那一半：两遍 oracle 与记录核对器的红位、两遍落到的根（实例代号、checkpoint_txg 低 32 位）、
/// 两遍的走读结局与失败点。池级 checker 的违例位、判到位与判不了的原因字不在里面。
#[derive(Debug, PartialEq, Eq)]
struct RecoveryHalfOfTheCrossCheckWords {
    red_bits: u32,
    consulted_pass_landed_root: (u32, u32),
    ignored_pass_landed_root: (u32, u32),
    consulted_pass_outcome: u32,
    ignored_pass_outcome: u32,
    consulted_pass_failure_point: u32,
    ignored_pass_failure_point: u32,
}

fn recovery_half_of_the_cross_check_words(
    words: &CrossCheckWords,
) -> RecoveryHalfOfTheCrossCheckWords {
    RecoveryHalfOfTheCrossCheckWords {
        red_bits: words.red_bits,
        consulted_pass_landed_root: (
            words.consulted_pass_landed_root.instance,
            words.consulted_pass_landed_root.checkpoint_txg_low,
        ),
        ignored_pass_landed_root: (
            words.ignored_pass_landed_root.instance,
            words.ignored_pass_landed_root.checkpoint_txg_low,
        ),
        consulted_pass_outcome: words.consulted_pass_outcome,
        ignored_pass_outcome: words.ignored_pass_outcome,
        consulted_pass_failure_point: words.consulted_pass_failure_point,
        ignored_pass_failure_point: words.ignored_pass_failure_point,
    }
}

/// 每个状态 GPU 恢复内核都判得了（结论第 0 字判不了那一位没置）、恢复那一半与 CPU 逐状态相同；不是的状态列出来（前几个）。
fn assert_every_state_recovered_on_the_gpu_like_the_cpu_recovery(
    flow_name: &str,
    gpu: &[[u32; VERDICT_WORDS]],
    cpu: &[CpuJudgementOfAState],
) {
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
            let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
            let gpu_half =
                recovery_half_of_the_cross_check_words(&cross_check_words_of_gpu_verdict(gpu_words));
            let cpu_half =
                recovery_half_of_the_cross_check_words(&cpu_judgement.cross_check_words);
            (recovery_could_not_judge || gpu_half != cpu_half).then(|| {
                format!(
                    "状态 {ordinal}：GPU 恢复内核判不了 {recovery_could_not_judge}（第 1 字 {:#x}）；GPU {gpu_half:?}；CPU {cpu_half:?}",
                    gpu_words[1]
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {flow_name}：{} 个状态 GPU 恢复内核判不了或恢复那一半与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 要有被判那次的根槽写落了、看 journal 那一遍恢复落到那一版且读出了文件的状态（恢复读完了第一个文件的每个单元）。
fn assert_some_state_reads_the_file_of_the_judged_version(
    flow_name: &str,
    cpu: &[CpuJudgementOfAState],
) {
    assert!(
        cpu.iter().any(|judgement| {
            judgement.a_recovery_lands_on_the_judged_version
                && judgement.judged_root_persisted
                && judgement.cross_check_words.consulted_pass_outcome
                    == RECOVERY_OUTCOME_CODE_FILE_READ
        }),
        "流 {flow_name}：要有被判那次的根槽写落了、恢复落到那一版并读出文件的状态"
    );
}

/// 池级 checker 的一条不变量在清单里的下标（违例位与判到位的位号）。
fn invariant_index(invariant: &str) -> usize {
    IMPLEMENTED_INVARIANTS
        .iter()
        .position(|identifier| *identifier == invariant)
        .unwrap_or_else(|| panic!("{invariant} 在已实现的不变量里"))
}

/// CPU 池级 checker 红 `invariant` 的状态（至少一个），GPU 在每一个上也红。
fn assert_the_gpu_reddens_every_state_the_cpu_reddens(
    flow_name: &str,
    invariant: &str,
    gpu: &[[u32; VERDICT_WORDS]],
    cpu: &[CpuJudgementOfAState],
) {
    let bit = 1u128 << invariant_index(invariant);
    let reddening_states: Vec<usize> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| judgement.cross_check_words.pool_checker_violated_bits & bit != 0)
        .map(|(ordinal, _)| ordinal)
        .collect();
    assert!(
        !reddening_states.is_empty(),
        "流 {flow_name}：CPU 池级 checker 要在被判那一版落了的状态上红 {invariant}"
    );
    assert!(
        reddening_states.iter().all(|ordinal| {
            cross_check_words_of_gpu_verdict(&gpu[*ordinal]).pool_checker_violated_bits & bit != 0
        }),
        "流 {flow_name}：CPU 红 {invariant} 的状态 GPU 也红（{} 个状态）",
        reddening_states.len()
    );
}

/// 被判那次的根槽写落了的那些状态上，CPU 池级 checker 判没判到 `invariant`（判到 = 成立或违例；射程里有一部分没判与不适用都不算）。
fn the_cpu_judges_the_invariant_on_the_states_where_the_judged_root_persisted(
    invariant: &str,
    cpu: &[CpuJudgementOfAState],
) -> Vec<bool> {
    let bit = 1u128 << invariant_index(invariant);
    cpu.iter()
        .filter(|judgement| judgement.judged_root_persisted)
        .map(|judgement| judgement.cross_check_words.pool_checker_judged_bits & bit != 0)
        .collect()
}

// ───────── 改坏被判那次的单元写、一路重封到根（另抄一份 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 那几个函数的写法）

/// journal 记录头 487 字节（D23（journal 的角色与格式） 已定项 4），点名项从这里起、每项 56 字节（已定项 17）。
const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
const JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD: usize = 56;
/// 一条位置条目的字节数：盘 4 + 槽 6 + 整单元校验和 4（D19（块指针的结构与宽度预算））。
const LOCATION_ENTRY_BYTES: usize = 14;
/// 单元头里类标签的偏移；码 2 节点头里层级与 key 宽的偏移（`unit::parse_index_node`）。
const UNIT_CLASS_OFFSET: usize = 6;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
/// 码 2 节点头去掉最小、最大两个 key 之后的字节数，与头末到第一条条目之间的字节数（`unit::parse_index_node`：头到 86 + 2 × key 宽为止，条目从头之后 29 字节起）。
const INDEX_NODE_HEADER_BYTES_WITHOUT_ITS_TWO_KEYS: usize = 86;
const INDEX_NODE_BYTES_BETWEEN_THE_HEADER_AND_THE_ENTRIES: usize = 29;
/// 码 2 节点的字节数。
const INDEX_NODE_BYTES: usize = 16384;
/// extent 下段叶的 key 宽（locality 8 + inode 8 + 偏移 8）。
const EXTENT_LOWER_LEAF_KEY_BYTES: u8 = 24;
/// 码 3 打包记录单元头里打包记录类型与记录数的偏移（`unit::parse_packed_unit`）。
const PACKED_UNIT_RECORD_TYPE_OFFSET: usize = 51;
const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
/// 谱系记录（`records::LineageRecord`，24 字节）里 T_rb 与 T_old 的偏移。
const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
const LINEAGE_RECORD_PARENT_TXG_OFFSET: usize = 8;

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 码 2 节点的 key 宽、条目宽、第一条条目的偏移与条数。
fn index_node_key_width_entry_width_entries_start_and_entry_count(
    bytes: &[u8],
) -> (usize, usize, usize, usize) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = INDEX_NODE_HEADER_BYTES_WITHOUT_ITS_TWO_KEYS + 2 * key_width;
    (
        key_width,
        usize::from(read_u16_at(bytes, header_end - 2)),
        header_end + INDEX_NODE_BYTES_BETWEEN_THE_HEADER_AND_THE_ENTRIES,
        usize::from(read_u16_at(bytes, header_end - 4)),
    )
}

/// 一条位置条目写在单元里的 14 个字节。
fn location_entry_bytes(location: &LocationEntry) -> [u8; LOCATION_ENTRY_BYTES] {
    let mut bytes = [0u8; LOCATION_ENTRY_BYTES];
    bytes[..4].copy_from_slice(&location.device.0.to_le_bytes());
    bytes[4..10].copy_from_slice(&location.slot.0.to_le_bytes()[..6]);
    bytes[10..].copy_from_slice(&location.unit_checksum.to_le_bytes());
    bytes
}

/// `needle` 在 `haystack` 里每一处的起点。
fn positions_of(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(position, _)| position)
        .collect()
}

/// 一个单元重封自己的两道校验和：载荷 CRC（按类的偏移，罩明文头之后到单元末）与头校验和（字段 10，罩明文头）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[UNIT_CLASS_OFFSET] {
        1 => (105, 101),
        2 => {
            let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
            (86 + 2 * key_width, 76 + 2 * key_width)
        }
        3 => (107, 89),
        other => panic!("单元类标签 {other} 不在 1 / 2 / 3 里"),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, 10);
    bytes[10..42].copy_from_slice(&digest);
}

/// journal 记录重封：载荷校验和（95，罩点名项）再整条头校验和（46，罩 4096）。
fn reseal_journal_record(bytes: &mut [u8]) {
    let named = usize::try_from(u32::from_le_bytes(bytes[12..16].try_into().expect("4")))
        .expect("点名项数");
    let payload_crc = crc32_castagnoli(
        &bytes[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
    bytes[46..78].copy_from_slice(&digest);
}

/// 下一条记录该带的反向链：这一条 487 字节头、校验和字段按 0 参与的 CRC-32C。
fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..JOURNAL_RECORD_HEADER_BYTES].to_vec();
    header[46..78].fill(0);
    crc32_castagnoli(&header)
}

fn reseal_root_record(bytes: &mut [u8]) {
    let digest = wide_checksum_with_field_zeroed(bytes, bytes.len(), 138);
    bytes[138..170].copy_from_slice(&digest);
}

/// 改坏一批单元写（同一个单元的两份副本）之后把校验和一路重封到根：被改的单元先重封自己的两道校验和，再把每一处指着它的位置条目
/// （树节点、中央映射叶、根记录、journal 点名项里的 盘 4 + 槽 6 + 整单元校验和 4）换成新的整单元 CRC，被改的那些结构再各自重封
/// （单元继续往上传；journal 记录还要把同实例下一条的反向链改成它新的链值、一路传下去，等单元那一路传完再传一遍）。留下的坏法只剩被改的那几个字节。
fn corrupt_units_and_reseal_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
) {
    let mut journal_records_resealed: BTreeSet<usize> = BTreeSet::new();
    let mut pending: Vec<(usize, u32)> = Vec::new();
    for target in targets {
        let write = &mut flow.writes[*target];
        assert_eq!(write.kind, StepKind::UnitWrite, "改的是单元写");
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("单元写是普通写");
        };
        let old_crc = crc32_castagnoli(bytes);
        mutate(bytes);
        reseal_unit(bytes);
        pending.push((*target, old_crc));
    }
    // 迭代上界：每一轮重封一个单元、往上传一层，到根记录与 journal 记录为止
    while let Some((index, old_crc)) = pending.pop() {
        let (device, slot, new_crc) = {
            let write = &flow.writes[index];
            let bytes = write.bytes().expect("单元写带字节");
            (
                write.device.0,
                write.offset.0 / 16384,
                crc32_castagnoli(bytes),
            )
        };
        if old_crc == new_crc {
            continue;
        }
        let mut pattern = Vec::with_capacity(LOCATION_ENTRY_BYTES);
        pattern.extend_from_slice(&device.to_le_bytes());
        pattern.extend_from_slice(&slot.to_le_bytes()[..6]);
        pattern.extend_from_slice(&old_crc.to_le_bytes());
        for referrer in 0..flow.writes.len() {
            let kind = flow.writes[referrer].kind;
            let WrittenContents::Bytes(bytes) = &mut flow.writes[referrer].contents else {
                continue;
            };
            // 码 1 数据单元里装的是文件内容、没有位置条目：长文件那几条流上千个，不逐字节找
            if kind == StepKind::UnitWrite && bytes[UNIT_CLASS_OFFSET] == UNIT_CLASS_DATA {
                continue;
            }
            let mut replaced = false;
            let mut position = 0;
            while position + LOCATION_ENTRY_BYTES <= bytes.len() {
                if bytes[position..position + LOCATION_ENTRY_BYTES] == pattern[..] {
                    bytes[position + 10..position + LOCATION_ENTRY_BYTES]
                        .copy_from_slice(&new_crc.to_le_bytes());
                    replaced = true;
                    position += LOCATION_ENTRY_BYTES;
                } else {
                    position += 1;
                }
            }
            if !replaced {
                continue;
            }
            match kind {
                StepKind::UnitWrite => {
                    let referrer_old_crc = {
                        // 位置条目已换、校验和还没重封：整单元 CRC 要按重封之前的原样算，用写进去之前的字节复原
                        let mut before = bytes.clone();
                        let mut restore_position = 0;
                        while restore_position + LOCATION_ENTRY_BYTES <= before.len() {
                            if before[restore_position..restore_position + 10] == pattern[..10]
                                && before
                                    [restore_position + 10..restore_position + LOCATION_ENTRY_BYTES]
                                    == new_crc.to_le_bytes()
                            {
                                before[restore_position + 10
                                    ..restore_position + LOCATION_ENTRY_BYTES]
                                    .copy_from_slice(&old_crc.to_le_bytes());
                                restore_position += LOCATION_ENTRY_BYTES;
                            } else {
                                restore_position += 1;
                            }
                        }
                        crc32_castagnoli(&before)
                    };
                    reseal_unit(bytes);
                    pending.push((referrer, referrer_old_crc));
                }
                StepKind::RootRecordFua => reseal_root_record(bytes),
                StepKind::JournalRecord => {
                    reseal_journal_record(bytes);
                    journal_records_resealed.insert(referrer);
                }
                StepKind::ZeroFill
                | StepKind::Discard
                | StepKind::SystemConfigurationSlot
                | StepKind::Barrier => {
                    panic!("位置条目不该出现在 {kind:?} 里")
                }
            }
        }
    }
    carry_the_back_chain_forward_from(flow, journal_records_resealed);
}

/// journal 记录头里实例代号（4 字节，偏移 16）与计数器（6 字节，偏移 20）。
fn journal_record_instance_and_counter(bytes: &[u8]) -> (u32, u64) {
    let mut counter = [0u8; 8];
    counter[..6].copy_from_slice(&bytes[20..26]);
    (
        u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
        u64::from_le_bytes(counter),
    )
}

/// 反向链一路传下去：这几条 journal 记录写重封过，同盘、同实例、计数器大 1 的那一条带的是前一条头的链值——不等就改成它、重封，再往下传。
/// 按（盘、实例、计数器）从小到大过一遍（每条的链值只看计数器小 1 的那一条最后的头）：长文件那几条流一次发布上千条记录，每条至多重封一次。
fn carry_the_back_chain_forward_from(
    flow: &mut RecordedFlow,
    mut journal_records_resealed: BTreeSet<usize>,
) {
    if journal_records_resealed.is_empty() {
        return;
    }
    let mut journal_record_writes: BTreeMap<(u32, u32, u64), Vec<usize>> = BTreeMap::new();
    for (write_index, write) in flow.writes.iter().enumerate() {
        if write.kind != StepKind::JournalRecord {
            continue;
        }
        let Some(bytes) = write.bytes() else {
            continue;
        };
        let (instance, counter) = journal_record_instance_and_counter(bytes);
        journal_record_writes
            .entry((write.device.0, instance, counter))
            .or_default()
            .push(write_index);
    }
    let keys: Vec<(u32, u32, u64)> = journal_record_writes.keys().copied().collect();
    for (device, instance, counter) in keys {
        let Some(previous_counter) = counter.checked_sub(1) else {
            continue;
        };
        let Some(previous_records) =
            journal_record_writes.get(&(device, instance, previous_counter))
        else {
            continue;
        };
        let chains: Vec<u32> = previous_records
            .iter()
            .filter(|previous| journal_records_resealed.contains(*previous))
            .map(|previous| {
                journal_chain_value(flow.writes[*previous].bytes().expect("记录写带字节"))
            })
            .collect();
        let records = journal_record_writes[&(device, instance, counter)].clone();
        for chain in chains {
            for record in &records {
                let WrittenContents::Bytes(bytes) = &mut flow.writes[*record].contents else {
                    continue;
                };
                if bytes[91..95] == chain.to_le_bytes() {
                    continue;
                }
                bytes[91..95].copy_from_slice(&chain.to_le_bytes());
                reseal_journal_record(bytes);
                journal_records_resealed.insert(*record);
            }
        }
    }
}

/// 被判那次写里一个单元的两份副本：命中 `select` 的单元写（同一个单元写两盘，字节相同）。
fn the_two_copies_of_the_unit(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
) -> Vec<usize> {
    let copies: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        copies.len(),
        2,
        "被判那次写里「{what}」的单元写是一个单元的两份副本"
    );
    assert_eq!(
        flow.writes[copies[0]].bytes(),
        flow.writes[copies[1]].bytes(),
        "「{what}」两份副本字节相同"
    );
    copies
}

/// 被判那次顺序写的一个单元在中央映射里那一条：它所在的那片映射叶（两份副本的写下标）、条目在叶里的偏移、它的 27 字节 key 与那个单元的数据指针。
struct MappingEntryOfADataUnit {
    leaf_copies: Vec<usize>,
    entry_offset: usize,
    key: Vec<u8>,
    data_pointer: DataPointer,
}

/// 被判那次写的中央映射叶（码 2、key 宽 27、层级 0）里 key 最大的那条码 1 条目（映射叶序里排在最后的数据单元），连同它指的那个单元的数据指针
/// （两条位置条目与它逐字节相同的那一条）。
fn mapping_entry_of_the_data_unit_with_the_largest_mapping_key(
    flow: &RecordedFlow,
    output: &TransactionOutput,
) -> MappingEntryOfADataUnit {
    let key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let mut largest: Option<(Vec<u8>, usize, usize)> = None;
    for (write_index, write) in flow.writes.iter().enumerate() {
        let Some(bytes) = write.bytes() else {
            continue;
        };
        if write.kind != StepKind::UnitWrite
            || bytes[UNIT_CLASS_OFFSET] != UNIT_CLASS_INDEX_NODE
            || usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]) != key_bytes
            || bytes[INDEX_NODE_LEVEL_OFFSET] != 0
        {
            continue;
        }
        let (_, entry_width, entries_start, entry_count) =
            index_node_key_width_entry_width_entries_start_and_entry_count(bytes);
        for entry_index in 0..entry_count {
            let entry_offset = entries_start + entry_index * entry_width;
            let key = &bytes[entry_offset..entry_offset + key_bytes];
            if key[0] == UNIT_CLASS_DATA
                && largest
                    .as_ref()
                    .is_none_or(|(largest_key, _, _)| largest_key.as_slice() < key)
            {
                largest = Some((key.to_vec(), write_index, entry_offset));
            }
        }
    }
    let (key, write_index, entry_offset) = largest.expect("被判那次写里有映射叶、叶里有码 1 条目");
    let leaf = flow.writes[write_index].bytes().expect("单元写带字节");
    let locations = leaf
        [entry_offset + key_bytes..entry_offset + key_bytes + 2 * LOCATION_ENTRY_BYTES]
        .to_vec();
    let data_pointer = *output
        .data_pointers
        .iter()
        .find(|pointer| {
            let mut bytes = Vec::with_capacity(2 * LOCATION_ENTRY_BYTES);
            for location in &pointer.locations {
                bytes.extend_from_slice(&location_entry_bytes(location));
            }
            bytes == locations
        })
        .expect("key 最大的那条码 1 条目指着被判那次顺序写的一个单元");
    let leaf_copies = the_two_copies_of_the_unit(
        flow,
        "key 最大的码 1 映射条目所在的映射叶",
        &|bytes| bytes == leaf,
    );
    MappingEntryOfADataUnit {
        leaf_copies,
        entry_offset,
        key,
        data_pointer,
    }
}

/// 这一版的映射 key 里比 `key` 小的有几条：`key` 那一条在映射叶序里排第几（从 0 数）。
fn rank_among_the_mapping_keys(output: &TransactionOutput, key: &[u8]) -> usize {
    output
        .mapping_keys
        .iter()
        .filter(|candidate| candidate.as_slice() < key)
        .count()
}

/// 被判那次顺序写的一个单元（`data_pointer`）：extent 下段叶里它那条数据指针的两条位置提示改指另一个单元（`decoy`）的两个槽
/// （那两槽里是别的单元，整单元校验和对不上），下段叶两份副本都改、一路重封到根；映射里它那条条目不动。
/// 恢复（CPU 的冷走读与 GPU 恢复内核）读它都经映射回退，查映射要读到它那条条目。
fn make_the_location_hints_of_a_data_unit_stale(
    flow: &mut RecordedFlow,
    data_pointer: &DataPointer,
    decoy: &DataPointer,
) {
    let hints = data_pointer
        .locations
        .map(|location| location_entry_bytes(&location));
    let stale_hints = [0, 1].map(|copy| {
        assert_ne!(
            decoy.locations[copy].slot, data_pointer.locations[copy].slot,
            "两个单元不在同一槽"
        );
        let mut stale = hints[copy];
        stale[4..10].copy_from_slice(&decoy.locations[copy].slot.0.to_le_bytes()[..6]);
        stale
    });
    let lower_leaf_copies = the_two_copies_of_the_unit(
        flow,
        "那个单元的数据指针所在的 extent 下段叶",
        &|bytes| {
            bytes[UNIT_CLASS_OFFSET] == UNIT_CLASS_INDEX_NODE
                && bytes[INDEX_NODE_KEY_WIDTH_OFFSET] == EXTENT_LOWER_LEAF_KEY_BYTES
                && bytes[INDEX_NODE_LEVEL_OFFSET] == 0
                && hints
                    .iter()
                    .all(|hint| positions_of(bytes, hint).len() == 1)
        },
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &lower_leaf_copies, &|bytes| {
        for (hint, stale) in hints.iter().zip(&stale_hints) {
            let position = positions_of(bytes, hint)[0];
            bytes[position..position + LOCATION_ENTRY_BYTES].copy_from_slice(stale);
        }
    });
}

/// 被判那次写的谱系容器（码 3、打包记录类型 5；两份副本）里的记录区按 `rewrite` 改（记录区从 136 起，每条 24 字节），一路重封到根。
/// 交回之前核记录数是 `records`。
fn rewrite_the_lineage_records_of_the_judged_version(
    flow: &mut RecordedFlow,
    records: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    let records_offset = usize::try_from(PACKED_UNIT_RECORDS_OFFSET).expect("136");
    let record_bytes = usize::try_from(LINEAGE_RECORD_BYTES).expect("24");
    let container_copies =
        the_two_copies_of_the_unit(flow, "被判那一版的谱系容器", &|bytes| {
            bytes[UNIT_CLASS_OFFSET] == UNIT_CLASS_PACKED
                && read_u16_at(bytes, PACKED_UNIT_RECORD_TYPE_OFFSET) == PACKED_TYPE_LINEAGE
        });
    let container = flow.writes[container_copies[0]]
        .bytes()
        .expect("单元写带字节");
    assert_eq!(
        usize::from(read_u16_at(container, PACKED_UNIT_RECORD_COUNT_OFFSET)),
        records,
        "被判那一版谱系容器的记录数"
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &container_copies, &|bytes| {
        rewrite(&mut bytes[records_offset..records_offset + records * record_bytes]);
    });
}

/// 记录区第 `record` 条的一个 8 字节 txg 字段。
fn lineage_record_txg(records: &[u8], record: usize, field_offset: usize) -> u64 {
    let record_bytes = usize::try_from(LINEAGE_RECORD_BYTES).expect("24");
    read_u64_at(records, record * record_bytes + field_offset)
}

fn set_lineage_record_txg(records: &mut [u8], record: usize, field_offset: usize, txg: u64) {
    let record_bytes = usize::try_from(LINEAGE_RECORD_BYTES).expect("24");
    let start = record * record_bytes + field_offset;
    records[start..start + 8].copy_from_slice(&txg.to_le_bytes());
}

/// 谱系记录改坏那三条的流：谱系那种流回退 [`ROLLBACKS_BEFORE_THE_JUDGED_ONE_OF_A_SHORT_LINEAGE`] 次、被判的是再下一次回退（产品容量），
/// 被判那一版谱系一片容器、记录按 T_rb 升序，按 `rewrite` 改完重封到根。
fn record_a_short_lineage_flow_with_its_records_rewritten(
    tag: &str,
    rewrite: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let (mut flow, pool) = record_rollbacks_after_new_inodes(
        tag,
        LineageContainerCapacity::FromTheContainerFormat,
        ROLLBACKS_BEFORE_THE_JUDGED_ONE_OF_A_SHORT_LINEAGE,
    );
    let records = records_of_each_lineage_container(&pool);
    let record_count =
        usize::try_from(ROLLBACKS_BEFORE_THE_JUDGED_ONE_OF_A_SHORT_LINEAGE + 1).expect("几条");
    assert_eq!(
        records,
        vec![record_count],
        "被判那一版谱系一片容器、每次回退一条记录"
    );
    rewrite_the_lineage_records_of_the_judged_version(&mut flow, record_count, rewrite);
    flow
}

/// 映射条目过千（[`DATA_UNITS_OF_THE_FILE_WHOSE_MAPPING_PASSES_A_THOUSAND_ENTRIES`] 个单元），映射里 key 最大的那个数据单元的两条位置提示改指别的槽、重封到根：
/// 恢复读它要经映射回退，它的映射条目排在第 1024 条之后。每个状态 GPU 恢复内核判得了、恢复那一半与 CPU 逐状态相同，有状态恢复落到被判那一版并读出文件。
/// 池级 checker 两个内核在这条流上判不了（走读内核 `C_MAX_IN_VERSION` 那几张按单元记的表先满，文件头），不比。
/// 恢复内核记映射条目改回一张定长的表、满了记判不了时每个落到那一版的状态判不了；查映射只读第一片映射叶时那个单元映射里查不到、恢复失败，两样都让这条红。
#[test]
fn the_gpu_recovery_kernel_reads_a_unit_through_a_central_mapping_of_more_than_a_thousand_entries_like_the_cpu_recovery(
) {
    const TEST_NAME: &str = "the_gpu_recovery_kernel_reads_a_unit_through_a_central_mapping_of_more_than_a_thousand_entries_like_the_cpu_recovery";
    const FLOW_NAME: &str = "long-file-whose-central-mapping-passes-a-thousand-entries";
    let (mut flow, output) = record_a_long_file_flow(
        "crash-judge-gpu-mapping-past-a-thousand-entries",
        DATA_UNITS_OF_THE_FILE_WHOSE_MAPPING_PASSES_A_THOUSAND_ENTRIES,
    );
    assert!(
        output.mapping_keys.len() > MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS,
        "被判那一版的映射条目 {} 条，多于 {MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS} 条",
        output.mapping_keys.len()
    );
    let mapping_entry = mapping_entry_of_the_data_unit_with_the_largest_mapping_key(&flow, &output);
    assert!(
        rank_among_the_mapping_keys(&output, &mapping_entry.key)
            >= MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS,
        "那个单元的映射条目排在第 {MAPPING_ENTRIES_A_VERSION_MAPPING_TABLE_OF_THE_POOL_CHECKER_KERNELS_HOLDS} 条之后"
    );
    let decoy = *output.data_pointers.first().expect("长文件有单元");
    make_the_location_hints_of_a_data_unit_stale(&mut flow, &mapping_entry.data_pointer, &decoy);
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_some_state_reads_the_file_of_the_judged_version(FLOW_NAME, &cpu);
    assert_every_state_recovered_on_the_gpu_like_the_cpu_recovery(FLOW_NAME, &gpu, &cpu);
}

/// 长文件（[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元、几片映射叶）映射里 key 最大的那个数据单元的映射条目两条位置条目对调（盘序成了 1、0）、映射叶重封到根：
/// 那条条目不在第一片映射叶里。被判那一版落了的状态 CPU 池级 checker 红 I-2.5，GPU 在同样的状态上也红，每个状态整份对拍相同。
/// 走读内核走映射条目只读第一片映射叶时那条条目不判、GPU 不红，这条红。
#[test]
fn the_gpu_judge_reddens_like_the_cpu_judge_on_a_mapping_entry_out_of_device_order_past_the_first_mapping_leaf(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_like_the_cpu_judge_on_a_mapping_entry_out_of_device_order_past_the_first_mapping_leaf";
    const FLOW_NAME: &str = "long-file-with-a-mapping-entry-out-of-device-order";
    let (mut flow, output) =
        record_the_long_file_flow("crash-judge-gpu-mapping-entry-out-of-order");
    let mapping_entry = mapping_entry_of_the_data_unit_with_the_largest_mapping_key(&flow, &output);
    let key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let entries_of_a_full_mapping_leaf = (INDEX_NODE_BYTES
        - (INDEX_NODE_HEADER_BYTES_WITHOUT_ITS_TWO_KEYS
            + 2 * key_bytes
            + INDEX_NODE_BYTES_BETWEEN_THE_HEADER_AND_THE_ENTRIES))
        / usize::try_from(MAPPING_ENTRY_BYTES).expect("55");
    assert!(
        rank_among_the_mapping_keys(&output, &mapping_entry.key) >= entries_of_a_full_mapping_leaf,
        "那个单元的映射条目排在一片映射叶装得下的 {entries_of_a_full_mapping_leaf} 条之后，不在第一片映射叶里"
    );
    let entry_offset = mapping_entry.entry_offset;
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &mapping_entry.leaf_copies, &|bytes| {
        let first_location = entry_offset + key_bytes;
        let second_location = first_location + LOCATION_ENTRY_BYTES;
        let mut swapped = bytes[first_location..second_location + LOCATION_ENTRY_BYTES].to_vec();
        swapped.rotate_left(LOCATION_ENTRY_BYTES);
        bytes[first_location..second_location + LOCATION_ENTRY_BYTES].copy_from_slice(&swapped);
    });
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_some_state_recovers_to_the_judged_version(FLOW_NAME, &cpu);
    assert_the_gpu_reddens_every_state_the_cpu_reddens(
        FLOW_NAME,
        LOCATION_ENTRIES_IN_ASCENDING_DEVICE_ORDER,
        &gpu,
        &cpu,
    );
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 被判那一版谱系叶序里头两条对调（T_rb 不再升序、没有两条同一个 T_rb、每条 T_old < T_rb）：谱系接得成链，被判那一版落了的状态上
/// 最新根 I-9.30 ④ 照判。扫描内核走「叶序不是升序就两两比」那一支、比不出同一个 T_rb。每个状态 GPU 与 CPU 整份对拍相同。
/// 那一支改成叶序不是升序就当接不成链时 GPU 在那些状态上判不到 I-9.30、与 CPU 对不上，这条红。
#[test]
fn the_gpu_judge_chains_a_lineage_whose_records_are_out_of_leaf_order_like_the_cpu_judge() {
    const TEST_NAME: &str =
        "the_gpu_judge_chains_a_lineage_whose_records_are_out_of_leaf_order_like_the_cpu_judge";
    const FLOW_NAME: &str = "lineage-with-two-records-swapped";
    let flow = record_a_short_lineage_flow_with_its_records_rewritten(
        "crash-judge-gpu-lineage-records-swapped",
        &|records| {
            let record_bytes = usize::try_from(LINEAGE_RECORD_BYTES).expect("24");
            assert!(
                lineage_record_txg(records, 0, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET)
                    < lineage_record_txg(records, 1, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
                "写路径追加的谱系按 T_rb 升序"
            );
            records[..2 * record_bytes].rotate_left(record_bytes);
        },
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    let judged = the_cpu_judges_the_invariant_on_the_states_where_the_judged_root_persisted(
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        &cpu,
    );
    assert!(
        judged.iter().any(|is_judged| *is_judged),
        "流 {FLOW_NAME}：谱系接得成链，被判那一版落了的状态里要有 CPU 判到 I-9.30 的（{judged:?}）"
    );
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 被判那一版谱系第二条的 T_rb 改成与第一条相同（同一个 T_rb 两条，T_old 仍小于 T_rb）：谱系接不成链（`walk.rs` 的 `LineageRecordsInconsistent`），
/// 被判那一版落了的状态上最新根 I-9.30 判不到（CPU 报不适用）。扫描内核走两两比那一支、比出同一个 T_rb。每个状态 GPU 与 CPU 整份对拍相同。
/// 两两比比不出同一个 T_rb 时 GPU 当谱系接得成链、照判 I-9.30，这条红。
#[test]
fn the_gpu_judge_refuses_a_lineage_with_two_records_of_one_rollback_txg_like_the_cpu_judge() {
    const TEST_NAME: &str =
        "the_gpu_judge_refuses_a_lineage_with_two_records_of_one_rollback_txg_like_the_cpu_judge";
    const FLOW_NAME: &str = "lineage-with-two-records-of-one-rollback-txg";
    let flow = record_a_short_lineage_flow_with_its_records_rewritten(
        "crash-judge-gpu-lineage-one-rollback-txg-twice",
        &|records| {
            let rollback_txg_of_the_first =
                lineage_record_txg(records, 0, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
            assert!(
                lineage_record_txg(records, 1, LINEAGE_RECORD_PARENT_TXG_OFFSET)
                    < rollback_txg_of_the_first,
                "第二条的 T_old 小于第一条的 T_rb：改完只剩同一个 T_rb 两条这一处不合"
            );
            set_lineage_record_txg(
                records,
                1,
                LINEAGE_RECORD_ROLLBACK_TXG_OFFSET,
                rollback_txg_of_the_first,
            );
        },
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    let judged = the_cpu_judges_the_invariant_on_the_states_where_the_judged_root_persisted(
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        &cpu,
    );
    assert!(
        !judged.is_empty() && judged.iter().all(|is_judged| !*is_judged),
        "流 {FLOW_NAME}：谱系接不成链，被判那一版落了的每个状态 CPU 都不判到 I-9.30（{judged:?}）"
    );
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}

/// 被判那一版谱系第一条的 T_old 改成等于它的 T_rb：谱系接不成链（同上），被判那一版落了的状态上最新根 I-9.30 判不到（CPU 报不适用）。
/// 每个状态 GPU 与 CPU 整份对拍相同。扫描内核不核每条 T_old < T_rb 时 GPU 当谱系接得成链、照判 I-9.30，这条红。
#[test]
fn the_gpu_judge_refuses_a_lineage_record_not_younger_than_its_parent_like_the_cpu_judge() {
    const TEST_NAME: &str =
        "the_gpu_judge_refuses_a_lineage_record_not_younger_than_its_parent_like_the_cpu_judge";
    const FLOW_NAME: &str = "lineage-with-a-record-not-younger-than-its-parent";
    let flow = record_a_short_lineage_flow_with_its_records_rewritten(
        "crash-judge-gpu-lineage-record-not-younger-than-its-parent",
        &|records| {
            let rollback_txg = lineage_record_txg(records, 0, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
            set_lineage_record_txg(records, 0, LINEAGE_RECORD_PARENT_TXG_OFFSET, rollback_txg);
        },
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    let judged = the_cpu_judges_the_invariant_on_the_states_where_the_judged_root_persisted(
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        &cpu,
    );
    assert!(
        !judged.is_empty() && judged.iter().all(|is_judged| !*is_judged),
        "流 {FLOW_NAME}：谱系接不成链，被判那一版落了的每个状态 CPU 都不判到 I-9.30（{judged:?}）"
    );
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(FLOW_NAME, &gpu, &cpu);
}
