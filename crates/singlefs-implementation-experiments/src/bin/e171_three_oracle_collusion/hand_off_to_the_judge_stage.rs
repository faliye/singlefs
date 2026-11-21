//! 录入段交给判段的交接：字段的写法与读法、交接文件的写与读。

use super::*;

// ───────────────────────────── 两段的交接与选段 ─────────────────────────────

/// 点段的参数（取值照 `crash_library::RunStage` 的写法：`record` 只跑录入段、`judge` 只跑判段，不给就两段先后跑）。
pub(super) const STAGE_ARGUMENT: &str = "--stage";

/// 只跑录入段那一趟留给判段的交接，放在输出目录里：判段读完、写完产物之后删掉；两段先后跑的那一趟在内存里交、不落这个文件。
/// 里面没有判定（O2 录下的只是盘的摘要，盘在判定库里）：是录入段的观测与结果行要的东西。
pub(super) const HAND_OFF_TO_THE_JUDGE_STAGE_FILE: &str = "hand-off-to-the-judge-stage.bin";

/// 交接字节的头：换了布局就换头，判段只认这一版。
const HAND_OFF_MAGIC: &[u8; 8] = b"E171HO01";

/// 只跑录入段那一趟录完的退出码（停机条款归判段）。
pub(super) const EXIT_CODE_OF_A_RECORD_ONLY_RUN: i32 = 0;

/// 交接里的一样东西怎么写成字节、怎么读回：整数小端定长，串与列表先写长度，枚举先写一个字节的成员号。
pub(super) trait HandOffField: Sized {
    fn put_into(&self, bytes: &mut Vec<u8>);
    /// 字节不够、成员号认不得、串不是 UTF-8 交回 `None`。
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self>;
}

/// 从交接字节里按次序往下读。
pub(super) struct HandOffReader<'bytes> {
    pub(super) bytes: &'bytes [u8],
    pub(super) position: usize,
}

impl<'bytes> HandOffReader<'bytes> {
    fn take_bytes(&mut self, count: usize) -> Option<&'bytes [u8]> {
        let end = self.position.checked_add(count)?;
        let taken = self.bytes.get(self.position..end)?;
        self.position = end;
        Some(taken)
    }

    fn is_at_the_end(&self) -> bool {
        self.position == self.bytes.len()
    }
}

impl HandOffField for u8 {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self);
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        reader.take_bytes(1).map(|taken| taken[0])
    }
}

impl HandOffField for bool {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(u8::from(*self));
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(false),
            1 => Some(true),
            2..=u8::MAX => None,
        }
    }
}

impl HandOffField for u64 {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.to_le_bytes());
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        Some(u64::from_le_bytes(reader.take_bytes(8)?.try_into().ok()?))
    }
}

impl HandOffField for u128 {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.to_le_bytes());
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        Some(u128::from_le_bytes(reader.take_bytes(16)?.try_into().ok()?))
    }
}

impl HandOffField for usize {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        u64::try_from(*self)
            .expect("usize 装得进 u64")
            .put_into(bytes);
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        usize::try_from(u64::take_from(reader)?).ok()
    }
}

impl HandOffField for String {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        self.len().put_into(bytes);
        bytes.extend_from_slice(self.as_bytes());
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        let length = usize::take_from(reader)?;
        String::from_utf8(reader.take_bytes(length)?.to_vec()).ok()
    }
}

impl HandOffField for [u8; DIGEST_BYTES] {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(self);
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        reader.take_bytes(DIGEST_BYTES)?.try_into().ok()
    }
}

impl<Element: HandOffField> HandOffField for Option<Element> {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            None => bytes.push(0),
            Some(value) => {
                bytes.push(1);
                value.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(None),
            1 => Element::take_from(reader).map(Some),
            2..=u8::MAX => None,
        }
    }
}

impl<Element: HandOffField> HandOffField for Vec<Element> {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        self.len().put_into(bytes);
        for element in self {
            element.put_into(bytes);
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        let count = usize::take_from(reader)?;
        (0..count)
            .map(|_element_position| Element::take_from(reader))
            .collect()
    }
}

/// 结构体的交接：字段按写出的次序逐个写、逐个读（构造时每个字段都要写出来，漏了一个编不过）。
macro_rules! hand_off_fields_of_a_struct {
    ($structure:ident { $($field:ident),+ $(,)? }) => {
        impl HandOffField for $structure {
            fn put_into(&self, bytes: &mut Vec<u8>) {
                $( self.$field.put_into(bytes); )+
            }
            fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
                Some(Self {
                    $( $field: HandOffField::take_from(reader)?, )+
                })
            }
        }
    };
}

hand_off_fields_of_a_struct!(StopCheck {
    name,
    local,
    crates
});
hand_off_fields_of_a_struct!(RecordedCall {
    label,
    first_operation,
    return_point,
    outcome,
    model
});
hand_off_fields_of_a_struct!(TornScan {
    segment_index,
    scanned,
    found
});
hand_off_fields_of_a_struct!(ImplementationVerdict {
    consulted_root,
    consulted_outcome,
    ignored_root,
    ignored_outcome,
    red_items,
    pool_checker_red,
    root_without_record,
    claimed_state_missing_unit
});
hand_off_fields_of_a_struct!(RecoverySummary {
    root,
    outcome,
    read_back_sha256,
    records_applied
});
hand_off_fields_of_a_struct!(StateObservation {
    batch,
    ordinal,
    segment_index,
    landing_vector,
    image_digest,
    verdict,
    judge_nanoseconds,
    mask_matches_the_facts,
    touched_call,
    crash_judge,
    pool_checker,
    consulted,
    ignored,
    crash_judge_nanoseconds,
    pool_checker_nanoseconds,
    recovery_nanoseconds
});
hand_off_fields_of_a_struct!(WrittenFile {
    name,
    bytes,
    sha256,
    sha256_without_timing
});
hand_off_fields_of_a_struct!(StreamExport {
    name,
    checks,
    calls,
    ending,
    model_stopped_at_call,
    not_exported,
    operation_count,
    write_count,
    segment_count,
    tearable_write_count,
    full_states,
    in_place_subsets_states,
    torn_scans,
    observations,
    files,
    rebuilt_digest_mismatches,
    rebuilt_states,
    elapsed_nanoseconds,
    drive_nanoseconds,
    observe_nanoseconds,
    write_nanoseconds
});
hand_off_fields_of_a_struct!(RecordedTableReference { table_name, digest });
hand_off_fields_of_a_struct!(RecordedDump {
    stream_independent_checks,
    exports
});
hand_off_fields_of_a_struct!(RecordedKnownBadEntry {
    target,
    patch_count,
    digest_equals_the_direct_one,
    direct,
    patched
});
hand_off_fields_of_a_struct!(RecordedE170BadRow {
    label,
    row_identifier,
    base_digest,
    digests_equal,
    e170_pool_checker_column,
    base,
    patched
});
hand_off_fields_of_a_struct!(RecordedCleanImage { entry_count, table });
hand_off_fields_of_a_struct!(PanicOfTheRecordStage { step, message });
hand_off_fields_of_a_struct!(RecordedKnownBadImages {
    clean_image_checks,
    clean,
    entries,
    states_text,
    patches_file,
    e170_bad_rows,
    panic_of_the_record_stage
});

impl HandOffField for KnownBadImagesStep {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(match self {
            Self::CleanImage => 0,
            Self::KnownBadEntries => 1,
            Self::E170BadRows => 2,
        });
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::CleanImage),
            1 => Some(Self::KnownBadEntries),
            2 => Some(Self::E170BadRows),
            3..=u8::MAX => None,
        }
    }
}

impl HandOffField for StreamName {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(match self {
            Self::NewPoolFileCreation => 0,
            Self::FixedScriptReuseAfterRaisingFloorThenUnmount => 1,
        });
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::NewPoolFileCreation),
            1 => Some(Self::FixedScriptReuseAfterRaisingFloorThenUnmount),
            2..=u8::MAX => None,
        }
    }
}

impl HandOffField for CallOutcome {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Succeeded => bytes.push(0),
            Self::ReturnedError(member) => {
                bytes.push(1);
                member.put_into(bytes);
            }
            Self::Panicked(text) => {
                bytes.push(2);
                text.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::Succeeded),
            1 => Some(Self::ReturnedError(String::take_from(reader)?)),
            2 => Some(Self::Panicked(String::take_from(reader)?)),
            3..=u8::MAX => None,
        }
    }
}

impl HandOffField for ModelVerdictOfCall {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::NotModelled => bytes.push(0),
            Self::Agrees => bytes.push(1),
            Self::Disagrees {
                aspect,
                model_answer,
                implementation_answer,
            } => {
                bytes.push(2);
                aspect.put_into(bytes);
                model_answer.put_into(bytes);
                implementation_answer.put_into(bytes);
            }
            Self::SkippedAfterTheModelStopped => bytes.push(3),
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::NotModelled),
            1 => Some(Self::Agrees),
            2 => Some(Self::Disagrees {
                aspect: String::take_from(reader)?,
                model_answer: String::take_from(reader)?,
                implementation_answer: String::take_from(reader)?,
            }),
            3 => Some(Self::SkippedAfterTheModelStopped),
            4..=u8::MAX => None,
        }
    }
}

impl HandOffField for StreamEnding {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Completed => bytes.push(0),
            Self::StoppedByAnError { call_index, member } => {
                bytes.push(1);
                call_index.put_into(bytes);
                member.put_into(bytes);
            }
            Self::ImplementationCrashed { what } => {
                bytes.push(2);
                what.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::Completed),
            1 => Some(Self::StoppedByAnError {
                call_index: usize::take_from(reader)?,
                member: String::take_from(reader)?,
            }),
            2 => Some(Self::ImplementationCrashed {
                what: String::take_from(reader)?,
            }),
            3..=u8::MAX => None,
        }
    }
}

impl HandOffField for CrashJudgeVerdict {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::NotApplicable => bytes.push(0),
            Self::Green => bytes.push(1),
            Self::Red { what } => {
                bytes.push(2);
                what.put_into(bytes);
            }
            Self::Off => bytes.push(3),
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::NotApplicable),
            1 => Some(Self::Green),
            2 => Some(Self::Red {
                what: String::take_from(reader)?,
            }),
            3 => Some(Self::Off),
            4..=u8::MAX => None,
        }
    }
}

impl HandOffField for PoolCheckerOfAState {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Off => bytes.push(0),
            Self::RecordedForTheJudgeStage(table) => {
                bytes.push(1);
                table.put_into(bytes);
            }
            Self::Judged {
                red,
                matches_the_judgement,
            } => {
                bytes.push(2);
                red.put_into(bytes);
                matches_the_judgement.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::Off),
            1 => Some(Self::RecordedForTheJudgeStage(
                RecordedTableReference::take_from(reader)?,
            )),
            2 => Some(Self::Judged {
                red: Vec::take_from(reader)?,
                matches_the_judgement: bool::take_from(reader)?,
            }),
            3..=u8::MAX => None,
        }
    }
}

/// 交接的全部字节：头、这一趟的参数（判段照它核两段是不是同一条命令）、录入段交出的那一份。
pub(super) fn hand_off_bytes<HandOff: HandOffField>(
    run_arguments: &[String],
    hand_off: &HandOff,
) -> Vec<u8> {
    let mut bytes = HAND_OFF_MAGIC.to_vec();
    run_arguments.to_vec().put_into(&mut bytes);
    hand_off.put_into(&mut bytes);
    bytes
}

/// 读回交接：头对得上、参数与这一趟的相同、录入段交出的那一份解得出、末尾没有多出的字节。
///
/// # Errors
/// 上面四样有一样不成，一句话说哪一样、出路是什么。
pub(super) fn hand_off_of_the_bytes<HandOff: HandOffField>(
    bytes: &[u8],
    run_arguments: &[String],
) -> Result<HandOff, String> {
    let mut reader = HandOffReader { bytes, position: 0 };
    if reader.take_bytes(HAND_OFF_MAGIC.len()) != Some(HAND_OFF_MAGIC.as_slice()) {
        return Err(format!(
            "交接的头不是 {}：不是这一版装置的录入段写的，重跑录入段（{STAGE_ARGUMENT} record）",
            String::from_utf8_lossy(HAND_OFF_MAGIC)
        ));
    }
    let recorded_arguments = Vec::<String>::take_from(&mut reader)
        .ok_or_else(|| "交接里的参数解不出：重跑录入段".to_string())?;
    if recorded_arguments != run_arguments {
        return Err(format!(
            "录入段那一趟的参数是 {recorded_arguments:?}，这一趟是 {run_arguments:?}：判段照录入段那一趟的参数起（只换 {STAGE_ARGUMENT}）"
        ));
    }
    let hand_off = HandOff::take_from(&mut reader)
        .ok_or_else(|| "交接里录入段交出的那一份解不出：重跑录入段".to_string())?;
    if !reader.is_at_the_end() {
        return Err("交接末尾多出字节：重跑录入段".to_string());
    }
    Ok(hand_off)
}

#[cfg(test)]
mod tests;
