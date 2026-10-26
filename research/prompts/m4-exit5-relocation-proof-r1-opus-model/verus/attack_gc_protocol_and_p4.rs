// 攻方腿 m4-exit5-relocation-proof-r1 的 Verus 反例（副本上直接起 verus 验，不入库、不进门禁）。
// 同目录的 gc_crash_proofs.rs 与 compaction_plan_proofs.rs 由 tools/rerun.sh 从冻结副本原样拷来，一字不改；这里只引用它们的 spec，
// 用构造出的具体状态证三件事（每件都是「协议或 P4 的前提成立、定理照样成立，而盘上会让某个读者读坏」）：
//   一、协议允许：搬迁、checkpoint 之后没有读者时立刻回收并覆写源槽；core 的管理员回退恰要复活这个源槽（mount.rs 的
//       roll_back_by_a_forward_publish 文档「整理在两版之间搬过的单元……回退释放的正是目的落点、复活的是源落点」），协议里没有回退目标这一类视图。
//   二、把「回退目标」当读者塞进协议：一旦它是另一个逻辑版本（用户在它之后覆盖写过，key 集合不同），gc_all_views_match 对任何 logical 都假，
//       定理的前提在这类真实盘面上空真。
//   三、P4 的目的地前提只比起点槽号：两槽的目的地罩住一个一槽单元的槽，destinations_are_distinct_and_free 成立，按槽内字节读的那个单元变了。
use vstd::prelude::*;
mod gc_crash_proofs;
mod compaction_plan_proofs;
use gc_crash_proofs::*;
use compaction_plan_proofs::*;

verus! {

// 一：七步里合法的五步，之后搬迁之前那一版的映射（回退目标）读到的不是 logical。
proof fn reclaiming_the_source_after_a_checkpoint_is_enabled_and_breaks_the_pre_compaction_root() {
    let bytes = seq![42u8];
    let garbage = seq![0u8];
    let logical = Map::<u64, Seq<u8>>::empty().insert(7u64, bytes);
    let before = Map::<u64, u64>::empty().insert(7u64, 10u64);
    let s0 = GcCrashState { disk: Map::empty().insert(10u64, bytes), current: before, durable: before, readers: Seq::empty() };
    assert(gc_all_views_match(s0, logical));
    let w = GcCrashStep::Write { slot: 20, bytes };
    assert(gc_step_enabled(s0, w));
    let s1 = gc_next(s0, w);
    lemma_gc_step_preserves_current_durable_and_old_readers(s0, w, logical);
    let r = GcCrashStep::Relocate { key: 7, destination: 20 };
    assert(s1.disk[s1.current[7]] == s1.disk[20u64]);
    assert(gc_step_enabled(s1, r));
    let s2 = gc_next(s1, r);
    lemma_gc_step_preserves_current_durable_and_old_readers(s1, r, logical);
    let c = GcCrashStep::Checkpoint;
    let s3 = gc_next(s2, c);
    lemma_gc_step_preserves_current_durable_and_old_readers(s2, c, logical);
    assert(s3.current[7] == 20 && s3.durable[7] == 20);
    assert(gc_map_avoids(s3.current, 10));
    assert(gc_map_avoids(s3.durable, 10));
    assert(gc_slot_is_unreferenced(s3, 10));
    let reclaim = GcCrashStep::Reclaim { slot: 10 };
    assert(gc_step_enabled(s3, reclaim));
    let s4 = gc_next(s3, reclaim);
    lemma_gc_step_preserves_current_durable_and_old_readers(s3, reclaim, logical);
    assert(gc_slot_is_unreferenced(s4, 10));
    let overwrite = GcCrashStep::Write { slot: 10, bytes: garbage };
    assert(gc_step_enabled(s4, overwrite));
    let s5 = gc_next(s4, overwrite);
    lemma_gc_step_preserves_current_durable_and_old_readers(s4, overwrite, logical);
    // 定理说的三类视图都还读得对：
    assert(gc_all_views_match(s5, logical));
    // 而搬迁之前那一版的映射（core 回退复活的就是它）读坏了：
    assert(gc_view_matches(before, s0.disk, logical));
    assert(s5.disk[10u64] == garbage);
    assert(garbage != bytes) by { assert(garbage[0] != bytes[0]); }
    assert(!gc_view_matches(before, s5.disk, logical));
}

// 一之二：中间插一次 Crash 也一样（Crash 清空读者；回退目标在盘上的根环里，崩溃之后还在）。
proof fn a_crash_drops_every_reader_while_the_rollback_target_survives_on_disk() {
    let bytes = seq![42u8];
    let logical = Map::<u64, Seq<u8>>::empty().insert(7u64, bytes);
    let before = Map::<u64, u64>::empty().insert(7u64, 10u64);
    let after = Map::<u64, u64>::empty().insert(7u64, 20u64);
    let disk = Map::<u64, Seq<u8>>::empty().insert(10u64, bytes).insert(20u64, bytes);
    let s = GcCrashState { disk, current: after, durable: after, readers: seq![before] };
    assert(gc_all_views_match(s, logical));
    assert(!gc_slot_is_unreferenced(s, 10)) by { assert(!gc_map_avoids(s.readers[0], 10)) by { assert(s.readers[0][7] == 10); } }
    let crashed = gc_next(s, GcCrashStep::Crash);
    lemma_gc_step_preserves_current_durable_and_old_readers(s, GcCrashStep::Crash, logical);
    assert(crashed.readers.len() == 0);
    assert(gc_map_avoids(crashed.current, 10));
    assert(gc_slot_is_unreferenced(crashed, 10));
    assert(gc_step_enabled(crashed, GcCrashStep::Reclaim { slot: 10 }));
}

// 二：读者里有一个 key 集合不同的映射（另一个逻辑版本），前提对任何 logical 都不成立。
proof fn a_reader_of_another_logical_version_makes_the_premise_false(s: GcCrashState, logical: Map<u64, Seq<u8>>, index: int)
    requires
        0 <= index < s.readers.len(),
        s.readers[index].dom() != s.current.dom(),
    ensures
        !gc_all_views_match(s, logical),
{
    if gc_all_views_match(s, logical) {
        assert(gc_view_matches(s.readers[index], s.disk, logical));
        assert(s.readers[index].dom() == logical.dom());
        assert(s.current.dom() == logical.dom());
    }
}

// 三：P4 的目的地前提成立，而两槽目的地（起点 2，罩 2、3 两槽）盖住一槽单元（key 5 在槽 3）。
// 按槽内容的具体盘：每个槽一段字节；一个跨度 span 的单元读 [起点, 起点 + span) 那几槽。P4 自己的 storage 只按起点槽号取一段，看不见重叠。
pub open spec fn slot_bytes_after_writing_a_unit(slots: spec_fn(u64) -> Seq<u8>, start: u64, span: u64, unit: spec_fn(u64) -> Seq<u8>) -> spec_fn(u64) -> Seq<u8> {
    |slot: u64| if start <= slot && slot < start + span { unit((slot - start) as u64) } else { slots(slot) }
}

proof fn a_two_slot_destination_over_a_one_slot_unit_passes_the_p4_premise() {
    // 映射：key 5 → 槽 3（一槽单元），key 9 → 槽 6（两槽单元，要搬）。目的地：槽 2（两槽）。
    let mapping = Map::<u64, u64>::empty().insert(5u64, 3u64).insert(9u64, 6u64);
    let destinations = seq![2u64];
    assert(destination_is_free_in_the_mapping(mapping, 2u64)) by {
        assert forall|key: u64| #[trigger] mapping.contains_key(key) implies mapping[key] != 2u64 by {
            if key == 5 { assert(mapping[key] == 3); } else { assert(key == 9); assert(mapping[key] == 6); }
        }
    }
    assert(destinations_are_distinct_and_free(mapping, destinations, 1));
    // 具体盘：槽 3 装着 key 5 的字节 [5]，槽 6、7 装着 key 9 的两半 [9]、[90]。
    let slots: spec_fn(u64) -> Seq<u8> = |slot: u64| if slot == 3 { seq![5u8] } else if slot == 6 { seq![9u8] } else if slot == 7 { seq![90u8] } else { seq![0u8] };
    let unit_of_key_nine: spec_fn(u64) -> Seq<u8> = |offset: u64| if offset == 0 { seq![9u8] } else { seq![90u8] };
    let moved = slot_bytes_after_writing_a_unit(slots, 2, 2, unit_of_key_nine);
    assert(slots(3) == seq![5u8]);
    assert(moved(3) == seq![90u8]);
    assert(moved(3) != slots(3)) by { assert(moved(3)[0] != slots(3)[0]); }
}

fn main() {}

} // verus!
