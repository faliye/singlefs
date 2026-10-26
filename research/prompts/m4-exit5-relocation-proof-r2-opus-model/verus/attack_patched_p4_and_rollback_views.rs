// 攻的是第一轮判决给的两处修补合起来的形：
//   P4 目的地前提按区间收严（目的区间两两不交、与映射里每个活单元的区间不交）；
//   gc 协议补持久回退目标视图（Crash 不清、Write / Reclaim 要避开），每个视图带自己的逻辑版本。
// 两样都只用冻结副本里的 spec（P4 的 mapping_after_relocating / storage_after_relocating，gc 协议的 gc_view_matches / gc_map_avoids）
// 加这里写的收严，不改冻结副本。
use vstd::prelude::*;
use crate::compaction_plan_proofs::*;
use crate::gc_crash_proofs::*;

verus! {

// ---- 收严之后的 P4 目的地前提（区间版）----
pub open spec fn intervals_are_disjoint(a_start: u64, a_span: u64, b_start: u64, b_span: u64) -> bool {
    a_start + a_span <= b_start || b_start + b_span <= a_start
}

pub open spec fn destination_interval_is_free_in_the_mapping(
    mapping: Map<u64, u64>, spans: Map<u64, u64>, destination: u64, span: u64,
) -> bool {
    forall|key: u64| #[trigger] mapping.contains_key(key) ==>
        intervals_are_disjoint(mapping[key], spans[key], destination, span)
}

pub open spec fn destinations_are_disjoint_intervals_and_free(
    mapping: Map<u64, u64>, spans: Map<u64, u64>, keys: Seq<u64>, destinations: Seq<u64>, count: int,
) -> bool {
    &&& count <= destinations.len() && count <= keys.len()
    &&& forall|earlier: int, later: int|
        #![trigger destinations[earlier], destinations[later]]
        0 <= earlier < later < count ==> intervals_are_disjoint(
            destinations[earlier], spans[keys[earlier]], destinations[later], spans[keys[later]])
    &&& forall|index: int| 0 <= index < count ==> destination_interval_is_free_in_the_mapping(
        mapping, spans, #[trigger] destinations[index], spans[keys[index]])
}

// ---- 补了持久回退目标视图的协议里，Write 的允许条件 ----
// 现行、持久根、留存读者照冻结副本；另加每个持久回退视图（根环里仍是回退候选的根）。
pub open spec fn patched_slot_is_unreferenced(
    s: GcCrashState, rollback_views: Seq<Map<u64, u64>>, slot: u64,
) -> bool {
    &&& gc_slot_is_unreferenced(s, slot)
    &&& forall|i: int| 0 <= i < rollback_views.len() ==> gc_map_avoids(#[trigger] rollback_views[i], slot)
}

// 具体状态：一个单槽单元 key 7 在槽 10，是现行那一版唯一的 key；
// 回退目标（用户删掉 key 4 之前的那一版）里 key 4 在槽 20，现行映射里已没有 key 4，槽 20 已释放、还被回退窗口护着。
pub open spec fn current_mapping() -> Map<u64, u64> { map![7u64 => 10u64] }
pub open spec fn rollback_target_mapping() -> Map<u64, u64> { map![7u64 => 10u64, 4u64 => 20u64] }
pub open spec fn spans() -> Map<u64, u64> { map![7u64 => 1u64, 4u64 => 1u64] }
pub open spec fn storage_before() -> spec_fn(u64) -> Seq<u8> {
    |slot: u64| if slot == 10 { seq![42u8] } else if slot == 20 { seq![99u8] } else { seq![0u8] }
}
pub open spec fn rollback_target_logical() -> Map<u64, Seq<u8>> { map![7u64 => seq![42u8], 4u64 => seq![99u8]] }

// 收严之后的 P4 前提成立、把 key 7 搬到槽 20；现行那一版读到的不变（P4 的结论），
// 回退目标读 key 4 读到的变了，而补了回退视图的协议里往槽 20 写是不允许的。
pub proof fn the_interval_premise_lets_a_relocation_overwrite_a_slot_the_rollback_target_reads() {
    let mapping = current_mapping();
    let keys = seq![7u64];
    let sources = seq![10u64];
    let destinations = seq![20u64];
    assert(destinations_are_disjoint_intervals_and_free(mapping, spans(), keys, destinations, 1)) by {
        assert forall|key: u64| #[trigger] mapping.contains_key(key) implies
            intervals_are_disjoint(mapping[key], spans()[key], 20u64, spans()[7u64]) by {
            assert(key == 7u64);
        }
    }
    // 冻结副本那一版（起点互异）的前提同样成立。
    assert(destinations_are_distinct_and_free(mapping, destinations, 1)) by {
        assert forall|index: int| 0 <= index < 1 implies destination_is_free_in_the_mapping(
            mapping, #[trigger] destinations[index]) by {
            assert forall|key: u64| #[trigger] mapping.contains_key(key) implies mapping[key] != destinations[index] by {
                assert(key == 7u64);
            }
        }
    }
    let moved_mapping = mapping_after_relocating(mapping, keys, destinations, 1);
    let moved_storage = storage_after_relocating(storage_before(), sources, destinations, 1);
    assert(mapping_after_relocating(mapping, keys, destinations, 0) == mapping);
    assert(storage_after_relocating(storage_before(), sources, destinations, 0) == storage_before());
    // 现行那一版：key 7 读到的字节不变。
    assert(moved_mapping[7u64] == 20u64);
    assert(moved_storage(20u64) == seq![42u8]);
    assert(bytes_read_through_the_mapping(moved_mapping, moved_storage, 7u64)
        == bytes_read_through_the_mapping(mapping, storage_before(), 7u64));
    // 回退目标：key 4 读到的从 [99] 变成 [42]。
    assert(bytes_read_through_the_mapping(rollback_target_mapping(), storage_before(), 4u64) == seq![99u8]);
    assert(bytes_read_through_the_mapping(rollback_target_mapping(), moved_storage, 4u64) == seq![42u8]);
    assert(bytes_read_through_the_mapping(rollback_target_mapping(), moved_storage, 4u64)
        != bytes_read_through_the_mapping(rollback_target_mapping(), storage_before(), 4u64)) by {
        assert(seq![42u8][0] != seq![99u8][0]);
    }
    // 补了回退视图的协议：同一个写不允许。两半之间缺一条桥。
    let s = GcCrashState {
        disk: map![10u64 => seq![42u8], 20u64 => seq![99u8]],
        current: mapping,
        durable: mapping,
        readers: Seq::empty(),
    };
    assert(rollback_target_mapping().contains_key(4u64));
    assert(!gc_map_avoids(rollback_target_mapping(), 20u64));
    assert(!patched_slot_is_unreferenced(s, seq![rollback_target_mapping()], 20u64)) by {
        assert(seq![rollback_target_mapping()][0] == rollback_target_mapping());
    }
    // 冻结副本那一版协议（没有回退视图）却允许。
    assert(gc_slot_is_unreferenced(s, 20u64)) by {
        assert forall|key: u64| s.current.contains_key(key) implies #[trigger] s.current[key] != 20u64 by {
            assert(key == 7u64);
        }
        assert forall|key: u64| s.durable.contains_key(key) implies #[trigger] s.durable[key] != 20u64 by {
            assert(key == 7u64);
        }
    }
}

// 第二个变体：两槽单元。协议按起点槽号避让（gc_map_avoids 只比 mapping[key] != slot），
// 回退目标里 key 4 是两槽单元、起点 20 罩 20、21；往槽 21 写在补了回退视图的协议里照样允许。
// 判决给 P4 的区间收严碰不到协议自己的避让条件。
pub proof fn the_patched_protocol_still_avoids_by_start_slot_only() {
    let s = GcCrashState {
        disk: map![10u64 => seq![42u8], 20u64 => seq![99u8, 98u8]],
        current: current_mapping(),
        durable: current_mapping(),
        readers: Seq::empty(),
    };
    let views = seq![rollback_target_mapping()];
    assert(patched_slot_is_unreferenced(s, views, 21u64)) by {
        assert forall|key: u64| s.current.contains_key(key) implies #[trigger] s.current[key] != 21u64 by {
            assert(key == 7u64);
        }
        assert forall|key: u64| s.durable.contains_key(key) implies #[trigger] s.durable[key] != 21u64 by {
            assert(key == 7u64);
        }
        assert forall|i: int| 0 <= i < views.len() implies gc_map_avoids(#[trigger] views[i], 21u64) by {
            assert(views[i] == rollback_target_mapping());
            assert forall|key: u64| rollback_target_mapping().contains_key(key) implies
                #[trigger] rollback_target_mapping()[key] != 21u64 by {
                assert(key == 7u64 || key == 4u64);
            }
        }
    }
    // 罩 20..22 的那个两槽单元，第二槽是 21：区间里有它。
    assert(!intervals_are_disjoint(rollback_target_mapping()[4u64], 2u64, 21u64, 1u64));
}

} // verus!
