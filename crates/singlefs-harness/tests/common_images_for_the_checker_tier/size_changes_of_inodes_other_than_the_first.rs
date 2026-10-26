//! 两档共用场景；池级 checker 镜像交给 checker 档给定状态引擎两段判。
//! 第一个文件之外的 inode 缩短与变长（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S4、S6、S7，
//! 用户 2026-10-26 定射程「任意活 inode 都能缩短与变长」）：攻方腿 H1–H5 那五段历史在实现之后都是合法镜像——四个读者（只读挂载态读者、
//! 扫描重建一版、冷恢复、可写挂载）都收、读回等于期望，池级 checker 一条不红；加上正推腿 7.1 节行为层的几条（K9 叶边界对任意 inode、
//! 批间停下再重挂续做、快照下缩短、带洞时被 unlink、两个 inode 的回退），与「有记录、没有条目、size > 0」那一形读者拒、checker 红。
//!
//! 期望值都来自条款（K2 先落目标、K3 尾单元、K5 洞与标签、S2 粘住的标签 0）与写进去的字节，不来自实现的计划。
//! 判别力：只删拒绝那一支、别处照旧只认第一个文件的那一种放开（`crates/mutations.tsv` 删掉的那一行的变异体）上，H1 那一条红在「写标签 0」，
//! H2–H5 红在读者与读回；`crates/mutations.tsv` 以「m4 多 inode 改 size」起头的那几行逐条退回一处只认第一个文件的写法，各有一条在这里红。

#![allow(dead_code, reason = "两档各只调用其中一部分场景与帮手")]

use crate::common::{build_pool, parameters, BuiltPool, IMAGE_BYTES};
use crate::common_deletion_in_progress::{
    change_the_inode_record, publish_new_inodes_and_advance_the_pool, publish_the_intent_of,
    roll_back_to, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_truncation::{
    batches_of_one, payload_capacity, publish_size_change_of_and_advance,
    publish_the_next_truncation_step_and_advance, remount_with_batches_of_one, truncation_entry_of,
    walk_the_queues_and_advance,
};
use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::extent_tree::{
    ExtentLowerNodePosition, ExtentUpperLeafEntry, ExtentUpperLeafTarget,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::InodeRecord;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_create_snapshot, publish_sequential_write_of_an_inode, publish_unlink_step, PoolWriter,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{parse_data_unit, parse_index_node};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::model::ObservedFile;
use singlefs_harness::model_comparison::observed_inodes_other_than_the_first_of_file_version;

/// 新池新建文件之后建的两个 inode：号 2 留着 size 0、上段没有条目；号 3 按 inode 号写数据。
const THE_INODE_WITHOUT_DATA: u64 = 2;
const THE_INODE_WITH_DATA: u64 = 3;

/// `length` 个字节、一个 0 都没有（读回里的 0 只来自洞与补零）。
fn content_without_zeros(length: u64, seed: u64) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 251 + 1).expect("1 到 251"))
        .collect()
}

/// `content` 截到 `size` 字节、之后补 0 到 `size`（变长读回的样子）。
fn content_resized_with_zeros(content: &[u8], size: u64) -> Vec<u8> {
    let mut resized = content.to_vec();
    resized.resize(usize::try_from(size).expect("size 装得进 usize"), 0);
    resized
}

/// 新池新建文件之后建 inode 2 与 3（size 0），给 inode 3 按 inode 号顺序写 `content_of_inode_three`（空的不写：inode 3 也留着 size 0）。
fn pool_with_inodes_two_and_three(tag: &str, content_of_inode_three: &[u8]) -> BuiltPool {
    let mut pool = build_pool(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, 2);
    if !content_of_inode_three.is_empty() {
        sequential_write_of_an_inode_and_advance(
            &mut pool,
            THE_INODE_WITH_DATA,
            content_of_inode_three,
        );
    }
    pool
}

/// 同一个进程里按 inode 号顺序写一次，池接到它那一版上。
fn sequential_write_of_an_inode_and_advance(
    pool: &mut BuiltPool,
    inode: u64,
    content: &[u8],
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let written = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        content,
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("按 inode 号写 inode {inode}：{refusal:?}"));
    pool.output = written.clone();
    written
}

/// 改 `inode` 的 size，发不成就停下。
fn change_the_size_and_advance(
    pool: &mut BuiltPool,
    inode: u64,
    new_size: u64,
) -> TransactionOutput {
    publish_size_change_of_and_advance(pool, InodeNumber(inode), new_size)
        .unwrap_or_else(|refusal| panic!("inode {inode} 改成 {new_size} 字节：{refusal:?}"))
}

/// 这一版 extent 树上段（只有根兼叶：号都在第 0 片叶里）里 `inode` 那一条的标签（0 到 4）；没有条目交 `None`。
fn upper_leaf_tag_of(version: &TransactionOutput, inode: u64) -> Option<u8> {
    let leaf = parse_index_node(&version.unit(TransactionUnit::ExtentRoot).bytes)
        .expect("这一版的 extent 树根解得开");
    leaf.entries
        .iter()
        .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("上段叶条目解得开"))
        .find(|entry| entry.inode == inode)
        .map(|entry| match entry.target {
            ExtentUpperLeafTarget::NoDataUnit => 0,
            ExtentUpperLeafTarget::LowerSegmentRoot(_) => 1,
            ExtentUpperLeafTarget::InlineDataUnit(_) => 2,
            ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_) => 3,
            ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_) => 4,
        })
}

/// 这一版里 `inode` 那个文件的数据单元数（上段里没有它为 0）。
fn data_units_of(version: &TransactionOutput, inode: u64) -> usize {
    version
        .file_other_than_the_first(InodeNumber(inode))
        .map_or(0, |file| file.data_pointers.len())
}

/// 这一版里 `inode` 那个文件第 `unit` 个数据单元头里的对象与声明长度。
fn object_and_declared_length_of(version: &TransactionOutput, inode: u64, unit: u64) -> (u64, u16) {
    let header = parse_data_unit(
        &version
            .unit(TransactionUnit::DataOfAFileOtherThanTheFirst {
                inode: InodeNumber(inode),
                unit: DataUnitIndexInFile(unit),
            })
            .bytes,
    )
    .expect("数据单元解得开");
    (header.identity.object, header.declared_length)
}

/// 这一版 inode 树里 `inode` 那条记录。
fn record_of(version: &TransactionOutput, inode: u64) -> InodeRecord {
    version
        .inode_record_of(InodeNumber(inode))
        .unwrap_or_else(|| panic!("inode {inode} 的记录在这一版里"))
}

/// 镜像的两块盘各拷一份（可写挂载要写，不动原镜像）。
fn devices_copied_from(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.clone();
            (*identity, copy)
        })
        .collect()
}

/// 四个读者在 `image`（`version` 是写出它的那一版）上都收下：只读挂载态读者打开 `inode`、从 0 读到 size 读出 `expected`；扫描重建出来的那一版、
/// 可写挂载落到的那一版、发布交回的那一版经模型胶水读 `inode` 都是 `expected`（判决第二节 S5 的胶水按标签与截断条目读）；冷恢复读回第一个文件。
fn assert_every_reader_accepts_and_reads(
    image: &MemoryPool,
    version: &TransactionOutput,
    inode: u64,
    expected: &[u8],
    what: &str,
) {
    let expected_observed = ObservedFile::Decoded(expected.to_vec());
    let mounted =
        mount_read_only(image).unwrap_or_else(|failure| panic!("{what}：只读挂载：{failure:?}"));
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .unwrap_or_else(|failure| panic!("{what}：挂载态读者打开 inode {inode}：{failure:?}"));
    let size = file.inode_record().size;
    let read = file
        .read_at(image, FileOffsetInBytes(0), size)
        .unwrap_or_else(|failure| panic!("{what}：挂载态读者读 inode {inode}：{failure:?}"))
        .bytes;
    assert!(
        read == expected,
        "{what}：挂载态读者读出 {} 字节，期望 {} 字节（首个不同在 {:?}）",
        read.len(),
        expected.len(),
        read.iter()
            .zip(expected)
            .position(|(got, wanted)| got != wanted)
    );
    assert_eq!(
        observed_inodes_other_than_the_first_of_file_version(version).get(&inode),
        Some(&expected_observed),
        "{what}：发布交回的那一版经胶水读 inode {inode}"
    );
    let rebuilt = match rebuild_version(image, &version.root, Some(version.record.clone())) {
        Ok(RebuiltVersion::WithFile(rebuilt)) => rebuilt,
        Ok(other) => panic!("{what}：扫描重建出的不是带文件的一版：{other:?}"),
        Err(failure) => panic!("{what}：扫描重建拒：{failure:?}"),
    };
    assert_eq!(
        observed_inodes_other_than_the_first_of_file_version(&rebuilt).get(&inode),
        Some(&expected_observed),
        "{what}：扫描重建出的那一版经胶水读 inode {inode}"
    );
    let recovered = recover(image, JournalPolicy::Consult).outcome;
    assert!(
        matches!(recovered, RecoveryOutcome::FileRead { .. }),
        "{what}：冷恢复读回第一个文件：{recovered:?}"
    );
    let mut devices = devices_copied_from(image);
    let writable = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|failure| panic!("{what}：可写挂载：{failure:?}"));
    let mounted_version = writable
        .current
        .version_of_trees()
        .unwrap_or_else(|| panic!("{what}：可写挂载落到的一版有树"));
    assert_eq!(
        observed_inodes_other_than_the_first_of_file_version(mounted_version).get(&inode),
        Some(&expected_observed),
        "{what}：可写挂载落到的那一版经胶水读 inode {inode}"
    );
}

/// 交一张池级 checker 一条都不红的镜像。
fn hand_out_a_clean_image(images: &mut PoolCheckerImages, image: &MemoryPool, what: &str) {
    let label = format!("{}-{what}", images.handed_out());
    images.hand_out(
        &label,
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 攻方 H1（判决第二节 S2「粘住的标签 0」）：inode 2（建出来 size 0、上段没有条目）变长到 5000：一次只改记录的发布、这次起写它那一条标签 0
/// （上段根重写），四个读者都读出 5000 个 0；再变长到 9000 只改记录、上段整棵照抄、标签 0 留着；缩到 0（截断意图 + 放 0 个单元的最后一批）
/// 之后标签 0 那一条照旧留着、读回空。每一版池级 checker 一条不红。
pub fn extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_inodes_two_and_three("size-change-sticky-tag-zero", &[]);
    assert_eq!(
        upper_leaf_tag_of(&pool.output, THE_INODE_WITHOUT_DATA),
        None,
        "前提：inode 2 建出来 size 0、上段没有它的条目"
    );
    let extended = change_the_size_and_advance(&mut pool, THE_INODE_WITHOUT_DATA, 5000);
    assert_eq!(
        upper_leaf_tag_of(&extended, THE_INODE_WITHOUT_DATA),
        Some(0),
        "变长到 size > 0 而一个单元都没有：写标签 0"
    );
    assert!(
        extended.rewritten.contains(&TransactionUnit::ExtentRoot),
        "第一次写上标签 0：上段根重写"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &extended,
        THE_INODE_WITHOUT_DATA,
        &[0u8; 5000],
        "inode 2 变长到 5000",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "变长到 5000");
    let extended_again = change_the_size_and_advance(&mut pool, THE_INODE_WITHOUT_DATA, 9000);
    assert!(
        !extended_again
            .rewritten
            .contains(&TransactionUnit::ExtentRoot),
        "标签 0 已经在：再变长只改记录、上段整棵照抄：{:?}",
        extended_again.rewritten
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &extended_again,
        THE_INODE_WITHOUT_DATA,
        &[0u8; 9000],
        "inode 2 再变长到 9000",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "再变长到 9000");
    let intent = change_the_size_and_advance(&mut pool, THE_INODE_WITHOUT_DATA, 0);
    let (_, named) = truncation_entry_of(&intent).expect("缩到 0 走截断意图");
    assert_eq!(
        (named.inode.0, named.target_size_in_bytes),
        (THE_INODE_WITHOUT_DATA, 0),
        "截断条目点名 inode 2、目标 0"
    );
    let steps = walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    assert_eq!(steps.len(), 1, "放 0 个单元的最后一批一次做完");
    assert!(
        truncation_entry_of(&pool.output).is_none(),
        "最后一批删了截断条目"
    );
    assert_eq!(
        upper_leaf_tag_of(&pool.output, THE_INODE_WITHOUT_DATA),
        Some(0),
        "缩到 0 之后标签 0 那一条照旧留着"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &pool.output.clone(),
        THE_INODE_WITHOUT_DATA,
        &[],
        "inode 2 缩到 0",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "缩到 0");
}

/// 攻方 H2（K5）：inode 3（内联一个单元、100 字节）变长到 5000：单元 0 补零重写成声明长度 5000、单元头的对象是 inode 3，标签仍是 2（内联、没有洞），
/// 四个读者读出前 100 字节加 4900 个 0。池级 checker 一条不红。
pub fn extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(100, 3);
    let mut pool = pool_with_inodes_two_and_three("size-change-inside-one-unit", &content);
    let extended = change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 5000);
    assert_eq!(
        object_and_declared_length_of(&extended, THE_INODE_WITH_DATA, 0),
        (THE_INODE_WITH_DATA, 5000),
        "单元 0 补零重写：对象是 inode 3、声明长度 5000"
    );
    assert_eq!(upper_leaf_tag_of(&extended, THE_INODE_WITH_DATA), Some(2));
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &extended,
        THE_INODE_WITH_DATA,
        &content_resized_with_zeros(&content, 5000),
        "inode 3 变长到 5000",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "变长到 5000");
}

/// 攻方 H3（K5）：inode 3（100 字节）变长跨单元到 2 个净荷 + 7：单元 0 补零重写到满净荷，之后到 size 是洞，标签换成 4（内联、之后是洞），
/// 单元数仍是 1；四个读者读出前 100 字节、之后到 size 全 0。池级 checker 一条不红。
pub fn extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(100, 5);
    let mut pool = pool_with_inodes_two_and_three("size-change-across-units", &content);
    let new_size = 2 * payload_capacity() + 7;
    let extended = change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, new_size);
    assert_eq!(
        (
            upper_leaf_tag_of(&extended, THE_INODE_WITH_DATA),
            data_units_of(&extended, THE_INODE_WITH_DATA),
            object_and_declared_length_of(&extended, THE_INODE_WITH_DATA, 0),
        ),
        (
            Some(4),
            1,
            (
                THE_INODE_WITH_DATA,
                u16::try_from(payload_capacity()).expect("净荷装得进 u16")
            )
        ),
        "标签 4、一个单元、单元 0 补零到满净荷"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &extended,
        THE_INODE_WITH_DATA,
        &content_resized_with_zeros(&content, new_size),
        "inode 3 变长到 2 个净荷 + 7",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "变长跨单元");
}

/// 攻方 H4（K2、K3、K11）：inode 3（100 字节）截到 50：意图那次单元 0 重写成声明长度 50、记录 size 50、截断条目点名 inode 3（对象出生代、目标 50），
/// 读者读出前 50 字节；走空一次最后一批（放 0 个单元）删掉条目。两版池级 checker 都一条不红。
pub fn truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(100, 7);
    let mut pool =
        pool_with_inodes_two_and_three("size-change-truncation-inside-one-unit", &content);
    let object_birth = record_of(&pool.output, THE_INODE_WITH_DATA).object_birth;
    let intent = change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 50);
    let (_, named) = truncation_entry_of(&intent).expect("截短走截断意图");
    assert_eq!(
        (
            named.inode.0,
            named.object_birth,
            named.target_size_in_bytes
        ),
        (THE_INODE_WITH_DATA, object_birth, 50),
        "截断条目点名 inode 3、它的对象出生代、目标 50"
    );
    assert_eq!(
        object_and_declared_length_of(&intent, THE_INODE_WITH_DATA, 0),
        (THE_INODE_WITH_DATA, 50),
        "意图那次单元 0 重写成声明长度 50"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &intent,
        THE_INODE_WITH_DATA,
        &content[..50],
        "inode 3 截到 50 的意图那一版",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "截到 50 的意图");
    let steps = walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    assert_eq!(steps.len(), 1, "一个放 0 个单元的最后一批");
    assert!(truncation_entry_of(&pool.output).is_none());
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &pool.output.clone(),
        THE_INODE_WITH_DATA,
        &content[..50],
        "inode 3 截到 50 走空之后",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "截到 50 走空之后");
}

/// 攻方 H5（K2、K3，判决 m4-trunc-r2 R2）：inode 3（三个满单元）截到 100：意图那次单元 0 重写成声明长度 100、截断字段写成（这次 txg, 100）、
/// 三个单元都挂着，读者读出前 100 字节；批大小 1：一批放到两个、最后一批放到一个并删条目。每一版池级 checker 一条不红。
pub fn truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(3 * payload_capacity(), 11);
    let mut pool =
        pool_with_inodes_two_and_three("size-change-truncation-of-three-units", &content);
    let intent = change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 100);
    let record = record_of(&intent, THE_INODE_WITH_DATA);
    assert_eq!(
        (
            record.truncation_field.truncation_txg(),
            record.truncation_field.truncated_size_in_bytes(),
            data_units_of(&intent, THE_INODE_WITH_DATA),
            object_and_declared_length_of(&intent, THE_INODE_WITH_DATA, 0),
        ),
        (
            intent.root.checkpoint_txg,
            100,
            3,
            (THE_INODE_WITH_DATA, 100)
        ),
        "截断字段（这次 txg, 100）、三个单元都挂着、单元 0 重写成 100"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &intent,
        THE_INODE_WITH_DATA,
        &content[..100],
        "inode 3 截到 100 的意图那一版",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "截到 100 的意图");
    let batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert_eq!(
        (
            data_units_of(&batch, THE_INODE_WITH_DATA),
            truncation_entry_of(&batch).is_some()
        ),
        (2, true),
        "一批放到两个、条目还在"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &batch,
        THE_INODE_WITH_DATA,
        &content[..100],
        "inode 3 截断的一批之后",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "截断的一批之后");
    let final_batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert_eq!(
        (
            data_units_of(&final_batch, THE_INODE_WITH_DATA),
            truncation_entry_of(&final_batch).is_some(),
            upper_leaf_tag_of(&final_batch, THE_INODE_WITH_DATA),
        ),
        (1, false, Some(2)),
        "最后一批放到一个、删条目、内联"
    );
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &final_batch,
        THE_INODE_WITH_DATA,
        &content[..100],
        "inode 3 截断的最后一批之后",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "截断的最后一批之后");
}

/// K9 叶边界对任意 inode（判决第二节 S3「意图那次罩着被重写单元的叶到根恒重写」）：inode 3（150 个单元）截到单元 143 中间（留下 144 个，
/// 单元 143 在第 0 片下段叶的末尾）：意图那次重写 inode 3 的单元 143、它的第 0 片下段叶与下段根、上段根；之后那一批截掉第 1 片叶，读回前
/// 143 × 净荷 + 9 字节。池级 checker 一条不红。
pub fn truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(150 * payload_capacity(), 13);
    let mut pool = pool_with_inodes_two_and_three("size-change-leaf-boundary", &content);
    let new_size = 143 * payload_capacity() + 9;
    let intent = change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, new_size);
    let the_other_inode = InodeNumber(THE_INODE_WITH_DATA);
    for role in [
        TransactionUnit::DataOfAFileOtherThanTheFirst {
            inode: the_other_inode,
            unit: DataUnitIndexInFile(143),
        },
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
            inode: the_other_inode,
            position: ExtentLowerNodePosition { level: 0, index: 0 },
        },
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
            inode: the_other_inode,
            position: ExtentLowerNodePosition { level: 1, index: 0 },
        },
        TransactionUnit::ExtentRoot,
    ] {
        assert!(
            intent.rewritten.contains(&role),
            "意图那次重写 {role:?}：{:?}",
            intent.rewritten
        );
    }
    walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    assert_eq!(data_units_of(&pool.output, THE_INODE_WITH_DATA), 144);
    assert_every_reader_accepts_and_reads(
        &pool.memory_pool(),
        &pool.output.clone(),
        THE_INODE_WITH_DATA,
        &content[..usize::try_from(new_size).expect("长度")],
        "inode 3 截到叶边界走空之后",
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "截到叶边界走空之后");
}

/// 批间停下再重挂续做（判决第二节 S3、S4「续做按号认领」）：inode 3（三个满单元）截到 100、批大小 1 发完一批就关掉，可写挂载的续做照截断条目
/// 点名的 inode 3 认领、走完最后一批（单元放到一个、条目删掉），重挂之后四个读者读出前 100 字节。池级 checker 一条不红。
pub fn a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(3 * payload_capacity(), 17);
    let mut pool = pool_with_inodes_two_and_three("size-change-continued-at-the-remount", &content);
    change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 100);
    publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert!(
        truncation_entry_of(&pool.output).is_some(),
        "前提：停在批间"
    );
    let (mounted, _devices) = remount_with_batches_of_one(&mut pool);
    let continued = mounted
        .current
        .version_of_trees()
        .expect("续做之后的一版有树")
        .clone();
    assert_eq!(
        (
            data_units_of(&continued, THE_INODE_WITH_DATA),
            truncation_entry_of(&continued).is_some(),
        ),
        (1, false),
        "续做认领 inode 3、走完最后一批"
    );
    let image = pool.memory_pool();
    assert_every_reader_accepts_and_reads(
        &image,
        &continued,
        THE_INODE_WITH_DATA,
        &content[..100],
        "重挂续做之后",
    );
    hand_out_a_clean_image(images, &image, "重挂续做之后");
}

/// 快照下缩短别的 inode（判决第二节 S3「快照下被换下的上一版角色进 deadlist 那条链」，K4）：inode 3（三个满单元，生在建快照之前）截到 100：
/// 意图那次换下的单元 0 旧那一份、最后一批放掉的单元 1、2 的映射 key 都进 deadlist、映射条目留存，它们的落点都不在释放清单里；读回前 100 字节。
/// 池级 checker 一条不红。
pub fn truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(3 * payload_capacity(), 19);
    let mut pool = pool_with_inodes_two_and_three("size-change-under-a-snapshot", &content);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let created = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &previous,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("建快照")
    };
    pool.output = created.clone();
    let mapping_key_of = |version: &TransactionOutput, unit: u64| {
        version
            .mapped_units
            .iter()
            .find(|(role, _)| {
                *role
                    == TransactionUnit::DataOfAFileOtherThanTheFirst {
                        inode: InodeNumber(THE_INODE_WITH_DATA),
                        unit: DataUnitIndexInFile(unit),
                    }
            })
            .map(|(_, key)| key.clone())
            .expect("inode 3 的数据单元进映射")
    };
    let keys_referenced_by_the_snapshot: Vec<Vec<u8>> =
        (0..3).map(|unit| mapping_key_of(&created, unit)).collect();
    let intent = change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 100);
    assert!(
        intent
            .deadlist_entries
            .iter()
            .any(|entry| entry.mapping_key == keys_referenced_by_the_snapshot[0]),
        "意图那次换下的单元 0 旧那一份进 deadlist"
    );
    let steps = walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    assert_eq!(steps.len(), 1, "产品批大小：一次最后一批放掉单元 1、2");
    for (unit, mapping_key) in keys_referenced_by_the_snapshot.iter().enumerate() {
        assert!(
            pool.output
                .deadlist_entries
                .iter()
                .any(|entry| &entry.mapping_key == mapping_key)
                && pool
                    .output
                    .mapping_entries_kept_for_the_snapshots
                    .iter()
                    .any(|(kept_key, _)| kept_key == mapping_key),
            "单元 {unit} 旧那一份在 deadlist 里、映射条目留存"
        );
    }
    let image = pool.memory_pool();
    assert_every_reader_accepts_and_reads(
        &image,
        &pool.output.clone(),
        THE_INODE_WITH_DATA,
        &content[..100],
        "快照下截到 100 走空之后",
    );
    hand_out_a_clean_image(images, &image, "快照下截到 100 走空之后");
}

/// 带洞时被 unlink（判决第二节 S2「unlink 的批之后标签从 3、4 换回 1、2」）：inode 3（三个满单元）变长到 5 个净荷（标签 3：下段根、之后是洞），
/// 再 unlink、批大小 1：意图那一版标签仍是 3，第一批放到两个、size 改成 2 个净荷、标签换回 1；最后一批删掉记录与条目。每一版池级 checker 一条不红。
pub fn unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it(
    images: &mut PoolCheckerImages,
) {
    let content = content_without_zeros(3 * payload_capacity(), 23);
    let mut pool = pool_with_inodes_two_and_three("size-change-unlink-with-a-hole", &content);
    let extended =
        change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 5 * payload_capacity());
    assert_eq!(
        upper_leaf_tag_of(&extended, THE_INODE_WITH_DATA),
        Some(3),
        "标签 3"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "变长到 5 个净荷");
    let intent = publish_the_intent_of(&mut pool, THE_INODE_WITH_DATA);
    assert_eq!(
        upper_leaf_tag_of(&intent, THE_INODE_WITH_DATA),
        Some(3),
        "意图那一版不动 extent 树"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "unlink 的意图");
    let batch = unlink_step_of_the_inode_with_data_in_batches_of_one(&mut pool);
    assert_eq!(
        (
            data_units_of(&batch, THE_INODE_WITH_DATA),
            record_of(&batch, THE_INODE_WITH_DATA).size,
            upper_leaf_tag_of(&batch, THE_INODE_WITH_DATA),
        ),
        (2, 2 * payload_capacity(), Some(1)),
        "第一批放到两个、size 2 个净荷、标签换回 1"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "unlink 的第一批");
    unlink_step_of_the_inode_with_data_in_batches_of_one(&mut pool);
    let final_batch = unlink_step_of_the_inode_with_data_in_batches_of_one(&mut pool);
    assert_eq!(
        (
            final_batch.inode_record_of(InodeNumber(THE_INODE_WITH_DATA)),
            upper_leaf_tag_of(&final_batch, THE_INODE_WITH_DATA),
        ),
        (None, None),
        "最后一批删掉记录与条目"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "unlink 的最后一批");
}

/// inode 3 unlink 的下一步（批大小 1），池接到它那一版上。
fn unlink_step_of_the_inode_with_data_in_batches_of_one(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let step = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(THE_INODE_WITH_DATA),
        batches_of_one(),
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("unlink 的下一步");
    pool.output = step.clone();
    step
}

/// 两个 inode 的回退（判决第二节 S4「回退合不出两条截断条目」）：inode 3 截到批间那一版（条目点名 3），走完；再截 inode 2（条目点名 2）；
/// 回退到 inode 3 批间那一版：回退那次 logged_ops 里恰好一条截断条目、点名 inode 3，inode 2 取 R_old 的（三个单元、原来的 size）；
/// 走空续做按号认领 inode 3 截到底。池级 checker 一条不红。
pub fn a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode(
    images: &mut PoolCheckerImages,
) {
    let content_of_inode_two = content_without_zeros(3 * payload_capacity(), 29);
    let content_of_inode_three = content_without_zeros(3 * payload_capacity(), 31);
    let mut pool = pool_with_inodes_two_and_three(
        "size-change-rollback-of-two-inodes",
        &content_of_inode_three,
    );
    sequential_write_of_an_inode_and_advance(
        &mut pool,
        THE_INODE_WITHOUT_DATA,
        &content_of_inode_two,
    );
    change_the_size_and_advance(&mut pool, THE_INODE_WITH_DATA, 100);
    let between_the_batches_of_inode_three =
        publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    walk_the_queues_and_advance(&mut pool, batches_of_one());
    change_the_size_and_advance(&mut pool, THE_INODE_WITHOUT_DATA, 100);
    let rolled_back = roll_back_to(&mut pool, &between_the_batches_of_inode_three);
    let truncation_entries: Vec<u64> = rolled_back
        .logged_operations
        .iter()
        .filter_map(|entry| match entry.operation {
            singlefs_core::logged_operations::LoggedOperation::Truncation(intent) => {
                Some(intent.inode.0)
            }
            singlefs_core::logged_operations::LoggedOperation::Compaction(_) => None,
        })
        .collect();
    assert_eq!(
        truncation_entries,
        [THE_INODE_WITH_DATA],
        "回退那次恰好一条截断条目、点名 inode 3"
    );
    assert_eq!(
        (
            data_units_of(&rolled_back, THE_INODE_WITHOUT_DATA),
            record_of(&rolled_back, THE_INODE_WITHOUT_DATA).size,
        ),
        (3, 3 * payload_capacity()),
        "inode 2 取 R_old 的"
    );
    let steps = walk_the_queues_and_advance(&mut pool, batches_of_one());
    assert_eq!(steps.len(), 1, "续做一个最后一批");
    assert_eq!(
        (
            data_units_of(&pool.output, THE_INODE_WITH_DATA),
            truncation_entry_of(&pool.output).is_some(),
        ),
        (1, false)
    );
    let image = pool.memory_pool();
    assert_every_reader_accepts_and_reads(
        &image,
        &pool.output.clone(),
        THE_INODE_WITH_DATA,
        &content_of_inode_three[..100],
        "回退之后续做到底：inode 3",
    );
    assert_every_reader_accepts_and_reads(
        &image,
        &pool.output.clone(),
        THE_INODE_WITHOUT_DATA,
        &content_of_inode_two,
        "回退之后续做到底：inode 2",
    );
    hand_out_a_clean_image(images, &image, "回退之后续做到底");
}

/// 判决第二节 S2 的必红世界：起点镜像上 inode 2（建出来 size 0、上段没有条目）的记录 size 直接改成 5000（不经写者，重封叶、校验和补到根）：
/// 挂载态读者打开它拒、扫描重建拒、可写挂载拒（「有记录、没有条目、size > 0」三处读者判同一个结论），池级 checker 只红 I-4.6
/// （GPU 检查内核同判，checker 档那一侧走 GPU 时一并核）。
pub fn a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count(
    images: &mut PoolCheckerImages,
) {
    let pool = pool_with_inodes_two_and_three("size-change-record-without-an-entry", &[]);
    let mut image = pool.memory_pool();
    change_the_inode_record(&mut image, THE_INODE_WITHOUT_DATA, |bytes| {
        let mut record = InodeRecord::parse(bytes).expect("记录解得开");
        record.size = 5000;
        bytes.copy_from_slice(&record.to_bytes());
    });
    let mounted = mount_read_only(&image).expect("只读挂载：第一个文件那一版照收");
    assert!(
        mounted
            .mounted
            .open_file(&image, InodeNumber(THE_INODE_WITHOUT_DATA))
            .is_err(),
        "挂载态读者打开 inode 2 拒"
    );
    // 改了记录、校验和补到了根槽：按改过之后那条最新的根重建（记录照写出那一版的那一条顶着）。
    let rebuilt = rebuild_version(
        &image,
        &crate::common_pointer_head_corruption::newest_root(&image),
        Some(pool.output.record.clone()),
    );
    assert!(
        matches!(
            &rebuilt,
            Err(singlefs_core::recovery::RebuildVersionFailure::Walk(
                singlefs_core::recovery::RecoveryFailure::InvariantViolated { detail, .. }
            )) if *detail == singlefs_core::recovery::RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE
        ),
        "扫描重建拒成「有记录、没有条目、size > 0」：{rebuilt:?}"
    );
    let mut devices = devices_copied_from(&image);
    assert!(
        mount_writable(&parameters(), &mut devices).is_err(),
        "可写挂载拒"
    );
    images.hand_out(
        "有记录没条目而 size 5000",
        || image.clone(),
        PoolCheckerExpectation::exactly_these_violated(&["I-4.6"]),
    );
}
