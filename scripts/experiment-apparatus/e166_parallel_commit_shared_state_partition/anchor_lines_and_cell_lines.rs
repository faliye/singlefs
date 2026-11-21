//! 停机对拍与锚点的各组行、产物行的格式化：格行、轨迹行、轮数行。

use super::*;

// ── 钉绝对值的断言（登记第七节）与停机对拍 ───────────────────────────────────────

pub(super) struct AnchorLine {
    pub(super) identifier: String,
    pub(super) observed: String,
    pub(super) expected: String,
}

impl AnchorLine {
    pub(super) fn new(identifier: &str, observed: String, expected: &str) -> AnchorLine {
        AnchorLine { identifier: identifier.to_string(), observed, expected: expected.to_string() }
    }

    pub(super) fn matches(&self) -> bool {
        self.observed == self.expected
    }
}

pub(super) fn specification_of(
    arm: Arm,
    variant: ArmVariant,
    stage: ParallelStage,
    shape_kind: ShapeKind,
    initial_has_file: bool,
    transaction_count: usize,
    device_count: usize,
) -> CellSpecification {
    CellSpecification::default_geometry(arm, variant, stage, shape_kind, initial_has_file, transaction_count, device_count)
}

pub(super) fn should_memoize(specification: &CellSpecification) -> bool {
    specification.arm == Arm::DevicePartition
}

pub(super) fn enumerate_specification(specification: CellSpecification) -> Enumeration {
    enumerate_schedules(&Model::new(specification), should_memoize(&specification))
}

fn duplicate_pair(aggregate: &Aggregate, resource: Resource) -> String {
    format!("{}/{}", aggregate.duplicate_total[resource.index()], aggregate.duplicate_schedules[resource.index()])
}

pub(super) fn slot_text(key: u64) -> String {
    format!("({},{})", key >> 32, key & 0xffff_ffff)
}

fn join_numbers<Number: ToString>(numbers: &[Number]) -> String {
    numbers.iter().map(ToString::to_string).collect::<Vec<String>>().join(",")
}

/// 甲组：串行基线 S 与 Q1 候选的定义（登记第七节甲 1–8）。
pub(super) fn anchors_from_the_clauses() -> Vec<AnchorLine> {
    let mut lines = Vec::new();
    let full_stage = ParallelStage::ManyPublishesInFlight;
    let serial_model = Model::new(specification_of(
        Arm::WholeLock,
        ArmVariant::Normal,
        full_stage,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    let serial_state = run_serially(&serial_model);
    let held: Vec<Holdings> = serial_state.transactions[..3].iter().map(|transaction| transaction.items).collect();
    let txgs: Vec<u8> = held.iter().map(|each| each.txg).collect();
    lines.push(AnchorLine::new("甲1", join_numbers(&txgs), "6,7,8"));
    for (slots_per_region, expected) in [(4u32, "(0,2),(1,2),(2,2)"), (1u32, "(0,0),(1,0),(2,0)")] {
        let slots: Vec<String> = txgs.iter().map(|txg| slot_text(root_ring_slot_key(*txg, slots_per_region))).collect();
        lines.push(AnchorLine::new(&format!("甲2 S={slots_per_region}"), slots.join(","), expected));
    }
    let counters: Vec<u8> = held.iter().map(|each| each.counter).collect();
    lines.push(AnchorLine::new("甲3 计数器", join_numbers(&counters), "6,7,8"));
    let ring_slots: Vec<String> =
        counters.iter().map(|counter| journal_ring_slot(*counter, DEFAULT_JOURNAL_RING_SLOTS).to_string()).collect();
    lines.push(AnchorLine::new("甲3 环槽", ring_slots.join(","), "5,6,7"));
    lines.push(AnchorLine::new(
        "甲3 默认环槽数",
        (JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES).to_string(),
        &DEFAULT_JOURNAL_RING_SLOTS.to_string(),
    ));
    let transaction_numbers: Vec<u8> = held.iter().map(|each| each.transaction_number).collect();
    lines.push(AnchorLine::new("甲4 事务号", join_numbers(&transaction_numbers), "2,3,4"));
    let counter_minus_transaction: Vec<u8> =
        held.iter().map(|each| each.counter - each.transaction_number).collect();
    lines.push(AnchorLine::new("甲4 计数器减事务号", join_numbers(&counter_minus_transaction), "4,4,4"));
    let inode_starts: Vec<u8> = held.iter().map(|each| each.inode_start).collect();
    lines.push(AnchorLine::new("甲5 inode 号", join_numbers(&inode_starts), "2,3,4"));
    lines.push(AnchorLine::new("甲5 期末水位", serial_state.shared.inode_watermark.to_string(), "5"));
    let user_start_slots: Vec<String> = held
        .iter()
        .map(|each| (u32::from(each.user_pairs[0]) * USER_DATA_SPAN_SLOTS).to_string())
        .collect();
    lines.push(AnchorLine::new("甲7 用户数据落点起点", user_start_slots.join(","), "0,2,4"));
    let commit_nodes: Vec<String> = held
        .iter()
        .map(|each| format!("{{{},{}}}", each.commit_slot_start, each.commit_slot_start + 1))
        .collect();
    lines.push(AnchorLine::new("甲7 提交内生块落点", commit_nodes.join(","), "{0,1},{2,3},{4,5}"));

    let no_file_model = Model::new(specification_of(
        Arm::WholeLock,
        ArmVariant::Normal,
        full_stage,
        ShapeKind::SameShape,
        false,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    let no_file_state = run_serially(&no_file_model);
    let tree_starts: Vec<TreeIdentifierNumber> =
        no_file_state.transactions[..3].iter().map(|transaction| transaction.items.tree_start).collect();
    lines.push(AnchorLine::new("甲6 发树 ID 的起点（0 = 没发）", join_numbers(&tree_starts), "65536,0,0"));
    // 期末水位的期望取字面量（D8 已定项 11「第一个文件版本那次之后 65548」），不取常量本身：常量错了期望不跟着错。
    lines.push(AnchorLine::new("甲6 期末树 ID 水位", no_file_state.shared.tree_watermark.to_string(), "65548"));
    let mut keys = Vec::new();
    push_held_keys(&no_file_model, Resource::TreeIdentifier, &no_file_state.transactions[0].items, &mut keys);
    lines.push(AnchorLine::new(
        "甲6 新池新建文件发的树 ID",
        keys.iter().map(u64::to_string).collect::<Vec<String>>().join(","),
        "65536,65537,65538,65539,65540,65541,65542,65543,65544,65545,65546,65547",
    ));

    let one_publish_model = Model::new(specification_of(
        Arm::WholeLock,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    let one_publish_state = run_serially(&one_publish_model);
    let batch = batch_publisher_holdings(&one_publish_state.shared);
    let transactions_with_own_txg =
        one_publish_state.transactions[..3].iter().filter(|transaction| transaction.items.txg != 0).count();
    lines.push(AnchorLine::new(
        "甲8 一个 txg 与一个根环槽",
        format!(
            "batch_txg={} batch_root_slot={} transactions_with_own_txg={}",
            batch.txg,
            slot_text(root_ring_slot_key(batch.txg, DEFAULT_ROOT_SLOTS_PER_REGION)),
            transactions_with_own_txg
        ),
        "batch_txg=6 batch_root_slot=(0,2) transactions_with_own_txg=0",
    ));
    lines
}

fn whole_copy_controls() -> [(Arm, ArmVariant); 3] {
    [
        (Arm::WholeLock, ArmVariant::PositiveControl),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl),
        (Arm::OptimisticReadSet, ArmVariant::PositiveControl),
    ]
}

/// 乙组与补充锚点：设计时用命令独立算出的数（登记第七节乙 1–7；补 1 是执行员另写 Python 算的 A字面 锁外两列）。
pub(super) fn anchors_from_independent_calculations() -> Vec<AnchorLine> {
    let mut lines = Vec::new();
    // 乙 1：整份写回，同形，初始态有文件，Q1甲 的资源集合。
    for (arm, variant) in whole_copy_controls() {
        for (transaction_count, expected) in [
            (2, "schedules=6 U=4/4 G=8/4 I=4/4 X=4/4"),
            (3, "schedules=90 U=120/84 G=240/84 I=120/84 X=120/84"),
        ] {
            let enumeration = enumerate_specification(specification_of(
                arm,
                variant,
                ParallelStage::PrepareParallelCommitSerial,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let aggregate = &enumeration.aggregate;
            lines.push(AnchorLine::new(
                &format!("乙1 {} n={transaction_count}", arm.label(variant)),
                format!(
                    "schedules={} U={} G={} I={} X={}",
                    aggregate.schedules,
                    duplicate_pair(aggregate, Resource::UserDataSlot),
                    duplicate_pair(aggregate, Resource::CommitGeneratedSlot),
                    duplicate_pair(aggregate, Resource::InodeNumber),
                    duplicate_pair(aggregate, Resource::TransactionNumber)
                ),
                expected,
            ));
        }
        // 乙 1 末句的结构延伸（登记 5.3 末句）：Q1丙 下单号资源各自等于「单件」那一行、G 等于「两件」那一行；
        // Q1乙 下 G、TXG、RS、T 归批发布者、不争用，所以是 0。
        for (transaction_count, single, double) in [(2usize, "4/4", "8/4"), (3, "120/84", "240/84")] {
            let full_stage = enumerate_specification(specification_of(
                arm,
                variant,
                ParallelStage::ManyPublishesInFlight,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let mut observed = Vec::new();
            for resource in [
                Resource::UserDataSlot,
                Resource::InodeNumber,
                Resource::TransactionNumber,
                Resource::JsnCounter,
                Resource::JournalRingSlot,
                Resource::CheckpointTxg,
                Resource::RootRingSlot,
            ] {
                observed.push(format!("{}={}", resource.label(), duplicate_pair(&full_stage.aggregate, resource)));
            }
            observed.push(format!("G={}", duplicate_pair(&full_stage.aggregate, Resource::CommitGeneratedSlot)));
            let expected_text = format!(
                "U={single} I={single} X={single} J={single} JS={single} TXG={single} RS={single} G={double}"
            );
            lines.push(AnchorLine::new(
                &format!("乙1 Q1丙 {} n={transaction_count}", arm.label(variant)),
                observed.join(" "),
                &expected_text,
            ));
            let one_publish = enumerate_specification(specification_of(
                arm,
                variant,
                ParallelStage::OnePublishCarriesManyTransactions,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let mut observed_one_publish = Vec::new();
            for resource in [
                Resource::UserDataSlot,
                Resource::InodeNumber,
                Resource::TransactionNumber,
                Resource::JsnCounter,
                Resource::JournalRingSlot,
                Resource::CommitGeneratedSlot,
                Resource::CheckpointTxg,
                Resource::RootRingSlot,
                Resource::TreeIdentifier,
            ] {
                observed_one_publish
                    .push(format!("{}={}", resource.label(), duplicate_pair(&one_publish.aggregate, resource)));
            }
            let expected_one_publish = format!(
                "U={single} I={single} X={single} J={single} JS={single} G=0/0 TXG=0/0 RS=0/0 T=0/0"
            );
            lines.push(AnchorLine::new(
                &format!("乙1 Q1乙 {} n={transaction_count}", arm.label(variant)),
                observed_one_publish.join(" "),
                &expected_one_publish,
            ));
        }
    }
    // 乙 2：整份写回与 P区同，初始态无文件，树 ID 一列（Q1甲）。
    let tree_controls = [
        (Arm::WholeLock, ArmVariant::PositiveControl),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl),
        (Arm::OptimisticReadSet, ArmVariant::PositiveControl),
        (Arm::RegionPartition, ArmVariant::PositiveControl),
    ];
    for (arm, variant) in tree_controls {
        for (transaction_count, expected) in [(2, "T=48/4"), (3, "T=1296/72")] {
            let enumeration = enumerate_specification(specification_of(
                arm,
                variant,
                ParallelStage::PrepareParallelCommitSerial,
                ShapeKind::SameShape,
                false,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            lines.push(AnchorLine::new(
                &format!("乙2 {} n={transaction_count}", arm.label(variant)),
                format!("T={}", duplicate_pair(&enumeration.aggregate, Resource::TreeIdentifier)),
                expected,
            ));
        }
    }
    // 乙 3：P区同，同形，有文件。
    for (transaction_count, expected) in [
        (2, "schedules=6 U=4/4 G=8/4 I=6/6 X=6/6"),
        (3, "schedules=90 U=120/84 G=240/84 I=180/90 X=180/90"),
    ] {
        let enumeration = enumerate_specification(specification_of(
            Arm::RegionPartition,
            ArmVariant::PositiveControl,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let aggregate = &enumeration.aggregate;
        lines.push(AnchorLine::new(
            &format!("乙3 P区同 n={transaction_count}"),
            format!(
                "schedules={} U={} G={} I={} X={}",
                aggregate.schedules,
                duplicate_pair(aggregate, Resource::UserDataSlot),
                duplicate_pair(aggregate, Resource::CommitGeneratedSlot),
                duplicate_pair(aggregate, Resource::InodeNumber),
                duplicate_pair(aggregate, Resource::TransactionNumber)
            ),
            expected,
        ));
        // J、JS、TXG、RS 在它们争用的 Q1 候选下等于 I 那一列。
        let inode_number_column = if transaction_count == 2 { "6/6" } else { "180/90" };
        for (stage, resources) in [
            (ParallelStage::OnePublishCarriesManyTransactions, vec![Resource::JsnCounter, Resource::JournalRingSlot]),
            (
                ParallelStage::ManyPublishesInFlight,
                vec![Resource::JsnCounter, Resource::JournalRingSlot, Resource::CheckpointTxg, Resource::RootRingSlot],
            ),
        ] {
            let contended = enumerate_specification(specification_of(
                Arm::RegionPartition,
                ArmVariant::PositiveControl,
                stage,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let observed: Vec<String> = resources
                .iter()
                .map(|resource| format!("{}={}", resource.label(), duplicate_pair(&contended.aggregate, *resource)))
                .collect();
            let expected_text: Vec<String> =
                resources.iter().map(|resource| format!("{}={}", resource.label(), inode_number_column)).collect();
            lines.push(AnchorLine::new(
                &format!("乙3 P区同 {} n={transaction_count}", stage.label()),
                observed.join(" "),
                &expected_text.join(" "),
            ));
        }
    }
    // 乙 4 / 乙 5：O整 的重复数恒 0；重算（调度数、重算合计、有重算的调度数、单调度最大重算）。
    for (transaction_count, expected) in [(2, "schedules=6 retries=4 retry_schedules=4 max_retries=1"), (3, "schedules=498 retries=1140 retry_schedules=492 max_retries=3")] {
        let mut every_cell_matches = true;
        let mut nonzero_duplicate_cells = 0;
        let mut first_observed = String::new();
        for stage in ALL_STAGES {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for initial_has_file in [true, false] {
                    let enumeration = enumerate_specification(specification_of(
                        Arm::OptimisticWholeVersion,
                        ArmVariant::Normal,
                        stage,
                        shape_kind,
                        initial_has_file,
                        transaction_count,
                        DEFAULT_DEVICE_COUNT,
                    ));
                    let aggregate = &enumeration.aggregate;
                    let observed = format!(
                        "schedules={} retries={} retry_schedules={} max_retries={}",
                        aggregate.schedules,
                        aggregate.retries_total,
                        aggregate.retries_schedules,
                        aggregate.retries_maximum_schedule
                    );
                    if first_observed.is_empty() {
                        first_observed = observed.clone();
                    }
                    every_cell_matches &= observed == first_observed;
                    if aggregate.duplicate_total.iter().any(|total| *total != 0) {
                        nonzero_duplicate_cells += 1;
                    }
                }
            }
        }
        lines.push(AnchorLine::new(&format!("乙4 O整 重复数全 0 的格外个数 n={transaction_count}"), nonzero_duplicate_cells.to_string(), "0"));
        lines.push(AnchorLine::new(
            &format!("乙5 O整 重算（12 格逐格相同）n={transaction_count}"),
            if every_cell_matches { first_observed } else { "12 格不全相同".to_string() },
            expected,
        ));
    }
    // 乙 6：A字面去锁（四步），同形，Q1甲，U 一列。
    for (transaction_count, expected) in [(2, "schedules=70 U_total=36 U_schedules=36"), (3, "schedules=34650 U_total=37368 U_schedules=29268")] {
        let enumeration = enumerate_specification(specification_of(
            Arm::LiteralLock,
            ArmVariant::PositiveControl,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let aggregate = &enumeration.aggregate;
        lines.push(AnchorLine::new(
            &format!("乙6 A字面去锁 n={transaction_count}"),
            format!(
                "schedules={} U_total={} U_schedules={}",
                aggregate.schedules,
                aggregate.duplicate_total[Resource::UserDataSlot.index()],
                aggregate.duplicate_schedules[Resource::UserDataSlot.index()]
            ),
            expected,
        ));
    }
    // 乙 7：P设 的 U 一列，同形，重算上限 2。
    let device_partition_expectations: [(ArmVariant, usize, usize, &str); 8] = [
        (ArmVariant::Normal, 1, 2, "70,0,0,0,0,0"),
        (ArmVariant::Normal, 1, 3, "34650,0,0,0,0,0"),
        (ArmVariant::Normal, 2, 2, "14982,53010,56376,3366,0,0"),
        (ArmVariant::Normal, 2, 3, "1029677534586,6043544394480,6199013074122,155468679642,0,0"),
        (ArmVariant::PositiveControl, 1, 2, "252,0,0,0,120,120"),
        (ArmVariant::PositiveControl, 1, 3, "756756,0,0,0,767880,616680"),
        (ArmVariant::PositiveControl, 2, 2, "2510072,9546090,11143686,1597596,1516320,758160"),
        (
            ArmVariant::PositiveControl,
            2,
            3,
            "15204801979811387676,90958422775773687714,102730730512623627846,11772307736849940132,21393243243591046560,9560362788127086480",
        ),
    ];
    for (variant, device_count, transaction_count, expected) in device_partition_expectations {
        let enumeration = enumerate_specification(specification_of(
            Arm::DevicePartition,
            variant,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            transaction_count,
            device_count,
        ));
        lines.push(AnchorLine::new(
            &format!("乙7 {} D={device_count} n={transaction_count}", Arm::DevicePartition.label(variant)),
            device_partition_tuple(&enumeration.aggregate),
            expected,
        ));
    }
    // 补 1：A字面 锁外两列（事务号、树 ID），Q1甲，同形；独立 Python（`literal_outside_anchor.py`）算出。
    for (initial_has_file, transaction_count, expected) in [
        (true, 2, "schedules=20 X=18/18 T=0/0"),
        (true, 3, "schedules=1680 X=2970/1674 T=0/0"),
        (false, 2, "schedules=20 X=18/18 T=216/18"),
        (false, 3, "schedules=1680 X=2970/1674 T=34992/1620"),
    ] {
        let enumeration = enumerate_specification(specification_of(
            Arm::LiteralLock,
            ArmVariant::Normal,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            initial_has_file,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let aggregate = &enumeration.aggregate;
        lines.push(AnchorLine::new(
            &format!("补1 A字面 锁外 file={initial_has_file} n={transaction_count}"),
            format!(
                "schedules={} X={} T={}",
                aggregate.schedules,
                duplicate_pair(aggregate, Resource::TransactionNumber),
                duplicate_pair(aggregate, Resource::TreeIdentifier)
            ),
            expected,
        ));
    }
    lines
}

/// P设 的 U 一列按乙 7 口径：（调度数，重算合计，不一致合计，截断数，重复合计，有重复的调度数）。
fn device_partition_tuple(aggregate: &Aggregate) -> String {
    format!(
        "{},{},{},{},{},{}",
        aggregate.schedules,
        aggregate.retries_total,
        aggregate.retries_total + aggregate.truncated,
        aggregate.truncated,
        aggregate.duplicate_total[Resource::UserDataSlot.index()],
        aggregate.duplicate_schedules[Resource::UserDataSlot.index()]
    )
}

fn factorial(value: u32) -> u128 {
    (1..=u128::from(value)).product()
}

/// 丙 1：没有重算、没有 FALLBACK 的臂，调度数 = (n·k)! ÷ (k!)ⁿ。
pub(super) fn schedule_count_without_retries(transaction_count: u32, steps_per_transaction: u32) -> u128 {
    factorial(transaction_count * steps_per_transaction) / factorial(steps_per_transaction).pow(transaction_count)
}

/// 丙组：结构性计数。
pub(super) fn anchors_from_structural_counts() -> Vec<AnchorLine> {
    let mut lines = Vec::new();
    let stage = ParallelStage::PrepareParallelCommitSerial;
    let cases: [(Arm, ArmVariant, usize, u32); 10] = [
        (Arm::WholeLock, ArmVariant::Normal, DEFAULT_DEVICE_COUNT, 1),
        (Arm::RegionPartition, ArmVariant::Normal, DEFAULT_DEVICE_COUNT, 2),
        (Arm::RegionPartition, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::WholeLock, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::OptimisticReadSet, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::LiteralLock, ArmVariant::Normal, DEFAULT_DEVICE_COUNT, 3),
        (Arm::LiteralLock, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 4),
        (Arm::DevicePartition, ArmVariant::Normal, SAMPLE_DEVICE_COUNT, 4),
        (Arm::DevicePartition, ArmVariant::PositiveControl, SAMPLE_DEVICE_COUNT, 5),
    ];
    for (arm, variant, device_count, steps_per_transaction) in cases {
        for transaction_count in [2usize, 3] {
            let enumeration = enumerate_specification(specification_of(
                arm,
                variant,
                stage,
                ShapeKind::SameShape,
                true,
                transaction_count,
                device_count,
            ));
            let expected = schedule_count_without_retries(
                u32::try_from(transaction_count).expect("n 很小"),
                steps_per_transaction,
            );
            lines.push(AnchorLine::new(
                &format!("丙1 {} k={steps_per_transaction} n={transaction_count}", arm.label(variant)),
                enumeration.aggregate.schedules.to_string(),
                &expected.to_string(),
            ));
        }
    }
    // 丙 2：串行基线 S 的调度数 1，重复数 0，重算 0。
    for transaction_count in [2usize, 3] {
        let model = Model::new(specification_of(
            Arm::WholeLock,
            ArmVariant::Normal,
            ParallelStage::ManyPublishesInFlight,
            ShapeKind::SameShape,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let state = run_serially(&model);
        let metrics = state_metrics(&model, &state);
        let retries: u32 = state.transactions[..transaction_count].iter().map(|each| u32::from(each.retries)).sum();
        lines.push(AnchorLine::new(
            &format!("丙2 串行基线 n={transaction_count}"),
            format!("duplicates={} retries={}", metrics.duplicates.iter().sum::<u32>(), retries),
            "duplicates=0 retries=0",
        ));
    }
    // 丙 3：P设 的记忆化计数与逐叶枚举逐项相等（整份汇总，含轨迹，n = 2）。
    for variant in [ArmVariant::Normal, ArmVariant::PositiveControl] {
        for device_count in [1usize, 2] {
            let model = Model::new(specification_of(
                Arm::DevicePartition,
                variant,
                stage,
                ShapeKind::SameShape,
                true,
                2,
                device_count,
            ));
            let leafwise = enumerate_schedules(&model, false);
            let memoized = enumerate_schedules(&model, true);
            lines.push(AnchorLine::new(
                &format!("丙3 {} D={device_count} n=2 记忆化 = 逐叶", Arm::DevicePartition.label(variant)),
                (leafwise.aggregate == memoized.aggregate).to_string(),
                "true",
            ));
        }
    }
    lines
}

pub(super) fn all_anchor_lines() -> Vec<AnchorLine> {
    let mut lines = anchors_from_the_clauses();
    lines.extend(anchors_from_independent_calculations());
    lines.extend(anchors_from_structural_counts());
    lines
}

// ── 产物：每格的结果行 ─────────────────────────────────────────────────────

pub(super) fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn initial_label(initial_has_file: bool) -> &'static str {
    if initial_has_file {
        "file"
    } else {
        "nofile"
    }
}

fn cell_prefix(name: &str, specification: &CellSpecification) -> String {
    format!(
        "name={name} arm={} stage={} shape={} initial={} transactions={} devices={}",
        specification.arm.label(specification.variant),
        specification.stage.label(),
        specification.shape_kind.label(),
        initial_label(specification.initial_has_file),
        specification.transaction_count,
        specification.device_count
    )
}

pub(super) fn cell_line(name: &str, specification: &CellSpecification, aggregate: &Aggregate) -> String {
    let mut line = cell_prefix(name, specification);
    write!(line, " schedules={}", aggregate.schedules).expect("写进 String 不会失败");
    for resource in ALL_RESOURCES {
        let index = resource.index();
        write!(
            line,
            " {label}_total={} {label}_schedules={} {label}_max_single={}",
            aggregate.duplicate_total[index],
            aggregate.duplicate_schedules[index],
            aggregate.duplicate_maximum_single_schedule[index],
            label = resource.label()
        )
        .expect("写进 String 不会失败");
    }
    write!(
        line,
        " retries_total={} retries_schedules={} retries_max_schedule={} retries_max_transaction={} \
         device_disagreements={} truncated={} fallbacks_total={} deadlocks={}",
        aggregate.retries_total,
        aggregate.retries_schedules,
        aggregate.retries_maximum_schedule,
        aggregate.retries_maximum_transaction,
        if specification.arm == Arm::DevicePartition { aggregate.retries_total + aggregate.truncated } else { 0 },
        aggregate.truncated,
        aggregate.fallbacks_total,
        aggregate.deadlocks
    )
    .expect("写进 String 不会失败");
    line
}

fn margin_text(value: i32) -> String {
    if value == i32::MAX {
        "none".to_string()
    } else {
        value.to_string()
    }
}

pub(super) fn trajectory_line(specification: &CellSpecification, aggregate: &Aggregate) -> String {
    let mut line = cell_prefix("trajectory", specification);
    for resource in ALL_RESOURCES {
        let index = resource.index();
        write!(
            line,
            " {label}_peak={} {label}_positive_steps={}",
            aggregate.duplicate_peak[index],
            aggregate.duplicate_positive_steps[index],
            label = resource.label()
        )
        .expect("写进 String 不会失败");
    }
    write!(
        line,
        " inode_margin_minimum={} inode_margin_negative_steps={} inode_margin_final_minimum={} \
         busy_regions_peak={} busy_regions_positive_steps={} busy_regions_final_maximum={} \
         unused_reserved_peak={} unused_reserved_positive_steps={} unused_reserved_final_maximum={}",
        margin_text(aggregate.inode_margin_minimum),
        aggregate.inode_margin_negative_steps,
        margin_text(aggregate.inode_margin_final_minimum),
        aggregate.busy_regions_peak,
        aggregate.busy_regions_positive_steps,
        aggregate.busy_regions_final_maximum,
        aggregate.unused_reserved_peak,
        aggregate.unused_reserved_positive_steps,
        aggregate.unused_reserved_final_maximum
    )
    .expect("写进 String 不会失败");
    line
}

fn counts_text(counts: &[u32]) -> String {
    if counts.is_empty() {
        "none".to_string()
    } else {
        counts.iter().map(u32::to_string).collect::<Vec<String>>().join("/")
    }
}

/// A 与 A字面 的锁里装了多少计划是 Q1 的测量，登记第六节 Q-9 不对它们报 R_n。
pub(super) fn reports_rounds(specification: &CellSpecification) -> bool {
    match specification.arm {
        Arm::WholeLock | Arm::LiteralLock => false,
        Arm::RegionPartition | Arm::DevicePartition | Arm::OptimisticWholeVersion | Arm::OptimisticReadSet => true,
    }
}

pub(super) fn rounds_line(specification: &CellSpecification, outcome: &RoundsOutcome) -> String {
    let mut line = cell_prefix("rounds", specification);
    let serial_rounds = specification.transaction_count;
    write!(
        line,
        " rounds={} serial_rounds={serial_rounds} commits_per_round={} peak_commits_in_a_round={} rounds_with_commits={} last_round_commits={} \
         parallel_gain_seen_in_model={}",
        outcome.rounds.map_or("none".to_string(), |rounds| rounds.to_string()),
        counts_text(&outcome.commits_per_round),
        outcome.commits_per_round.iter().copied().max().unwrap_or(0),
        outcome.commits_per_round.iter().filter(|commits| **commits > 0).count(),
        outcome.commits_per_round.last().copied().unwrap_or(0),
        outcome
            .rounds
            .is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") < serial_rounds)
    )
    .expect("写进 String 不会失败");
    line
}
