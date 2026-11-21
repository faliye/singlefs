//! G5：字节数与外推（第六节 G5、A6）。

use super::*;

// ───────────────────────────── G5：字节数与外推（第六节 G5、A6） ─────────────────────────────

/// 一种装入在一个容量上的判定（第六节 G5 表下：95% 到 100% 之间记临界）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CapacityVerdict {
    Fits,
    Critical,
    DoesNotFit,
}

impl CapacityVerdict {
    pub(super) fn of(required: u128, capacity: u128) -> Self {
        if required > capacity {
            Self::DoesNotFit
        } else if required * 100 >= capacity * CRITICAL_BAND_PERCENT {
            Self::Critical
        } else {
            Self::Fits
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Fits => "fits",
            Self::Critical => "critical",
            Self::DoesNotFit => "does_not_fit",
        }
    }
}

/// G5 的四部分（字节）：P1 去重后的内容、P2 每状态子集掩码、P3 读集、P4 集合键（分每状态那部分与每个不同键那部分）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeviceMemoryParts {
    pub(super) contents: u128,
    pub(super) masks: u128,
    pub(super) read_sets: u128,
    pub(super) keys_per_state: u128,
    keys_per_distinct_key: u128,
}

impl DeviceMemoryParts {
    pub(super) fn total(&self) -> u128 {
        self.contents
            + self.masks
            + self.read_sets
            + self.keys_per_state
            + self.keys_per_distinct_key
    }

    /// 五卡切开：P1 每张卡各放一份，其余切开放（第六节 Q5b 的式子）。
    fn five_cards(&self) -> CapacityVerdict {
        let per_card = SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE;
        let all_cards = FIVE_CARDS_MEBIBYTES * MEBIBYTE;
        if self.contents > per_card || CARD_COUNT * self.contents > all_cards {
            return CapacityVerdict::DoesNotFit;
        }
        let contents_verdict = CapacityVerdict::of(self.contents, per_card);
        let rest_verdict = CapacityVerdict::of(
            self.total() - self.contents,
            all_cards - CARD_COUNT * self.contents,
        );
        match (contents_verdict, rest_verdict) {
            (CapacityVerdict::DoesNotFit, _) | (_, CapacityVerdict::DoesNotFit) => {
                CapacityVerdict::DoesNotFit
            }
            (CapacityVerdict::Critical, _) | (_, CapacityVerdict::Critical) => {
                CapacityVerdict::Critical
            }
            (CapacityVerdict::Fits, CapacityVerdict::Fits) => CapacityVerdict::Fits,
        }
    }

    /// Q5b：四部分都按 N_ext / N 放大。
    pub(super) fn linear(&self, from_states: u128, to_states: u128) -> Self {
        let scale = |bytes: u128| bytes * to_states / from_states;
        Self {
            contents: scale(self.contents),
            masks: scale(self.masks),
            read_sets: scale(self.read_sets),
            keys_per_state: scale(self.keys_per_state),
            keys_per_distinct_key: scale(self.keys_per_distinct_key),
        }
    }

    /// Q5c：去重的部分（P1、P3、P4 每个不同键那部分）不涨，每状态的部分按 N_ext / N 放大。
    fn deduplicated_parts_fixed(&self, from_states: u128, to_states: u128) -> Self {
        let scale = |bytes: u128| bytes * to_states / from_states;
        Self {
            contents: self.contents,
            masks: scale(self.masks),
            read_sets: self.read_sets,
            keys_per_state: scale(self.keys_per_state),
            keys_per_distinct_key: self.keys_per_distinct_key,
        }
    }
}

/// 每状态子集掩码的字节数：⌈W / 8⌉。
pub(super) fn mask_bytes_per_state(write_table_length: usize) -> u128 {
    u128::try_from(write_table_length.div_ceil(8)).expect("掩码字节")
}

/// 一格在一条 K4 臂下的四部分。
pub(super) fn device_memory_parts(
    summary: &CellSummary,
    write_table_length: usize,
    checker_arm: Arm,
) -> DeviceMemoryParts {
    let states = u128::from(summary.states);
    let read_set_bytes = |arm: Arm| {
        u128::from(summary.arm_distinct_key_elements[arm.slot()])
            * u128::from(arm.bytes_per_element())
    };
    DeviceMemoryParts {
        contents: summary.content_bytes_by_category.iter().sum(),
        masks: mask_bytes_per_state(write_table_length) * states,
        read_sets: read_set_bytes(Arm::RecoveryConsult)
            + read_set_bytes(Arm::RecoveryIgnore)
            + read_set_bytes(checker_arm),
        keys_per_state: states * u128::from(KEYS_STORED_PER_STATE * KEY_BYTES),
        keys_per_distinct_key: u128::from(
            summary.arm_distinct_keys[Arm::RecoveryConsult.slot()]
                + summary.arm_distinct_keys[Arm::RecoveryIgnore.slot()]
                + summary.arm_distinct_keys[checker_arm.slot()],
        ) * u128::from(DISTINCT_KEY_ENTRY_BYTES),
    }
}

/// 一组外推的判定字段：单卡 32 GB、16 GB 那一档单卡、五卡切开。
fn capacity_fields(prefix: &str, parts: &DeviceMemoryParts) -> (String, [CapacityVerdict; 3]) {
    let largest = CapacityVerdict::of(parts.total(), LARGEST_CARD_MEBIBYTES * MEBIBYTE);
    let sixteen = CapacityVerdict::of(
        parts.total(),
        SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE,
    );
    let five = parts.five_cards();
    (
        format!(
            "{prefix}_bytes={} {prefix}_largest_card={} {prefix}_sixteen_gigabyte_card={} {prefix}_five_cards={}",
            parts.total(),
            largest.label(),
            sixteen.label(),
            five.label()
        ),
        [largest, sixteen, five],
    )
}

/// G5 的一格一臂：四部分、每状态平均、Q5b / Q5c / Q5d / Q5e（在各判定字段里）、Q5f、Q5g。
pub(super) fn report_device_memory(
    output: &mut ResultLines,
    summary: &CellSummary,
    write_table_length: usize,
    second_stream_write_table_length: usize,
    checker_arm: Arm,
) -> [[CapacityVerdict; 3]; 2] {
    let parts = device_memory_parts(summary, write_table_length, checker_arm);
    let states = u128::from(summary.states);
    let (linear_fields, linear) =
        capacity_fields("q5b", &parts.linear(states, EXTRAPOLATED_STATES));
    let (fixed_fields, fixed) = capacity_fields(
        "q5c",
        &parts.deduplicated_parts_fixed(states, EXTRAPOLATED_STATES),
    );
    let agreement: Vec<String> = ["largest_card", "sixteen_gigabyte_card", "five_cards"]
        .iter()
        .zip(linear.iter().zip(&fixed))
        .map(|(capacity, (left, right))| format!("q5d_{capacity}_agree={}", left == right))
        .collect();
    let second_stream_parts = DeviceMemoryParts {
        contents: parts.contents,
        masks: mask_bytes_per_state(second_stream_write_table_length) * states,
        read_sets: parts.read_sets,
        keys_per_state: parts.keys_per_state,
        keys_per_distinct_key: parts.keys_per_distinct_key,
    };
    let (full_linear_fields, _) = capacity_fields(
        "q5f_linear",
        &second_stream_parts.linear(states, SECOND_STREAM_FULL_STATES),
    );
    let (full_fixed_fields, _) = capacity_fields(
        "q5f_fixed",
        &second_stream_parts.deduplicated_parts_fixed(states, SECOND_STREAM_FULL_STATES),
    );
    let per_state_of_largest_segment = DeviceMemoryParts {
        contents: parts.contents,
        masks: mask_bytes_per_state(second_stream_write_table_length)
            * SECOND_STREAM_LARGEST_SEGMENT_STATES,
        read_sets: 0,
        keys_per_state: SECOND_STREAM_LARGEST_SEGMENT_STATES
            * u128::from(KEYS_STORED_PER_STATE * KEY_BYTES),
        keys_per_distinct_key: 0,
    };
    let (segment_batch_fields, _) =
        capacity_fields("q5g_largest_segment_batch", &per_state_of_largest_segment);
    output.line(format!(
        "name=g5 cell={} checker_arm={} states={states} w={write_table_length} p1_bytes={} p2_bytes={} p3_bytes={} p4_per_state_bytes={} p4_distinct_key_bytes={} total_bytes={} bytes_per_state={} {linear_fields} {fixed_fields} {} {full_linear_fields} {full_fixed_fields} {segment_batch_fields}",
        summary.label,
        checker_arm.label(),
        parts.contents,
        parts.masks,
        parts.read_sets,
        parts.keys_per_state,
        parts.keys_per_distinct_key,
        parts.total(),
        ratio_text(parts.total(), states),
        agreement.join(" "),
    ));
    [linear, fixed]
}
