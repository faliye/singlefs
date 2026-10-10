//! 池级 checker 的输入与第一层判定：读盘的口子、每条不变量的四种判定、系统配置与根环（I-7 类）、指针与位置条目。
//! 与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5）：偏移在这里按字段表各写一份。

use std::collections::BTreeMap;

use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, JOURNAL_RECORD_BYTES,
    JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, ROOT_RECORD_BYTES, ROOT_RING_BASE_SLOT,
    ROOT_RING_SLOTS_PER_REGION_MAXIMUM, ROOT_RING_SLOTS_PER_REGION_MINIMUM, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::{
    check_root_slot, check_system_configuration_slot, read_six_byte_unsigned, read_u16, read_u32,
    read_u64, RootView, SystemConfigurationView,
};

/// checker 读镜像的口子。它不依赖实现的块设备抽象：harness 的崩溃镜像、宿主上的盘镜像各自接一份。
pub trait ImageReader {
    fn devices(&self) -> Vec<u32>;
    fn device_bytes(&self, device: u32) -> Option<u64>;
    /// 读不到（没有这块盘、越界）返回 None。
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>>;
    /// 扫描方向的候选：可能有单元头的 16 KiB 槽号。None = 从单元区起点扫到盘尾。
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>>;
    /// 扫描方向的候选：可能有 journal 记录的环内槽号（从环起点数，每槽一条 4096 字节的记录）。None = 整条环都扫。
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>>;
}

/// 一条不变量在一个镜像上的判定。「不适用」必须带理由，不许静默算成立。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvariantVerdict {
    Holds,
    Violated(String),
    NotApplicable(&'static str),
    /// 部分没判（用户 2026-10-04 定，第四种判定）：判到的那一部分都对得上，射程里另有一部分这张镜像上判不了，说明逐条列出没判到的
    /// 是什么（例如 I-9.28 那几条根：盘号、txg、为什么没判）。**它不是红，也不是成立**：计红、计违例的地方算不红（层 0 与崩溃放量
    /// 把它算成不红），计「成立」的地方不算成立。今天 I-9.28 报它（树表读不出、条目比字段表短的根），I-5.5 报它（某一版树表读不出、
    /// 条目窄、extent 树或分配记录树读不全、种类 1 / 3 条数不对，`research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3）。
    PartiallyJudged(String),
}

/// 「中央映射条目的 key 与它指的单元头相符」那一条（用户 2026-09-27 定新立，实审 B2），
/// 编号 I-1.11（映射 key 与单元头相符），登记在 `.claude/kb/invariants.md`。
pub const MAPPING_KEY_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";

/// 「自证过的系统配置槽里，池级字段都在这一版读者收的范围里」那一条（实审 A3-checker-2，用户 2026-09-27 定系统配置越界整池拒）：
/// 格式版本 = 1、加密类型 = 0（关）、固定结构槽距、`physical_block_size`、journal 环长在各自的上下界里（[`crate::Verdict`] 的
/// `FormatVersionNotRecognized` / `EncryptionTypeNotOff` / `FixedStructureSlotSpacingOutsideTheFormatRange` /
/// `PhysicalBlockSizeOutsideTheRootSlotBounds` / `JournalRingBytesOutsideTheSupportedRange`）；journal 环起点（偏移 325）与根环起点
/// （偏移 371）是第一版的格式常量、单元区起始槽号（偏移 417）是环长现算的那个槽且落在聚簇段边界上
/// （`JournalRingStartSlotNotTheFirstVersionConstant` / `RootRingBaseSlotNotTheFirstVersionConstant` /
/// `UnitAreaStartNotTheSlotAfterTheJournalRing` / `UnitAreaStartOffTheClusterSegmentBoundary`；实审 Y4-a，用户 2026-09-27 定
/// 「读字段，不等于常量就整池拒」，与实现收同一张表）。**编号是暂立的，待 kb 第六批登记时定**；改号只改这一处。
pub const SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS: &str = "I-7.13";

/// 「每块盘每个自证过的系统配置槽里的本盘设备号，等于这块盘在池里的身份」那一条（实审 Y2-b 越格线索：盘体对调之后的池
/// 可写挂载因本盘设备号拒，而 checker 读本盘设备号却没有一条不变量拿它比盘的身份，0 违例）。实现那一侧是
/// `crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`（`OwnDeviceNumberDiffersFromTheIdentityHandedIn`），
/// checker 自己按字段表读一份（[`judge_own_device_number_is_the_device_identity`]）。编号 I-7.14，登记在 `.claude/kb/invariants.md`；
/// 改号只改这一处。
pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str = "I-7.14";

/// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
///
/// I-2.4（头校验和覆盖范围） 的射程从「单元头 29 字节预留位」扩到「单元头与指针头部的加密预留位，加密关着时恒 0」
/// （主 agent 2026-09-27 定，实审 A3-checker-2；`invariants.md` 那一行由 kb 第六批改）：池级走读在单元头上判格式版本认得、
/// 29 字节预留位全 0（`walk` 的 `judge_unit_header`），在每条跟随的指针上判头部 MAC 16 + nonce 12 全 0
/// （[`judge_pointer_mac_and_nonce_are_zero`]，D19（块指针的结构与宽度预算） 已定项 3 射程）。
///
/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：实审 A3b Q2 交上来，用户 2026-09-27 定「立不变量并同步」；
/// `invariants.md` 那一行由 kb 第八批写。池级走读每读一版的树表判一次（`walk` 的 `judge_tree_table_entries_ordering`）。
///
/// 第 50 条起是里程碑三二·1 那一批（两份判决 `research/prompts/m3-inv-a-r1-main-verification.md`、
/// `research/prompts/m3-inv-b-r1-main-verification.md`，用户 2026-10-01 定案），**只往末尾追加**：层 0 的违例位按下标 i ↔ 这张表第 i 条，
/// 前 49 条的下标不动。位宽改到 ≥ 128 之前，下标 64 起的那一条在层 0 的 GPU 判器上没有位（另一个实现员在 checker 档改）。
/// 第 66 条（下标 65）是 I-3.5（里程碑三第 8 批）：GPU 判器内核还没判它（`crash_judge_gpu` 的覆盖表另列）。
///
/// 第 67、68 条（下标 66、67）是 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤ ⑥ 那两条：I-9.27（树表条目的头 ID 取值）与
/// I-5.5（从 mkfs 水位建树那一版的 t1 落点），同样只往末尾追加；GPU 判器内核两条都判。
///
/// 第 69 条（下标 68）是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立，里程碑三第 9 批），同样只往末尾追加；
/// GPU 判器内核还没判它（`crash_judge_gpu` 那一侧要补，归 GPU 那一簇）。
///
/// 第 70–74 条（下标 69–73）是里程碑三·3 删除线那五条（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节，
/// 用户 2026-10-05 定 I-4.4、I-4.6 取乙）：I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，同样只往末尾追加；GPU 判器的扫描内核判前四条
/// （下标 69–72，删除态扫描，第 18 批 P2），下标 73 的 I-4.4 第一版恒报不适用、内核不置位（`crash_judge_gpu` 的覆盖表）。
///
/// 第 75–83 条（下标 74–82）是里程碑三·3 墓碑与谱系那九条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节与
/// 第四节用户定案，次序照第二节 T13 那一格）：I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，同样只往末尾追加。
/// I-9.17、I-9.23 在主走读里判（墓碑树、谱系树的内部条目），其余在删除态扫描里每一版各判（I-9.29 跨版本比）；
/// I-9.21、I-9.22 的旋钮值从镜像外给（`walk::check_pool_image_given_the_tombstone_container_capacity_knob`）。
/// GPU 判器的内核这一批不置这九位（`crash_judge_gpu` 的覆盖表照登记，内核第 23 批补）。
///
/// 第 84 条（下标 83）是 I-3.14（实例表每片记录形态合法，用户 2026-10-06 定另立，根环线第 10j 批），同样只往末尾追加；
/// 主走读沿实例表链每一片判（`walk` 的 `judge_instance_table_page_record_forms` 与链指针记录那一格）。
/// 不成立的那一片解不开：行不收、链不往下走（第 10k 批）。GPU 判器的走读内核同判（`crash_judge_gpu` 的覆盖表补了下标 83，第 10k 批）。
///
/// 第 85 条（下标 84）是 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态，第 49 批），同样只往末尾追加；
/// 在最新的根上判（`walk` 的 `judge_inode_number_watermark_steps_of_the_newest_root`，挨着 I-9.6）。GPU 判器的扫描内核同判（`crash_judge_gpu` 的覆盖表补了下标 84）。
///
/// 第 86–90 条（下标 85–89）是退役树那五条（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条，正推
/// `research/prompts/m3-retire-r1-sonnet-output.md` K5.1，第 50 批）：I-9.31、I-9.32、I-9.33、I-9.34、I-9.35，同样只往末尾追加。
/// I-9.31 在主走读里判（退役树的内部条目，与 I-9.17、I-9.23 同一个走法），其余在墓碑与谱系那一遍里每一版各判（I-9.35 跨相邻两版比）。
/// GPU 判器同判 I-9.31，其余四条在有退役记录的状态上判不了（`crash_judge_gpu` 的覆盖表与它的文档）。
///
/// 第 91、92 条（下标 90、91）是谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，
/// 第 53 批，与 I-9.21、I-9.22 同形）：I-9.36、I-9.37，同样只往末尾追加。在墓碑与谱系那一遍里每一版各判；I-9.37 的旋钮值从镜像外给
/// （`walk::check_pool_image_given_the_container_capacity_knobs`）。GPU 判器的扫描内核同判 I-9.36；第 56 批 G6 起旋钮值经扫描内核源码前面的常量
/// 传进 GPU（`crash_judge_gpu` 的 `JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），装了旋钮的那一棵 GPU 同判 I-9.37
/// （墓碑旋钮同判 I-9.22）；层 0 与放量走产品那一份（三棵都没装），I-9.22、I-9.37 在那里照旧不置位。
pub const IMPLEMENTED_INVARIANTS: [&str; 92] = [
    "I-1.1",
    "I-1.2",
    "I-1.3",
    "I-1.4",
    "I-1.6",
    "I-1.7",
    "I-1.8",
    "I-1.10",
    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    "I-2.1",
    "I-2.3",
    "I-2.4",
    "I-2.5",
    "I-3.1",
    "I-3.8",
    "I-3.9",
    "I-3.10",
    "I-3.11",
    "I-4.2",
    "I-4.8",
    "I-5.1",
    "I-5.2",
    "I-5.4",
    "I-7.1",
    "I-7.2",
    "I-7.3",
    "I-7.4",
    "I-7.6",
    "I-7.7",
    "I-7.8",
    "I-7.9",
    "I-7.12",
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
    SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
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
    "I-9.16",
    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
    UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
    ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
    INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
    ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
    DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
    JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    REFERENCE_INTERVALS_ARE_EXACT,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
];

/// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
/// 不判（inode 内部节点指着叶容器的那一类由 I-9.2 罩住，不重判）。
pub const POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER: &str = "I-1.12";
/// I-8.10：带末条标志、同 (实例代号, checkpoint_txg) 只此一条的记录，新根段七段与根环里同 (实例代号, checkpoint_txg) 的根逐字节相同
/// （树表指针、映射根指针、树 ID 水位、F 四段，加实例表指针、分配记录树根指针、卸载记号三段：用户 2026-10-03 定扩到七段）。
pub const LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH: &str = "I-8.10";
/// I-8.11：点名项的一条位置条目所指那一份整单元校验和相符时，单元头的类标签、诞生代号、key 尾段（与码 1 / 码 3 的出生树）与点名项逐字段相同。
pub const NAMED_ENTRY_MATCHES_THE_UNIT_HEADER: &str = "I-8.11";
/// I-4.9：回退候选集里不是最新的每一条根，它引用的单元诞生代号不大于这条根的 checkpoint_txg（最新根那一条归 I-4.2）。
pub const UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT: &str = "I-4.9";
/// I-9.25：树表条目的 `previous_snapshot_txg`：不是 inode 树的条目恒 0；inode 树那条不大于引用这张树表的根的 checkpoint_txg。
pub const PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES: &str = "I-9.25";
/// I-3.13：一版的记账树里每一行的代段等于这一版的 checkpoint_txg。
pub const ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION: &str = "I-3.13";
/// I-9.26：inode 记录的改动计数不小于对象出生代（创建算一次改动）、不大于所在叶容器头的诞生代号。
pub const INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH: &str = "I-9.26";
/// I-3.12：根环里任一自证过的根，它指着的实例表每一行 (i, T, W) 满足 T < 这条根的 checkpoint_txg。
pub const INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM: &str = "I-3.12";
/// I-7.16：每块盘上可择的系统配置槽 i，世代号 ≥ 2 时世代号 mod 2 == i；两槽都可择时世代号相差恰为 1，或都是 mkfs 写的 1。
pub const SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION: &str = "I-7.16";
/// I-7.17：根环里任一自证过的根住 (区域 txg mod R, 槽 (txg div R) mod S)；第 0 代的根只判槽 0。
pub const ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES: &str = "I-7.17";
/// I-8.12：同一块盘 journal 环里同一实例的记录按计数器从小到大，checkpoint_txg 不减。
pub const JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE: &str = "I-8.12";
/// I-7.15：各盘所选的系统配置槽里的设备数相同（形态抄 I-1.4 的 fsid 那一判）。
pub const DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE: &str = "I-7.15";
/// I-4.1：所选根（重放之后那一版）指向的树全可遍历；checker 自己按 D23（journal 的角色与格式） 已定项 14 的规则只读重放（`walk` 的
/// `replay_the_next_mount_would_do`），一条记录都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验证不过（起点含介质损坏）时报不适用。
pub const TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE: &str = "I-4.1";
/// I-8.1：自证过的系统配置槽里在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096、F = 格式常量 `JOURNAL_SAFETY_FACTOR`
/// （三格任一不等，整池不作保，同 I-7.13）；同一块盘环里同 (实例代号, checkpoint_txg) 的记录不多于在飞上限条。
pub const JOURNAL_RING_GEOMETRY_FIELDS_AGREE: &str = "I-8.1";
/// I-7.5：每个根槽的起始偏移是判定宽度的整数倍；判定宽度 = 各盘所选系统配置声明的 max(physical_block_size, io_min) 里最大的那个
/// （声明值，未探测：[`crate::DecisionWidth::DeclaredOnly`]）。R × S 个槽逐个判。
pub const ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH: &str = "I-7.5";
/// I-8.5（V-池）：全池至少一块盘有一个自证过的系统配置槽，全池皆废判红。单盘全废、哪块盘少于两槽自证过（可写挂载会拒）
/// 不进违例通道，走旁路报告 [`system_configuration_redundancy_report`]。
pub const SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION: &str = "I-8.5";
/// I-3.5（引用区间的精确性）的出现规则：回退候选集按 txg 升序、同 txg 并成一条，相邻两版 (R⁻, R)，R 引用而 R⁻ 不引用的每个单元
/// 诞生代号大于 R⁻.txg，或落在 R 的一棵整棵继承下来的用户可见树里（按树继承下界）；候选集有洞时按实例表行判接不接得上，
/// 接不上的那一对只判用户可见树之外的单元（`research/prompts/m3-birth-floor-r3-main-verification.md`，用户 2026-10-03 定丙）。
/// ③ ④（deadlist）这一版不判。
pub const REFERENCE_INTERVALS_ARE_EXACT: &str = "I-3.5";
/// I-9.27：走读到的任一版树表里，条目偏移 116 的头 ID 按种类取值——inode 树（种类 2）等于它自己的树 ID；extent 树（1）、`deleted_inodes`（9）、
/// 墓碑树（10）等于这张树表里某一条种类 2 条目的树 ID（没有种类 2 条目时判红）；种类 3、4、6、7、8、11、12 等于 0；别的种类码不判。
/// 不拿字面 65537 比（D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」句末的关系判据，判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1）。
pub const HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES: &str = "I-9.27";
/// I-5.5：从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条诞生 txg 都等于这一版的 `checkpoint_txg`）引用的数据单元里，
/// 每块盘上落点槽号最小的那一个等于这一版分配记录照 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、
/// 两槽都空、不在开放段里；新池新建文件那一次是单元区起始槽号 U + 4，回收之后不再是）（D3（空间分配） 已定项 8 射程，判决
/// `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2；按分配记录现算：判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2）。
pub const FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK: &str = "I-5.5";
/// I-9.28：根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID
/// （条目偏移 0）的条目，诞生 txg 相同。与 I-9.14 的分工：I-9.14 只比同一条时间线上的根，这一条不分时间线（树 ID 被重发时
/// 新线与老线上同一个 ID 的树诞生 txg 不同，C342（树 ID 水位在根读不出时退回去重发））。
pub const TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT: &str = "I-9.28";
/// I-9.5（删除队列里的号仍在树里），两个方向：① `deleted_inodes` 里的每个号在同一版的 inode 树里有记录且 nlink = 0；
/// ② inode 树里 nlink = 0 的每条记录，号都在同一版的 `deleted_inodes` 里（队列 = inode 树里 nlink = 0 的号集合，
/// D8（核心索引结构） 已定项 16 第 3 条；C708）。判法（判哪几版、经头 ID 配对、读不全报射程里判不了）是下面四条的共用判法。
pub const DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES: &str = "I-9.5";
/// I-3.4（可用空间扣待删占用） 只落「记账标签 4 恒 0」那一句：判的每一版里走全了的记账树都有 (4, 0, 0xFFFF_FFFF) 那一行、值为 0，缺行判红
/// （判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F5 三格）。
pub const PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO: &str = "I-3.4";
/// I-4.7（删除中 inode 有账），对象取丙：inode 树里 nlink = 0 的记录的号，与 extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 记录的号；
/// 这些号要么在 `deleted_inodes` 里，要么一个数据单元都不剩（同一判决第二节 F1）。
pub const INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT: &str = "I-4.7";
/// I-4.6（中间态其余不变量成立） 取乙：每条 nlink = 0 的记录，extent 树里它的数据单元数与 size 对得上——size 为 0 时 0 或 1 个，
/// size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`）。
pub const DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE: &str = "I-4.6";
/// I-4.4（意图只描述目标态）：第一版报不适用（用户 2026-10-05 定，岔路单同上），理由见 [`INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`]。
pub const INTENT_DESCRIBES_ONLY_THE_TARGET_STATE: &str = "I-4.4";
/// I-4.4 在第一版报不适用的理由，照 `.claude/kb/invariants.md` I-4.4 那一行的状态列。
pub const INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION: &str = "第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段；logged_ops 有字段表时实现";
/// I-9.17（谱系树条目身份与子头相符）：谱系树每条内部条目的类型段 ∈ {0, 5}；0 ⇒ 子单元是码 2、身份引用三段为 0；5 ⇒ 子容器头四段与身份引用逐字相等、
/// 子指针头部出生树相等、子容器打包记录类型 5。身份不符单独判红，不记走读失败（判决第二节 T7 取 a）。
pub const LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.17";
/// I-9.18（删除队列里的号有谱系上的墓碑）：每一版 `deleted_inodes` 里的号 N，这一版墓碑树里有 (N, N 的记录的对象出生代, 整对象) 且死亡 txg
/// 在这一版的父链上（每版从自己的 txg 与自己树表里的谱系树起算，判决第二节 T3 取 a）；N 在 inode 树里没有记录的那一形报射程里判不了。
pub const QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE: &str = "I-9.18";
/// I-9.19（谱系上的墓碑不指活对象）：每一版墓碑树里在谱系上的墓碑，这一版 inode 树里有号与对象出生代都相同的记录时，那条记录 nlink = 0。
pub const TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT: &str = "I-9.19";
/// I-9.20（谱系上的墓碑不重号且记录合法）：在谱系上的墓碑按 (号, 对象出生代) 不重复；全部墓碑每条整对象 [0, 2⁶⁴ − 1]、
/// 对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每片墓碑容器记录数 ≤ 815（判决第二节 T14 与第四节用户定案「补」）。
pub const TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED: &str =
    "I-9.20";
/// I-9.21（产品镜像里非最右墓碑容器装满）：没装只供测试的容量旋钮时，每一版墓碑树里沿叶序第一片容器号是 1、相邻两片容器号加一、
/// 每片（含最右那片）至少一条（第 56 批 G2）、最右那片之外每片记录数 == 815；零根或只一片容器没有对象。
pub const NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE: &str = "I-9.21";
/// I-9.22（测试镜像里墓碑容器不超旋钮值）：装了旋钮（新入口给了旋钮值）时，每一版墓碑树里沿叶序第一片容器号是 1、相邻两片容器号加一、
/// 每片至少一条（第 56 批 G2）、每片记录数 ≤ 旋钮值；零根没有对象。
pub const TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE: &str = "I-9.22";
/// I-9.23（墓碑树条目身份与子头相符）：与 I-9.17 同形，类型段 ∈ {0, 1}、子容器打包记录类型 1。
pub const TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.23";
/// I-9.24（删除队列至多一条 key）：每一版树表里每一条 `deleted_inodes` 条目都判，不管头 ID 配不配得上；空队列的一版报成立（判决第二节 T8 取 a）。
pub const DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY: &str = "I-9.24";
/// I-9.29（同号只许一个出生代）：回退候选集里各版的 inode 记录与各版自己谱系上的墓碑合起来，同一棵 inode 树里同一个号只许一个对象出生代
/// （判决第二节 T10 乙、第四节 T15「写」：D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」）。
pub const ONE_OBJECT_BIRTH_PER_INODE_NUMBER: &str = "I-9.29";
/// I-3.14（实例表每片记录形态合法，用户 2026-10-06 定另立）：走读到的实例表每一片里，每条记录的 `kind` 只有 0（行）或 1（链指针记录）；
/// 链指针记录恰好是这一片的最后一条（合法：有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值），片中间不出现；
/// `kind` 0 行的 flags 字节为 0（D18（块里携带什么信息） 已定项 11：flags「第一版恒 0、非 0 拒收」）。行唯一与低于挂载根归 I-3.8。
/// 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」）：主走读与只读不判的读法都不收它的行。
pub const INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED: &str = "I-3.14";
/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，拟定编号；W3 的 C2aL 形态，D5（快照 / 空间记账机制） 已定项 2 射程、
/// D18（块里携带什么信息） 已定项 24 第 5 条）：最新根记账里 inode 号水位那一统计量的台阶行 (g, H)——①水位不小于每一级的 H；
/// ② 最新根 inode 树里的记录与谱系上的墓碑 (号 n, 对象出生代 b) 没有一级 H > n 且 b > g；④ 每条谱系记录上，代段 ≤ T_rb − 1 的台阶里最大的 H
/// 等于记录带的 H_cur，H_cur 为 0 也判红（正推草案 `research/prompts/m3-w3-r1-sonnet-output.md` 3.1 节，加第二轮攻方 C2aL 最强形的那一判）。
pub const INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE: &str =
    "I-9.30";

/// I-9.31（退役树条目身份与子头相符）：与 I-9.17 同形，类型段 ∈ {0, 3}、子容器打包记录类型 3（判决
/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条，逐字照 I-9.17、I-9.23）。
pub const RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.31";
/// I-9.32（退役记录合法）：退役树里每条记录（不论在不在谱系上）被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg；
/// 每片退役容器的记录数 ≤ 959（正推 K5.1；记录宽 34 由 I-1.7 判）。
pub const EVERY_RETIREMENT_RECORD_IS_WELL_FORMED: &str = "I-9.32";
/// I-9.33（谱系上的退役记录按身份不重复）：死亡 txg 落在这一版父链上的退役记录，按（出生树、打包记录类型、容器号、容器出生代）不重复；
/// 只判在谱系上的（不滤谱系会在「删 → 回退 → 再删」的合法历史上误报，判决第二节 K5 那一行两次抽样坐实）。
pub const RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT: &str = "I-9.33";
/// I-9.34（谱系上的退役记录不指本版活叶）：在谱系上的退役记录指认的身份，不出现在同一版、头 ID 配得上的那棵 inode 树的叶容器身份集合里
/// （读内部条目的身份引用，不开叶）。
pub const RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF: &str = "I-9.34";
/// I-9.35（退役完备）：相邻两版（这一版的 txg 恰是这一版谱系树里某条记录的 T_rb 时父版是那条记录的 T_old，否则父版是 txg − 1；父版在回退候选集里
/// 读得出才判）：父版叶身份集 − 本版叶身份集 = 本版谱系上、死亡 txg == 本版 txg 的退役记录的身份集，两个方向各报。最新根的实例表读不出、
/// 滤不掉被崩溃恢复抛弃的根时报部分没判（照 I-9.29 的门）。被攻过零轮（判决第四节）。
pub const RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION: &str = "I-9.35";
/// I-9.36（产品镜像里非最右谱系与退役容器装满，与 I-9.21 同形）：那一棵没装只供测试的容量旋钮时，每一版谱系树与退役树里沿叶序第一片容器号是 1、
/// 相邻两片容器号加一、每片（含最右那片）至少一条（第 56 批 G2）、最右那片之外每片记录数 == 产品容量（谱系 1359、退役 959；开放容器满了才关，
/// D18（块里携带什么信息） 已定项 24）；零根或只一片容器没有对象。
pub const NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE: &str =
    "I-9.36";
/// I-9.37（测试镜像里谱系与退役容器不超旋钮值，与 I-9.22 同形）：那一棵装了旋钮（新入口给了旋钮值）时，每一版那棵树里沿叶序第一片容器号是 1、
/// 相邻两片容器号加一、每片至少一条（第 56 批 G2）、每片记录数 ≤ 旋钮值；零根没有对象。
pub const LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE: &str =
    "I-9.37";

/// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
#[derive(Default)]
pub struct Judgements {
    evaluated: BTreeMap<&'static str, u64>,
    violations: BTreeMap<&'static str, u64>,
    first_violation: BTreeMap<&'static str, String>,
    not_applicable: BTreeMap<&'static str, &'static str>,
    part_of_the_range_not_judged: BTreeMap<&'static str, &'static str>,
    partially_judged: BTreeMap<&'static str, String>,
}

impl Judgements {
    /// 评估一次：条件不成立就记下这一处（只留第一处）。
    pub fn judge(&mut self, invariant: &'static str, holds: bool, detail: impl FnOnce() -> String) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        *self.evaluated.entry(invariant).or_insert(0) += 1;
        if !holds {
            *self.violations.entry(invariant).or_insert(0) += 1;
            self.first_violation.entry(invariant).or_insert_with(detail);
        }
    }

    /// 到此为止判了几次违例：走读一条候选根前后各读一次，差就是这条根引用的单元里对不上的那些（I-7.4、I-4.8 按根判）。
    #[must_use]
    pub fn violation_count(&self, invariant: &'static str) -> u64 {
        self.violations.get(invariant).copied().unwrap_or(0)
    }
    /// 这个镜像上整条不适用（例如第 0 代根下面没有记账树）。已经评估过的不改。
    pub fn not_applicable(&mut self, invariant: &'static str, reason: &'static str) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.not_applicable.entry(invariant).or_insert(reason);
    }
    /// 这条不变量射程里有一部分在这个镜像上判不了（例如 I-7.4（近 K 代块未被复用） 被抛弃根那一半：根环有读不出的槽）：
    /// 判了的那一部分成立不等于整条成立，没有违例时整条报「不适用」带这个理由、不报成立；判出了违例照报违例。
    pub fn part_of_the_range_not_judged(&mut self, invariant: &'static str, reason: &'static str) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.part_of_the_range_not_judged
            .entry(invariant)
            .or_insert(reason);
    }
    /// 这条不变量射程里有一部分在这个镜像上判不了、要报「部分没判」（[`InvariantVerdict::PartiallyJudged`]）：`unjudged` 逐条列出没判到的是什么。
    /// 判出了违例照报违例；没有违例时报部分没判，压过「成立」与「不适用」。只记第一次给的说明。
    pub fn partially_judged(&mut self, invariant: &'static str, unjudged: String) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.partially_judged.entry(invariant).or_insert(unjudged);
    }
    /// 这条不变量到此为止的第一处违例说明：一次走读前后各看一眼，走读里第一处对不上的是什么（I-7.4 被抛弃根那一半报它）。
    #[must_use]
    pub fn first_violation_of(&self, invariant: &'static str) -> Option<&str> {
        self.first_violation.get(invariant).map(String::as_str)
    }
    #[must_use]
    pub fn into_report(self) -> Vec<(&'static str, InvariantVerdict)> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                let verdict = if let Some(detail) = self.first_violation.get(invariant) {
                    InvariantVerdict::Violated(detail.clone())
                } else if let Some(unjudged) = self.partially_judged.get(invariant) {
                    InvariantVerdict::PartiallyJudged(unjudged.clone())
                } else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {
                    InvariantVerdict::NotApplicable(reason)
                } else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {
                    InvariantVerdict::Holds
                } else {
                    InvariantVerdict::NotApplicable(
                        self.not_applicable
                            .get(invariant)
                            .copied()
                            .unwrap_or("这个镜像上没有它判的对象"),
                    )
                };
                (*invariant, verdict)
            })
            .collect()
    }
}

/// 系统配置里 checker 要用的几何（偏移按系统配置字段表各写一份：物理块 317、journal 环起点 325 与长度 333、记录尺寸 341、
/// 在飞上限 345、最坏占用 349、安全系数 F 357、R 361、S 362、P 363、chunk 367、根环基址 371、逐区域设备 379、单元区起点 417、
/// io_min 425、槽距 429）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolGeometry {
    pub filesystem_identifier: [u8; 16],
    pub physical_block_size: u32,
    pub journal_ring_start_slot: u64,
    pub journal_ring_bytes: u64,
    /// 槽里自述的 journal 在飞记录数上限（偏移 345，4 字节）：I-8.1 拿它与 ⌊环长 ÷ 4096 ÷ F⌋ 比、拿它数同一次发布的记录条数。
    /// I-4.1 的重放前缀不按它截，按环长与格式常量 F 现算（D23（journal 的角色与格式） 已定项 18，与实现恢复同一个读法）。
    pub declared_in_flight_record_limit: u64,
    /// 槽里自述的 journal 最坏占用（偏移 349，8 字节，单位字节）：I-8.1 拿它与在飞上限 × 4096 比。
    pub declared_worst_case_occupancy_bytes: u64,
    /// 槽里自述的安全系数 F（偏移 357，4 字节）：I-8.1 判它等于格式常量、拿它算在飞上限。
    pub declared_safety_factor: u64,
    /// 槽里自述的 io_min（偏移 425，4 字节；0 = 没声明）：I-7.5 的判定宽度取 max(physical_block_size, io_min)。
    pub declared_minimum_input_output_bytes: u32,
    pub regions: u64,
    pub slots_per_region: u64,
    pub prime_step: u64,
    pub chunk_bytes: u64,
    pub base_slot: u64,
    pub region_devices: [u32; 3],
    pub unit_area_start_slot: u64,
    pub slot_spacing: u64,
}

/// 系统配置字段表给逐区域设备身份留了几个字段：偏移 379 / 383 / 387，一共三个。
/// R（偏移 361 那一字节）是**盘上读来的值**，可以是 0..255 里的任何一个；它大于这个数时，
/// 第 3 个及以后的区域在字段表里根本没有设备身份可读——这份系统配置这个格式版本读不了。
pub const REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION: usize = 3;

/// 从一个自证过的系统配置槽里取几何。
///
/// # Errors
/// R 大于 [`REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION`] ⇒
/// [`crate::Verdict::RegionCountPastTheRegionDeviceFields`]：不夹成 3 往下走（那是静默跳过几个区域，
/// 根环少读几个槽而没人说），也不 panic（panic 面普查 R12：`root_slot_positions` 拿它下标一个长 3 的数组）。
///
/// S（偏移 362 那一字节）不在格式承诺的区间里 ⇒
/// [`crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval`]，同一条理由：
/// 夹一下往下走就是按一个不存在的几何走读。两条判据都在盘上那一字节进 [`PoolGeometry`] 的唯一入口上。
///
/// 固定结构槽距、`physical_block_size`、journal 环长同样是盘上读来的数，同一个入口上判
/// （`fixed_structure_slot_spacing_lies_in_the_format_range`、`physical_block_size_fits_a_root_slot`、
/// `journal_ring_bytes_lie_in_the_supported_range`），越界各报一个成员。这三个成员与 R、S 那两个处置不同：池级读法把它们认成
/// 「带这一版读者不收的值」（[`value_refused_by_this_reader`]），报违例、整池不作保（用户 2026-09-27 定系统配置越界整池拒）；
/// R、S 越界照旧是这一槽不可择。槽距先判：它是根槽宽的上界。
///
/// journal 环起点（325）、根环起点（371）不等于第一版的格式常量，单元区起始槽号（417）不是环长现算的那个槽或不在聚簇段边界上，
/// 同样各报一个成员、同样认成「带这一版读者不收的值」（实审 Y4-a，用户 2026-09-27 定「读字段，不等于常量就整池拒」）。
/// 两个起点先于槽距判：槽距的上界按同一槽自述的根环起点算，起点不对时报起点、不报槽距。
pub fn geometry_of(
    slot: &[u8],
    view: &SystemConfigurationView,
) -> Result<PoolGeometry, crate::Verdict> {
    let regions = u64::from(slot[361]);
    if regions
        > u64::try_from(REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION).expect("三个区域字段")
    {
        return Err(crate::Verdict::RegionCountPastTheRegionDeviceFields);
    }
    let slots_per_region = u64::from(slot[362]);
    if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)
        .contains(&slots_per_region)
    {
        return Err(crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval);
    }
    let journal_ring_start_slot = read_u64(slot, 325);
    let journal_ring_bytes = read_u64(slot, 333);
    let base_slot = read_u64(slot, 371);
    let unit_area_start_slot = read_u64(slot, 417);
    let slot_spacing = read_u32(slot, 429);
    if journal_ring_start_slot != JOURNAL_RING_START_SLOT {
        return Err(crate::Verdict::JournalRingStartSlotNotTheFirstVersionConstant);
    }
    if base_slot != ROOT_RING_BASE_SLOT {
        return Err(crate::Verdict::RootRingBaseSlotNotTheFirstVersionConstant);
    }
    if !fixed_structure_slot_spacing_lies_in_the_format_range(slot_spacing, base_slot) {
        return Err(crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange);
    }
    if !physical_block_size_fits_a_root_slot(view.declared_physical_block_size, slot_spacing) {
        return Err(crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds);
    }
    if !journal_ring_bytes_lie_in_the_supported_range(
        journal_ring_start_slot,
        journal_ring_bytes,
        unit_area_start_slot,
    ) {
        return Err(crate::Verdict::JournalRingBytesOutsideTheSupportedRange);
    }
    if !unit_area_start_slot_is_the_slot_after_the_journal_ring(
        journal_ring_start_slot,
        journal_ring_bytes,
        unit_area_start_slot,
    ) {
        return Err(crate::Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing);
    }
    if !unit_area_start_slot.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
        return Err(crate::Verdict::UnitAreaStartOffTheClusterSegmentBoundary);
    }
    Ok(PoolGeometry {
        filesystem_identifier: view.filesystem_identifier,
        physical_block_size: view.declared_physical_block_size,
        journal_ring_start_slot,
        journal_ring_bytes,
        declared_in_flight_record_limit: u64::from(read_u32(slot, 345)),
        declared_worst_case_occupancy_bytes: read_u64(slot, 349),
        declared_safety_factor: u64::from(read_u32(slot, 357)),
        declared_minimum_input_output_bytes: read_u32(slot, 425),
        regions,
        slots_per_region,
        prime_step: u64::from(read_u32(slot, 363)),
        chunk_bytes: u64::from(read_u32(slot, 367)),
        base_slot,
        region_devices: [
            read_u32(slot, 379),
            read_u32(slot, 383),
            read_u32(slot, 387),
        ],
        unit_area_start_slot,
        slot_spacing: u64::from(slot_spacing),
    })
}

/// 固定结构槽距在格式允许的区间里：不小于 4096（D2（RAID 条带策略） 已定项 19「槽距 = 4096 向上取整到 io_min 的整数倍」），
/// 且系统配置槽 1（设备内偏移 = 槽距、宽 4096，D22（单元原子性怎么合成） 已定项 16）整槽落在根环起点之前——
/// 根环起点取同一槽「根环起点」字段 × 16384（已定项 16 第 1 句），不取编译期常量：checker 走读根环也按这个字段。
/// 那个字段是盘上读来的 8 字节，乘出来超过 u64 时根环起点在任何设备偏移之外，槽 1 自然落在它之前。
fn fixed_structure_slot_spacing_lies_in_the_format_range(
    slot_spacing: u32,
    root_ring_base_slot: u64,
) -> bool {
    let slot_one_end = u64::from(slot_spacing) + SYSTEM_CONFIGURATION_SLOT_BYTES;
    u64::from(slot_spacing) >= FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        && root_ring_base_slot
            .checked_mul(SLOT_BYTES)
            .is_none_or(|root_ring_base_offset| slot_one_end <= root_ring_base_offset)
}

/// `physical_block_size` 是根槽宽（D22（单元原子性怎么合成） 已定项 2「根槽宽等于挂载时探测的 physical_block_size」）：
/// 装得下一条根记录（`ROOT_RECORD_BYTES`，已定项 7），且不宽过固定结构槽距（根槽 j 在区域起点 + j × 槽距，已定项 16 第 2 句；
/// 宽过槽距相邻两槽就叠在一起）。读根槽按它开缓冲，不设上界时盘上 4 字节就定得了每一次读要多少内存。
fn physical_block_size_fits_a_root_slot(physical_block_size: u32, slot_spacing: u32) -> bool {
    u64::from(physical_block_size) >= ROOT_RECORD_BYTES && physical_block_size <= slot_spacing
}

/// journal 环长装得下 F 条记录（在飞记录数上限 = 环槽数 ÷ F 不是 0，D23（journal 的角色与格式） 已定项 18；F 取格式常量
/// `JOURNAL_SAFETY_FACTOR`，不读槽里那 4 字节，那一格没有上下界可判），且环的末端（同一槽「journal 环起点」字段 × 16384 + 环长）
/// 不越过同一槽的「单元区起始槽号」字段 × 16384（已定项 19 ③：单元区起始槽号随环长走；checker 扫单元区也按这个字段）。
/// 两个槽号字段都是盘上读来的 8 字节：环末端乘加溢出算越界；单元区起点乘出来超过 u64 时它在任何设备偏移之外。
fn journal_ring_bytes_lie_in_the_supported_range(
    journal_ring_start_slot: u64,
    journal_ring_bytes: u64,
    unit_area_start_slot: u64,
) -> bool {
    let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;
    let journal_ring_end = journal_ring_start_slot
        .checked_mul(SLOT_BYTES)
        .and_then(|journal_ring_start| journal_ring_start.checked_add(journal_ring_bytes));
    in_flight_record_limit >= 1
        && journal_ring_end.is_some_and(|journal_ring_end| {
            unit_area_start_slot
                .checked_mul(SLOT_BYTES)
                .is_none_or(|unit_area_start| journal_ring_end <= unit_area_start)
        })
}

/// 单元区起始槽号是 journal 环末尾的下一个槽：同一槽自述的环起点 + 环长向上取整到 16 KiB 槽（D3（空间分配） 已定项 10 ④
/// 「第一版 = journal 环末尾的下一个槽」；D23（journal 的角色与格式） 已定项 19 ③「单元区起始槽号随环长走」）。
/// checker 自己按条款算一份，不引实现的 `journal::slot_after_the_journal_ring`（门禁 94 号）。环长是盘上读来的 8 字节：
/// 槽数不超过 2⁶⁴ ÷ 16384，加环起点溢出时算不相等。
fn unit_area_start_slot_is_the_slot_after_the_journal_ring(
    journal_ring_start_slot: u64,
    journal_ring_bytes: u64,
    unit_area_start_slot: u64,
) -> bool {
    journal_ring_start_slot
        .checked_add(journal_ring_bytes.div_ceil(SLOT_BYTES))
        .is_some_and(|slot_after_the_journal_ring| {
            unit_area_start_slot == slot_after_the_journal_ring
        })
}

/// 一个系统配置槽读下来是什么样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemConfigurationSlotReading {
    /// 自证过、incompat 位认得、池级字段都在这一版读者收的范围里：可择。
    Usable(SystemConfigurationView, PoolGeometry),
    /// 自证过（magic 与整槽校验和都对）、incompat 位认得、R 与 S 在区间里，而一个池级字段越出这一版读者收的范围
    /// （[`value_refused_by_this_reader`] 认的那几个 [`crate::Verdict`] 成员；逐档读槽 1 时还有 I-8.1 三个字段等式，
    /// [`RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees`]）。实现遇到这一槽整池拒绝挂载
    /// （`singlefs_core::recovery::RecoveryFailure::SystemConfigurationValueRefused`）；checker 同一个处置：报
    /// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`] 违例、不替这个池作保，不退成「这一槽不可择」静默跳过
    /// （用户 2026-09-27 定系统配置越界整池拒）。
    CarriesAValueThisReaderRefuses {
        offset_in_bytes: u64,
        refused: RefusedSystemConfigurationValue,
    },
    /// 不自证：读不到、不足 4096 字节、magic 或整槽校验和不对（I-8.5（tail 槽有效集非空） 的「自证过」= magic 与整槽校验和都对）。
    NotSelfVerified,
    /// 自证过（magic 与整槽校验和都对），而 incompat 位不认得、或 R / S 越界：这一槽不可择（R、S 的既有处置，不动）。它仍是 I-8.5 数的「自证过的槽」：tail 住在自证过的槽里（D23（journal 的角色与格式） 已定项 3）。
    SelfVerifiedButNotChoosable,
}

/// 一个带这一版读者不收之值的系统配置槽（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）拒在哪一格。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefusedSystemConfigurationValue {
    /// 自证与几何那几道（[`crate::check_system_configuration_slot`]、[`geometry_of`]）拒的，是 [`value_refused_by_this_reader`] 认的那几个成员之一。
    RefusedByTheSlotChecks(crate::Verdict),
    /// I-8.1（环几何够大） 的三个字段等式里第一个不成立的那一格（[`journal_ring_geometry_field_that_disagrees`]）。只在逐档读槽 1 时这样交回
    /// （[`slot_one_read_at_every_slot_spacing_the_format_allows`]）：实现那张「读者不收之值」表的最后三格就是这三个等式
    /// （`singlefs_core::recovery` 的 `system_configuration_values_this_reader_accepts`），逐档读到它整池拒；
    /// 别的读法上可择的槽照旧由 [`judge_journal_ring_geometry_fields_agree`] 按 I-8.1 判。
    JournalRingGeometryFieldDisagrees(JournalRingGeometryFieldThatDisagrees),
}

/// I-8.1（环几何够大） 三个字段等式里的哪一格不成立，次序与实现判这三格的次序相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRingGeometryFieldThatDisagrees {
    /// 槽里的安全系数 F 不等于格式常量 `JOURNAL_SAFETY_FACTOR`。
    SafetyFactorNotTheFormatConstant,
    /// 槽里记的在飞上限不等于 ⌊环长 ÷ 4096 ÷ F⌋。
    InFlightRecordLimitNotRecomputedFromTheRingLength,
    /// 槽里记的最坏占用不等于在飞上限 × 4096（乘出来溢出也算不等）。
    WorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes,
}

/// I-8.1 第二格右边：按同一槽的环长与 F 现算的在飞上限 ⌊环长 ÷ 4096 ÷ F⌋；F 为 0 时没有这个数（None，那一格判不等、不除）。
const fn in_flight_record_limit_recomputed_from_the_ring(geometry: &PoolGeometry) -> Option<u64> {
    (geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES)
        .checked_div(geometry.declared_safety_factor)
}

/// I-8.1 第三格右边：槽里记的在飞上限 × 4096；乘出来溢出时没有这个数（None，那一格判不等）。
const fn worst_case_occupancy_recomputed_from_the_limit(geometry: &PoolGeometry) -> Option<u64> {
    geometry
        .declared_in_flight_record_limit
        .checked_mul(JOURNAL_RECORD_BYTES)
}

/// I-8.1（环几何够大） 的三个字段等式按 F、在飞上限、最坏占用的次序判，交回第一个不成立的；三格都成立交回 None。
/// 三格与 [`judge_journal_ring_geometry_fields_agree`] 判的是同三个等式（后两格的右边共用上面两个算式）。
#[must_use]
pub fn journal_ring_geometry_field_that_disagrees(
    geometry: &PoolGeometry,
) -> Option<JournalRingGeometryFieldThatDisagrees> {
    if geometry.declared_safety_factor != JOURNAL_SAFETY_FACTOR {
        return Some(JournalRingGeometryFieldThatDisagrees::SafetyFactorNotTheFormatConstant);
    }
    if in_flight_record_limit_recomputed_from_the_ring(geometry)
        != Some(geometry.declared_in_flight_record_limit)
    {
        return Some(
            JournalRingGeometryFieldThatDisagrees::InFlightRecordLimitNotRecomputedFromTheRingLength,
        );
    }
    if worst_case_occupancy_recomputed_from_the_limit(geometry)
        != Some(geometry.declared_worst_case_occupancy_bytes)
    {
        return Some(
            JournalRingGeometryFieldThatDisagrees::WorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes,
        );
    }
    None
}

/// 一个自证过的系统配置槽被拒的理由是不是「池级字段越出这一版读者收的范围」。穷举写全：新加一个 [`crate::Verdict`] 成员时这里编译不过，
/// 逼着判它归哪一边。
#[must_use]
pub const fn value_refused_by_this_reader(verdict: &crate::Verdict) -> bool {
    match verdict {
        crate::Verdict::FormatVersionNotRecognized
        | crate::Verdict::EncryptionTypeNotOff
        | crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange
        | crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds
        | crate::Verdict::JournalRingBytesOutsideTheSupportedRange
        | crate::Verdict::JournalRingStartSlotNotTheFirstVersionConstant
        | crate::Verdict::RootRingBaseSlotNotTheFirstVersionConstant
        | crate::Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing
        | crate::Verdict::UnitAreaStartOffTheClusterSegmentBoundary => true,
        crate::Verdict::Valid
        | crate::Verdict::BadMagic
        | crate::Verdict::ChecksumMismatch
        | crate::Verdict::UnknownIncompatBit
        | crate::Verdict::FilesystemIdentifierMismatch
        | crate::Verdict::NonZeroFlags
        | crate::Verdict::TooShort
        | crate::Verdict::WrongUnitClass
        | crate::Verdict::DeclaredLengthMismatch
        | crate::Verdict::KeyWidthMismatch
        | crate::Verdict::KeysNotStrictlyAscending
        | crate::Verdict::KeyOutsideDeclaredRange
        | crate::Verdict::UnknownRecordType
        | crate::Verdict::RegionCountPastTheRegionDeviceFields
        | crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
        | crate::Verdict::EncryptionReservedBytesNotZero
        | crate::Verdict::EntryWidthZeroWithEntries
        | crate::Verdict::RecordWidthZeroWithRecords
        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero => false,
    }
}

/// 一个系统配置槽被拒的理由是不是「它自己证不了自己」：不足 4096 字节、magic 不对、整槽校验和不对
/// （[`crate::check_system_configuration_slot`] 头三道，过了这三道之后拒的都是自证过的槽）。穷举写全，理由同 [`value_refused_by_this_reader`]。
#[must_use]
pub const fn verdict_says_the_slot_does_not_verify_itself(verdict: &crate::Verdict) -> bool {
    match verdict {
        crate::Verdict::TooShort | crate::Verdict::BadMagic | crate::Verdict::ChecksumMismatch => {
            true
        }
        crate::Verdict::Valid
        | crate::Verdict::FormatVersionNotRecognized
        | crate::Verdict::EncryptionTypeNotOff
        | crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange
        | crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds
        | crate::Verdict::JournalRingBytesOutsideTheSupportedRange
        | crate::Verdict::JournalRingStartSlotNotTheFirstVersionConstant
        | crate::Verdict::RootRingBaseSlotNotTheFirstVersionConstant
        | crate::Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing
        | crate::Verdict::UnitAreaStartOffTheClusterSegmentBoundary
        | crate::Verdict::UnknownIncompatBit
        | crate::Verdict::FilesystemIdentifierMismatch
        | crate::Verdict::NonZeroFlags
        | crate::Verdict::WrongUnitClass
        | crate::Verdict::DeclaredLengthMismatch
        | crate::Verdict::KeyWidthMismatch
        | crate::Verdict::KeysNotStrictlyAscending
        | crate::Verdict::KeyOutsideDeclaredRange
        | crate::Verdict::UnknownRecordType
        | crate::Verdict::RegionCountPastTheRegionDeviceFields
        | crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
        | crate::Verdict::EncryptionReservedBytesNotZero
        | crate::Verdict::EntryWidthZeroWithEntries
        | crate::Verdict::RecordWidthZeroWithRecords
        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero => false,
    }
}

fn read_system_configuration_slot(
    reader: &dyn ImageReader,
    device: u32,
    offset_in_bytes: u64,
) -> SystemConfigurationSlotReading {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let Some(bytes) = reader.read(device, offset_in_bytes, slot_bytes) else {
        return SystemConfigurationSlotReading::NotSelfVerified;
    };
    let refused_or_not_choosable = |verdict: crate::Verdict| {
        if value_refused_by_this_reader(&verdict) {
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                offset_in_bytes,
                refused: RefusedSystemConfigurationValue::RefusedByTheSlotChecks(verdict),
            }
        } else if verdict_says_the_slot_does_not_verify_itself(&verdict) {
            SystemConfigurationSlotReading::NotSelfVerified
        } else {
            SystemConfigurationSlotReading::SelfVerifiedButNotChoosable
        }
    };
    let view = match check_system_configuration_slot(&bytes) {
        Ok(view) => view,
        Err(verdict) => return refused_or_not_choosable(verdict),
    };
    match geometry_of(&bytes, &view) {
        Ok(geometry) => SystemConfigurationSlotReading::Usable(view, geometry),
        Err(verdict) => refused_or_not_choosable(verdict),
    }
}

/// 逐档读槽 1 时相邻两档槽距之差：槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍（D2（RAID 条带策略） 已定项 19），
/// io_min 是逻辑块宽的整数倍、逻辑块宽最小 512 字节，所以格式允许的槽距都是 512 的整数倍。
const SLOT_SPACING_STEP_BYTES: u64 = 512;

/// 逐档读槽 1 时槽距的上界：槽 1 整槽落在根环起点之前（[`fixed_structure_slot_spacing_lies_in_the_format_range`]），
/// 根环起点取这一版读者收的那一个（「根环起点」字段只收第一版常量 [`ROOT_RING_BASE_SLOT`]，[`geometry_of`] 判），× 16384 − 4096。
const LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS: u64 =
    ROOT_RING_BASE_SLOT * SLOT_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;

/// 全池没有一个可用的槽 0 时，一块盘的槽 1：槽距无处可读，在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、
/// 不越过 [`LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS`]），只收可用、且自述的固定结构槽距等于所在偏移的槽
/// （槽 i 的设备内偏移 = i × 固定结构槽距，D22（单元原子性怎么合成） 已定项 16）；几档都收得到时取世代号最大的，
/// 世代号相同取偏移小的（C651（全池没有可用的系统配置槽 0 时 checker 与实现找槽 1 的读法不同） 第一轮判决第四节第 1、2 条，
/// `research/prompts/m3-c651-r1-main-verification.md`）。
///
/// 读到的某一档带这一版读者不收的值（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）：交回按偏移升序读到的第一个，
/// 由 I-7.13（系统配置池级字段在读者收的范围里） 报违例、整池不作保——D22（单元原子性怎么合成） 已定项 16 第 5 句「越界即拒」，
/// 与槽 0、槽 1 一个处置；它记的槽距可能正是越界的那一格，不核偏移。
/// 读到可用的一档，先按实现那张「读者不收之值」表剩下的三格判（I-8.1（环几何够大） 三个字段等式，[`journal_ring_geometry_field_that_disagrees`]），
/// 有一格不成立就同样当场交回带不收之值，不再核偏移、比世代号：已定项 16 定案「逐档读到自证过而池级字段越界的槽整池拒」不要求
/// 槽距等于偏移，实现照字面先判值后核偏移（`singlefs_core::recovery` 的 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`；
/// 代码轮第二轮判决 K3-1，`research/prompts/m3-formal-code-r2-main-verification.md`）。
/// 一档可用的都没收到时，有自证过而不可择的就交回 [`SystemConfigurationSlotReading::SelfVerifiedButNotChoosable`]
/// （I-8.5（tail 槽有效集非空） 数它；它记的槽距可能读不出来，不核偏移），再没有就是 [`SystemConfigurationSlotReading::NotSelfVerified`]。
fn slot_one_read_at_every_slot_spacing_the_format_allows(
    reader: &dyn ImageReader,
    device: u32,
) -> SystemConfigurationSlotReading {
    let mut highest_generation_accepted: Option<(SystemConfigurationView, PoolGeometry)> = None;
    let mut some_slot_verifies_itself_but_is_not_choosable = false;
    // 迭代次数的上界是档数 (LARGEST − 4096) ÷ 512 + 1 = 2033；跨轮带着的是至今世代号最大的那一槽与「见过自证过而不可择的」，
    // 前者只在世代号严格更大时换（同号留偏移小的），后者只置不清；提前出口只有读到带不收之值的那一档。
    for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        ..=LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS)
        .step_by(usize::try_from(SLOT_SPACING_STEP_BYTES).expect("512"))
    {
        match read_system_configuration_slot(reader, device, candidate_spacing) {
            SystemConfigurationSlotReading::Usable(view, geometry) => {
                if let Some(disagreeing_field) =
                    journal_ring_geometry_field_that_disagrees(&geometry)
                {
                    return SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                        offset_in_bytes: candidate_spacing,
                        refused: RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees(
                            disagreeing_field,
                        ),
                    };
                }
                if geometry.slot_spacing != candidate_spacing {
                    continue;
                }
                if highest_generation_accepted
                    .as_ref()
                    .is_none_or(|(accepted, _)| view.slot_generation > accepted.slot_generation)
                {
                    highest_generation_accepted = Some((view, geometry));
                }
            }
            refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {
                return refused;
            }
            SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {
                some_slot_verifies_itself_but_is_not_choosable = true;
            }
            SystemConfigurationSlotReading::NotSelfVerified => {}
        }
    }
    match (
        highest_generation_accepted,
        some_slot_verifies_itself_but_is_not_choosable,
    ) {
        (Some((view, geometry)), _) => SystemConfigurationSlotReading::Usable(view, geometry),
        (None, true) => SystemConfigurationSlotReading::SelfVerifiedButNotChoosable,
        (None, false) => SystemConfigurationSlotReading::NotSelfVerified,
    }
}

/// 每盘两槽各读成一个 [`SystemConfigurationSlotReading`]：槽 0 在偏移 0；槽 1 的偏移按这块盘槽 0 记的槽距，这块盘的槽 0 不可用时
/// 按池里（`reader.devices()` 的次序）第一块槽 0 可用的盘记的槽距（2026-09-14 用户收尾弹窗定甲；借别的盘那一档与恢复
/// `crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration` 同一个读法，实审 A2a 第 37 条改的，这里跟着对齐）；
/// 全池的槽 0 都不可用时逐档读（[`slot_one_read_at_every_slot_spacing_the_format_allows`]，C651 第一轮判决定逐档，取代此前的最小槽距 4096）。
#[must_use]
pub fn system_configuration_slot_readings(
    reader: &dyn ImageReader,
) -> Vec<(u32, [SystemConfigurationSlotReading; 2])> {
    let slot_zero_of_each_device: Vec<(u32, SystemConfigurationSlotReading)> = reader
        .devices()
        .into_iter()
        .map(|device| (device, read_system_configuration_slot(reader, device, 0)))
        .collect();
    let spacing_recorded_by_slot_zero = |reading: &SystemConfigurationSlotReading| match reading {
        SystemConfigurationSlotReading::Usable(_, geometry) => Some(geometry.slot_spacing),
        SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
        | SystemConfigurationSlotReading::NotSelfVerified
        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
    };
    let spacing_recorded_by_the_first_valid_slot_zero = slot_zero_of_each_device
        .iter()
        .find_map(|(_, slot_zero)| spacing_recorded_by_slot_zero(slot_zero));
    slot_zero_of_each_device
        .into_iter()
        .map(|(device, slot_zero)| {
            let slot_one = match spacing_recorded_by_slot_zero(&slot_zero)
                .or(spacing_recorded_by_the_first_valid_slot_zero)
            {
                Some(spacing) => read_system_configuration_slot(reader, device, spacing),
                None => slot_one_read_at_every_slot_spacing_the_format_allows(reader, device),
            };
            (device, [slot_zero, slot_one])
        })
        .collect()
}

/// 每盘两槽里可择的系统配置（[`SystemConfigurationSlotReading::Usable`]），槽的读法见 [`system_configuration_slot_readings`]。
/// I-7.7（系统配置实例代号不低于根环） 按这个读法取每盘的实例代号（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）。
#[must_use]
pub fn verified_system_configuration_slots(
    reader: &dyn ImageReader,
) -> Vec<(u32, Vec<(SystemConfigurationView, PoolGeometry)>)> {
    system_configuration_slot_readings(reader)
        .into_iter()
        .map(|(device, readings)| {
            let usable = readings
                .into_iter()
                .filter_map(|reading| match reading {
                    SystemConfigurationSlotReading::Usable(view, geometry) => {
                        Some((view, geometry))
                    }
                    SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
                    | SystemConfigurationSlotReading::NotSelfVerified
                    | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
                })
                .collect();
            (device, usable)
        })
        .collect()
}

/// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`]：每个可择的槽判一次成立，每个带越界值的槽判一次违例。
/// 交回有没有带越界值的槽——有就是实现整池拒绝挂载的镜像（`RecoveryFailure::SystemConfigurationValueRefused`，任一盘任一槽）。
pub fn judge_system_configuration_values_the_reader_accepts(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) -> bool {
    let mut any_slot_carries_a_refused_value = false;
    for (device, slots) in readings {
        for slot in slots {
            match slot {
                SystemConfigurationSlotReading::Usable(..) => judgements.judge(
                    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                    true,
                    String::new,
                ),
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                    offset_in_bytes,
                    refused,
                } => {
                    any_slot_carries_a_refused_value = true;
                    judgements.judge(
                        SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                        false,
                        || {
                            format!(
                                "盘 {device} 偏移 {offset_in_bytes} 的系统配置槽自证过，而池级字段越出这一版读者收的范围（{refused:?}）：实现整池拒绝挂载"
                            )
                        },
                    );
                }
                SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {}
            }
        }
    }
    any_slot_carries_a_refused_value
}

/// [`SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY`]：每块盘两槽里每个自证过、属于本池（fsid 相同）的系统配置槽，
/// 自述的本盘设备号等于这块盘在池里的身份（`ImageReader::devices` 给的那个号），每槽判一次。读法与实现可写挂载比盘表时同一个：
/// 自证过的槽、fsid 与择到的那份相同的槽（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`）；
/// 别的池的槽归 I-1.4 判，不在这里。盘体对调（身份 0 的位置上装着自述 1 的盘）时两块盘各红一处。
pub fn judge_own_device_number_is_the_device_identity(
    verified_system_configurations_of_this_pool: &[(u32, Vec<crate::SystemConfigurationView>)],
    judgements: &mut Judgements,
) {
    for (device_identity, slots) in verified_system_configurations_of_this_pool {
        for view in slots {
            judgements.judge(
                SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
                view.this_device == *device_identity,
                || {
                    format!(
                        "盘 {device_identity} 上世代 {} 的系统配置槽自述本盘设备号 {}：与这块盘在池里的身份不同",
                        view.slot_generation, view.this_device
                    )
                },
            );
        }
    }
}

/// 每盘择一个系统配置：两槽里世代号大的那一份（相等取槽 0）。
#[must_use]
pub fn chosen_system_configurations(
    reader: &dyn ImageReader,
) -> Vec<(u32, Option<(SystemConfigurationView, PoolGeometry)>)> {
    verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(device, slots)| {
            let chosen = slots.into_iter().fold(
                None,
                |best: Option<(SystemConfigurationView, PoolGeometry)>, candidate| match best {
                    Some(current) if candidate.0.slot_generation <= current.0.slot_generation => {
                        Some(current)
                    }
                    Some(_) | None => Some(candidate),
                },
            );
            (device, chosen)
        })
        .collect()
}

/// 根环全部槽：(区域, 槽, 盘, 偏移)，择根与 I-7.7（系统配置实例代号不低于根环） 的「读不全」判定共用这一个算式。
#[must_use]
pub fn root_slot_positions(geometry: &PoolGeometry) -> Vec<(u64, u64, u32, u64)> {
    let mut positions = Vec::new();
    for region in 0..geometry.regions {
        // 下标在范围内不是这里判的：`geometry_of` 是盘上那一字节进 `PoolGeometry` 的唯一入口，
        // 它已经把 R > 3 的槽拒成 `Verdict::RegionCountPastTheRegionDeviceFields`、不交出 `PoolGeometry`。
        let device = geometry.region_devices[usize::try_from(region).expect(
            "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
        )];
        for slot in 0..geometry.slots_per_region {
            let offset = geometry.base_slot * SLOT_BYTES
                + region * geometry.prime_step * geometry.chunk_bytes
                + slot * geometry.slot_spacing;
            positions.push((region, slot, device, offset));
        }
    }
    positions
}

/// mkfs 写下的两个系统配置槽的世代号（D22（单元原子性怎么合成） 已定项 16，用户 2026-10-01 定写进那一项）：两槽相等只许都是它。
pub const SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS: u64 = 1;

/// 每块盘的系统配置槽数（D22（单元原子性怎么合成） 已定项 8：每盘两份）：世代号 ≥ 2 的那一份住「世代号 mod 这个数」号槽。
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;

/// [`SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION`]（I-7.16）：每块盘上，可择的系统配置槽 i（[`SystemConfigurationSlotReading::Usable`]，
/// 带越界值的槽归 I-7.13 整池拒、不进这一条）世代号 ≥ 2 时世代号 mod 2 == i，每槽判一格；两槽都可择时世代号相差恰为 1，
/// 或两槽都是 mkfs 写下的 1，每盘判一格（D22（单元原子性怎么合成） 已定项 16）。
pub fn judge_system_configuration_slot_generations_follow_the_rotation(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    let generation_of_a_choosable_slot = |reading: &SystemConfigurationSlotReading| match reading {
        SystemConfigurationSlotReading::Usable(view, _) => Some(view.slot_generation),
        SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
        | SystemConfigurationSlotReading::NotSelfVerified
        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
    };
    for (device, slots) in readings {
        let generations = [
            generation_of_a_choosable_slot(&slots[0]),
            generation_of_a_choosable_slot(&slots[1]),
        ];
        for (slot_index, generation) in (0u64..).zip(generations) {
            let Some(generation) = generation else {
                continue;
            };
            if generation > SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS {
                judgements.judge(
                    SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
                    generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE == slot_index,
                    || {
                        format!(
                            "盘 {device} 槽 {slot_index} 的系统配置世代号 {generation}：世代号 ≥ 2 的那一份该住槽 {}",
                            generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                        )
                    },
                );
            }
        }
        if let [Some(slot_zero_generation), Some(slot_one_generation)] = generations {
            let both_written_by_mkfs = slot_zero_generation
                == SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS
                && slot_one_generation == SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS;
            judgements.judge(
                SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
                both_written_by_mkfs || slot_zero_generation.abs_diff(slot_one_generation) == 1,
                || {
                    format!(
                        "盘 {device} 两个系统配置槽的世代号 {slot_zero_generation} / {slot_one_generation}：相差不是 1，也不是 mkfs 写下的两个 1"
                    )
                },
            );
        }
    }
}

/// [`ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES`]（I-7.17）：根环里每条自证过的根，`checkpoint_txg` ≥ 1 时住
/// (区域 txg mod R, 槽 (txg div R) mod S)；第 0 代（mkfs 把它种进全部区域，D22（单元原子性怎么合成） 已定项 8）只判槽 0。
/// 公式是 D22（单元原子性怎么合成） 已定项 16 的定案。
pub fn judge_root_records_live_in_the_slot_their_txg_names(
    roots: &[(u64, u64, RootView)],
    geometry: &PoolGeometry,
    judgements: &mut Judgements,
) {
    for (region, slot, root) in roots {
        let txg = root.checkpoint_txg;
        let (expected_region, expected_slot) = if txg == GENESIS_ROOT_CHECKPOINT_TXG {
            (None, 0)
        } else {
            let region_of_the_txg = txg
                .checked_rem(geometry.regions)
                .expect("根环里读出了根 ⇒ `root_slot_positions` 至少排出一个区域，R ≥ 1");
            let slot_of_the_txg = (txg / geometry.regions)
                .checked_rem(geometry.slots_per_region)
                .expect("根环里读出了根 ⇒ 每个区域至少一个槽，S ≥ 1");
            (Some(region_of_the_txg), slot_of_the_txg)
        };
        let holds =
            expected_region.is_none_or(|expected| expected == *region) && expected_slot == *slot;
        judgements.judge(ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES, holds, || {
            format!(
                "实例 {}、txg {txg} 的根住在区域 {region} 槽 {slot}：按 txg 算该在区域 {expected_region:?}（第 0 代不判区域）槽 {expected_slot}",
                root.instance
            )
        });
    }
}

/// [`JOURNAL_RING_GEOMETRY_FIELDS_AGREE`]（I-8.1）的三个字段等式：每块盘每个可择的系统配置槽（[`SystemConfigurationSlotReading::Usable`]；
/// 带越界值的槽归 I-7.13 整池拒、不可择的槽 checker 不读它的几何），三格各判一次——
/// 槽里的 F 等于格式常量 `JOURNAL_SAFETY_FACTOR`（`.claude/kb/invariants.md` I-8.1 那一行「F = 格式常量，任一不等判红」，
/// 与实现择系统配置时同判，`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3 行）；
/// 在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋（F 取同一槽里的那个字段，F 为 0 时这一格判红、不除）；最坏占用 = 在飞上限 × 4096
/// （乘出来溢出判红）。「环长 ≥ F × 最坏占用」是这三格的推论，不另判（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行）。
/// 三格有一格不成立，实现整池拒绝挂载，`walk::check_pool_image` 照 I-7.13 那一支整池不作保。
/// 同一次发布的记录数不多于在飞上限那一格读 journal 环，在 `walk` 里判。
pub fn judge_journal_ring_geometry_fields_agree(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    for (device, slots) in readings {
        for (slot_index, reading) in (0u64..).zip(slots) {
            let geometry = match reading {
                SystemConfigurationSlotReading::Usable(_, geometry) => geometry,
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
                | SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => continue,
            };
            let safety_factor = geometry.declared_safety_factor;
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                safety_factor == JOURNAL_SAFETY_FACTOR,
                || {
                    format!(
                        "盘 {device} 系统配置槽 {slot_index} 的安全系数 F = {safety_factor}，不等于格式常量 {JOURNAL_SAFETY_FACTOR}"
                    )
                },
            );
            let in_flight_record_limit_from_the_ring =
                in_flight_record_limit_recomputed_from_the_ring(geometry);
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                in_flight_record_limit_from_the_ring
                    == Some(geometry.declared_in_flight_record_limit),
                || {
                    format!(
                        "盘 {device} 系统配置槽 {slot_index} 记的在飞上限 {}，按环长 {} ÷ 4096 ÷ F {safety_factor} 算是 {in_flight_record_limit_from_the_ring:?}",
                        geometry.declared_in_flight_record_limit, geometry.journal_ring_bytes
                    )
                },
            );
            let worst_case_occupancy_from_the_limit =
                worst_case_occupancy_recomputed_from_the_limit(geometry);
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                worst_case_occupancy_from_the_limit
                    == Some(geometry.declared_worst_case_occupancy_bytes),
                || {
                    format!(
                        "盘 {device} 系统配置槽 {slot_index} 记的最坏占用 {} 字节，按在飞上限 {} × 4096 算是 {worst_case_occupancy_from_the_limit:?}",
                        geometry.declared_worst_case_occupancy_bytes,
                        geometry.declared_in_flight_record_limit
                    )
                },
            );
        }
    }
}

/// I-7.5 的判定宽度：各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的那个（`io_min` 为 0 视为没声明，
/// 取 max 时自然落到 `physical_block_size`；异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条）。
/// checker 不探测设备，这个宽度一律是声明值（[`crate::DecisionWidth::DeclaredOnly`]，探测比对仍欠 C388（checker 的判定宽度不自己探测））。
/// 没有一块盘择得到系统配置时交 None。
#[must_use]
pub fn decision_width_declared_by_the_chosen_system_configurations(
    chosen_geometries: &[PoolGeometry],
) -> Option<crate::DecisionWidth> {
    chosen_geometries
        .iter()
        .map(|geometry| {
            geometry
                .physical_block_size
                .max(geometry.declared_minimum_input_output_bytes)
        })
        .max()
        .map(|bytes| crate::DecisionWidth::DeclaredOnly { bytes })
}

/// [`ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH`]（I-7.5）：根环 R × S 个槽逐个判，起始偏移（设备内字节）是判定宽度的整数倍。
/// 只判第 0 个不够：偏移半个槽的槽照样跨两个原子单元（E41（根环槽几何 2×4×256））。判定宽度 0 不会出现
/// （`geometry_of` 判过 `physical_block_size` ≥ 根记录宽），出现了这一格判红、不除。
pub fn judge_root_slots_aligned_to_the_decision_width(
    geometry: &PoolGeometry,
    decision_width: crate::DecisionWidth,
    judgements: &mut Judgements,
) {
    let width_in_bytes = u64::from(decision_width.bytes());
    for (region, slot, device, offset) in root_slot_positions(geometry) {
        judgements.judge(
            ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
            offset
                .checked_rem(width_in_bytes)
                .is_some_and(|remainder| remainder == 0),
            || {
                format!(
                    "根环区域 {region} 槽 {slot}（盘 {device} 偏移 {offset}）不是判定宽度 {decision_width:?}（声明值，未探测）的整数倍"
                )
            },
        );
    }
}

/// 一个系统配置槽是不是自证过（magic 与整槽校验和都对）：I-8.5（tail 槽有效集非空） 与旁路报告都按这个数。
const fn slot_verifies_itself(reading: &SystemConfigurationSlotReading) -> bool {
    match reading {
        SystemConfigurationSlotReading::Usable(..)
        | SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => true,
        SystemConfigurationSlotReading::NotSelfVerified => false,
    }
}

/// [`SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION`]（I-8.5，V-池）判一格：全池至少一块盘的两个系统配置槽里有一个自证过。
/// 对着只读恢复判（`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 3 条）：单盘全废只读恢复照样挂得上，不判违例，
/// 走旁路报告 [`system_configuration_redundancy_report`]。
pub fn judge_some_device_carries_a_self_verified_system_configuration(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    let some_slot_verifies_itself = readings
        .iter()
        .any(|(_, slots)| slots.iter().any(slot_verifies_itself));
    judgements.judge(
        SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
        some_slot_verifies_itself,
        || {
            format!(
                "池里 {} 块盘的系统配置槽一个都没自证过：tail 没有见证者",
                readings.len()
            )
        },
    );
}

/// 系统配置冗余的旁路报告：不进违例通道（I-8.5 对着只读恢复判，这两样只读恢复都挂得上）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationRedundancyReport {
    /// 两个槽都没自证过的盘（单盘全废）。
    pub devices_without_a_self_verified_slot: Vec<u32>,
    /// 少于两个槽自证过的盘：可写挂载读阶段读每块盘两槽，有一槽读不出或自证不过就判不出、按真，重读一次仍不成就拒可写
    /// （D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」）。单盘全废的盘也在这里。
    pub devices_whose_writable_mount_would_be_refused: Vec<u32>,
}

/// 逐盘数自证过的系统配置槽（magic 与整槽校验和都对），交回 [`SystemConfigurationRedundancyReport`]。槽的读法同
/// [`system_configuration_slot_readings`]。「本池自证不过」那一半（fsid 是别的池的槽）这里不分：数的是 I-8.5 那一行的「自证过」。
#[must_use]
pub fn system_configuration_redundancy_report(
    reader: &dyn ImageReader,
) -> SystemConfigurationRedundancyReport {
    let mut report = SystemConfigurationRedundancyReport {
        devices_without_a_self_verified_slot: Vec::new(),
        devices_whose_writable_mount_would_be_refused: Vec::new(),
    };
    for (device, slots) in system_configuration_slot_readings(reader) {
        let self_verified_slots = slots
            .iter()
            .filter(|reading| slot_verifies_itself(reading))
            .count();
        if self_verified_slots == 0 {
            report.devices_without_a_self_verified_slot.push(device);
        }
        if self_verified_slots < slots.len() {
            report
                .devices_whose_writable_mount_would_be_refused
                .push(device);
        }
    }
    report
}

/// 第 0 代根的 `checkpoint_txg`：mkfs 种在每个区域的槽 0（D22（单元原子性怎么合成） 已定项 8）。
const GENESIS_ROOT_CHECKPOINT_TXG: u64 = 0;

/// 根环全部槽里自证过的根（I-7.1 的集合 S），带它住的区域与槽号。
#[must_use]
pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {
    let slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    root_slot_positions(geometry)
        .into_iter()
        .filter_map(|(region, slot, device, offset)| {
            reader
                .read(device, offset, slot_bytes)
                .and_then(|bytes| check_root_slot(&bytes, &geometry.filesystem_identifier).ok())
                .map(|view| (region, slot, view))
        })
        .collect()
}

/// 位置条目：设备身份 4 + 槽号 6 + 整单元校验和 4。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerLocation {
    pub device: u32,
    pub slot: u64,
    pub checksum: u32,
}

fn location_at(bytes: &[u8], offset: usize) -> PointerLocation {
    PointerLocation {
        device: read_u32(bytes, offset),
        slot: read_six_byte_unsigned(bytes, offset + 4),
        checksum: read_u32(bytes, offset + 10),
    }
}

/// 指针的公共部分：头部 50（出生树在 34、出生 txg 在 42）+ 位置条目 14 × 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerView {
    pub birth_tree: u64,
    pub birth_txg: u64,
    pub locations: [PointerLocation; 2],
    /// 码 2 / 码 3 指针：实例代号 + 出生序号；码 1 指针：写序的实例代号 + 事务号。
    pub instance: u32,
    pub tail: u64,
    pub all_zero: bool,
    /// 头部 MAC 16（偏移 0）与 nonce 12（偏移 16）那 28 字节全是 0（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
    /// 加密关着时它们恒 0，读者遇到非 0 判该指针所在的结构损坏（已定项 3 射程）；池级走读在每条跟随的指针上按 I-2.4 判
    /// （[`judge_pointer_mac_and_nonce_are_zero`]）。
    pub is_mac_and_nonce_all_zero: bool,
    /// 头部 MAC / nonce 之后、extent 偏移之前的加密与压缩那几段（偏移 28..32）读到的原值。第一版的取值都是 0，池级走读在每条跟随的指针上
    /// 按 I-2.4 判（[`judge_pointer_encryption_and_compression_fields_take_the_first_version_values`]）。
    pub encryption_and_compression_fields: PointerHeadEncryptionAndCompressionFields,
    /// 头部 extent 偏移那 2 字节（偏移 32，D19（块指针的结构与宽度预算） 已定项 11 的偏移表）是 0：第一版指针指整个单元，恒 0，
    /// 读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（已定项 3；`m3-inv-a-r1` 判决第二节第 11 行）。
    pub is_extent_offset_zero: bool,
}

/// 指针头部 MAC 16 + nonce 12：从偏移 0 起连着的 28 字节（D19（块指针的结构与宽度预算） 已定项 11）。
const POINTER_MAC_AND_NONCE_BYTES: usize = 16 + 12;

/// 指针头部里 MAC / nonce 之后的加密与压缩那几段（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的字段表：算法类型 1 在偏移 28、
/// 压缩算法码 1 在 29、压后长度 2 在 30），读到的原值。与 core `PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`
/// 判的是同一组字段，checker 自己按偏移表读一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerHeadEncryptionAndCompressionFields {
    pub algorithm_type: u8,
    pub compression_algorithm_code: u8,
    pub compressed_length: u16,
}

/// 指针头部算法类型 1 字节的位置（MAC 16 + nonce 12 之后）。
const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;
/// 指针头部压缩算法码 1 字节的位置（算法类型之后）。
const POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET: usize = 29;
/// 指针头部压后长度 2 字节的位置（压缩算法码之后）。
const POINTER_COMPRESSED_LENGTH_OFFSET: usize = 30;

fn encryption_and_compression_fields_of(
    pointer_bytes: &[u8],
) -> PointerHeadEncryptionAndCompressionFields {
    PointerHeadEncryptionAndCompressionFields {
        algorithm_type: pointer_bytes[POINTER_ALGORITHM_TYPE_OFFSET],
        compression_algorithm_code: pointer_bytes[POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET],
        compressed_length: read_u16(pointer_bytes, POINTER_COMPRESSED_LENGTH_OFFSET),
    }
}

/// 指针头部 extent 偏移 2 字节的位置：MAC 16 + nonce 12 + 算法类型等 4 之后（D19（块指针的结构与宽度预算） 已定项 11）。
const POINTER_EXTENT_OFFSET_FIELD: std::ops::Range<usize> = 32..34;

fn extent_offset_is_zero(pointer_bytes: &[u8]) -> bool {
    pointer_bytes[POINTER_EXTENT_OFFSET_FIELD]
        .iter()
        .all(|byte| *byte == 0)
}

fn mac_and_nonce_are_all_zero(pointer_bytes: &[u8]) -> bool {
    pointer_bytes[..POINTER_MAC_AND_NONCE_BYTES]
        .iter()
        .all(|byte| *byte == 0)
}

/// 指向码 2 / 码 3 的指针 86：尾段 = 实例代号 4 + 出生序号 4。
#[must_use]
pub fn parse_node_pointer(bytes: &[u8]) -> PointerView {
    PointerView {
        birth_tree: read_u64(bytes, 34),
        birth_txg: read_u64(bytes, 42),
        locations: [location_at(bytes, 50), location_at(bytes, 64)],
        instance: read_u32(bytes, 78),
        tail: u64::from(read_u32(bytes, 82)),
        all_zero: bytes[..86].iter().all(|byte| *byte == 0),
        is_mac_and_nonce_all_zero: mac_and_nonce_are_all_zero(bytes),
        encryption_and_compression_fields: encryption_and_compression_fields_of(bytes),
        is_extent_offset_zero: extent_offset_is_zero(bytes),
    }
}

/// 指向码 1 的指针 88：尾段 = 写序（实例代号 4 + 事务号 6）。
#[must_use]
pub fn parse_data_pointer(bytes: &[u8]) -> PointerView {
    PointerView {
        birth_tree: read_u64(bytes, 34),
        birth_txg: read_u64(bytes, 42),
        locations: [location_at(bytes, 50), location_at(bytes, 64)],
        instance: read_u32(bytes, 78),
        tail: read_six_byte_unsigned(bytes, 82),
        all_zero: bytes[..88].iter().all(|byte| *byte == 0),
        is_mac_and_nonce_all_zero: mac_and_nonce_are_all_zero(bytes),
        encryption_and_compression_fields: encryption_and_compression_fields_of(bytes),
        is_extent_offset_zero: extent_offset_is_zero(bytes),
    }
}

/// I-2.4（头校验和覆盖范围） 射程里指针那一半（主 agent 2026-09-27 把射程从单元头扩到指针头部）：走读跟随的一条指针，头部 MAC 16 +
/// nonce 12 全是 0。这一版 checker 只收加密类型 0 的系统配置（`check_system_configuration_slot`），它判的池都是加密关着的，
/// 那 28 字节只能是 0（D19（块指针的结构与宽度预算） 已定项 3 射程；实现那一侧是 `crates/singlefs-core/src/pointer.rs` 的
/// `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`，checker 自己按偏移表判一份）。只判不断：指针指的单元照走。
pub fn judge_pointer_mac_and_nonce_are_zero(
    judgements: &mut Judgements,
    pointer: &PointerView,
    what: &str,
) {
    judgements.judge("I-2.4", pointer.is_mac_and_nonce_all_zero, || {
        format!("{what}：指针头部 MAC 16 / nonce 12 有非 0 字节，加密关着时它们恒 0")
    });
}

/// I-2.4（头校验和覆盖范围） 射程里指针那一半的加密与压缩那几段：走读跟随的一条指针，头部算法类型是 0（不加密）、压缩算法码是 0（不压缩）、
/// 压后长度是 0（码 0 时恒 0）。第一版不支持加密（D9（加密） 已定项 10），压缩算法码登记表第一版只有码 0（D9（加密） 已定项 14），
/// 读者遇到别的取值判该指针所在的结构损坏，与 MAC / nonce 同一处置（D19（块指针的结构与宽度预算） 已定项 3；实现那一侧是
/// `crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion` 的算法类型、压缩算法码、压后长度三个成员；
/// 判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 那一行的改法 F2）。三段各判一格，违例那一句写清是哪一段。只判不断。
pub fn judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
    judgements: &mut Judgements,
    pointer: &PointerView,
    what: &str,
) {
    let fields = pointer.encryption_and_compression_fields;
    judgements.judge("I-2.4", fields.algorithm_type == 0, || {
        format!(
            "{what}：指针头部算法类型是 {}，第一版不加密、这 1 字节恒 0",
            fields.algorithm_type
        )
    });
    judgements.judge("I-2.4", fields.compression_algorithm_code == 0, || {
        format!(
            "{what}：指针头部压缩算法码是 {}，第一版登记表只有码 0（不压缩）",
            fields.compression_algorithm_code
        )
    });
    judgements.judge("I-2.4", fields.compressed_length == 0, || {
        format!(
            "{what}：指针头部压后长度是 {}，压缩算法码 0 时这 2 字节恒 0",
            fields.compressed_length
        )
    });
}

/// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节（D22（单元原子性怎么合成） 已定项 7 的字段表：偏移 428、429、441，合计到 457）。
const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC: std::ops::Range<usize> = 428..457;

/// I-2.4（头校验和覆盖范围） 射程里根记录那一处：根环里每一条读得出、自证过的根（`valid_roots`，不按候选集滤），根槽偏移 428..457
/// （算法类型、nonce、MAC）全是 0。这一版 checker 只收加密类型 0 的系统配置，加密关着时它们恒 0、读者遇到非 0 判该结构损坏——
/// 与单元头那 29 字节、指针头部 MAC / nonce 由同一条规则守（`.claude/kb/invariants.md` I-2.4 那一行；判决
/// `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker，
/// 挂载透明不比）。实现今天不判这 29 字节（`crates/singlefs-core/src/root_record.rs` 的 `parse_slot` 跳过它们），checker 自己判一份。
/// 只判不断：这条根照样进候选集、照样走。
pub fn judge_root_record_algorithm_nonce_and_mac_are_zero(
    judgements: &mut Judgements,
    roots: &[(u64, u64, RootView)],
) {
    for (region, slot, root) in roots {
        let reserved = &root.record_bytes[ROOT_RECORD_ALGORITHM_NONCE_AND_MAC];
        judgements.judge("I-2.4", reserved.iter().all(|byte| *byte == 0), || {
            let first_non_zero = reserved
                .iter()
                .position(|byte| *byte != 0)
                .expect("判红时这 29 字节里有非 0 的");
            format!(
                "区域 {region} 槽 {slot} 的根（实例 {}、txg {}）：根槽偏移 {} 的字节是 {:#04x}，算法类型 / nonce / MAC 那 29 字节（428..457）加密关着时恒 0",
                root.instance,
                root.checkpoint_txg,
                ROOT_RECORD_ALGORITHM_NONCE_AND_MAC.start + first_non_zero,
                reserved[first_non_zero]
            )
        });
    }
}

/// I-2.4（头校验和覆盖范围） 射程里指针那一半的另一格：走读跟随的一条指针，头部 extent 偏移（偏移 32 的 2 字节）为 0。
/// 第一版指针指整个单元，读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关
/// （D19（块指针的结构与宽度预算） 已定项 3，`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。只判不断。
pub fn judge_pointer_extent_offset_is_zero(
    judgements: &mut Judgements,
    pointer: &PointerView,
    what: &str,
) {
    judgements.judge("I-2.4", pointer.is_extent_offset_zero, || {
        format!("{what}：指针头部 extent 偏移非 0，第一版指针指整个单元、这 2 字节恒 0")
    });
}

/// I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免。
pub fn judge_location_order(judgements: &mut Judgements, pointer: &PointerView, what: &str) {
    if pointer.all_zero {
        return;
    }
    judge_location_entries_order(judgements, &pointer.locations, what);
}

/// I-2.5 对一个位置条目数组判一次（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17）：
/// 设备身份严格升序，相同或逆序判损坏。不带「整条全零」的豁免——那一条射程只写给 86 字节全零的指针（「注册了但还没有根」），
/// journal 点名项点的是这次发布写出的单元，没有「还没有」的形态。
pub fn judge_location_entries_order(
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    what: &str,
) {
    judgements.judge("I-2.5", locations[0].device < locations[1].device, || {
        format!(
            "{what} 的位置条目设备身份 {} / {} 不是严格升序",
            locations[0].device, locations[1].device
        )
    });
}

/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。
pub fn read_referenced_unit(
    reader: &dyn ImageReader,
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    unit_bytes: usize,
    what: &str,
) -> Option<Vec<u8>> {
    read_referenced_unit_with_the_location_it_was_read_from(
        reader, judgements, locations, unit_bytes, what,
    )
    .map(|(bytes, _location_it_was_read_from)| bytes)
}

/// 同 [`read_referenced_unit`]，另交回读回的那一份（第一份对得上的）是哪一条位置条目：认「走过」要按读回、自证过的那一份的落点
/// （实例表链上第 k 片的指针，盘 0 那条位置条目指错、盘 1 那条完好时，按盘 0 那条认会把它认成别的片，攻方 H-b，
/// `research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）。
pub fn read_referenced_unit_with_the_location_it_was_read_from(
    reader: &dyn ImageReader,
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    unit_bytes: usize,
    what: &str,
) -> Option<(Vec<u8>, PointerLocation)> {
    let mut first_good = None;
    for location in locations {
        let bytes = reader.read(location.device, location.slot * SLOT_BYTES, unit_bytes);
        let matches = bytes
            .as_ref()
            .is_some_and(|content| crate::crc32_castagnoli_table(content) == location.checksum);
        judgements.judge("I-2.1", matches, || {
            format!(
                "{what} 在盘 {} 槽 {} 的那一份与位置条目里的校验和对不上",
                location.device, location.slot
            )
        });
        if matches && first_good.is_none() {
            first_good = bytes.map(|content| (content, *location));
        }
    }
    first_good
}
