//! MODE=enum：瞬时隔离（与不注入的对照、写者漏释放）那一段历史里事件那次覆盖写的写流，照崩溃点重放的枚举域枚举
//! （`enumerate_crash_states_versions`：每一段都展开，原地覆写三态），每个状态恢复之后由枚举器跑池级 checker；
//! 先按 `crash_state_count_with_torn_in_place_overwrites` 算状态数，超过 10⁶ 不跑（主 agent 定了要跑的，设 OPUS_STATES_CAP 放宽）。
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use singlefs_checker_tier::crash::{crash_state_count_with_torn_in_place_overwrites, CrashPointReplaySegmentExpansion};
use singlefs_checker_tier::crash_state_enumeration::{enumerate_crash_states_versions, CrashPointReplayItem};
use singlefs_core::address::DeviceIdentity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, PublishedVersion};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

use crate::flaky::{start_pool, Faults};
use crate::grid::{env_list, env_u64};
use crate::pool::{fixed_geometry_of_the_384_slot_small_devices, OVERWRITE_BYTES};

/// 设点文件只用来取结论版本与节点身份：库只认 checker 档包 tests/ 下的候选文件，这里借一份现成文件的名字（不读它的代码、不拷）；判定库在草稿目录里、每趟清空。
const SETTING_FILE: &str = "crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs";

pub fn main_enum() {
    let pre = env_u64("PRE", 2);
    for shape in env_list("SHAPES", "control,transient,leak") {
        let stream = SharedStream::retaining_contents();
        let faults = Rc::new(RefCell::new(Faults::default()));
        let mut pool = start_pool(&stream, &faults);
        for _ in 0..pre {
            pool.overwrite(OVERWRITE_BYTES).expect("事件之前的覆盖写");
        }
        let current = pool.current_file_version().clone();
        let slot1 = current.data_pointers[0].locations.iter().find(|l| l.device == DeviceIdentity(1)).expect("盘 1").slot.0;
        match shape.as_str() {
            "transient" => faults.borrow_mut().read_fail.push((DeviceIdentity(1), slot1 * SLOT_BYTES, 2)),
            "leak" => std::env::set_var("OPUS_C794_LEAK_SLOT", slot1.to_string()),
            "control" => {}
            other => panic!("形 {other} 认不得"),
        }
        let before = stream.retained_operations().len();
        pool.overwrite(OVERWRITE_BYTES).expect("事件那次覆盖写");
        std::env::remove_var("OPUS_C794_LEAK_SLOT");
        let quarantined = pool.current_file_version().quarantined_after_release_checksum_mismatch.len();
        let ops = stream.retained_operations();
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], pool.device_bytes);
        base.apply(&ops[..before]);
        let (writes, segments) = writes_and_segments(&ops[before..], &fixed_geometry_of_the_384_slot_small_devices());
        let expand = |_i: usize, _s: &[usize]| CrashPointReplaySegmentExpansion::EveryProperSubset;
        let states = crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &expand);
        let closed = closed_form_state_count(&segments);
        if states > env_u64("OPUS_STATES_CAP", 1_000_000) {
            println!("OPUS-ENUM-SKIP shape={shape} states={states} closed_form={closed} reason=over_1e6");
            continue;
        }
        let root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("流里有根槽写");
        let versions: Vec<PublishedVersion> = pool
            .content_of_each_root
            .iter()
            .map(|((instance, txg), content)| PublishedVersion { instance: *instance, checkpoint_txg: *txg, content: content.clone() })
            .collect();
        let started = Instant::now();
        let name = format!("opus-c794-{shape}");
        let tally = enumerate_crash_states_versions(
            CrashPointReplayItem { name: &name, setting_file: SETTING_FILE },
            &base,
            &writes,
            &segments,
            root_index,
            &versions,
        )
        .judged()
        .expect("单机：整份判完");
        println!(
            "OPUS-ENUM shape={shape} quarantined_copies={quarantined} writes={} segments={} states_counted={states} closed_form={closed} enumerated={} root_persisted={} oracle_violations={} checker_violated={:?} not_applicable_i3_11={:?} secs={:.1}",
            writes.len(),
            segments.len(),
            tally.states,
            tally.root_persisted_states,
            tally.violations,
            tally.checker_violated_states,
            tally.checker_not_applicable_states.get("I-3.11"),
            started.elapsed().as_secs_f64()
        );
    }
}
