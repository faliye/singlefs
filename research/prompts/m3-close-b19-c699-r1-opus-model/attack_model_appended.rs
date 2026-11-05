
// ============================================================================
// 攻方腿模型（三方轮 m3-close-b19-c699-r1，攻方腿）：复跑时由 tools/rerun.sh 接在 E198（取号跳过认出生代号的前提） 装置源码末尾，
// 装置自己的 main 改名不用，拼成一个独立 bin 包（只在草稿副本里，不进仓）。只复用装置的造历史、账本真值与判法读镜像那几段，
// 另加三样：
//   乙（甲加读坏槽护住）：甲认作接得上，且 (R⁻.txg, R.txg) 之间每个 txg 的落点槽都不是「读得出不是全零而自证不过、或读不出」；
//   乙计全零：同上，全零的落点槽也算（D16（发布语义） 已定项 1 把读回全零的槽与读坏槽同样当作可能有根，乙的定义没并它）；
//   丁（攻方自提，被攻过零轮）：今天的三条件成立，且 R 那条根的分配记录里没有一条代落在 (R⁻.txg, R.txg) 开区间里。
// 组：
//   sw-read-control   E198 的 sw-read 原形（锚：甲与今天的臂各认错，E198 第二段产物 Q10 那一行 value_pairs=315）
//   sw-crash-control  E198 的 sw-crash 原形（锚：一对洞对都没有，E198 第二段产物 Q6 那一行 hole_pairs=0）
//   sw-crash-wrapped  sw-crash 的清扫实例在清扫之后改用顺序写（c × 524 288 字节），把清扫之前那几版的 journal 记录盖掉（Q2.2）
//   skr-gap-fault     E198 的 skr 原形，每张镜像另出一张：洞里那个 txg（实例 2 崩掉的写行那次）的落点槽读坏（Q2.1）
//   rs-two-files      rs1 换成两个文件：实例 1 另建 inode 2 并写它，回退之后写 inode 2，读坏实例 2 在 [r, R) 里的根槽，跑今天的池级 checker（Q2.3）
// 崩溃：沿用装置的单一崩溃点（那一步写流截在第一个根槽写之前），没有调 enumerate_crash_states，限度写在报告里。
// ============================================================================

use singlefs_core::address::InodeNumber;
use singlefs_core::allocator::AllocationRecordQuarantineMark;
use singlefs_core::recovery::allocation_records_under_root;
use singlefs_core::transaction::publish_overwrite_of_an_inode;

/// 洞里一个 txg 的落点槽此刻是什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttackGapSlotState {
    RootOf(VersionKey),
    AllZero,
    ReadableNotSelfVerified,
    Unreadable,
}

impl AttackGapSlotState {
    fn text(self) -> String {
        match self {
            AttackGapSlotState::RootOf(key) => format!("root{}", key.text()),
            AttackGapSlotState::AllZero => "zero".to_string(),
            AttackGapSlotState::ReadableNotSelfVerified => "bad".to_string(),
            AttackGapSlotState::Unreadable => "unreadable".to_string(),
        }
    }

    /// 乙的定义：读得出不是全零而自证不过，或读不出。
    fn is_read_bad(self) -> bool {
        matches!(
            self,
            AttackGapSlotState::ReadableNotSelfVerified | AttackGapSlotState::Unreadable
        )
    }
}

/// 洞里每个 txg 的落点槽（本地落点公式，与装置记账本用的同一个）此刻的样子：住着一条自证过的根就记它是哪一版，否则按读出的字节分。
fn attack_gap_slot_states(
    image: &MemoryPool,
    slots_per_region: u64,
    previous_txg: u64,
    version_txg: u64,
) -> Vec<(u64, AttackGapSlotState)> {
    let Ok(system_configuration) = choose_system_configuration(image) else {
        return Vec::new();
    };
    let immutable = &system_configuration.immutable;
    let ring = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let mut states = Vec::new();
    for txg in previous_txg + 1..version_txg {
        let (region, slot) = ring_slot_by_the_local_formula(txg, slots_per_region);
        let occupant = ring
            .iter()
            .find(|(read_slot, _)| read_slot.region == region && read_slot.slot == slot)
            .map(|(_, root)| VersionKey::of_root(root));
        let state = match occupant {
            Some(key) => AttackGapSlotState::RootOf(key),
            None => {
                let device =
                    immutable.region_devices[usize::try_from(region).expect("区域号装得进 usize")];
                let offset = slot_offset(
                    RootRingSlot { region, slot },
                    immutable.sizes.fixed_structure_slot_spacing,
                );
                match singlefs_core::recovery::PoolReader::read(
                    image,
                    device,
                    offset,
                    usize::try_from(PHYSICAL_BLOCK_BYTES).expect("512 装得进 usize"),
                ) {
                    None => AttackGapSlotState::Unreadable,
                    Some(bytes) if bytes.iter().all(|byte| *byte == 0) => {
                        AttackGapSlotState::AllZero
                    }
                    Some(_) => AttackGapSlotState::ReadableNotSelfVerified,
                }
            }
        };
        states.push((txg, state));
    }
    states
}

/// R 那条根的分配记录里，代落在 (R⁻.txg, R.txg) 开区间里的那几条（已释放的是释放代、未释放的是分配代、带隔离记号的另列）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AttackGapGenerations {
    released: Vec<u64>,
    allocated: Vec<u64>,
    quarantined: Vec<u64>,
}

impl AttackGapGenerations {
    fn is_empty_apart_from_quarantine(&self) -> bool {
        self.released.is_empty() && self.allocated.is_empty()
    }

    fn text(&self) -> String {
        let list = |values: &[u64]| {
            if values.is_empty() {
                "-".to_string()
            } else {
                values
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join("/")
            }
        };
        format!(
            "released:{};allocated:{};quarantined:{}",
            list(self.released.as_slice()),
            list(self.allocated.as_slice()),
            list(self.quarantined.as_slice())
        )
    }
}

/// R 是根环里一条自证过的根时，读它的分配记录、取代落在洞里的那几条；R 不在根环里（由记录施加出来的版本）或账读不出交 `Err` 说明。
fn attack_gap_generations_in_the_accounting(
    image: &MemoryPool,
    version: VersionKey,
    previous_txg: u64,
) -> Result<AttackGapGenerations, String> {
    let system_configuration = choose_system_configuration(image)
        .map_err(|failure| format!("system_configuration_unreadable:{failure:?}"))?;
    let immutable = &system_configuration.immutable;
    let ring = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let root = ring
        .iter()
        .map(|(_, root)| root)
        .find(|root| VersionKey::of_root(root) == version)
        .ok_or_else(|| "not_a_root_on_the_ring".to_string())?;
    let records = allocation_records_under_root(image, root)
        .map_err(|failure| format!("accounting_unreadable:{failure:?}"))?;
    let mut generations = AttackGapGenerations::default();
    for record in &records {
        let generation = record.generation.0;
        if generation <= previous_txg || generation >= version.txg {
            continue;
        }
        if record.is_released {
            generations.released.push(generation);
        } else if matches!(
            record.quarantine_mark,
            AllocationRecordQuarantineMark::NotQuarantined
        ) {
            generations.allocated.push(generation);
        } else {
            generations.quarantined.push(generation);
        }
    }
    for values in [
        &mut generations.released,
        &mut generations.allocated,
        &mut generations.quarantined,
    ] {
        values.sort_unstable();
        values.dedup();
    }
    Ok(generations)
}

/// 一个洞对在五条臂下的分类与真值。
#[derive(Clone, Debug)]
struct AttackPairJudgement {
    text: String,
    truth: Truth,
    today: bool,
    with_pointer_birth: bool,
    read_bad_gap_slot_guarded: bool,
    read_bad_or_zero_gap_slot_guarded: bool,
    accounting_generation_guarded: Option<bool>,
}

/// 五条臂各自的 E1、E2 与配套计数（一组合起来）。
#[derive(Clone, Debug, Default)]
struct AttackArmTotals {
    images: u64,
    unreadable_images: u64,
    hole_pairs: u64,
    truths: BTreeMap<String, u64>,
    errors: BTreeMap<String, u64>,
    accounting_unknown_pairs: u64,
    all_three_candidates_e1: u64,
    pool_checker_reference_interval: BTreeMap<String, u64>,
    pool_checker_release_generation: BTreeMap<String, u64>,
    histories: u64,
    stopped_histories: u64,
}

impl AttackArmTotals {
    fn add(&mut self, other: &AttackArmTotals) {
        self.images += other.images;
        self.unreadable_images += other.unreadable_images;
        self.hole_pairs += other.hole_pairs;
        self.accounting_unknown_pairs += other.accounting_unknown_pairs;
        self.all_three_candidates_e1 += other.all_three_candidates_e1;
        self.histories += other.histories;
        self.stopped_histories += other.stopped_histories;
        for (into, from) in [
            (&mut self.truths, &other.truths),
            (&mut self.errors, &other.errors),
            (
                &mut self.pool_checker_reference_interval,
                &other.pool_checker_reference_interval,
            ),
            (
                &mut self.pool_checker_release_generation,
                &other.pool_checker_release_generation,
            ),
        ] {
            for (key, value) in from {
                *into.entry(key.clone()).or_insert(0) += value;
            }
        }
    }

    fn note_pair(&mut self, pair: &AttackPairJudgement) {
        self.hole_pairs += 1;
        *self.truths.entry(pair.truth.name().to_string()).or_insert(0) += 1;
        let directly_bridged = match pair.truth {
            Truth::DirectlyBridged => true,
            Truth::ExistedVersionsInBetween | Truth::PreviousNotOnTheChain => false,
            Truth::VersionMissingFromTheLedger => return,
        };
        let mut note = |arm: &str, bridged: bool| {
            if bridged && !directly_bridged {
                *self.errors.entry(format!("{arm}_e1")).or_insert(0) += 1;
            }
            if directly_bridged && !bridged {
                *self.errors.entry(format!("{arm}_e2")).or_insert(0) += 1;
            }
        };
        note("today", pair.today);
        note("with_pointer_birth", pair.with_pointer_birth);
        note("read_bad_gap_slot_guarded", pair.read_bad_gap_slot_guarded);
        note(
            "read_bad_or_zero_gap_slot_guarded",
            pair.read_bad_or_zero_gap_slot_guarded,
        );
        if let Some(bridged) = pair.accounting_generation_guarded {
            note("accounting_generation_guarded", bridged);
        }
        if pair.accounting_generation_guarded.is_none() {
            self.accounting_unknown_pairs += 1;
        }
        if !directly_bridged
            && pair.today
            && pair.with_pointer_birth
            && pair.read_bad_gap_slot_guarded
        {
            self.all_three_candidates_e1 += 1;
        }
    }

    fn text(&self) -> String {
        let map_text = |map: &BTreeMap<String, u64>| {
            if map.is_empty() {
                "none".to_string()
            } else {
                map.iter()
                    .map(|(key, value)| format!("{key}:{value}"))
                    .collect::<Vec<_>>()
                    .join(",")
            }
        };
        format!(
            "histories={} stopped_histories={} images={} unreadable_images={} hole_pairs={} truths={} errors={} accounting_unknown_pairs={} all_three_candidates_e1={} pool_checker_reference_interval={} pool_checker_release_generation={}",
            self.histories,
            self.stopped_histories,
            self.images,
            self.unreadable_images,
            self.hole_pairs,
            map_text(&self.truths),
            map_text(&self.errors),
            self.accounting_unknown_pairs,
            self.all_three_candidates_e1,
            map_text(&self.pool_checker_reference_interval),
            map_text(&self.pool_checker_release_generation)
        )
    }
}

/// 判一张镜像：装置的读法取洞对与前两条臂，账本取真值，另算乙、乙计全零、丁；`run_the_pool_checker` 时另跑今天的 `check_pool_image`。
/// 交回这一张的结果行（镜像一行、每个洞对一行）与计数。
// gpu-exempt: attack-model-measures-the-cpu-judge-itself
fn attack_judge_the_image(
    image: &MemoryPool,
    ledger: &Ledger,
    slots_per_region: u64,
    prefix: &str,
    run_the_pool_checker: bool,
) -> (Vec<String>, AttackArmTotals) {
    let mut lines = Vec::new();
    let mut totals = AttackArmTotals {
        images: 1,
        ..AttackArmTotals::default()
    };
    let reading = match read_the_image(image) {
        Ok(reading) => reading,
        Err(reason) => {
            totals.unreadable_images = 1;
            lines.push(format!(
                "name=image {prefix} unreadable={}",
                reason.replace(' ', "_")
            ));
            return (lines, totals);
        }
    };
    let mut pair_texts = Vec::new();
    for pair in &reading.hole_pairs {
        let truth = truth_of_the_hole_pair(ledger, &pair.previous_group, pair.version.key);
        let gap_slots = attack_gap_slot_states(
            image,
            slots_per_region,
            pair.previous_txg,
            pair.version.key.txg,
        );
        let read_bad_gap = gap_slots.iter().any(|(_, state)| state.is_read_bad());
        let zero_gap = gap_slots
            .iter()
            .any(|(_, state)| *state == AttackGapSlotState::AllZero);
        let generations =
            attack_gap_generations_in_the_accounting(image, pair.version.key, pair.previous_txg);
        let accounting_generation_guarded = generations
            .as_ref()
            .ok()
            .map(|found| pair.bridged_today && found.is_empty_apart_from_quarantine());
        let judgement = AttackPairJudgement {
            text: String::new(),
            truth,
            today: pair.bridged_today,
            with_pointer_birth: pair.bridged_with_the_pointer_birth,
            read_bad_gap_slot_guarded: pair.bridged_with_the_pointer_birth && !read_bad_gap,
            read_bad_or_zero_gap_slot_guarded: pair.bridged_with_the_pointer_birth
                && !read_bad_gap
                && !zero_gap,
            accounting_generation_guarded,
        };
        let gap_slots_text = if gap_slots.is_empty() {
            "none".to_string()
        } else {
            gap_slots
                .iter()
                .map(|(txg, state)| format!("{txg}:{}", state.text()))
                .collect::<Vec<_>>()
                .join(",")
        };
        let generations_text = match &generations {
            Ok(found) => found.text(),
            Err(reason) => reason.chars().take(80).collect::<String>().replace(' ', "_"),
        };
        let bit = |value: bool| u8::from(value);
        let text = format!(
            "name=pair {prefix} pair={}>{} truth={} r_pointer_birth={} today={} with_pointer_birth={} read_bad_gap_slot_guarded={} read_bad_or_zero_gap_slot_guarded={} accounting_generation_guarded={} gap_slots={gap_slots_text} gap_generations={generations_text}",
            keys_text(pair.previous_group.iter().copied()),
            pair.version.key.text(),
            truth.name(),
            pointer_birth_text(pair.version.pointer_birth),
            bit(judgement.today),
            bit(judgement.with_pointer_birth),
            bit(judgement.read_bad_gap_slot_guarded),
            bit(judgement.read_bad_or_zero_gap_slot_guarded),
            judgement
                .accounting_generation_guarded
                .map_or("unknown".to_string(), |value| bit(value).to_string()),
        );
        totals.note_pair(&AttackPairJudgement {
            text: text.clone(),
            ..judgement
        });
        pair_texts.push(text);
    }
    let checker_text = if run_the_pool_checker {
        let verdicts = check_pool_image(image);
        let verdict_of = |invariant: &str| {
            verdicts
                .iter()
                .find(|(name, _)| *name == invariant)
                .map(|(_, verdict)| verdict)
        };
        let name_of = |invariant: &str| {
            verdict_of(invariant).map_or("missing", verdict_name).to_string()
        };
        let reference_interval = name_of("I-3.5");
        let release_generation = name_of("I-3.9");
        *totals
            .pool_checker_reference_interval
            .entry(reference_interval.clone())
            .or_insert(0) += 1;
        *totals
            .pool_checker_release_generation
            .entry(release_generation.clone())
            .or_insert(0) += 1;
        let violation_text = match verdict_of("I-3.5") {
            Some(InvariantVerdict::Violated(text)) => text.chars().take(600).collect::<String>(),
            _ => "-".to_string(),
        };
        format!(
            " reference_interval_verdict={reference_interval} release_generation_verdict={release_generation} reference_interval_violation=「{violation_text}」"
        )
    } else {
        String::new()
    };
    lines.push(format!(
        "name=image {prefix} newest={} floor={} candidates={} applied_only_by_records={} hole_pairs={}{checker_text}",
        reading.newest.text(),
        reading.effective_floor,
        keys_text(reading.candidates.iter().map(|version| version.key)),
        keys_text(reading.applied_only_by_records.iter().map(|version| version.key)),
        reading.hole_pairs.len()
    ));
    lines.extend(pair_texts);
    (lines, totals)
}

/// rs-two-files 回退之后的动作：写 inode 2（第一个文件那个旧数据单元留着）、覆盖写第一个文件（对照：旧单元被换掉）、空发布（对照）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttackTwoFileAction {
    OverwriteTheSecondInode,
    OverwriteTheFirstFile,
    EmptyPublish,
}

impl AttackTwoFileAction {
    fn name(self) -> &'static str {
        match self {
            AttackTwoFileAction::OverwriteTheSecondInode => "second",
            AttackTwoFileAction::OverwriteTheFirstFile => "first",
            AttackTwoFileAction::EmptyPublish => "empty",
        }
    }
}

/// 这份模型的一段历史。
#[derive(Clone, Debug)]
enum AttackHistory {
    SweepReadControl { slots_per_region: u64, first_instance_last: u64, extra_overwrites: u64 },
    SweepCrashControl { slots_per_region: u64, first_instance_last: u64, extra_overwrites: u64 },
    SweepCrashWrapped { slots_per_region: u64, first_instance_last: u64, extra_overwrites: u64, chunks: u64 },
    SkippedRowGapFault { slots_per_region: u64, last: u64 },
    RollbackTwoFiles { target: u64, actions: Vec<AttackTwoFileAction> },
    /// E198 第一段 nrb、rs1 两组原样重造（主点），只换成这份模型的判法：看乙、丁在 C698（读坏一个根槽的镜像上的误红） 那两组上分不分对。
    ApparatusRecheck(HistorySpecification),
}

impl AttackHistory {
    fn group(&self) -> &'static str {
        match self {
            AttackHistory::SweepReadControl { .. } => "sw-read-control",
            AttackHistory::SweepCrashControl { .. } => "sw-crash-control",
            AttackHistory::SweepCrashWrapped { .. } => "sw-crash-wrapped",
            AttackHistory::SkippedRowGapFault { .. } => "skr-gap-fault",
            AttackHistory::RollbackTwoFiles { .. } => "rs-two-files",
            AttackHistory::ApparatusRecheck(specification) => match specification.group {
                Group::NewestRowBad => "nrb-recheck",
                _ => "rs1-recheck",
            },
        }
    }

    fn slots_per_region(&self) -> u64 {
        match self {
            AttackHistory::SweepReadControl { slots_per_region, .. }
            | AttackHistory::SweepCrashControl { slots_per_region, .. }
            | AttackHistory::SweepCrashWrapped { slots_per_region, .. }
            | AttackHistory::SkippedRowGapFault { slots_per_region, .. } => *slots_per_region,
            AttackHistory::RollbackTwoFiles { .. } => 4,
            AttackHistory::ApparatusRecheck(specification) => specification.slots_per_region,
        }
    }

    fn name(&self) -> String {
        let prefix = format!("{}.S{}", self.group(), self.slots_per_region());
        match self {
            AttackHistory::SweepReadControl { first_instance_last, extra_overwrites, .. }
            | AttackHistory::SweepCrashControl { first_instance_last, extra_overwrites, .. } => {
                format!("{prefix}.a{first_instance_last}.d{extra_overwrites}")
            }
            AttackHistory::SweepCrashWrapped { first_instance_last, extra_overwrites, chunks, .. } => {
                format!("{prefix}.a{first_instance_last}.d{extra_overwrites}.c{chunks}")
            }
            AttackHistory::SkippedRowGapFault { last, .. } => format!("{prefix}.last{last}"),
            AttackHistory::RollbackTwoFiles { target, actions } => {
                let names: Vec<&str> = actions.iter().map(|action| action.name()).collect();
                format!("{prefix}.target{target}.actions{}", names.join("+"))
            }
            AttackHistory::ApparatusRecheck(specification) => {
                format!("{}.{}", self.group(), history_name(specification))
            }
        }
    }

    /// 装置的建池参数只读 S 与 journal 环长：形状随便填一个。
    fn builder(&self) -> HistoryBuilder {
        if let AttackHistory::ApparatusRecheck(specification) = self {
            return HistoryBuilder::new(specification);
        }
        HistoryBuilder::new(&HistorySpecification {
            group: Group::SkippedRow,
            point: SamplingPoint::Main,
            slots_per_region: self.slots_per_region(),
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::SkippedRow { last: 0 },
        })
    }
}

/// 全部历史：sw 三组照 E198 的取样域（S ∈ {4, 5, 6}、a ∈ {4, 6, 8}、d ∈ {0, 1, 2}），盖记录那一组另扫 c ∈ {3, 4}；
/// skr 照 E198 主点与 G3（S ∈ {4, 6}、last ∈ {6, 8, 10}）；rs-two-files 的回退目标与动作串在 `attack_rollback_two_file_histories`。
fn attack_histories() -> Vec<AttackHistory> {
    let mut histories = Vec::new();
    for slots_per_region in [4, 5, 6] {
        for first_instance_last in [4, 6, 8] {
            for extra_overwrites in [0, 1, 2] {
                histories.push(AttackHistory::SweepReadControl { slots_per_region, first_instance_last, extra_overwrites });
                histories.push(AttackHistory::SweepCrashControl { slots_per_region, first_instance_last, extra_overwrites });
                for chunks in [3, 4] {
                    histories.push(AttackHistory::SweepCrashWrapped { slots_per_region, first_instance_last, extra_overwrites, chunks });
                }
            }
        }
    }
    for slots_per_region in [4, 6] {
        for last in [6, 8, 10] {
            histories.push(AttackHistory::SkippedRowGapFault { slots_per_region, last });
        }
    }
    histories.extend(attack_rollback_two_file_histories());
    for specification in newest_row_bad_specifications(SamplingPoint::Main, 4)
        .into_iter()
        .chain(rollback_specifications(SamplingPoint::Main, 4, &[5, 6, 7, 8, 9]))
    {
        histories.push(AttackHistory::ApparatusRecheck(specification));
    }
    histories
}

/// rs-two-files：回退目标取 4 … 9（实例 1 在 txg 1 写第一个文件、2 建 inode 2、3 写 inode 2，4 … 10 覆盖写第一个文件；
/// 账本里实际的 txg 由模型打出，与这里推的不同时照实报）；动作串是写 inode 2 那几种与两条对照。
fn attack_rollback_two_file_histories() -> Vec<AttackHistory> {
    use AttackTwoFileAction::{EmptyPublish, OverwriteTheFirstFile, OverwriteTheSecondInode};
    let action_strings: Vec<Vec<AttackTwoFileAction>> = vec![
        vec![OverwriteTheSecondInode],
        vec![OverwriteTheFirstFile],
        vec![EmptyPublish],
        vec![EmptyPublish, OverwriteTheSecondInode],
        vec![OverwriteTheSecondInode, OverwriteTheFirstFile],
        vec![OverwriteTheSecondInode, OverwriteTheSecondInode],
    ];
    let mut histories = Vec::new();
    for target in 4..=9 {
        for actions in &action_strings {
            histories.push(AttackHistory::RollbackTwoFiles { target, actions: actions.clone() });
        }
    }
    histories
}

/// 写 inode 2 那一类发布：建一个 inode，或覆盖写一个不是第一个文件的 inode。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttackSecondInodePublish {
    NewInode,
    OverwriteOfAnInode { inode: u64 },
}

impl HistoryBuilder {
    /// 此刻镜像的一份拷贝，把 `txgs` 每个的落点槽（本地落点公式）第 40 字节取反：不管那一槽住着什么（全零、别的版本的根）。
    fn attack_copy_with_flipped_landing_slots(&self, txgs: &[u64]) -> MemoryPool {
        let mut image = self.image.clone();
        for txg in txgs {
            let (region, slot) = ring_slot_by_the_local_formula(*txg, self.slots_per_region());
            let device = REGION_DEVICES[usize::try_from(region).expect("区域号装得进 usize")];
            let offset = slot_offset(
                RootRingSlot { region, slot },
                self.parameters.geometry.fixed_structure_slot_spacing,
            );
            image.flip_byte(DeviceIdentity(device), offset, CORRUPTED_BYTE_IN_THE_ROOT_SLOT);
        }
        image
    }

    /// 建 inode 或写一个别的 inode：与装置的 `publish` 同一套落盘与记账本，发布路径记成新 inode 或覆盖写。
    fn attack_publish_on_a_second_inode(
        &mut self,
        session: &mut Session,
        publish: AttackSecondInodePublish,
    ) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let instance = session.instance;
        let current = session.current.clone();
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let salt = current.root().checkpoint_txg.0 + 1;
            let write_time_seconds = WRITE_TIME_SECONDS + salt;
            let previous = current
                .file_version()
                .ok_or_else(|| "现行那一版没有文件".to_string())?;
            let published = match publish {
                AttackSecondInodePublish::NewInode => publish_new_inodes(
                    &mut writer,
                    allocator,
                    previous,
                    1,
                    write_time_seconds,
                    instance,
                ),
                AttackSecondInodePublish::OverwriteOfAnInode { inode } => {
                    let content =
                        deterministic_content(salt.wrapping_add(1_000_000), OVERWRITE_CONTENT_BYTES);
                    publish_overwrite_of_an_inode(
                        &mut writer,
                        allocator,
                        previous,
                        InodeNumber(inode),
                        &content,
                        write_time_seconds,
                        instance,
                    )
                }
            };
            published.map_err(|error| format!("{publish:?}：{error:?}"))
        });
        let output = result?;
        self.note_returned_roots(&[output.root]);
        let kind = match publish {
            AttackSecondInodePublish::NewInode => PublishKind::NewInode,
            AttackSecondInodePublish::OverwriteOfAnInode { .. } => PublishKind::Overwrite,
        };
        let kind_of = move |_key: VersionKey| kind;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let last = self
            .land_operations(&operations, &context, Some(session.last_key))
            .ok_or_else(|| format!("{publish:?} 那一次发布的写流里没有根槽写"))?;
        session.current = PoolVersion::of_a_version_of_trees(output);
        session.last_key = last;
        Ok(last)
    }
}

/// 一段历史的结果行与计数。
struct AttackHistoryOutcome {
    lines: Vec<String>,
    totals: AttackArmTotals,
}

/// 收一张镜像的判定进这段历史的结果。
fn attack_collect(
    outcome: &mut AttackHistoryOutcome,
    image: &MemoryPool,
    ledger: &Ledger,
    slots_per_region: u64,
    prefix: String,
    run_the_pool_checker: bool,
) {
    let (lines, totals) =
        attack_judge_the_image(image, ledger, slots_per_region, &prefix, run_the_pool_checker);
    outcome.lines.extend(lines);
    outcome.totals.add(&totals);
}

/// sw 三组（照装置 `build_the_sweep` 不攒行那一支）：实例 1 到 a → 实例 2 覆盖写到 b = a + 3S + d → 实例 3 挂载（写行、暖机、清扫）→
/// sw-read：读坏实例 3 在清扫那一版之前每一版的根槽、每一步一张；sw-crash：覆盖写（盖记录那一组改顺序写 c × 524 288 字节）到 b + 3S − 1，
/// 下一次覆盖写崩在根槽写之前 → 实例 4 挂载 → 覆盖写 k 次，k = 0 … 3S 每个 k 一张。
fn attack_build_the_sweep(
    builder: &mut HistoryBuilder,
    outcome: &mut AttackHistoryOutcome,
    history_prefix: &str,
    first_instance_last: u64,
    extra_overwrites: u64,
    sweep_instance_write: Option<UserWrite>,
) -> Result<(), String> {
    let configuration = MountConfiguration::SweepAtTheEndAboveTheThreshold {
        threshold: SWEEP_THRESHOLD_ANY_ROW,
    };
    let slots_per_region = builder.slots_per_region();
    let ring_slots = 3 * slots_per_region;
    build_the_first_instance(builder, configuration, first_instance_last, UserWrite::Overwrite)?;
    let (mut second, _) = builder.mount(configuration)?;
    let last_txg_before_the_sweep_instance = first_instance_last + ring_slots + extra_overwrites;
    write_until(
        builder,
        &mut second,
        last_txg_before_the_sweep_instance,
        UserWrite::Overwrite,
    )?;
    drop(second);
    let (mut sweep_session, sweep_summary) = builder.mount(configuration)?;
    outcome.lines.push(format!(
        "name=sweep {history_prefix} sweep_instance={} sweep_outcome={} sweep_version={}",
        sweep_summary.instance,
        sweep_summary.sweep_outcome,
        sweep_summary
            .sweep_version
            .map_or("none".to_string(), VersionKey::text)
    ));
    let last_before_the_crash = last_txg_before_the_sweep_instance + ring_slots - 1;
    match sweep_instance_write {
        None => {
            let first_version_on_the_image = sweep_session.last_key.txg;
            let bad: Vec<VersionKey> = builder
                .ledger
                .versions
                .keys()
                .filter(|key| {
                    key.instance == sweep_summary.instance && key.txg < first_version_on_the_image
                })
                .copied()
                .collect();
            let mut step = 0;
            loop {
                let (image, ledger, _skipped) = builder.copy_with_corrupted_root_slots(&bad);
                attack_collect(
                    outcome,
                    &image,
                    &ledger,
                    slots_per_region,
                    format!(
                        "{history_prefix} image=version_txg={} step={step}",
                        sweep_session.last_key.txg
                    ),
                    false,
                );
                if sweep_session.last_key.txg >= last_before_the_crash {
                    break;
                }
                builder.publish(&mut sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
                step += 1;
            }
        }
        Some(user_write) => {
            write_until(builder, &mut sweep_session, last_before_the_crash, user_write)?;
            builder.publish_crashing(sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
            let (mut next_session, _) = builder.mount(configuration)?;
            for step in 0..=ring_slots {
                if step > 0 {
                    builder.publish(&mut next_session, Publish::UserWrite(UserWrite::Overwrite))?;
                }
                attack_collect(
                    outcome,
                    &builder.image,
                    &builder.ledger,
                    slots_per_region,
                    format!("{history_prefix} image=publishes_after_the_mount={step} step={step}"),
                    false,
                );
            }
        }
    }
    Ok(())
}

/// skr（照装置 `build_the_skipped_row`）：实例 1 到 last → 实例 2 产品挂载崩在写行那次根槽写之前（那一版 txg = last + 1）→ 实例 3 产品挂载 →
/// 覆盖写 k 次，k = 0 … 14；每个 k 两张：原样一张，洞里那个 txg（last + 1）的落点槽第 40 字节取反一张（读坏一个根槽，用户 2026-10-04 定算合法镜像）。
fn attack_build_the_skipped_row_with_a_gap_fault(
    builder: &mut HistoryBuilder,
    outcome: &mut AttackHistoryOutcome,
    history_prefix: &str,
    last: u64,
) -> Result<(), String> {
    let slots_per_region = builder.slots_per_region();
    build_the_first_instance(builder, MountConfiguration::Product, last, UserWrite::Overwrite)?;
    let (_, crashed) =
        builder.mount_crashing_at_the_first_root_slot_write(MountConfiguration::Product)?;
    outcome.lines.push(format!(
        "name=crashed_row_publish {history_prefix} version={}",
        crashed.text()
    ));
    let (mut session, _) = builder.mount(MountConfiguration::Product)?;
    for step in 0..=14 {
        if step > 0 {
            builder.publish(&mut session, Publish::UserWrite(UserWrite::Overwrite))?;
        }
        attack_collect(
            outcome,
            &builder.image,
            &builder.ledger,
            slots_per_region,
            format!("{history_prefix} image=publishes_after_the_mount={step}.clean step={step}"),
            false,
        );
        let faulted = builder.attack_copy_with_flipped_landing_slots(&[crashed.txg]);
        attack_collect(
            outcome,
            &faulted,
            &builder.ledger,
            slots_per_region,
            format!("{history_prefix} image=publishes_after_the_mount={step}.gap_fault step={step}"),
            false,
        );
    }
    Ok(())
}

/// rs-two-files（照装置 `build_the_rollback_slot_bad`，第一个实例多一个文件）：mkfs → 实例 1 挂载 → 第一个文件 → 建 inode 2 → 写 inode 2 →
/// 覆盖写第一个文件到 txg 10 → 实例 2 产品挂载（写行 r）→ 回退到 (实例 1, target)（那一版 v）→ 动作串 →
/// R ∈ {v, …, 最后一版} 每个一张：读坏实例 2 在 [r, R) 里每一版的根槽，跑今天的池级 checker；另出一张一槽都不读坏的（对照）。
fn attack_build_the_rollback_with_two_files(
    builder: &mut HistoryBuilder,
    outcome: &mut AttackHistoryOutcome,
    history_prefix: &str,
    target: u64,
    actions: &[AttackTwoFileAction],
) -> Result<(), String> {
    let slots_per_region = builder.slots_per_region();
    builder.make_filesystem()?;
    let (mut first, _) = builder.mount(MountConfiguration::Product)?;
    let first_file = builder.publish(&mut first, Publish::FirstFile)?;
    let second_inode = first
        .current
        .file_version()
        .map(TransactionOutput::inode_number_watermark)
        .ok_or_else(|| "第一个文件那一版没有文件".to_string())?;
    let created = builder.attack_publish_on_a_second_inode(&mut first, AttackSecondInodePublish::NewInode)?;
    let written = builder.attack_publish_on_a_second_inode(
        &mut first,
        AttackSecondInodePublish::OverwriteOfAnInode { inode: second_inode },
    )?;
    write_until(builder, &mut first, ROLLBACK_FIRST_INSTANCE_LAST, UserWrite::Overwrite)?;
    drop(first);
    outcome.lines.push(format!(
        "name=first_instance {history_prefix} first_file={} second_inode={second_inode} created={} second_inode_written={}",
        first_file.text(),
        created.text(),
        written.text()
    ));
    if target < written.txg {
        return Err(format!(
            "target_{target}_is_before_the_second_inode_is_written_at_{}",
            written.txg
        ));
    }
    let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
    let row_publish = summary
        .row_publish
        .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
    let rollback = builder.roll_back(&mut session, target)?;
    for action in actions {
        match action {
            AttackTwoFileAction::OverwriteTheSecondInode => {
                builder.attack_publish_on_a_second_inode(
                    &mut session,
                    AttackSecondInodePublish::OverwriteOfAnInode { inode: second_inode },
                )?;
            }
            AttackTwoFileAction::OverwriteTheFirstFile => {
                builder.publish(&mut session, Publish::UserWrite(UserWrite::Overwrite))?;
            }
            AttackTwoFileAction::EmptyPublish => {
                builder.publish(
                    &mut session,
                    Publish::RollbackAction {
                        action: RollbackAction::EmptyPublish,
                        target,
                    },
                )?;
            }
        }
    }
    let last_txg = session.last_key.txg;
    outcome.lines.push(format!(
        "name=second_instance {history_prefix} row_publish={} rollback={} last={}",
        row_publish.text(),
        rollback.text(),
        session.last_key.text()
    ));
    attack_collect(
        outcome,
        &builder.image,
        &builder.ledger,
        slots_per_region,
        format!("{history_prefix} image=no_slot_read_bad step=none"),
        true,
    );
    for (step, first_readable_txg) in (rollback.txg..=last_txg).enumerate() {
        let bad: Vec<VersionKey> = builder
            .ledger
            .versions
            .keys()
            .filter(|key| {
                key.instance == row_publish.instance
                    && key.txg >= row_publish.txg
                    && key.txg < first_readable_txg
            })
            .copied()
            .collect();
        let (image, ledger, _skipped) = builder.copy_with_corrupted_root_slots(&bad);
        attack_collect(
            outcome,
            &image,
            &ledger,
            slots_per_region,
            format!("{history_prefix} image=R={first_readable_txg} step={step}"),
            true,
        );
    }
    Ok(())
}

/// 建一段历史、判每一张镜像。写者返回 `Err` 是合法结局，记在历史行里，这段到此为止。
fn attack_run_the_history(history: &AttackHistory) -> AttackHistoryOutcome {
    let mut builder = history.builder();
    let mut outcome = AttackHistoryOutcome {
        lines: Vec::new(),
        totals: AttackArmTotals {
            histories: 1,
            ..AttackArmTotals::default()
        },
    };
    let prefix = format!("group={} history={}", history.group(), history.name());
    let result = match history {
        AttackHistory::SweepReadControl { first_instance_last, extra_overwrites, .. } => {
            attack_build_the_sweep(&mut builder, &mut outcome, &prefix, *first_instance_last, *extra_overwrites, None)
        }
        AttackHistory::SweepCrashControl { first_instance_last, extra_overwrites, .. } => {
            attack_build_the_sweep(&mut builder, &mut outcome, &prefix, *first_instance_last, *extra_overwrites, Some(UserWrite::Overwrite))
        }
        AttackHistory::SweepCrashWrapped { first_instance_last, extra_overwrites, chunks, .. } => {
            let content_bytes = usize::try_from(*chunks).expect("块数装得进 usize") * SEQUENTIAL_WRITE_CHUNK_BYTES;
            attack_build_the_sweep(&mut builder, &mut outcome, &prefix, *first_instance_last, *extra_overwrites, Some(UserWrite::SequentialWrite { content_bytes }))
        }
        AttackHistory::SkippedRowGapFault { last, .. } => {
            attack_build_the_skipped_row_with_a_gap_fault(&mut builder, &mut outcome, &prefix, *last)
        }
        AttackHistory::RollbackTwoFiles { target, actions } => {
            attack_build_the_rollback_with_two_files(&mut builder, &mut outcome, &prefix, *target, actions)
        }
        AttackHistory::ApparatusRecheck(specification) => {
            attack_build_the_apparatus_shape(&mut builder, &mut outcome, &prefix, &specification.shape)
        }
    };
    let stopped = result.err();
    if stopped.is_some() {
        outcome.totals.stopped_histories = 1;
    }
    outcome.lines.insert(
        0,
        format!(
            "name=history {prefix} stopped={}",
            stopped.map_or("none".to_string(), |reason| reason.chars().take(200).collect::<String>().replace(' ', "_"))
        ),
    );
    outcome
}

/// 估全量挂钟的上限（攻方腿定义第 3b 步：估出来超过 40 分钟就停，先缩取样）。
const ATTACK_ESTIMATE_LIMIT_SECONDS: f64 = 2400.0;

/// 并行跑一串历史（装置的线程数取法），结果按下标交回。
fn attack_run_in_parallel(histories: &[AttackHistory]) -> Vec<AttackHistoryOutcome> {
    let threads = worker_threads().max(1);
    let next = AtomicUsize::new(0);
    let results: Mutex<BTreeMap<usize, AttackHistoryOutcome>> = Mutex::new(BTreeMap::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.min(histories.len().max(1)) {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(history) = histories.get(index) else {
                    break;
                };
                let started = std::time::Instant::now();
                let outcome = attack_run_the_history(history);
                eprintln!(
                    "progress history={} seconds={:.3}",
                    history.name(),
                    started.elapsed().as_secs_f64()
                );
                results.lock().expect("结果表的锁没中毒").insert(index, outcome);
            });
        }
    });
    results
        .into_inner()
        .expect("结果表的锁没中毒")
        .into_values()
        .collect()
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let full = arguments.iter().any(|argument| argument == "full");
    let histories = attack_histories();
    let threads = worker_threads().max(1);
    // 先每组跑一段（各组第一段），按它估全量挂钟。
    let mut first_of_each_group: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut count_of_each_group: BTreeMap<&'static str, u64> = BTreeMap::new();
    for (index, history) in histories.iter().enumerate() {
        first_of_each_group.entry(history.group()).or_insert(index);
        *count_of_each_group.entry(history.group()).or_insert(0) += 1;
    }
    let mut estimated_serial_seconds = 0.0;
    for (group, index) in &first_of_each_group {
        let started = std::time::Instant::now();
        let probe = attack_run_the_history(&histories[*index]);
        let seconds = started.elapsed().as_secs_f64();
        let count = count_of_each_group[group];
        estimated_serial_seconds += seconds * count as f64;
        println!(
            "OPUSMODEL name=probe group={group} history={} probe_images={} histories_in_group={count}",
            histories[*index].name(),
            probe.totals.images
        );
        eprintln!("probe group={group} seconds={seconds:.3} histories_in_group={count}");
    }
    let estimated_seconds = estimated_serial_seconds / threads as f64;
    eprintln!(
        "estimate histories={} threads={threads} estimated_wall_clock_seconds={estimated_seconds:.1} limit_seconds={ATTACK_ESTIMATE_LIMIT_SECONDS}",
        histories.len()
    );
    println!("OPUSMODEL name=estimate histories={} within_the_limit={}", histories.len(), estimated_seconds <= ATTACK_ESTIMATE_LIMIT_SECONDS);
    if !full {
        println!("OPUSMODEL name=done mode=probe_only");
        return;
    }
    if estimated_seconds > ATTACK_ESTIMATE_LIMIT_SECONDS {
        println!("OPUSMODEL name=done mode=stopped_over_the_estimate_limit");
        std::process::exit(3);
    }
    let outcomes = attack_run_in_parallel(&histories);
    let mut totals_of_each_group: BTreeMap<&'static str, AttackArmTotals> = BTreeMap::new();
    for (history, outcome) in histories.iter().zip(&outcomes) {
        for line in &outcome.lines {
            println!("OPUSMODEL {line}");
        }
        totals_of_each_group
            .entry(history.group())
            .or_default()
            .add(&outcome.totals);
    }
    for (group, totals) in &totals_of_each_group {
        println!("OPUSMODEL name=group group={group} {}", totals.text());
    }
    println!("OPUSMODEL name=done mode=full histories={}", histories.len());
}

/// nrb、rs1 原样重造（照装置 `build_the_newest_row_bad` 与 `build_the_rollback_slot_bad`），每张镜像换成这份模型的判法；不跑池级 checker
/// （那两组的 I-3.5、I-3.9 读数在 E198 第一段产物里，这里只看五条臂的分类）。
fn attack_build_the_apparatus_shape(
    builder: &mut HistoryBuilder,
    outcome: &mut AttackHistoryOutcome,
    history_prefix: &str,
    shape: &HistoryShape,
) -> Result<(), String> {
    let slots_per_region = builder.slots_per_region();
    match shape {
        HistoryShape::NewestRowBad { last, steps_after_the_mount, bad_slots, user_write } => {
            build_the_first_instance(builder, MountConfiguration::Product, *last, *user_write)?;
            let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
            let row_publish = summary
                .row_publish
                .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
            for _ in 0..*steps_after_the_mount {
                builder.publish(&mut session, Publish::UserWrite(*user_write))?;
            }
            let bad: Vec<VersionKey> = (row_publish.txg..row_publish.txg + *bad_slots)
                .map(|txg| VersionKey { instance: row_publish.instance, txg })
                .collect();
            let (image, ledger, _skipped) = builder.copy_with_corrupted_root_slots(&bad);
            attack_collect(
                outcome,
                &image,
                &ledger,
                slots_per_region,
                format!("{history_prefix} image=bad={} step=0", keys_text(bad.iter().copied())),
                false,
            );
            Ok(())
        }
        HistoryShape::RollbackSlotBad { target, actions } => {
            build_the_first_instance(
                builder,
                MountConfiguration::Product,
                ROLLBACK_FIRST_INSTANCE_LAST,
                UserWrite::Overwrite,
            )?;
            let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
            let row_publish = summary
                .row_publish
                .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
            let rollback = builder.roll_back(&mut session, *target)?;
            for action in actions {
                builder.publish(
                    &mut session,
                    Publish::RollbackAction { action: *action, target: *target },
                )?;
            }
            let last_txg = session.last_key.txg;
            for (step, first_readable_txg) in (rollback.txg..=last_txg).enumerate() {
                let bad: Vec<VersionKey> = builder
                    .ledger
                    .versions
                    .keys()
                    .filter(|key| {
                        key.instance == row_publish.instance
                            && key.txg >= row_publish.txg
                            && key.txg < first_readable_txg
                    })
                    .copied()
                    .collect();
                let (image, ledger, _skipped) = builder.copy_with_corrupted_root_slots(&bad);
                attack_collect(
                    outcome,
                    &image,
                    &ledger,
                    slots_per_region,
                    format!("{history_prefix} image=R={first_readable_txg} step={step}"),
                    false,
                );
            }
            Ok(())
        }
        _ => Err("这份模型只重造 nrb、rs1 两种形状".to_string()),
    }
}
