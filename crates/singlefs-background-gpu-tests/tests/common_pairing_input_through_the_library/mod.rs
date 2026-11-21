//! 真卡对拍的测试目标共用：一条用例的输入编成字节、过判定库（验证包 `singlefs_verification::crash_library::pairing_input_through_the_pairing_library`，
//! 录入键、清单、按内容摘要存都是验证包那一份逻辑，这里不另算键、不另存）。对拍分两段：录入段把输入写进库、登清单，只录那一趟到此返回、不开卡；
//! 判段照同一份输入现算录入键、从库里读回，开卡之后交给 GPU 的与算 CPU 参照用的都是读回的那一份。
//! 段由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 定（`record` 只录、`judge` 只判、不设是同一个进程里先录后判）。
//! 一边是 CPU 参照、一边要上卡的对拍，判段判完的结论也入库（[`pairing_concluded_on_the_cards_through_the_pairing_library`]）：交给验证包
//! `crash_library::pairing_conclusion_reused_or_judged` 的输入摘要是读回输入的摘要拼上参照一侧与 GPU 一侧的判法单元键
//! （[`pairing_inputs_digest_with_both_sides`]：两边是被测的产品代码与用例的持卡判入口，单测的复用键照旧收它们），库里有两边一致的复用、
//! 不领卡不开卡；没有、或记着红的，领卡、跑持卡判、照同一把键存（红的照存）；哪一边拒算照判、不入库。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::cell::Cell;

use crate::common_discrete_gpus_of_this_host::{
    note_the_cards_leased_on_demand_for_this_process,
    VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER,
};
use singlefs_verification::crash_identity::{
    judging_units_key_of_the_group_in_the_table_text, JudgingUnitsKey,
};
use singlefs_verification::crash_library::{
    pairing_conclusion_reused_or_judged,
    pairing_input_and_reference_through_the_library_under_the_recording_key,
    pairing_input_and_reference_through_the_pairing_library,
    pairing_input_through_the_pairing_library,
    recording_key_of_the_pairing_input_and_the_reference_units, CrashLibrary, CrashLibraryClass,
    ItemName, PairingConclusionSource, PairingJudged, RecordingKeyOfThisRun, RunStage,
    StageOutcome,
};
use singlefs_verification::gpu_card_configuration::{
    cards_handed_over_or_leased_on_demand, CardLeaseChannelOfTheCaller,
};
use singlefs_verification::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_verification::verdict_store::PairingAgreement;

/// 交给验证包对拍结论入口的输入摘要：读回输入的摘要、参照一侧与 GPU 一侧的判法单元指纹带标签接起来取 SHA-256。验证包的对拍结论键只是
/// 输入摘要；这两边是被测的产品代码（参照与 GPU 那一份实现）与用例的持卡判入口，单测的复用键照旧收它们，拼进输入摘要里：哪一边的代码变了，
/// 摘要跟着变、那一条现判。
#[must_use]
pub fn pairing_inputs_digest_with_both_sides(
    input_digest: &[u8; DIGEST_BYTES],
    reference_units: &JudgingUnitsKey,
    gpu_side_units: &JudgingUnitsKey,
) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs background gpu pairing inputs with both sides\n".to_vec();
    message.extend_from_slice(input_digest);
    message.extend_from_slice(reference_units.fingerprint());
    message.extend_from_slice(gpu_side_units.fingerprint());
    sha256_digest(&message)
}

/// 构建脚本写下的参照单元键表（`build.rs`，入口登记在 `src/reference_units_entries.rs`）。
const REFERENCE_UNITS_KEYS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/reference_units_keys.tsv"));

/// 构建脚本写下的 GPU 一侧单元键表（`build.rs`，入口登记在 `src/gpu_side_units_entries.rs`，组名就是用例名）。
pub const GPU_SIDE_UNITS_KEYS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/gpu_side_units_keys.tsv"));

/// 从库里读回的对拍输入与参照一侧在只录那一段算好的结果。
pub struct PairingInputAndReference {
    pub input: Vec<u8>,
    pub reference: Vec<u8>,
}

/// 一边是 CPU 参照、一边要上卡的对拍用例的头一步：`input_bytes_of_the_test` 造的输入过判定库，只录那一段另调 `reference_of_the_input`
/// 在 CPU 上算参照一侧的结果、一起写进库（验证包 `pairing_input_and_reference_through_the_pairing_library`）；录入键拼进参照一侧的判法单元键
/// （登记表里 `reference_units_group` 那一组，构建脚本算好编进来）。只录那一趟打一行 `RECORDED_ONLY`、交回 `None`（用例到此返回，不开卡）；
/// 判段交回从库里读回的输入与参照，持卡的那一段不再算参照。
///
/// # Panics
/// 库开不了、写不进、读不回。
#[must_use]
pub fn pairing_input_and_reference_read_back_through_the_pairing_library(
    test_name: &str,
    reference_units_group: &str,
    input_bytes_of_the_test: impl FnOnce() -> Vec<u8>,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Option<PairingInputAndReference> {
    let input_bytes = input_bytes_of_the_test();
    let reference_units = judging_units_key_of_the_group_in_the_table_text(
        REFERENCE_UNITS_KEYS_TEXT,
        reference_units_group,
    );
    match pairing_input_and_reference_through_the_pairing_library(
        test_name,
        &input_bytes,
        &reference_units,
        reference_of_the_input,
    ) {
        StageOutcome::RecordedOnly => {
            println!(
                "RECORDED_ONLY test={test_name} input_bytes={} reference_units={reference_units_group}",
                input_bytes.len()
            );
            None
        }
        StageOutcome::Judged(read_back) => {
            lease_the_cards_on_demand_before_the_card_step(test_name);
            Some(PairingInputAndReference {
                input: read_back.input.bytes().to_vec(),
                reference: read_back.reference.bytes().to_vec(),
            })
        }
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => panic!(
            "对拍输入过库不分片，不该交回只判了这一台那一份（这台 {blocks_judged_here} 块、别的台 {blocks_left_to_other_hosts} 块）"
        ),
    }
}

/// 判段读回输入之后、开卡之前调：环境变量没给卡（服务起跑没交）而有领卡通道时，这一刻才经验证包的领卡通道领
/// （`singlefs_verification::gpu_card_configuration` 的 `cards_handed_over_or_leased_on_demand`，进程里只领一回、进程退出时还），
/// 领到的记给取卡那一份（`common_discrete_gpus_of_this_host`）。只录那一趟走不到这里，不领卡。要先 `mod common_discrete_gpus_of_this_host;`。
///
/// # Panics
/// 领卡没成（被拒、找不到领卡客户端）：运行条件不满足，不退回「本次未跑」。
fn lease_the_cards_on_demand_before_the_card_step(test_name: &str) {
    match cards_handed_over_or_leased_on_demand(
        std::env::var(VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER).ok(),
        CardLeaseChannelOfTheCaller::read_from_the_environment_now().as_ref(),
        &|channel| channel.cards_leased_by_this_process().map(str::to_string),
    ) {
        Ok(Some(cards)) => note_the_cards_leased_on_demand_for_this_process(&cards),
        Ok(None) => {}
        Err(refusal) => panic!("对拍用例「{test_name}」判段要开卡，按需领卡没成：{refusal}"),
    }
}

/// 持卡判那一段在这一台的卡上没跑成（服务没交这一台的卡、或这张卡撑不起这一条，打过 `NOT_RUN_NO_GPU_ADAPTER`）时带出去的记号：
/// [`judgement_of_the_assertions_on_the_cards`] 照它 `panic_any`，[`pairing_concluded_on_the_cards_in_the_library`] 认出它就当这一趟没判、结论不入库
/// （写法同验证包 GPU 判器对拍用例的 `PairingNotRun`）。
pub struct PairingNotRunOnTheseCards;

/// 持卡判那一段（开卡、跑 GPU、比）跑下来的样子。
pub enum ComparisonOnTheCards {
    /// 开了卡、跑完、每一处比都过了。
    EveryComparisonHeld,
    /// 服务没交这一台的卡、或这张卡撑不起这一条（打过 `NOT_RUN_NO_GPU_ADAPTER`）：没比。
    NotRunOnTheseCards,
}

/// panic 带出来的文字（`panic!` 与断言带的是 `String` 或 `&str`）。
fn text_of_the_panic_payload(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&'static str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "panic 带的不是文字".to_string())
}

/// 每条用例的持卡判入口（登记在 `src/gpu_side_units_entries.rs`）都经它交回判定：`run_and_compare_on_the_cards` 照常开卡、跑 GPU、用断言比，
/// 断言都过了交回两边一致（结论是一句「每一处比都过了」）；有一处 panic（断言不过、开卡判红这一类）交回两边不一致，结论是 panic 带的那一句；
/// 交回 [`ComparisonOnTheCards::NotRunOnTheseCards`] 的照 [`PairingNotRunOnTheseCards`] `panic_any`（没判，不入库）。它在每条持卡判入口的闭包里，
/// 比法改了 GPU 一侧的键跟着变。
///
/// # Panics
/// 没跑成时照上面带记号 panic。
pub fn judgement_of_the_assertions_on_the_cards(
    test_name: &str,
    run_and_compare_on_the_cards: impl FnOnce() -> ComparisonOnTheCards,
) -> PairingJudged {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_and_compare_on_the_cards)) {
        Ok(ComparisonOnTheCards::EveryComparisonHeld) => PairingJudged {
            conclusion: format!("{test_name}：持卡那一段每一处比都过了").into_bytes(),
            agreement: PairingAgreement::BothSidesAgree,
        },
        Ok(ComparisonOnTheCards::NotRunOnTheseCards) => {
            std::panic::panic_any(PairingNotRunOnTheseCards)
        }
        Err(payload) => PairingJudged {
            conclusion: format!("{test_name}：{}", text_of_the_panic_payload(&*payload))
                .into_bytes(),
            agreement: PairingAgreement::SidesDisagree,
        },
    }
}

/// 一条真卡对拍用例过库与结论入库用的三把键：录入键（输入连同参照一侧的判法单元）、参照一侧与 GPU 一侧的判法单元键（拒算的是 `Err`）。
pub struct KeysOfAPairingOnTheCards {
    pub recording: RecordingKeyOfThisRun,
    pub reference_units: Result<JudgingUnitsKey, String>,
    pub gpu_side_units: Result<JudgingUnitsKey, String>,
}

/// 一条真卡对拍用例这一趟的结局（判出两边不一致的不交回：结论记红之后 panic）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingOnTheCardsOutcome {
    /// 只录那一趟：输入与参照写进库，不领卡、不开卡。
    RecordedOnly,
    /// 库里这一把键下有两边一致的结论：不领卡、不开卡、不判。
    ConclusionReusedFromTheLibrary,
    /// 这一趟领卡判的，两边一致，结论照这一把键存进库（哪一边拒算的不存）。
    JudgedNowAndBothSidesAgree,
    /// 持卡判没跑成（打过 `NOT_RUN_NO_GPU_ADAPTER`）：没判，不入库，下一趟照判。
    NotRunOnTheseCards,
}

/// [`pairing_concluded_on_the_cards_through_the_pairing_library`] 的本体：库、选段与三把键由调用方给（不上卡的那几条测试在一次性的库上跑）。
/// 输入与参照照录入键过库（验证包 `pairing_input_and_reference_through_the_library_under_the_recording_key`）；只录那一趟打一行
/// `RECORDED_ONLY test=<用例名> input_bytes=<字节数>`、到此返回。判段拿读回的输入摘要、参照一侧与 GPU 一侧的键问库里的对拍结论
/// （验证包 `pairing_conclusion_reused_or_judged`）：两边一致的复用、打一行 `PAIRING_CONCLUSION_REUSED test=<用例名>`，`judge_on_the_cards` 不调；
/// 要现判的才调它（收读回的输入与参照），判出一致打一行 `PAIRING_CONCLUSION_JUDGED test=<用例名> agreement=both_sides_agree`。
///
/// # Panics
/// 项名不合规矩；库写不进、读不回；判出两边不一致（结论已照这一把键记红，消息带用例名与结论的文字）。
pub fn pairing_concluded_on_the_cards_in_the_library(
    library: &CrashLibrary,
    stage: RunStage,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    input_bytes: &[u8],
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
    judge_on_the_cards: impl FnOnce(&[u8], &[u8]) -> PairingJudged,
) -> PairingOnTheCardsOutcome {
    let item =
        ItemName::new(test_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let read_back = match pairing_input_and_reference_through_the_library_under_the_recording_key(
        library,
        stage,
        &item,
        &keys.recording,
        input_bytes,
        reference_of_the_input,
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{test_name}」的输入与参照过库失败：{refusal}"))
    {
        StageOutcome::RecordedOnly => {
            println!(
                "RECORDED_ONLY test={test_name} input_bytes={}",
                input_bytes.len()
            );
            return PairingOnTheCardsOutcome::RecordedOnly;
        }
        StageOutcome::Judged(read_back) => read_back,
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => panic!(
            "对拍输入过库不分片，不该交回只判了这一台那一份（这台 {blocks_judged_here} 块、别的台 {blocks_left_to_other_hosts} 块）"
        ),
    };
    let agreement_judged_now = Cell::new(None);
    let judge_now = || {
        let judged = judge_on_the_cards(read_back.input.bytes(), read_back.reference.bytes());
        agreement_judged_now.set(Some(judged.agreement));
        judged
    };
    let concluded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match (&keys.reference_units, &keys.gpu_side_units) {
            (Ok(reference_units), Ok(gpu_side_units)) => pairing_conclusion_reused_or_judged(
                library,
                &pairing_inputs_digest_with_both_sides(
                    read_back.input.digest(),
                    reference_units,
                    gpu_side_units,
                ),
                judge_now,
            ),
            (Err(_), _) | (_, Err(_)) => {
                println!("PAIRING_KEY_REFUSED test={test_name}：判了、不入库");
                Ok((judge_now().conclusion, PairingConclusionSource::JudgedNow))
            }
        }
    }));
    let (conclusion, source) = match concluded {
        Ok(Ok(concluded)) => concluded,
        Ok(Err(reason)) => panic!("对拍用例「{test_name}」的对拍结论读写库失败：{reason}"),
        Err(payload) if payload.is::<PairingNotRunOnTheseCards>() => {
            return PairingOnTheCardsOutcome::NotRunOnTheseCards;
        }
        Err(payload) => std::panic::resume_unwind(payload),
    };
    match source {
        PairingConclusionSource::ReusedFromTheLibrary => {
            println!("PAIRING_CONCLUSION_REUSED test={test_name}");
            PairingOnTheCardsOutcome::ConclusionReusedFromTheLibrary
        }
        PairingConclusionSource::JudgedNow => match agreement_judged_now.get() {
            Some(PairingAgreement::BothSidesAgree) => {
                println!("PAIRING_CONCLUSION_JUDGED test={test_name} agreement=both_sides_agree");
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree
            }
            Some(PairingAgreement::SidesDisagree) => panic!(
                "对拍用例「{test_name}」判出两边不一致（结论已照这一把键记红，下一趟现判）：{}",
                String::from_utf8_lossy(&conclusion)
            ),
            None => panic!(
                "对拍用例「{test_name}」的结论是这一趟现判的，持卡判却没交回判定：验证包的对拍结论入口没调判法就交回了现判"
            ),
        },
    }
}

/// GPU 一侧登记表里这条用例那一组的判法单元键（组名就是用例名）；组算不出（拒算）交回 `Err`，照验证包的规矩照判、不入库。
///
/// # Panics
/// 登记表里没有这条用例那一组：每条真卡对拍用例都要登记自己的持卡判入口。
fn gpu_side_units_key_of_the_test(test_name: &str) -> Result<JudgingUnitsKey, String> {
    let group_line_start = format!("group\t{test_name}\t");
    assert!(
        GPU_SIDE_UNITS_KEYS_TEXT
            .lines()
            .any(|line| line.starts_with(&group_line_start)),
        "GPU 一侧登记表（src/gpu_side_units_entries.rs）里没有用例「{test_name}」那一组：每条真卡对拍用例都要登记自己的持卡判入口"
    );
    judging_units_key_of_the_group_in_the_table_text(GPU_SIDE_UNITS_KEYS_TEXT, test_name)
}

/// 一边是 CPU 参照、一边要上卡的真卡对拍用例走这一条：`input_bytes_of_the_test` 造的输入连同 `reference_of_the_input` 在 CPU 上算的参照过判定库
/// （录入键拼进登记表里 `reference_units_group` 那一组的判法单元键），判段读回之后对拍结论入库、按键复用（[`pairing_concluded_on_the_cards_in_the_library`]）。
/// 要现判的那一刻才按需领卡（`lease_the_cards_on_demand_before_the_card_step`），再调 `judged_on_the_cards`（这条用例的持卡判入口：收用例名、读回的输入与参照，
/// 开卡、跑 GPU、比，交回判定）；复用的那一趟不领卡、不开卡，找不到显卡也照通过。
///
/// # Panics
/// 库开不了、写不进、读不回；GPU 一侧登记表里没有这条用例；判出两边不一致；领卡没成。
pub fn pairing_concluded_on_the_cards_through_the_pairing_library(
    test_name: &str,
    reference_units_group: &str,
    input_bytes_of_the_test: impl FnOnce() -> Vec<u8>,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
    judged_on_the_cards: impl FnOnce(&str, &[u8], &[u8]) -> PairingJudged,
) {
    let input_bytes = input_bytes_of_the_test();
    let reference_units = judging_units_key_of_the_group_in_the_table_text(
        REFERENCE_UNITS_KEYS_TEXT,
        reference_units_group,
    );
    let gpu_side_units = gpu_side_units_key_of_the_test(test_name);
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing)
        .unwrap_or_else(|refusal| panic!("对拍用例「{test_name}」开不了库：{refusal}"));
    let keys = KeysOfAPairingOnTheCards {
        recording: recording_key_of_the_pairing_input_and_the_reference_units(
            test_name,
            &input_bytes,
            &reference_units,
        ),
        reference_units,
        gpu_side_units,
    };
    pairing_concluded_on_the_cards_in_the_library(
        &library,
        RunStage::of_this_process(),
        test_name,
        &keys,
        &input_bytes,
        reference_of_the_input,
        |input_read_back, reference_read_back| {
            lease_the_cards_on_demand_before_the_card_step(test_name);
            judged_on_the_cards(test_name, input_read_back, reference_read_back)
        },
    );
}

/// 一条对拍用例的头一步：`input_bytes_of_the_test` 造的输入过判定库，项名用用例名。只录那一趟打一行
/// `RECORDED_ONLY test=<用例名> input_bytes=<字节数>`、交回 `None`（用例到此返回，不开卡）；判段交回从库里读回的字节。
///
/// # Panics
/// 库开不了、写不进、读不回。
#[must_use]
pub fn pairing_input_read_back_through_the_pairing_library(
    test_name: &str,
    input_bytes_of_the_test: impl FnOnce() -> Vec<u8>,
) -> Option<Vec<u8>> {
    let input_bytes = input_bytes_of_the_test();
    match pairing_input_through_the_pairing_library(test_name, &input_bytes) {
        StageOutcome::RecordedOnly => {
            println!(
                "RECORDED_ONLY test={test_name} input_bytes={}",
                input_bytes.len()
            );
            None
        }
        StageOutcome::Judged(input_read_back) => {
            lease_the_cards_on_demand_before_the_card_step(test_name);
            Some(input_read_back.bytes().to_vec())
        }
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => panic!(
            "对拍输入过库不分片，不该交回只判了这一台那一份（这台 {blocks_judged_here} 块、别的台 {blocks_left_to_other_hosts} 块）"
        ),
    }
}

/// 往对拍输入的字节里追加一个数（8 字节小端）。
pub fn append_number(input_bytes: &mut Vec<u8>, number: u64) {
    input_bytes.extend_from_slice(&number.to_le_bytes());
}

/// 往对拍输入的字节里追加一个个数或长度（8 字节小端）。
pub fn append_count(input_bytes: &mut Vec<u8>, count: usize) {
    append_number(
        input_bytes,
        u64::try_from(count).expect("个数与长度装得进 u64"),
    );
}

/// 往对拍输入的字节里追加一段带长度的字节：长度（8 字节小端）接字节本身。
pub fn append_length_prefixed_bytes(input_bytes: &mut Vec<u8>, bytes: &[u8]) {
    append_count(input_bytes, bytes.len());
    input_bytes.extend_from_slice(bytes);
}

/// 从库里读回的对拍输入，从头往后取；取完调 [`PairingInputBeingRead::finish`] 核一个字节都没剩。
pub struct PairingInputBeingRead<'input> {
    remaining: &'input [u8],
}

impl<'input> PairingInputBeingRead<'input> {
    #[must_use]
    pub fn from_the_start_of(input_bytes: &'input [u8]) -> Self {
        Self {
            remaining: input_bytes,
        }
    }

    /// 取 `byte_count` 个字节。
    ///
    /// # Panics
    /// 剩下的不够。
    pub fn take_bytes(&mut self, byte_count: usize) -> &'input [u8] {
        assert!(
            byte_count <= self.remaining.len(),
            "读回的对拍输入不够：要取 {byte_count} 字节，只剩 {} 字节",
            self.remaining.len()
        );
        let (taken, rest) = self.remaining.split_at(byte_count);
        self.remaining = rest;
        taken
    }

    /// 取一个数（[`append_number`] 写下的）。
    pub fn take_number(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes(8).try_into().expect("刚取的正好 8 字节"))
    }

    /// 取一个个数或长度（[`append_count`] 写下的）。
    pub fn take_count(&mut self) -> usize {
        usize::try_from(self.take_number()).expect("写下的个数装得进 usize")
    }

    /// 取一段带长度的字节（[`append_length_prefixed_bytes`] 写下的）。
    pub fn take_length_prefixed_bytes(&mut self) -> &'input [u8] {
        let length = self.take_count();
        self.take_bytes(length)
    }

    /// # Panics
    /// 取完还有剩：录的与读的布局对不上。
    pub fn finish(self) {
        assert!(
            self.remaining.is_empty(),
            "读回的对拍输入取完还剩 {} 字节：录的与读的布局对不上",
            self.remaining.len()
        );
    }
}

/// 一个池每块盘写过的扇区编成对拍输入的字节：盘数，逐块盘是盘号、扇区数，逐个扇区是扇区号与带长度的字节。数都是 8 字节小端。
#[must_use]
pub fn pairing_input_bytes_of_the_written_sectors(
    written_sectors: &[(
        singlefs_core::address::DeviceIdentity,
        std::collections::BTreeMap<u64, Vec<u8>>,
    )],
) -> Vec<u8> {
    let mut input_bytes = Vec::new();
    append_count(&mut input_bytes, written_sectors.len());
    for (device, sectors) in written_sectors {
        append_number(&mut input_bytes, u64::from(device.0));
        append_count(&mut input_bytes, sectors.len());
        for (sector, sector_bytes) in sectors {
            append_number(&mut input_bytes, *sector);
            append_length_prefixed_bytes(&mut input_bytes, sector_bytes);
        }
    }
    input_bytes
}

/// [`pairing_input_bytes_of_the_written_sectors`] 的反向。
///
/// # Panics
/// 字节与这份布局对不上（短了、取完还有剩）。
#[must_use]
pub fn written_sectors_of_the_pairing_input_bytes(
    input_bytes: &[u8],
) -> Vec<(
    singlefs_core::address::DeviceIdentity,
    std::collections::BTreeMap<u64, Vec<u8>>,
)> {
    let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
    let device_count = input.take_count();
    let written_sectors = (0..device_count)
        .map(|_device| {
            let device = singlefs_core::address::DeviceIdentity(
                u32::try_from(input.take_number()).expect("写下的盘号装得进 u32"),
            );
            let sector_count = input.take_count();
            let sectors = (0..sector_count)
                .map(|_sector| {
                    let sector = input.take_number();
                    (sector, input.take_length_prefixed_bytes().to_vec())
                })
                .collect();
            (device, sectors)
        })
        .collect();
    input.finish();
    written_sectors
}
