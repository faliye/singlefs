//! 两档共用场景；池级 checker 镜像交给 checker 档给定状态引擎两段判。
//! 按 inode 号写数据（里程碑「删除与多次 COW」一·1，用户 2026-10-18 定「三条都新写，先给 core 加多 inode 写数据」）的验收：
//! 会话的 `UserChange::OverwriteOfAnInode`、`UserChange::SequentialWriteOfAnInode` 与它们落到的
//! `transaction::publish_overwrite_of_an_inode`、`transaction::publish_sequential_write_of_an_inode`。
//!
//! 四件事各一条用例：两个 inode 交错各写几次、中间关掉再可写挂载两次，读回各自逐字节等于最后写的；写一个 inode 那次发布不碰别的文件的
//! 数据单元与下段（盘上那几个槽的字节一个不变、它们不在这次换下的落点里）；inode 号落在上段第一片叶之外时上段长一层，重挂之后每个文件都读得回；
//! 对不在 inode 树里的号写在任何写之前拒成 `WriteToAnInodeThatIsNotInTheTree`。另一条钉住第一个文件之外有文件带数据单元时删那个文件、
//! 再删第一个文件都做成（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3；此前拒成「条款没写」）。
//! 每一版都交池级 checker 判一遍：按 inode 号写出来的镜像一条违例都不许有。
//! 代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 2 条补的四条（正推腿 `m3-close-w5-code-r1-sonnet-output.md`
//! W1、W5 两节列的缺口）：别的号 nlink = 0 时写入口拒（缺口 A）；第一个文件删完、上段只剩零条目根之后写删之前就有的号，零条目根换下（缺口 G）；
//! 别的号的数据单元与下段节点按结构种类记的写与第一个文件同一套（新加的穷举分支，`write_accounting.rs`）；空间准入数别的号的单元与第一个文件同一套
//! （新加的穷举分支，`admission.rs`）。

#![allow(dead_code, reason = "两档各只调用其中一部分场景与帮手")]

use crate::common::{
    build_pool, disk_snapshot, file_content, parameters, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::{units_of_a_publish_to_land, MetadataBlocks, UnitsOfAPublishToLand};
use singlefs_core::extent_tree::{ExtentLowerNodePosition, ExtentUpperNodePosition};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite_of_an_inode, publish_sequential_write_of_an_inode,
    publish_unlink_intent, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::write_accounting::WrittenStructureKind;
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 按 inode 号写的那几版的写入时间：比池里第一个文件晚，建 inode 那次之后。
const WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER: u64 = FIXED_WRITE_TIME_SECONDS + 600;
/// 建 inode 那次的写入时间。
const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = FIXED_WRITE_TIME_SECONDS + 60;
/// 这个进程（新池新建文件与它之后在同一个进程里的发布）的实例代号。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
/// 上段一片叶罩 143 个 inode 号（`singlefs_format::EXTENT_TREE_UPPER_LEAF_INODES`）：号 150 落在第二片叶上，写它上段要长一层。
const INODE_PAST_THE_FIRST_UPPER_LEAF: u64 = 150;

/// 定长的伪随机字节流（xorshift64，固定种子）：错位一个字节就对不上。
fn content_of(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

fn payload_capacity() -> usize {
    data_unit_payload_capacity()
}

/// 新池新建文件之后，在同一个进程、实例 1 里建 `count` 个 size 0 的 inode（号 2 起连着发）。
fn pool_with_inodes_created(tag: &str, count: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            count,
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("建 inode")
    };
    pool.output = output;
    pool
}

/// 同一个进程里按 inode 号顺序写一版，发布完把它记成池的当前版。
fn sequential_write_in_process(
    pool: &mut BuiltPool,
    inode: u64,
    content: &[u8],
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write_of_an_inode(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(inode),
            content,
            WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("按 inode 号顺序写")
    };
    pool.output = output.clone();
    output
}

/// 同一个进程里按 inode 号覆盖写一版（一个数据单元以内），发布完把它记成池的当前版。
fn overwrite_in_process(pool: &mut BuiltPool, inode: u64, content: &[u8]) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite_of_an_inode(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(inode),
            content,
            WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("按 inode 号覆盖写")
    };
    pool.output = output.clone();
    output
}

/// 关掉镜像（进程退出）、重开可写挂载，交回接着这次挂载的会话与它的盘。
fn session_after_a_writable_remount(
    pool: &mut BuiltPool,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

/// 经会话按 inode 号覆盖写一次。
fn overwrite_through_the_session(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
    inode: u64,
    content: &[u8],
) {
    session
        .publish_user_change(
            devices,
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(inode),
                content,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .unwrap_or_else(|refusal| panic!("会话按 inode 号覆盖写 inode {inode}：{refusal:?}"));
}

/// 经会话按 inode 号顺序写一次。
fn sequential_write_through_the_session(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
    inode: u64,
    content: &[u8],
) {
    session
        .publish_user_change(
            devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(inode),
                content,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .unwrap_or_else(|refusal| panic!("会话按 inode 号顺序写 inode {inode}：{refusal:?}"));
}

/// 只读挂载这张镜像一次（扫环只这一次）、逐个打开 `inodes` 里的号、各从偏移 0 读整个文件（挂载态的读，`mounted_read`），按 `inodes` 的次序交回。
fn contents_read_back(image: &MemoryPool, inodes: &[u64]) -> Vec<Vec<u8>> {
    let mounted = mount_read_only(image).expect("只读挂载");
    inodes
        .iter()
        .map(|inode| {
            let file = mounted
                .mounted
                .open_file(image, InodeNumber(*inode))
                .unwrap_or_else(|failure| panic!("打开 inode {inode}：{failure:?}"));
            let size = file.inode_record().size;
            file.read_at(image, FileOffsetInBytes(0), size)
                .unwrap_or_else(|failure| panic!("读 inode {inode} 的 {size} 字节：{failure:?}"))
                .bytes
        })
        .collect()
}

/// 一版里 `inode` 那个文件每个单元在盘上占的那几个槽（数据单元两槽、下段节点一槽，`TransactionUnit::span_slots`）的字节，
/// 两块盘各一份，按数据单元、再下段节点的次序：第一个文件读 `data_pointers` 与 `extent_tree.lower_nodes`，别的读它那一项。
fn bytes_on_disk_of_the_units_of(
    image: &MemoryPool,
    version: &TransactionOutput,
    inode: u64,
) -> Vec<Vec<u8>> {
    let (data_slots, lower_node_slots): (Vec<_>, Vec<_>) = if inode == FIRST_INODE_NUMBER {
        (
            version
                .data_pointers
                .iter()
                .map(|pointer| pointer.locations)
                .collect(),
            version
                .extent_tree
                .lower_nodes
                .iter()
                .map(|(_, pointer)| pointer.locations)
                .collect(),
        )
    } else {
        let file = version
            .file_other_than_the_first(InodeNumber(inode))
            .unwrap_or_else(|| panic!("这一版里 inode {inode} 有数据单元"));
        (
            file.data_pointers
                .iter()
                .map(|pointer| pointer.locations)
                .collect(),
            file.extent_lower_nodes
                .iter()
                .map(|(_, pointer)| pointer.locations)
                .collect(),
        )
    };
    let bytes_of = |locations: [singlefs_core::pointer::LocationEntry; 2], length: u64| {
        locations
            .iter()
            .map(|location| {
                PoolReader::read(
                    image,
                    location.device,
                    location.slot.to_device_offset(),
                    usize::try_from(length).expect("一个单元的字节数"),
                )
                .expect("单元的落点读得到")
            })
            .collect::<Vec<Vec<u8>>>()
    };
    data_slots
        .into_iter()
        .flat_map(|locations| bytes_of(locations, DATA_UNIT_BYTES))
        .chain(
            lower_node_slots
                .into_iter()
                .flat_map(|locations| bytes_of(locations, NODE_BYTES)),
        )
        .collect()
}

/// 两个 inode 交错各写三次（覆盖写），中间关掉再可写挂载两次（三个实例上各写一轮：新池新建文件那个进程里一轮，两次重开可写挂载的会话里各一轮）：
/// 每次可写挂载都从盘上重建现行那一版（第一个文件之外两个文件的数据单元与上段条目一起读回来），下一次写其中一个照抄另一个；
/// 最后读回 inode 2、inode 3 各自逐字节等于最后写的那一份，第一个文件还是新池新建文件写的那一份，池级 checker 一条违例都没有。
pub fn two_inodes_overwritten_in_turn_across_two_writable_remounts_each_read_back_what_was_last_written_to_it(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_inodes_created("writes-by-inode-number-in-turn", 2);
    let contents_of_inode_two = [
        content_of(1000, 0x2001),
        content_of(payload_capacity(), 0x2002),
        content_of(7, 0x2003),
    ];
    let contents_of_inode_three = [
        content_of(2000, 0x3001),
        content_of(1, 0x3002),
        content_of(payload_capacity() - 1, 0x3003),
    ];
    // 新池新建文件那个进程里一轮：先写 inode 2、再写 inode 3。
    overwrite_in_process(&mut pool, 2, &contents_of_inode_two[0]);
    overwrite_in_process(&mut pool, 3, &contents_of_inode_three[0]);
    // 重开可写挂载两次，各一轮；这两轮先写 inode 3、再写 inode 2：上段那片叶里两条条目轮流一条重写、一条照抄。
    for (content_of_inode_two, content_of_inode_three) in contents_of_inode_two[1..]
        .iter()
        .zip(&contents_of_inode_three[1..])
    {
        let (mut session, mut devices) = session_after_a_writable_remount(&mut pool);
        overwrite_through_the_session(&mut session, &mut devices, 3, content_of_inode_three);
        overwrite_through_the_session(&mut session, &mut devices, 2, content_of_inode_two);
    }
    let image = pool.memory_pool();
    assert_eq!(
        contents_read_back(&image, &[2, 3, FIRST_INODE_NUMBER]),
        vec![
            contents_of_inode_two[2].clone(),
            contents_of_inode_three[2].clone(),
            file_content(),
        ],
        "inode 2、inode 3 各读回最后一次写的那一份；第一个文件没被写过，还是新池新建文件那一份"
    );
    hand_out_a_clean_image(
        images,
        &image,
        "按 inode 号写出来的镜像池级 checker 一条违例都没有",
    );
}

/// 写 inode 2 那次发布不碰别的文件：inode 3（跨三个单元、有下段）与第一个文件的每个数据单元、下段节点在盘上的字节一个不变，
/// 它们的指针在这一版里与上一版相同、不在这次换下的落点里、不是这次重写的角色；读回 inode 3 仍是它写的那一份。
pub fn writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_inodes_created("writes-by-inode-number-untouched", 2);
    let content_of_inode_three = content_of(2 * payload_capacity() + 100, 0x3101);
    sequential_write_in_process(&mut pool, 3, &content_of_inode_three);
    overwrite_in_process(&mut pool, 2, &content_of(500, 0x2101));
    let before = pool.output.clone();
    let image_before = pool.memory_pool();
    let after = overwrite_in_process(&mut pool, 2, &content_of(900, 0x2102));
    let image_after = pool.memory_pool();
    for inode in [FIRST_INODE_NUMBER, 3] {
        assert_eq!(
            bytes_on_disk_of_the_units_of(&image_after, &after, inode),
            bytes_on_disk_of_the_units_of(&image_before, &before, inode),
            "写 inode 2 之后 inode {inode} 的数据单元与下段节点在盘上的字节一个不变"
        );
    }
    assert_eq!(
        (
            after.file_other_than_the_first(InodeNumber(3)),
            &after.data_pointers,
            &after.extent_tree.lower_nodes,
        ),
        (
            before.file_other_than_the_first(InodeNumber(3)),
            &before.data_pointers,
            &before.extent_tree.lower_nodes,
        ),
        "inode 3 与第一个文件的数据指针、下段节点指针照抄上一版"
    );
    let untouched_slots: Vec<u64> = before
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations[0].slot.0)
        .chain(
            before
                .file_other_than_the_first(InodeNumber(3))
                .expect("inode 3 有数据单元")
                .data_pointers
                .iter()
                .map(|pointer| pointer.locations[0].slot.0),
        )
        .collect();
    assert!(
        after
            .released
            .iter()
            .all(|placement| !untouched_slots.contains(&placement.slot.0)),
        "这次换下的落点里没有 inode 3 与第一个文件的数据单元：{:?}",
        after.released
    );
    assert!(
        after.rewritten.iter().all(|role| !matches!(
            role,
            TransactionUnit::Data(_) | TransactionUnit::ExtentLowerNode(_)
        ) && !matches!(
            role,
            TransactionUnit::DataOfAFileOtherThanTheFirst { inode, .. }
                | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { inode, .. }
                if inode.0 == 3
        )),
        "这次重写的角色里没有第一个文件与 inode 3 的单元：{:?}",
        after.rewritten
    );
    assert_eq!(
        contents_read_back(&image_after, &[3]),
        vec![content_of_inode_three],
        "读回 inode 3 仍是它写的那一份"
    );
    hand_out_a_clean_image(images, &image_after, "池级 checker 一条违例都没有");
}

/// inode 号落在上段第一片叶之外（号 150，上段一片叶罩 143 个号）：写它那次上段从根兼叶长成两层（旧根照抄成第一片叶、新根在上面重写），
/// 之后再写 inode 2 只重写第一片叶到根那一串；重开可写挂载（从盘上重建两层的上段）之后再写一次 inode 150，三个文件都读得回、
/// 池级 checker 一条违例都没有。
pub fn a_write_to_an_inode_past_the_first_upper_leaf_grows_the_upper_segment_and_every_file_reads_back_after_a_remount(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_inodes_created(
        "writes-by-inode-number-past-the-first-upper-leaf",
        INODE_PAST_THE_FIRST_UPPER_LEAF,
    );
    let content_of_inode_two = content_of(3000, 0x2201);
    let content_of_the_inode_past_the_first_upper_leaf = content_of(payload_capacity() + 1, 0x9601);
    let after_the_growth = sequential_write_in_process(
        &mut pool,
        INODE_PAST_THE_FIRST_UPPER_LEAF,
        &content_of(4000, 0x9600),
    );
    assert_eq!(
        after_the_growth.extent_tree.upper_root().level,
        1,
        "写了号 150 之后上段根在第 1 层"
    );
    overwrite_in_process(&mut pool, 2, &content_of_inode_two);
    {
        let (mut session, mut devices) = session_after_a_writable_remount(&mut pool);
        sequential_write_through_the_session(
            &mut session,
            &mut devices,
            INODE_PAST_THE_FIRST_UPPER_LEAF,
            &content_of_the_inode_past_the_first_upper_leaf,
        );
    }
    let image = pool.memory_pool();
    assert_eq!(
        contents_read_back(
            &image,
            &[INODE_PAST_THE_FIRST_UPPER_LEAF, 2, FIRST_INODE_NUMBER]
        ),
        vec![
            content_of_the_inode_past_the_first_upper_leaf,
            content_of_inode_two,
            file_content(),
        ],
        "号 150 读回重挂之后写的那一份，inode 2 读回它写的那一份，第一个文件还是新池新建文件那一份"
    );
    hand_out_a_clean_image(images, &image, "池级 checker 一条违例都没有");
}

/// 对不在 inode 树里的号（只建了 inode 2、3，写号 99）按 inode 号写：会话交回 `Publish`，原因是 `WriteToAnInodeThatIsNotInTheTree`，
/// 在任何写之前——系统配置槽、根环、录制流步数都不变。
pub fn a_write_to_an_inode_number_that_is_not_in_the_inode_tree_is_refused_before_anything_reaches_the_disk(
) {
    let mut pool = pool_with_inodes_created("writes-by-inode-number-not-in-the-tree", 2);
    let (mut session, mut devices) = session_after_a_writable_remount(&mut pool);
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refusal = session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(99),
                content: &content_of(100, 0x9901),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .expect_err("号 99 不在 inode 树里");
    let UserChangeRefused::Publish { cause, .. } = refusal else {
        panic!("该是发布被拒（不是空间不够），实际 {refusal:?}");
    };
    assert!(
        matches!(
            cause,
            PublishError::WriteToAnInodeThatIsNotInTheTree { inode: 99 }
        ),
        "原因是号 99 不在 inode 树里，实际 {cause:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前：系统配置槽、根环、录制流步数都不变"
    );
}

/// 第一个文件之外有文件带数据单元（inode 2 写过、inode 3 只建了）时，删那个文件、再删第一个文件都做成（判决
/// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3，此前在任何写之前拒成「条款没写」）：经会话 unlink，
/// 每次一路走空；删完 inode 2 之后它的单元与上段条目都没了、第一个文件读回不变，删完第一个文件之后剩 size 0 的 inode 3，extent 树是零条目根，
/// 这一版仍是带文件的一版（inode 树有条目）；每一版池级 checker 一条违例都没有。
// harness-test-granularity:one-scenario 同一个池上先删别的文件、再删第一个文件，是「多文件删除」一个场景的两步
pub fn unlink_while_a_file_other_than_the_first_holds_data_units_deletes_it_and_then_the_first_file(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_inodes_created("writes-by-inode-number-unlink-of-several-files", 2);
    overwrite_in_process(&mut pool, 2, &content_of(500, 0x2301));
    let (mut session, mut devices) = session_after_a_writable_remount(&mut pool);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(2),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .unwrap_or_else(|refusal| panic!("unlink inode 2、一路走空：{refusal:?}"));
    let after_inode_two = session
        .current
        .file_version()
        .expect("删完 inode 2 还是带文件的一版")
        .clone();
    assert!(
        after_inode_two.files_other_than_the_first.is_empty()
            && after_inode_two.inode_record_of(InodeNumber(2)).is_none(),
        "inode 2 的单元与记录都没了"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "删完 inode 2");
    assert_eq!(
        contents_read_back(&pool.memory_pool(), &[FIRST_INODE_NUMBER]),
        vec![file_content()],
        "第一个文件读回不变"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .unwrap_or_else(|refusal| panic!("unlink 第一个文件、一路走空：{refusal:?}"));
    let after_the_first_file = session
        .current
        .file_version()
        .expect("删完第一个文件而 inode 3 还在：带文件的一版");
    assert!(
        after_the_first_file.inode_record.is_none()
            && after_the_first_file.data_pointers.is_empty()
            && after_the_first_file
                .inode_record_of(InodeNumber(3))
                .is_some_and(|record| record.size == 0),
        "没有第一个文件那条记录、没有数据单元，只剩 size 0 的 inode 3"
    );
    assert_eq!(
        after_the_first_file.extent_tree.upper_nodes.len(),
        1,
        "有条目的号集合空：extent 树上段是一个零条目根兼叶"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "删完第一个文件");
}

/// 删完第一个文件（经会话 unlink，一路走空到有树而文件集合为空）、再建一个 inode（号 2，水位接着第一个文件之后）之后按 inode 号给它写数据：
/// D8（核心索引结构） 已定项 16 射程「删完之后不再建带数据的文件（K-U5，记欠账）」，用户 2026-10-19 定「β 时序级」——会话交回
/// `Publish`，原因是 `DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`（点名写的那个号；K-U5 的 β 时序级读法，
/// inode 2 的对象出生代晚于 inode 1 墓碑的死亡 txg），在任何写之前：系统配置槽、根环、
/// 录制流步数都不变，会话的现行那一版还是建 inode 那一版（没有第一个文件那条记录、inode 2 的 size 0）。
pub fn a_data_write_to_an_inode_created_after_the_first_file_was_deleted_is_refused_before_anything_reaches_the_disk(
) {
    let mut pool = build_pool("writes-by-inode-number-after-the-first-file-was-deleted");
    let (mut session, mut devices) = session_after_a_writable_remount(&mut pool);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            },
        )
        .unwrap_or_else(|refusal| panic!("unlink 第一个文件、一路走空：{refusal:?}"));
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            },
        )
        .unwrap_or_else(|refusal| panic!("删完之后建一个 inode：{refusal:?}"));
    let created = InodeNumber(2);
    let version_with_the_created_inode = session
        .current
        .file_version()
        .expect("删完之后建过 inode 的一版归带文件的一版（inode 树有条目）")
        .clone();
    assert!(
        version_with_the_created_inode.inode_record.is_none()
            && version_with_the_created_inode
                .inode_record_of(created)
                .is_some_and(|record| record.size == 0),
        "现行那一版没有第一个文件那条记录、有一条 size 0 的 inode 2"
    );
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refusal = session
        .publish_user_change(
            &mut devices,
            UserChange::OverwriteOfAnInode {
                inode: created,
                content: &content_of(700, 0x2401),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .expect_err("删完第一个文件之后给建的 inode 写数据，第一版拒");
    let UserChangeRefused::Publish { cause, .. } = refusal else {
        panic!("该是发布被拒（不是空间不够），实际 {refusal:?}");
    };
    assert!(
        matches!(
            cause,
            PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion {
                inode: 2,
                ..
            }
        ),
        "原因是删完之后不建带数据的文件（K-U5），点名 inode 2，实际 {cause:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前：系统配置槽、根环、录制流步数都不变"
    );
    assert_eq!(
        session.current.file_version(),
        Some(&version_with_the_created_inode),
        "会话的现行那一版还是建 inode 那一版"
    );
}

/// D8（核心索引结构） 已定项 16 第 4 条「写入口拒 nlink = 0 的 inode」对第一个文件之外的号（正推腿缺口 A）：inode 2 写过数据之后发它的 unlink 意图
/// （记录 nlink 写 0、队列 {2}），同一个进程里不先走空就直调按 inode 号写 inode 2——顺序写与覆盖写两个入口都在任何写之前拒成
/// `WriteToAnInodeBeingDeleted`（点名 inode 2），系统配置槽、根环、录制流步数都不变。
/// 判别力：那一判拿掉（`crates/mutations.tsv` 以「多文件写入口：别的号 nlink = 0」起头那一行）时写入口走到「承诺类意图队列已走空」那道断言，这一条红。
pub fn a_write_to_another_inode_being_deleted_is_refused_by_both_entries_before_anything_reaches_the_disk(
) {
    let mut pool = pool_with_inodes_created("writes-by-inode-number-to-an-inode-being-deleted", 1);
    sequential_write_in_process(&mut pool, 2, &content_of(2 * payload_capacity(), 0x2501));
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let intent = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(2),
            WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("inode 2 的 unlink 意图")
    };
    pool.output = intent.clone();
    assert!(
        intent
            .inode_record_of(InodeNumber(2))
            .is_some_and(|record| record.nlink == 0)
            && intent.deleted_inodes == vec![InodeNumber(2)],
        "意图之后 inode 2 那条记录 nlink 0、队列 {{2}}"
    );
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let content = content_of(700, 0x2502);
    let refusals = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        [
            publish_sequential_write_of_an_inode(
                &mut writer,
                &mut pool.allocator,
                &intent,
                InodeNumber(2),
                &content,
                WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
                INSTANCE_OF_THE_PROCESS,
            )
            .expect_err("顺序写删除中的 inode 2：拒"),
            publish_overwrite_of_an_inode(
                &mut writer,
                &mut pool.allocator,
                &intent,
                InodeNumber(2),
                &content,
                WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
                INSTANCE_OF_THE_PROCESS,
            )
            .expect_err("覆盖写删除中的 inode 2：拒"),
        ]
    };
    for refusal in &refusals {
        assert!(
            matches!(
                refusal,
                PublishError::WriteToAnInodeBeingDeleted { inode: 2 }
            ),
            "拒成写删除中的号、点名 inode 2：{refusal:?}"
        );
    }
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前：系统配置槽、根环、录制流步数都不变"
    );
}

/// 上段规范形「上一版有而新形状没有的换下」那一半（正推腿缺口 G；判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：
/// 第一个文件旁边建 149 个空 inode（号 2 … 150），经会话删完第一个文件（有条目的号集合空：上段是层级 0 第 0 格一个零条目根兼叶），再按 inode 号写
/// 删之前就有的 150（β 照写）：新形状是 150 那片叶 (0, 1) 加根 (1, 0)，旧的零条目根不罩 150、不在新形状里，这次换下——它的落点在这次的释放清单里；
/// 150 读回写的那一份，池级 checker 一条违例都没有。
/// 判别力：换下判据只留「罩着变了的号」那一半（`crates/mutations.tsv` 以「上段规范形换下判据：」起头、说不在新形状里那一行）时旧的零条目根不换下、
/// 不在释放清单里，这一条红。
pub fn writing_a_file_after_the_first_file_emptied_the_upper_segment_replaces_the_zero_entry_root(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_inodes_created(
        "writes-by-inode-number-after-the-upper-segment-was-emptied",
        INODE_PAST_THE_FIRST_UPPER_LEAF - 1,
    );
    let (mut session, mut devices) = session_after_a_writable_remount(&mut pool);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
            },
        )
        .unwrap_or_else(|refusal| panic!("unlink 第一个文件、一路走空：{refusal:?}"));
    let emptied = session
        .current
        .file_version()
        .expect("删完第一个文件而 149 个空 inode 还在：带文件的一版")
        .clone();
    let zero_entry_root = ExtentUpperNodePosition { level: 0, index: 0 };
    assert_eq!(
        (
            emptied.inodes_with_an_extent_upper_leaf_entry().len(),
            emptied
                .extent_tree
                .upper_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect::<Vec<_>>(),
        ),
        (0, vec![zero_entry_root]),
        "有条目的号集合空：上段只有层级 0 第 0 格的零条目根兼叶"
    );
    let slot_of_the_zero_entry_root = emptied.extent_tree.upper_nodes[0].1.locations[0].slot;
    let content = content_of(3 * payload_capacity() - 5, 0x1503);
    sequential_write_through_the_session(
        &mut session,
        &mut devices,
        INODE_PAST_THE_FIRST_UPPER_LEAF,
        &content,
    );
    let written = session
        .current
        .file_version()
        .expect("写 150 之后带文件的一版")
        .clone();
    assert_eq!(
        written
            .extent_tree
            .upper_nodes
            .iter()
            .map(|(position, _)| *position)
            .collect::<Vec<_>>(),
        vec![
            ExtentUpperNodePosition { level: 0, index: 1 },
            ExtentUpperNodePosition { level: 1, index: 0 },
        ],
        "新形状：150 那片叶与层级 1 的根"
    );
    assert!(
        written
            .released
            .iter()
            .any(|placement| placement.slot == slot_of_the_zero_entry_root),
        "旧的零条目根不在新形状里，这次换下、它的落点在释放清单里：{:?}",
        written.released
    );
    let image = pool.memory_pool();
    assert_eq!(
        contents_read_back(&image, &[INODE_PAST_THE_FIRST_UPPER_LEAF]),
        vec![content],
        "150 读回写的那一份"
    );
    hand_out_a_clean_image(images, &image, "写 150 之后");
}

/// 按结构种类记的写（里程碑「覆盖写、释放、回退与复用」增补 1 第 1 件）对第一个文件之外的号与第一个文件同一套（`write_accounting.rs` 新加的两臂）：
/// 同一个池上先给 inode 2 顺序写 145 个单元、再给第一个文件顺序写 145 个单元（两个都在上段第 0 片叶里，下段都是两片叶加一个根），
/// 两次发布的数据单元写与 extent 树节点写逐项相同，数据单元写是 145 个单元 × 两块盘的镜像。
/// 判别力：别的号的数据单元记成 extent 树节点、或它的下段节点记成数据单元（`crates/mutations.tsv` 以「多文件写账：」起头的两行）时两项对不上，这一条红。
pub fn the_writes_by_structure_kind_of_another_inode_are_counted_as_the_first_files_are() {
    let data_units = 145;
    let mut pool = pool_with_inodes_created("writes-by-inode-number-writes-by-structure-kind", 1);
    let content_of_inode_two = content_of(data_units * payload_capacity() - 5, 0x2601);
    let content_of_the_first_file = content_of(data_units * payload_capacity() - 5, 0x1601);
    let inode_two_written = sequential_write_in_process(&mut pool, 2, &content_of_inode_two);
    let first_file_written =
        sequential_write_in_process(&mut pool, FIRST_INODE_NUMBER, &content_of_the_first_file);
    for kind in [
        WrittenStructureKind::DataUnit,
        WrittenStructureKind::ExtentTreeNode,
    ] {
        assert_eq!(
            inode_two_written.writes.of(kind),
            first_file_written.writes.of(kind),
            "{kind:?}：写 inode 2 与写第一个文件同样多"
        );
    }
    assert_eq!(
        inode_two_written
            .writes
            .of(WrittenStructureKind::DataUnit)
            .write_calls,
        u64::try_from(data_units * 2).expect("小"),
        "数据单元写：145 个单元、两块盘各一份"
    );
}

/// 空间准入数一次发布要落的单元（`admission::units_of_a_publish_to_land`，C545）对第一个文件之外的号与第一个文件同一套（`admission.rs` 新加的臂：
/// 别的号的两种角色归普通分配、它的数据单元按成对的槽数、下段节点按提交内生块的槽数）：三个数据单元加一个下段节点，两种号数出来相同，
/// 都是三个数据单元、一个下段节点的一槽加 checkpoint 的块数。
/// 判别力：别的号的数据单元不当数据单元数（`crates/mutations.tsv` 以「多文件准入：」起头的三行：数据单元那一判、提交内生块那一判、
/// 两种角色归 checkpoint 保留池）时数出来的不同或断言「只交普通分配那一半」停下，这一条红。
pub fn the_space_admission_counts_the_units_of_another_inode_as_it_counts_the_first_files() {
    let checkpoint_cost = MetadataBlocks(7);
    let lower_leaf = ExtentLowerNodePosition { level: 0, index: 0 };
    let roles_of_the_first_file: Vec<TransactionUnit> = (0..3)
        .map(|unit| TransactionUnit::Data(DataUnitIndexInFile(unit)))
        .chain([TransactionUnit::ExtentLowerNode(lower_leaf)])
        .collect();
    let roles_of_inode_two: Vec<TransactionUnit> = (0..3)
        .map(|unit| TransactionUnit::DataOfAFileOtherThanTheFirst {
            inode: InodeNumber(2),
            unit: DataUnitIndexInFile(unit),
        })
        .chain([TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
            inode: InodeNumber(2),
            position: lower_leaf,
        }])
        .collect();
    let expected = UnitsOfAPublishToLand {
        data_units: 3,
        commit_generated_slots: TransactionUnit::ExtentLowerNode(lower_leaf).span_slots()
            + checkpoint_cost.0,
    };
    assert_eq!(
        units_of_a_publish_to_land(&roles_of_the_first_file, checkpoint_cost),
        expected,
        "第一个文件：三个数据单元，提交内生块是一个下段节点加 checkpoint 的块数"
    );
    assert_eq!(
        units_of_a_publish_to_land(&roles_of_inode_two, checkpoint_cost),
        expected,
        "inode 2 与第一个文件数出来相同"
    );
}

fn hand_out_a_clean_image(images: &mut PoolCheckerImages, image: &MemoryPool, what: &str) {
    let label = format!("{}-{what}", images.handed_out());
    images.hand_out(
        &label,
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
