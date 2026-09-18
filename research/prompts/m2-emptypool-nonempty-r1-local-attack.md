Local attack leg prompt, round m2-emptypool-nonempty-r1, attack surface Z6 (mutations and test pinning)

Section 0. Rules for your answer

Write your entire answer in English.
Do not use any markdown emphasis anywhere in your answer: no asterisks, no underscores used as emphasis, no bold, no italics, no markdown headers written with a hash mark. Plain text only. You may use blank lines, numbered lines, and the vertical bar character to lay out rows of a table.
Never write two colon characters next to each other. This document writes the Rust path separator using two at-sign characters, @@, instead of two colon characters. When you write Rust code or refer to a Rust item path in your answer, write it with @@ the same way this document does, never with two colon characters.
Do not use Chinese characters anywhere in your answer.
Number your answers exactly as the numbered questions in section 7 below (Question 1 through Question 6).
Do not answer any grid cell with only "yes" or "no". Every grid cell must contain the specific value requested (a test id, a mutation id, the word None, or the word Covered or Not-covered) plus the one sentence of reasoning requested for that cell.
Base every answer only on the facts given in sections 1 through 6 of this document. Do not assume anything about the wider feature beyond what is written here.

Section 1. What this round is about

This is about a Rust project named singlefs. In this project there is a file named crates/mutations.tsv. Each line of that file (after its header line) describes one source-code mutation: a short label, a source file, a literal piece of original source text, a literal piece of replacement source text, a cargo test filter, and the name of one Rust test function that is required to fail (turn red) when the replacement text is compiled in place of the original text.

This round is about ten specific mutations, on rows 62 through 71 of crates/mutations.tsv. Section 4 gives you the literal content of those ten rows. Section 3 gives you the full source text of every Rust test function named by those ten rows, plus the full source text of every shared helper function or struct those test functions call. Section 6 gives you the full source text of two comparison mechanisms the tests use to check that nothing was written to disk before a refusal.

Your only job is to answer the three grids and three follow-up questions in section 7, using only the facts given in sections 1 through 6.

Section 2. Notation used in this document

Two at-sign characters, @@, stand for the Rust path separator. In real Rust source code that separator is written as two colon characters next to each other. Example: in this document, TransactionUnit@@InstanceTable means the same item that real Rust source would write as TransactionUnit followed by two colon characters followed by InstanceTable.
File paths are written relative to the repository root.
"row 62" and similar phrases refer to the line number in crates/mutations.tsv.
A quoted block that starts with the word CODE and ends with the word ENDCODE is a literal excerpt of Rust source text, copied without changing anything except that every pair of colon characters was replaced by @@ as described above.

Section 3. The nine test functions named by the ten mutations

Table of test ids used throughout this document:

T1 | torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling | crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs | function body at source lines 321-362, doc comment at lines 317-319
T2 | the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it | crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs | function body at source lines 368-391, doc comment at lines 364-366
T3 | root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root | crates/singlefs-core/src/mount.rs, inside a test module named non_empty_root_tests | function body at source lines 1103-1128, doc comment at lines 1100-1101
T4 | writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs | function body at source lines 118-284, doc comment at lines 113-116
T5 | the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs | function body at source lines 120-125, doc comment at line 118
T6 | every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs | function body at source lines 128-166, doc comment at line 126
T7 | writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs | function body at source lines 62-111, doc comment at lines 58-60
T8 | rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write | crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs | function body at source lines 87-110, doc comment at lines 84-85
T9 | raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable | crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs | function body at source lines 427-481, doc comment at lines 423-425

T4 and T7 are in the same source file and both call two shared helper functions defined in that file: region_device and assert_checker_verdicts (their source is given in section 3.2 below).
T5 and T6 are in the same source file and both call a shared helper function named prepare, which itself calls mount_writable and contains an assertion on the shape of the write stream (its source is given in section 3.3 below). T6 also calls a second shared helper named assert_checker_and_record_checker_clean (source given in section 3.3).
T1, T2, and T9 are in the same source file and all call shared helper functions named build_through_rollback, four_overwrites_after_the_rollback, overwrite_in_process, content_of, and raise_floor (source given in section 3.1 below). T9 additionally calls a helper named allocator_state, whose source is given in section 6.

Section 3.1. Shared helpers used by T1, T2, T9 (source file crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs)

CODE
fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8@@try_from((index * 7 + seed) % 253).expect("less than 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("image still open");
    let mut writer = PoolWriter@@new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("overwrite");
    pool.output = output.clone();
    output
}

// build_through_rollback runs a fixed script up to a rollback target named D:
// publish A, publish B, remount (acquire instance 2, write the row, warm up
// twice), publish C, remount and roll back to A (instance 1, checkpoint_txg 3).
fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("writable mount");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("current version has a file after reopening past B");
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger@@On,
    )
    .expect("rollback");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back
        .current
        .into_file_version()
        .expect("current version has a file after rolling back to A");
    pool
}

// Publishes four more overwrites after the rollback (checkpoint_txg 11 through 14).
fn four_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [17usize, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(3))
        })
        .collect()
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("image still open");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger@@On,
    );
    pool.output = current;
    raised
}
ENDCODE

Section 3.2. T1, T2, T9 full source text

T1, source lines 317-362. English translation of the doc comment (lines 317-319): Non-empty is recognized from disk (decision D16 item 1, decided by the user on 2026-09-17): compare the inode-tree and extent-tree root pointers in the tree table, do not look at the journal record. Corrupt the record for the txg-14 overwrite on both devices (the root slot and the units are left untouched); the valid non-empty roots are still 14, 13, 12, 11, the ceiling is still 11, and raising to 12 is refused. Under the reading (recognize non-empty when the ring has that root's own record and its transaction number is non-zero), txg 14 would become an empty root, the fourth-newest non-empty root would drop to A at txg 3, and the ceiling would drop to 3.

CODE
fn torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling() {
    let mut pool = build_through_rollback("step-five-ceiling-torn-record");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    let fourth = &overwrites[3];
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(14), 14)
    );
    {
        let offset = record_offset(
            fourth.record.counter,
            parameters().geometry.journal_ring_bytes,
        );
        let devices = pool.devices.as_mut().expect("image still open");
        for (_, recorded) in devices.iter_mut() {
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("read record");
            bytes[300] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability@@Plain)
                .expect("corrupt record");
        }
    }
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("superblock");
    assert!(
        !scan_journal(&image, &superblock).contains_key(&(InstanceGeneration(3), 14)),
        "both copies of the txg 14 record are unreadable"
    );
    let refused = raise_floor(&mut pool, CheckpointTxg(12));
    assert!(
        matches!(
            refused,
            Err(MountError@@RollbackFloorAboveCeiling {
                requested: CheckpointTxg(12),
                ceiling: CheckpointTxg(11)
            })
        ),
        "record for txg 14 is corrupt, its tree table still differs from txg 13's: ceiling is still 11: {:?}",
        refused.as_ref().err()
    );
}
ENDCODE

Note: T1 does not call disk_snapshot and does not compare any before/after state; its only assertions are the two shown above (the assert_eq on (checkpoint_txg, counter), the assert on scan_journal not containing the key, and the final assert on the matches! pattern for refused).

T2, source lines 368-391. English translation of the doc comment (lines 364-366): The previous root is looked up only among valid roots: the rollback publish D (txg 9) copies A's file, so the two tree-root pointers in D's tree table equal those of the previous valid root A (txg 3), hence D counts as empty. The abandoned root C (txg 8), which sits between A and D in txg order, carries the third overwrite's content; comparing D against C instead would make D count as non-empty. After the rollback, three more overwrites are published (txg 11, 12, 13): the non-empty valid roots are 13, 12, 11, 3; the fourth-newest is 3; the newest valid root on each device is 12 (device 0) and 13 (device 1); the ceiling is min(12, 3) = 3; raising to 4 is refused.

CODE
fn the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it(
) {
    let mut pool = build_through_rollback("step-five-ceiling-previous-valid-root");
    for seed in [17usize, 19, 23] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(3),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    let refused = raise_floor(&mut pool, CheckpointTxg(4));
    assert!(
        matches!(
            refused,
            Err(MountError@@RollbackFloorAboveCeiling {
                requested: CheckpointTxg(4),
                ceiling: CheckpointTxg(3)
            })
        ),
        "D does not count as non-empty, the fourth-newest non-empty valid root is A: {:?}",
        refused.as_ref().err()
    );
}
ENDCODE

Note: T2 does not call disk_snapshot and does not compare any before/after state; its only assertions are the assert_eq on pool.output.root.checkpoint_txg and the final assert on the matches! pattern for refused.

T9, source lines 427-481. English translation of the doc comment (lines 423-425): When computing the ceiling, one valid root's tree table cannot be read: raising F is refused instead of guessing empty or non-empty (per the user: an unreadable tree table must go through repair, it may not be skipped or guessed). After the rollback, overwrite four more times, corrupt the tree table of the valid root at txg 12 on both devices, then raise F to 11; this returns RollbackFloorCeilingNeedsUnreadableValidRootTreeTable naming (3, 12). The allocator, the current version, and the recorded-operation stream are all unchanged (nothing is reclaimed, nothing is published). Under the reading (guess empty when unreadable), txg 12 and 13 would both differ from their previous root, the ceiling would still be 11, and raising F would succeed.

CODE
fn raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable() {
    let mut pool = build_through_rollback("step-five-ceiling-unreadable-valid-root");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    let damaged = &overwrites[1];
    assert_eq!(damaged.root.checkpoint_txg, CheckpointTxg(12));
    {
        let devices = pool.devices.as_mut().expect("image still open");
        for location in &damaged.root.tree_table.locations {
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("the device holding txg 12's tree table");
            let offset = location.slot.to_device_offset();
            let mut bytes = vec![0u8; usize@@try_from(singlefs_format@@NODE_BYTES).expect("16384")];
            recorded
                .read_at(offset, &mut bytes)
                .expect("read txg 12's tree table");
            bytes[200] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability@@Plain)
                .expect("corrupt txg 12's tree table");
        }
    }
    let allocator_before = allocator_state(&pool);
    let current_before = pool.output.clone();
    let recorded_before = pool.stream.operations().len();
    let refused = raise_floor(&mut pool, CheckpointTxg(11));
    assert!(
        matches!(
            refused,
            Err(
                MountError@@RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                    root: RollbackTarget {
                        instance: InstanceGeneration(3),
                        checkpoint_txg: CheckpointTxg(12)
                    },
                    ..
                }
            )
        ),
        "valid root at txg 12 has an unreadable tree table: {:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        allocator_state(&pool),
        allocator_before,
        "nothing reclaimed, nothing isolated"
    );
    assert_eq!(pool.output, current_before, "current version did not move");
    assert_eq!(
        pool.stream.operations().len(),
        recorded_before,
        "not a single write, not a single barrier was issued"
    );
}
ENDCODE

Note: the bytes[200] ^= 0xff corruption of the txg-12 tree table happens before allocator_before, current_before and recorded_before are captured; it is test setup, not part of the code path under test. The before/after comparison window for this test is only around the raise_floor(&mut pool, CheckpointTxg(11)) call. T9 does not call disk_snapshot at all; it does not compare superblock_slots or readable_roots.

Section 3.3. T3 full source text (crates/singlefs-core/src/mount.rs, lines 1088-1128, a #[cfg(test)] module inside the same file as the mutated function)

English translation of the doc comment (lines 1100-1101): If either tree's root pointer changed, it counts as non-empty: a publish that only changes the extent tree (an in-place content overwrite of the same inode, where the inode record itself does not move) and a publish that only changes the inode tree must both be recognized. A publish where neither tree changed (a zero-unit publish, or a rollback publish that copies the previous version) does not count. The oldest valid root, which has no previous root, is compared against a value meaning the tree table contains neither tree.

CODE
#[cfg(test)]
mod non_empty_root_tests {
    use super@@user_visible_trees_changed;
    use crate@@recovery@@UserVisibleTreeRootPointers;

    fn pointers(inode_tree: u8, extent_tree: u8) -> UserVisibleTreeRootPointers {
        UserVisibleTreeRootPointers {
            inode_tree: Some(vec![inode_tree; 86]),
            extent_tree: Some(vec![extent_tree; 86]),
        }
    }

    fn root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root(
    ) {
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &pointers(1, 3)),
            "only the extent tree root pointer changed"
        );
        assert!(
            user_visible_trees_changed(&pointers(4, 2), &pointers(1, 2)),
            "only the inode tree root pointer changed"
        );
        assert!(
            !user_visible_trees_changed(&pointers(1, 2), &pointers(1, 2)),
            "both trees copied unchanged: a zero-unit publish"
        );
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &UserVisibleTreeRootPointers@@ABSENT),
            "the oldest valid root has a file below it"
        );
        assert!(
            !user_visible_trees_changed(
                &UserVisibleTreeRootPointers@@ABSENT,
                &UserVisibleTreeRootPointers@@ABSENT
            ),
            "mkfs's genesis root and the warm-up root: the tree table contains neither tree"
        );
    }
}
ENDCODE

The function under test, user_visible_trees_changed, is defined a few lines above this module in the same file (crates/singlefs-core/src/mount.rs, around line 430):

CODE
pub fn user_visible_trees_changed(
    root_pointers: &UserVisibleTreeRootPointers,
    previous_valid_root_pointers: &UserVisibleTreeRootPointers,
) -> bool {
    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
}
ENDCODE

Mutation M3 (section 4 below) replaces the body of this exact function.

Section 3.4. Shared helpers used by T4 and T7 (source file crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs, lines 25-56)

CODE
fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize@@try_from(target.region).expect("region number")]
}

const NOT_APPLICABLE_WITHOUT_FILE: [&str; 8] = [
    "I-3.1", "I-5.2", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13",
];

fn assert_checker_verdicts(pool: &FormattedPool, step: &str, not_applicable: &[&str]) {
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        verdicts.len(),
        singlefs_checker@@image@@IMPLEMENTED_INVARIANTS.len()
    );
    for (invariant, verdict) in &verdicts {
        if not_applicable.contains(invariant) {
            assert!(
                matches!(verdict, InvariantVerdict@@NotApplicable(_)),
                "{step}: {invariant} should report not applicable: {verdict:?}"
            );
        } else {
            assert_eq!(
                *verdict,
                InvariantVerdict@@Holds,
                "{step}: {invariant} must have actually been evaluated and hold"
            );
        }
    }
}
ENDCODE

Section 3.5. T7 full source text (crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs, lines 58-111)

English translation of the doc comment (lines 58-60): Writing instance-table rows on a version whose tree table is empty is unsupported (undecided by design), refused before acquiring an instance: crash after the first transaction has acquired its instance and warmed up twice but before the first file version (in the same process, stop right after warm-up and close the image), then remount writable. The chosen root is (1, 2), its tree table is empty, the instance number that would be acquired is 2, and the row range to write [1, 2) is non-empty, so this returns InstanceRowsOnVersionWithoutFileUnsupported. All four superblock slots across both devices stay byte-identical (the instance number is still 1), the root ring gets no new root, and the recorded-operation stream does not gain a single step.

CODE
fn writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance(
) {
    let mut formatted = format_pool("step-three-formatted-crash-after-warm-up");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("image still open");
        let mut writer = PoolWriter@@new(&publish_parameters, open_devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("acquire instance");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up(&mut writer, &formatted.genesis.root, instance).expect("warm up");
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .max(),
        Some((InstanceGeneration(1), CheckpointTxg(2))),
        "crash point: both roots from the two warm-up publishes are on disk"
    );
    let mut devices = formatted.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError@@InstanceRowsOnVersionWithoutFileUnsupported {
                chosen_root: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(2)
                },
                first_row_instance: InstanceGeneration(1),
                instance_to_acquire: InstanceGeneration(2),
            })
        ),
        "the version whose tree table has 0 entries would need to write (1, 2, 0): {:?}",
        refused.as_ref().err()
    );
    let after = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        after.superblock_slots, before.superblock_slots,
        "both devices' superblock slots stay byte-identical: no instance was acquired"
    );
    assert_eq!(after.readable_roots, before.readable_roots, "the root ring gets no new root");
    assert_eq!(
        after.recorded_operations, before.recorded_operations,
        "not a single write, not a single barrier was issued"
    );
}
ENDCODE

Note: T7 calls disk_snapshot twice (before and after) and compares the three DiskSnapshot fields with three separate assert_eq! calls, not with one assert_eq! on the whole struct.

Section 3.6. T4 full source text (crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs, lines 118-284)

English translation of the doc comment (lines 113-116): Acceptance: instance number acquired is 1, no rows written; the row-writing publish is (1, 1), journal sequence number 1, transaction number 0, back_chain 0, zero units, lands on device 1; one warm-up publish (1, 2), journal sequence number 2, back-chain links to journal sequence number 1, lands on device 0; the allocator only has mkfs's two units (3 slots per device); the first file version is (1, 3), journal sequence number 3, transaction number 1, the data unit lands on slot 50180, replacing the tree table mkfs wrote; the recorded operation stream after mkfs is byte-identical to running the first transaction in the same process right after mkfs, and the published version is also identical; a cold restart reads the file back; the checker reports zero violations: after mkfs and after mounting, 18 invariants hold and the 8 invariants that need an accounting tree and an inode tree report not-applicable; after the first file version, all 26 hold.

CODE
fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
) {
    let mut formatted = format_pool("step-three-formatted-pool");
    assert_checker_verdicts(&formatted, "after mkfs", &NOT_APPLICABLE_WITHOUT_FILE);

    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("writable mount of a pool that has only been through mkfs");
    formatted.devices = Some(devices);
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(1),
        "acquired instance = max(superblock 0, root ring 0) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(0), CheckpointTxg(0)),
        "the chosen root is mkfs's generation 0 root"
    );
    assert_eq!(output.effective_root, output.chosen_root);
    assert_eq!(output.journal.valid_records, 0, "the ring has zero records");
    assert!(output.rows_written.is_empty(), "previous instance is 0: no rows are written");
    let PoolVersion@@WithoutFile(row) = &output.row_publish else {
        panic!("tree table has 0 entries: the row-writing publish writes zero units")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (InstanceGeneration(1), CheckpointTxg(1), 1, 0, 0, 0),
        "txg = max(root ring 0, record none) + 1; journal sequence number starts at 1; transaction number 0; this instance's first back_chain is 0; no names"
    );
    assert_eq!(
        (
            row.root.tree_table,
            row.root.instance_table,
            row.root.mapping_root
        ),
        (
            formatted.genesis.root.tree_table,
            formatted.genesis.root.instance_table,
            formatted.genesis.root.mapping_root
        ),
        "the root record follows mkfs's root: tree table, instance table, mapping tree root are all unchanged"
    );
    assert_eq!(region_device(1), DeviceIdentity(1), "txg 1 lands on device 1");
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "txg 2 lands on device 0, one publish covers both devices"
    );
    let PoolVersion@@WithoutFile(warm_up) = &output.warm_up_publishes[0] else {
        panic!("tree table has 0 entries: the warm-up publish writes zero units")
    };
    assert_eq!(
        (
            warm_up.root.checkpoint_txg,
            warm_up.record.counter,
            warm_up.record.transaction,
            warm_up.record.back_chain
        ),
        (CheckpointTxg(2), 2, 0, back_chain_of(&row.record_bytes))
    );
    assert_eq!(region_device(2), DeviceIdentity(0), "txg 2 lands on device 0");
    assert_eq!(
        mounted.current, output.warm_up_publishes[0],
        "the next publish continues after txg 2"
    );
    assert_eq!(
        output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "no abandoned root"
    );
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (3, 0),
            "only mkfs's instance table (2 slots) and tree table (1 slot)"
        );
    }
    assert_checker_verdicts(&formatted, "after writable mount", &NOT_APPLICABLE_WITHOUT_FILE);
    assert_eq!(
        recover(&formatted.memory_pool(), JournalPolicy@@Consult).outcome,
        RecoveryOutcome@@NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );

    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("image still open");
        let mut writer = PoolWriter@@new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            current.record_bytes(),
        )
        .expect("first file version")
    };
    assert_eq!(
        (
            first.root.instance,
            first.root.checkpoint_txg,
            first.record.counter,
            first.record.transaction,
            first.record.back_chain
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(3),
            3,
            1,
            back_chain_of(current.record_bytes())
        )
    );
    assert_eq!(
        first.data_pointer.locations[0].slot,
        SlotNumber(50180),
        "mkfs's two units occupy 50176 through 50178"
    );
    assert_eq!(
        first.released,
        vec![Placement {
            slot: SlotNumber(50178),
            span: 1
        }],
        "replaces mkfs's tree table"
    );

    let reference = build_pool("step-three-formatted-pool-reference");
    assert_eq!(
        formatted.retained_operations()[formatted.mkfs_operation_count..],
        reference.retained_operations()[reference.mkfs_operation_count..],
        "the recorded operation stream after mkfs is byte-identical to running the first transaction in the same process right after mkfs"
    );
    assert_eq!(first, reference.output, "the published version is also identical");

    assert_checker_verdicts(&formatted, "after the first file version", &[]);
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy@@Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome@@FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content
        }
    );
    assert_eq!(report.journal.valid_records, 3);
}
ENDCODE

Note: T4 does not call disk_snapshot. It compares state through the returned output values (output.instance, output.chosen_root, output.warm_up_publishes, mounted.allocator.devices, and so on), through assert_checker_verdicts (the checker's verdicts on the in-memory image), through recover(...) outcomes, and through one byte-for-byte comparison of formatted.retained_operations() against a freshly built reference pool's retained_operations(), both sliced from mkfs_operation_count onward.

Section 3.7. Shared helpers used by T5 and T6 (source file crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs, lines 18-116)

English translation of the assertion message at line 57 (a literal Rust string inside the code below, translated here because it names the shape of every one of the ten write segments): acquiring the instance is two writes, then the row-writing publish's record is two writes, then a root, then the superblock is two writes, then the warm-up's record is two writes, then a root, then the superblock is two writes together with the first file version's sixteen unit writes, then a record is two writes, then a root, then the superblock is two writes.

CODE
struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    // judged_root_index: the root-slot force-unit-access write that is judged: the first file version's root.
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn prepare(tag: &str) -> Prepared {
    let mut formatted = format_pool(tag);
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("writable mount of a pool that has only been through mkfs");
    let mut allocator = mounted.allocator;
    let content = file_content();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter@@new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            mounted.current.record_bytes(),
        )
        .expect("first file version");
    }
    formatted.devices = Some(devices);
    let base = formatted.memory_pool_after_mkfs();
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec@@len).collect@@<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 2],
        "acquiring the instance is two writes, then the row-writing record is two writes, then a root, then the superblock is two writes, then the warm-up record is two writes, then a root, then the superblock is two writes together with the first file version's sixteen unit writes, then a record is two writes, then a root, then the superblock is two writes"
    );
    assert_eq!(
        writes.len(),
        33,
        "acquiring the instance is 2, plus two zero-unit publishes at 5 each, plus the file version at 21"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind@@RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 3, "txg 1, 2, 3 each contribute one root-slot write");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("three root-slot writes"),
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content,
        }],
    }
}

fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "the record checker's two criteria are always 0 under the decided persistence order"
    );
    for invariant in singlefs_checker@@image@@IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant}: the number of states judged as a violation must be 0: {:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for must_evaluate in ["I-3.1", "I-5.2", "I-5.1", "I-7.2", "I-7.7"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} must have actually been evaluated in at least one state"
        );
    }
}
ENDCODE

Important fact: both T5 and T6 call prepare(tag), and the assert_eq! on segments.iter().map(Vec@@len) shown above (checking the shape [2, 2, 1, 2, 2, 1, 18, 2, 1, 2]) runs inside prepare(), before either T5 or T6 reaches its own test-specific assertions. If this assert_eq! inside prepare() fails, it fails for whichever of T5 or T6 called it, since both call the same prepare() function with no test-specific variant of it.

Section 3.8. T5 and T6 full source text (crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs, lines 118-166)

English translation of T5's doc comment (line 118): Segment sequence: ten segments, closed-form count 262165 (1 + six 2-write segments each contributing 3 + three 1-write segments each contributing 1 + one 18-write segment contributing 262143), identical to the stream from running the first transaction in the same process right after mkfs.

CODE
fn the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence() {
    let prepared = prepare("formatted-layer0-registered");
    assert_eq!(prepared.segments.len(), 10);
    assert_eq!(closed_form_state_count(&prepared.segments), 262_165);
}
ENDCODE

English translation of T6's doc comment (line 126): The version that runs normally: the 18-write segment is not expanded (it enters later states only as a whole, either fully persisted or not at all); every other segment is expanded over any subset.

CODE
fn every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims(
) {
    let prepared = prepare("formatted-layer0-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "expanded segments follow the closed form"
    );
    assert_eq!(tally.states, 22, "1 + six 2-write segments each contributing 3 + three 1-write segments each contributing 1");
    assert_eq!(
        tally.violations, 0,
        "first violation: {:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "the recovery pass that ignores the journal also passes the oracle: {:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_checker_and_record_checker_clean(&tally);
}
ENDCODE

Note: neither T5 nor T6 calls disk_snapshot or DiskSnapshot; they do not compare a before state to an after state at all. T5's own assertions are only the two assert_eq! calls shown (segments.len() and closed_form_state_count). T6's own assertions are the ones shown above, on the Layer0Tally returned by enumerate_layer0_selecting_versions, plus whatever prepare() itself asserts before returning.

Section 3.9. T8 full source text (crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs, lines 84-110)

English translation of the doc comment (lines 84-85): Rolling back to a root whose tree table is empty (the txg-2 warm-up root from the first transaction) is unsupported (undecided by design), refused before any write: after the first transaction, exit the process, remount and roll back to (1, 2), which returns RollbackToVersionWithoutFileUnsupported; both devices' superblock slots stay byte-identical, the root ring gets no new root, and the recorded-operation stream does not gain a single step.

CODE
fn rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write() {
    let mut pool = build_pool("step-four-rollback-to-warm-up-root");
    let warm_up_root = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut devices = pool.reopen_recorded();
    let refused = mount_rollback(&parameters(), &mut devices, warm_up_root, ShadowLedger@@On);
    pool.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError@@RollbackToVersionWithoutFileUnsupported(target)) if target == warm_up_root
        ),
        "the warm-up root has no file version below it: {:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "byte-identical on disk"
    );
}
ENDCODE

Note: T8 compares disk_snapshot(...) to before with one single assert_eq! on the whole DiskSnapshot struct (using its PartialEq derive), not with three separate field comparisons the way T7 does.

Section 4. The ten mutations (crates/mutations.tsv, rows 62-71)

Each mutation below names the file it is in, gives the surrounding function it sits inside (so you can see what runs before and after the exact lines that change), then gives the literal original text and literal replacement text exactly as recorded in crates/mutations.tsv, the cargo test filter, and the test id (from section 3) that crates/mutations.tsv names as the test that must turn red.

First, five context functions that the ten mutations sit inside. All five are in crates/singlefs-core/src/mount.rs except publish_without_units and publish_empty_after, which are in crates/singlefs-core/src/transaction.rs.

Context function 1: rollback_floor_ceiling (crates/singlefs-core/src/mount.rs, around lines 438-519). This function is called only while computing the ceiling for raising F; it only reads devices, it performs no writes. M1, M2, M3, and M10 (defined below) each change one part of this function or a function it calls.

CODE
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    superblock: &crate@@superblock@@Superblock,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
) -> Result<CheckpointTxg, MountError> {
    let readable = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let valid: Vec<RootRecord> = readable
        .iter()
        .copied()
        .filter(|root| root.checkpoint_txg >= current_floor)
        .filter(|root| !abandoned_by_table(root, table))
        .collect();
    let mut newest_per_device: std@@collections@@BTreeMap<DeviceIdentity, CheckpointTxg> =
        std@@collections@@BTreeMap@@new();
    for root in &valid {
        let device = superblock.region_devices
            [usize@@try_from(target_for_publish(root.checkpoint_txg).region).expect("region number")];
        let newest = newest_per_device
            .entry(device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_on_every_device = newest_per_device
        .values()
        .copied()
        .min()
        .ok_or(RecoveryFailure@@NoValidRoot)?;
    // this is what mutation M10 (defined below) changes: if the tree table cannot be read or cannot be parsed, refuse instead of guessing:
    // it cannot be decided whether this root is non-empty or what the next root should be compared against
    // (only comparing tree table root pointers is decided).
    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(devices, root) {
        Ok(pointers) => Ok(pointers),
        Err(failure) => Err(
            MountError@@RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
                failure,
            },
        ),
    };
    // "the previous one" is only looked up among valid roots: an abandoned-timeline root and a root below F do not count.
    let previous_candidates: &[RootRecord] = &valid;
    let mut non_empty: Vec<CheckpointTxg> = Vec@@new();
    for root in &valid {
        let previous_valid_root = previous_candidates
            .iter()
            .filter(|candidate| {
                (candidate.checkpoint_txg, candidate.instance)
                    < (root.checkpoint_txg, root.instance)
            })
            .max_by_key(|candidate| (candidate.checkpoint_txg, candidate.instance));
        let previous_pointers = match previous_valid_root {
            Some(previous) => pointers_of(previous)?,
            None => UserVisibleTreeRootPointers@@ABSENT,
        };
        let root_pointers = pointers_of(root)?;
        if user_visible_trees_changed(&root_pointers, &previous_pointers) {
            non_empty.push(root.checkpoint_txg);
        }
    }
    Ok(
        ceiling_from_newest_and_non_empty_roots(newest_on_every_device, non_empty, &valid)
            .ok_or(RecoveryFailure@@NoValidRoot)?,
    )
}
ENDCODE

Context function 2: format_time_allocator (crates/singlefs-core/src/mount.rs, around lines 385-425). This function builds the allocator for a version whose tree table has 0 entries, right after mounting. It marks the two units mkfs wrote (the instance table and the tree table) as allocated, so a later publish does not hand out those same slots again. M4 and M7 (defined below) each change one line inside it.

CODE
fn format_time_allocator(
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    if root.instance_table.head.birth_txg != CheckpointTxg(0)
        || root.tree_table.head.birth_txg != CheckpointTxg(0)
    {
        return Err(MountError@@VersionWithoutFileNotWrittenByMakeFilesystem {
            root: RollbackTarget {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
            },
            instance_table_birth_txg: root.instance_table.head.birth_txg,
            tree_table_birth_txg: root.tree_table.head.birth_txg,
        });
    }
    let placement_of = |pointer: &crate@@pointer@@NodePointer, identity: TransactionUnit| {
        assert_eq!(
            pointer.locations[0].slot, pointer.locations[1].slot,
            "same slot on both devices"
        );
        Placement {
            slot: pointer.locations[0].slot,
            span: identity.span_slots(),
        }
    };
    let mut allocator = PoolAllocator@@new(device_maps);
    allocator.mark_format_time_units(
        placement_of(&root.instance_table, TransactionUnit@@InstanceTable),
        placement_of(&root.tree_table, TransactionUnit@@TreeTable),
    );
    Ok(allocator)
}
ENDCODE

Fact confirmed by the T4 test itself (section 3.6 above): TransactionUnit@@InstanceTable's span is 2 slots per device, TransactionUnit@@TreeTable's span is 1 slot per device (T4 asserts allocated_slots() is 3 per device after mkfs: 2 for the instance table plus 1 for the tree table).
Context function 3: publish_without_units (crates/singlefs-core/src/transaction.rs, around lines 392-433). This is the zero-unit publish used both for warming up a pool that has only been through mkfs and for warming up any pool whose current version has an empty tree table. M6 (defined below) removes one of the two Barrier steps inside it.

CODE
pub fn publish_without_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> Result<ZeroUnitPublishOutput, BlockDeviceError> {
    pool.perform(CommitStep@@Barrier)?;
    let record = JournalRecord {
        instance: plan.instance,
        counter: plan.counter,
        checkpoint_txg: plan.txg,
        transaction: 0,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
        new_tree_table: previous_root.tree_table,
        new_mapping_root: previous_root.mapping_root,
        new_tree_identifier_watermark: previous_root.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named: Vec@@new(),
    };
    let record_bytes = record.to_bytes();
    pool.perform(CommitStep@@WriteJournalRecordToEveryDevice {
        counter: plan.counter,
        record: &record_bytes,
    })?;
    pool.perform(CommitStep@@Barrier)?;
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance: plan.instance,
        checkpoint_txg: plan.txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: previous_root.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: previous_root.instance_table,
        mapping_root: previous_root.mapping_root,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());
    pool.perform(CommitStep@@WriteRootRecordForceUnitAccess {
        checkpoint_txg: plan.txg,
        root_slot: &root_slot,
    })?;
    pool.perform(CommitStep@@RotateSuperblockSlots {
        journal_tail: plan.counter,
        journal_instance: plan.instance,
    })?;
    // ... (the rest of this function builds and returns ZeroUnitPublishOutput; not shown, not relevant to any of the ten mutations)
}
ENDCODE

Context function 4: publish_empty_after (crates/singlefs-core/src/transaction.rs, around lines 699-737). This is the function that decides, for each zero-unit publish (writing the row, and each warm-up publish), what back_chain value to pass into ZeroUnitPublishPlan. M5 (defined below) changes the WithoutFile branch's back_chain value.

CODE
fn publish_empty_after<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &PoolVersion,
    instance: InstanceGeneration,
) -> Result<PoolVersion, PublishError> {
    match current {
        PoolVersion@@WithFile(current_file_version) => publish_version(
            pool,
            allocator,
            PublishPlan {
                txg: CheckpointTxg(current_file_version.root.checkpoint_txg.0 + 1),
                counter: current_file_version.record.counter + 1,
                transaction: 0,
                instance,
                back_chain: back_chain_of(&current_file_version.record_bytes),
                file: None,
                instance_table: InstanceTablePlan@@Carry(current_file_version.root.instance_table),
                tree_birth_txg: current_file_version.tree_birth_txg(),
                tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
                rollback_floor: current_file_version.root.rollback_floor,
            },
            Some(current_file_version),
        )
        .map(PoolVersion@@WithFile),
        PoolVersion@@WithoutFile(current_version_without_file) => publish_without_units(
            pool,
            &current_version_without_file.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(current_version_without_file.root.checkpoint_txg.0 + 1),
                counter: current_version_without_file.record.counter + 1,
                instance,
                back_chain: back_chain_of(&current_version_without_file.record_bytes),
                rollback_floor: current_version_without_file.root.rollback_floor,
            },
        )
        .map(PoolVersion@@WithoutFile)
        .map_err(PublishError@@from),
    }
}
ENDCODE

Context function 5a: establish_instance (crates/singlefs-core/src/mount.rs, around lines 764-780, only the beginning shown). This is the shared second half of both a normal writable mount and a rollback mount: acquire the instance number, then write instance-table rows. instance_generation_to_acquire only reads devices; it performs no writes. M8 (defined below) swaps the order of the two lines shown last.

CODE
fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    let mut pool = PoolWriter@@new(parameters, devices.as_mut_slice());
    let instance_to_acquire = instance_generation_to_acquire(&pool);
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;
    let instance = acquire_instance(&mut pool).map_err(MountError@@Acquisition)?;
    // ... (row-writing continues below; not shown, not relevant to any of the ten mutations)
}
ENDCODE

Fact: acquire_instance (crates/singlefs-core/src/transaction.rs, shown earlier in this document's background is not repeated here, but its behavior is stated as a fact) writes one superblock slot on each device in turn using CommitStep@@RotateSuperblockSlots (through write_superblock_slot), then performs one CommitStep@@Barrier. If any device's write or the barrier fails partway, it rolls the already-written devices back to the previous instance number. If it succeeds, both devices' superblock slots now hold the new (incremented) instance number.

Context function 5b: the relevant part of mount_rollback (crates/singlefs-core/src/mount.rs, around lines 966-1017, only the part before and including the check M9 changes is shown; the rest of the function, which performs writes, is cut off after the check). Every call shown before the check only reads devices.

CODE
pub fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let newest_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure@@NoValidRoot)?;
    let records = scan_journal(&*devices, &superblock);
    let roots = readable_roots(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let target_root = roots
        .iter()
        .find(|root| {
            root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
        })
        .copied()
        .ok_or(MountError@@RollbackTargetNotInRing(target))?;
    let newest_table = instance_table_of_root(&*devices, &newest_root)
        .ok_or(MountError@@InstanceTableMalformed)?;
    let effective_floor = effective_rollback_floor(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    if target.checkpoint_txg < effective_floor {
        return Err(MountError@@RollbackTargetNotACandidate {
            target,
            reason: "txg is below the effective rollback floor F",
        });
    }
    if newest_table
        .rows
        .iter()
        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)
    {
        return Err(MountError@@RollbackTargetNotACandidate {
            target,
            reason: "the row for that instance in the instance table has a smaller T: this is an abandoned-timeline root",
        });
    }
    if tree_table_has_no_entries(&*devices, &target_root)? {
        return Err(MountError@@RollbackToVersionWithoutFileUnsupported(target));
    }
    // ... (past this point the function performs writes; cut off here, not shown, not relevant to any of the ten mutations)
}
ENDCODE
Now the ten mutations. Each one replaces the literal original text shown with the literal replacement text shown, inside the file named, and crates/mutations.tsv names exactly one test that must turn red when that one replacement (and no other) is applied.

Mutation M1, tsv row 62.
Label: fall back the non-empty test to the journal ring's transaction number (if the record for txg 14 is corrupt on both devices, this reading treats the root as empty and the ceiling drops to 3).
File: crates/singlefs-core/src/mount.rs, inside context function 1 (rollback_floor_ceiling).
Original text:
CODE
        if user_visible_trees_changed(&root_pointers, &previous_pointers) {
ENDCODE
Replacement text:
CODE
        if scan_journal(devices, superblock).values().any(|record| {
            record.instance == root.instance
                && record.checkpoint_txg == root.checkpoint_txg
                && record.transaction != 0
        }) {
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_five_reuse -- torn_journal_record_does_not_turn
Named test that must turn red: T1 (torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling)

Mutation M2, tsv row 63.
Label: the previous-root lookup for non-empty takes any readable root, not just valid ones (does not exclude abandoned-timeline roots or roots below F).
File: crates/singlefs-core/src/mount.rs, inside context function 1 (rollback_floor_ceiling).
Original text:
CODE
    let previous_candidates: &[RootRecord] = &valid;
ENDCODE
Replacement text:
CODE
    let previous_candidates: &[RootRecord] = &readable;
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_five_reuse -- the_rollback_publish_is_compared
Named test that must turn red: T2 (the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it)

Mutation M3, tsv row 64.
Label: the non-empty test compares only the inode-tree root pointer, not the extent-tree root pointer.
File: crates/singlefs-core/src/mount.rs, this is the body of user_visible_trees_changed itself, shown in section 3.3 above (the function T3's unit test calls directly).
Original text:
CODE
    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
ENDCODE
Replacement text:
CODE
    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
ENDCODE
Cargo test filter: -p singlefs-core --lib -- root_is_non_empty_when_either
Named test that must turn red: T3 (root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root)

Mutation M4, tsv row 65.
Label: after remounting a pool that has only been through mkfs, the allocator does not mark mkfs's two units in the unit region as allocated.
File: crates/singlefs-core/src/mount.rs, inside context function 2 (format_time_allocator).
Original text:
CODE
    allocator.mark_format_time_units(
        placement_of(&root.instance_table, TransactionUnit@@InstanceTable),
        placement_of(&root.tree_table, TransactionUnit@@TreeTable),
    );
ENDCODE
Replacement text:
CODE
    let _ = (
        placement_of(&root.instance_table, TransactionUnit@@InstanceTable),
        placement_of(&root.tree_table, TransactionUnit@@TreeTable),
    );
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_three_formatted_pool
Named test that must turn red: T4 (writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold)

Mutation M5, tsv row 66.
Label: on a pool that has only been through mkfs, the zero-unit warm-up publish writes back_chain as 0 instead of computing it from the previous record.
File: crates/singlefs-core/src/mount.rs. Note: this exact line of source text is inside context function 4 (publish_empty_after), which crates/mutations.tsv records under the path crates/singlefs-core/src/mount.rs in this row; the file column in the tsv is what is authoritative.
Original text:
CODE
                back_chain: back_chain_of(&current_version_without_file.record_bytes),
ENDCODE
Replacement text:
CODE
                back_chain: 0,
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_three_formatted_pool
Named test that must turn red: T4 (writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold)

Mutation M6, tsv row 67.
Label: the zero-unit publish is missing one barrier between the journal record write and the root write.
File: crates/singlefs-core/src/transaction.rs, inside context function 3 (publish_without_units).
Original text:
CODE
    pool.perform(CommitStep@@Barrier)?;
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
ENDCODE
Replacement text:
CODE
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream
Named test that must turn red: T5 (the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence)

Mutation M7, tsv row 68.
Label: after remounting a pool that has only been through mkfs, the mkfs instance table's placement is computed using the tree table's span (1 slot) instead of the instance table's own span (2 slots).
File: crates/singlefs-core/src/mount.rs, inside context function 2 (format_time_allocator). This changes the second argument of the first placement_of call only (the call that computes the instance table's placement); the second placement_of call (for the tree table) is unchanged.
Original text:
CODE
        placement_of(&root.instance_table, TransactionUnit@@InstanceTable),
ENDCODE
Replacement text:
CODE
        placement_of(&root.instance_table, TransactionUnit@@TreeTable),
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool
Named test that must turn red: T6 (every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims)

Mutation M8, tsv row 69.
Label: when a version whose tree table is empty needs to write instance-table rows, the refusal is moved to after acquiring the instance (it now returns only after the superblock has already been written with the new instance number).
File: crates/singlefs-core/src/mount.rs, inside context function 5a (establish_instance). This swaps the order of the two lines shown.
Original text:
CODE
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;
    let instance = acquire_instance(&mut pool).map_err(MountError@@Acquisition)?;
ENDCODE
Replacement text:
CODE
    let instance = acquire_instance(&mut pool).map_err(MountError@@Acquisition)?;
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_between_warm_up
Named test that must turn red: T7 (writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance)

Mutation M9, tsv row 70.
Label: rolling back to a root whose tree table is empty is no longer refused before any write (the check is replaced by a condition that is always false, so the refusal branch can never run).
File: crates/singlefs-core/src/mount.rs, inside context function 5b (mount_rollback).
Original text:
CODE
    if tree_table_has_no_entries(&*devices, &target_root)? {
ENDCODE
Replacement text:
CODE
    if false && tree_table_has_no_entries(&*devices, &target_root)? {
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root
Named test that must turn red: T8 (rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write)

Mutation M10, tsv row 71.
Label: when computing the ceiling for raising F, an unreadable tree table on a valid root is guessed as containing neither tree, instead of being refused.
File: crates/singlefs-core/src/mount.rs, inside context function 1 (rollback_floor_ceiling), inside the pointers_of closure.
Original text:
CODE
        Ok(pointers) => Ok(pointers),
        Err(failure) => Err(
ENDCODE
Replacement text:
CODE
        Ok(pointers) => Ok(pointers),
        Err(_failure) if true => Ok(UserVisibleTreeRootPointers@@ABSENT),
        Err(failure) => Err(
ENDCODE
Cargo test filter: -p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_is_refused_when
Named test that must turn red: T9 (raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable)

Section 6. The write model and the before/after comparison mechanisms

6.1. DiskSnapshot and disk_snapshot (crates/singlefs-harness/tests/common/mod.rs, lines 242-280)

English translation of the doc comment (lines 242-243): A comparable on-disk snapshot: the raw bytes of both superblock slots on each of the two devices, every self-certified root in the root ring, and how many steps the recorded-operation stream already has. Only when every field is equal before and after a refused mount or refused raise-F does it count as refused before any write (the recorded-operation stream not gaining a step means not a single write and not a single barrier was issued).

CODE
pub struct DiskSnapshot {
    pub superblock_slots: Vec<Vec<u8>>,
    pub readable_roots: Vec<singlefs_core@@root_record@@RootRecord>,
    pub recorded_operations: usize,
}

pub fn disk_snapshot(image: &MemoryPool, stream: &SharedStream) -> DiskSnapshot {
    use singlefs_core@@recovery@@PoolReader;
    let spacing = u64@@from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize@@try_from(singlefs_format@@SUPERBLOCK_SLOT_BYTES).expect("4096");
    let mut superblock_slots = Vec@@new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, spacing] {
            superblock_slots.push(
                PoolReader@@read(
                    image,
                    device,
                    singlefs_core@@address@@DeviceOffsetInBytes(offset),
                    slot_bytes,
                )
                .expect("superblock slot is readable"),
            );
        }
    }
    let superblock = singlefs_core@@recovery@@choose_superblock(image).expect("superblock");
    DiskSnapshot {
        superblock_slots,
        readable_roots: singlefs_core@@recovery@@readable_roots(
            image,
            &superblock.region_devices,
            &superblock.geometry,
            &superblock.filesystem_identifier,
        ),
        recorded_operations: stream.operations().len(),
    }
}
ENDCODE

Fact: readable_roots only returns roots that are self-certified (their own checksum or equivalent self-check passes). A root record whose bytes were written to disk but which does not pass self-certification (for example, a torn write that only reached one device, or a write whose checksum does not match its content) is not included in readable_roots, whether it appears before or after the comparison.

Fact: superblock_slots reads the raw bytes at two fixed offsets (0 and the configured spacing) on each device; it does not filter or validate those bytes in any way, it returns them exactly as they are on disk.

6.2. RecordedOperation, RecordedOperationKind, and the recorded-operation stream (crates/singlefs-harness/src/lib.rs)

English translation of the module-level doc comment (line 1 and line 3): A recorder wrapped around a block device, turning every write request and every barrier into one stream, saved to a file for crash-point replay to use. Each record holds: device identity, offset, length, a content hash, and a kind (plain write, force-unit-access write, or barrier). Reads are not recorded: crash states are cut only at writes and barriers.

CODE
pub enum RecordedOperationKind {
    Write,
    WriteForceUnitAccess,
    Barrier,
}

pub struct RecordedOperation {
    pub device: DeviceIdentity,
    pub kind: RecordedOperationKind,
    pub offset: DeviceOffsetInBytes,
    pub length: u64,
    pub content_hash: u64,
}
ENDCODE

Fact, from how the stream records a push (crates/singlefs-harness/src/lib.rs, around lines 101-115): several consecutive barriers with no write between them are recorded as a single entry in the stream, not one entry per barrier call. The relevant part of that function:

CODE
    fn push(&self, operation: RecordedOperation, contents: Option<&[u8]>) {
        let mut state = self.0.borrow_mut();
        let previous_is_barrier = state
            .operations
            .last()
            .is_some_and(|previous| previous.kind == RecordedOperationKind@@Barrier);
        if operation.kind == RecordedOperationKind@@Barrier && previous_is_barrier {
            return;
        }
        // ... (otherwise the operation is pushed and stream.operations().len() increases by 1)
    }
ENDCODE

6.3. CommitStep, the closed set of write kinds a publish can perform (crates/singlefs-core/src/transaction.rs, around lines 64-84 for the enum, 110-162 for how each kind is carried out)

CODE
pub enum CommitStep<'publish> {
    WriteUnitToEveryDevice {
        slot: SlotNumber,
        unit: &'publish [u8],
    },
    WriteJournalRecordToEveryDevice {
        counter: u64,
        record: &'publish [u8],
    },
    WriteRootRecordForceUnitAccess {
        checkpoint_txg: CheckpointTxg,
        root_slot: &'publish [u8],
    },
    RotateSuperblockSlots {
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    },
    Barrier,
}
ENDCODE

Facts about how PoolWriter carries out each CommitStep kind, and which RecordedOperationKind it becomes in the stream (confirmed by the harness's own test code at crates/singlefs-harness/src/device_log.rs, which pairs WriteDurability@@Plain with RecordedOperationKind@@Write and WriteDurability@@ForceUnitAccess with RecordedOperationKind@@WriteForceUnitAccess):

WriteUnitToEveryDevice: for each device, one write at the given slot, using WriteDurability@@Plain (becomes RecordedOperationKind@@Write).
WriteJournalRecordToEveryDevice: for each device, one write at the offset for that journal counter, using WriteDurability@@Plain (becomes RecordedOperationKind@@Write).
WriteRootRecordForceUnitAccess: one write, on the single device that owns the region for this checkpoint_txg, using WriteDurability@@ForceUnitAccess (becomes RecordedOperationKind@@WriteForceUnitAccess).
RotateSuperblockSlots: for each device, one write_superblock_slot call, which itself performs one write using WriteDurability@@Plain at one of the two fixed superblock-slot offsets on that device (becomes RecordedOperationKind@@Write). This is the same write_superblock_slot function acquire_instance calls directly (not through CommitStep@@RotateSuperblockSlots) for each device, followed by one CommitStep@@Barrier.
Barrier: for each device, one barrier call, but only if a write has happened on this PoolWriter since the last barrier (becomes RecordedOperationKind@@Barrier in the stream, subject to the consecutive-barrier collapsing described in 6.2 above).
6.4. allocator_state and AllocatorState, used only by T9 (crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs, around lines 393-420)

CODE
fn allocator_state(pool: &BuiltPool) -> AllocatorState {
    AllocatorState {
        records: pool.allocator.records().to_vec(),
        open_segment: pool.allocator.open_segment(),
        per_device: pool
            .allocator
            .devices
            .iter()
            .map(|device| {
                [
                    device.allocated_slots(),
                    device.free_slots(),
                    device.deferred_slots(),
                    device.isolated_slots(),
                    device.free_runs(),
                    device.empty_segments(),
                ]
            })
            .collect(),
    }
}

struct AllocatorState {
    records: Vec<singlefs_core@@allocator@@AllocationRecord>,
    open_segment: Option<SlotNumber>,
    per_device: Vec<[u64; 6]>,
}
ENDCODE

Fact: AllocatorState reads only from pool.allocator, an in-memory structure. It does not read any bytes from the devices and it does not call disk_snapshot, readable_roots, or anything that re-scans the root ring or the superblock slots.

6.5. BuiltPool, the struct T1, T2, T4 (through FormattedPool, a related struct with the same kind of fields), T7, T8, T9 all use to hold pool state across a test (crates/singlefs-harness/tests/common/mod.rs, around lines 71-80)

CODE
pub struct BuiltPool {
    pub paths: Vec<PathBuf>,
    pub devices: Option<Vec<(DeviceIdentity, Recorded)>>,
    pub stream: SharedStream,
    pub genesis: MakeFilesystemOutput,
    pub warm_up: WarmUpOutput,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub mkfs_operation_count: usize,
}
ENDCODE

Fact: pool.output has type TransactionOutput. It is an in-process value that is only updated when test code explicitly assigns a new value to it (for example, after a successful publish_first_file, publish_overwrite, or a successful mount_writable/mount_rollback call that the test code chooses to assign into pool.output). Reading pool.output does not re-scan the disk; it only returns whatever was last assigned to that field in memory.

6.6. Summary of what each of the three refusal tests (T7, T8, T9) actually compares before versus after the operation it expects to be refused

T7 (writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance): calls disk_snapshot(...) once before and once after, then makes three separate assert_eq! calls: after.superblock_slots against before.superblock_slots, after.readable_roots against before.readable_roots, after.recorded_operations against before.recorded_operations.

T8 (rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write): calls disk_snapshot(...) once before, then makes one assert_eq! comparing the full DiskSnapshot struct returned by a second disk_snapshot(...) call against the first one. Since DiskSnapshot derives PartialEq over all three of its fields, this one assert_eq! is equivalent in coverage to T7's three separate ones (it checks superblock_slots, readable_roots, and recorded_operations, all three, through the derived PartialEq).

T9 (raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable): does not call disk_snapshot at all. It captures allocator_before = allocator_state(&pool), current_before = pool.output.clone(), and recorded_before = pool.stream.operations().len() before the operation, then after the operation makes three assert_eq! calls: allocator_state(&pool) against allocator_before, pool.output against current_before, and pool.stream.operations().len() against recorded_before. It never reads or compares superblock_slots or readable_roots (it never calls disk_snapshot, choose_superblock, or readable_roots after the operation).

Section 7. The task

Answer the following six questions, in order, using only the facts given in sections 1 through 6 above. Follow the rules in section 0 exactly.

Question 1, Grid A. For each of the ten mutations M1 through M10, using only the literal original and replacement code shown in section 4 and the test source text shown in section 3, decide which OTHER tests among T1 through T9 (not the one test that crates/mutations.tsv names for that mutation) would, in your judgment, also turn red if only that one mutation's replacement text were applied by itself (with every other mutation left at its original, unmutated text). Write your answer as ten lines, one per mutation, in this exact format:

M1: <test id or test ids, comma separated, or the word None> ; reason: <for each test id you listed, name the specific assertion in that other test's source text, shown in section 3, that you believe would then fail, and why>

Do the same for M2 through M10. If you write None for a mutation, still write one sentence explaining why you believe none of the other eight tests would be affected.

Question 2, Grid B. For each of the nine tests T1 through T9, using the same facts, decide which OTHER mutations among M1 through M10 (not the one mutation or mutations that name it as their required red test) would, in your judgment, also make this test turn red if applied by itself. Write your answer as nine lines, one per test, in this exact format:

T1: <mutation id or mutation ids, comma separated, or the word None> ; reason: <for each mutation id you listed, name the specific assertion in T1's own source text, shown in section 3, that you believe would then fail, and why>

Do the same for T2 through T9. If you write None for a test, still write one sentence explaining why.

Question 3, Grid C. Below are seven candidate writes or state changes, labeled W1 through W7. For each of the three refusal tests T7, T8, T9, decide whether that test's own before/after comparison (as described precisely in section 6.6) would detect (answer Covered) or would not detect (answer Not-covered) that this candidate write happened, if it happened on its own with no other change, somewhere between that test's before-capture and after-capture. Base your answer only on which specific fields each test actually reads and compares, as given in section 6.6, not on whether the write is likely to happen in practice.

W1: a superblock slot write on one device only (RotateSuperblockSlots for a single device, or equivalently one call to write_superblock_slot for one device index), with no other change.
W2: a journal record write on both devices (WriteJournalRecordToEveryDevice), with no other change.
W3: a root record force-unit-access write that succeeds on both devices and passes self-certification, so it becomes a new entry in readable_roots, with no other change.
W4: a root record force-unit-access write that only reaches one device (a torn write), so it does not pass self-certification and does not become a new entry in readable_roots on either device, with no other change.
W5: a unit write (WriteUnitToEveryDevice), for example writing a tree table unit or an instance table unit, with no corresponding new root record written, with no other change.
W6: a barrier call issued right after another barrier that was already the last entry in the recorded-operation stream, with no write in between, with no other change.
W7: an in-memory allocator state change only (for example, marking two slots as allocated in pool.allocator), with no device touched at all, with no other change.

Write your answer as twenty-one lines, in this exact format, one line per combination of write and test, going through W1 for T7, T8, T9, then W2 for T7, T8, T9, and so on through W7:

W1/T7: <Covered or Not-covered> ; reason: <one sentence naming the specific field this test compares, or the specific field it does not compare, that drives your answer>

Continue through W1/T8, W1/T9, W2/T7, W2/T8, W2/T9, and so on through W7/T9, for all twenty-one combinations.

Question 4. In one sentence, what fact would have to be true, if someone actually compiled each of the ten mutations one at a time and ran the full test suite, for your Grid A answers (Question 1) to be shown wrong.

Question 5. In one sentence, what fact would have to be true, if someone actually compiled each of the ten mutations one at a time and ran the full test suite, for your Grid B answers (Question 2) to be shown wrong.

Question 6. In one sentence, what fact would have to be true, if someone read the actual full source code of PoolReader@@read, readable_roots, and choose_superblock (not shown in full in this document, only described by name and by what section 6.1 says they do), for your Grid C answers (Question 3) to be shown wrong.
