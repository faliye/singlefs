//! 界（登记 5.2、5.6）：要判的段、界的结局与缩域的配置字段；钉绝对值的断言（登记第七节；不符作废条款 V1）。

use super::*;

// ── 界（登记 5.2、5.6）

/// 这一趟判哪几段登记界（三族的命令行都照它取）：不给 `--bounds` 是不指定范围，登记的每一段（[`BoundsTier::REGISTERED`]）都判、
/// 按登记的次序一段接一段；给了是指定范围，只判给的那一段。
pub(super) fn bounds_to_judge_of_the_option(given: Option<BoundsTier>) -> Vec<BoundsTier> {
    match given {
        Some(bounds) => vec![bounds],
        None => BoundsTier::REGISTERED.to_vec(),
    }
}

/// 一段登记界判完的结局：有没有作废或停机的条件（有一条，整个进程以非 0 退出，见 [`RunFindings`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OutcomeOfTheBounds {
    NeitherVoidNorHalted,
    VoidOrHalted,
}

impl OutcomeOfTheBounds {
    pub(super) fn of_the_findings(findings: &RunFindings) -> Self {
        if findings.void_conditions.is_empty() && findings.halt_conditions.is_empty() {
            Self::NeitherVoidNorHalted
        } else {
            Self::VoidOrHalted
        }
    }
}

/// 几段登记界都判完之后：有一段作废或停机，进程以非 0 退出（每一段都先判完、各自的行都打出来，再看结局）。
pub(super) fn exit_when_some_bounds_ended_void_or_halted(outcomes: &[OutcomeOfTheBounds]) {
    if outcomes.contains(&OutcomeOfTheBounds::VoidOrHalted) {
        std::process::exit(1);
    }
}

/// 给了缩域参数（`--max-n`、`--max-u`）的那一趟，配置行末尾标一段 ` narrowed_below_the_registered_bounds=<参数名>`：
/// 缩过的产物写明缩过、不当正式产物；不给就一个字不加，跑登记界的那一趟产物逐字节不变。
pub(super) fn narrowing_field_of_the_configuration_line(
    cap: Option<usize>,
    narrowing_option: &str,
) -> String {
    match cap {
        Some(_) => format!(" narrowed_below_the_registered_bounds={narrowing_option}"),
        None => String::new(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BoundsTier {
    SegmentOne,
    SegmentTwo,
}

impl BoundsTier {
    /// 登记的每一段界（登记 5.2、5.6），按登记的次序：不给 `--bounds` 的那一趟逐段都判（[`bounds_to_judge_of_the_option`]）。
    pub(super) const REGISTERED: [Self; 2] = [Self::SegmentOne, Self::SegmentTwo];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::SegmentOne => "segment1",
            Self::SegmentTwo => "segment2",
        }
    }

    /// 这一档里这个域扫到的最大 N（或 Nc、n）；`None` 是这一档不跑这个域。
    pub(super) const fn largest_size(self, domain: Domain) -> Option<usize> {
        match self {
            Self::SegmentOne => Some(match domain {
                Domain::LegalBigRing => 10,
                Domain::LegalSmallRingSix => 6,
                Domain::LegalSmallRingNine => 9,
                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => 6,
                Domain::MonotoneBigRing => 6,
                Domain::MonotoneSmallRingSix => 5,
                Domain::AdversarialBigRing => 4,
                Domain::AdversarialSmallRingSix | Domain::AdversarialBackChain => 3,
                Domain::CrossInstance => 7,
                Domain::TxgOneBelowTheLargest => 4,
                Domain::TxgAtTheLargest => 3,
                Domain::Residue => 5,
                // 登记 r2 5.3：M、M6 到 N = 6，MC 到 Nc = 4。
                Domain::MirroredBigRing | Domain::MirroredSmallRingSix => 6,
                Domain::MirroredMonotoneBigRing => 4,
            }),
            Self::SegmentTwo => match domain {
                Domain::LegalBigRing => Some(8),
                Domain::LegalSmallRingSix => Some(6),
                Domain::LegalSmallRingNine => None,
                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => Some(5),
                Domain::MonotoneBigRing | Domain::MonotoneSmallRingSix => Some(5),
                Domain::AdversarialBigRing
                | Domain::AdversarialSmallRingSix
                | Domain::AdversarialBackChain => Some(3),
                Domain::CrossInstance => Some(6),
                Domain::TxgOneBelowTheLargest => Some(4),
                Domain::TxgAtTheLargest => Some(3),
                Domain::Residue => Some(5),
                // 第二段的变体不跑多盘域（登记 r2 5.6：多盘是第三段的事）。
                Domain::MirroredBigRing
                | Domain::MirroredSmallRingSix
                | Domain::MirroredMonotoneBigRing => None,
            },
        }
    }
}

// ── 钉绝对值的断言（登记第七节；不符 ⇒ 作废条款 V1）

pub(super) fn assert_the_pinned_absolute_values(domain: Domain, size: usize, tally: &Tally) {
    let size_index = size - domain.smallest_size();
    assert_eq!(
        tally.count_of(Counter::States),
        pinned_state_counts(domain)[size_index],
        "作废条款 V1：{} 域 N = {size} 的状态数与登记第七节不等",
        domain.label()
    );
    if domain == Domain::LegalBigRing {
        if let Some((farthest_at_least_one, farthest_at_least_two)) =
            PINNED_FARTHEST_VERSION_WITNESSES_OF_THE_BIG_RING.get(size_index)
        {
            assert_eq!(
                (
                    tally.count_of(Counter::FarthestAtLeastOne),
                    tally.count_of(Counter::FarthestAtLeastTwo)
                ),
                (*farthest_at_least_one, *farthest_at_least_two),
                "作废条款 V1：L 域 N = {size} 的见证数（j* ≥ 1, j* ≥ 2）与登记第七节不等"
            );
        }
    }
    if let Some(pinned) = pinned_monotone_counts(domain).get(size_index) {
        let counted = [
            tally.pair(PairSet::Wide, PairColumn::Pairs),
            tally.count_of(Counter::LegalCrashStatesWhenTheRootWriteIsItsOwnSegment),
            tally.pair(PairSet::CutOwnLegal, PairColumn::Pairs),
            tally.pair(PairSet::CutOwnSameSegment, PairColumn::Pairs),
            tally.count_of(Counter::LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment),
            tally.pair(PairSet::CutWithLegal, PairColumn::Pairs),
            tally.pair(PairSet::CutWithSameSegment, PairColumn::Pairs),
        ];
        let expected = [
            pinned.wide_pairs,
            pinned.cut_own_legal_states,
            pinned.cut_own_legal_pairs,
            pinned.cut_own_same_segment_pairs,
            pinned.cut_with_legal_states,
            pinned.cut_with_legal_pairs,
            pinned.cut_with_same_segment_pairs,
        ];
        assert_eq!(
            counted,
            expected,
            "作废条款 V1：{} 域 Nc = {size} 的状态对数或合法崩溃状态数与登记第七节、第十二节不等（次序：宽对数、甲合法态数、甲合法对数、甲同段对数、乙合法态数、乙合法对数、乙同段对数）",
            domain.label()
        );
    }
    if let Some(pinned) = pinned_mirrored_counts(domain).get(size_index) {
        let class_states = |class: LaggingClass| tally.lagging_class_count(class, Counter::States);
        assert_eq!(
            [
                class_states(LaggingClass::Equal),
                class_states(LaggingClass::FirstDeviceBehind)
                    + class_states(LaggingClass::SecondDeviceBehind),
                class_states(LaggingClass::Crossed),
                class_states(LaggingClass::FirstDeviceBehind),
                class_states(LaggingClass::SecondDeviceBehind),
                tally.count_of(Counter::FarthestAtLeastOne),
                tally.count_of(Counter::FarthestAtLeastTwo),
            ],
            [
                pinned.equal_states,
                pinned.lagging_states,
                pinned.crossed_states,
                pinned.lagging_states_in_one_direction,
                pinned.lagging_states_in_one_direction,
                pinned.farthest_at_least_one,
                pinned.farthest_at_least_two,
            ],
            "作废条款 SV7：{} 域 N = {size} 的落后类别或见证数与登记 r2 第七节乙类不等（次序：相等、落后、互不包含、0 号盘落后、1 号盘落后、j* ≥ 1、j* ≥ 2）",
            domain.label()
        );
        assert_eq!(
            LaggingClass::ALL
                .iter()
                .map(|class| class_states(*class))
                .sum::<u64>(),
            tally.count_of(Counter::States),
            "作废条款 SV7：{} 域 N = {size} 四类状态数之和不等于状态数（登记 r2 5.5「落后类别」那一行）",
            domain.label()
        );
    }
    if let Some(pinned) = pinned_mirrored_monotone_counts(domain).get(size_index) {
        assert_eq!(
            [
                tally.pair(PairSet::Wide, PairColumn::Pairs),
                tally.count_of(Counter::ChosenRootIsALaterVersion),
                tally.count_of(Counter::FarthestAtLeastOne),
                tally.count_of(Counter::FarthestAtLeastTwo),
            ],
            [
                pinned.wide_pairs,
                pinned.chosen_root_is_a_later_version,
                pinned.farthest_at_least_one,
                pinned.farthest_at_least_two,
            ],
            "作废条款 SV7：MC 域 Nc = {size} 的宽对数或见证数与登记 r2 第七节乙类不等（次序：宽对数、所选根不是 R_0、j* − c ≥ 1、j* − c ≥ 2）"
        );
    }
}

#[cfg(test)]
mod tests;
