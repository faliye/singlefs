//! 真卡对拍的判定库存取（`common_pairing_input_through_the_library`：产品测试自己一族的键，经 prune-server 的容错客户端读写）不上卡就核得了的那几件：
//! 持卡判换成一个只比字节的假判法，在这个进程里起的临时 prune-server 上跑（`PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process`，不连真服务）——
//! 同一份输入、同几把键第二趟录入复用清单、参照不再算；参照一侧的键或输入的一个字节变了照录；同两边的键第二趟结论复用、持卡判不被调；GPU 一侧的键变了照判；
//! 判出不一致的记红、用例 panic、下一趟照判；持卡判没跑成的不入库、下一趟照判；交来的卡换一版驱动照判，驱动没报驱动信息的照判、不入库，
//! 没交到卡的既不复用也不判；临时服务取不到这一台的地址不起服务、报错带出路，
//! 绑的地址是这一台的地址接端口 0。另核三把键各随哪几样变（录入键只由场景标识与输入摘要组成，代码一个单元都不进），与测试支撑包的构建脚本
//! 编进来的 GPU 一侧键表：每条登记的真卡对拍用例一组、都算出了键、闭包收到产品 GPU 包里它走到的核。
//! 不要卡、不挂特性，`cargo test -p singlefs-background-gpu --test <本目标>` 就能跑。
mod common_discrete_gpus_of_this_host;
mod common_pairing_input_through_the_library;
mod common_pairing_side_units_entries;

use std::cell::Cell;
use std::collections::BTreeSet;

use common_pairing_side_units_entries::{registered_groups_of_the_side, PairingSideOfTheEntries};

use common_pairing_input_through_the_library::{
    adapters_of_the_handed_over_gpus, conclusion_key, judgement_of_the_assertions_on_the_cards,
    network_address_of_a_temporary_service_on, pairing_concluded_on_the_cards_in_the_library,
    recording_inputs_digest_of_the_input,
    recording_inputs_digest_of_the_input_and_the_reference_units,
    recording_key_of_the_pairing_input_and_the_reference_units,
    recording_key_of_the_scenario_and_the_inputs, AdapterOfAHandedOverGpu,
    AdaptersSeenBeforeTheConclusion, ComparisonOnTheCards, KeysOfAPairingOnTheCards, PairingJudged,
    PairingLibraryOfTheProductTests, PairingOnTheCardsOutcome, PairingSideUnitsKey, RecordingKey,
    RunStage, GPU_SIDE_UNITS_KEYS_TEXT,
};
use singlefs_unit_graph::sha256::{sha256_of, Sha256Digest, DIGEST_BYTES};

/// 假的对拍输入：持卡判与参照只拿它的字节比。
const INPUT_OF_THE_PAIRING: &[u8] = b"input of a pairing on the cards";

/// 假的对拍用例名：录入键里的场景标识。
const SCENARIO_OF_THE_PAIRING: &str = "a_pairing_on_the_cards";

/// 造一把判法单元键：指纹是 `fingerprint_byte` 铺满（只当键用）。
fn units_key_filled_with(fingerprint_byte: u8) -> Result<PairingSideUnitsKey, String> {
    Ok(PairingSideUnitsKey {
        fingerprint: Sha256Digest([fingerprint_byte; DIGEST_BYTES]),
    })
}

/// 输入 `input`、参照一侧键 `reference_byte`、GPU 一侧键 `gpu_side_byte` 的三把键（录入键的场景是 [`SCENARIO_OF_THE_PAIRING`]）。
fn keys_of_the_input_with_the_sides(
    input: &[u8],
    reference_byte: u8,
    gpu_side_byte: u8,
) -> KeysOfAPairingOnTheCards {
    let reference_units = units_key_filled_with(reference_byte);
    KeysOfAPairingOnTheCards {
        recording: recording_key_of_the_pairing_input_and_the_reference_units(
            SCENARIO_OF_THE_PAIRING,
            input,
            &reference_units,
        ),
        reference_units,
        gpu_side_units: units_key_filled_with(gpu_side_byte),
    }
}

/// 输入是 [`INPUT_OF_THE_PAIRING`]，参照一侧与 GPU 一侧的键由用例给。
fn keys_with_the_sides(reference_byte: u8, gpu_side_byte: u8) -> KeysOfAPairingOnTheCards {
    keys_of_the_input_with_the_sides(INPUT_OF_THE_PAIRING, reference_byte, gpu_side_byte)
}

/// 一张交来的卡的适配器描述：驱动信息由用例给（空串是驱动没报）。
fn adapter_with_the_driver_information(driver_information: &str) -> AdapterOfAHandedOverGpu {
    AdapterOfAHandedOverGpu {
        name: "card of the pairing tests".to_string(),
        vendor: 0x10DE,
        device: 0x2B85,
        driver: "NVIDIA".to_string(),
        driver_information: driver_information.to_string(),
        backend: "Vulkan".to_string(),
    }
}

/// 判段问库之前看到的交来的卡：一张、驱动信息是 `driver_information`。
fn adapters_seen_with_the_driver_information(
    driver_information: &str,
) -> AdaptersSeenBeforeTheConclusion {
    AdaptersSeenBeforeTheConclusion::Seen(adapters_of_the_handed_over_gpus(&[
        adapter_with_the_driver_information(driver_information),
    ]))
}

/// 用例里默认交来的那张卡的驱动信息。
const DRIVER_INFORMATION_OF_THE_CARD: &str = "580.95.05";

/// 假的 CPU 参照：输入倒过来。
fn reversed_input(input_bytes: &[u8]) -> Vec<u8> {
    input_bytes.iter().rev().copied().collect()
}

/// 在临时服务上照先录后判跑一趟：参照是倒过来的输入，交来一张驱动信息是 [`DRIVER_INFORMATION_OF_THE_CARD`] 的卡，持卡判是 `judge_on_the_cards`。
fn one_run_on_the_temporary_library(
    library: &PairingLibraryOfTheProductTests,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    judge_on_the_cards: impl FnOnce(&[u8], &[u8]) -> PairingJudged,
) -> PairingOnTheCardsOutcome {
    one_run_on_the_temporary_library_with_the_adapters(
        library,
        test_name,
        keys,
        adapters_seen_with_the_driver_information(DRIVER_INFORMATION_OF_THE_CARD),
        judge_on_the_cards,
    )
}

/// 同 [`one_run_on_the_temporary_library`]，判段问库之前看到的交来的卡由用例给。
fn one_run_on_the_temporary_library_with_the_adapters(
    library: &PairingLibraryOfTheProductTests,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    adapters: AdaptersSeenBeforeTheConclusion,
    judge_on_the_cards: impl FnOnce(&[u8], &[u8]) -> PairingJudged,
) -> PairingOnTheCardsOutcome {
    pairing_concluded_on_the_cards_in_the_library(
        library,
        RunStage::RecordThenJudge,
        test_name,
        keys,
        INPUT_OF_THE_PAIRING,
        reversed_input,
        || adapters,
        judge_on_the_cards,
    )
}

/// 在临时服务上只录一趟（不判）：输入是 [`INPUT_OF_THE_PAIRING`]，见 [`one_recording_of_the_input_on_the_temporary_library`]。
fn one_recording_on_the_temporary_library(
    library: &PairingLibraryOfTheProductTests,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    references_computed: &Cell<u32>,
) -> PairingOnTheCardsOutcome {
    one_recording_of_the_input_on_the_temporary_library(
        library,
        test_name,
        keys,
        INPUT_OF_THE_PAIRING,
        references_computed,
    )
}

/// 在临时服务上把 `input` 只录一趟（不判）：参照是倒过来的输入，每算一次参照 `references_computed` 加一；持卡判调到就是错。
fn one_recording_of_the_input_on_the_temporary_library(
    library: &PairingLibraryOfTheProductTests,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    input: &[u8],
    references_computed: &Cell<u32>,
) -> PairingOnTheCardsOutcome {
    pairing_concluded_on_the_cards_in_the_library(
        library,
        RunStage::RecordOnly,
        test_name,
        keys,
        input,
        |input_bytes: &[u8]| {
            references_computed.set(references_computed.get() + 1);
            reversed_input(input_bytes)
        },
        || -> AdaptersSeenBeforeTheConclusion { panic!("只录那一趟不看交来的卡") },
        |_input_read_back: &[u8], _reference_read_back: &[u8]| -> PairingJudged {
            panic!("只录那一趟不调持卡判")
        },
    )
}

/// 假的持卡判：数自己被调了几次，照常断言读回的参照是倒过来的读回输入。
fn judge_counted_that_compares_the_reversed_input<'counter>(
    test_name: &'counter str,
    judgements: &'counter Cell<u32>,
) -> impl FnOnce(&[u8], &[u8]) -> PairingJudged + 'counter {
    move |input_read_back: &[u8], reference_read_back: &[u8]| {
        judgements.set(judgements.get() + 1);
        judgement_of_the_assertions_on_the_cards(test_name, || {
            assert_eq!(
                reversed_input(input_read_back),
                reference_read_back,
                "读回的参照是倒过来的读回输入"
            );
            ComparisonOnTheCards::EveryComparisonHeld
        })
    }
}

/// 场景：同一份输入、同几把键只录两趟。
/// 预期：两趟都只录、不判；参照只在头一趟算一次（第二趟照录入键找到清单，不再录）。
/// 判别力（`crates/mutations.tsv`）：「产品测试判定库：录入键命中也照录」那一行让这一条红。
#[test]
fn the_second_recording_of_the_same_input_under_the_same_keys_reuses_the_manifest_without_computing_the_reference_again(
) {
    let test_name = "the_second_recording_of_the_same_input_under_the_same_keys_reuses_the_manifest_without_computing_the_reference_again";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-recording-reused",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let references_computed = Cell::new(0_u32);
    let outcomes = [
        one_recording_on_the_temporary_library(&library, test_name, &keys, &references_computed),
        one_recording_on_the_temporary_library(&library, test_name, &keys, &references_computed),
    ];
    assert_eq!(
        (outcomes, references_computed.get()),
        (
            [
                PairingOnTheCardsOutcome::RecordedOnly,
                PairingOnTheCardsOutcome::RecordedOnly
            ],
            1
        ),
        "第二趟录入复用清单：参照只算一次"
    );
}

/// 场景：同一份输入只录两趟，参照一侧的键（参照在 CPU 上算的那段代码的判法单元）第二趟换一把。
/// 预期：两趟各算一次参照（录入键拼着参照一侧的指纹，参照的代码变了，库里旧的清单不再被读到）。
/// 判别力（`crates/mutations.tsv`）：「产品测试判定库：参照一侧的键不进录入键」那一行让这一条红。
#[test]
fn a_changed_reference_side_key_records_the_input_and_its_reference_again() {
    let test_name = "a_changed_reference_side_key_records_the_input_and_its_reference_again";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-reference-side-changed",
    );
    let references_computed = Cell::new(0_u32);
    let outcomes = [
        one_recording_on_the_temporary_library(
            &library,
            test_name,
            &keys_with_the_sides(0x11, 0x22),
            &references_computed,
        ),
        one_recording_on_the_temporary_library(
            &library,
            test_name,
            &keys_with_the_sides(0x33, 0x22),
            &references_computed,
        ),
    ];
    assert_eq!(
        (outcomes, references_computed.get()),
        (
            [
                PairingOnTheCardsOutcome::RecordedOnly,
                PairingOnTheCardsOutcome::RecordedOnly
            ],
            2
        ),
        "参照一侧的键换了：第二趟照录"
    );
}

/// 场景：只录两趟，参照一侧与 GPU 一侧的键不变，第二趟的输入改了一个字节。
/// 预期：两趟各算一次参照（录入键拼着输入的摘要，输入变了，库里旧的清单不再被读到）。
/// 判别力（`crates/mutations.tsv`）：「产品测试判定库：录入键不拼输入摘要」那一行让这一条红。
#[test]
fn an_input_changed_by_one_byte_records_the_input_and_its_reference_again() {
    let test_name = "an_input_changed_by_one_byte_records_the_input_and_its_reference_again";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-input-changed-by-one-byte",
    );
    let mut input_changed_by_one_byte = INPUT_OF_THE_PAIRING.to_vec();
    input_changed_by_one_byte[0] ^= 0x01;
    let references_computed = Cell::new(0_u32);
    let outcomes = [
        one_recording_of_the_input_on_the_temporary_library(
            &library,
            test_name,
            &keys_of_the_input_with_the_sides(INPUT_OF_THE_PAIRING, 0x11, 0x22),
            INPUT_OF_THE_PAIRING,
            &references_computed,
        ),
        one_recording_of_the_input_on_the_temporary_library(
            &library,
            test_name,
            &keys_of_the_input_with_the_sides(&input_changed_by_one_byte, 0x11, 0x22),
            &input_changed_by_one_byte,
            &references_computed,
        ),
    ];
    assert_eq!(
        (outcomes, references_computed.get()),
        (
            [
                PairingOnTheCardsOutcome::RecordedOnly,
                PairingOnTheCardsOutcome::RecordedOnly
            ],
            2
        ),
        "输入改了一个字节：第二趟照录"
    );
}

/// 场景：照录入键的算法算一条只有输入的对拍与一条一边是参照的对拍的录入键，与照组成现拼的那一把比（不连库）。
/// 预期：两把都恰好是「场景标识 ‖ 录入输入摘要」取的 SHA-256（只有输入的摘要是输入的 SHA-256，一边是参照的摘要另拼参照一侧的指纹），
/// 凭命中可跳过、没有拒算那一句：算法拿到的只有场景与输入，存取代码、产品代码与测试服务的代码一个单元都不进，
/// 代码改了而输入没变的那一趟算出的还是这一把。
/// 判别力（`crates/mutations.tsv`）：「产品测试判定库：录入键又拼上 GPU 一侧键表（代码改了键跟着变）」那一行让这一条红。
#[test]
fn the_recording_key_is_the_scenario_and_the_input_digest_with_no_unit_of_code() {
    let composed_key = |inputs_digest: &Sha256Digest| {
        RecordingKey(sha256_of(
            format!(
                "singlefs background gpu pairing recording key\nscenario={SCENARIO_OF_THE_PAIRING}\ninputs={}\n",
                inputs_digest.hexadecimal()
            )
            .as_bytes(),
        ))
    };
    let input_alone = recording_key_of_the_scenario_and_the_inputs(
        SCENARIO_OF_THE_PAIRING,
        &recording_inputs_digest_of_the_input(INPUT_OF_THE_PAIRING),
    );
    assert_eq!(
        (
            input_alone.key,
            input_alone.skips_the_recording_on_a_hit,
            input_alone.refusal
        ),
        (composed_key(&sha256_of(INPUT_OF_THE_PAIRING)), true, None),
        "只有输入的录入键就是场景与输入摘要拼起来的那一把"
    );
    let reference_units = PairingSideUnitsKey {
        fingerprint: Sha256Digest([0x11; DIGEST_BYTES]),
    };
    let with_the_reference = recording_key_of_the_pairing_input_and_the_reference_units(
        SCENARIO_OF_THE_PAIRING,
        INPUT_OF_THE_PAIRING,
        &Ok(reference_units),
    );
    assert_eq!(
        (
            with_the_reference.key,
            with_the_reference.skips_the_recording_on_a_hit,
            with_the_reference.refusal
        ),
        (
            composed_key(
                &recording_inputs_digest_of_the_input_and_the_reference_units(
                    INPUT_OF_THE_PAIRING,
                    &reference_units
                )
            ),
            true,
            None
        ),
        "一边是参照的录入键就是场景与连着参照一侧指纹的输入摘要拼起来的那一把"
    );
}

/// 场景：照录入键与对拍结论的键的算法，逐样换一样再算（不连库）。
/// 预期：同样的几样算两次逐字相同；录入键随场景标识、参照一侧的指纹与输入各自变，不随 GPU 一侧变；对拍结论的键随输入摘要、
/// 参照一侧与 GPU 一侧的指纹、交来的卡的驱动各自变。
/// 判别力（`crates/mutations.tsv`）：「产品测试判定库：结论的键不拼 GPU 一侧的指纹」「产品测试判定库：录入键不拼场景标识」两行各让这一条红。
#[test]
fn the_recording_key_and_the_conclusion_key_change_with_each_of_their_parts_and_only_with_them() {
    let recording_key_of = |scenario: &str, reference_byte: u8, input: &[u8]| {
        recording_key_of_the_pairing_input_and_the_reference_units(
            scenario,
            input,
            &units_key_filled_with(reference_byte),
        )
        .key
    };
    let base = recording_key_of(SCENARIO_OF_THE_PAIRING, 0x11, INPUT_OF_THE_PAIRING);
    assert_eq!(
        base,
        recording_key_of(SCENARIO_OF_THE_PAIRING, 0x11, INPUT_OF_THE_PAIRING),
        "同样的几样同一把录入键"
    );
    assert_ne!(
        base,
        recording_key_of("another_pairing_on_the_cards", 0x11, INPUT_OF_THE_PAIRING),
        "场景变了录入键变"
    );
    assert_ne!(
        base,
        recording_key_of(SCENARIO_OF_THE_PAIRING, 0x33, INPUT_OF_THE_PAIRING),
        "参照一侧变了录入键变"
    );
    assert_ne!(
        base,
        recording_key_of(SCENARIO_OF_THE_PAIRING, 0x11, b"another input"),
        "输入变了录入键变"
    );
    assert_eq!(
        keys_with_the_sides(0x11, 0x22).recording,
        keys_with_the_sides(0x11, 0x44).recording,
        "GPU 一侧不进录入键"
    );
    let side = |byte: u8| units_key_filled_with(byte).expect("造的键都算得出");
    let card = |driver_information: &str| {
        adapters_of_the_handed_over_gpus(&[adapter_with_the_driver_information(driver_information)])
            .expect("驱动名与驱动信息都报了")
    };
    let input_digest = sha256_of(INPUT_OF_THE_PAIRING);
    let base_conclusion = conclusion_key(
        &input_digest,
        &side(0x11),
        &side(0x22),
        &card(DRIVER_INFORMATION_OF_THE_CARD),
    );
    assert_eq!(
        base_conclusion,
        conclusion_key(
            &input_digest,
            &side(0x11),
            &side(0x22),
            &card(DRIVER_INFORMATION_OF_THE_CARD)
        ),
        "同样的几样同一把结论的键"
    );
    for (changed, what) in [
        (
            conclusion_key(
                &sha256_of(b"another input"),
                &side(0x11),
                &side(0x22),
                &card(DRIVER_INFORMATION_OF_THE_CARD),
            ),
            "输入摘要",
        ),
        (
            conclusion_key(
                &input_digest,
                &side(0x33),
                &side(0x22),
                &card(DRIVER_INFORMATION_OF_THE_CARD),
            ),
            "参照一侧",
        ),
        (
            conclusion_key(
                &input_digest,
                &side(0x11),
                &side(0x44),
                &card(DRIVER_INFORMATION_OF_THE_CARD),
            ),
            "GPU 一侧",
        ),
        (
            conclusion_key(&input_digest, &side(0x11), &side(0x22), &card("590.10.01")),
            "交来的卡的驱动",
        ),
    ] {
        assert_ne!(base_conclusion, changed, "{what}变了对拍结论的键变");
    }
}

/// 场景：同一份输入、同两边的键连跑两趟。
/// 预期：头一趟现判、两边一致；第二趟复用库里那一条，持卡判一共只被调一次（复用的那一趟不开卡）。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：复用时仍调持卡判」那一行让这一条红。
#[test]
fn the_second_run_with_the_same_keys_reuses_the_conclusion_without_calling_the_judge_on_the_cards()
{
    let test_name =
        "the_second_run_with_the_same_keys_reuses_the_conclusion_without_calling_the_judge_on_the_cards";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-reused",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let outcomes = [
        one_run_on_the_temporary_library(
            &library,
            test_name,
            &keys,
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        ),
        one_run_on_the_temporary_library(
            &library,
            test_name,
            &keys,
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        ),
    ];
    assert_eq!(
        (outcomes, judgements.get()),
        (
            [
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree,
                PairingOnTheCardsOutcome::ConclusionReusedFromTheLibrary
            ],
            1
        ),
        "头一趟现判、第二趟复用：持卡判只被调一次"
    );
}

/// 场景：同一份输入、同一个参照一侧的键，GPU 一侧的键换一把再跑一趟。
/// 预期：两趟都现判，持卡判被调两次（GPU 一侧走的代码或比法变了，库里旧的结论不复用）。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：GPU 一侧的键不进结论的键」那一行让这一条红。
#[test]
fn a_changed_gpu_side_key_judges_the_same_input_again_on_the_cards() {
    let test_name = "a_changed_gpu_side_key_judges_the_same_input_again_on_the_cards";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-gpu-side-changed",
    );
    let judgements = Cell::new(0_u32);
    let outcomes = [
        one_run_on_the_temporary_library(
            &library,
            test_name,
            &keys_with_the_sides(0x11, 0x22),
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        ),
        one_run_on_the_temporary_library(
            &library,
            test_name,
            &keys_with_the_sides(0x11, 0x33),
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        ),
    ];
    assert_eq!(
        (outcomes, judgements.get()),
        (
            [
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree,
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree
            ],
            2
        ),
        "GPU 一侧的键换了：第二趟照判"
    );
}

/// 场景：持卡判的断言不过（拿读回的输入本身当参照比），连跑两趟。
/// 预期：每一趟都现判、用例 panic，消息带用例名与断言那一句；第二趟不复用记着红的那一条，持卡判被调两次。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：断言不过判成两边一致」那一行让这一条红。
#[test]
fn a_disagreement_is_stored_red_panics_and_is_judged_again_on_the_next_run() {
    let test_name = "a_disagreement_is_stored_red_panics_and_is_judged_again_on_the_next_run";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-disagreement",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let panic_messages: Vec<String> = (0..2)
        .map(|_run| {
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                one_run_on_the_temporary_library(
                    &library,
                    test_name,
                    &keys,
                    |input_read_back: &[u8], reference_read_back: &[u8]| {
                        judgements.set(judgements.get() + 1);
                        judgement_of_the_assertions_on_the_cards(test_name, || {
                            assert_eq!(
                                input_read_back, reference_read_back,
                                "拿读回的输入本身当参照比：两边不一致"
                            );
                            ComparisonOnTheCards::EveryComparisonHeld
                        })
                    },
                )
            }));
            match run {
                Ok(outcome) => format!("这一趟没 panic，交回 {outcome:?}"),
                Err(payload) => payload
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_else(|| "panic 带的不是 String".to_string()),
            }
        })
        .collect();
    assert_eq!(judgements.get(), 2, "记着红的不复用：第二趟照判");
    for panic_message in &panic_messages {
        assert!(
            panic_message.contains(&format!("对拍用例「{test_name}」判出两边不一致"))
                && panic_message.contains("拿读回的输入本身当参照比"),
            "每一趟都判出不一致、panic 带用例名与断言那一句：{panic_messages:?}"
        );
    }
}

/// 场景：持卡判在这一台的卡上没跑成（交回没跑成），连跑两趟。
/// 预期：两趟都交回没跑成、不入库，第二趟照调持卡判（没跑成的那一趟不被当成两边一致复用）。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：没跑成判成两边一致」那一行让这一条红。
#[test]
fn a_run_not_run_on_these_cards_stores_no_conclusion_and_the_next_run_calls_the_judge_again() {
    let test_name =
        "a_run_not_run_on_these_cards_stores_no_conclusion_and_the_next_run_calls_the_judge_again";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-not-run",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let outcomes: Vec<PairingOnTheCardsOutcome> = (0..2)
        .map(|_run| {
            one_run_on_the_temporary_library(
                &library,
                test_name,
                &keys,
                |_input_read_back: &[u8], _reference_read_back: &[u8]| {
                    judgements.set(judgements.get() + 1);
                    judgement_of_the_assertions_on_the_cards(test_name, || {
                        ComparisonOnTheCards::NotRunOnTheseCards
                    })
                },
            )
        })
        .collect();
    assert_eq!(
        (outcomes, judgements.get()),
        (
            vec![
                PairingOnTheCardsOutcome::NotRunOnTheseCards,
                PairingOnTheCardsOutcome::NotRunOnTheseCards
            ],
            2
        ),
        "没跑成的不入库：第二趟照调持卡判"
    );
}

/// 场景：起临时 prune-server 时取这一台的地址交回错误（多机配置认不出这一台）。
/// 预期：交回错误，说明带着取不到的原话与出路，不起服务（不拿缺省地址照起）。
/// 判别力（`crates/mutations.tsv`）：「产品测试临时服务：取不到这一台的地址时拿回环地址照起」那一行让这一条红。
#[test]
fn the_temporary_service_is_not_started_when_the_address_of_this_host_is_not_read() {
    let problem_of_the_address = "多机配置判法认不出这一台（用例造的错）";
    let outcome = PairingLibraryOfTheProductTests::on_a_temporary_service_bound_to_this_host(
        "pairing-without-the-address-of-this-host",
        || Err(problem_of_the_address.to_string()),
    );
    let Err(description) = outcome else {
        panic!("取不到这一台的地址却起了临时 prune-server");
    };
    assert!(
        description.contains(problem_of_the_address) && description.contains("出路："),
        "说明带着取不到的原话与出路：{description}"
    );
}

/// 场景：这一台在多机配置 `HOSTS` 里写的是 IPv6 地址（文档用的地址段 2001:db8::/32）。
/// 预期：临时服务绑的是方括号括起的地址接端口 0。
/// 判别力（`crates/mutations.tsv`）：「产品测试临时服务：IPv6 地址不加方括号」那一行让这一条红。
#[test]
fn the_temporary_service_binds_an_ipv6_address_of_this_host_in_brackets_on_port_zero() {
    assert_eq!(
        network_address_of_a_temporary_service_on("2001:db8::81"),
        "[2001:db8::81]:0",
        "IPv6 地址加方括号再接端口 0"
    );
}

/// 一组在编进来的 GPU 一侧键表里的那几行：组那一行的结论（`keyed`、`refused`、`not-compiled`）与键（算出了的才有），组成清单里 WGSL 计算入口所在的文件。
struct GpuSideGroupInTheCompiledTable {
    verdict: String,
    key_hexadecimal: Option<String>,
    files_of_the_wgsl_compute_entries: Vec<String>,
}

/// 编进来的 GPU 一侧键表里组 `group_name` 那几行（表的写法在 `singlefs-unit-graph` 的 `entry_groups` 文件头）。
fn gpu_side_group_in_the_compiled_table(group_name: &str) -> GpuSideGroupInTheCompiledTable {
    let mut verdict = String::new();
    let mut key_hexadecimal = None;
    let mut files_of_the_wgsl_compute_entries = Vec::new();
    for line in GPU_SIDE_UNITS_KEYS_TEXT.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            ["group", name, "keyed", key, ..] if *name == group_name => {
                verdict = "keyed".to_string();
                key_hexadecimal = Some((*key).to_string());
            }
            ["group", name, group_verdict, ..] if *name == group_name => {
                verdict = (*group_verdict).to_string();
            }
            ["unit", name, "wgsl_compute", _qualified_name, file_and_lines, ..]
                if *name == group_name =>
            {
                let (file, _lines) = file_and_lines
                    .rsplit_once(':')
                    .expect("组成清单的位置写成 <文件>:<起>-<止>");
                files_of_the_wgsl_compute_entries.push(file.to_string());
            }
            _ => {}
        }
    }
    GpuSideGroupInTheCompiledTable {
        verdict,
        key_hexadecimal,
        files_of_the_wgsl_compute_entries,
    }
}

/// 一个真卡对拍测试目标的持卡判至少要走到的 WGSL 文件（产品 GPU 包里它对拍的那两份核）；端到端那一个目标的几条走的核各不相同，只要求至少走到一份。
fn kernel_files_every_gpu_side_group_of_the_target_reaches(
    target: &str,
) -> &'static [&'static str] {
    match target {
        "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference"
        | "test:checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference"
        | "test:checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference" => &[
            "crates/singlefs-background-gpu/src/checksum_kernel_table_driven.wgsl",
            "crates/singlefs-background-gpu/src/checksum_kernel_bitwise.wgsl",
        ],
        "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference" => &[
            "crates/singlefs-background-gpu/src/live_slot_kernel_walking_every_record.wgsl",
            "crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl",
        ],
        "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference" => &[
            "crates/singlefs-background-gpu/src/region_record_kernel_walking_the_mapping.wgsl",
            "crates/singlefs-background-gpu/src/region_record_kernel_searching_sorted_indexes.wgsl",
        ],
        _ => &[],
    }
}

/// 场景：读测试支撑包的构建脚本编进来的 GPU 一侧键表，逐个对登记表（`tests/pairing_units_entries.tsv` 的 GPU 一侧那几行）里的组。
/// 预期：每条登记的用例一组、组名各不相同、都算出了键、键两两不同（两条用例不落到同一把 GPU 一侧的键上）；每组的闭包里至少有一个
/// 产品 GPU 包的 WGSL 计算入口，校验和、活槽、区域记录三类目标的每一组都走到它那两份核。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：GPU 一侧的组入口换成参照那一侧」「真卡对拍结论：两条用例登记同一个持卡判入口」两行各让这一条红。
#[test]
fn every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate() {
    let gpu_side_groups = registered_groups_of_the_side(PairingSideOfTheEntries::GpuSide);
    let mut group_names: Vec<&str> = gpu_side_groups
        .iter()
        .map(|group| group.group_name)
        .collect();
    group_names.sort_unstable();
    group_names.dedup();
    assert_eq!(
        group_names.len(),
        gpu_side_groups.len(),
        "一条用例一组：组名各不相同"
    );
    let groups_falling_short: Vec<String> = gpu_side_groups
        .iter()
        .filter_map(|group| {
            let compiled = gpu_side_group_in_the_compiled_table(group.group_name);
            let product_kernels_reached: Vec<&String> = compiled
                .files_of_the_wgsl_compute_entries
                .iter()
                .filter(|file| file.starts_with("crates/singlefs-background-gpu/src/"))
                .collect();
            let kernel_files_missed: Vec<&str> =
                kernel_files_every_gpu_side_group_of_the_target_reaches(group.target)
                    .iter()
                    .copied()
                    .filter(|kernel_file| {
                        !compiled
                            .files_of_the_wgsl_compute_entries
                            .iter()
                            .any(|file| file == kernel_file)
                    })
                    .collect();
            (compiled.verdict != "keyed"
                || product_kernels_reached.is_empty()
                || !kernel_files_missed.is_empty())
            .then(|| {
                format!(
                    "{}：结论 {:?}，走到的产品核 {product_kernels_reached:?}，没走到 {kernel_files_missed:?}",
                    group.group_name, compiled.verdict
                )
            })
        })
        .collect();
    assert!(
        groups_falling_short.is_empty(),
        "每组都算出了键、闭包收到产品 GPU 包里它走到的核：{groups_falling_short:#?}"
    );
    let distinct_keys: BTreeSet<String> = gpu_side_groups
        .iter()
        .filter_map(|group| gpu_side_group_in_the_compiled_table(group.group_name).key_hexadecimal)
        .collect();
    assert_eq!(
        distinct_keys.len(),
        gpu_side_groups.len(),
        "每组一把键、两两不同：两条用例不落到同一把 GPU 一侧的键上"
    );
}

/// 场景：同一份输入、同两边的键，交来的卡换一版驱动（驱动信息不同）再跑一趟。
/// 预期：两趟都现判，持卡判被调两次（判法单元的键只管代码，换一版驱动 GPU 一侧走的是另一份机器码，库里旧的结论不复用）。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：交来的卡的适配器不进结论的键」那一行让这一条红。
#[test]
fn a_card_on_another_driver_judges_the_same_input_again_on_the_cards() {
    let test_name = "a_card_on_another_driver_judges_the_same_input_again_on_the_cards";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-driver-changed",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let outcomes = [DRIVER_INFORMATION_OF_THE_CARD, "590.10.01"].map(|driver_information| {
        one_run_on_the_temporary_library_with_the_adapters(
            &library,
            test_name,
            &keys,
            adapters_seen_with_the_driver_information(driver_information),
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        )
    });
    assert_eq!(
        (outcomes, judgements.get()),
        (
            [
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree,
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree
            ],
            2
        ),
        "交来的卡换了一版驱动：第二趟照判"
    );
}

/// 场景：交来的卡的驱动没报驱动信息（空串），同一份输入、同两边的键连跑两趟。
/// 预期：交回的适配器描述是拒算那一句；两趟都现判、不入库，持卡判被调两次。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：驱动没报驱动信息也算出适配器」那一行让这一条红。
#[test]
fn a_card_whose_driver_reports_no_driver_information_is_judged_without_storing_the_conclusion() {
    let test_name =
        "a_card_whose_driver_reports_no_driver_information_is_judged_without_storing_the_conclusion";
    assert!(
        adapters_of_the_handed_over_gpus(&[adapter_with_the_driver_information("")]).is_err(),
        "驱动信息是空的：读不出是哪一版驱动"
    );
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-driver-not-reported",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let outcomes = [(), ()].map(|()| {
        one_run_on_the_temporary_library_with_the_adapters(
            &library,
            test_name,
            &keys,
            adapters_seen_with_the_driver_information(""),
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        )
    });
    assert_eq!(
        (outcomes, judgements.get()),
        (
            [
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree,
                PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree
            ],
            2
        ),
        "适配器读不全：照判、不入库，第二趟照判"
    );
}

/// 场景：判段问库之前这一台没交到卡（看到的是没交到卡），连跑两趟；头一趟之前库里已有交到卡时判过、两边一致的那一条。
/// 预期：两趟都交回没跑成，持卡判一次都不调（键算不出：既不复用，也不判）。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：没交到卡也拿空的适配器问库」那一行让这一条红。
#[test]
fn a_host_with_no_gpu_handed_over_neither_reuses_nor_judges_the_conclusion() {
    let test_name = "a_host_with_no_gpu_handed_over_neither_reuses_nor_judges_the_conclusion";
    let library = PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process(
        "pairing-conclusion-no-gpu-handed-over",
    );
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let judged_with_a_card = one_run_on_the_temporary_library(
        &library,
        test_name,
        &keys,
        judge_counted_that_compares_the_reversed_input(test_name, &judgements),
    );
    let outcomes_without_a_card = [(), ()].map(|()| {
        one_run_on_the_temporary_library_with_the_adapters(
            &library,
            test_name,
            &keys,
            AdaptersSeenBeforeTheConclusion::NoGpuHandedOver,
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        )
    });
    assert_eq!(
        (
            judged_with_a_card,
            outcomes_without_a_card,
            judgements.get()
        ),
        (
            PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree,
            [
                PairingOnTheCardsOutcome::NotRunOnTheseCards,
                PairingOnTheCardsOutcome::NotRunOnTheseCards
            ],
            1
        ),
        "没交到卡：既不复用也不判"
    );
}
