import re,sys
root=sys.argv[1]
def sub(path, old, new, count=1):
    p=root+'/'+path; s=open(p).read()
    n=s.count(old)
    assert n==count, (path, old[:60], n)
    s=s.replace(old,new); open(p,'w').write(s)
# allocator: occupants accessor
sub('crates/singlefs-core/src/allocator.rs',
"""    pub fn slots_per_region(&self) -> RootRingSlotsPerRegion {""",
"""    pub fn opus_occupants(&self) -> &BTreeMap<RootRingSlot, RootRingOccupant> {
        &self.occupants
    }

    #[must_use]
    pub fn slots_per_region(&self) -> RootRingSlotsPerRegion {""")
# recovery: T2 thread-locals and hook
sub('crates/singlefs-core/src/recovery.rs',
"""pub fn readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once<""",
"""thread_local! {
    /// 攻方原型（m3-batch10o-code-r1 opus）T2「回退不拒」臂：回退那一遍里新类槽记着的根。
    pub static OPUS_T2_RECORDS: std::cell::RefCell<std::collections::BTreeMap<RootRingSlot, RootRecord>> = std::cell::RefCell::new(std::collections::BTreeMap::new());
    pub static OPUS_T2_PUSHES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub fn readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once<""")
sub('crates/singlefs-core/src/recovery.rs',
"""        match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(reread) => {
                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {""",
"""        match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(_) if OPUS_T2_RECORDS.with(|r| r.borrow().contains_key(&ring_slot)) => {
                let recorded = OPUS_T2_RECORDS.with(|r| r.borrow()[&ring_slot]);
                OPUS_T2_PUSHES.with(|c| c.set(c.get() + 1));
                roots.push((ring_slot, recorded));
            }
            RootRingSlotReading::Bad(reread) => {
                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {""")
# mount: counters TL
sub('crates/singlefs-core/src/mount.rs',
"""fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(""",
"""thread_local! {
    /// 攻方原型计数（m3-batch10o-code-r1 opus）：[抬 F 重算那一遍新类仍坏顶上的次数, 其中被抛弃那一半, 回退那一遍新类仍坏顶上（T2 push 臂）, 上限那一遍新类仍坏顶上]。
    pub static OPUS_I1_TL: std::cell::RefCell<[u64; 4]> = const { std::cell::RefCell::new([0; 4]) };
}
fn opus_bump(i: usize) {
    OPUS_I1_TL.with(|c| c.borrow_mut()[i] += 1);
}

fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(""")
sub('crates/singlefs-core/src/mount.rs',
"""                RootRingSlotReading::Bad(_still_bad) => {
                    roots.push(*root_record_self_verified_on_one_rebuild_pass);
                }""",
"""                RootRingSlotReading::Bad(_still_bad) => {
                    opus_bump(3);
                    roots.push(*root_record_self_verified_on_one_rebuild_pass);
                }""")
sub('crates/singlefs-core/src/mount.rs',
"""            RootRingSlotReading::Bad(_still_bad) => roots_with_ring_slots
                .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),""",
"""            RootRingSlotReading::Bad(_still_bad) => {
                opus_bump(0);
                if matches!(
                    allocator.root_ring_occupancy().and_then(|o| o.opus_occupants().get(ring_slot)),
                    Some(crate::allocator::RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                        root: crate::allocator::RootReadOnOneRebuildPass::AbandonedRoot { .. },
                        ..
                    })
                ) {
                    opus_bump(1);
                }
                roots_with_ring_slots.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass))
            }""")
sub('crates/singlefs-core/src/mount.rs',
"""    let current_table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?
        .records;
    let target_root = rollback_candidate(""",
"""    let current_table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?
        .records;
    struct OpusT2Guard;
    impl Drop for OpusT2Guard {
        fn drop(&mut self) {
            let pushes = crate::recovery::OPUS_T2_PUSHES.with(|c| c.replace(0));
            OPUS_I1_TL.with(|c| c.borrow_mut()[2] += pushes);
            crate::recovery::OPUS_T2_RECORDS.with(|r| r.borrow_mut().clear());
        }
    }
    let _opus_t2_guard = OpusT2Guard;
    if std::env::var("OPUS_T2_ARM").as_deref() == Ok("push") {
        let records = ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator);
        crate::recovery::OPUS_T2_RECORDS.with(|r| *r.borrow_mut() = records);
    }
    let target_root = rollback_candidate(""")
print("ok")
