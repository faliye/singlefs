//! 对拍用例的两段：对拍输入、参照与判表经对拍库录入读回，对拍结论按键复用或现判。

use super::*;

// ============================== 对拍用例的两段与对拍结论 ==============================

/// 对拍用例的事实表清单里事实表那一样的名字。
const PAIRING_MANIFEST_FACTS: &str = "facts";

/// 第 ② 段核对的 GPU 与 CPU 参照对拍用例用：录入段把事实表按摘要写进库、按录入键（事实表摘要）登清单；判段照同一张事实表现算录入键、
/// 读清单、按摘要读回，核对只拿读回的那一份。只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn facts_through_the_pairing_library(
    item_name: &str,
    facts: &FlowFacts,
) -> StageOutcome<FlowFactsReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let digest = facts.digest();
    two_stages(
        CrashLibraryClass::FactsPairing,
        &item,
        &RecordingInputsDigest(digest),
        |library| {
            library
                .lock()
                .store_flow_facts(&digest, &facts.to_bytes())
                .map_err(|error| format!("事实表写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "facts_pairing".to_string(),
                digests: vec![(PAIRING_MANIFEST_FACTS.to_string(), digest)],
            })
        },
        |library, manifest| {
            Ok(StageOutcome::Judged(read_flow_facts_back(
                &library.lock(),
                &manifest.digest_named(PAIRING_MANIFEST_FACTS)?,
            )?))
        },
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的事实表过库失败：{refusal}"))
}

/// 对拍用例的输入清单里输入那一样的名字。
const PAIRING_MANIFEST_INPUT: &str = "input";

/// 从判定库里读回、核过摘要的一份对拍输入：对拍的两边只收它。字段私有，只有 [`PairingInputReadBack::from_the_library_bytes`] 造得出。
#[derive(Debug)]
pub struct PairingInputReadBack {
    bytes: Vec<u8>,
    digest: [u8; DIGEST_BYTES],
}

impl PairingInputReadBack {
    /// 库里按 `digest` 存的那份字节：核摘要。
    ///
    /// # Errors
    /// 字节的摘要对不上（库里那份被改过）。
    pub fn from_the_library_bytes(
        bytes: Vec<u8>,
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        if sha256_digest(&bytes) != *digest {
            return Err("对拍输入摘要对不上：库里那份坏了，重跑录入段".to_string());
        }
        Ok(Self {
            bytes,
            digest: *digest,
        })
    }

    /// 读回的字节。
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// 这份输入的摘要（对拍结论的键里输入那一半，[`pairing_conclusion_reused_or_judged`]）。
    #[must_use]
    pub fn digest(&self) -> &[u8; DIGEST_BYTES] {
        &self.digest
    }
}

/// 按摘要从库里读对拍输入：读字节、核摘要。
///
/// # Errors
/// 库读失败；库里没有这个摘要；字节的摘要对不上。
pub fn read_pairing_input_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PairingInputReadBack, String> {
    let bytes = store
        .read_pairing_input(digest)
        .map_err(|error| format!("读对拍输入失败：{error:?}"))?
        .ok_or_else(|| "对拍输入不在库里：先跑录入段".to_string())?;
    PairingInputReadBack::from_the_library_bytes(bytes, digest)
}

/// 输入是调用方给的字节的对拍用例用：录入段把输入按摘要写进库、按录入键（输入摘要）登清单；判段照同一份输入现算录入键、读清单、
/// 按摘要读回，对拍的两边只拿读回的那一份。只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
/// 与 [`facts_through_the_pairing_library`] 同一个走法，输入不限是事实表。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn pairing_input_through_the_pairing_library(
    item_name: &str,
    input_bytes: &[u8],
) -> StageOutcome<PairingInputReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing)
        .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」开不了库：{refusal}"));
    let recording = recording_key_of_the_scenario_and_the_inputs(
        item.as_str(),
        &RecordingInputsDigest(sha256_digest(input_bytes)),
    );
    pairing_input_through_the_library_under_the_recording_key(
        &library,
        RunStage::of_this_process(),
        &item,
        &recording,
        input_bytes,
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的输入过库失败：{refusal}"))
}

/// [`pairing_input_through_the_pairing_library`] 的本体：库、选段与录入键由调用方给（单测在一次性的库上跑）。
///
/// # Errors
/// 写不进库；登清单、读清单不成；库里没有清单指到的那一份；读回的字节摘要对不上。
pub fn pairing_input_through_the_library_under_the_recording_key(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    input_bytes: &[u8],
) -> Result<StageOutcome<PairingInputReadBack>, String> {
    let digest = sha256_digest(input_bytes);
    two_stages_under_the_recording_key(
        library,
        stage,
        item,
        recording,
        |library_recorded_into| {
            library_recorded_into
                .lock()
                .store_pairing_input(&digest, input_bytes)
                .map_err(|error| format!("对拍输入写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "input_pairing".to_string(),
                digests: vec![(PAIRING_MANIFEST_INPUT.to_string(), digest)],
            })
        },
        |library_read_from, manifest| {
            Ok(StageOutcome::Judged(read_pairing_input_back(
                &library_read_from.lock(),
                &manifest.digest_named(PAIRING_MANIFEST_INPUT)?,
            )?))
        },
    )
}

/// 对拍用例的清单里参照那一样的名字。
const PAIRING_MANIFEST_REFERENCE: &str = "reference";

/// 从库里读回的对拍输入与参照一侧的结果（[`pairing_input_and_reference_through_the_pairing_library`] 判段交回的）。
#[derive(Debug)]
pub struct PairingInputAndReferenceReadBack {
    pub input: PairingInputReadBack,
    /// 参照一侧在录入段算出、写进库的结果。
    pub reference: PairingInputReadBack,
}

/// 对拍输入连同参照一侧的录入输入摘要：输入的摘要与参照一侧判法单元指纹拼起来取 SHA-256。参照的代码变了、或输入变了，录入键都变，
/// 库里旧的那一份参照不再被读到。
#[must_use]
pub fn recording_inputs_digest_of_the_pairing_input_and_the_reference_units(
    input_bytes: &[u8],
    reference_units: &JudgingUnitsKey,
) -> RecordingInputsDigest {
    RecordingInputsDigest(sha256_digest(
        format!(
            "singlefs pairing input with the reference\nreference_units={}\ninput={}\n",
            hexadecimal_text(reference_units.fingerprint()),
            hexadecimal_text(&sha256_digest(input_bytes))
        )
        .as_bytes(),
    ))
}

/// 一边是参照（在 CPU 上算）、一边要上卡的对拍用例用：录入段把输入写进库，调 `reference_of_the_input` 在 CPU 上算参照一侧的结果、也写进库，
/// 按录入键登清单；判段照同一份输入与参照一侧的判法单元现算录入键、读清单，输入与参照都从库里读回，持卡的那一段不再算参照。
/// 录入键照 [`recording_key_of_the_scenario_and_the_inputs`] 同一个算法（场景是项名），输入摘要取 [`recording_inputs_digest_of_the_pairing_input_and_the_reference_units`]；
/// `reference_units` 拒算时键只拼输入的摘要、照录、不凭命中跳过。
/// 只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn pairing_input_and_reference_through_the_pairing_library(
    item_name: &str,
    input_bytes: &[u8],
    reference_units: &Result<JudgingUnitsKey, String>,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> StageOutcome<PairingInputAndReferenceReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing)
        .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」开不了库：{refusal}"));
    let recording = recording_key_of_the_pairing_input_and_the_reference_units(
        item.as_str(),
        input_bytes,
        reference_units,
    );
    pairing_input_and_reference_through_the_library_under_the_recording_key(
        &library,
        RunStage::of_this_process(),
        &item,
        &recording,
        input_bytes,
        reference_of_the_input,
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的输入与参照过库失败：{refusal}"))
}

/// [`pairing_input_and_reference_through_the_pairing_library`] 的录入键：`scenario` 是场景标识（项名）。
#[must_use]
pub fn recording_key_of_the_pairing_input_and_the_reference_units(
    scenario: &str,
    input_bytes: &[u8],
    reference_units: &Result<JudgingUnitsKey, String>,
) -> RecordingKeyOfThisRun {
    match reference_units {
        Ok(reference_units) => recording_key_of_the_scenario_and_the_inputs(
            scenario,
            &recording_inputs_digest_of_the_pairing_input_and_the_reference_units(
                input_bytes,
                reference_units,
            ),
        ),
        Err(sentence) => RecordingKeyOfThisRun {
            skips_the_recording_on_a_hit: false,
            refusal: Some(format!("参照一侧的判法单元拒算：{sentence}")),
            ..recording_key_of_the_scenario_and_the_inputs(
                scenario,
                &RecordingInputsDigest(sha256_digest(input_bytes)),
            )
        },
    }
}

/// [`pairing_input_and_reference_through_the_pairing_library`] 的本体：库、选段与录入键由调用方给（单测在一次性的库上跑）。
///
/// # Errors
/// 写不进库；登清单、读清单不成；库里没有清单指到的那一份；读回的字节摘要对不上。
pub fn pairing_input_and_reference_through_the_library_under_the_recording_key(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    input_bytes: &[u8],
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<StageOutcome<PairingInputAndReferenceReadBack>, String> {
    let input_digest = sha256_digest(input_bytes);
    two_stages_under_the_recording_key(
        library,
        stage,
        item,
        recording,
        |library_recorded_into| {
            let reference_bytes = reference_of_the_input(input_bytes);
            let reference_digest = sha256_digest(&reference_bytes);
            let mut store = library_recorded_into.lock();
            store
                .store_pairing_input(&input_digest, input_bytes)
                .map_err(|error| format!("对拍输入写不进库：{error:?}"))?;
            store
                .store_pairing_input(&reference_digest, &reference_bytes)
                .map_err(|error| format!("对拍参照写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "input_pairing_with_the_reference".to_string(),
                digests: vec![
                    (PAIRING_MANIFEST_INPUT.to_string(), input_digest),
                    (PAIRING_MANIFEST_REFERENCE.to_string(), reference_digest),
                ],
            })
        },
        |library_read_from, manifest| {
            let store = library_read_from.lock();
            Ok(StageOutcome::Judged(PairingInputAndReferenceReadBack {
                input: read_pairing_input_back(
                    &store,
                    &manifest.digest_named(PAIRING_MANIFEST_INPUT)?,
                )?,
                reference: read_pairing_input_back(
                    &store,
                    &manifest.digest_named(PAIRING_MANIFEST_REFERENCE)?,
                )?,
            }))
        },
    )
}

#[cfg(feature = "gpu")]
/// 对拍用例的判表清单里判表那一样的名字。
const PAIRING_MANIFEST_JUDGE_TABLES: &str = "judge_tables";

#[cfg(feature = "gpu")]
/// GPU 判器与 CPU 判器对拍用例用：录入段把打包判表按摘要写进库、按录入键（判表摘要）登清单；判段照同一份判表现算录入键、读清单、
/// 按摘要读回，GPU 判器只收读回的那一份。只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn judge_tables_through_the_pairing_library(
    item_name: &str,
    tables: &PackedJudgeTables,
) -> StageOutcome<PackedJudgeTablesReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    two_stages(
        CrashLibraryClass::JudgeTablesPairing,
        &item,
        &RecordingInputsDigest(digest),
        |library| {
            library
                .lock()
                .store_packed_judge_tables(&digest, &bytes)
                .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "judge_tables_pairing".to_string(),
                digests: vec![(PAIRING_MANIFEST_JUDGE_TABLES.to_string(), digest)],
            })
        },
        |library, manifest| {
            Ok(StageOutcome::Judged(read_packed_judge_tables_back(
                &library.lock(),
                &manifest.digest_named(PAIRING_MANIFEST_JUDGE_TABLES)?,
            )?))
        },
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的判表过库失败：{refusal}"))
}

/// 一条对拍结论是从哪来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingConclusionSource {
    /// 库里同一键下存着，这一趟没判。
    ReusedFromTheLibrary,
    /// 这一趟判的（库里没有、或两边哪一边拒算），判完按键存进库（拒算的不存）。
    JudgedNow,
}

/// 判一次对拍交回的：结论的字节（写法由调用方定），与两边比下来一致还是不一致（调用方照它自己的比法判，[`PairingAgreement`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingJudged {
    pub conclusion: Vec<u8>,
    pub agreement: PairingAgreement,
}

/// 库里这一把键下存着的对拍结论能不能复用：两边一致的复用；记着不一致（红）的不复用、这一趟现判（照同一把键记红、撤掉原有的绿，
/// 键记着红的下一趟现判、不当绿复用：`.claude/rules/verification.md`「一个门禁：两种跑法」那一条）。
pub(super) fn stored_pairing_conclusion_is_reused(stored: &StoredPairingConclusion) -> bool {
    match stored.agreement {
        PairingAgreement::BothSidesAgree => true,
        PairingAgreement::SidesDisagree => false,
    }
}

/// 对拍结论的键：调用方按输入数据算的那一份输入摘要拼上判器引到的被检测产品代码（`judged_product_code`：CPU 参照那一侧调到的产品恢复、
/// 挂载与格式代码），取 SHA-256。改产品恢复，对拍重拍；两边判器与比法自己的代码不进。那一组拒算时拼的是这个进程的占位，每趟现判。
#[must_use]
pub fn pairing_conclusion_key_under_the_judged_product_code(
    inputs_digest: &[u8; DIGEST_BYTES],
    judged_product_code: &JudgedProductCodeOfThisBinary,
) -> PairingConclusionKey {
    let mut bytes = b"singlefs pairing conclusion\n".to_vec();
    bytes.extend_from_slice(inputs_digest);
    bytes.extend_from_slice(judged_product_code.key_text().as_bytes());
    PairingConclusionKey {
        inputs_digest: sha256_digest(&bytes),
    }
}

/// 这个二进制里一份对拍输入摘要的对拍结论键（[`pairing_conclusion_key_under_the_judged_product_code`] 交
/// [`judged_product_code_of_this_binary`]）。
#[must_use]
pub fn pairing_conclusion_key_of_the_inputs(
    inputs_digest: &[u8; DIGEST_BYTES],
) -> PairingConclusionKey {
    pairing_conclusion_key_under_the_judged_product_code(
        inputs_digest,
        judged_product_code_of_this_binary(),
    )
}

/// 对拍结论入库（「对拍结论入库」）：键是输入摘要拼判器引到的被检测产品代码（[`pairing_conclusion_key_of_the_inputs`]；两边判器与比法的代码
/// 不进键），库里有两边一致的就复用、不再判；
/// 没有、或记着不一致（红）的，调 `judge` 判一遍、照同一把键存进库（红的照样存：记红，同一把键原有的那一行被换掉）。
/// 交回结论的字节与它从哪来。
///
/// # Errors
/// 库读写失败。
pub fn pairing_conclusion_reused_or_judged(
    library: &CrashLibrary,
    inputs_digest: &[u8; DIGEST_BYTES],
    judge: impl FnOnce() -> PairingJudged,
) -> Result<(Vec<u8>, PairingConclusionSource), String> {
    let key = pairing_conclusion_key_of_the_inputs(inputs_digest);
    if let Some(stored) = library
        .lock()
        .read_pairing_conclusion(&key)
        .map_err(|error| format!("读对拍结论失败：{error:?}"))?
    {
        if stored_pairing_conclusion_is_reused(&stored) {
            return Ok((
                stored.conclusion,
                PairingConclusionSource::ReusedFromTheLibrary,
            ));
        }
        println!(
            "CRASH_LIBRARY_PAIRING_STORED_RED class={} inputs={}：库里这一把键记着两边不一致，这一趟现判",
            library.class.name(),
            hexadecimal_text(&inputs_digest[..8])
        );
    }
    let judged = judge();
    {
        let mut store = library.lock();
        store
            .store_pairing_conclusion(
                &key,
                &StoredPairingConclusion {
                    conclusion: judged.conclusion.clone(),
                    agreement: judged.agreement,
                    circumstances: JudgementCircumstances::of_this_process_now(),
                },
            )
            .map_err(|error| format!("对拍结论写不进库：{error:?}"))?;
    }
    Ok((judged.conclusion, PairingConclusionSource::JudgedNow))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 场景：两份输入摘要、判器引到的被检测产品代码两把指纹，两两配着取对拍结论的键。
    /// 预期：输入或判器引到的产品代码有一样不同，键就不同（改产品恢复，对拍重拍）；两样都相同，键相同（下一趟复用）。
    /// 判别力（`crates/mutations.tsv`）：键不拼判器引到的产品代码，这条红在「产品代码换一把」那一句。
    #[test]
    fn the_pairing_conclusion_key_takes_the_inputs_and_the_judged_product_code() {
        let judged = |byte: u8| JudgedProductCodeOfThisBinary::Keyed {
            fingerprint: [byte; DIGEST_BYTES],
        };
        let key = |inputs: u8, product_code: u8| {
            pairing_conclusion_key_under_the_judged_product_code(
                &[inputs; DIGEST_BYTES],
                &judged(product_code),
            )
        };
        assert_eq!(key(1, 7), key(1, 7), "两样都相同，键相同");
        assert_ne!(key(1, 7), key(2, 7), "输入摘要换一份，键就变");
        assert_ne!(
            key(1, 7),
            key(1, 8),
            "判器引到的被检测产品代码换一把，键就变"
        );
    }
    use crate::crash_identity::JudgingUnitFingerprint;

    /// 对拍输入两段过库：只录那一趟写进库、不交回输入；只判那一趟照同一份输入现算录入键、从库里读回同样的字节与摘要；
    /// 没录过的另一份输入只判时报库里没有清单。
    /// 判别力：只判那一趟不读库、直接交回调用方手里的字节，没录过的那一份就不报错，红在最后一条断言。
    #[test]
    fn a_pairing_input_is_recorded_in_one_run_and_read_back_from_the_library_in_another() {
        // 一次性库留着：自证对拍输入只录、只判两趟：要一份空的临时服务
        let disposable =
            DisposableLibrary::new("input-pairing").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了库");
        let item = ItemName::new("input-pairing").expect("项名合规矩");
        let recorded_input = b"units to verify and their expected checksums".to_vec();
        let recording_key_of = |input: &[u8]| {
            recording_key_of_the_scenario_and_the_inputs(
                item.as_str(),
                &RecordingInputsDigest(sha256_digest(input)),
            )
        };
        let recorded = pairing_input_through_the_library_under_the_recording_key(
            &library,
            RunStage::RecordOnly,
            &item,
            &recording_key_of(&recorded_input),
            &recorded_input,
        )
        .expect("只录那一趟写得进库");
        assert!(
            matches!(recorded, StageOutcome::RecordedOnly),
            "只录那一趟不交回输入：{recorded:?}"
        );
        let judged = pairing_input_through_the_library_under_the_recording_key(
            &library,
            RunStage::JudgeOnly,
            &item,
            &recording_key_of(&recorded_input),
            &recorded_input,
        )
        .expect("只判那一趟读得回");
        let StageOutcome::Judged(read_back) = judged else {
            panic!("只判那一趟交回读回的输入：{judged:?}")
        };
        assert_eq!(read_back.bytes(), recorded_input.as_slice());
        assert_eq!(read_back.digest(), &sha256_digest(&recorded_input));
        let never_recorded = b"another input nobody recorded".to_vec();
        assert!(
            pairing_input_through_the_library_under_the_recording_key(
                &library,
                RunStage::JudgeOnly,
                &item,
                &recording_key_of(&never_recorded),
                &never_recorded,
            )
            .is_err(),
            "没录过的输入只判：库里没有它的清单，报错、不现录"
        );
    }

    /// 对拍输入连同参照两段过库：只录那一趟算一次参照、写进库；只判那一趟不算参照，输入与参照都从库里读回；同一把键再录一趟命中、不再算参照；
    /// 参照一侧的判法单元换了一把，录入键变，只判读不到旧的那一份。
    /// 判别力：录入键不拼参照一侧的单元指纹，换了一把之后只判照样读到旧参照，红在最后一条断言；判段现算参照，「只判那一趟不算参照」那一条红。
    #[test]
    fn a_pairing_reference_is_computed_once_at_the_recording_and_keyed_by_the_reference_units() {
        // 一次性库留着：自证对拍参照只录、只判两趟：要一份空的临时服务
        let disposable = DisposableLibrary::new("input-pairing-reference")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了库");
        let units_fingerprinted = |fingerprint: u8| {
            Ok(JudgingUnitsKey::of_the_parts(
                [fingerprint; DIGEST_BYTES],
                vec![JudgingUnitFingerprint {
                    unit: "unit".to_string(),
                    fingerprint: [fingerprint; DIGEST_BYTES],
                }],
            ))
        };
        let item = ItemName::new("input-pairing-reference").expect("项名合规矩");
        let input = b"units to verify".to_vec();
        let times_the_reference_was_computed = std::cell::Cell::new(0_u32);
        let through_the_library =
            |stage: RunStage, reference_units: &Result<JudgingUnitsKey, String>| {
                pairing_input_and_reference_through_the_library_under_the_recording_key(
                    &library,
                    stage,
                    &item,
                    &recording_key_of_the_pairing_input_and_the_reference_units(
                        item.as_str(),
                        &input,
                        reference_units,
                    ),
                    &input,
                    |input_bytes| {
                        times_the_reference_was_computed
                            .set(times_the_reference_was_computed.get() + 1);
                        input_bytes.iter().rev().copied().collect()
                    },
                )
            };
        let reference_units = units_fingerprinted(11);
        let recorded = through_the_library(RunStage::RecordOnly, &reference_units)
            .expect("只录那一趟写得进库");
        assert!(matches!(recorded, StageOutcome::RecordedOnly));
        assert_eq!(
            times_the_reference_was_computed.get(),
            1,
            "录入段算一次参照"
        );
        let StageOutcome::Judged(read_back) =
            through_the_library(RunStage::JudgeOnly, &reference_units).expect("只判那一趟读得回")
        else {
            panic!("只判那一趟交回读回的输入与参照")
        };
        assert_eq!(
            times_the_reference_was_computed.get(),
            1,
            "只判那一趟不算参照"
        );
        assert_eq!(read_back.input.bytes(), input.as_slice());
        assert_eq!(
            read_back.reference.bytes(),
            input.iter().rev().copied().collect::<Vec<u8>>().as_slice()
        );
        assert!(matches!(
            through_the_library(RunStage::RecordOnly, &reference_units).expect("同一把键再录一趟"),
            StageOutcome::RecordedOnly
        ));
        assert_eq!(
            times_the_reference_was_computed.get(),
            1,
            "录入键命中：不再算参照"
        );
        let refused: Result<JudgingUnitsKey, String> = Err("样本：拒算".to_string());
        for _run in 0..2 {
            assert!(matches!(
                through_the_library(RunStage::RecordOnly, &refused).expect("参照一侧拒算照录"),
                StageOutcome::RecordedOnly
            ));
        }
        assert_eq!(
            times_the_reference_was_computed.get(),
            3,
            "参照一侧拒算：每一趟都照录、不凭命中跳过"
        );
        assert!(
            through_the_library(RunStage::JudgeOnly, &units_fingerprinted(12)).is_err(),
            "参照一侧的判法单元换了一把：录入键变，库里没有这把键的清单"
        );
    }

    /// 读回的对拍输入核摘要：字节与摘要对得上才造得出，差一个字节就拒。
    #[test]
    fn a_pairing_input_read_back_is_refused_when_its_bytes_do_not_match_the_digest() {
        let input = b"pairing input".to_vec();
        let digest = sha256_digest(&input);
        assert!(PairingInputReadBack::from_the_library_bytes(input.clone(), &digest).is_ok());
        let mut one_byte_changed = input;
        one_byte_changed[0] ^= 1;
        assert!(
            PairingInputReadBack::from_the_library_bytes(one_byte_changed, &digest).is_err(),
            "库里那份被改过一个字节：摘要对不上，拒"
        );
    }

    /// 对拍结论入库（L10）：同一份输入第二趟复用、不再判；换了输入摘要照判。
    /// 判别力（`crates/mutations.tsv`）：读库那一处一律当库里没有，第二趟照判、这条红。
    #[test]
    fn a_pairing_conclusion_is_reused_the_second_time_and_judged_again_when_the_inputs_change() {
        // 一次性库留着：自证对拍结论复用与重判：要从空库数判了几次
        let disposable =
            DisposableLibrary::new("pairing-reuse").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let judgements = std::cell::Cell::new(0u32);
        let judge = || {
            judgements.set(judgements.get() + 1);
            PairingJudged {
                conclusion: format!("agree {}", judgements.get()).into_bytes(),
                agreement: PairingAgreement::BothSidesAgree,
            }
        };
        let inputs = [1; DIGEST_BYTES];
        let first =
            pairing_conclusion_reused_or_judged(&library, &inputs, judge).expect("对拍得了");
        assert_eq!(
            first,
            (b"agree 1".to_vec(), PairingConclusionSource::JudgedNow)
        );
        let second =
            pairing_conclusion_reused_or_judged(&library, &inputs, judge).expect("对拍得了");
        assert_eq!(
            second,
            (
                b"agree 1".to_vec(),
                PairingConclusionSource::ReusedFromTheLibrary
            ),
            "第二趟复用库里那一条"
        );
        assert_eq!(judgements.get(), 1, "复用的那一趟不判");
        let other_inputs = pairing_conclusion_reused_or_judged(&library, &[2; DIGEST_BYTES], judge)
            .expect("对拍得了");
        assert_eq!(
            other_inputs.1,
            PairingConclusionSource::JudgedNow,
            "换了输入摘要照判"
        );
        assert_eq!(judgements.get(), 2);
    }

    /// 场景：同一份输入，头一趟判出两边不一致（红），第二趟起判出一致；连跑三趟。
    /// 预期：头一趟照这一把键记红（库里那一行是不一致的那一份）；第二趟不复用记着红的那一份、现判，判出一致的那一份换掉它；
    /// 第三趟复用一致的那一份、不判（照同一把键记红、撤掉原有的绿，键记着红的下一趟现判、不当绿复用）。
    /// 判别力（`crates/mutations.tsv`）：「对拍结论：记着红的照复用」「对拍结论：单条那一路问复用不看红绿」两行各让这一条红。
    #[test]
    fn a_pairing_conclusion_stored_red_is_judged_again_and_the_agreeing_one_replaces_it() {
        // 一次性库留着：要从空库数判了几次、读库里那一行记的红绿
        let disposable = DisposableLibrary::new("pairing-stored-red")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let inputs = [8; DIGEST_BYTES];
        let key = pairing_conclusion_key_of_the_inputs(&inputs);
        let agreement_stored = || {
            library
                .lock()
                .read_pairing_conclusion(&key)
                .expect("读得了库")
                .map(|stored| stored.agreement)
        };
        let judgements = std::cell::Cell::new(0u32);
        let judge = || {
            judgements.set(judgements.get() + 1);
            PairingJudged {
                conclusion: format!("judged {}", judgements.get()).into_bytes(),
                agreement: if judgements.get() == 1 {
                    PairingAgreement::SidesDisagree
                } else {
                    PairingAgreement::BothSidesAgree
                },
            }
        };
        let first =
            pairing_conclusion_reused_or_judged(&library, &inputs, judge).expect("对拍得了");
        assert_eq!(
            first,
            (b"judged 1".to_vec(), PairingConclusionSource::JudgedNow)
        );
        assert_eq!(
            agreement_stored(),
            Some(PairingAgreement::SidesDisagree),
            "头一趟判红：照这一把键记红"
        );
        let second =
            pairing_conclusion_reused_or_judged(&library, &inputs, judge).expect("对拍得了");
        assert_eq!(
            second,
            (b"judged 2".to_vec(), PairingConclusionSource::JudgedNow),
            "库里记着红的不复用：第二趟现判"
        );
        assert_eq!(
            agreement_stored(),
            Some(PairingAgreement::BothSidesAgree),
            "现判出一致的那一份换掉记着红的那一份"
        );
        let third =
            pairing_conclusion_reused_or_judged(&library, &inputs, judge).expect("对拍得了");
        assert_eq!(
            third,
            (
                b"judged 2".to_vec(),
                PairingConclusionSource::ReusedFromTheLibrary
            ),
            "一致的那一份照复用"
        );
        assert_eq!(judgements.get(), 2, "复用的那一趟不判");
    }
}
