//! harness 测试文件 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, crash_state_devices, disk_snapshot, file_content, format_pool, geometry,
    memory_pool_of_sparse_devices, parameters, unreadable_root_slot_of, with_unreadable_ranges,
    without_unreadable_ranges, FailingReadsOfARange, FormattedPool, SharedUnreadableRanges,
    UnreadableRange, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::Placement;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::TREE_TABLE_GENESIS_SLOT;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    BeforeTheOneReread, InstanceRow, MountError, NewerPublishWitness, ReadStageSettled,
    RollbackTarget, SelectedVersionAgainstTheWitness, ShadowLedger, StillUnreadableAfterOneReread,
    WitnessedCounterComparison,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::SharedStream;

pub fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 45 条；
/// 其余 39 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 85 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
/// 一条映射条目都没有。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
/// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由，比的是根环里全部读得出的根（不按实例表判有效）：根环里每条根的树表都是
/// mkfs 种的那一版或零单元发布照抄的，一条条目都没有——这个文件里每一步 I-9.28 与 I-9.14 同报不适用（下面五张表都同时有这两条）；
/// I-5.4（分配记录罩住的槽互不相交） 同一个理由：第 0 版树表里没有分配记录树，候选集里一条根的记录都走不到；
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 也是这个理由。
/// I-1.10（码 2 条目宽等于字段表宽） 也是这个理由：一条树表条目都没有，就没有哪个码 2 节点的条目宽可比。
/// I-8.7（实例内事务号不重号） 在这里报不适用是因为环里一条非 0 事务号的记录都没有：写行与暖机的空发布都写 0
/// （D23（journal 的角色与格式） 已定项 19 ①）。I-8.8（前缀里的事务不被切开） 是因为没有哪个非 0 事务号落了两条记录、
/// 也没有哪一组一条提交标记都没有。I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉） 与 I-9.7 同一个理由：一条 inode 记录都没有。
/// I-7.9（回退下界 F 不高于抬 F 的上限） 是因为没抬过 F：每条根带的 F 都是 0（这个文件里哪一步都不抬，下面四张表都有它）。
/// I-4.1（重放后树可遍历） 五张表都有：这个文件里每一步的镜像都是发布完才看的，最新根之上没有记录，重放一次都不施加
/// （阳性对照与坏镜像在 `checker_known_bad_images.rs` 那份「发布 B 的记录落了、根槽没落」的崩溃镜像上）。
/// I-3.5（引用区间的精确性） 在这里报不适用是因为回退候选集里相邻两版之间一个新出现的单元都没有：第一次可写挂载不写行，
/// 写行那次发布（txg 1）与暖机（txg 2）都是零单元发布，树表、实例表照抄 mkfs 那一版、映射根全零，后一版引用的单元前一版都引用过，
/// 出现规则一个单元都判不到（mkfs 之后那一格的理由不同，见 `NOT_APPLICABLE_RIGHT_AFTER_MKFS`）。
/// I-9.27（树表条目的头 ID 取值） 与 I-5.5（从 mkfs 水位建树那一版的 t1 落点） 三张没有文件的表都有：每条根的树表都是 0 条，
/// 头 ID 一格都没有，也没有哪一版是从 mkfs 水位第一次建树的那一版（第一个文件版本之后两条都真被评估过且成立）。
/// 删除线那五条（里程碑三·3）五张没有文件的表都有：I-9.5（删除队列里的号仍在树里） 与 I-3.4（可用空间扣待删占用） 要树表里的
/// `deleted_inodes`、inode 树与记账树，每一版树表都是 0 条（第一个文件版本之后两条都真被评估过且成立）；I-4.7（删除中 inode 有账） 与
/// I-4.6（中间态其余不变量成立） 没有处于删除中的号；I-4.4（意图只描述目标态） 第一版恒报不适用（下面每一张表都有它）。
/// 墓碑与谱系那九条（里程碑三·3，第 22 批）五张没有文件的表都有：I-9.17–I-9.23 要墓碑树、谱系树（每一版树表都是 0 条，有了文件版本也是零根，
/// 下面两张有文件的表照样有这七条），I-9.24（删除队列至多一条 key） 要 `deleted_inodes` 条目、I-9.29（同号只许一个出生代） 要 inode 记录
/// （第一个文件版本之后两条都真被评估过且成立）。I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 要最新根下面的记账树
/// （第一个文件版本之后真被评估过且成立：那一版自己写的那一行就是第一级台阶）。退役树那五条（I-9.31–I-9.35，第 50 批）同样没有对象：
/// 每一版树表都是 0 条，没有退役树、没有 inode 树。谱系树与退役树的容器容量那两条（I-9.36、I-9.37，第 53 批）同理没有对象
/// （有了文件版本两棵也是零根，下面两张有文件的表照样有它们；旧入口没有谱系与退役的旋钮值，I-9.37 一格都不判）。
pub const NOT_APPLICABLE_WITHOUT_FILE: [&str; 53] = [
    "I-9.27",
    "I-5.5",
    "I-1.10",
    "I-3.1",
    "I-3.9",
    "I-3.10",
    "I-3.11",
    "I-5.2",
    "I-5.4",
    "I-7.9",
    "I-8.7",
    "I-8.8",
    "I-9.1",
    "I-9.2",
    "I-9.4",
    "I-9.6",
    "I-9.7",
    "I-9.10",
    "I-9.12",
    "I-9.13",
    "I-9.14",
    "I-9.15",
    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    "I-8.11",
    "I-9.25",
    "I-3.13",
    "I-9.26",
    "I-3.12",
    "I-4.1",
    "I-3.5",
    "I-9.28",
    "I-9.5",
    "I-3.4",
    "I-4.7",
    "I-4.6",
    "I-4.4",
    "I-9.17",
    "I-9.18",
    "I-9.19",
    "I-9.20",
    "I-9.21",
    "I-9.22",
    "I-9.23",
    "I-9.24",
    "I-9.29",
    "I-9.30",
    "I-9.31",
    "I-9.32",
    "I-9.33",
    "I-9.34",
    "I-9.35",
    "I-9.36",
    "I-9.37",
];

/// mkfs 刚写完那一份比 `NOT_APPLICABLE_WITHOUT_FILE` 多报四条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法）、
/// I-8.9（一次发布的记录序号连续且只有末条带标志）、I-8.10（记录新根段与同代根相同） 与 I-8.12（同实例记录的 txg 不减） 都没有判的对象。
/// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这四条只在 mkfs 之后那一格。
/// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
/// 凑不出相邻的一对。
/// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
pub const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 57] = [
    "I-9.27",
    "I-5.5",
    "I-1.10",
    "I-3.1",
    "I-3.9",
    "I-3.10",
    "I-3.11",
    "I-5.2",
    "I-5.4",
    "I-7.9",
    "I-8.6",
    "I-8.7",
    "I-8.8",
    "I-8.9",
    "I-9.1",
    "I-9.2",
    "I-9.4",
    "I-9.6",
    "I-9.7",
    "I-9.10",
    "I-9.12",
    "I-9.13",
    "I-9.14",
    "I-9.15",
    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    "I-8.10",
    "I-8.11",
    "I-9.25",
    "I-3.13",
    "I-9.26",
    "I-3.12",
    "I-8.12",
    "I-4.1",
    "I-3.5",
    "I-9.28",
    "I-9.5",
    "I-3.4",
    "I-4.7",
    "I-4.6",
    "I-4.4",
    "I-9.17",
    "I-9.18",
    "I-9.19",
    "I-9.20",
    "I-9.21",
    "I-9.22",
    "I-9.23",
    "I-9.24",
    "I-9.29",
    "I-9.30",
    "I-9.31",
    "I-9.32",
    "I-9.33",
    "I-9.34",
    "I-9.35",
    "I-9.36",
    "I-9.37",
];

/// 写完第一个文件版本之后仍报不适用的那六条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
/// I-7.9（回退下界 F 不高于抬 F 的上限）——没抬过 F，没有抬 F 的根可判（坏镜像与阳性对照在 `checker_known_bad_images.rs` 步 5 那段历史上）；
/// I-9.14——树表条目只出现在这一条根的树表里（mkfs 种的第 0 版树表是空的、两次暖机空发布不写树表），
/// 它要同一棵树的条目出现在两个树表单元里才比得出来；I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由（根环里别的根
/// 都指着空树表）；
/// I-8.7——这个实例只发过一个承载事务的版本（事务号 1），环里其余记录都是事务号 0 的空发布，
/// 同一实例凑不出两条非 0 的号来比；
/// I-8.8（前缀里的事务不被切开）——第一版一事务一条、每条都带提交标记，③ ④ 没有对象。
/// 三条比得出来的镜像都在 `checker_known_bad_images.rs`（再覆盖写一次：树表两版、事务号 1 与 2；
/// B 之后再接一事务两条记录的阳性对照与坏镜像）。
/// I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立）——没有删除，没有处于删除中的号、没有 nlink = 0 的记录
/// （阳性对照与坏镜像在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）；I-4.4（意图只描述目标态） 第一版恒报不适用；
/// I-9.17–I-9.23——没有删除、没有回退，墓碑树与谱系树都是零根；I-9.31–I-9.34——没有叶容器身份消失，退役树零根
/// （I-9.35 在第一个文件版本那一版上真被评估过且成立：父版树表 0 条、叶身份集是空集）。
pub const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 22] = [
    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4", "I-9.17",
    "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.31", "I-9.32", "I-9.33",
    "I-9.34", "I-9.36", "I-9.37",
];

/// 只有实例 1 的池上第一个文件版本之后：比 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 多 I-3.12（实例表行 T 低于引用它的根的 txg）——
/// 实例表一行都没有（写过行的那一份上它真被评估过且成立）。
pub const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 23] = [
    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.31", "I-9.32",
    "I-9.33", "I-9.34", "I-9.36", "I-9.37",
];

/// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少七条——I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
/// I-1.10（码 2 条目宽等于字段表宽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-5.4（分配记录罩住的槽互不相交）、
/// I-8.11（点名项与被指单元头相同）、I-3.12（实例表行 T 低于引用它的根的 txg） 与 I-3.5（引用区间的精确性）。
/// 写行那次发布写了一棵分配记录树、根指针住根记录（C512（树表 0 条的一版上被换下的单元记在哪），
/// D16（发布语义） 已定项 9），里面未释放的那几条记录起点槽上都读得出单元头，
/// I-3.10 在这一格真被评估过；这棵树按位置寻址（D8（核心索引结构） 已定项 14），checker 从根记录那一项按位置走下去、每个节点判条目宽，
/// I-1.10 也真被评估过。checker 把根记录直接持有的那一棵并进候选根的分配记录（实审 B2）：被换下的 mkfs 那片实例表带释放标志，
/// I-3.9 判它的释放代；I-5.4 判这棵树里的记录两两不相交——两条都真被评估过且成立。
/// I-3.12：实例表里有了实例 1 那一行，判它的 T 低于引用它的根的 txg；I-8.11：写行那次发布的记录点名它写的实例表单元与分配记录树节点，
/// 判点名项与单元头逐字段相同——两条都真被评估过且成立。
/// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
/// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
/// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
pub const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 46] = [
    "I-9.27",
    "I-5.5",
    "I-3.1",
    "I-3.11",
    "I-5.2",
    "I-7.9",
    "I-8.7",
    "I-8.8",
    "I-9.1",
    "I-9.2",
    "I-9.4",
    "I-9.6",
    "I-9.7",
    "I-9.10",
    "I-9.12",
    "I-9.13",
    "I-9.14",
    "I-9.15",
    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    "I-9.25",
    "I-3.13",
    "I-9.26",
    "I-4.1",
    "I-9.28",
    "I-9.5",
    "I-3.4",
    "I-4.7",
    "I-4.6",
    "I-4.4",
    "I-9.17",
    "I-9.18",
    "I-9.19",
    "I-9.20",
    "I-9.21",
    "I-9.22",
    "I-9.23",
    "I-9.24",
    "I-9.29",
    "I-9.30",
    "I-9.31",
    "I-9.32",
    "I-9.33",
    "I-9.34",
    "I-9.35",
    "I-9.36",
    "I-9.37",
];

/// 交 checker 档：这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用（对拍字只分判到与没判到：核没判到），
/// 其余每一条都真被评估过且成立（一条违例都没有）。
pub fn hand_out_expecting_the_registered_verdicts(
    images: &mut PoolCheckerImages,
    pool: &FormattedPool,
    step: &str,
    not_applicable: &[&'static str],
) {
    let expected = singlefs_checker::image::IMPLEMENTED_INVARIANTS.iter().fold(
        PoolCheckerExpectation::no_invariant_violated(),
        |expected, invariant| {
            if not_applicable.contains(invariant) {
                expected.and_not_judged(invariant)
            } else {
                expected.and_holds(invariant)
            }
        },
    );
    images.hand_out(step, || pool.memory_pool(), expected);
}

/// 建池 → 可写挂载（不写文件）→ 退出 → 再可写挂载（写行）→ 退出 → **第三次可写挂载做得成**
/// （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案：根记录加一项放分配记录树的根指针）：
/// 写行那次发布把这一版的全部分配记录写成一个节点，被换下的那片实例表带着已释放标志进了它，
/// 重开时从它重建账 ⇒ **那一片没有被当空闲槽发出去**。
/// 三条断言各盯一处：那一片在重建出来的账里带着已释放标志、第三次挂载新写的那几个单元一个都没落在它的槽上、池级 checker 一条违例都没有。
/// 判别力自证：把写行那次发布里建最小分配记录树那一步去掉（`crates/mutations.tsv` 那两行），这三条各自变红。
pub fn the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool(
    images: &mut PoolCheckerImages,
) {
    let mut formatted = format_pool("step-three-formatted-third-mount-succeeds");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    let root_after_the_row = *row_written.current.root();
    let swapped_out_instance_table = formatted
        .genesis
        .root
        .instance_table
        .locations
        .map(|location| location.slot);
    formatted.devices = Some(second);
    assert_ne!(
        root_after_the_row.instance_table.head.birth_txg,
        CheckpointTxg(0),
        "写过行之后实例表的诞生 txg 不再是 0"
    );
    assert_ne!(
        root_after_the_row.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写行那次发布把分配记录树的根写进了根记录"
    );

    let mut third = formatted.reopen_recorded();
    let third_mount = mount_writable(&parameters(), &mut third).expect("第三次可写挂载做得成");
    let reused = third_mount
        .output
        .row_publish
        .root()
        .instance_table
        .locations
        .iter()
        .any(|location| swapped_out_instance_table.contains(&location.slot));
    formatted.devices = Some(third);
    assert!(
        !reused,
        "被换下的那片实例表（槽 {swapped_out_instance_table:?}）没有被当空闲槽发给第三次挂载写的那片"
    );
    let rebuilt_record_says_released = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| swapped_out_instance_table.contains(&record.slot))
        .all(|record| record.is_released);
    let rebuilt_record_count = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| swapped_out_instance_table.contains(&record.slot))
        .count();
    assert_eq!(rebuilt_record_count, 2, "两盘各一条记录罩着被换下的那一片");
    assert!(
        rebuilt_record_says_released,
        "重建出来的账里那两条记录带着已释放标志：写行那次发布的释放从盘上取回来了"
    );
    images.hand_out(
        "第三次可写挂载之后",
        || formatted.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
/// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
/// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有（条数按今天的
/// `IMPLEMENTED_INVARIANTS` 85 条数）——mkfs 之后 35 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 39 条成立，
/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 68 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
/// I-3.12、I-4.1、I-4.7、I-4.6、I-4.4 与墓碑树、谱系树那七条（I-9.17–I-9.23）仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
/// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录；没有删除；I-4.4 第一版恒报不适用）。
pub fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
    images: &mut PoolCheckerImages,
) {
    let mut formatted = format_pool("step-three-formatted-pool");
    hand_out_expecting_the_registered_verdicts(
        images,
        &formatted,
        "mkfs 之后",
        &NOT_APPLICABLE_RIGHT_AFTER_MKFS,
    );

    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    formatted.devices = Some(devices);
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(1),
        "取号 = max(系统配置 0, 根环 0) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(0), CheckpointTxg(0)),
        "所选根是 mkfs 的第 0 代根"
    );
    assert_eq!(output.effective_root, output.chosen_root);
    assert_eq!(output.journal.valid_records, 0, "环里一条记录都没有");
    assert!(output.rows_written.is_empty(), "上一个实例是 0：不写行");
    let PoolVersion::WithoutFile(row) = &output.row_publish else {
        panic!("树表 0 条：写行那次发布写零个单元")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (InstanceGeneration(1), CheckpointTxg(1), 1, 0, 0, 0),
        "txg = max(根环 0, 记录无) + 1；jsn 从 1 起；事务号 0；本实例第一条反向链 0；不点名"
    );
    assert_eq!(
        (
            row.root.tree_table,
            row.root.instance_table,
            row.root.mapping_root
        ),
        (
            formatted.genesis.root.tree_table,
            formatted.genesis.root.instance_table,
            formatted.genesis.root.mapping_root
        ),
        "根记录照 mkfs 的根：树表、实例表、映射树根都没换"
    );
    assert_eq!(region_device(1), DeviceIdentity(1), "txg 1 落盘 1");
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "txg 2 落盘 0，一次就覆盖两块盘"
    );
    let PoolVersion::WithoutFile(warm_up) = &output.warm_up_publishes[0] else {
        panic!("树表 0 条：暖机写零个单元")
    };
    assert_eq!(
        (
            warm_up.root.checkpoint_txg,
            warm_up.record.counter,
            warm_up.record.transaction,
            warm_up.record.back_chain
        ),
        (CheckpointTxg(2), 2, 0, back_chain_of(&row.record_bytes))
    );
    assert_eq!(region_device(2), DeviceIdentity(0), "txg 2 落盘 0");
    assert_eq!(
        mounted.current, output.warm_up_publishes[0],
        "接下来的发布接在 txg 2 后面"
    );
    assert_eq!(
        output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "没有被抛弃的根"
    );
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (3, 0),
            "只有 mkfs 的实例表 2 槽与树表 1 槽"
        );
    }
    hand_out_expecting_the_registered_verdicts(
        images,
        &formatted,
        "可写挂载之后",
        &NOT_APPLICABLE_WITHOUT_FILE,
    );
    assert_eq!(
        recover(&formatted.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );

    // 第一个文件版本：接在 txg 2 那条零单元发布后面。
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (
            first.root.instance,
            first.root.checkpoint_txg,
            first.record.counter,
            first.record.transaction,
            first.record.back_chain
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(3),
            3,
            1,
            back_chain_of(current.record_bytes())
        )
    );
    assert_eq!(
        first.data_pointers[0].locations[0].slot,
        SlotNumber(50180),
        "mkfs 的两个单元占着 50176–50178"
    );
    assert_eq!(
        first.released,
        vec![Placement {
            slot: SlotNumber(50178),
            span: 1
        }],
        "换下 mkfs 那片第 0 版树表"
    );

    let reference = build_pool("step-three-formatted-pool-reference");
    assert_eq!(
        formatted.retained_operations()[formatted.mkfs_operation_count..],
        reference.retained_operations()[reference.mkfs_operation_count..],
        "mkfs 之后的录制流与 mkfs 同一个进程里跑新池新建文件逐字节相同"
    );
    assert_eq!(first, reference.output, "写出的那一版也相同");

    hand_out_expecting_the_registered_verdicts(
        images,
        &formatted,
        "第一个文件版本之后",
        &NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE,
    );
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content
        }
    );
    assert_eq!(report.journal.valid_records, 3);
}

/// **2026-09-23 用户定案的那条零故障历史，端到端**：建池 → 可写挂载 → 不写文件 → 正常退出 → 再可写挂载 → 写文件 →
/// 冷启动读回来逐字节相同。今天被一条拒绝链挡着的每一道在这里都要放行：
/// R2（回退候选集里排除树表 0 条）、R1（树表 0 条的一版上写实例表行）、R4（空池挂载的形状判定）、
/// `publish_first_file` 写死的 txg / jsn / 实例表指针。
///
/// 每一格的数：
/// - 第一次可写挂载：取号 1、不写行、零单元发布 txg 1（落盘 1）、暖机 txg 2（落盘 0）；
/// - 第二次可写挂载：取号 2、写行 (1, 2, 0)，写行那次发布**只重写实例表一个单元**、txg 3 jsn 3（落盘 0）、暖机 txg 4（落盘 1）；
/// - 第一个文件版本：txg 5、jsn 5、事务号 1，照抄的是**写行之后那一版**的实例表指针（不是 mkfs 那一版——照抄错了这一行就丢）；
/// - 冷启动：择 (2, 5) 读回那个文件，内容逐字节相同；盘上那片实例表里仍是写行时那一条。
pub fn a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold(
    images: &mut PoolCheckerImages,
) {
    let mut formatted = format_pool("step-three-formatted-two-mounts-then-a-file");

    // 一、第一次可写挂载：不写文件，正常退出。
    let mut first_mount_devices = formatted.reopen_recorded();
    let first_mount =
        mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载");
    formatted.devices = Some(first_mount_devices);
    assert_eq!(first_mount.output.instance, InstanceGeneration(1));
    assert!(
        first_mount.output.rows_written.is_empty(),
        "上一个实例是 0：不写行"
    );
    assert_eq!(
        (
            first_mount.output.row_publish.root().checkpoint_txg,
            first_mount.current.root().checkpoint_txg
        ),
        (CheckpointTxg(1), CheckpointTxg(2))
    );
    assert_eq!(
        first_mount.output.row_publish.record().named.len(),
        0,
        "不写行那一次是零单元发布"
    );
    drop(first_mount);

    // 二、再可写挂载：写行。
    let mut second_mount_devices = formatted.reopen_recorded();
    let second_mount =
        mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载");
    formatted.devices = Some(second_mount_devices);
    assert_eq!(second_mount.output.instance, InstanceGeneration(2));
    assert_eq!(
        second_mount.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
        }],
        "上一个实例 1 那一行：选的根 txg 2、W = 0"
    );
    let PoolVersion::WithoutFile(row) = &second_mount.output.row_publish else {
        panic!("树表 0 条：写行那次发布仍是「没有文件版本」的一版")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (InstanceGeneration(2), CheckpointTxg(3), 3, 0, 0, 6),
        "写行那次发布：txg 3、jsn 3、事务号 0、反向链 0、点名实例表与分配记录树五个节点（按位置寻址，D8（核心索引结构） 已定项 14）"
    );
    assert_eq!(
        row.root.tree_table, formatted.genesis.root.tree_table,
        "树表照抄 mkfs 那片 0 条的"
    );
    assert_ne!(
        row.root.instance_table.locations[0].slot,
        formatted.genesis.root.instance_table.locations[0].slot,
        "实例表 COW 到新落点：mkfs 那一片被换下"
    );
    assert_eq!(region_device(3), DeviceIdentity(0), "txg 3 落盘 0");
    assert_eq!(
        second_mount
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(4)],
        "txg 4 落盘 1，一次就覆盖两块盘"
    );
    assert_eq!(region_device(4), DeviceIdentity(1));
    // mkfs 那片实例表进了 defer 队列（释放代 3）：还被根环里 txg ≤ 2 的候选根引用着，抬 F 之前不回收。
    for device_map in &second_mount.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (10, 2),
            "mkfs 的实例表 2 槽 + 树表 1 槽 + 新写的实例表 2 槽 + 这一版自己那棵分配记录树五个节点 5 槽（C512；按位置寻址，D8（核心索引结构） 已定项 14）；换下的 2 槽在 defer 队列里"
        );
    }
    hand_out_expecting_the_registered_verdicts(
        images,
        &formatted,
        "写行那次发布之后",
        &NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE,
    );

    // 三、写文件：接在写行之后那一版后面。
    let mut allocator = second_mount.allocator;
    let current = second_mount.current;
    assert_eq!(
        current.root().instance_table,
        row.root.instance_table,
        "暖机照抄写行那一版的实例表指针"
    );
    let content = file_content();
    let first_file = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(2),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (
            first_file.root.instance,
            first_file.root.checkpoint_txg,
            first_file.record.counter,
            first_file.record.transaction,
            first_file.record.back_chain
        ),
        (
            InstanceGeneration(2),
            CheckpointTxg(5),
            5,
            1,
            back_chain_of(current.record_bytes())
        ),
        "第一个文件版本 txg 5、jsn 5：从现行那一版接着算，不写死 3"
    );
    assert_eq!(
        first_file.root.instance_table, row.root.instance_table,
        "照抄的是写行之后那一版的实例表指针，不是 mkfs 那一版"
    );
    // 写行那一版自己那棵分配记录树的五个节点（按位置寻址，D8（核心索引结构） 已定项 14）：它那次发布点名的单元里实例表之外的那几个，
    // 先叶后根；第一个文件版本改的记录都在那两片叶里，五个都重写、都换下。
    let allocation_record_tree_nodes_of_the_row_version: Vec<Placement> = row
        .record
        .named
        .iter()
        .map(|named| named.locations[0].slot)
        .filter(|slot| *slot != row.root.instance_table.locations[0].slot)
        .map(|slot| Placement { slot, span: 1 })
        .collect();
    assert_eq!(allocation_record_tree_nodes_of_the_row_version.len(), 5);
    assert_eq!(
        first_file.released,
        std::iter::once(Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1
        })
        .chain(allocation_record_tree_nodes_of_the_row_version)
        .collect::<Vec<_>>(),
        "换下 mkfs 那片第 0 版树表，加写行那一版自己那棵分配记录树的五个节点（C512）"
    );

    // 四、冷启动读回来逐字节相同，行还在。
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(5)),
            content
        },
        "冷启动择 (2, 5) 读回那个文件，逐字节相同"
    );
    assert_eq!(report.journal.valid_records, 5);
    let system_configuration = choose_system_configuration(&reopened).expect("择系统配置");
    let newest = choose_root(&reopened, &system_configuration).expect("择根");
    let table = singlefs_core::recovery::instance_table_of_root(&reopened, &newest)
        .expect("盘上那片实例表解得开");
    assert_eq!(
        table.rows,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
        }],
        "写行那一条还在：第一个文件版本照抄的是写行之后那一版的指针"
    );
    hand_out_expecting_the_registered_verdicts(
        images,
        &formatted,
        "第一个文件版本之后",
        &NOT_APPLICABLE_WITH_ONE_FILE_VERSION,
    );
}

/// 录制流里 mkfs 之后那一段切在 txg 为 `checkpoint_txg` 的那次发布的根槽 FUA 写为止（含它）：之前的写都已被屏障放行、持久，
/// 之后的写（这次发布的系统配置轮换，与以后的发布）一个都没落盘——崩在根槽 FUA 之后、系统配置轮换之前（D16（发布语义） 已定项 7），
/// 系统配置没见证到这次发布。交回那个崩溃状态的两块内存盘（录进一条新流）。
pub fn devices_crashed_right_after_the_root_slot_of(
    formatted: &FormattedPool,
    checkpoint_txg: CheckpointTxg,
) -> Vec<(
    DeviceIdentity,
    singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
)> {
    let operations = formatted.retained_operations();
    let (writes, _segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    let root_slot_offset = slot_offset(
        target_for_publish(checkpoint_txg, geometry().root_ring_slots_per_region),
        geometry().fixed_structure_slot_spacing,
    );
    let root_slot_write = writes
        .iter()
        .position(|write| {
            write.is_force_unit_access
                && write.device == region_device(checkpoint_txg.0)
                && write.offset == root_slot_offset
        })
        .expect("录制流里有这次发布的根槽 FUA 写");
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index <= root_slot_write)
        .collect();
    crash_state_devices(
        &formatted.memory_pool_after_mkfs(),
        &writes,
        &persisted,
        &SharedStream::retaining_contents(),
    )
}

/// 一次发布写到的单元落点（两盘同槽，取第一条位置条目的槽）：带文件的一版按这次重写的角色，树表 0 条的一版按那条记录的点名项。
pub fn slots_written_by(version: &PoolVersion) -> Vec<SlotNumber> {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output
            .rewritten
            .iter()
            .map(|role| output.unit(*role).slot)
            .collect(),
        PoolVersion::WithoutFile(without_file) => without_file
            .record
            .named
            .iter()
            .map(|named| named.locations[0].slot)
            .collect(),
    }
}

/// M3 那一次挂载的读故障持续多久。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFaultOfTheThirdMount {
    PersistsThroughTheOneReread,
    LiftedBeforeTheOneReread,
}

/// C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在树表 0 条那一版上的样子，C554 乙（用户 2026-09-27 定，R = 1）之后：
/// mkfs → M1（实例 1，零单元 txg 1、2）→ M2（实例 2，写行 txg 3 点名实例表与分配记录树五个节点，暖机 txg 4）都做完，系统配置见证到 jsn 4。
/// M3 这一次挂载里 txg 3、4 的根槽与写行那条记录点名的实例表盘 0 那一份读不出——读阶段落到 (1, 2)（它的末条 jsn 2 < 4，判据 N-配置 为真）：
/// - 读故障持续到重读之后 ⇒ 拒可写（`NewerStateStillUnreadableAfterOneReread`，两遍都落到 (1, 2)），取号之前、`DiskSnapshot` 不变；
///   冷启动恢复（只读）照常落到 (1, 2)。C554 乙之前这一格照常挂上、把 (2, 3) (2, 4) 当成被抛弃，看不见它们，
///   把写行那次点名的单元逐槽再发一遍（池级 checker 的 I-7.4 照实红）。
/// - 读故障撤在重读之前（钩子里撤）⇒ 重读落到 (2, 4)，照常可写挂载：实例 2 那一行记到 txg 4、没有被抛弃的根，池级 checker 一条不红。
pub fn crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_refuses_writable_when_the_read_fault_persists_through_the_one_reread(
    images: &mut PoolCheckerImages,
) {
    crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file(
        images,
        ReadFaultOfTheThirdMount::PersistsThroughTheOneReread,
    );
}

pub fn crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread(
    images: &mut PoolCheckerImages,
) {
    crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file(
        images,
        ReadFaultOfTheThirdMount::LiftedBeforeTheOneReread,
    );
}

pub fn crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file(
    images: &mut PoolCheckerImages,
    fault: ReadFaultOfTheThirdMount,
) {
    let publish_parameters = parameters();
    let mut formatted = format_pool(match fault {
        ReadFaultOfTheThirdMount::PersistsThroughTheOneReread => {
            "step-three-formatted-witnessed-row-publish-stays-unreadable"
        }
        ReadFaultOfTheThirdMount::LiftedBeforeTheOneReread => {
            "step-three-formatted-witnessed-row-publish-lifted"
        }
    });
    let mut first = formatted.reopen_recorded();
    mount_writable(&publish_parameters, &mut first).expect("M1：实例 1，零单元");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let second_mount = mount_writable(&publish_parameters, &mut second).expect("M2：实例 2，写行");
    formatted.devices = Some(second);
    let PoolVersion::WithoutFile(row_publish) = &second_mount.output.row_publish else {
        panic!("M2 的写行接在树表 0 条的一版上，仍是树表 0 条")
    };
    assert_eq!(
        std::iter::once(row_publish.root.checkpoint_txg)
            .chain(
                second_mount
                    .output
                    .warm_up_publishes
                    .iter()
                    .map(|publish| publish.root().checkpoint_txg)
            )
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(3), CheckpointTxg(4)],
        "{fault:?}：实例 2：写行 txg 3、暖机 txg 4"
    );
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let instance_table_location = row_publish.record.named[0].locations[0];
    let unreadable = SharedUnreadableRanges::new(
        vec![
            unreadable_root_slot_of(CheckpointTxg(3), FailingReadsOfARange::Every),
            unreadable_root_slot_of(CheckpointTxg(4), FailingReadsOfARange::Every),
            UnreadableRange {
                device: instance_table_location.device,
                offset_in_bytes: instance_table_location.slot.to_device_offset().0,
                length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            },
        ],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut third = with_unreadable_ranges(formatted.reopen_recorded(), &unreadable);
    let first_read = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 4,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter: 2,
            },
        },
    };
    match fault {
        ReadFaultOfTheThirdMount::PersistsThroughTheOneReread => {
            let refused = mount_writable(&publish_parameters, &mut third);
            assert!(
                matches!(
                    &refused,
                    Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                        if **still_unreadable
                            == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                                first_read,
                                reread: first_read,
                            }
                ),
                "M3：两遍都落到 (1, 2)、系统配置见证到 jsn 4：{:?}",
                refused.as_ref().err()
            );
            assert_eq!(
                recover(&third, JournalPolicy::Consult).outcome,
                RecoveryOutcome::NoFile {
                    root: (InstanceGeneration(1), CheckpointTxg(2))
                },
                "同一组读故障下冷启动恢复（只读）照常，落到 (1, 2)"
            );
            formatted.devices = Some(without_unreadable_ranges(third));
            assert_eq!(
                disk_snapshot(&formatted.memory_pool(), &formatted.stream),
                before,
                "取号之前拒：一个写、一道屏障都没发"
            );
        }
        ReadFaultOfTheThirdMount::LiftedBeforeTheOneReread => {
            let lifting = unreadable.clone();
            let mut lift_before_the_reread = || lifting.lift();
            let third_mount =
                mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
                    &publish_parameters,
                    &mut third,
                    SpaceAdmission::JudgedByTheFormula,
                    ShadowLedger::On,
                    BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_before_the_reread),
                )
                .expect("M3：重读读得出 (2, 4)，照常可写挂载");
            formatted.devices = Some(without_unreadable_ranges(third));
            assert_eq!(
                third_mount.output.rereads.read_stage,
                ReadStageSettled::OnTheOneReread {
                    first_read,
                    reread: SelectedVersionAgainstTheWitness {
                        selected_version: RollbackTarget {
                            instance: InstanceGeneration(2),
                            checkpoint_txg: CheckpointTxg(4),
                        },
                        witness: NewerPublishWitness {
                            witnessed_journal_counter: 4,
                            comparison:
                                WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                                    selected_version_last_record_counter: 4,
                                },
                        },
                    },
                },
                "M3：第一遍落到 (1, 2)、判据为真；重读落到 (2, 4)"
            );
            assert_eq!(
                third_mount.output.rows_written,
                vec![InstanceRow {
                    instance: InstanceGeneration(2),
                    selected_root_txg: CheckpointTxg(4),
                    applied_transaction_high_water: 0,
                }],
                "M3：实例 2 那一行记到 txg 4，(2, 3) (2, 4) 都不被抛弃"
            );
            assert_eq!(
                third_mount.output.isolated_slots_per_device,
                vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
                "M3：没有被抛弃的根，影子账一槽不隔离"
            );
            // M3 之后池级 checker 一条不红（I-7.4 在内）
            images.hand_out(
                "M3 之后",
                || formatted.memory_pool(),
                PoolCheckerExpectation::no_invariant_violated(),
            );
        }
    }
}

/// 影子账里树表 0 条的被抛弃根那一支（`mount.rs` 的 `placements_referenced_by_root` 树表 0 条那一臂）在向前回退下的造法
/// （实三交回 Q3，调度记录 2026-09-24 第三节实三交回那一行「实四造得出就补」）：崩溃恢复抛弃一条写过行的树表 0 条根，
/// 之后第一个文件与正常卸载把 F 抬过它引用的槽。C554 乙之后崩溃恢复只抛弃得了系统配置没见证到的根，所以这一段历史崩在写行那次的
/// 根槽 FUA 之后、系统配置轮换之前（见证过的那一形见上一条用例，M3 被拒）。
///
/// 历史：mkfs → 可写挂载 M1（实例 1，零单元 txg 1、2）→ 退出 → M2（实例 2，写行 txg 3 点名实例表 50240 与分配记录树五个节点
/// 50242..50246）崩在 txg 3 的根槽 FUA 之后（录制流切到那一写为止：系统配置只见证到 (1, 2)，没有暖机 txg 4）→ M3 这一次挂载里
/// txg 3 的根槽与写行那条记录点名的实例表盘 0 那一份读不出 ⇒ 择根落到 (1, 2)、那条记录不跨实例、不施加，判据 N-配置 为假、不重读；
/// M3 取号 3、写行 txg 4（环里最大的 txg 是 (2, 3) 那条记录的 3；点名的六个单元与 M2 那次**逐槽相同**：两次写行接在同一版、同一份账上，
/// 落点同一条规则）、暖机 5 ⇒ M3 看不见 txg 3，隔离 0；撤掉读故障之后 (2, 3) 读得出，按实例 3 的表判被抛弃。
/// 同一个进程里实例 3 发第一个文件（txg 6：换下 mkfs 树表 50178 与那五个分配记录树节点，释放代 6）→ 正常卸载把 F 抬到 6
/// ⇒ 这一串回收释放代 ≤ 6 的槽，其中 50178（被抛弃根 txg 3 的树表）与 50246（它的分配记录树根，内容已被 M3 盖成实例 3 的，
/// 走不下去，只认根指针自己那一片）仍被环里的被抛弃根引用 ⇒ 影子账每盘隔离这 2 槽，卸载那一串与下一次挂载 M4 都不写它们；
/// M4 重算影子账，照样每盘 2 槽，没有一条被抛弃根算成「账读不出」。两处池级 checker 都只红 I-7.4：M3 看不见被抛弃根 (2, 3)，
/// 把它引用的实例表与分配记录树根逐槽再发了一遍（系统配置没见证到的最新根暂时读不出照旧被抛弃，C554 乙罩不到的那一格，照实红）。
///
/// 判别力：那一臂交回 `None`（树表 0 条的被抛弃根当成账读不出），卸载与 M4 都隔离 0、M4 报 1 条账读不出（`crates/mutations.tsv`）；
/// 那一臂漏掉分配记录树的节点，每盘只隔离 50178 那 1 槽。
pub fn crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them(
    images: &mut PoolCheckerImages,
) {
    let publish_parameters = parameters();
    let mut formatted = format_pool("step-three-formatted-abandoned-unwitnessed-row-publish");
    let mut first = formatted.reopen_recorded();
    mount_writable(&publish_parameters, &mut first).expect("M1：实例 1，零单元");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let second_mount = mount_writable(&publish_parameters, &mut second).expect("M2：实例 2，写行");
    formatted.devices = Some(second);
    let PoolVersion::WithoutFile(abandoned_row_publish) = &second_mount.output.row_publish else {
        panic!("M2 的写行接在树表 0 条的一版上，仍是树表 0 条")
    };
    let abandoned_root = abandoned_row_publish.root;
    assert_eq!(
        (abandoned_root.instance, abandoned_root.checkpoint_txg),
        (InstanceGeneration(2), CheckpointTxg(3)),
        "实例 2 写行 txg 3"
    );
    let abandoned_tree_table_slot = abandoned_root.tree_table.locations[0].slot;
    let abandoned_allocation_record_tree_root_slot =
        abandoned_root.allocation_record_tree_root.locations[0].slot;
    assert_eq!(
        (
            abandoned_tree_table_slot,
            abandoned_allocation_record_tree_root_slot
        ),
        (TREE_TABLE_GENESIS_SLOT, SlotNumber(50246)),
        "被抛弃的那条根：树表照 mkfs 那一片，分配记录树根是写行那次五个节点里最后取落点的那个（先叶后根）"
    );
    let second_row_publish_slots = slots_written_by(&second_mount.output.row_publish);

    let crashed = devices_crashed_right_after_the_root_slot_of(&formatted, CheckpointTxg(3));
    let crashed_image = memory_pool_of_sparse_devices(&crashed);
    let crashed_system_configuration =
        choose_system_configuration(&crashed_image).expect("崩溃状态上择得出系统配置");
    let ring_roots: Vec<(InstanceGeneration, CheckpointTxg)> =
        singlefs_core::recovery::readable_roots(
            &crashed_image,
            &crashed_system_configuration.immutable.region_devices,
            &crashed_system_configuration.immutable.sizes,
            &crashed_system_configuration.immutable.filesystem_identifier,
        )
        .iter()
        .map(|root| (root.instance, root.checkpoint_txg))
        .collect();
    assert!(
        ring_roots.contains(&(InstanceGeneration(2), CheckpointTxg(3)))
            && !ring_roots.contains(&(InstanceGeneration(2), CheckpointTxg(4))),
        "崩溃状态：(2, 3) 的根槽落了盘，暖机 txg 4 没发：{ring_roots:?}"
    );
    let instance_table_location = abandoned_row_publish.record.named[0].locations[0];
    let unreadable = SharedUnreadableRanges::new(
        vec![
            unreadable_root_slot_of(CheckpointTxg(3), FailingReadsOfARange::Every),
            UnreadableRange {
                device: instance_table_location.device,
                offset_in_bytes: instance_table_location.slot.to_device_offset().0,
                length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            },
        ],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut third_with_the_fault = with_unreadable_ranges(crashed, &unreadable);
    // M3 关着影子账（只供测试的开关，连 C694 的 C 一起关，删除线第 33 批 T）：(2, 3) 的记录读得出、在水位之上没施加，产品路径的 C
    // 会把它点名的落点隔离、M3 不再是看不见它的那次挂载；这条用例要的是看不见它、把它的槽发出去之后，之后的挂载与抬 F 怎么护。
    let third_mount = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &publish_parameters,
        &mut third_with_the_fault,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::Off,
        BeforeTheOneReread::RereadImmediately,
    )
    .expect("M3：写行那条根读不出、系统配置只见证到 (1, 2)，择根落到 (1, 2)，照常可写挂载");
    let mut third = without_unreadable_ranges(third_with_the_fault);
    assert_eq!(
        third_mount.output.rereads.read_stage,
        ReadStageSettled::OnTheFirstRead {
            first_read: SelectedVersionAgainstTheWitness {
                selected_version: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(2),
                },
                witness: NewerPublishWitness {
                    witnessed_journal_counter: 2,
                    comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                        selected_version_last_record_counter: 2,
                    },
                },
            },
        },
        "M3：系统配置见证到的只有 (1, 2)（jsn 2），判据为假，不重读"
    );
    assert_eq!(
        (
            third_mount.output.instance,
            third_mount.output.effective_root.instance,
            third_mount.output.effective_root.checkpoint_txg,
            third_mount.output.row_publish.root().checkpoint_txg
        ),
        (
            InstanceGeneration(3),
            InstanceGeneration(1),
            CheckpointTxg(2),
            CheckpointTxg(4)
        ),
        "M3：实例 3 接在 (1, 2) 后面，写行 txg 4"
    );
    assert_eq!(
        slots_written_by(&third_mount.output.row_publish),
        second_row_publish_slots,
        "M3 的写行与 M2 的写行逐槽相同：M3 这时看不见被抛弃的根，隔离不到它，只能靠之后的挂载与抬 F"
    );
    assert!(
        third_mount
            .allocator
            .devices
            .iter()
            .all(|device_map| device_map.isolated_slots() == 0),
        "M3 看不见 txg 3：一槽都没隔离"
    );

    let mut allocator = third_mount.allocator;
    let content = file_content();
    let first_file = {
        let mut writer = PoolWriter::new(&publish_parameters, third.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            third_mount.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            third_mount.output.instance,
            third_mount.current.record_bytes(),
        )
        .expect("实例 3 的第一个文件版本")
    };
    assert_eq!(
        first_file.root.checkpoint_txg,
        CheckpointTxg(6),
        "M3 写行 4、暖机 5，第一个文件版本 6"
    );
    for slot in [
        abandoned_tree_table_slot,
        abandoned_allocation_record_tree_root_slot,
    ] {
        assert!(
            first_file
                .allocation_records
                .iter()
                .filter(|record| record.slot == slot)
                .all(|record| record.is_released && record.generation == CheckpointTxg(6)),
            "第一个文件版本换下槽 {slot:?}，两盘各一条已释放、释放代 6"
        );
    }
    let mut current = PoolVersion::WithFile(first_file);
    let unmounted = singlefs_core::mount::unmount(
        &publish_parameters,
        &mut third,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载");
    let singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        panic!("现行那一版带文件：卸载把 F 抬到它")
    };
    assert_eq!(raised.rollback_floor, CheckpointTxg(6));
    assert_eq!(
        raised.abandoned_roots_unreadable, 0,
        "被抛弃的那条根认得出它引用的落点"
    );
    for slot in [
        abandoned_tree_table_slot,
        abandoned_allocation_record_tree_root_slot,
    ] {
        assert!(
            raised
                .reclaimed
                .iter()
                .any(|placement| placement.slot == slot),
            "F 抬到 6：释放代 6 的槽 {slot:?} 回收了"
        );
        assert!(
            raised.publishes.iter().all(|publish| publish
                .rewritten
                .iter()
                .all(|role| publish.unit(*role).slot != slot)),
            "卸载那一串没把槽 {slot:?} 发出去"
        );
        for device_map in &allocator.devices {
            assert!(
                !device_map.is_free(slot),
                "盘 {}：槽 {slot:?} 仍被环里的被抛弃根引用，回收之后隔离着",
                device_map.device.0
            );
        }
    }
    assert!(
        allocator
            .devices
            .iter()
            .all(|device_map| device_map.isolated_slots() == 2),
        "每盘隔离的正好是被抛弃根的树表与分配记录树根这 2 槽：{:?}",
        allocator
            .devices
            .iter()
            .map(|device_map| device_map.isolated_slots())
            .collect::<Vec<_>>()
    );
    hand_out_expecting_only_the_abandoned_root_the_blind_mount_reused_to_redden_the_reuse_invariant(
        images,
        || memory_pool_of_sparse_devices(&third),
        "卸载之后",
    );

    let fourth_mount = mount_writable(&publish_parameters, &mut third).expect("M4");
    assert_eq!(
        (
            fourth_mount.output.isolated_slots_per_device.clone(),
            fourth_mount.output.abandoned_roots_unreadable
        ),
        (vec![(DeviceIdentity(0), 2), (DeviceIdentity(1), 2)], 0),
        "M4 重算影子账：每盘照样隔离那 2 槽，没有一条被抛弃根算成账读不出"
    );
    let slots_written_by_the_fourth_mount: Vec<SlotNumber> =
        std::iter::once(&fourth_mount.output.row_publish)
            .chain(&fourth_mount.output.warm_up_publishes)
            .flat_map(slots_written_by)
            .collect();
    assert!(
        !slots_written_by_the_fourth_mount.contains(&abandoned_tree_table_slot)
            && !slots_written_by_the_fourth_mount
                .contains(&abandoned_allocation_record_tree_root_slot),
        "M4 的写行与暖机没写被抛弃根引用的那两槽：{slots_written_by_the_fourth_mount:?}"
    );
    hand_out_expecting_only_the_abandoned_root_the_blind_mount_reused_to_redden_the_reuse_invariant(
        images,
        || memory_pool_of_sparse_devices(&third),
        "M4 之后",
    );
}

/// 这段历史里 M3 看不见被抛弃的那条根（根槽读不出、系统配置没见证到它），把写行那次点名的单元逐槽再发了一遍：被抛弃根 (2, 3) 引用的实例表与
/// 分配记录树根在它离开根环之前被重新分配，池级 checker 的 I-7.4（近 K 代块未被复用） 被抛弃根那一半照实红——C554（崩溃恢复抛弃的根暂时读不出时
/// 影子账算不到） 那一形在系统配置没见证到的根上还在（实审 B2 补上被抛弃根那一半，主 agent 2026-09-27 定照实红）。别的不变量一条都不红
/// （判不了的照报不适用：这段历史里实例 3 只写过一条非 0 事务号的记录，I-8.7 / I-8.8 这类凑不出判的对象）。
pub fn hand_out_expecting_only_the_abandoned_root_the_blind_mount_reused_to_redden_the_reuse_invariant(
    images: &mut PoolCheckerImages,
    build_the_image: impl FnOnce() -> MemoryPool,
    step: &str,
) {
    // 只该红 I-7.4，红在 M3 看不见的被抛弃根 (2, 3) 上
    images.hand_out(
        step,
        build_the_image,
        PoolCheckerExpectation::exactly_these_violated(&["I-7.4"])
            .and_violated_with_a_detail_containing("I-7.4", "被抛弃的根（实例 2、txg 3）"),
    );
}
