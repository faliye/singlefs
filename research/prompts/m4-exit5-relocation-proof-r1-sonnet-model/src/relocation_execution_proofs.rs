//! 搬迁执行的最小原型核（推论轮 m4-exit5-relocation-proof-r1，正推腿）：核外提候选的四样里能在副本上验的三样。
//! - 条目判定：`transaction.rs` 的 `relocation_checked_against_the_version` 去掉读树之后剩下的那一判（钉住拒、源要等于映射位置且跨度相同、字节非空）。
//! - 目的地判定：一次发布的目的地两两不同、都不在受保护集里（受保护集 = 分配器里没回收的记录，由调用方给）。
//! - 一次发布的持久前缀：单元写（任意子集、任意次序）→ 屏障 → 换映射（Relocate）→ 根（Checkpoint），在任一前缀上崩溃，所有视图读到的字节不变。
//! 归约到 `gc_crash_proofs` 的协议：这里不另立抽象状态，用它的 `GcCrashState`、`gc_next`、`gc_step_enabled` 和已证的一步保持引理。
#[allow(unused_imports)]
use crate::gc_crash_proofs::*;
use vstd::prelude::*;

verus! {

// ===== 一、条目判定 =====
pub enum RelocationItemVerdict {
    Accepted,
    PinnedUnit,
    SourceIsNotTheMappingPosition,
    BytesNotAvailable,
}

// 胶水（transaction.rs 的 relocation_checked_against_the_version）读完树、取到角色与映射位置之后交给核的事实。
pub struct RelocationItemFacts {
    pub role_is_pinned: bool,
    pub mapping_copy_slots: Option<(u64, u64)>,
    pub planned_source_slot: u64,
    pub planned_source_span: u64,
    pub role_span_slots: u64,
    pub bytes_available: bool,
}

pub open spec fn source_is_the_mapping_position(facts: RelocationItemFacts) -> bool {
    match facts.mapping_copy_slots {
        Some((first, second)) => first == facts.planned_source_slot && second == facts.planned_source_slot
            && facts.planned_source_span == facts.role_span_slots,
        None => false,
    }
}

pub open spec fn relocation_item_is_acceptable(facts: RelocationItemFacts) -> bool {
    &&& !facts.role_is_pinned
    &&& source_is_the_mapping_position(facts)
    &&& facts.bytes_available
}

pub fn judge_relocation_item(facts: &RelocationItemFacts) -> (verdict: RelocationItemVerdict)
    ensures
        (verdict is Accepted) <==> relocation_item_is_acceptable(*facts),
        (verdict is PinnedUnit) <==> facts.role_is_pinned,
        (verdict is SourceIsNotTheMappingPosition) <==> (!facts.role_is_pinned
            && !source_is_the_mapping_position(*facts)),
        (verdict is BytesNotAvailable) <==> (!facts.role_is_pinned
            && source_is_the_mapping_position(*facts) && !facts.bytes_available),
{
    if facts.role_is_pinned {
        return RelocationItemVerdict::PinnedUnit;
    }
    let at_the_position = match facts.mapping_copy_slots {
        Some((first, second)) => first == facts.planned_source_slot
            && second == facts.planned_source_slot
            && facts.planned_source_span == facts.role_span_slots,
        None => false,
    };
    if !at_the_position {
        return RelocationItemVerdict::SourceIsNotTheMappingPosition;
    }
    if !facts.bytes_available {
        return RelocationItemVerdict::BytesNotAvailable;
    }
    RelocationItemVerdict::Accepted
}

// ===== 二、目的地判定 =====
pub open spec fn slots_are_pairwise_distinct(slots: Seq<u64>) -> bool {
    forall|earlier: int, later: int|
        #![trigger slots[earlier], slots[later]]
        0 <= earlier < later < slots.len() ==> slots[earlier] != slots[later]
}

pub open spec fn no_slot_is_protected(slots: Seq<u64>, protected: Seq<u64>) -> bool {
    forall|position: int, other: int|
        #![trigger slots[position], protected[other]]
        0 <= position < slots.len() && 0 <= other < protected.len() ==> slots[position] != protected[other]
}

pub open spec fn relocation_destinations_are_acceptable_spec(destinations: Seq<u64>, protected: Seq<u64>) -> bool {
    slots_are_pairwise_distinct(destinations) && no_slot_is_protected(destinations, protected)
}

pub fn relocation_destinations_are_acceptable(destinations: &[u64], protected: &[u64]) -> (acceptable: bool)
    ensures
        acceptable == relocation_destinations_are_acceptable_spec(destinations@, protected@),
{
    let mut position: usize = 0;
    while position < destinations.len()
        invariant
            position <= destinations.len(),
            forall|earlier: int, later: int|
                #![trigger destinations@[earlier], destinations@[later]]
                0 <= earlier < later < position ==> destinations@[earlier] != destinations@[later],
            forall|earlier: int, other: int|
                #![trigger destinations@[earlier], protected@[other]]
                0 <= earlier < position && 0 <= other < protected@.len() ==> destinations@[earlier] != protected@[other],
        decreases destinations.len() - position,
    {
        let mut earlier: usize = 0;
        while earlier < position
            invariant
                earlier <= position,
                position < destinations.len(),
                forall|index: int| #![trigger destinations@[index]]
                    0 <= index < earlier ==> destinations@[index] != destinations@[position as int],
            decreases position - earlier,
        {
            if destinations[earlier] == destinations[position] {
                assert(!slots_are_pairwise_distinct(destinations@)) by {
                    assert(destinations@[earlier as int] == destinations@[position as int]);
                }
                return false;
            }
            earlier += 1;
        }
        let mut other: usize = 0;
        while other < protected.len()
            invariant
                other <= protected.len(),
                position < destinations.len(),
                forall|index: int| #![trigger protected@[index]]
                    0 <= index < other ==> destinations@[position as int] != protected@[index],
            decreases protected.len() - other,
        {
            if destinations[position] == protected[other] {
                assert(!no_slot_is_protected(destinations@, protected@)) by {
                    assert(destinations@[position as int] == protected@[other as int]);
                }
                return false;
            }
            other += 1;
        }
        position += 1;
    }
    true
}

// 调用方的前提（allocator 一侧）：每个被当前根、持久根、留存读者引用的槽都在受保护集里。
// 这是 gc_crash_proofs 自己列的「floor 不超过受保护读者与根的版本」那一条未连接前提的落点；这里不证它，写成参数。
pub open spec fn protected_slots_cover_every_view(s: GcCrashState, protected: Seq<u64>) -> bool {
    forall|slot: u64| !gc_slot_is_unreferenced(s, slot) ==> exists|index: int|
        #![trigger protected[index]]
        0 <= index < protected.len() && protected[index] == slot
}

pub proof fn lemma_acceptable_destinations_are_unreferenced(
    s: GcCrashState, protected: Seq<u64>, destinations: Seq<u64>,
)
    requires
        protected_slots_cover_every_view(s, protected),
        relocation_destinations_are_acceptable_spec(destinations, protected),
    ensures
        slots_are_pairwise_distinct(destinations),
        forall|position: int| 0 <= position < destinations.len() ==>
            gc_slot_is_unreferenced(s, #[trigger] destinations[position]),
{
    assert forall|position: int| 0 <= position < destinations.len() implies
        gc_slot_is_unreferenced(s, #[trigger] destinations[position]) by {
        if !gc_slot_is_unreferenced(s, destinations[position]) {
            let index = choose|index: int| 0 <= index < protected.len() && protected[index] == destinations[position];
            assert(destinations[position] != protected[index]);
        }
    }
}

// ===== 三、一次发布的持久前缀 =====
// 持久顺序（D16 已定项 7）：单元写 → 屏障 → journal → 屏障 → 根 FUA。协议里对应：每项一个 Write（目的字节 = 源字节），
// 每项一个 Relocate，最后一个 Checkpoint。屏障把「Write 全在 Relocate 与 Checkpoint 之前」钉成次序；设备真守不守屏障不在这里证。
pub open spec fn relocation_copy_bytes(s: GcCrashState, key: u64) -> Seq<u8> {
    s.disk[s.current[key]]
}

pub open spec fn state_after_the_copies(
    s: GcCrashState, keys: Seq<u64>, destinations: Seq<u64>, count: int,
) -> GcCrashState
    decreases count,
{
    if count <= 0 {
        s
    } else {
        gc_next(
            state_after_the_copies(s, keys, destinations, count - 1),
            GcCrashStep::Write { slot: destinations[count - 1], bytes: relocation_copy_bytes(s, keys[count - 1]) },
        )
    }
}

pub open spec fn state_after_the_mapping_swaps(
    copied: GcCrashState, keys: Seq<u64>, destinations: Seq<u64>, count: int,
) -> GcCrashState
    decreases count,
{
    if count <= 0 {
        copied
    } else {
        gc_next(
            state_after_the_mapping_swaps(copied, keys, destinations, count - 1),
            GcCrashStep::Relocate { key: keys[count - 1], destination: destinations[count - 1] },
        )
    }
}

// 一次搬迁发布的形状：key 两两不同、目的地两两不同、源在当前映射里、目的地没被任何视图引用。
pub open spec fn relocation_publish_is_well_formed(s: GcCrashState, keys: Seq<u64>, destinations: Seq<u64>) -> bool {
    &&& keys.len() == destinations.len()
    &&& slots_are_pairwise_distinct(keys)
    &&& slots_are_pairwise_distinct(destinations)
    &&& forall|position: int| 0 <= position < keys.len() ==>
        s.current.contains_key(#[trigger] keys[position])
            && gc_slot_is_unreferenced(s, destinations[position])
}

// 任意一组写（任意次序、任意子集、可重复），只要每个槽没被任何视图引用，所有视图读到的字节不变。
pub open spec fn state_after_the_writes(s: GcCrashState, writes: Seq<(u64, Seq<u8>)>) -> GcCrashState
    decreases writes.len(),
{
    if writes.len() == 0 {
        s
    } else {
        gc_next(
            state_after_the_writes(s, writes.drop_last()),
            GcCrashStep::Write { slot: writes.last().0, bytes: writes.last().1 },
        )
    }
}

pub proof fn lemma_any_unreferenced_writes_keep_the_views(
    s: GcCrashState, logical: Map<u64, Seq<u8>>, writes: Seq<(u64, Seq<u8>)>,
)
    requires
        gc_all_views_match(s, logical),
        forall|position: int| 0 <= position < writes.len() ==> gc_slot_is_unreferenced(s, #[trigger] writes[position].0),
    ensures
        gc_all_views_match(state_after_the_writes(s, writes), logical),
        state_after_the_writes(s, writes).current == s.current,
        state_after_the_writes(s, writes).durable == s.durable,
        state_after_the_writes(s, writes).readers == s.readers,
        gc_all_views_match(gc_next(state_after_the_writes(s, writes), GcCrashStep::Crash), logical),
    decreases writes.len(),
{
    if writes.len() > 0 {
        let before = writes.drop_last();
        assert forall|position: int| 0 <= position < before.len() implies
            gc_slot_is_unreferenced(s, #[trigger] before[position].0) by {
            assert(before[position] == writes[position]);
        }
        lemma_any_unreferenced_writes_keep_the_views(s, logical, before);
        let earlier = state_after_the_writes(s, before);
        let step = GcCrashStep::Write { slot: writes.last().0, bytes: writes.last().1 };
        assert(writes.last() == writes[writes.len() - 1]);
        assert(gc_slot_is_unreferenced(earlier, writes.last().0));
        lemma_gc_step_preserves_current_durable_and_old_readers(earlier, step, logical);
    }
    let end = state_after_the_writes(s, writes);
    lemma_gc_step_preserves_current_durable_and_old_readers(end, GcCrashStep::Crash, logical);
}

pub proof fn lemma_the_copies_keep_the_views_and_the_mappings(
    s: GcCrashState, logical: Map<u64, Seq<u8>>, keys: Seq<u64>, destinations: Seq<u64>, count: int,
)
    requires
        gc_all_views_match(s, logical),
        relocation_publish_is_well_formed(s, keys, destinations),
        0 <= count <= keys.len(),
    ensures
        gc_all_views_match(state_after_the_copies(s, keys, destinations, count), logical),
        state_after_the_copies(s, keys, destinations, count).current == s.current,
        state_after_the_copies(s, keys, destinations, count).durable == s.durable,
        state_after_the_copies(s, keys, destinations, count).readers == s.readers,
        forall|position: int| 0 <= position < count ==>
            state_after_the_copies(s, keys, destinations, count).disk.contains_key(#[trigger] destinations[position])
            && state_after_the_copies(s, keys, destinations, count).disk[destinations[position]]
                == relocation_copy_bytes(s, keys[position]),
        forall|slot: u64| s.disk.contains_key(slot) ==>
            state_after_the_copies(s, keys, destinations, count).disk.contains_key(slot),
        forall|slot: u64| #![trigger s.disk.contains_key(slot)]
            (forall|position: int| 0 <= position < count ==> destinations[position] != slot) ==>
            state_after_the_copies(s, keys, destinations, count).disk.contains_key(slot) == s.disk.contains_key(slot),
        forall|slot: u64| #![trigger s.disk[slot]]
            (forall|position: int| 0 <= position < count ==> destinations[position] != slot)
                && s.disk.contains_key(slot) ==>
            state_after_the_copies(s, keys, destinations, count).disk[slot] == s.disk[slot],
    decreases count,
{
    if count > 0 {
        lemma_the_copies_keep_the_views_and_the_mappings(s, logical, keys, destinations, count - 1);
        let earlier = state_after_the_copies(s, keys, destinations, count - 1);
        let step = GcCrashStep::Write { slot: destinations[count - 1], bytes: relocation_copy_bytes(s, keys[count - 1]) };
        assert(gc_slot_is_unreferenced(s, destinations[count - 1]));
        assert(gc_slot_is_unreferenced(earlier, destinations[count - 1]));
        lemma_gc_step_preserves_current_durable_and_old_readers(earlier, step, logical);
        let after = state_after_the_copies(s, keys, destinations, count);
        assert(after == gc_next(earlier, step));
        assert(after.disk == earlier.disk.insert(destinations[count - 1], relocation_copy_bytes(s, keys[count - 1])));
        assert forall|slot: u64| (forall|position: int| 0 <= position < count ==> destinations[position] != slot) implies
            after.disk.contains_key(slot) == s.disk.contains_key(slot) by {
            assert(destinations[count - 1] != slot);
            assert forall|position: int| 0 <= position < count - 1 implies destinations[position] != slot by {
                assert(destinations[position] != slot);
            }
            assert(earlier.disk.contains_key(slot) == s.disk.contains_key(slot));
        }
        assert forall|slot: u64| (forall|position: int| 0 <= position < count ==> destinations[position] != slot)
            && s.disk.contains_key(slot) implies after.disk[slot] == s.disk[slot] by {
            assert(destinations[count - 1] != slot);
            assert forall|position: int| 0 <= position < count - 1 implies destinations[position] != slot by {
                assert(destinations[position] != slot);
            }
            assert(earlier.disk.contains_key(slot) == s.disk.contains_key(slot));
            assert(earlier.disk[slot] == s.disk[slot]);
        }
    }
}

pub proof fn lemma_the_mapping_swaps_keep_the_views(
    s: GcCrashState, logical: Map<u64, Seq<u8>>, keys: Seq<u64>, destinations: Seq<u64>, count: int,
)
    requires
        gc_all_views_match(s, logical),
        relocation_publish_is_well_formed(s, keys, destinations),
        0 <= count <= keys.len(),
    ensures
        gc_all_views_match(
            state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
                keys, destinations, count), logical),
        state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
            keys, destinations, count).disk == state_after_the_copies(s, keys, destinations, keys.len() as int).disk,
        state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
            keys, destinations, count).durable == s.durable,
        state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
            keys, destinations, count).readers == s.readers,
        forall|position: int| 0 <= position < count ==>
            state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
                keys, destinations, count).current.contains_key(#[trigger] keys[position])
            && state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
                keys, destinations, count).current[keys[position]] == destinations[position],
        forall|key: u64| #![trigger s.current.contains_key(key)]
            (forall|position: int| 0 <= position < count ==> keys[position] != key) ==>
            state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
                keys, destinations, count).current.contains_key(key) == s.current.contains_key(key),
        forall|key: u64| #![trigger s.current[key]]
            (forall|position: int| 0 <= position < count ==> keys[position] != key) && s.current.contains_key(key) ==>
            state_after_the_mapping_swaps(state_after_the_copies(s, keys, destinations, keys.len() as int),
                keys, destinations, count).current[key] == s.current[key],
    decreases count,
{
    let all = keys.len() as int;
    lemma_the_copies_keep_the_views_and_the_mappings(s, logical, keys, destinations, all);
    let copied = state_after_the_copies(s, keys, destinations, all);
    if count > 0 {
        lemma_the_mapping_swaps_keep_the_views(s, logical, keys, destinations, count - 1);
        let earlier = state_after_the_mapping_swaps(copied, keys, destinations, count - 1);
        let key = keys[count - 1];
        let destination = destinations[count - 1];
        let source = s.current[key];
        assert(s.current.contains_key(key));
        assert(s.disk.contains_key(source));
        assert forall|other: int| 0 <= other < count - 1 implies keys[other] != key by {
            assert(keys[other] != keys[count - 1]);
        }
        assert(earlier.current[key] == source);
        // 源槽不是任何目的地：目的地没被当前映射引用，而源槽被它引用。
        assert forall|position: int| 0 <= position < all implies destinations[position] != source by {
            assert(gc_slot_is_unreferenced(s, destinations[position]));
            assert(gc_map_avoids(s.current, destinations[position]));
            assert(s.current.contains_key(key));
        }
        assert(copied.disk[source] == s.disk[source]);
        assert(copied.disk[destination] == s.disk[source]);
        let step = GcCrashStep::Relocate { key, destination };
        assert(gc_step_enabled(earlier, step));
        lemma_gc_step_preserves_current_durable_and_old_readers(earlier, step, logical);
    }
}

// 主定理：一次搬迁发布（任意 K 项）在持久顺序的每个前缀上、每个崩溃点上，所有视图读到的字节不变；发布完成后根指向目的地。
pub proof fn theorem_a_relocation_publish_keeps_every_view_at_every_durable_prefix(
    s: GcCrashState, logical: Map<u64, Seq<u8>>, keys: Seq<u64>, destinations: Seq<u64>,
)
    requires
        gc_all_views_match(s, logical),
        relocation_publish_is_well_formed(s, keys, destinations),
    ensures
        forall|count: int| 0 <= count <= keys.len() ==> {
            &&& gc_all_views_match(state_after_the_copies(s, keys, destinations, count), logical)
            &&& gc_all_views_match(gc_next(state_after_the_copies(s, keys, destinations, count), GcCrashStep::Crash), logical)
        },
        forall|count: int| 0 <= count <= keys.len() ==> {
            &&& gc_all_views_match(state_after_the_mapping_swaps(
                state_after_the_copies(s, keys, destinations, keys.len() as int), keys, destinations, count), logical)
            &&& gc_all_views_match(gc_next(state_after_the_mapping_swaps(
                state_after_the_copies(s, keys, destinations, keys.len() as int), keys, destinations, count),
                GcCrashStep::Crash), logical)
        },
        ({
            let done = gc_next(state_after_the_mapping_swaps(
                state_after_the_copies(s, keys, destinations, keys.len() as int), keys, destinations, keys.len() as int),
                GcCrashStep::Checkpoint);
            &&& gc_all_views_match(done, logical)
            &&& gc_all_views_match(gc_next(done, GcCrashStep::Crash), logical)
            &&& done.durable == done.current
            &&& forall|position: int| 0 <= position < keys.len() ==> done.current[#[trigger] keys[position]] == destinations[position]
        }),
{
    let all = keys.len() as int;
    assert forall|count: int| 0 <= count <= all implies {
        &&& gc_all_views_match(state_after_the_copies(s, keys, destinations, count), logical)
        &&& gc_all_views_match(gc_next(state_after_the_copies(s, keys, destinations, count), GcCrashStep::Crash), logical)
    } by {
        lemma_the_copies_keep_the_views_and_the_mappings(s, logical, keys, destinations, count);
        lemma_gc_step_preserves_current_durable_and_old_readers(
            state_after_the_copies(s, keys, destinations, count), GcCrashStep::Crash, logical);
    }
    let copied = state_after_the_copies(s, keys, destinations, all);
    assert forall|count: int| 0 <= count <= all implies {
        &&& gc_all_views_match(state_after_the_mapping_swaps(copied, keys, destinations, count), logical)
        &&& gc_all_views_match(gc_next(state_after_the_mapping_swaps(copied, keys, destinations, count),
            GcCrashStep::Crash), logical)
    } by {
        lemma_the_mapping_swaps_keep_the_views(s, logical, keys, destinations, count);
        lemma_gc_step_preserves_current_durable_and_old_readers(
            state_after_the_mapping_swaps(copied, keys, destinations, count), GcCrashStep::Crash, logical);
    }
    lemma_the_mapping_swaps_keep_the_views(s, logical, keys, destinations, all);
    let swapped = state_after_the_mapping_swaps(copied, keys, destinations, all);
    lemma_gc_step_preserves_current_durable_and_old_readers(swapped, GcCrashStep::Checkpoint, logical);
    let done = gc_next(swapped, GcCrashStep::Checkpoint);
    lemma_gc_step_preserves_current_durable_and_old_readers(done, GcCrashStep::Crash, logical);
}

// 无前提的构造见证：两个单元、一个持有源槽的留存读者，一次两项的搬迁发布满足形状要求，防止上面的定理在空集上空真。
pub proof fn witness_a_two_unit_relocation_publish_is_well_formed()
    ensures
        exists|s: GcCrashState, logical: Map<u64, Seq<u8>>, keys: Seq<u64>, destinations: Seq<u64>|
            gc_all_views_match(s, logical) && relocation_publish_is_well_formed(s, keys, destinations)
                && keys.len() == 2 && s.readers.len() == 1,
{
    let first = seq![1u8];
    let second = seq![2u8];
    let logical = Map::empty().insert(7u64, first).insert(8u64, second);
    let mapping = Map::empty().insert(7u64, 10u64).insert(8u64, 11u64);
    let disk = Map::empty().insert(10u64, first).insert(11u64, second);
    let s = GcCrashState { disk, current: mapping, durable: mapping, readers: seq![mapping] };
    let keys = seq![7u64, 8u64];
    let destinations = seq![20u64, 21u64];
    assert(gc_view_matches(mapping, disk, logical)) by {
        assert(mapping.dom() =~= logical.dom());
    }
    assert(gc_all_views_match(s, logical));
    assert(relocation_publish_is_well_formed(s, keys, destinations)) by {
        assert forall|position: int| 0 <= position < 2 implies
            s.current.contains_key(#[trigger] keys[position]) && gc_slot_is_unreferenced(s, destinations[position]) by {
            if position == 0 {
                assert(gc_map_avoids(mapping, 20u64));
            } else {
                assert(gc_map_avoids(mapping, 21u64));
            }
        }
    }
}

} // verus!
