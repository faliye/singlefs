//! 在正推 r2 的改后协议（gc_ring_proofs + relocation_interval_proofs）上复跑第二轮攻方两个反例的变体。
//!
//! 结论：第二轮攻方「搬迁写进回退目标还在读的槽」（单槽）在根环版里被排除；
//! 「两槽单元的第二槽照样可写」那一形**没有被排除**：根环版只按起点槽号护（单槽模型，无跨度），
//! 区间版的前提只对现行映射量化（没有回退候选 / 持久根这一维）。两份改后协议各缺对方的那一维，
//! 合不成「护住回退候选的多槽单元的每一槽」。
use vstd::prelude::*;
use crate::gc_ring_proofs::*;
use crate::relocation_interval_proofs::*;

verus! {

// ========== A1：根环版的保护是起点槽号粒度（第二槽不设防） ==========
// 现行只有 key7 在槽 10；唯一的回退候选（roots[0]）里 key4 在槽 20。
// 在 core 里 key4 的单元跨两槽 [20,21]（见报告引的 span_slots）；根环模型记不下跨度，只记 mapping[key4]=20。
// 证：① 这是一个合规（ring_all_views_read_right）的状态；
//     ② 往起点槽 20 写**不**允许（第二轮攻方单槽变体一在这一版被排除）；
//     ③ 往第二槽 21 写**允许**（gap：候选单元的第二槽不设防）。
pub proof fn a1_ring_protection_is_start_slot_only()
    ensures
        ({
            let b7 = seq![42u8];
            let b4 = seq![99u8];
            let head = RootView {
                mapping: Map::<u64, u64>::empty().insert(7u64, 10u64),
                logical: Map::<u64, Seq<u8>>::empty().insert(7u64, b7),
            };
            let candidate = RootView {
                mapping: Map::<u64, u64>::empty().insert(4u64, 20u64),
                logical: Map::<u64, Seq<u8>>::empty().insert(4u64, b4),
            };
            let s = GcRingState {
                disk: Map::<u64, Seq<u8>>::empty().insert(10u64, b7).insert(20u64, b4).insert(21u64, seq![7u8]),
                roots: seq![candidate],
                floor: 0,
                head,
                readers: Seq::empty(),
                window: 2,
            };
            &&& ring_all_views_read_right(s)
            &&& !ring_step_enabled(s, RingStep::Write { slot: 20u64, bytes: seq![0u8] })
            &&& ring_step_enabled(s, RingStep::Write { slot: 21u64, bytes: seq![0u8] })
        }),
{
    let b7 = seq![42u8];
    let b4 = seq![99u8];
    let head = RootView {
        mapping: Map::<u64, u64>::empty().insert(7u64, 10u64),
        logical: Map::<u64, Seq<u8>>::empty().insert(7u64, b7),
    };
    let candidate = RootView {
        mapping: Map::<u64, u64>::empty().insert(4u64, 20u64),
        logical: Map::<u64, Seq<u8>>::empty().insert(4u64, b4),
    };
    let s = GcRingState {
        disk: Map::<u64, Seq<u8>>::empty().insert(10u64, b7).insert(20u64, b4).insert(21u64, seq![7u8]),
        roots: seq![candidate],
        floor: 0,
        head,
        readers: Seq::empty(),
        window: 2,
    };
    // ① 合规
    assert(view_reads_right(head, s.disk)) by {
        assert(head.mapping.dom() =~= head.logical.dom());
    }
    assert(s.roots[0] == candidate);
    assert(view_reads_right(candidate, s.disk)) by {
        assert(candidate.mapping.dom() =~= candidate.logical.dom());
    }
    assert(ring_all_views_read_right(s)) by {
        assert forall|i: int| s.floor <= i < s.roots.len() implies view_reads_right(#[trigger] s.roots[i], s.disk) by {
            assert(i == 0);
        }
    }
    // ② 起点槽 20 被候选护着：写不允许
    assert(!ring_slot_is_unreferenced(s, 20u64)) by {
        assert(!ring_map_avoids(s.roots[0].mapping, 20u64)) by {
            assert(s.roots[0].mapping.contains_key(4u64));
            assert(s.roots[0].mapping[4u64] == 20u64);
        }
    }
    // ③ 第二槽 21 不设防：写允许
    assert(ring_map_avoids(s.head.mapping, 21u64)) by {
        assert forall|key: u64| s.head.mapping.contains_key(key) implies #[trigger] s.head.mapping[key] != 21u64 by {
            assert(key == 7u64);
        }
    }
    assert(ring_slot_is_unreferenced(s, 21u64)) by {
        assert forall|i: int| s.floor <= i < s.roots.len() implies ring_map_avoids(#[trigger] s.roots[i].mapping, 21u64) by {
            assert(i == 0);
            assert forall|key: u64| s.roots[0].mapping.contains_key(key) implies #[trigger] s.roots[0].mapping[key] != 21u64 by {
                assert(key == 4u64);
            }
        }
    }
}

// ========== A2：区间版 P4 的前提只对现行映射量化，回退目标那一维不在里面 ==========
// 现行映射只有 key7 -> [10,11)；回退目标里 key4 -> [20,22)（两槽单元）。
// 搬 key7 到目的区间 [21,22)：它与现行映射里唯一的区间 [10,11) 不重叠，所以计划良构（前提过）。
// 定理保住现行映射每个 key 的读；但回退目标 key4 的那两槽读变了（第二槽 21 被覆写）。
pub open spec fn storage_before() -> spec_fn(int) -> Seq<u8> {
    |slot: int|
        if slot == 10 { seq![42u8] }
        else if slot == 20 { seq![99u8] }
        else if slot == 21 { seq![88u8] }
        else { seq![0u8] }
}

pub proof fn a2_interval_premise_ignores_rollback_target_runs()
    ensures
        ({
            let current = Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 });
            let keys = seq![7u64];
            let sources = seq![SlotRun { start: 10, span: 1 }];
            let destinations = seq![SlotRun { start: 21, span: 1 }];
            let rollback_run = SlotRun { start: 20, span: 2 };
            let moved = storage_after_relocating_runs(storage_before(), sources, destinations, 1);
            // 计划良构：区间前提只看现行映射，过
            &&& relocation_plan_is_well_formed(current, keys, sources, destinations, 1)
            // 现行映射里 key7 的读保住（定理的结论域）
            &&& unit_read(moved, SlotRun { start: 21, span: 1 })
                == unit_read(storage_before(), SlotRun { start: 10, span: 1 })
            // 回退目标 key4（两槽单元 [20,22)）读变了——改后协议没护住它
            &&& unit_read(moved, rollback_run) != unit_read(storage_before(), rollback_run)
        }),
{
    let current = Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 });
    let keys = seq![7u64];
    let sources = seq![SlotRun { start: 10, span: 1 }];
    let destinations = seq![SlotRun { start: 21, span: 1 }];
    let rollback_run = SlotRun { start: 20, span: 2 };
    // 前提
    assert(relocation_plan_is_well_formed(current, keys, sources, destinations, 1)) by {
        assert forall|key: u64| #[trigger] current.contains_key(key) implies current[key].span >= 1 by {
            assert(key == 7u64);
        }
        assert(current.contains_key(keys[0]));
        assert(current[keys[0]] == sources[0]);
        assert(destination_runs_are_disjoint_and_free(current, destinations, 1)) by {
            assert forall|index: int, key: u64|
                0 <= index < 1 && #[trigger] current.contains_key(key) implies
                !runs_overlap(#[trigger] destinations[index], current[key]) by {
                assert(index == 0);
                assert(key == 7u64);
                // [21,22) 与 [10,11) 不重叠
            }
        }
    }
    let moved = storage_after_relocating_runs(storage_before(), sources, destinations, 1);
    assert(storage_after_relocating_runs(storage_before(), sources, destinations, 0) == storage_before());
    // moved(21) = storage_before(10) = [42]；moved(20) = storage_before(20) = [99]
    assert(moved =~= storage_after_copying_a_run(storage_before(), sources[0], destinations[0]));
    assert(moved(21) == storage_before()(10)) by {
        assert(run_covers(destinations[0], 21));
    }
    assert(moved(20) == storage_before()(20)) by {
        assert(!run_covers(destinations[0], 20));
    }
    // 现行 key7 读保住：读 [21,22) = moved(21) = [42] = storage_before 读 [10,11)
    assert(unit_read(moved, SlotRun { start: 21, span: 1 }) =~= unit_read(storage_before(), SlotRun { start: 10, span: 1 })) by {
        assert(unit_read(moved, SlotRun { start: 21, span: 1 }).len() == 1);
        assert(unit_read(moved, SlotRun { start: 21, span: 1 })[0] == moved(21));
        assert(unit_read(storage_before(), SlotRun { start: 10, span: 1 })[0] == storage_before()(10));
    }
    // 回退目标 key4 两槽读变了：第二槽（21）从 [88] 变成 [42]
    assert(unit_read(moved, rollback_run)[1] == moved(21)) by {
        assert(rollback_run.span == 2);
    }
    assert(unit_read(storage_before(), rollback_run)[1] == storage_before()(21));
    assert(moved(21) == seq![42u8]);
    assert(storage_before()(21) == seq![88u8]);
    assert(unit_read(moved, rollback_run) != unit_read(storage_before(), rollback_run)) by {
        assert(unit_read(moved, rollback_run)[1] != unit_read(storage_before(), rollback_run)[1]) by {
            assert(seq![42u8][0] != seq![88u8][0]);
        }
    }
}

} // verus!
