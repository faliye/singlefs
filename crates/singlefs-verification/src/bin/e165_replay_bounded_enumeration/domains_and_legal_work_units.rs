//! 域（登记 5.2）的族与钉住的状态数，合法历史一族的工作单元与按历史的计数，C 域（记录子集与根槽子集）与多盘域（登记 r2 5.3）。

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Family {
    LegalHistory,
    Adversarial,
    /// 登记 r2 5.3 的多盘域 M、M6、MC：只在 `--profile r2 --family multi_disk` 时跑。
    MultiDisk,
}

impl Domain {
    pub(super) const fn ring(self) -> RingGeometry {
        match self {
            Self::LegalBigRing
            | Self::NamedUnitsVerified
            | Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::AdversarialBigRing
            | Self::AdversarialBackChain
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::Residue
            | Self::MirroredBigRing
            | Self::MirroredMonotoneBigRing => RingGeometry::BIG,
            Self::LegalSmallRingSix
            | Self::MonotoneSmallRingSix
            | Self::AdversarialSmallRingSix
            | Self::MirroredSmallRingSix => RingGeometry::SMALL_SIX,
            Self::LegalSmallRingNine => RingGeometry::SMALL_NINE,
        }
    }

    pub(super) const fn family(self) -> Family {
        match self {
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::NamedUnitsVerified
            | Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::Residue => Family::LegalHistory,
            Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain => Family::Adversarial,
            Self::MirroredBigRing | Self::MirroredSmallRingSix | Self::MirroredMonotoneBigRing => {
                Family::MultiDisk
            }
        }
    }

    /// `--only` 认的名字：U 同时选中验证开、关两遍。
    pub(super) const fn selection_name(self) -> &'static str {
        match self {
            Self::NamedUnitsVerified | Self::NamedUnitsNotVerified => "U",
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::Residue
            | Self::MirroredBigRing
            | Self::MirroredSmallRingSix
            | Self::MirroredMonotoneBigRing => self.label(),
        }
    }

    /// 这个域里最小的 N（或 Nc、n）。
    pub(super) const fn smallest_size(self) -> usize {
        match self {
            Self::CrossInstance | Self::Residue => 2,
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::NamedUnitsVerified
            | Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::MirroredBigRing
            | Self::MirroredSmallRingSix
            | Self::MirroredMonotoneBigRing => 1,
        }
    }

    /// 这个域的状态进不进「S 自己的对照」（登记 5.4：L、L 小环、U 开着验证、X、E1）。
    pub(super) const fn specification_self_check(self) -> SpecificationSelfCheck {
        match self {
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::NamedUnitsVerified
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest => SpecificationSelfCheck::Judged,
            Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain
            | Self::TxgAtTheLargest
            | Self::Residue
            | Self::MirroredBigRing
            | Self::MirroredSmallRingSix
            | Self::MirroredMonotoneBigRing => SpecificationSelfCheck::NotJudged,
        }
    }
}

/// 每个域每个 N 的状态数（登记第七节乙类锚点，出自登记里那份脚本的闭式与暴力枚举；下标 0 对应这个域最小的 N）。
pub(super) const fn pinned_state_counts(domain: Domain) -> &'static [u64] {
    match domain {
        Domain::LegalBigRing => &[
            2, 12, 72, 432, 2_592, 15_552, 93_312, 559_872, 3_359_232, 20_155_392,
        ],
        Domain::LegalSmallRingSix => &[2, 12, 40, 176, 672, 2_752],
        Domain::LegalSmallRingNine => &[2, 12, 72, 304, 1_568, 7_872, 38_016, 189_184, 934_400],
        Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => {
            &[6, 108, 1_944, 34_992, 629_856, 11_337_408]
        }
        Domain::MonotoneBigRing => &[4, 32, 256, 2_048, 16_384, 131_072],
        Domain::MonotoneSmallRingSix => &[4, 32, 192, 1_280, 8_192],
        Domain::AdversarialBigRing => &[146, 10_658, 778_034, 56_796_482],
        Domain::AdversarialSmallRingSix => &[146, 10_658, 778_034],
        Domain::AdversarialBackChain => &[290, 42_050, 6_097_250],
        Domain::CrossInstance => &[4, 48, 432, 3_456, 25_920, 186_624],
        Domain::TxgOneBelowTheLargest => &[2, 12, 64, 320],
        Domain::TxgAtTheLargest => &[2, 8, 32],
        Domain::Residue => &[4, 64, 688, 6_208],
        // 登记 r2 第七节乙类：M 是 4 × 12^(N − 1)；M6 每次发布至多 2 条；MC 是 Nc = 1..4。
        Domain::MirroredBigRing => &[4, 48, 576, 6_912, 82_944, 995_328],
        Domain::MirroredSmallRingSix => &[4, 48, 320, 2_816, 21_504, 176_128],
        Domain::MirroredMonotoneBigRing => &[8, 128, 2_048, 32_768],
    }
}

/// L 域的见证（j\* ≥ 1 的态数, j\* ≥ 2 的态数），N = 1..7（登记第七节乙类锚点：照条款写的 j\* 的分布，不是实现的结果）。
pub(super) const PINNED_FARTHEST_VERSION_WITNESSES_OF_THE_BIG_RING: [(u64, u64); 7] = [
    (0, 0),
    (2, 0),
    (16, 2),
    (104, 20),
    (640, 144),
    (3_872, 928),
    (23_296, 5_728),
];

/// C 域每个 Nc 的锚点（登记第七节乙类；小环的「甲合法」「乙合法」两组是登记第十二节补的）。
pub(super) struct PinnedMonotoneCounts {
    pub(super) wide_pairs: u64,
    pub(super) cut_own_legal_states: u64,
    pub(super) cut_own_legal_pairs: u64,
    pub(super) cut_own_same_segment_pairs: u64,
    pub(super) cut_with_legal_states: u64,
    pub(super) cut_with_legal_pairs: u64,
    pub(super) cut_with_same_segment_pairs: u64,
}

pub(super) const fn pinned_monotone_counts(domain: Domain) -> &'static [PinnedMonotoneCounts] {
    const fn row(
        wide_pairs: u64,
        cut_own: (u64, u64, u64),
        cut_with: (u64, u64, u64),
    ) -> PinnedMonotoneCounts {
        PinnedMonotoneCounts {
            wide_pairs,
            cut_own_legal_states: cut_own.0,
            cut_own_legal_pairs: cut_own.1,
            cut_own_same_segment_pairs: cut_own.2,
            cut_with_legal_states: cut_with.0,
            cut_with_legal_pairs: cut_with.1,
            cut_with_same_segment_pairs: cut_with.2,
        }
    }
    const BIG_RING: [PinnedMonotoneCounts; 6] = [
        row(9, (3, 6, 5), (4, 9, 9)),
        row(135, (15, 43, 31), (23, 80, 71)),
        row(2_025, (71, 280, 189), (118, 600, 489)),
        row(30_375, (323, 1_721, 1_143), (565, 4_075, 3_159)),
        row(455_625, (1_427, 10_210, 6_885), (2_584, 25_995, 19_737)),
        row(
            6_834_375,
            (6_167, 59_319, 41_391),
            (11_443, 159_266, 121_095),
        ),
    ];
    const SMALL_RING_SIX: [PinnedMonotoneCounts; 5] = [
        row(9, (3, 6, 5), (4, 9, 9)),
        row(135, (15, 43, 31), (23, 80, 71)),
        row(1_701, (35, 136, 73), (54, 276, 165)),
        row(22_599, (99, 481, 215), (157, 1_059, 503)),
        row(295_245, (231, 1_354, 505), (368, 3_083, 1_181)),
    ];
    match domain {
        Domain::MonotoneBigRing => &BIG_RING,
        Domain::MonotoneSmallRingSix => &SMALL_RING_SIX,
        Domain::LegalBigRing
        | Domain::LegalSmallRingSix
        | Domain::LegalSmallRingNine
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain
        | Domain::CrossInstance
        | Domain::TxgOneBelowTheLargest
        | Domain::TxgAtTheLargest
        | Domain::Residue
        | Domain::MirroredBigRing
        | Domain::MirroredSmallRingSix
        | Domain::MirroredMonotoneBigRing => &[],
    }
}

/// M、M6 每个 N 的锚点（登记 r2 第七节乙类，`anchors_r2.py`、`anchors_r2b.py`、`anchors_r2c.py`）：三类（相等, 落后, 互不包含）、
/// 落后的一个方向（0 号盘落后与 1 号盘落后两行相等）、见证（j\* ≥ 1, j\* ≥ 2；记录任一份在即在）。
pub(super) struct PinnedMirroredCounts {
    pub(super) equal_states: u64,
    pub(super) lagging_states: u64,
    pub(super) crossed_states: u64,
    pub(super) lagging_states_in_one_direction: u64,
    pub(super) farthest_at_least_one: u64,
    pub(super) farthest_at_least_two: u64,
}

pub(super) const fn pinned_mirrored_counts(domain: Domain) -> &'static [PinnedMirroredCounts] {
    const fn row(
        classes: (u64, u64, u64),
        lagging_states_in_one_direction: u64,
        witnesses: (u64, u64),
    ) -> PinnedMirroredCounts {
        PinnedMirroredCounts {
            equal_states: classes.0,
            lagging_states: classes.1,
            crossed_states: classes.2,
            lagging_states_in_one_direction,
            farthest_at_least_one: witnesses.0,
            farthest_at_least_two: witnesses.1,
        }
    }
    const MIRRORED_BIG_RING: [PinnedMirroredCounts; 6] = [
        row((2, 2, 0), 1, (0, 0)),
        row((12, 30, 6), 15, (12, 0)),
        row((72, 342, 162), 171, (216, 36)),
        row((432, 3_510, 2_970), 1_755, (3_024, 864)),
        row((2_592, 34_182, 46_170), 17_091, (38_880, 14_256)),
        row((15_552, 323_190, 656_586), 161_595, (482_112, 202_176)),
    ];
    const MIRRORED_SMALL_RING_SIX: [PinnedMirroredCounts; 6] = [
        row((2, 2, 0), 1, (0, 0)),
        row((12, 30, 6), 15, (12, 0)),
        row((40, 190, 90), 95, (216, 36)),
        row((176, 1_430, 1_210), 715, (1_824, 432)),
        row((672, 8_862, 11_970), 4_431, (14_208, 2_880)),
        row((2_752, 57_190, 116_186), 28_595, (115_200, 25_344)),
    ];
    match domain {
        Domain::MirroredBigRing => &MIRRORED_BIG_RING,
        Domain::MirroredSmallRingSix => &MIRRORED_SMALL_RING_SIX,
        Domain::LegalBigRing
        | Domain::LegalSmallRingSix
        | Domain::LegalSmallRingNine
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::MonotoneBigRing
        | Domain::MonotoneSmallRingSix
        | Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain
        | Domain::CrossInstance
        | Domain::TxgOneBelowTheLargest
        | Domain::TxgAtTheLargest
        | Domain::Residue
        | Domain::MirroredMonotoneBigRing => &[],
    }
}

/// MC 每个 Nc 的锚点（登记 r2 第七节乙类）：宽对数；见证（所选根不是 R_0 的态数, j\* − c ≥ 1, j\* − c ≥ 2）。
pub(super) struct PinnedMirroredMonotoneCounts {
    pub(super) wide_pairs: u64,
    pub(super) chosen_root_is_a_later_version: u64,
    pub(super) farthest_at_least_one: u64,
    pub(super) farthest_at_least_two: u64,
}

pub(super) const fn pinned_mirrored_monotone_counts(
    domain: Domain,
) -> &'static [PinnedMirroredMonotoneCounts] {
    const fn row(wide_pairs: u64, witnesses: (u64, u64, u64)) -> PinnedMirroredMonotoneCounts {
        PinnedMirroredMonotoneCounts {
            wide_pairs,
            chosen_root_is_a_later_version: witnesses.0,
            farthest_at_least_one: witnesses.1,
            farthest_at_least_two: witnesses.2,
        }
    }
    const MIRRORED_MONOTONE_BIG_RING: [PinnedMirroredMonotoneCounts; 4] = [
        row(27, (4, 3, 0)),
        row(1_215, (80, 42, 9)),
        row(54_675, (1_472, 636, 180)),
        row(2_460_375, (25_856, 9_960, 2_988)),
    ];
    match domain {
        Domain::MirroredMonotoneBigRing => &MIRRORED_MONOTONE_BIG_RING,
        Domain::LegalBigRing
        | Domain::LegalSmallRingSix
        | Domain::LegalSmallRingNine
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::MonotoneBigRing
        | Domain::MonotoneSmallRingSix
        | Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain
        | Domain::CrossInstance
        | Domain::TxgOneBelowTheLargest
        | Domain::TxgAtTheLargest
        | Domain::Residue
        | Domain::MirroredBigRing
        | Domain::MirroredSmallRingSix => &[],
    }
}

/// 位数为 `bit_count` 的全部子集掩码，空集在内。
pub(super) fn every_subset_mask(bit_count: usize) -> Range<u32> {
    0..(1u32 << bit_count)
}

fn bit_is_set(mask: u32, bit: usize) -> bool {
    (mask >> bit) & 1 == 1
}

// ── 合法历史一族的工作单元

/// 一段历史就是一个工作单元；R 域另带残留落在哪一条记录上。
#[derive(Clone, Debug)]
pub(super) struct LegalWorkUnit {
    pub(super) plan: HistoryPlan,
    residue_record_index: Option<usize>,
}

pub(super) fn history_plan(
    domain: Domain,
    first_txg: u64,
    chosen_instance_publishes: Vec<PublishShape>,
    next_instance_publishes: Vec<PublishShape>,
    run_profile: RunProfile,
) -> HistoryPlan {
    HistoryPlan {
        run_profile,
        ring: domain.ring(),
        first_txg,
        chosen_instance_publishes,
        next_instance_publishes,
        names_units: match domain {
            Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => true,
            Domain::LegalBigRing
            | Domain::LegalSmallRingSix
            | Domain::LegalSmallRingNine
            | Domain::MonotoneBigRing
            | Domain::MonotoneSmallRingSix
            | Domain::AdversarialBigRing
            | Domain::AdversarialSmallRingSix
            | Domain::AdversarialBackChain
            | Domain::CrossInstance
            | Domain::TxgOneBelowTheLargest
            | Domain::TxgAtTheLargest
            | Domain::Residue
            | Domain::MirroredBigRing
            | Domain::MirroredSmallRingSix
            | Domain::MirroredMonotoneBigRing => false,
        },
    }
}

/// 一个合法历史域在记录总数 `size` 上的全部工作单元（登记 5.2 每个域的定义）。
pub(super) fn legal_work_units(
    domain: Domain,
    size: usize,
    run_profile: RunProfile,
) -> Vec<LegalWorkUnit> {
    let plain = |first_txg: u64, sequences: Vec<Vec<PublishShape>>| -> Vec<LegalWorkUnit> {
        sequences
            .into_iter()
            .map(|sequence| LegalWorkUnit {
                plan: history_plan(domain, first_txg, sequence, Vec::new(), run_profile),
                residue_record_index: None,
            })
            .collect()
    };
    let in_flight_limit = domain.ring().local_in_flight_limit();
    match domain {
        // M：同 L，每条记录在两块盘上各一份（登记 r2 5.3）；落盘子集在 [`tally_of_a_mirrored_history`] 里两盘各取。
        Domain::LegalBigRing
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::MirroredBigRing => plain(ORDINARY_FIRST_TXG, publish_sequences_of(size, None)),
        // 小环：每次发布不超过上限条（合法写侧一次发布最多切上限条）；M6 同 L6。
        Domain::LegalSmallRingSix | Domain::LegalSmallRingNine | Domain::MirroredSmallRingSix => {
            plain(
                ORDINARY_FIRST_TXG,
                publish_sequences_of(size, Some(in_flight_limit)),
            )
        }
        // X：实例 g 的一段合法历史之后接实例 g + 1 的一段合法历史，两段各至少 1 条。
        Domain::CrossInstance => {
            let mut units = Vec::new();
            for chosen_instance_records in 1..size {
                for chosen_sequence in publish_sequences_of(chosen_instance_records, None) {
                    for next_sequence in publish_sequences_of(size - chosen_instance_records, None)
                    {
                        units.push(LegalWorkUnit {
                            plan: history_plan(
                                domain,
                                ORDINARY_FIRST_TXG,
                                chosen_sequence.clone(),
                                next_sequence,
                                run_profile,
                            ),
                            residue_record_index: None,
                        });
                    }
                }
            }
            units
        }
        // E1：t_0 = u64 最大值 − 1、至多两次发布。
        Domain::TxgOneBelowTheLargest => plain(
            u64::MAX - 1,
            publish_sequences_of(size, None)
                .into_iter()
                .filter(|sequence| sequence.len() <= 2)
                .collect(),
        ),
        // E2：t_0 = u64 最大值、只有 P_0。
        Domain::TxgAtTheLargest => plain(
            u64::MAX,
            publish_sequences_of(size, None)
                .into_iter()
                .filter(|sequence| sequence.len() == 1)
                .collect(),
        ),
        // R：至少两次发布；残留落在 P_0 之后的哪一条全扫。
        Domain::Residue => {
            let mut units = Vec::new();
            for sequence in publish_sequences_of(size, None) {
                if sequence.len() < 2 {
                    continue;
                }
                for residue_record_index in sequence[0].record_count..size {
                    units.push(LegalWorkUnit {
                        plan: history_plan(
                            domain,
                            ORDINARY_FIRST_TXG,
                            sequence.clone(),
                            Vec::new(),
                            run_profile,
                        ),
                        residue_record_index: Some(residue_record_index),
                    });
                }
            }
            units
        }
        // C：P_0 一条记录，之后 Nc 条记录；小环每次发布至多上限条。MC 同 C 大环（登记 r2 5.3）。
        Domain::MonotoneBigRing
        | Domain::MonotoneSmallRingSix
        | Domain::MirroredMonotoneBigRing => {
            let largest_publish =
                (domain == Domain::MonotoneSmallRingSix).then_some(in_flight_limit);
            publish_sequences_of(size, largest_publish)
                .into_iter()
                .map(|sequence| {
                    let mut publishes = vec![PublishShape {
                        record_count: 1,
                        commit_marker_bits: 0,
                    }];
                    publishes.extend(sequence);
                    LegalWorkUnit {
                        plan: history_plan(
                            domain,
                            ORDINARY_FIRST_TXG,
                            publishes,
                            Vec::new(),
                            run_profile,
                        ),
                        residue_record_index: None,
                    }
                })
                .collect()
        }
        Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain => {
            unreachable!("对抗一族不走合法历史的生成器")
        }
    }
}

/// 一段历史的全部落盘子集（U 域：每条在盘上的记录另取点名单元五态），逐个判。所选根是 R_0。
pub(super) fn tally_of_a_legal_history(
    domain: Domain,
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let mut history = generate_history(&unit.plan);
    let residue = unit
        .residue_record_index
        .map(|record_index| generate_residue(&history, record_index));
    if let Some(generated_residue) = &residue {
        history
            .record_truth
            .note_the_record(&generated_residue.record);
    }
    let record_count = history.slots.len();
    let verify_named_units = domain != Domain::NamedUnitsNotVerified;
    let mut tally = Tally::new();
    let mut presence = vec![RecordPresence::NotOnDisk; record_count];
    let judge = |presence_of_the_state: &[RecordPresence],
                 is_the_first_state: bool,
                 tally_of_the_unit: &mut Tally| {
        let state = LegalState {
            history: &history,
            presence: presence_of_the_state,
            mirroring: JournalMirroring::FirstDeviceOnly,
            residue: residue.as_ref(),
            chosen_version: 0,
            verify_named_units,
        };
        judge_a_legal_state(
            &state,
            implementation,
            domain.specification_self_check(),
            unit_index == 0 && is_the_first_state && record_count == 1,
            tally_of_the_unit,
        );
    };
    if unit.plan.names_units {
        // 每条记录六态：不在，或在而点名单元取五态之一。按六进制数数过去。
        let mut codes = vec![0usize; record_count];
        let mut is_the_first_state = true;
        loop {
            for (record_index, code) in codes.iter().enumerate() {
                presence[record_index] = match code.checked_sub(1) {
                    None => RecordPresence::NotOnDisk,
                    Some(unit_state_index) => {
                        RecordPresence::OnDisk(UnitState::ALL[unit_state_index])
                    }
                };
            }
            judge(&presence, is_the_first_state, &mut tally);
            is_the_first_state = false;
            // 进位：迭代次数的上界是记录条数；每一位都回到 0 时数完了。
            let mut position = 0;
            while position < record_count && codes[position] == UnitState::ALL.len() {
                codes[position] = 0;
                position += 1;
            }
            if position == record_count {
                break;
            }
            codes[position] += 1;
        }
    } else {
        for records_on_the_first_device in every_subset_mask(record_count) {
            for (record_index, record_presence) in presence.iter_mut().enumerate() {
                *record_presence = if bit_is_set(records_on_the_first_device, record_index) {
                    RecordPresence::OnDisk(UnitState::BothCopiesGood)
                } else {
                    RecordPresence::NotOnDisk
                };
            }
            judge(&presence, records_on_the_first_device == 0, &mut tally);
        }
    }
    tally
}

// ── C 域：状态 = (记录子集, 根槽子集)，所选根 = 落了盘的根里最新的那条；状态对上判 J5a、J5b

/// 一个状态是哪几段的「停在这一段」的状态：前面各段全落、之后各段全没落、这一段任意。返回按段号置位的掩码。
fn stop_segment_mask(state_mask: u32, segments: &[u32]) -> u32 {
    let mut stop_mask = 0u32;
    let mut earlier_segments = 0u32;
    for (segment_index, segment) in segments.iter().enumerate() {
        let every_earlier_write_is_on_disk = state_mask & earlier_segments == earlier_segments;
        let no_later_write_is_on_disk = state_mask & !(earlier_segments | segment) == 0;
        if every_earlier_write_is_on_disk && no_later_write_is_on_disk {
            stop_mask |= 1 << segment_index;
        }
        earlier_segments |= segment;
    }
    stop_mask
}

/// C 域一段历史：先逐状态判（今天的实现与理想恢复各得一个施加区间），再在全部 s ⊆ u 的状态对上数五个集合的 J5a、J5b。
pub(super) fn tally_of_a_monotone_history(
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let history = generate_history(&unit.plan);
    let later_record_count = history.slots.len() - 1;
    let root_write_count = history.chosen_instance_publish_count - 1;
    let state_bit_count = later_record_count + root_write_count;
    let mut tally = Tally::new();
    let mut today_intervals: Vec<Option<AppliedInterval>> = Vec::new();
    let mut ideal_intervals: Vec<AppliedInterval> = Vec::new();
    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];
    // 状态掩码：低 Nc 位是 P_0 之后各条记录在不在盘上，高 m 位是各次发布的根槽写落没落。
    for state_mask in every_subset_mask(state_bit_count) {
        for (later_record, record_presence) in presence.iter_mut().skip(1).enumerate() {
            *record_presence = if bit_is_set(state_mask, later_record) {
                RecordPresence::OnDisk(UnitState::BothCopiesGood)
            } else {
                RecordPresence::NotOnDisk
            };
        }
        let chosen_version = (1..=root_write_count)
            .rev()
            .find(|version| bit_is_set(state_mask, later_record_count + version - 1))
            .unwrap_or(0);
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::FirstDeviceOnly,
            residue: None,
            chosen_version,
            verify_named_units: true,
        };
        let today_interval = judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && later_record_count == 1,
            &mut tally,
        );
        if today_interval.is_none() {
            tally.add(Counter::PairStatesWithoutAnInterval);
        }
        today_intervals.push(today_interval);
        ideal_intervals.push(AppliedInterval {
            low: chosen_version,
            high: farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
        });
    }

    // 两种切法的段（登记 5.3）。甲：根槽那次写自成一段；乙：根槽那次写与它那次发布的记录同段。
    let mut segments_when_the_root_write_is_its_own_segment: Vec<u32> = Vec::new();
    let mut segments_when_the_root_write_shares_the_record_segment: Vec<u32> = Vec::new();
    let mut root_write_bits = 0u32;
    for version in 1..=root_write_count {
        let record_bits = history
            .record_indexes_of_publish(version)
            .fold(0u32, |bits, record_index| bits | (1 << (record_index - 1)));
        let root_bit = 1u32 << (later_record_count + version - 1);
        root_write_bits |= root_bit;
        segments_when_the_root_write_is_its_own_segment.push(record_bits);
        segments_when_the_root_write_is_its_own_segment.push(root_bit);
        segments_when_the_root_write_shares_the_record_segment.push(record_bits | root_bit);
    }
    // 甲切法里偶数号段是记录段、奇数号段是根槽段。
    let record_segment_bits_of_cut_own =
        (0..root_write_count).fold(0u32, |bits, version| bits | (1 << (2 * version)));
    let root_segment_bits_of_cut_own = record_segment_bits_of_cut_own << 1;
    let stop_masks_of_cut_own: Vec<u32> = every_subset_mask(state_bit_count)
        .map(|state_mask| {
            stop_segment_mask(state_mask, &segments_when_the_root_write_is_its_own_segment)
        })
        .collect();
    let stop_masks_of_cut_with: Vec<u32> = every_subset_mask(state_bit_count)
        .map(|state_mask| {
            stop_segment_mask(
                state_mask,
                &segments_when_the_root_write_shares_the_record_segment,
            )
        })
        .collect();
    for state_mask in every_subset_mask(state_bit_count) {
        let state_index = usize::try_from(state_mask).expect("状态掩码不超过 12 位");
        if stop_masks_of_cut_own[state_index] != 0 {
            tally.add(Counter::LegalCrashStatesWhenTheRootWriteIsItsOwnSegment);
        }
        if stop_masks_of_cut_with[state_index] != 0 {
            tally.add(Counter::LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment);
        }
    }

    // 全部 s ⊆ u 的有序对（含 s = u）：对每个 u 枚举它的子掩码。迭代次数合计 3^(Nc + m)。
    for outer_mask in every_subset_mask(state_bit_count) {
        let outer_index = usize::try_from(outer_mask).expect("状态掩码不超过 12 位");
        let mut inner_mask = outer_mask;
        loop {
            let inner_index = usize::try_from(inner_mask).expect("状态掩码不超过 12 位");
            let shared_segments_of_cut_own =
                stop_masks_of_cut_own[inner_index] & stop_masks_of_cut_own[outer_index];
            let shared_segments_of_cut_with =
                stop_masks_of_cut_with[inner_index] & stop_masks_of_cut_with[outer_index];
            let root_write_differs = (inner_mask ^ outer_mask) & root_write_bits != 0;
            let memberships = [
                (PairSet::Wide, true),
                (
                    PairSet::CutOwnLegal,
                    stop_masks_of_cut_own[inner_index] != 0
                        && stop_masks_of_cut_own[outer_index] != 0,
                ),
                (PairSet::CutOwnSameSegment, shared_segments_of_cut_own != 0),
                (
                    PairSet::CutOwnSameRecordSegment,
                    shared_segments_of_cut_own & record_segment_bits_of_cut_own != 0,
                ),
                (
                    PairSet::CutOwnSameRootSegment,
                    shared_segments_of_cut_own & root_segment_bits_of_cut_own != 0,
                ),
                (
                    PairSet::CutWithLegal,
                    stop_masks_of_cut_with[inner_index] != 0
                        && stop_masks_of_cut_with[outer_index] != 0,
                ),
                (
                    PairSet::CutWithSameSegment,
                    shared_segments_of_cut_with != 0,
                ),
                (
                    PairSet::CutWithSameSegmentRootWriteDiffers,
                    shared_segments_of_cut_with != 0 && root_write_differs,
                ),
                (
                    PairSet::CutWithSameSegmentRootWriteAgrees,
                    shared_segments_of_cut_with != 0 && !root_write_differs,
                ),
            ];
            let today_pair = today_intervals[inner_index].zip(today_intervals[outer_index]);
            let ideal_inner = ideal_intervals[inner_index];
            let ideal_outer = ideal_intervals[outer_index];
            for (set, is_member) in memberships {
                if !is_member {
                    continue;
                }
                tally.add_pair(set, PairColumn::Pairs);
                if let Some((today_inner, today_outer)) = today_pair {
                    if !applied_set_is_contained(today_inner, today_outer) {
                        tally.add_pair(set, PairColumn::TodayAppliedSetViolations);
                    }
                    if landed_version_is_later(today_inner, today_outer) {
                        tally.add_pair(set, PairColumn::TodayLandedVersionViolations);
                    }
                }
                if !applied_set_is_contained(ideal_inner, ideal_outer) {
                    tally.add_pair(set, PairColumn::IdealAppliedSetViolations);
                }
                if landed_version_is_later(ideal_inner, ideal_outer) {
                    tally.add_pair(set, PairColumn::IdealLandedVersionViolations);
                }
            }
            if inner_mask == 0 {
                break;
            }
            inner_mask = (inner_mask - 1) & outer_mask;
        }
    }
    tally
}

// ── 多盘域（登记 r2 5.3）：每条记录在 0、1 号盘同一落点各一份，各自落或没落；没有点名单元

/// M、M6 一段历史：每条记录四态（0 号盘那一份落没落 × 1 号盘那一份落没落），所选根 R_0；每个状态另按两盘落盘集合的关系记进落后类别。
pub(super) fn tally_of_a_mirrored_history(
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let history = generate_history(&unit.plan);
    let record_count = history.slots.len();
    let mut tally = Tally::new();
    let mut presence = vec![RecordPresence::NotOnDisk; record_count];
    let mut second_device_copy_is_on_disk = vec![false; record_count];
    // 状态掩码：低 N 位是各条记录 0 号盘那一份，高 N 位是 1 号盘那一份。
    for state_mask in every_subset_mask(2 * record_count) {
        let first_device_mask = state_mask & ((1u32 << record_count) - 1);
        let second_device_mask = state_mask >> record_count;
        for (record_index, (record_presence, second_device_copy)) in presence
            .iter_mut()
            .zip(second_device_copy_is_on_disk.iter_mut())
            .enumerate()
        {
            *record_presence = if bit_is_set(first_device_mask, record_index) {
                RecordPresence::OnDisk(UnitState::BothCopiesGood)
            } else {
                RecordPresence::NotOnDisk
            };
            *second_device_copy = bit_is_set(second_device_mask, record_index);
        }
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::BothDevices {
                second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
            },
            residue: None,
            chosen_version: 0,
            verify_named_units: true,
        };
        let counts_before = tally.counts.clone();
        judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && record_count == 1,
            &mut tally,
        );
        // 探针跳过的状态不判、不进落后类别。
        if tally.count_of(Counter::States) > counts_before[Counter::States.index()] {
            tally.add_to_the_lagging_class(
                lagging_class_of(first_device_mask, second_device_mask),
                &counts_before,
            );
        }
    }
    tally
}

/// MC 一段历史（登记 r2 5.3）：P_0 一条两盘都在；其后 Nc 条记录各四态；P_1… 每次发布一个根槽写落或没落；所选根 = 落了的根里
/// 最新那条（R_0 恒在）。先逐状态判，再在「宽」集合上数 J5a、J5b（甲合法、乙合法与同段那几个集合这一次不建）。
pub(super) fn tally_of_a_mirrored_monotone_history(
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let history = generate_history(&unit.plan);
    let later_record_count = history.slots.len() - 1;
    let root_write_count = history.chosen_instance_publish_count - 1;
    let state_bit_count = 2 * later_record_count + root_write_count;
    let mut tally = Tally::new();
    let mut today_intervals: Vec<Option<AppliedInterval>> = Vec::new();
    let mut ideal_intervals: Vec<AppliedInterval> = Vec::new();
    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];
    let mut second_device_copy_is_on_disk = vec![true; history.slots.len()];
    // 状态掩码：低 Nc 位是 P_0 之后各条记录 0 号盘那一份，其上 Nc 位是 1 号盘那一份，最高 m 位是各次发布的根槽写落没落。
    for state_mask in every_subset_mask(state_bit_count) {
        for (later_record, (record_presence, second_device_copy)) in presence
            .iter_mut()
            .zip(second_device_copy_is_on_disk.iter_mut())
            .skip(1)
            .enumerate()
        {
            *record_presence = if bit_is_set(state_mask, later_record) {
                RecordPresence::OnDisk(UnitState::BothCopiesGood)
            } else {
                RecordPresence::NotOnDisk
            };
            *second_device_copy = bit_is_set(state_mask, later_record_count + later_record);
        }
        let chosen_version = (1..=root_write_count)
            .rev()
            .find(|version| bit_is_set(state_mask, 2 * later_record_count + version - 1))
            .unwrap_or(0);
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::BothDevices {
                second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
            },
            residue: None,
            chosen_version,
            verify_named_units: true,
        };
        let today_interval = judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && later_record_count == 1,
            &mut tally,
        );
        if today_interval.is_none() {
            tally.add(Counter::PairStatesWithoutAnInterval);
        }
        today_intervals.push(today_interval);
        ideal_intervals.push(AppliedInterval {
            low: chosen_version,
            high: farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
        });
    }
    add_the_wide_pairs(
        &mut tally,
        state_bit_count,
        &today_intervals,
        &ideal_intervals,
    );
    tally
}

/// 「宽」集合（登记 r2 5.3：两个状态的落盘集合与根槽集合都是包含关系，含相等）上数今天的实现与理想恢复各自的 J5a、J5b：
/// 对每个 u 枚举它的子掩码 s，迭代次数合计 3^(状态位数)。
fn add_the_wide_pairs(
    tally: &mut Tally,
    state_bit_count: usize,
    today_intervals: &[Option<AppliedInterval>],
    ideal_intervals: &[AppliedInterval],
) {
    for outer_mask in every_subset_mask(state_bit_count) {
        let outer_index = usize::try_from(outer_mask).expect("状态掩码不超过 12 位");
        let mut inner_mask = outer_mask;
        loop {
            let inner_index = usize::try_from(inner_mask).expect("状态掩码不超过 12 位");
            if inner_mask & outer_mask == inner_mask {
                tally.add_pair(PairSet::Wide, PairColumn::Pairs);
                if let Some((today_inner, today_outer)) =
                    today_intervals[inner_index].zip(today_intervals[outer_index])
                {
                    if !applied_set_is_contained(today_inner, today_outer) {
                        tally.add_pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations);
                    }
                    if landed_version_is_later(today_inner, today_outer) {
                        tally.add_pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations);
                    }
                }
                let (ideal_inner, ideal_outer) =
                    (ideal_intervals[inner_index], ideal_intervals[outer_index]);
                if !applied_set_is_contained(ideal_inner, ideal_outer) {
                    tally.add_pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations);
                }
                if landed_version_is_later(ideal_inner, ideal_outer) {
                    tally.add_pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations);
                }
            }
            if inner_mask == 0 {
                break;
            }
            inner_mask = (inner_mask - 1) & outer_mask;
        }
    }
}
