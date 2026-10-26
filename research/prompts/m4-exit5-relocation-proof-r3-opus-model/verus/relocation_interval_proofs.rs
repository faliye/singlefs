//! P4 的区间版：搬迁的目的地按「槽区间不重叠」说，不再只比起点槽号。
//!
//! 第一轮攻方打中的是 `compaction_plan_proofs::destinations_are_distinct_and_free` 只要求目的地起点两两不同、不等于映射里任何一个起点：
//! 两槽目的地（起点 2，罩 2、3 两槽）罩住了起点在 3 的一槽单元，前提成立而那个单元的字节变了。
//! 这里的具体状态是「每个槽一段字节」，映射值是「起点加跨度」，读一个 key 是读它的区间里每个槽。
//! 前提：目的区间两两不重叠、都不与映射里任何一个区间重叠、跨度等于源区间的跨度。
//! 槽号与跨度用 int（协议层的数学整数），接 core 的 u64 时要另证 `start + span` 不溢出（见报告 Y5）。
use vstd::prelude::*;

verus! {

pub struct SlotRun {
    pub start: int,
    pub span: int,
}

pub open spec fn run_covers(run: SlotRun, slot: int) -> bool {
    run.start <= slot < run.start + run.span
}

pub open spec fn runs_overlap(a: SlotRun, b: SlotRun) -> bool {
    a.start < b.start + b.span && b.start < a.start + a.span
}

// 读一个单元：区间里每个槽上的字节，按槽序排成一串。
pub open spec fn unit_read(storage: spec_fn(int) -> Seq<u8>, run: SlotRun) -> Seq<Seq<u8>> {
    Seq::new(run.span as nat, |offset: int| storage(run.start + offset))
}

// 搬一个单元：源区间每个槽上的字节原样写到目的区间对应的槽，别的槽不动。
pub open spec fn storage_after_copying_a_run(
    storage: spec_fn(int) -> Seq<u8>,
    source: SlotRun,
    destination: SlotRun,
) -> spec_fn(int) -> Seq<u8> {
    |slot: int|
        if run_covers(destination, slot) {
            storage(source.start + (slot - destination.start))
        } else {
            storage(slot)
        }
}

pub open spec fn storage_after_relocating_runs(
    storage: spec_fn(int) -> Seq<u8>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
) -> spec_fn(int) -> Seq<u8>
    decreases count,
{
    if count <= 0 {
        storage
    } else {
        storage_after_copying_a_run(
            storage_after_relocating_runs(storage, sources, destinations, count - 1),
            sources[count - 1],
            destinations[count - 1],
        )
    }
}

pub open spec fn mapping_after_relocating_runs(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    destinations: Seq<SlotRun>,
    count: int,
) -> Map<u64, SlotRun>
    decreases count,
{
    if count <= 0 {
        mapping
    } else {
        mapping_after_relocating_runs(mapping, keys, destinations, count - 1).insert(
            keys[count - 1],
            destinations[count - 1],
        )
    }
}

// 目的区间的前提（按区间，不是按起点）：前 count 个目的区间两两不重叠，且与映射里任何一个区间都不重叠。
pub open spec fn destination_runs_are_disjoint_and_free(
    mapping: Map<u64, SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
) -> bool {
    &&& 0 <= count <= destinations.len()
    &&& forall|earlier: int, later: int|
        #![trigger destinations[earlier], destinations[later]]
        0 <= earlier < later < count ==> !runs_overlap(destinations[earlier], destinations[later])
    &&& forall|index: int, key: u64|
        0 <= index < count && #[trigger] mapping.contains_key(key) ==> !runs_overlap(
            #[trigger] destinations[index],
            mapping[key],
        )
}

pub open spec fn relocation_plan_is_well_formed(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
) -> bool {
    &&& 0 <= count <= keys.len()
    &&& count <= sources.len()
    &&& forall|key: u64| #[trigger] mapping.contains_key(key) ==> mapping[key].span >= 1
    &&& forall|earlier: int, later: int|
        #![trigger keys[earlier], keys[later]]
        0 <= earlier < later < count ==> keys[earlier] != keys[later]
    &&& forall|index: int|
        0 <= index < count ==> mapping.contains_key(#[trigger] keys[index]) && mapping[keys[index]]
            == sources[index]
    &&& forall|index: int|
        0 <= index < count ==> #[trigger] sources[index].span >= 1 && destinations[index].span
            == sources[index].span
    &&& destination_runs_are_disjoint_and_free(mapping, destinations, count)
}

// 没被任何目的区间罩住的槽，搬完之后字节不变。
proof fn lemma_slots_outside_every_destination_keep_their_bytes(
    storage: spec_fn(int) -> Seq<u8>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
    slot: int,
)
    requires
        0 <= count <= sources.len(),
        count <= destinations.len(),
        forall|index: int| 0 <= index < count ==> !run_covers(#[trigger] destinations[index], slot),
    ensures
        storage_after_relocating_runs(storage, sources, destinations, count)(slot) == storage(slot),
    decreases count,
{
    if count > 0 {
        lemma_slots_outside_every_destination_keep_their_bytes(
            storage, sources, destinations, count - 1, slot,
        );
    }
}

// 被第 j 个目的区间罩住的槽，搬完之后是源区间对应槽的原字节；前提是目的区间两两不重叠、也不压住任何源区间。
proof fn lemma_a_slot_under_a_destination_holds_the_source_byte(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    storage: spec_fn(int) -> Seq<u8>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
    which: int,
    slot: int,
)
    requires
        relocation_plan_is_well_formed(mapping, keys, sources, destinations, count),
        0 <= which < count,
        run_covers(destinations[which], slot),
    ensures
        storage_after_relocating_runs(storage, sources, destinations, count)(slot)
            == storage(sources[which].start + (slot - destinations[which].start)),
    decreases count,
{
    let last = count - 1;
    let before = storage_after_relocating_runs(storage, sources, destinations, last);
    if which == last {
        // 最后一次拷贝就盖住了这个槽，它读的源槽不在任何更早的目的区间里（源区间在映射里）。
        let source_slot = sources[last].start + (slot - destinations[last].start);
        assert(run_covers(sources[last], source_slot));
        assert forall|index: int| 0 <= index < last implies !run_covers(
            #[trigger] destinations[index],
            source_slot,
        ) by {
            assert(mapping.contains_key(keys[last]));
            assert(mapping[keys[last]] == sources[last]);
            assert(!runs_overlap(destinations[index], mapping[keys[last]]));
        }
        lemma_slots_outside_every_destination_keep_their_bytes(
            storage, sources, destinations, last, source_slot,
        );
    } else {
        // 更早的拷贝盖住了它，最后一次拷贝的目的区间与第 which 个不重叠，不会再动这个槽。
        assert(!runs_overlap(destinations[which], destinations[last]));
        assert(!run_covers(destinations[last], slot));
        lemma_a_slot_under_a_destination_holds_the_source_byte(
            mapping, keys, storage, sources, destinations, last, which, slot,
        );
    }
}

// 两段区间读出的单元相等：跨度相同、逐槽字节相同。
proof fn lemma_unit_reads_agree_slot_by_slot(
    storage_a: spec_fn(int) -> Seq<u8>,
    run_a: SlotRun,
    storage_b: spec_fn(int) -> Seq<u8>,
    run_b: SlotRun,
)
    requires
        run_a.span == run_b.span,
        run_a.span >= 0,
        forall|offset: int| 0 <= offset < run_a.span ==> #[trigger] storage_a(run_a.start + offset) == storage_b(
            run_b.start + offset,
        ),
    ensures
        unit_read(storage_a, run_a) =~= unit_read(storage_b, run_b),
{
    assert forall|offset: int| 0 <= offset < run_a.span implies #[trigger] unit_read(
        storage_a,
        run_a,
    )[offset] == unit_read(storage_b, run_b)[offset] by {
        assert(unit_read(storage_a, run_a)[offset] == storage_a(run_a.start + offset));
        assert(unit_read(storage_b, run_b)[offset] == storage_b(run_b.start + offset));
    }
}

// P4 的区间版：按次序搬前 count 项之后，映射里每个 key 读到的单元（区间里每个槽上的字节）都不变，
// 被搬的 key 指到各自的目的区间，没被搬的 key 的区间原样。
pub proof fn theorem_relocating_runs_in_order_keeps_every_unit_read(
    mapping: Map<u64, SlotRun>,
    storage: spec_fn(int) -> Seq<u8>,
    keys: Seq<u64>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
)
    requires
        relocation_plan_is_well_formed(mapping, keys, sources, destinations, count),
    ensures
        ({
            let moved_mapping = mapping_after_relocating_runs(mapping, keys, destinations, count);
            let moved_storage = storage_after_relocating_runs(storage, sources, destinations, count);
            &&& moved_mapping.dom() == mapping.dom()
            &&& forall|key: u64|
                #[trigger] mapping.contains_key(key) ==> unit_read(moved_storage, moved_mapping[key])
                    =~= unit_read(storage, mapping[key])
            &&& forall|index: int|
                0 <= index < count ==> moved_mapping[#[trigger] keys[index]] == destinations[index]
        }),
{
    lemma_moved_mapping_shape(mapping, keys, sources, destinations, count);
    let moved_mapping = mapping_after_relocating_runs(mapping, keys, destinations, count);
    let moved_storage = storage_after_relocating_runs(storage, sources, destinations, count);
    assert forall|key: u64| #[trigger] mapping.contains_key(key) implies unit_read(
        moved_storage,
        moved_mapping[key],
    ) =~= unit_read(storage, mapping[key]) by {
        if exists|index: int| 0 <= index < count && keys[index] == key {
            let which = choose|index: int| 0 <= index < count && keys[index] == key;
            assert(moved_mapping[key] == destinations[which]);
            assert(mapping[key] == sources[which]);
            assert forall|offset: int| 0 <= offset < destinations[which].span implies #[trigger] moved_storage(
                destinations[which].start + offset,
            ) == storage(sources[which].start + offset) by {
                lemma_a_slot_under_a_destination_holds_the_source_byte(
                    mapping, keys, storage, sources, destinations, count, which,
                    destinations[which].start + offset,
                );
            }
            lemma_unit_reads_agree_slot_by_slot(moved_storage, destinations[which], storage, sources[which]);
        } else {
            assert(moved_mapping[key] == mapping[key]);
            assert forall|offset: int| 0 <= offset < mapping[key].span implies #[trigger] moved_storage(
                mapping[key].start + offset,
            ) == storage(mapping[key].start + offset) by {
                let slot = mapping[key].start + offset;
                assert forall|index: int| 0 <= index < count implies !run_covers(
                    #[trigger] destinations[index],
                    slot,
                ) by {
                    assert(!runs_overlap(destinations[index], mapping[key]));
                }
                lemma_slots_outside_every_destination_keep_their_bytes(
                    storage, sources, destinations, count, slot,
                );
            }
            lemma_unit_reads_agree_slot_by_slot(moved_storage, mapping[key], storage, mapping[key]);
        }
    }
}

// 搬完之后的映射：定义域不变；被搬的 key 指到各自的目的区间；其余 key 原样。
proof fn lemma_moved_mapping_shape(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
)
    requires
        relocation_plan_is_well_formed(mapping, keys, sources, destinations, count),
    ensures
        ({
            let moved = mapping_after_relocating_runs(mapping, keys, destinations, count);
            &&& moved.dom() =~= mapping.dom()
            &&& forall|index: int| 0 <= index < count ==> moved[#[trigger] keys[index]] == destinations[index]
            &&& forall|key: u64|
                #[trigger] mapping.contains_key(key) && (forall|index: int|
                    0 <= index < count ==> keys[index] != key) ==> moved[key] == mapping[key]
        }),
    decreases count,
{
    if count > 0 {
        let last = count - 1;
        lemma_moved_mapping_shape(mapping, keys, sources, destinations, last);
        let earlier = mapping_after_relocating_runs(mapping, keys, destinations, last);
        let moved = mapping_after_relocating_runs(mapping, keys, destinations, count);
        assert(moved == earlier.insert(keys[last], destinations[last]));
        assert(mapping.contains_key(keys[last]));
        assert(moved.dom() =~= mapping.dom());
        assert forall|index: int| 0 <= index < count implies moved[#[trigger] keys[index]]
            == destinations[index] by {
            if index < last {
                assert(keys[index] != keys[last]);
            }
        }
    }
}

// 第一轮攻方的反例在这个前提下被排除：映射 key 5 → 槽 3 的一槽单元，key 9 → 槽 6 起的两槽单元，目的区间 [2, 4) 压住了槽 3，前提不成立。
pub proof fn lemma_the_first_round_two_slot_counterexample_breaks_the_interval_premise()
    ensures
        ({
            let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
                9u64,
                SlotRun { start: 6, span: 2 },
            );
            let destinations = seq![SlotRun { start: 2, span: 2 }];
            !destination_runs_are_disjoint_and_free(mapping, destinations, 1)
        }),
{
    let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
        9u64,
        SlotRun { start: 6, span: 2 },
    );
    let destinations = seq![SlotRun { start: 2, span: 2 }];
    assert(mapping.contains_key(5u64));
    assert(mapping[5u64] == (SlotRun { start: 3, span: 1 }));
    assert(runs_overlap(destinations[0], mapping[5u64]));
}

// 无前提的见证：两个单元（一槽、两槽）、目的区间合格，计划整齐；防止上面的定理在空映射上空真。
pub proof fn witness_a_two_unit_plan_satisfies_the_interval_premise()
    ensures
        ({
            let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
                9u64,
                SlotRun { start: 6, span: 2 },
            );
            let keys = seq![9u64, 5u64];
            let sources = seq![SlotRun { start: 6, span: 2 }, SlotRun { start: 3, span: 1 }];
            let destinations = seq![SlotRun { start: 10, span: 2 }, SlotRun { start: 12, span: 1 }];
            relocation_plan_is_well_formed(mapping, keys, sources, destinations, 2)
        }),
{
    let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
        9u64,
        SlotRun { start: 6, span: 2 },
    );
    let keys = seq![9u64, 5u64];
    let sources = seq![SlotRun { start: 6, span: 2 }, SlotRun { start: 3, span: 1 }];
    let destinations = seq![SlotRun { start: 10, span: 2 }, SlotRun { start: 12, span: 1 }];
    assert(mapping.contains_key(9u64) && mapping.contains_key(5u64));
    assert(mapping[9u64] == sources[0]);
    assert(mapping[5u64] == sources[1]);
    assert forall|index: int, key: u64|
        0 <= index < 2 && #[trigger] mapping.contains_key(key) implies !runs_overlap(
        #[trigger] destinations[index],
        mapping[key],
    ) by {
        if key == 5u64 {
            assert(mapping[key] == (SlotRun { start: 3, span: 1 }));
        } else {
            assert(key == 9u64);
            assert(mapping[key] == (SlotRun { start: 6, span: 2 }));
        }
    }
}

} // verus!
