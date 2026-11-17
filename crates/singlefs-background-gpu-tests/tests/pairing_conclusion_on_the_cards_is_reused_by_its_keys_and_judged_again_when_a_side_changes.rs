//! 真卡对拍的结论入库（`common_pairing_input_through_the_library` 的 `pairing_concluded_on_the_cards_in_the_library`）不上卡就核得了的那几件：
//! 持卡判换成一个只比字节的假判法，在一次性的库上（验证包 `crash_library::DisposableLibrary`，它自己起的临时 prune-server，不连真服务）跑——
//! 同一份输入、同两边的键第二趟复用、持卡判不被调；GPU 一侧的键变了照判；判出不一致的记红、用例 panic、下一趟照判；持卡判没跑成的不入库、下一趟照判。
//! 另核构建脚本编进来的 GPU 一侧键表：每条登记的真卡对拍用例一组、都算出了键、闭包收到产品 GPU 包里它走到的核。
//! 不要卡、不挂特性，`cargo test -p singlefs-background-gpu-tests --test <本目标>` 就能跑。
mod common_discrete_gpus_of_this_host;
mod common_pairing_input_through_the_library;
#[path = "../src/gpu_side_units_entries.rs"]
mod gpu_side_units_entries;
#[allow(
    dead_code,
    reason = "这里只读 GPU 一侧那张登记表，参照一侧那张与特性名不用"
)]
#[path = "../src/reference_units_entries.rs"]
mod reference_units_entries;

use std::cell::Cell;
use std::collections::BTreeSet;

use common_pairing_input_through_the_library::{
    judgement_of_the_assertions_on_the_cards, pairing_concluded_on_the_cards_in_the_library,
    ComparisonOnTheCards, KeysOfAPairingOnTheCards, PairingOnTheCardsOutcome,
    GPU_SIDE_UNITS_KEYS_TEXT,
};
use singlefs_verification::crash_identity::JudgingUnitsKey;
use singlefs_verification::crash_library::{
    recording_key_of_the_pairing_input_and_the_reference_units, CrashLibrary, CrashLibraryClass,
    DisposableLibrary, PairingJudged, RunStage,
};
use singlefs_verification::sha256::DIGEST_BYTES;

/// 假的对拍输入：持卡判与参照只拿它的字节比。
const INPUT_OF_THE_PAIRING: &[u8] = b"input of a pairing on the cards";

/// 造一把判法单元键：指纹是 `fingerprint_byte` 铺满，组成清单空着（只当键用）。
fn judging_units_key_filled_with(fingerprint_byte: u8) -> Result<JudgingUnitsKey, String> {
    Ok(JudgingUnitsKey::of_the_parts(
        [fingerprint_byte; DIGEST_BYTES],
        Vec::new(),
    ))
}

/// 参照一侧键 `reference_byte`、GPU 一侧键 `gpu_side_byte` 的三把键（录入函数单元那一把固定）。
fn keys_with_the_sides(reference_byte: u8, gpu_side_byte: u8) -> KeysOfAPairingOnTheCards {
    let reference_units = judging_units_key_filled_with(reference_byte);
    KeysOfAPairingOnTheCards {
        recording: recording_key_of_the_pairing_input_and_the_reference_units(
            &judging_units_key_filled_with(0x5A),
            INPUT_OF_THE_PAIRING,
            &reference_units,
        ),
        reference_units,
        gpu_side_units: judging_units_key_filled_with(gpu_side_byte),
    }
}

/// 假的 CPU 参照：输入倒过来。
fn reversed_input(input_bytes: &[u8]) -> Vec<u8> {
    input_bytes.iter().rev().copied().collect()
}

/// 在一次性的库上照先录后判跑一趟：参照是倒过来的输入，持卡判是 `judge_on_the_cards`。
fn one_run_on_the_disposable_library(
    library: &CrashLibrary,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    judge_on_the_cards: impl FnOnce(&[u8], &[u8]) -> PairingJudged,
) -> PairingOnTheCardsOutcome {
    pairing_concluded_on_the_cards_in_the_library(
        library,
        RunStage::RecordThenJudge,
        test_name,
        keys,
        INPUT_OF_THE_PAIRING,
        reversed_input,
        judge_on_the_cards,
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

/// 场景：同一份输入、同两边的键连跑两趟。
/// 预期：头一趟现判、两边一致；第二趟复用库里那一条，持卡判一共只被调一次（复用的那一趟不领卡、不开卡）。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：复用时仍调持卡判」那一行让这一条红。
#[test]
fn the_second_run_with_the_same_keys_reuses_the_conclusion_without_calling_the_judge_on_the_cards()
{
    let test_name =
        "the_second_run_with_the_same_keys_reuses_the_conclusion_without_calling_the_judge_on_the_cards";
    // 一次性库留着：要从空库数持卡判被调了几次
    let disposable = DisposableLibrary::new("gpu-pairing-conclusion-reused")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了一次性库");
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let outcomes = [
        one_run_on_the_disposable_library(
            &library,
            test_name,
            &keys,
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        ),
        one_run_on_the_disposable_library(
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
    // 一次性库留着：要从空库数持卡判被调了几次
    let disposable = DisposableLibrary::new("gpu-pairing-conclusion-gpu-side-changed")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了一次性库");
    let judgements = Cell::new(0_u32);
    let outcomes = [
        one_run_on_the_disposable_library(
            &library,
            test_name,
            &keys_with_the_sides(0x11, 0x22),
            judge_counted_that_compares_the_reversed_input(test_name, &judgements),
        ),
        one_run_on_the_disposable_library(
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
    // 一次性库留着：要从空库数持卡判被调了几次
    let disposable = DisposableLibrary::new("gpu-pairing-conclusion-disagreement")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了一次性库");
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let panic_messages: Vec<String> = (0..2)
        .map(|_run| {
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                one_run_on_the_disposable_library(
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
    // 一次性库留着：要从空库数持卡判被调了几次
    let disposable = DisposableLibrary::new("gpu-pairing-conclusion-not-run")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了一次性库");
    let keys = keys_with_the_sides(0x11, 0x22);
    let judgements = Cell::new(0_u32);
    let outcomes: Vec<PairingOnTheCardsOutcome> = (0..2)
        .map(|_run| {
            one_run_on_the_disposable_library(
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

/// 场景：读构建脚本编进来的 GPU 一侧键表，逐个对登记表（`src/gpu_side_units_entries.rs`）里的组。
/// 预期：每条登记的用例一组、组名各不相同、都算出了键、键两两不同（两条用例不落到同一把 GPU 一侧的键上）；每组的闭包里至少有一个
/// 产品 GPU 包的 WGSL 计算入口，校验和、活槽、区域记录三类目标的每一组都走到它那两份核。
/// 判别力（`crates/mutations.tsv`）：「真卡对拍结论：GPU 一侧的组入口换成参照那一侧」「真卡对拍结论：两条用例登记同一个持卡判入口」两行各让这一条红。
#[test]
fn every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate() {
    let mut group_names: Vec<&str> = gpu_side_units_entries::GPU_SIDE_UNITS_ENTRY_GROUPS
        .iter()
        .map(|group| group.group_name)
        .collect();
    group_names.sort_unstable();
    group_names.dedup();
    assert_eq!(
        group_names.len(),
        gpu_side_units_entries::GPU_SIDE_UNITS_ENTRY_GROUPS.len(),
        "一条用例一组：组名各不相同"
    );
    let groups_falling_short: Vec<String> = gpu_side_units_entries::GPU_SIDE_UNITS_ENTRY_GROUPS
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
    let distinct_keys: BTreeSet<String> = gpu_side_units_entries::GPU_SIDE_UNITS_ENTRY_GROUPS
        .iter()
        .filter_map(|group| gpu_side_group_in_the_compiled_table(group.group_name).key_hexadecimal)
        .collect();
    assert_eq!(
        distinct_keys.len(),
        gpu_side_units_entries::GPU_SIDE_UNITS_ENTRY_GROUPS.len(),
        "每组一把键、两两不同：两条用例不落到同一把 GPU 一侧的键上"
    );
}
