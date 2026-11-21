//! 已知坏镜像共用的干净镜像与改盘：覆盖写之后的镜像与它那张坏镜像表、判一份镜像的结论、回退与恢复之后的镜像、
//! 改日志记录与根槽的辅助，以及各组都用的格式偏移。

use super::*;

/// 再覆盖写一次（发布 B，txg 4）之后单元区里的单元：A 的十四个槽仍在盘上（根环里 A 的根还引用着它们），加上 B 写出的十二个单元
/// （落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
pub(super) const UNITS_AFTER_OVERWRITE: [(u64, usize); 26] = [
    (50176, 32768),
    (50178, 16384),
    (50180, 32768),
    (50182, 32768),
    (50240, 16384),
    (50242, 32768),
    (50244, 16384),
    (50245, 16384),
    (50246, 16384),
    (50247, 16384),
    (50248, 16384),
    (50249, 16384),
    (50250, 16384),
    (50251, 16384),
    (50252, 16384),
    (50253, 16384),
    (50254, 32768),
    (50256, 16384),
    (50257, 16384),
    (50258, 16384),
    (50259, 16384),
    (50260, 16384),
    (50261, 16384),
    (50262, 16384),
    (50263, 16384),
    (50264, 16384),
];

/// 发布 B 写出的分配记录树两片叶（盘 0 的叶 61、盘 1 的叶 61；I-3.9、I-3.10、I-5.4 那几份坏镜像改的就是它们）与树表（I-9.14 那一份改它）。
/// 每片叶只装它那块盘上的记录：一个单元两盘各一份落点，两条记录分在两片叶里。
pub(super) const ALLOCATION_LEAVES_AFTER_OVERWRITE: [u64; 2] = [50257, 50258];

pub(super) const TREE_TABLE_AFTER_OVERWRITE: u64 = 50264;

/// journal 记录头里 `checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 4 / 已定项 17 的字段表）。
pub(super) const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;

pub(super) fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// journal 记录头里这三个字段的偏移（D23（journal 的角色与格式） 已定项 17 的字段表；记录头自证校验和在 46、罩整条 4096）。
pub(super) const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;

pub(super) const JOURNAL_RECORD_TRANSACTION_OFFSET: usize = 78;

/// 这几份镜像上环里的记录条数上界：读到这个计数器还没读完就说明镜像变了，断言会响（别让列表被悄悄截断）。
pub(super) const RECORDS_READ_BACK_AT_MOST: u64 = 64;

/// 一块盘的 journal 环里从计数器 1 起连着的记录，读成 (计数器, 实例代号, 事务号)；读到没有 magic 的那一格就停。
/// 计数器与环槽一一对应（`record_offset`），所以这里按计数器取偏移，不整条环扫。
pub(super) fn journal_records_on_device(pool: &MemoryPool, device: u32) -> Vec<(u64, u32, u64)> {
    let mut records = Vec::new();
    for counter in 1..=RECORDS_READ_BACK_AT_MOST {
        let offset = singlefs_core::journal::record_offset(
            counter,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        )
        .0;
        let bytes = read(pool, device, offset, 4096);
        if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
            break;
        }
        records.push((
            counter,
            u32::from_le_bytes(
                bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
                    .try_into()
                    .expect("4 字节"),
            ),
            get_u64(&bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
        ));
    }
    assert!(
        u64::try_from(records.len()).expect("条数") < RECORDS_READ_BACK_AT_MOST,
        "环里的记录多到读满了上界 {RECORDS_READ_BACK_AT_MOST}：这几份镜像上不该有这么多"
    );
    records
}

/// 一条 journal 记录的事务号改成 `transaction`，头校验和重封（罩整条 4096，字段在 46）；两盘各一份都改——
/// 事务号重号是写者算错号，不是一块盘的字节坏了，两盘会写出同一条记录。
pub(super) fn set_transaction_number_of_journal_record(
    pool: &mut MemoryPool,
    counter: u64,
    expected_transaction: u64,
    transaction: u64,
) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(pool, device, offset, 4096);
        assert_eq!(
            get_u64(&record, JOURNAL_RECORD_TRANSACTION_OFFSET),
            expected_transaction,
            "盘 {device} 计数器 {counter} 那条记录的事务号"
        );
        set_u64(&mut record, JOURNAL_RECORD_TRANSACTION_OFFSET, transaction);
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(pool, device, offset, &record);
    }
}

/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）几何段里几个 16 KiB 槽号与环长的偏移：journal 环起点 325、环长 333、
/// 根环起点 371、单元区起始槽号 417。checker 与实现今天各自按这几个偏移读；这里照字段表各写一份。
const SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET: usize = 325;

const SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET: usize = 333;

const SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET: usize = 371;

const SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET: usize = 417;

/// 第一版 mkfs 写进这几格的值（字段表那一列）：journal 环从槽 1024 起、默认环长 768 MiB、根环从槽 64 起、单元区从槽 50176 起。
const JOURNAL_RING_START_SLOT_OF_THE_POOL: u64 = 1024;

pub(super) const JOURNAL_RING_BYTES_OF_THE_POOL: u64 = 805_306_368;

const ROOT_RING_BASE_SLOT_OF_THE_POOL: u64 = 64;

pub(super) const UNIT_AREA_START_SLOT_OF_THE_POOL: u64 = 50176;

/// 聚簇段 64 槽（D3（空间分配） 已定项 10 ①）：单元区起始槽号要落在它的整数倍上。
const CLUSTER_SEGMENT_SLOTS_OF_THE_POOL: u64 = 64;

/// I-7.13（系统配置池级字段在读者收的范围里） 的几何那五格（A、B、C、D、E，读字段，不等于常量就整池拒）：
/// 盘 1 槽 0 的一个几何字段改掉、整槽重封，别的三槽照原样。每项带 checker 该报的那个 `Verdict` 成员名（违例说明里带着它）。
pub(super) fn known_bad_images_of_the_system_configuration_geometry(
) -> Vec<(&'static str, Mutation)> {
    let on_the_refused_slot = |change: fn(&mut Vec<u8>)| -> Mutation {
        Box::new(move |image: &mut MemoryPool| {
            mutate_system_configuration_slot(
                image,
                DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                change,
            );
        })
    };
    vec![
        // A：单元区起始槽号往后挪一段（50176 → 50240），环长不动：不是环长现算的那个槽。
        (
            "UnitAreaStartNotTheSlotAfterTheJournalRing",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
                    UNIT_AREA_START_SLOT_OF_THE_POOL + CLUSTER_SEGMENT_SLOTS_OF_THE_POOL,
                );
            }),
        ),
        // D：环长少一槽（768 MiB − 16 KiB），单元区起始槽号跟着改成它现算的 50175（两者自洽）：不在聚簇段边界上。
        (
            "UnitAreaStartOffTheClusterSegmentBoundary",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
                    JOURNAL_RING_BYTES_OF_THE_POOL - SLOT,
                );
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
                    UNIT_AREA_START_SLOT_OF_THE_POOL - 1,
                );
            }),
        ),
        // B：journal 环起点写 1023：实现按常量 1024 写记录、扫环，按字段扫的读者判的是错位 16 KiB 的另一个环。
        (
            "JournalRingStartSlotNotTheFirstVersionConstant",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET,
                    JOURNAL_RING_START_SLOT_OF_THE_POOL - 1,
                );
            }),
        ),
        // C：根环起点写 65：实现按常量 64 写根，按字段读的读者读的是另一个根环。
        (
            "RootRingBaseSlotNotTheFirstVersionConstant",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET,
                    ROOT_RING_BASE_SLOT_OF_THE_POOL + 1,
                );
            }),
        ),
        // E：根环起点写 0：先于槽距判，报起点、不报「槽 1 不在根环起点之前」。
        (
            "RootRingBaseSlotNotTheFirstVersionConstant",
            on_the_refused_slot(|bytes| {
                set_u64(bytes, SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET, 0);
            }),
        ),
    ]
}

/// 一条树表条目里诞生 txg 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8
/// （D8（核心索引结构） 已定项 8 的字段表）。
pub(super) const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;

/// 分配记录树叶里，把代字段等于 `from` 的已释放记录改成 `to`；返回改了几条。
pub(super) fn rewrite_released_generations(bytes: &mut [u8], from: u64, to: u64) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let mut rewritten = 0;
    for index in 0..count {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
        if span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0 && get_u64(bytes, record + 12) == from
        {
            set_u64(bytes, record + 12, to);
            rewritten += 1;
        }
    }
    rewritten
}

/// 发布 B（txg 4）之后的干净镜像：A 的十二个单元（十四个槽）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
/// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
/// 要的跨根比较到这里才有内容：写完新池新建文件的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
/// 两次暖机空发布不写树表），I-9.14 在那份镜像上报「不适用」。
pub(super) fn image_after_the_overwrite(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let previous = pool.output.clone();
    // 第二次写的内容与第一次不同长、不同字节；具体哪些字节这里不承重（落点与释放代由步 1 的验收钉住）。
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let second = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    assert_eq!(
        second.root.checkpoint_txg,
        CheckpointTxg(4),
        "发布 B 是 txg 4"
    );
    pool.memory_pool()
}

/// C374（释放代与树表诞生 txg 只有验收断言盯着） 立的那两条各一份坏镜像，都从发布 B 之后的干净镜像出发改一处。
pub(super) fn known_bad_images_after_the_overwrite() -> Vec<(&'static str, Mutation)> {
    vec![
        // 发布 B 那次的释放代写成上一次发布的 txg（4 → 3）：A 的十二个单元两盘各一条（两片叶各十二条），二十四条一起改。
        (
            "I-3.9",
            Box::new(|image: &mut MemoryPool| {
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES_AFTER_OVERWRITE,
                        |bytes| rewrite_released_generations(bytes, 4, 3),
                    ),
                    24,
                    "A 的十二个单元、两盘各一条"
                );
            }),
        ),
        // 树表里 extent 树（树 ID 65536）那条的诞生 txg 改成跟着这次发布的 txg（3 → 4）：A 写的那一版树表里仍是 3。
        (
            "I-9.14",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    TREE_TABLE_AFTER_OVERWRITE,
                    |bytes| {
                        let birth_txg =
                            tree_table_entry_offset(0) + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET;
                        assert_eq!(get_u64(bytes, birth_txg), 3, "extent 树的诞生 txg 本来是 3");
                        set_u64(bytes, birth_txg, 4);
                    },
                    true,
                );
            }),
        ),
        // I-4.9：A 的数据单元（50180，只有 A 的根 txg 3 还引用）诞生代号 3 → 4，引用它的四处一起改成 4（A 的 extent 树根里的数据指针、
        // A 的中央映射里那条码 1 key、A 的分配记录树两片叶里那两条未释放记录的分配代），各自重封、沿引用链补到 A 的根：
        // 指针、映射、分配代与单元头都对得上（I-1.2、I-1.11、I-3.10 不红），按已发布谓词判也是已发布的（4 ≤ 挂载根 txg 4），
        // 只剩「A 那条根引用了一个比它晚诞生的单元」。
        (
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    DATA_UNIT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                            3,
                            "A 的数据单元诞生代号本来是 3"
                        );
                        set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 4);
                    },
                    true,
                );
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    EXTENT_ROOT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET),
                            3,
                            "A 的数据指针出生 txg 本来是 3"
                        );
                        set_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, 4);
                    },
                    true,
                );
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    MAPPING_ROOT,
                    |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
                    true,
                );
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES,
                        |bytes| {
                            rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4)
                        }
                    ),
                    2,
                    "A 的数据单元两盘各一条未释放记录"
                );
            }),
        ),
        // I-8.11：B 那条记录（计数器 4）里码 1 那个点名项（B 的数据单元 50182）的诞生代号 4 → 3，两盘各一份，载荷校验和与头校验和重封。
        // 位置条目没动、50182 的整单元校验和照旧相符 ⇒ 点名项与单元头比得上，而诞生代号不同。B 是环里最后一条，没有后继的反向链要补。
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_birth_txg_of_the_data_unit_named_by_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    4,
                    3,
                );
            }),
        ),
        // I-8.11 点名项另外四个字段各一份（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 4 条），同一个点名项、
        // 同一种重封：类标签 1 → 3（码 3 与码 1 读同样长，位置条目的整单元校验和照旧相符）、出生树改成另一棵发过号的树、
        // key 尾段的写序实例代号 1 → 2、写序事务号加 1。
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        assert_eq!(entry[NAMED_ENTRY_UNIT_CLASS_OFFSET], 1, "点名的是码 1");
                        entry[NAMED_ENTRY_UNIT_CLASS_OFFSET] = 3;
                    },
                );
            }),
        ),
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        assert_eq!(
                            get_u64(entry, NAMED_ENTRY_BIRTH_TREE_OFFSET),
                            TREE_IDENTIFIER_EXTENT,
                            "数据单元的出生树本来是 extent 树"
                        );
                        set_u64(
                            entry,
                            NAMED_ENTRY_BIRTH_TREE_OFFSET,
                            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
                        );
                    },
                );
            }),
        ),
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        assert_eq!(
                            get_u32(entry, NAMED_ENTRY_KEY_TAIL_INSTANCE_OFFSET),
                            1,
                            "写序实例代号本来是 1"
                        );
                        set_u32(entry, NAMED_ENTRY_KEY_TAIL_INSTANCE_OFFSET, 2);
                    },
                );
            }),
        ),
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        let transaction_byte = NAMED_ENTRY_KEY_TAIL_TRANSACTION_OFFSET;
                        entry[transaction_byte] = entry[transaction_byte].wrapping_add(1);
                    },
                );
            }),
        ),
        // I-8.12：B 那条记录（计数器 4）的 checkpoint_txg 4 → 0，两盘各一份、头校验和重封：同实例前一条（计数器 3）是 txg 3。
        // 实例 1 没有 txg 0 的根（mkfs 种的第 0 代根属实例 0）⇒ I-8.10 没有可比的根；B 是最后一条，没有后继的反向链要补。
        (
            JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
            Box::new(|image: &mut MemoryPool| {
                set_checkpoint_txg_of_journal_record(image, LAST_COUNTER_AFTER_THE_OVERWRITE, 4, 0);
            }),
        ),
        // I-7.8 最新那条根那一格（`research/prompts/m3-inv-a-r1-opus-output.md` 三·3 攻方第一份）：只把 B 的根（txg 4，最新）的水位压到
        // mkfs 时的水位，B 那条记录的新根段跟着压（同一份新根段，I-8.10）。A 的根水位照旧盖得住 A 发的十二棵树 ⇒ 取 max 那一格不红。
        (
            "I-7.8",
            Box::new(|image: &mut MemoryPool| {
                let (device, offset) = root_slot_of(CheckpointTxg(4));
                let mut bytes = read(image, device, offset, 512);
                assert_eq!(get_u64(&bytes, 28), 4, "那一槽是 B 的根");
                set_u64(
                    &mut bytes,
                    ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET,
                    singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
                );
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                write(image, device, offset, &bytes);
                assert_eq!(
                    copy_each_root_into_the_new_root_segment_of_its_journal_records(image),
                    2,
                    "B 那条记录两盘各一份跟着压"
                );
            }),
        ),
    ]
}

/// 记录头里 fsid 那 8 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 373
/// （D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表）。
pub(super) const JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET: usize = 472;

/// 发布 B 之后那份镜像环里最后一条记录（B 那条，事务号 2）的计数器：I-8.7、I-8.8 的几份镜像都从它后面接着写。
pub(super) const LAST_COUNTER_AFTER_THE_OVERWRITE: u64 = 4;

/// 一份镜像上判红的不变量，次序照 checker 的清单。
pub(super) fn violated_invariants(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, found)| matches!(found, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect()
}

/// `changed` 之外每一条不变量在两份镜像上的判定：逐项不同的列出来（空 = 逐项相同）。
pub(super) fn verdicts_that_differ_outside<'verdicts>(
    verdicts: &'verdicts [(&'static str, InvariantVerdict)],
    verdicts_on_the_clean_image: &'verdicts [(&'static str, InvariantVerdict)],
    changed: &[&str],
) -> Vec<(
    &'static str,
    &'verdicts InvariantVerdict,
    &'verdicts InvariantVerdict,
)> {
    verdicts
        .iter()
        .zip(verdicts_on_the_clean_image)
        .filter(|((name, found), (_, before))| !changed.contains(name) && found != before)
        .map(|((name, found), (_, before))| (*name, before, found))
        .collect()
}

/// 新池新建文件那条根（txg 3）的根槽：区域 `3 mod 3` = 0（盘 0）的槽 `(3 div 3) mod 8` = 1（D22（单元原子性怎么合成）
/// 已定项 2 / 已定项 16；区域 0 起点 64 × 16384，槽距 4096）。
pub(super) const NEW_POOL_FILE_CREATION_ROOT_SLOT: (u32, u64) = (0, 64 * SLOT + 4096);

/// 这个池发过、又不是中央映射树的一个号（livelist 树 65541），低于水位 65548。
pub(super) const ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK: u64 = 65541;

/// A (1, 3)、B (1, 4) → 重开取号 2、写行、暖机两次、C (2, 8)，与 `rollback_by_a_forward_publish.rs` 的固定脚本同一段。
/// 交回池、A 与 C 那两版。
pub(super) fn pool_through_the_third_version(
    tag: &str,
) -> (BuiltPool, TransactionOutput, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
        .expect("重开取号 2");
    pool.devices = Some(devices_reopened_for_the_second_instance);
    pool.allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third_content: Vec<u8> = (0..2500usize)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect();
    let third = publish_overwrite_in_process(
        &mut pool,
        &current,
        &third_content,
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(2),
    )
    .expect("覆盖写 C");
    pool.output = third.clone();
    (pool, first, third)
}

/// 挂着的时候回退到 A 之后的干净镜像（D23（journal 的角色与格式） 已定项 14：一次向前发布 D (2, 9)，新根的 inode 树、extent 树取 A 的，
/// 树表现算、另写一片）。A、B、C、D 同在一条时间线上：没有哪条根被实例表判出局。
pub(super) struct ImageAfterTheRollbackToTheFirstRoot {
    pub(super) image: MemoryPool,
    /// 回退目标 A 的 txg：它那一版树表里 extent 树那条的诞生 txg 就是它。
    pub(super) rollback_target_txg: u64,
    /// D 那片树表单元的槽号（两盘同槽）与 D 那次发布的 txg。
    pub(super) tree_table_slot_of_the_rollback_publish: u64,
    pub(super) rollback_publish_txg: u64,
}

/// 两盘同槽（D2（RAID 条带策略） 已定项 10）：`mutate_unit_of` 读第一块盘那一份、两盘写同一个槽，靠的就是这一条。
pub(super) fn slot_shared_by_both_copies(
    pointer_locations: [singlefs_core::pointer::LocationEntry; 2],
) -> u64 {
    assert_eq!(
        pointer_locations[0].slot, pointer_locations[1].slot,
        "两盘同槽：这份镜像的改法按一个槽号改两盘"
    );
    pointer_locations[0].slot.0
}

pub(super) fn image_after_rolling_back_to_the_first_root(
    tag: &str,
) -> ImageAfterTheRollbackToTheFirstRoot {
    let (mut pool, first, mut third) = pool_through_the_third_version(tag);
    let rollback_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        devices,
        &mut pool.allocator,
        &mut third,
        RollbackTarget {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
        },
    )
    .expect("挂着的时候回退到 A");
    let rollback_publish = third;
    ImageAfterTheRollbackToTheFirstRoot {
        image: pool.memory_pool(),
        rollback_target_txg: first.root.checkpoint_txg.0,
        tree_table_slot_of_the_rollback_publish: slot_shared_by_both_copies(
            rollback_publish.root.tree_table.locations,
        ),
        rollback_publish_txg: rollback_publish.root.checkpoint_txg.0,
    }
}

/// 崩溃恢复抛弃 C 之后的干净镜像（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：
/// C 的根槽与数据单元暂时读不出，恢复落到 (2, 7)，实例 3 写行与暖机，再把 C 写回）：C (2, 8) 读得出、按实例 3 的实例表判出局。
/// 交回镜像、A 那一版记的诞生 txg、C 那片树表单元的槽号与 C 那次发布的 txg。
pub(super) fn image_after_a_recovery_abandoned_the_third_root(
    tag: &str,
) -> (MemoryPool, u64, u64, u64) {
    let (mut pool, first, third) = pool_through_the_third_version(tag);
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    (
        pool.memory_pool(),
        first.root.checkpoint_txg.0,
        slot_shared_by_both_copies(third.root.tree_table.locations),
        third.root.checkpoint_txg.0,
    )
}

/// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树的两片叶（`ALLOCATION_LEAVES_AFTER_OVERWRITE`）里，未释放、分配代 4，
/// 单元头里的诞生代号也是 4（落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
pub(super) const DATA_UNIT_AFTER_OVERWRITE: u64 = 50182;

/// checkpoint_txg 为 `txg` 的那条根住的根槽：(盘, 偏移)（与写者同一条根环公式）。
pub(super) fn root_slot_of(txg: CheckpointTxg) -> (u32, u64) {
    let target = singlefs_core::root_ring::target_for_publish(
        txg,
        parameters().geometry.root_ring_slots_per_region,
    );
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = singlefs_core::root_ring::slot_offset(
        target,
        parameters().geometry.fixed_structure_slot_spacing,
    );
    (device.0, offset.0)
}

/// 记录头里点名项数（4 字节）与载荷校验和（4 字节）的偏移、记录头宽、一条点名项的宽（D23（journal 的角色与格式） 已定项 4 /
/// 已定项 17 的字段表：magic 4 + 类型 2 + 算法 1 + 记录标志 1 + 记录长 4 之后是点名项数；事务段 78 + 事务号 8 + 提交标记 1 +
/// 本次发布内序号 4 + 反向链 4 之后是载荷校验和，罩记录头之后的点名项）。
pub(super) const JOURNAL_RECORD_NAMED_COUNT_OFFSET: usize = 12;

pub(super) const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;

pub(super) const JOURNAL_RECORD_HEADER_BYTES: usize = 496;

pub(super) const JOURNAL_NAMED_ENTRY_BYTES: usize = 56;

/// 根记录里树 ID 水位那 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表：树 ID 水位 122、自证校验和 138）。
pub(super) const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET: usize = 122;

/// 一条 journal 记录（两盘各一份）里码 1 那个点着 `data_unit_slot` 的点名项，诞生代号从 `from` 改成 `to`；
/// 载荷校验和（罩点名项）与头校验和（罩整条 4096，字段在 46）重封。
fn rewrite_the_birth_txg_of_the_data_unit_named_by_journal_record(
    image: &mut MemoryPool,
    counter: u64,
    data_unit_slot: u64,
    from: u64,
    to: u64,
) {
    rewrite_the_named_entry_of_the_data_unit_in_journal_record(
        image,
        counter,
        data_unit_slot,
        |entry| {
            assert_eq!(
                get_u64(entry, NAMED_ENTRY_BIRTH_TXG_OFFSET),
                from,
                "计数器 {counter} 那条记录点名的诞生代号本来是 {from}"
            );
            set_u64(entry, NAMED_ENTRY_BIRTH_TXG_OFFSET, to);
        },
    );
}

/// 点名项（56 字节，D23（journal 的角色与格式） 已定项 17）里的字段偏移：第一条位置条目的槽号（盘 4 之后）、类标签 28、出生树 29、
/// 诞生代号 37、key 尾段 45（码 1：写序实例代号 4 + 事务号 6）。
const NAMED_ENTRY_FIRST_LOCATION_SLOT_OFFSET: usize = 4;

const NAMED_ENTRY_UNIT_CLASS_OFFSET: usize = 28;

const NAMED_ENTRY_BIRTH_TREE_OFFSET: usize = 29;

const NAMED_ENTRY_BIRTH_TXG_OFFSET: usize = 37;

const NAMED_ENTRY_KEY_TAIL_INSTANCE_OFFSET: usize = 45;

const NAMED_ENTRY_KEY_TAIL_TRANSACTION_OFFSET: usize = 49;

/// 一条 journal 记录（两盘各一份）里码 1 那个点着 `data_unit_slot` 的点名项交给 `change` 改（拿到的是那 56 字节）；
/// 载荷校验和（罩点名项）与头校验和（罩整条 4096，字段在 46）重封。
fn rewrite_the_named_entry_of_the_data_unit_in_journal_record(
    image: &mut MemoryPool,
    counter: u64,
    data_unit_slot: u64,
    change: impl Fn(&mut [u8]),
) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(image, device, offset, 4096);
        let named_count = usize::try_from(u32::from_le_bytes(
            record[JOURNAL_RECORD_NAMED_COUNT_OFFSET..JOURNAL_RECORD_NAMED_COUNT_OFFSET + 4]
                .try_into()
                .expect("4 字节"),
        ))
        .expect("点名项数");
        let entry_start = (0..named_count)
            .map(|index| JOURNAL_RECORD_HEADER_BYTES + JOURNAL_NAMED_ENTRY_BYTES * index)
            .find(|entry_start| {
                let mut slot_bytes = [0u8; 8];
                slot_bytes[..6].copy_from_slice(
                    &record[entry_start + NAMED_ENTRY_FIRST_LOCATION_SLOT_OFFSET
                        ..entry_start + NAMED_ENTRY_FIRST_LOCATION_SLOT_OFFSET + 6],
                );
                record[entry_start + NAMED_ENTRY_UNIT_CLASS_OFFSET] == 1
                    && u64::from_le_bytes(slot_bytes) == data_unit_slot
            })
            .expect("那条记录点名了这个数据单元");
        change(&mut record[entry_start..entry_start + JOURNAL_NAMED_ENTRY_BYTES]);
        let payload_end = JOURNAL_RECORD_HEADER_BYTES + JOURNAL_NAMED_ENTRY_BYTES * named_count;
        let payload_checksum = crc32_castagnoli(&record[JOURNAL_RECORD_HEADER_BYTES..payload_end]);
        record[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&payload_checksum.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(image, device, offset, &record);
    }
}

/// 一条 journal 记录（两盘各一份）的 `checkpoint_txg` 从 `from` 改成 `to`，头校验和重封。
fn set_checkpoint_txg_of_journal_record(image: &mut MemoryPool, counter: u64, from: u64, to: u64) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(image, device, offset, 4096);
        assert_eq!(
            get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
            from,
            "盘 {device} 计数器 {counter} 那条记录的 txg 本来是 {from}"
        );
        set_u64(&mut record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET, to);
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(image, device, offset, &record);
    }
}

/// inode 叶容器第 0 条记录的起点（码 3 记录区从 136 起）与记录里改动计数的偏移（D8（核心索引结构） 已定项 6：偏移 88）。
pub(super) const INODE_LEAF_FIRST_RECORD_START: usize = 136;

/// 根记录里实例表指针的起点（86 字节，D22（单元原子性怎么合成） 已定项 7）与指针头部 extent 偏移（D19（块指针的结构与宽度预算） 已定项 11：
/// MAC 16 + nonce 12 + 算法类型 1 + 压缩算法 1 + 压后长度 2 之后的 2 字节）。
pub(super) const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;

pub(super) const POINTER_EXTENT_OFFSET_FIELD_OFFSET: usize = 32;

/// 指针（节点指针 86、数据指针 88 同一个前缀）里两条位置条目的起点与每条宽（头部 50 之后，位置条目 = 盘 4 + 槽 6 + 整单元校验和 4，
/// D19（块指针的结构与宽度预算） 已定项 11）。
pub(super) const POINTER_FIRST_LOCATION_OFFSET: usize = 50;

pub(super) const POINTER_LOCATION_BYTES: usize = 14;

/// 树的种类码里 inode 树那一个（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.25 那一行「种类码 ≠ 2」）。
pub(super) const TREE_KIND_INODE: u16 = 2;

/// 位置条目里整单元校验和的偏移（盘 4 + 槽 6 之后）。
pub(super) const LOCATION_CHECKSUM_OFFSET: usize = 10;

/// 树的种类码里 extent 树那一个（D8（核心索引结构） 已定项 9 的登记表）。
pub(super) const TREE_KIND_EXTENT: u16 = 1;

/// 改 checkpoint_txg 为 `txg` 的那条根的根槽、重算整槽自证校验和，再把每条根抄进它那几条记录的新根段（I-8.10 照旧成立）。
pub(super) fn rewrite_the_root_slot_of(
    image: &mut MemoryPool,
    txg: CheckpointTxg,
    change: impl Fn(&mut [u8]),
) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = read(image, device, offset, 512);
    assert_eq!(get_u64(&bytes, 28), txg.0, "那一槽是 txg {} 的根", txg.0);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(image, device, offset, &bytes);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

pub(super) const NODE_POINTER_BYTES: usize = 86;

/// 一份判定里 `invariant` 那一条。
pub(super) fn verdict_among(
    verdicts: &[(&'static str, InvariantVerdict)],
    invariant: &str,
) -> InvariantVerdict {
    verdicts
        .iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
        .clone()
}

/// 违例的那几条，按名字排成集合（比两份镜像红的是不是同一批，不管清单次序）。
pub(super) fn violated_invariant_set(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> std::collections::BTreeSet<&'static str> {
    violated_invariants(verdicts).into_iter().collect()
}
