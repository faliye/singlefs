//! 池级 checker 的输入与第一层判定：读盘的口子、每条不变量的四种判定、系统配置与根环（I-7 类）、指针与位置条目。
//! 与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5）：偏移在这里按字段表各写一份。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, JOURNAL_RECORD_BYTES,
    JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, ROOT_RECORD_BYTES, ROOT_RING_BASE_SLOT,
    ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS,
    ROOT_RING_SLOTS_PER_REGION_MAXIMUM, ROOT_RING_SLOTS_PER_REGION_MINIMUM, SLOT_BYTES,
    SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
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
    /// 是什么（例如 I-9.28 那几条根：盘号、txg、为什么没判）。**它不是红，也不是成立**：计红、计违例的地方算不红（崩溃点重放与崩溃放量
    /// 把它算成不红），计「成立」的地方不算成立。今天 I-9.28 报它（树表读不出、条目比字段表短的根），I-5.5 报它（某一版树表读不出、
    /// 条目窄、extent 树或分配记录树读不全、种类 1 / 3 条数不对，`research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3）。
    PartiallyJudged(String),
}

/// 「中央映射条目的 key 与它指的单元头相符」那一条（用户 2026-09-27 定新立，实审 B2），
/// 编号 I-1.11（映射 key 与单元头相符），登记在 `crates/singlefs-checker/invariants/01-块自描述.md`。
pub const MAPPING_KEY_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";

/// 「自证过的系统配置槽里，池级字段都在这一版读者收的范围里」那一条（实审 A3-checker-2，用户 2026-09-27 定系统配置越界整池拒）：
/// 格式版本 = 1、加密类型 = 0（关）、固定结构槽距、`physical_block_size`、journal 环长在各自的上下界里（[`crate::Verdict`] 的
/// `FormatVersionNotRecognized` / `EncryptionTypeNotOff` / `FixedStructureSlotSpacingOutsideTheFormatRange` /
/// `PhysicalBlockSizeOutsideTheRootSlotBounds` / `JournalRingBytesOutsideTheSupportedRange`）；journal 环起点（偏移 325）与根环起点
/// （偏移 371）是第一版的格式常量、单元区起始槽号（偏移 417）是环长现算的那个槽且落在聚簇段边界上
/// （`JournalRingStartSlotNotTheFirstVersionConstant` / `RootRingBaseSlotNotTheFirstVersionConstant` /
/// `UnitAreaStartNotTheSlotAfterTheJournalRing` / `UnitAreaStartOffTheClusterSegmentBoundary`；实审 Y4-a，用户 2026-09-27 定
/// 「读字段，不等于常量就整池拒」，与实现收同一张表）；区域数 R（偏移 361）与第一版常量 3 不等、每区槽数 S（偏移 362）越界同样归这一条
/// （`RegionCountZero` / `RegionCountBelowTheFirstVersionConstant` / `RegionCountPastTheRegionDeviceFields` /
/// `RootRingSlotsPerRegionOutsideTheFormatInterval`；用户 2026-10-20 定 C663）；根环区域步长 P（偏移 363）、chunk（偏移 367）与第一版常量不等、
/// S × 槽距大于 P × chunk 同样归这一条（`RootRingPrimeStepNotTheFirstVersionConstant` / `RootRingChunkBytesNotTheFirstVersionConstant` /
/// `RootRingRegionLongerThanTheRegionStride`；D22（单元原子性怎么合成） 已定项 16 第 1 句，用户 2026-10-22 定）。
/// **编号是暂立的，待 kb 第六批登记时定**；改号只改这一处。
pub const SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS: &str = "I-7.13";

/// 「每块盘每个自证过的系统配置槽里的本盘设备号，等于这块盘在池里的身份」那一条（实审 Y2-b 越格线索：盘体对调之后的池
/// 可写挂载因本盘设备号拒，而 checker 读本盘设备号却没有一条不变量拿它比盘的身份，0 违例）。实现那一侧是
/// `crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`（`OwnDeviceNumberDiffersFromTheIdentityHandedIn`），
/// checker 自己按字段表读一份（[`judge_own_device_number_is_the_device_identity`]）。编号 I-7.14，登记在 `crates/singlefs-checker/invariants/02-根环.md`；
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
/// `research/prompts/m3-inv-b-r1-main-verification.md`，用户 2026-10-01 定案），**只往末尾追加**：崩溃点重放的违例位按下标 i ↔ 这张表第 i 条，
/// 前 49 条的下标不动。位宽改到 ≥ 128 之前，下标 64 起的那一条在崩溃点重放的 GPU 判器上没有位（另一个实现员在 checker 档改）。
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
/// （下标 69–72，删除态扫描，第 18 批 P2），下标 73 的 I-4.4 第一版恒报不适用；里程碑四改判 logged_ops 条目之后 GPU 判器的引用与快照那一截照判、
/// 置位（`crash_judge_gpu` 的覆盖表）。
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
/// （墓碑旋钮同判 I-9.22）；崩溃点重放与放量走产品那一份（三棵都没装），I-9.22、I-9.37 在那里照旧不置位。
///
/// 第 93 条（下标 92）是只读扫描重建那一条（里程碑四四·2，题面 `.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」）：
/// I-1.13（扫描重建与遍历方向的索引相同），同样只往末尾追加。判在 `walk::check_pool_image` 的末尾，扫描重建本身在 `rebuild` 模块。
/// GPU 判器的覆盖表（`crash_judge_gpu` 的 `POOL_CHECKER_COVERED_INVARIANT_INDEXES`）收了下标 92：起初内核不置位、对拍不比（用户 2026-10-13 定
/// 「先进覆盖表但内核不置位，记欠账」，C821），用户 2026-10-18 定「补 GPU 扫描重建内核」之后扫描内核源码里的扫描重建入口照判、置位，
/// 结局码并进对拍字，GPU 与 CPU 逐位比。
///
/// 第 94、95 条（下标 93、94）是里程碑四只读快照 checker 侧那两条（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）：
/// I-3.6（deadlist 紧界）与 I-3.7（活快照可枚举），同样只往末尾追加；I-3.5 ③ ④ 与逆向并进下标 65 那一条。
/// 判在 `walk::judge_snapshots_and_deadlists_of_the_versions`（与删除态扫描同一份版本清单，每一版重读一遍树表）。
/// GPU 判器起初不加判法（两条进覆盖表、内核不置位、对拍不比）；之后扫描内核源码里的引用与快照那一截照判、置位，对拍逐位比。
///
/// 第 96–99 条（下标 95–98）是里程碑四自建 GC 的 checker 新判那四条（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3
/// 「新判：N2 反查、N3、N5、N6」与 G9「已死」类，正推 `research/prompts/m4-gc-r1-sonnet-output.md`「checker 多判哪几条」那张表）：
/// 下标 95 的 I-3.15（N2）、96 的 I-5.6（N3）、97 的 I-3.16（N5，C413）、98 的 I-1.14（N6），同样只往末尾追加；N4 是下标 73 那一条 I-4.4
/// 本身（logged_ops 有了字段表，不再恒报不适用）。判在 `walk::check_pool_image` 的末尾那几遍（与删除态扫描同一份版本清单，每一版重读自己的树表、
/// 分配记录与中央映射）。GPU 判器起初不加判法（四条进覆盖表、内核不置位、对拍不比）；之后扫描内核主入口与引用与快照那一截照判这四条与 I-4.4、
/// 置位，对拍逐位比。
///
/// 第 100、101 条（下标 99、100）是截断那一批的两条新判（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 的 T-2、T-4）：
/// 下标 99 的 I-4.10（截断中间态有账）与下标 100 的 I-9.38（数据单元声明长度对 size），同样只往末尾追加；判在删除态扫描里每一版各判
/// （照 I-9.5 的共用判法）。同一批改了三条已有的判法：I-4.4 收截断条目（T-1）、I-4.6 每条记录都判单元数对 size 与标签（T-3）、
/// I-1.13 两侧按 size 滤（T-5）。GPU 判器的扫描内核在删除态扫描里同判这两条、置位，覆盖表收了下标 99、100，对拍逐位比（判决第四节「截戊」那一批补的）。
///
/// 第 102 条（下标 101）是截断字段那一批的新判（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6，方向照第四节第 2 条改写）：
/// I-9.39（截断字段之外的活单元不早于它），同样只往末尾追加；判在删除态扫描里每一版各判（照 I-9.5 的共用判法）。同一批改了 I-9.7 的判法
/// （偏移 108–119、136–139 恒 0，120 起的截断字段按 incompat 位 2 判）、系统配置那一判收 incompat 位 2、扫描重建加读法 Z 第四条（R3）。
/// GPU 判器的扫描内核在删除态扫描里同判、置位，覆盖表收了下标 101，对拍逐位比（判决第四节、派发「截壬」那一批补的）。
///
/// 第 103 条（下标 102）是 extent 上段规范形那一条（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：
/// I-9.40（extent 上段形状由有条目的 inode 号集合定），同样只往末尾追加；判在删除态扫描里每一版各判（与 I-9.5 那几条同一份版本清单，
/// 每一版自己的树表与 extent 树另读一遍）。GPU 判器的扫描内核在删除态扫描里同判、置位，覆盖表收了下标 102，对拍逐位比
/// （`crash_judge_conclusions` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表：GPU 内核今天没有不置位的不变量）。
///
/// 第 104 条（下标 103）是隔离记号那一条（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，用户 2026-10-22 定落点甲、checker 只认记号）：
/// I-3.17（隔离记号与引用相符），同样只往末尾追加；判在 `walk::check_pool_image` 里最新根那棵分配记录树上（`judge_quarantine_marks_against_the_references`）。
/// 同一批把 I-3.1、I-3.11、I-3.15 的隔离豁免改成只认记号。GPU 判器的扫描内核同判、置位，覆盖表收了下标 103，对拍逐位比。
///
/// 第 105、106 条（下标 104、105）是钉也落盘那一批（C773（读坏与全零根槽的回收门槛钉只在内存里），根环三方第二、三轮判决与用户 2026-10-20 定案）：
/// I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根），同样只往末尾追加；只判最新根、只判建账的发布
/// （[`ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD`]）。GPU 判器的走读内核同判、置位。
///
/// 第 107、108 条（下标 106、107）是里程碑三收尾补的两条（`.claude/kb/checks-owed.md` 的 C868（系统配置管道身份没有 checker 判定）、
/// C411（分配记录的跨度段与单元类别不符没检查））：I-1.5（系统配置记管道身份）判在择系统配置那一段（逐盘），I-3.18（分配记录的跨度等于它罩住的单元
/// 那一类的槽数）判在 `walk::check_pool_image` 末尾那几遍（与 I-3.15 同一份版本清单），同样只往末尾追加。GPU 判器同判、置位：走读内核择系统配置时
/// 逐盘判 I-1.5，引用与快照那一截判 I-3.18，`crash_judge_gpu` 的覆盖表收了下标 106、107，对拍逐位比。
///
/// 第 109 条（下标 108）是同盘冲突的 journal 记录布局那一条（`.claude/kb/checks-owed.md` 的 C897（同盘冲突的 journal 记录布局没有判据））：
/// I-8.13（[`JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES`]），同样只往末尾追加；判在 `walk::check_pool_image` 扫 journal 环那一段
/// （`judge_journal_records_live_in_the_ring_slot_their_counter_names`）。GPU 判器的那一位由 checker 档按这个下标补（`crash_judge_gpu` 的覆盖表）。
pub const IMPLEMENTED_INVARIANTS: [&str; 109] = [
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
    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
    LIVE_SNAPSHOTS_ARE_ENUMERABLE,
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
    FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
    EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
    QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
    ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
    PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
    SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
    ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
    JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
];

/// I-8.13（记录落在它的计数器该落的环槽，C897（同盘冲突的 journal 记录布局没有判据））：逐盘，每条自证过（池级 checker 的记录读者收得下、
/// fsid 是本池的）的 journal 记录坐在环槽 `(计数器 − 1) mod 槽数`（D23（journal 的角色与格式） 已定项 18 的落点式），不在判红；
/// 同盘同 (实例代号, 计数器) 两条坐两槽时至少一条不在该落的槽上，连带判红。计数器 0 不判（归 I-8.6）。
/// 登记在 `crates/singlefs-checker/invariants/03-journal.md`。
pub const JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES: &str = "I-8.13";
/// I-8.13 一条都没判到时的理由。
pub const NO_SELF_VERIFIED_JOURNAL_RECORD_WITH_A_COUNTER_FROM_ONE: &str =
    "每块盘的 journal 环里都没有计数器从 1 起的自证过的记录";

/// I-1.5（系统配置记管道身份，C868（系统配置管道身份没有 checker 判定））：每块盘择到的系统配置（自证过、世代号最大的那一槽）自举头里
/// 「写入者身份 20」的实现标识 16 字节不全零（D22（单元原子性怎么合成） 已定项 9；core 写 `singlefs-rs`）。逐盘判一次。
/// 登记在 `crates/singlefs-checker/invariants/01-块自描述.md`。
pub const SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY: &str = "I-1.5";

/// I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，C411（分配记录的跨度段与单元类别不符没检查））：回退候选集里每条根与由记录施加出来的
/// 那几版，各自的分配记录树里每条未释放的记录，起点槽上的单元头用得了（头校验和对、类标签是码 1 / 2 / 3）时，跨度段 × 16384 等于那一类的
/// 单元大小（D18（块里携带什么信息） 已定项 11 的登记表：码 1 与码 3 是 32768 两槽，码 2 是 16384 一槽）。几版照抄同一棵分配记录树的只判一次。
/// 已释放的记录不判（落点可以已被别的类的单元合法复用），头用不了的没有对象（读不出归 I-1.1 与 I-2.1）。
/// 登记在 `crates/singlefs-checker/invariants/05-空间记账.md`。
pub const ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS: &str =
    "I-3.18";
/// I-3.18 一条都没判到时的理由。
pub const NO_UNRELEASED_ALLOCATION_RECORD_COVERING_A_USABLE_UNIT_HEADER: &str =
    "回退候选集里没有一版的分配记录树走得到未释放、起点槽上单元头用得了的记录";

/// I-3.17（隔离记号与引用相符，D19（块指针的结构与宽度预算） 已定项 5「新加判据『记号与引用相符』」）：最新根的分配记录树里，
/// ① 带隔离记号（跨度段 0x4000）的记录未释放；② 带记号的记录罩住的槽不被最新根引用；③ 同一单元各盘那条记录的记号要么都有、要么都没有。
/// 任一句不成立判红。登记在 `crates/singlefs-checker/invariants/05-空间记账.md`。
pub const QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES: &str = "I-3.17";
/// I-3.17 没有对象时报的理由（`walk::judge_quarantine_marks_against_the_references`）。
pub const NO_RECORD_CARRYING_THE_QUARANTINE_MARK_UNDER_THE_NEWEST_ROOT: &str =
    "最新根的分配记录树里没有带隔离记号的记录";

/// I-7.18（根记录的回收门槛等于现算的门槛，`crates/singlefs-checker/invariants/02-根环.md`）：最新根的账是它自己那次发布建的（树表里记账树那一条的
/// 根指针出生 txg 等于这条根的 txg）时，根记录里的回收门槛 t = max(这条根自己带的 F, min(环里按最新根的实例表仍有效的根的最小 txg, 钉值 p))
/// （p 是 255 时第二项只取前者，环里一条有效根都没有时取 F）。不是建账的发布（零单元发布、照抄上一版账的根）、最新根下面没有记账树时报不适用。
pub const ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD: &str = "I-7.18";
/// I-7.19（钉着的槽不住本实例写的根）：建账的最新根带的钉槽 p 不是 255 时，p 小于环长 3S、还原出的槽（区域 p mod 3、槽 p div 3）
/// 里没有一条实例代号等于最新根的自证过的根。适用条件同 I-7.18。
pub const PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE: &str = "I-7.19";

/// I-4.10（截断中间态有账，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2）：每条 inode 记录，存在的数据单元多于
/// ⌈size ÷ 净荷⌉.max(1)（size 之外还挂着单元）时，同一版 logged_ops 里有一条对得上的截断条目（inode 号、对象出生代相同、目标 = 记录的 size）。
/// 判哪几版照 I-9.5 的共用判法。登记在 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md`。
pub const DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT: &str = "I-4.10";
/// I-9.38（数据单元声明长度对 size，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-4）：每个存在而在 size 之内的数据单元，
/// 头里的声明长度 = min(size − 起点, 净荷容量)。判哪几版照 I-9.5 的共用判法。登记在 `crates/singlefs-checker/invariants/09-inode树结构.md`。
pub const DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE: &str = "I-9.38";
/// I-4.6、I-4.10、I-9.39 一次都没判到时的理由：走读到的每一版 inode 树里一条记录都没有。
pub const NO_INODE_RECORD_IN_ANY_WALKED_VERSION: &str = "走读到的每一版 inode 树里都没有记录";
/// I-9.38 一次都没判到时的理由：走读到的每一版里没有一个存在而在记录 size 之内的数据单元。
pub const NO_DATA_UNIT_WITHIN_THE_SIZE_OF_A_RECORD_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版里都没有存在而在它那条记录 size 之内的数据单元";
/// I-9.39（截断字段之外的活单元不早于它，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）：截断字段 T ≠ 0 的每条
/// inode 记录，extent 树里存在、在 size 之内、单元号不小于 ⌈N_w ÷ 净荷⌉.max(1) 的数据单元，单元头诞生代号都不早于 T。
/// 每条记录判一次（T = 0 的记录没有对象、照判成立，同 I-4.10），一条记录都没有时报 [`NO_INODE_RECORD_IN_ANY_WALKED_VERSION`]。
/// 判哪几版照 I-9.5 的共用判法。登记在 `crates/singlefs-checker/invariants/09-inode树结构.md`。
pub const LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT: &str = "I-9.39";
/// I-9.40（extent 上段形状由有条目的 inode 号集合定，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：
/// 每一版 extent 树上段的节点位置集合，等于由上段叶条目的 inode 号集合 S 定的那一个——根层级是罩得住 max(S) 的最低一层，节点是 S 里每个号
/// 从它那片叶到根那一串的并；S 空时是层级 0 第 0 格一个零条目根兼叶。判哪几版照 I-9.5 的共用判法。登记在
/// `crates/singlefs-checker/invariants/09-inode树结构.md`。
pub const EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY: &str = "I-9.40";

/// I-3.15（未释放记录罩住的单元由映射指在这一槽，N2 反查；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「新判：N2 反查」
/// 与 G9「已死」类）：回退候选集里每条根与由记录施加出来的那几版，各自的分配记录树里每条未释放的记录，罩住的那一槽上单元头里的出生身份
/// （类标签、诞生代号、写序实例、尾段，码 1 / 码 3 再加出生树）在同一版的中央映射里有一把 key 相符、且那把 key 的位置条目指着这一槽；
/// 豁免三类（映射树各节点、树表、实例表各片）与根记录直接持有的那几样，改成要被根记录或映射树、实例表链里的父指针直接指着。
/// 隔离的单元（这一版这一单元每块盘上的未释放记录都带隔离记号，D19（块指针的结构与宽度预算） 已定项 5「池级 checker 只认记号」，
/// G9 单列的「隔离」类）不判；单元头用不了的那一格没有对象（读不出本身归 I-1.1 与 I-2.1）。抓「只改映射不写副本」「只写副本不改映射」「有记录没人引用」三形。
/// 登记在 `crates/singlefs-checker/invariants/05-空间记账.md`。
pub const UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE: &str =
    "I-3.15";
/// I-5.6（活整理意图的区域里没有签发之后分配的记录，N3；判决同上第三节 G1、G3，D26（后台整理与放置回收） 已定项 1 第 3 条的 R 排他）：
/// 每一版 logged_ops 树里的每条整理意图，同一版分配记录树里罩进它的区域 R（两盘同槽时 R 罩住每块盘上的同号槽）的未释放记录，
/// 分配代都不大于意图里的 `c0`。一版里一条整理意图都没有时没有对象。登记在 `crates/singlefs-checker/invariants/07-分配器.md`。
pub const COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED: &str = "I-5.6";
/// I-3.16（碎片度与全空段统计对得上，N5 = C413（碎片度与全空段统计没有不变量））：最新根记账里每块盘的 runs（第 10 项，单元区空闲槽的连续段数）
/// 与全空聚簇段数（第 11 项，段内 64 槽都没有未释放分配记录的整段数，D3（空间分配） 已定项 10 ①、C372 按记录口径收口）等于 checker 全扫重算的值；
/// 重算不与增量维护共享代码。占着的槽取 I-3.1 那一份并集（全部走过的版本的引用、隔离豁免、F 之下仍在 defer 的），那块盘上 I-3.1 的等式不成立时
/// 这块盘不判。全空段那一半从最新根的分配记录现扫，defer 窗口里的已释放槽不算占用，两种口径分得开的盘上照判（分配器写记账行的也是记录口径）。
/// 登记在 `crates/singlefs-checker/invariants/05-空间记账.md`。
pub const FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN: &str = "I-3.16";
/// I-1.14（同一映射 key 在各版里的单元逐字节相同，N6；判决同上第三节 G3、G7「执行层靠 N6」）：回退候选集里任两版（连同由记录施加出来的那几版）
/// 的中央映射里同一把 key，各自的位置条目指着的那一份单元（按位置条目带的整单元校验和对得上的第一份）逐字节相同。两版的位置条目相同的不读盘；
/// 一份都读不出的那一对不比（读不出归 I-2.1）。登记在 `crates/singlefs-checker/invariants/01-块自描述.md`。
pub const UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS: &str = "I-1.14";
/// I-5.6 没有对象时的理由：走读到的每一版 logged_ops 树里都没有 R 对齐、在单元区内的整理意图。
pub const NO_COMPACTION_INTENT_WITH_A_WELL_FORMED_REGION_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版 logged_ops 树里都没有 R 对齐、在单元区内的整理意图：没有 R 可判";
/// I-1.14 没有对象时的理由：没有一把中央映射 key 在两版里落在不同的位置条目上（或那几份都读不出）。
pub const NO_MAPPING_KEY_PLACED_DIFFERENTLY_IN_TWO_WALKED_VERSIONS: &str =
    "回退候选集里没有一把中央映射 key 在两版里落在不同的位置条目上（或那几份都读不出）：比不出来";

/// I-1.13（扫描重建与遍历方向的索引相同）：只从扫描方向扫出的单元头（加当前版本那几片码 3 容器的记录）重建的三张派生表——中央映射、
/// 每个 inode 的数据单元集合、inode 记录集合——与遍历方向从最新根读出的三张表逐项相同（`.claude/kb/milestone/04-fourth-txn.md`
/// 「四·1 题面」；D21（权威态与派生态的分界） 已定项 13，D13（验证路线） 已定项 12）。违例写第一处差异：表名、key、两边的值；
/// 最新根读不出、实例表读不全、遍历方向的索引已被别的不变量判损坏与扫描重建停在条款没写之处的报不适用并写理由。登记在 `crates/singlefs-checker/invariants/01-块自描述.md`。
pub const SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX: &str = "I-1.13";

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
/// I-9.25：树表条目的 `previous_snapshot_txg`：不是 inode 树、也不是码 14（快照 inode 根）的条目恒 0；inode 树那条不大于引用这张树表的根的
/// checkpoint_txg；表里有码 14 时按前驱链判（码 14 按诞生 txg 排，第一条 0、之后是前一条的诞生 txg，inode 树那条是最后一条的诞生 txg）。
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
/// I-8.5（V-池）：全池至少一块盘有一个自证过的系统配置槽，全池皆废判红。单盘全废、哪块盘少于两槽自证过（带读故障时可能判不出而拒；
/// 没有读故障时挂得上，取号写两轮补回那一槽）不进违例通道，走旁路报告 [`system_configuration_redundancy_report`]。
pub const SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION: &str = "I-8.5";
/// I-3.5（引用区间的精确性）的出现规则：回退候选集按 txg 升序、同 txg 并成一条，相邻两版 (R⁻, R)，R 引用而 R⁻ 不引用的每个单元
/// 诞生代号大于 R⁻.txg，或落在 R 的一棵整棵继承下来的用户可见树里（按树继承下界）；候选集有洞时按实例表行判接不接得上，
/// 接不上的那一对只判用户可见树之外的单元（`research/prompts/m3-birth-floor-r3-main-verification.md`，用户 2026-10-03 定丙）。
/// ③ ④（deadlist）连同逆向（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）并在同一条下：每一版
/// （回退候选集里每条根与由记录施加出来的那几版）上，活快照引用而活头不引用的单元在 deadlist 里恰好一条、纪元落在它该在的那一侧；
/// 活头引用的单元不在 deadlist 里；deadlist 里每一条都被一个活快照引用或在待放集里（`walk::judge_snapshots_and_deadlists_of_the_versions`）。
pub const REFERENCE_INTERVALS_ARE_EXACT: &str = "I-3.5";
/// I-3.6（deadlist 紧界）：每一版的 deadlist 里每一条，birth ≤ 它归属那一侧（活快照或活头，按纪元落在哪一段认）的 `previous_snapshot_txg`；
/// 落在未完成销毁意图 [lo, hi) 里、birth > P 的待放条目豁免（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ②）。
/// 一条条目都没有的一版报成立（同 I-9.24 空队列报成立）。登记在 `crates/singlefs-checker/invariants/05-空间记账.md`。
pub const DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE: &str =
    "I-3.6";
/// I-3.7（活快照可枚举）：每一版树表里不带销毁中记号的码 14 条目就是这一版的全部活快照——每一条都是这一版之前的一次捕获
/// （诞生 txg 不大于这一版的 txg、两条根指针的出生 txg 不大于诞生 txg），回退候选集里 txg 相接的两版之间，活快照只经建快照那次发布多出来
/// （诞生 txg 就是后一版的 txg）、只经销毁意图与最后一批少下去（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S4）。
/// 前驱链那一份来源由 I-9.25 判，这一条不重判。登记在 `crates/singlefs-checker/invariants/05-空间记账.md`。
pub const LIVE_SNAPSHOTS_ARE_ENUMERABLE: &str = "I-3.7";
/// I-9.27：走读到的任一版树表里，条目偏移 116 的头 ID 按种类取值——inode 树（种类 2）等于它自己的树 ID；extent 树（1）、`deleted_inodes`（9）、
/// 墓碑树（10）、退役树（13）、快照 inode 根（14）等于这张树表里某一条种类 2 条目的树 ID（没有种类 2 条目时判红）；快照 extent 根（15）等于
/// 这张树表里某一条种类 14 条目的树 ID；种类 3、4、6、7、8、11、12 等于 0；别的种类码不判。
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
/// I-4.6（中间态其余不变量成立） 取乙（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`），截断那一批合并 C711
/// （判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-3 与第六节 S1）：每条记录（nlink = 0 与 ≥ 1 都判），存在的数据单元从 0 起
/// 连号；上段叶条目标签 1、2 的单元数等于 ⌈size ÷ 净荷⌉.max(1)、不许有洞，有一条对得上的截断条目时 size 之外可以多；标签 3、4 至少一个洞单元；
/// 标签 0（与没有上段叶条目的号）一个单元都没有。名字是这一条起初的射程（只判删除中的 inode），C711 并进来之后照判全部记录、名字没改：
/// checker 档与 harness 档有几处按这个名字取判定。
pub const DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE: &str = "I-4.6";
/// I-4.4（意图只描述目标态，N4；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1「I-4.4 的判法」）：每一版 logged_ops 树里的
/// 每条条目，种类码是登记过的（1 = 整理、2 = 截断）。整理条目：value 只装字段表里的那几段（白名单：R 的设备、起始槽、槽数、`c0`、`segs0`）、
/// 末尾补零 6 字节全 0、R 在单元区内且对齐（起始槽 64 对齐、槽数 = 64、设备是池里的盘）、`c0` 与签发 txg 都不晚于这一版的 txg。
/// 截断条目（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-1）：value 只装 inode 号、对象出生代、目标 size，末尾补零 10 字节全 0、
/// 签发 txg 不晚于这一版、点名的 inode 在同一版存在且 nlink ≥ 1、对象出生代相等、目标 = 记录的 size；有截断条目的那一版至多一条。
/// 一版里一条条目都没有时没有对象，理由见 [`NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION`]（别的意图载体——nlink = 0 的记录、
/// `deleted_inodes` 的 key、墓碑——没有可装增量的字段，用户 2026-10-05 定）。
pub const INTENT_DESCRIBES_ONLY_THE_TARGET_STATE: &str = "I-4.4";
/// I-4.4 没有对象时的理由：走读到的每一版 logged_ops 树里一条意图条目都没有，别的意图载体没有可装增量的字段。
pub const NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION: &str = "走读到的每一版 logged_ops 树里都没有意图条目；别的意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段";
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
/// 每片至少一条（第 56 批 G2）、每片记录数 ≤ 旋钮值、最右那片之外每片记录数 == 旋钮值（第 57 批 H1）；零根没有对象。
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
/// 相邻两片容器号加一、每片至少一条（第 56 批 G2）、每片记录数 ≤ 旋钮值、最右那片之外每片记录数 == 旋钮值（第 57 批 H1）；零根没有对象。
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
    /// 判红时点名的每一个（不变量, 盘, 槽），收全（[`Judgements::judge_naming_the_slots`]）；只有点名槽的那几判往里记。
    violation_sites: BTreeSet<ViolationSite>,
}

/// 一处违例点名的（不变量, 盘, 起点槽）：故障归类器按它比「被吞的写插回去之后仍红」（判决 `research/prompts/m4-i315-r3-main-verification.md`
/// 第三节 C3）。槽是那条分配记录的起点槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViolationSite {
    pub invariant: &'static str,
    pub device: u32,
    pub slot: u64,
}

/// 一张镜像的判定连同判红时点名的（不变量, 盘, 槽）（[`Judgements::into_report_naming_the_violation_sites`]）。
/// 点名槽的只有 I-3.1、I-3.11（已分配、没有根引用、没被隔离豁免的单元）、I-3.10 与 I-3.15（判红的那条记录）；
/// 别的不变量判红时这里没有它的行，要比就按名比。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerdictsNamingTheViolationSites {
    pub verdicts: Vec<(&'static str, InvariantVerdict)>,
    pub violation_sites: BTreeSet<ViolationSite>,
}

impl Judgements {
    /// 评估一次：条件不成立就记下这一处的说明（说明只留第一处；要收全违例点名的槽用 [`Judgements::judge_naming_the_slots`]）。
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
    /// 同 [`Judgements::judge`]，条件不成立时另把 `slots` 交回的每一个（盘, 起点槽）记进违例点名表（全记，不只留第一处）。
    pub fn judge_naming_the_slots(
        &mut self,
        invariant: &'static str,
        holds: bool,
        slots: impl FnOnce() -> Vec<(u32, u64)>,
        detail: impl FnOnce() -> String,
    ) {
        self.judge(invariant, holds, detail);
        if !holds {
            self.violation_sites
                .extend(slots().into_iter().map(|(device, slot)| ViolationSite {
                    invariant,
                    device,
                    slot,
                }));
        }
    }

    /// 同 [`Judgements::into_report`]，另交回判红时点名的（不变量, 盘, 槽）。
    #[must_use]
    pub fn into_report_naming_the_violation_sites(mut self) -> VerdictsNamingTheViolationSites {
        let violation_sites = std::mem::take(&mut self.violation_sites);
        VerdictsNamingTheViolationSites {
            verdicts: self.into_report(),
            violation_sites,
        }
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
    /// 槽里自述的区域步长 P（偏移 363）与 chunk（偏移 367）：[`geometry_of`] 判过两样都等于第一版常量才交出 [`PoolGeometry`]。
    /// 放区域不读这两个字段，按格式常量放（[`root_slot_positions`]，与实现的 `root_ring::region_start` 一致）。
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
/// R 是 0 ⇒ [`crate::Verdict::RegionCountZero`]；R 是 1 或 2 ⇒ [`crate::Verdict::RegionCountBelowTheFirstVersionConstant`]
/// （R 与第一版常量 `ROOT_RING_REGIONS` 不等，用户 2026-10-20 定 C663「R 读并整池拒」）。
///
/// S（偏移 362 那一字节）不在格式承诺的区间里 ⇒
/// [`crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval`]，同一条理由：
/// 夹一下往下走就是按一个不存在的几何走读。两条判据都在盘上那一字节进 [`PoolGeometry`] 的唯一入口上。
///
/// 固定结构槽距、`physical_block_size`、journal 环长同样是盘上读来的数，同一个入口上判
/// （`fixed_structure_slot_spacing_lies_in_the_format_range`、`physical_block_size_fits_a_root_slot`、
/// `journal_ring_bytes_lie_in_the_supported_range`），越界各报一个成员。R、S 那四个成员与这三个处置相同：池级读法都认成
/// 「带这一版读者不收的值」（[`value_refused_by_this_reader`]），报违例、整池不作保（用户 2026-09-27 定系统配置越界整池拒；
/// R、S 两样用户 2026-10-20 定 C663「S 照条款整池拒，R 读并整池拒」，S 那一半照 D22（单元原子性怎么合成） 已定项 2
/// 「越界整池拒绝挂载，与每区槽数 S 越界同一个处置」）。槽距先判：它是根槽宽的上界。
///
/// 根环区域步长 P（363）、chunk（367）不等于第一版的格式常量 ⇒ [`crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant`] /
/// [`crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant`]；两样都等而 S × 槽距大于 P × chunk ⇒
/// [`crate::Verdict::RootRingRegionLongerThanTheRegionStride`]（D22（单元原子性怎么合成） 已定项 16 第 1 句，用户 2026-10-22 定）。
/// 这三判排在根槽宽之后、环长之前，与实现的 `system_configuration_values_this_reader_accepts` 同在 R 之后、环长之前；
/// 区域按常量放之后根槽偏移装得进 u64（基槽等于 64、R 等于 3、S ≤ 16、槽距是 4 字节），不另判溢出。
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
    if regions == 0 {
        return Err(crate::Verdict::RegionCountZero);
    }
    if regions != ROOT_RING_REGIONS {
        return Err(crate::Verdict::RegionCountBelowTheFirstVersionConstant);
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
    let prime_step = u64::from(read_u32(slot, 363));
    let chunk_bytes = u64::from(read_u32(slot, 367));
    if prime_step != ROOT_RING_PRIME_STEP {
        return Err(crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant);
    }
    if chunk_bytes != ROOT_RING_CHUNK_BYTES {
        return Err(crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant);
    }
    if slots_per_region * u64::from(slot_spacing) > ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES {
        return Err(crate::Verdict::RootRingRegionLongerThanTheRegionStride);
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
        prime_step,
        chunk_bytes,
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
/// checker 自己按条款算一份，不引实现的 `journal::slot_after_the_journal_ring`（门禁 checker-independence-and-sync 的
/// checker-implementation-disjoint 格判 checker 不引实现）。环长是盘上读来的 8 字节：
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
    /// 自证过（magic 与整槽校验和都对）、incompat 位认得，而一个池级字段越出这一版读者收的范围（R 与 S 也算，用户 2026-10-20 定 C663）
    /// （[`value_refused_by_this_reader`] 认的那几个 [`crate::Verdict`] 成员；逐档读槽 1 时还有 I-8.1 三个字段等式，
    /// [`RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees`]）。实现遇到这一槽整池拒绝挂载
    /// （`singlefs_core::recovery::RecoveryFailure::SystemConfigurationValueRefused`）；checker 同一个处置：报
    /// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`] 违例、不替这个池作保，不退成「这一槽不可择」静默跳过
    /// （用户 2026-09-27 定系统配置越界整池拒）。
    CarriesAValueThisReaderRefuses {
        offset_in_bytes: u64,
        refused: RefusedSystemConfigurationValue,
    },
    /// 不自证：读不到、不足 4096 字节、magic 或整槽校验和不对（I-8.5（tail 槽有效集非空） 的「自证过」= magic 与整槽校验和都对）；
    /// 节点大小不是格式常量的槽也归这里（用户 2026-10-15 定 C840「当自证不过」，[`crate::Verdict::NodeBytesNotTheFormatConstant`]）。
    NotSelfVerified,
    /// 自证过（magic 与整槽校验和都对），而 incompat 位不认得：这一槽不可择。它仍是 I-8.5 数的「自证过的槽」：tail 住在自证过的槽里（D23（journal 的角色与格式） 已定项 3）。
    /// R、S 越界不在这里，归 [`Self::CarriesAValueThisReaderRefuses`]（C663）。
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
        // R 与第一版常量 3 不等、S 越界整池拒：用户 2026-10-20 定 C663「S 照条款整池拒，R 读并整池拒」
        crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
        | crate::Verdict::RegionCountPastTheRegionDeviceFields
        | crate::Verdict::RegionCountZero
        | crate::Verdict::RegionCountBelowTheFirstVersionConstant => true,
        // P、chunk 与第一版常量不等、S × 槽距大于 P × chunk 整池拒：D22（单元原子性怎么合成） 已定项 16 第 1 句，用户 2026-10-22 定
        crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant
        | crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant
        | crate::Verdict::RootRingRegionLongerThanTheRegionStride => true,
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
        | crate::Verdict::EncryptionReservedBytesNotZero
        | crate::Verdict::EntryWidthZeroWithEntries
        | crate::Verdict::RecordWidthZeroWithRecords
        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero
        | crate::Verdict::NodeBytesNotTheFormatConstant => false,
    }
}

/// 一个系统配置槽被拒的理由是不是「它自己证不了自己」：不足 4096 字节、magic 不对、整槽校验和不对
/// （[`crate::check_system_configuration_slot`] 头三道），以及节点大小不是格式常量（用户 2026-10-15 定 C840「当自证不过」，
/// 那一道排在格式版本与 incompat 位之后，但处置与头三道相同）。别的理由拒的都是自证过的槽。穷举写全，理由同 [`value_refused_by_this_reader`]。
#[must_use]
pub const fn verdict_says_the_slot_does_not_verify_itself(verdict: &crate::Verdict) -> bool {
    match verdict {
        crate::Verdict::TooShort
        | crate::Verdict::BadMagic
        | crate::Verdict::ChecksumMismatch
        | crate::Verdict::NodeBytesNotTheFormatConstant => true,
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
        | crate::Verdict::RegionCountZero
        | crate::Verdict::RegionCountBelowTheFirstVersionConstant
        | crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant
        | crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant
        | crate::Verdict::RootRingRegionLongerThanTheRegionStride
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
/// 世代号相同取偏移小的（C651（没有可用槽 0 时 checker 与实现找槽 1 读法不同） 第一轮判决第四节第 1、2 条，
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
        // 它已经把 R > 3 的槽拒成 `Verdict::RegionCountPastTheRegionDeviceFields`、R = 0 拒成 `Verdict::RegionCountZero`、
        // 基槽、R、S、槽距都过了它的判（基槽等于 64、R 等于 3、S ≤ 16、槽距是 4 字节），下面这几步乘加装得进 u64。
        let device = geometry.region_devices[usize::try_from(region).expect(
            "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
        )];
        for slot in 0..geometry.slots_per_region {
            let offset = geometry.base_slot * SLOT_BYTES
                + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
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
///
/// 同一处调用顺带判 I-2.4 的系统配置槽尾（[`judge_system_configuration_slot_bytes_past_the_field_table_are_zero`]，同一组可择的槽）：
/// 那一判该在 `walk::check_pool_image` 里自己占一个调用点，而 `walk.rs` 在 m4-c860-r1 那一轮三方的开工快照里、判决写完之前不改；
/// 判决之后把调用点挪过去、删掉这一句。
pub fn judge_system_configuration_slot_generations_follow_the_rotation(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    judge_system_configuration_slot_bytes_past_the_field_table_are_zero(readings, judgements);
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

/// I-2.4（头校验和覆盖范围） 射程里系统配置槽尾那一处：每块盘两槽里每个可择的系统配置槽（[`SystemConfigurationSlotReading::Usable`]），
/// 字段表之后到槽末（[489, 4096)，D22（单元原子性怎么合成） 已定项 9「槽内余 3607」那一段）全是 0，每槽判一格。
/// 第一版写 0、core 的读者不读这一段，往余量里加字段时同一次改动改这里的端点（用户 2026-10-20 定 C728「恒 0，由 checker 判」）。
/// 带越界值的槽归 I-7.13 整池拒、不判；incompat 位不认得的槽（[`SystemConfigurationSlotReading::SelfVerifiedButNotChoosable`]）按别的布局写，
/// 这一版读者不知道它的字段表到哪为止，也不判。只判不断。
pub fn judge_system_configuration_slot_bytes_past_the_field_table_are_zero(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    for (device, slots) in readings {
        for (slot_index, reading) in (0u64..).zip(slots) {
            let view = match reading {
                SystemConfigurationSlotReading::Usable(view, _) => view,
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
                | SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => continue,
            };
            let first_non_zero_byte = view.first_non_zero_byte_past_the_field_table;
            judgements.judge("I-2.4", first_non_zero_byte.is_none(), || {
                let crate::FirstNonZeroByteInTheSystemConfigurationSlot {
                    offset_in_the_slot,
                    value,
                } = first_non_zero_byte.expect("判红时字段表之后有非 0 字节");
                format!(
                    "盘 {device} 系统配置槽 {slot_index}（世代 {}）：槽内偏移 {offset_in_the_slot} 的字节是 {value:#04x}，字段表之后那一段（{SYSTEM_CONFIGURATION_BYTES}..{SYSTEM_CONFIGURATION_SLOT_BYTES}）第一版恒 0",
                    view.slot_generation
                )
            });
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
/// 槽里的 F 等于格式常量 `JOURNAL_SAFETY_FACTOR`（`crates/singlefs-checker/invariants/03-journal.md` I-8.1 那一行「F = 格式常量，任一不等判红」，
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

/// 一个系统配置槽是不是自证过（magic 与整槽校验和都对，节点大小是格式常量）：I-8.5（tail 槽有效集非空） 与旁路报告都按这个数。
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
    /// 少于两个槽自证过的盘（少一槽自证过；单盘全废的盘也在这里）。这类盘的可写挂载带读故障时可能判不出而拒（判不出要有槽设备报读错，
    /// D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」）；没有读故障时挂得上，取号写两轮补回那一槽。
    pub devices_with_fewer_than_two_self_verified_slots: Vec<u32>,
}

/// 逐盘数自证过的系统配置槽（magic 与整槽校验和都对），交回 [`SystemConfigurationRedundancyReport`]。槽的读法同
/// [`system_configuration_slot_readings`]。「本池自证不过」那一半（fsid 是别的池的槽）这里不分：数的是 I-8.5 那一行的「自证过」。
#[must_use]
pub fn system_configuration_redundancy_report(
    reader: &dyn ImageReader,
) -> SystemConfigurationRedundancyReport {
    let mut report = SystemConfigurationRedundancyReport {
        devices_without_a_self_verified_slot: Vec::new(),
        devices_with_fewer_than_two_self_verified_slots: Vec::new(),
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
                .devices_with_fewer_than_two_self_verified_slots
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
    /// 走读认的那两条位置条目：刚解出来时就是指针自己的（[`Self::location_hints`]）；豁免三类之外的单元经中央映射查到之后换成映射条目里的
    /// （[`Self::located_by_the_central_mapping`]，D19（块指针的结构与宽度预算） 已定项 5「中央映射是唯一入口」、已定项 8「豁免三类之外的
    /// 位置条目只当提示」；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「引用集合记映射位置」）。
    pub locations: [PointerLocation; 2],
    /// 指针自己那两条位置条目，原样：豁免三类（映射树自己的节点、树表单元、实例表单元）是权威，别的只是提示——走读先按它读、
    /// 两份都对不上再按 [`Self::locations`] 读（G3「走读先按提示读、两份都对不上再按映射位置读」）。
    pub location_hints: [PointerLocation; 2],
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

impl PointerView {
    /// 同一条指针，走读认的位置换成中央映射条目里的那两条（`mapping_locations`）；指针自己的位置条目留在 [`Self::location_hints`] 里。
    /// 查映射、认不认这一换由调用方定（豁免三类不查）。
    #[must_use]
    pub fn located_by_the_central_mapping(self, mapping_locations: [PointerLocation; 2]) -> Self {
        let mut located = self;
        located.locations = mapping_locations;
        located
    }

    /// 走读认的位置与指针自己的位置条目不同：这条指针的提示过期了（单元被搬走、映射条目换了位置），读的时候先试提示、对不上再按映射读。
    #[must_use]
    pub fn hints_differ_from_the_locations_the_walk_takes(&self) -> bool {
        self.location_hints != self.locations
    }
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
        location_hints: [location_at(bytes, 50), location_at(bytes, 64)],
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
        location_hints: [location_at(bytes, 50), location_at(bytes, 64)],
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

/// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节（D22（单元原子性怎么合成） 已定项 7 的字段表：回收门槛 t 与钉槽 p 之后，偏移 437、438、450，合计到 466）。
const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC: std::ops::Range<usize> = 437..466;

/// I-2.4（头校验和覆盖范围） 射程里根记录那一处：根环里每一条读得出、自证过的根（`valid_roots`，不按候选集滤），根槽偏移 437..466
/// （算法类型、nonce、MAC）全是 0。这一版 checker 只收加密类型 0 的系统配置，加密关着时它们恒 0、读者遇到非 0 判该结构损坏——
/// 与单元头那 29 字节、指针头部 MAC / nonce 由同一条规则守（`crates/singlefs-checker/invariants/04-校验和.md` I-2.4 那一行；判决
/// `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker，
/// 挂载透明不比）。实现今天不判这 29 字节（`crates/singlefs-core/src/root_record.rs` 的 `parse_slot` 跳过它们），checker 自己判一份。
/// 只判不断：这条根照样进候选集、照样走。
///
/// 同一处调用顺带判 I-2.4 的根槽尾（[`judge_root_slot_bytes_past_the_record_are_zero`]，同一组根）：那一判该在 `walk::check_pool_image`
/// 里自己占一个调用点，而 `walk.rs` 在 m4-c860-r1 那一轮三方的开工快照里、判决写完之前不改；判决之后把调用点挪过去、删掉这一句。
pub fn judge_root_record_algorithm_nonce_and_mac_are_zero(
    judgements: &mut Judgements,
    roots: &[(u64, u64, RootView)],
) {
    judge_root_slot_bytes_past_the_record_are_zero(judgements, roots);
    for (region, slot, root) in roots {
        let reserved = &root.record_bytes[ROOT_RECORD_ALGORITHM_NONCE_AND_MAC];
        judgements.judge("I-2.4", reserved.iter().all(|byte| *byte == 0), || {
            let first_non_zero = reserved
                .iter()
                .position(|byte| *byte != 0)
                .expect("判红时这 29 字节里有非 0 的");
            format!(
                "区域 {region} 槽 {slot} 的根（实例 {}、txg {}）：根槽偏移 {} 的字节是 {:#04x}，算法类型 / nonce / MAC 那 29 字节（437..466）加密关着时恒 0",
                root.instance,
                root.checkpoint_txg,
                ROOT_RECORD_ALGORITHM_NONCE_AND_MAC.start + first_non_zero,
                reserved[first_non_zero]
            )
        });
    }
}

/// I-2.4（头校验和覆盖范围） 射程里根槽尾那一处：根环里每一条读得出、自证过的根（`valid_roots`，不按候选集滤），根记录之后到槽宽
/// （[466, 槽宽)，槽宽是 `physical_block_size`，D22（单元原子性怎么合成） 已定项 7「512 槽内余量 46」那一段）全是 0，每条根判一格。
/// 第一版写 0（core 的 `RootRecord::to_slot` 补 0）、core 的读者不读这一段，往余量里加字段时同一次改动改这里的端点
/// （用户 2026-10-20 定 C728「恒 0，由 checker 判」）。只判不断：这条根照样进候选集、照样走。
pub fn judge_root_slot_bytes_past_the_record_are_zero(
    judgements: &mut Judgements,
    roots: &[(u64, u64, RootView)],
) {
    for (region, slot, root) in roots {
        let first_non_zero_byte = root.first_non_zero_byte_past_the_record;
        judgements.judge("I-2.4", first_non_zero_byte.is_none(), || {
            let crate::FirstNonZeroByteInTheRootSlot {
                offset_in_the_slot,
                value,
            } = first_non_zero_byte.expect("判红时根记录之后有非 0 字节");
            format!(
                "区域 {region} 槽 {slot} 的根（实例 {}、txg {}）：根槽偏移 {offset_in_the_slot} 的字节是 {value:#04x}，根记录之后到槽宽那一段（{ROOT_RECORD_BYTES} 起）第一版恒 0",
                root.instance, root.checkpoint_txg
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

#[cfg(test)]
mod tests {
    //! 系统配置槽 R、S 越界与格式版本、incompat 同在一槽的分类（C663），根槽与系统配置槽尾部恒 0（C728）：
    //! 两块盘的池按字段表手写四个系统配置槽（与一条第 0 代根），整槽校验和照 checker 自己的算法封上，交给 `walk::check_pool_image`。
    //! 池级 checker 只依赖 `singlefs-format`、没有 dev-dependencies，造不出 harness 档那种真池，坏镜像在这里按字节写。

    use super::*;

    /// 每块盘的字节数：装得下系统配置两槽、整条根环（区域 2 在 1 MiB + 2 × 3 × 1 MiB 起）与 journal 环（槽 1024 起 1 MiB）。
    const DEVICE_BYTES: usize = 18 << 20;
    /// 两块盘的身份。
    const DEVICES: [u32; 2] = [0, 1];
    /// 这个池的 fsid。
    const POOL_FILESYSTEM_IDENTIFIER: [u8; 16] = [0x5A; 16];
    /// 系统配置槽 1 的偏移（固定结构槽距，第一版 4096，D2（RAID 条带策略） 已定项 19）。
    const SLOT_SPACING_BYTES: u32 = 4096;
    /// 根槽宽（`physical_block_size`）。
    const ROOT_SLOT_BYTES: u32 = 512;
    /// journal 环长：1 MiB，单元区起始槽号 = 1024 + 64 = 1088，落在 64 槽段边界上。
    const JOURNAL_RING_BYTES: u64 = 1 << 20;
    /// mkfs 时的每区槽数（D22（单元原子性怎么合成） 已定项 2 的区间里）。
    const SLOTS_PER_REGION: u8 = 8;
    /// 系统配置字段表里 R、S 两字节的偏移（`geometry_of` 读 `slot[361]`、`slot[362]`）。
    const REGIONS_OFFSET: usize = 361;
    const SLOTS_PER_REGION_OFFSET: usize = 362;
    /// 系统配置自举头里 incompat 位图第一个字节的偏移（magic 4、格式版本 2 之后）。
    const INCOMPAT_FIRST_BYTE_OFFSET: usize = 6;
    /// 系统配置整槽校验和与根记录自证校验和的偏移（D22（单元原子性怎么合成） 已定项 7 / 已定项 9 的字段表）。
    const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
    const ROOT_CHECKSUM_OFFSET: usize = 138;

    struct DevicesInMemory {
        bytes_of_each_device: BTreeMap<u32, Vec<u8>>,
    }

    impl ImageReader for DevicesInMemory {
        fn devices(&self) -> Vec<u32> {
            self.bytes_of_each_device.keys().copied().collect()
        }
        fn device_bytes(&self, device: u32) -> Option<u64> {
            self.bytes_of_each_device
                .get(&device)
                .map(|bytes| u64::try_from(bytes.len()).expect("18 MiB"))
        }
        fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
            let bytes = self.bytes_of_each_device.get(&device)?;
            let start = usize::try_from(offset).ok()?;
            bytes
                .get(start..start.checked_add(length)?)
                .map(<[u8]>::to_vec)
        }
        fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
            Some(Vec::new())
        }
        fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
            Some(Vec::new())
        }
    }

    fn put_u32(slot: &mut [u8], offset: usize, value: u32) {
        slot[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put_u64(slot: &mut [u8], offset: usize, value: u64) {
        slot[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    /// 校验和字段先清零、对 `cover_end` 之前的字节算 CRC-32C、写进字段的前 4 字节（`checksum_field_holds` 的读法）。
    fn seal(bytes: &mut [u8], checksum_offset: usize) {
        bytes[checksum_offset..checksum_offset + 32].fill(0);
        let checksum = crate::crc32_castagnoli_table(bytes);
        bytes[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum.to_le_bytes());
    }

    /// 盘 `device` 上世代号 `slot_generation` 的系统配置槽：每个字段都取第一版读者收的值（偏移按 `.claude/kb/layout/01-first-txn.md` 一的字段表），
    /// 字段表之后全 0，整槽校验和封上。
    fn first_version_system_configuration_slot(device: u32, slot_generation: u64) -> Vec<u8> {
        let mut slot = vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")];
        slot[..4].copy_from_slice(b"SFSB");
        slot[4..6].copy_from_slice(&1u16.to_le_bytes());
        slot[INCOMPAT_FIRST_BYTE_OFFSET] = 0x02;
        slot[102..118].copy_from_slice(&POOL_FILESYSTEM_IDENTIFIER);
        put_u32(&mut slot, 139, device);
        put_u32(
            &mut slot,
            143,
            u32::try_from(DEVICES.len()).expect("两块盘"),
        );
        put_u64(&mut slot, 147, slot_generation);
        put_u32(
            &mut slot,
            301,
            u32::try_from(singlefs_format::NODE_BYTES).expect("16384"),
        );
        put_u32(&mut slot, 317, ROOT_SLOT_BYTES);
        put_u64(&mut slot, 325, JOURNAL_RING_START_SLOT);
        put_u64(&mut slot, 333, JOURNAL_RING_BYTES);
        let in_flight_record_limit =
            JOURNAL_RING_BYTES / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;
        put_u32(
            &mut slot,
            345,
            u32::try_from(in_flight_record_limit).expect("85"),
        );
        put_u64(
            &mut slot,
            349,
            in_flight_record_limit * JOURNAL_RECORD_BYTES,
        );
        put_u32(
            &mut slot,
            357,
            u32::try_from(JOURNAL_SAFETY_FACTOR).expect("3"),
        );
        slot[REGIONS_OFFSET] = u8::try_from(ROOT_RING_REGIONS).expect("3");
        slot[SLOTS_PER_REGION_OFFSET] = SLOTS_PER_REGION;
        put_u32(
            &mut slot,
            363,
            u32::try_from(singlefs_format::ROOT_RING_PRIME_STEP).expect("3"),
        );
        put_u32(
            &mut slot,
            367,
            u32::try_from(singlefs_format::ROOT_RING_CHUNK_BYTES).expect("1 MiB"),
        );
        put_u64(&mut slot, 371, ROOT_RING_BASE_SLOT);
        for (region, region_device) in singlefs_format::ROOT_RING_REGION_DEVICES.iter().enumerate()
        {
            put_u32(&mut slot, 379 + 4 * region, *region_device);
        }
        put_u64(
            &mut slot,
            417,
            JOURNAL_RING_START_SLOT + JOURNAL_RING_BYTES.div_ceil(SLOT_BYTES),
        );
        put_u32(&mut slot, 429, SLOT_SPACING_BYTES);
        seal(&mut slot, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        slot
    }

    /// 第 0 代根（mkfs 种在区域 0 槽 0 的那一条）：magic、fsid、四条指针全零，槽宽 [`ROOT_SLOT_BYTES`]、根记录之后全 0，自证校验和封上。
    fn genesis_root_slot() -> Vec<u8> {
        let mut slot = vec![0u8; usize::try_from(ROOT_SLOT_BYTES).expect("512")];
        slot[..4].copy_from_slice(b"SFSR");
        slot[4..20].copy_from_slice(&POOL_FILESYSTEM_IDENTIFIER);
        seal(&mut slot, ROOT_CHECKSUM_OFFSET);
        slot
    }

    /// 两块盘、每盘槽 0（世代 2）与槽 1（世代 1）都是第一版的值，区域 0 槽 0 有一条第 0 代根。
    fn pool_holding_only_first_version_values() -> DevicesInMemory {
        let mut bytes_of_each_device = BTreeMap::new();
        for device in DEVICES {
            let mut bytes = vec![0u8; DEVICE_BYTES];
            let slot_spacing = usize::try_from(SLOT_SPACING_BYTES).expect("4096");
            bytes[..slot_spacing]
                .copy_from_slice(&first_version_system_configuration_slot(device, 2));
            bytes[slot_spacing..2 * slot_spacing]
                .copy_from_slice(&first_version_system_configuration_slot(device, 1));
            bytes_of_each_device.insert(device, bytes);
        }
        let mut pool = DevicesInMemory {
            bytes_of_each_device,
        };
        write_root_slot_at_region_zero_slot_zero(&mut pool, &genesis_root_slot());
        pool
    }

    fn write_root_slot_at_region_zero_slot_zero(pool: &mut DevicesInMemory, root_slot: &[u8]) {
        let offset = usize::try_from(ROOT_RING_BASE_SLOT * SLOT_BYTES).expect("1 MiB");
        let device = singlefs_format::ROOT_RING_REGION_DEVICES[0];
        pool.bytes_of_each_device
            .get_mut(&device)
            .expect("区域 0 的盘在池里")[offset..offset + root_slot.len()]
            .copy_from_slice(root_slot);
    }

    /// 盘 1 槽 0 按 `rewrite` 改、整槽校验和重封（别的三槽照原样）。
    fn rewrite_and_reseal_device_one_slot_zero(
        pool: &mut DevicesInMemory,
        rewrite: impl FnOnce(&mut [u8]),
    ) {
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        let slot = &mut pool.bytes_of_each_device.get_mut(&1).expect("盘 1 在池里")[..slot_bytes];
        rewrite(slot);
        seal(slot, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    }

    fn verdict_of(
        report: &[(&'static str, InvariantVerdict)],
        invariant: &str,
    ) -> InvariantVerdict {
        report
            .iter()
            .find(|(name, _)| *name == invariant)
            .map(|(_, verdict)| verdict.clone())
            .unwrap_or_else(|| panic!("报告里有 {invariant}"))
    }

    /// 池级 checker 对这个池只报 I-7.13 违例、其余每一条都报不适用（整池不作保），盘 1 槽 0 读成带 `refused` 那个成员的不收之值。
    fn assert_device_one_slot_zero_refuses_the_whole_pool_naming(
        pool: &DevicesInMemory,
        refused: crate::Verdict,
    ) {
        let readings = system_configuration_slot_readings(pool);
        assert_eq!(
            readings[1].1[0],
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                offset_in_bytes: 0,
                refused: RefusedSystemConfigurationValue::RefusedByTheSlotChecks(refused.clone()),
            },
            "盘 1 槽 0 自证过、带这一版读者不收的值，点名 {refused:?}，不是这一槽不可择"
        );
        let report = crate::walk::check_pool_image(pool);
        let judged: Vec<&(&str, InvariantVerdict)> = report
            .iter()
            .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
            .collect();
        assert!(
            matches!(
                judged.as_slice(),
                [(
                    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                    InvariantVerdict::Violated(_)
                )]
            ),
            "整池拒：只有 I-7.13 判红、别的每一条都报不适用，实际判了的是 {judged:?}"
        );
    }

    /// 阳性对照：四槽都是第一版的值、尾部全 0，第 0 代根的槽尾全 0——四槽都可择，I-7.13 成立，I-2.4 不红。
    #[test]
    fn a_pool_holding_only_first_version_values_reads_every_slot_as_usable_and_keeps_header_checksum_coverage(
    ) {
        let pool = pool_holding_only_first_version_values();
        for (device, slots) in system_configuration_slot_readings(&pool) {
            for slot in slots {
                assert!(
                    matches!(slot, SystemConfigurationSlotReading::Usable(..)),
                    "盘 {device} 的槽读成 {slot:?}，该是可择"
                );
            }
        }
        let report = crate::walk::check_pool_image(&pool);
        assert_eq!(
            verdict_of(
                &report,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Holds,
            "四槽都只带第一版的值"
        );
        assert!(
            !matches!(verdict_of(&report, "I-2.4"), InvariantVerdict::Violated(_)),
            "槽尾全 0 的池上 I-2.4 不红：{:?}",
            verdict_of(&report, "I-2.4")
        );
    }

    /// C663 S：盘 1 槽 0 的每区槽数写 17（区间 [4, 16] 之外）、重封——D22（单元原子性怎么合成） 已定项 2「越界整池拒绝挂载」，
    /// 不再是「这一槽不可择」、别的槽照择。
    #[test]
    fn a_slot_declaring_seventeen_slots_per_region_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| {
            slot[SLOTS_PER_REGION_OFFSET] = 17
        });
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval,
        );
    }

    /// C663 R：盘 1 槽 0 的区域数写 2（与第一版常量 3 不等）、重封——用户 2026-10-20 定「R 读并整池拒」。
    #[test]
    fn a_slot_declaring_two_regions_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[REGIONS_OFFSET] = 2);
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RegionCountBelowTheFirstVersionConstant,
        );
    }

    /// C663 R：区域数写 0，报 0 那一头自己的成员，同一个处置（整池拒）。
    #[test]
    fn a_slot_declaring_zero_regions_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[REGIONS_OFFSET] = 0);
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RegionCountZero,
        );
    }

    /// C663 R：区域数写 4（比字段表留的三个逐区域设备字段多一个），报大于 3 那一头自己的成员，同一个处置（整池拒）。
    #[test]
    fn a_slot_declaring_four_regions_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[REGIONS_OFFSET] = 4);
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RegionCountPastTheRegionDeviceFields,
        );
    }

    /// C663 次序：盘 1 槽 0 格式版本写 2、incompat 第一个字节写 0x03（带退役的位 0，不认识）、重封——两样同在一槽时先判格式版本
    /// （用户 2026-10-20 定「格式版本先」）：读成带不收之值、整池拒，不是 incompat 不认得那一支的「这一槽不可择」。
    #[test]
    fn a_slot_with_an_unrecognized_format_version_and_an_unknown_incompat_bit_refuses_the_whole_pool_by_its_format_version(
    ) {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| {
            slot[4..6].copy_from_slice(&2u16.to_le_bytes());
            slot[INCOMPAT_FIRST_BYTE_OFFSET] = 0x03;
        });
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::FormatVersionNotRecognized,
        );
    }

    /// C728 系统配置槽尾：盘 1 槽 0 字段表之后的头一个字节（偏移 489）写 1、重封——整槽校验和罩着它、core 不读它，
    /// 池级 checker 判 I-2.4 红、点名那个偏移；它不是读者不收之值，I-7.13 照旧成立。
    #[test]
    fn a_non_zero_byte_right_past_the_system_configuration_field_table_reddens_header_checksum_coverage(
    ) {
        let mut pool = pool_holding_only_first_version_values();
        let field_table_end = usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("489");
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[field_table_end] = 1);
        let report = crate::walk::check_pool_image(&pool);
        let InvariantVerdict::Violated(detail) = verdict_of(&report, "I-2.4") else {
            panic!(
                "字段表之后有非 0 字节：I-2.4 该红，实际 {:?}",
                verdict_of(&report, "I-2.4")
            );
        };
        assert!(
            detail.contains("系统配置槽") && detail.contains(&format!("偏移 {field_table_end} ")),
            "违例点名系统配置槽偏移 {field_table_end}：{detail}"
        );
        assert_eq!(
            verdict_of(
                &report,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Holds,
            "槽尾非 0 不是读者不收之值，不整池拒"
        );
    }

    /// C728 根槽尾：第 0 代根的根记录之后头一个字节（偏移 466）写 1、重封自证校验和——池级 checker 判 I-2.4 红、点名那个偏移。
    #[test]
    fn a_non_zero_byte_right_past_the_root_record_reddens_header_checksum_coverage() {
        let mut pool = pool_holding_only_first_version_values();
        let record_end = usize::try_from(ROOT_RECORD_BYTES).expect("466");
        let mut root_slot = genesis_root_slot();
        root_slot[record_end] = 1;
        seal(&mut root_slot, ROOT_CHECKSUM_OFFSET);
        write_root_slot_at_region_zero_slot_zero(&mut pool, &root_slot);
        let report = crate::walk::check_pool_image(&pool);
        let InvariantVerdict::Violated(detail) = verdict_of(&report, "I-2.4") else {
            panic!(
                "根记录之后有非 0 字节：I-2.4 该红，实际 {:?}",
                verdict_of(&report, "I-2.4")
            );
        };
        assert!(
            detail.contains("根槽偏移 466 "),
            "违例点名根槽偏移 466：{detail}"
        );
    }
}
