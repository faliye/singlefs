//! GPU 判器一次派活多宽只是切法、不进判定结论：同一条流按几种宽度判出的结论字逐字相同，且与 CPU 参照判器逐状态相同。

use super::*;

/// 派活宽度那一档的判器一次派活最多带几个状态：比 1 宽、比一个就判完整条流窄，且不整除这条流的状态数（末一次派活不满）。
#[cfg(feature = "gpu")]
const DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH: usize = 7;

/// 派活宽度那一条流的状态数上限：几百个以内的小流，单跑几秒。
#[cfg(feature = "gpu")]
const MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW: u64 = 1000;

/// 派活宽度那一条流里段长不超过这个数的段逐个真子集展开，更长的段只以整段持久进入后面的状态（状态数压在几百个以内）。
#[cfg(feature = "gpu")]
const EXPAND_UP_TO_IN_THE_DISPATCH_WIDTH_FLOW: usize = 3;

/// 一次派活多宽由调用方定（`gpu_memory_ledger::states_in_one_dispatch_capped_by_the_state_count`、
/// `crate::crash_judge_dispatch::states_in_flight_within` 这几个）：「一次派活带几个状态只是切法，
/// 不进判定结论」（C1033（两个显存记账单元没有会红的检查））的蜕变断言：新池新建文件那条流（两块稀疏内存盘上 mkfs、取号、暖机、
/// 新建文件，录 mkfs 之后那一段）在同一张借来的卡上，按一次派活最多 1 个、
/// [`DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH`] 个与整条流的状态数各建一个判器——前两个照判器的派活宽度调度一次次派、
/// 最后一个一次派活判完整条流——逐状态的结论字逐字相同，且与 CPU 参照判器逐状态对拍相同。
/// 先核三个判器开出来的派活上限真是 1、7 与状态数（封顶那一处真起了作用），整条流的结论里至少两种不同的字（派活错位要判得出来）。
/// 三个判器在同一张卡上依次建、依次判（比的就是同一批状态换一种切法），不并行；借的是三个判器各自要的显存之和。
/// 判表过库与 CPU 参照取样在借卡之前、都开这条用例自己的一次性库，过库之后核判表写在这一份里：单跑与同一个进程里别的用例设着一次性库时开的是同一份。
/// 判别力（`crates/mutations.tsv`）：不把一次性库设成这个进程开的那一份，判表写到别处，核判表那一句红。
#[cfg(feature = "gpu")]
#[test]
fn verdict_words_of_every_state_are_the_same_at_a_dispatch_width_of_one_of_seven_and_of_every_state_and_agree_with_the_cpu_judge(
) {
    use crate::crash::{crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion};
    use crate::crash_judge_conclusions::{
        cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
        CrossCheckWords,
    };
    use crate::crash_judge_dispatch::{mebibytes_a_judge_needs, GpuCrashJudge, PackedJudgeTables};
    use crate::crash_judge_gpu::{cross_check_words_of_gpu_verdict, VERDICT_WORDS};
    use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
    use crate::crash_library::{
        judge_tables_through_the_pairing_library, DisposableLibrary, StageOutcome,
    };
    use crate::crash_state_enumeration::{sample_crash_states_on_the_cpu, CrashPointReplayItem};
    use crate::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    use crate::memory_pool::{
        writes_and_segments_with_stream_indexes, PublishedVersion, SparseBlockDevice,
    };
    use crate::segments::FixedGeometry;
    use crate::verdict_store::PruneServerFailureHandling;
    use crate::{RecordingBlockDevice, SharedStream};
    use singlefs_core::address::DeviceIdentity;
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;

    const TEST_NAME: &str = "verdict_words_of_every_state_are_the_same_at_a_dispatch_width_of_one_of_seven_and_of_every_state_and_agree_with_the_cpu_judge";
    const DEVICE_BYTES: u64 = 4 << 30;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let stream = SharedStream::retaining_contents();
    let mut devices = [0, 1].map(|device| {
        (
            DeviceIdentity(device),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device),
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                stream.clone(),
            ),
        )
    });
    let parameters = crate::known_bad_images::parameters_of_the_common_test_pool();
    let run =
        crate::scenario::run_new_pool_file_creation(&parameters, &mut devices, &stream, |_, _| {})
            .expect("新池新建文件那条路做得成");
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
    base.apply(&operations[..run.mkfs_operation_count]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[run.mkfs_operation_count..],
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == crate::segments::StepKind::RootRecordFua)
        .expect("新建文件那次发布有一条根槽写");
    let versions = vec![PublishedVersion {
        instance: run.output.root.instance,
        checkpoint_txg: run.output.root.checkpoint_txg,
        content: crate::scenario::first_file_content(),
    }];
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO_IN_THE_DISPATCH_WIDTH_FLOW {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &versions,
        expansion: &expansion,
    })
    .expect("新池新建文件那条流建得出判表");
    let state_count = tables.state_count;
    assert_eq!(
        state_count,
        crash_point_replay_plan_state_count(&base, &writes, &segments, &expansion),
        "判表的状态数与崩溃点重放的计划相同"
    );
    assert!(
            state_count
                > 2 * u64::try_from(DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH)
                    .expect("装得进 u64")
                && state_count <= MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW,
            "这条流 {state_count} 个状态：要跨过两次 {DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH} 个的派活，又在 {MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW} 个以内"
        );
    let every_state = usize::try_from(state_count).expect("装得进 usize");
    // 经产品入口开库的两处（判表过库、CPU 参照取样）开的都是这条用例自己的一次性库：没设就开，开到的是同一个进程里
    // 别的用例此刻设着的那一份（那条用例收尾时停掉它的临时服务，这一边接着读写就连不上），没人设着时开到全仓那一份。
    // 设着的那一段只罩这两处、不罩借卡：借卡等别的线程还卡，持着它等，设一次性库的别的用例都跟着等。
    let disposable_library = DisposableLibrary::new("dispatch-width-pairing")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let packed_tables = PackedJudgeTables::of(&tables);
    let (packed, cpu) = {
        let _the_library_of_this_process = disposable_library.make_it_the_library_of_this_process();
        // 判表先入库、再读库，GPU 只拿读回的那一份算
        let packed = match judge_tables_through_the_pairing_library(TEST_NAME, &packed_tables) {
            StageOutcome::Judged(packed) => packed,
            StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
                println!("JUDGE_TABLES_RECORDED_ONLY test={TEST_NAME}");
                return;
            }
        };
        let mut cpu: Vec<CrossCheckWords> = Vec::new();
        let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(
            CrashPointReplayItem {
                name: TEST_NAME,
                setting_file: "crates/singlefs-verification/src/crash_identity.rs",
            },
            &base,
            &writes,
            &segments,
            judged_root_index,
            &versions,
            &expansion,
            0..state_count,
            &mut |ordinal, _image, judgement| {
                assert_eq!(
                    ordinal,
                    u64::try_from(cpu.len()).expect("装得进 u64"),
                    "CPU 参照按序号次序交回"
                );
                cpu.push(cross_check_words_of_cpu_judgement(judgement));
            },
        )
        .judged() else {
            return;
        };
        assert_eq!(judged_on_the_cpu, state_count, "CPU 参照判了整条流");
        (packed, cpu)
    };
    assert!(
        disposable_library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .expect("开得了这条用例自己的一次性库")
            .read_packed_judge_tables(&sha256_digest(&packed_tables.to_bytes()))
            .expect("读得了这条用例自己的一次性库")
            .is_some(),
        "判表过库写进的是这条用例自己的一次性库（不是别的用例设着的那一份、也不是全仓那一份）"
    );
    let widths = [
        1,
        DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH,
        every_state,
    ];
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        widths
            .iter()
            .map(|width| mebibytes_a_judge_needs(&packed, *width))
            .sum::<u64>(),
    ) else {
        return;
    };
    let verdicts_at_each_width: Vec<Vec<[u32; VERDICT_WORDS]>> = widths
        .iter()
        .map(|width| {
            let judge = GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, *width);
            assert_eq!(
                judge.states_per_dispatch(),
                *width,
                "要一次派活至多 {width} 个状态的判器开出来的派活上限"
            );
            if *width == every_state {
                judge.judge_states_in_one_dispatch(0..state_count)
            } else {
                judge.judge_states(0..state_count)
            }
        })
        .collect();
    drop(lease);
    let in_one_dispatch = &verdicts_at_each_width[widths.len() - 1];
    assert_eq!(
        in_one_dispatch.len(),
        every_state,
        "一次派活判完的结论一个状态一份"
    );
    let distinct_verdicts: std::collections::BTreeSet<&[u32; VERDICT_WORDS]> =
        in_one_dispatch.iter().collect();
    assert!(
        distinct_verdicts.len() >= 2,
        "整条流的结论只有一种字：派活错了位也比不出来"
    );
    for (width, verdicts) in widths.iter().zip(&verdicts_at_each_width) {
        assert_eq!(
            verdicts.len(),
            every_state,
            "一次派活至多 {width} 个时的结论个数"
        );
        let first_difference = verdicts
            .iter()
            .zip(in_one_dispatch)
            .position(|(at_this_width, at_once)| at_this_width != at_once);
        assert_eq!(
                first_difference, None,
                "一次派活至多 {width} 个状态判出的结论字与一次派活判完整条流的不同，第一个不同的状态序号在这里"
            );
    }
    let disagreements: Vec<String> = cpu
        .iter()
        .zip(in_one_dispatch)
        .enumerate()
        .filter_map(|(ordinal, (cpu_words, gpu_verdict))| {
            let gpu_words = cross_check_words_of_gpu_verdict(gpu_verdict);
            (!cross_check_words_agree_in_the_cross_check(&gpu_words, cpu_words))
                .then(|| format!("状态 {ordinal}：CPU {cpu_words:?}；GPU {gpu_words:?}"))
        })
        .collect();
    assert!(
        disagreements.is_empty(),
        "{} 个状态 GPU 与 CPU 对拍的字不同（前几个）：\n{}",
        disagreements.len(),
        disagreements
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!(
            "CRASH_JUDGE_DISPATCH_WIDTHS test={TEST_NAME} states={state_count} widths={widths:?} distinct_verdicts={}",
            distinct_verdicts.len()
        );
}
