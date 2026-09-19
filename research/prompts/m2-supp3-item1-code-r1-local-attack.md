Task for a local reviewer: two fact-checking exercises about a Rust file-system project.

General instructions, read first.
- Write your whole answer in English.
- Do not use any markdown emphasis anywhere in your answer: no bold, no italics, no underline. Plain sentences and plain numbered lines only.
- Number every answer line to match the row label given in the fact tables below (for example Z5-1, Z5-2, ..., Z1-A, Z1-B, ...).
- After every answer line, add one more line that starts with "Would be wrong if:" and states a concrete, checkable observation that would prove that specific answer wrong. A vague line such as "if the logic were different" does not count; name the concrete sector numbers, or the concrete sentence, that would have to change.
- Do not write any source-code line numbers or file line numbers anywhere in your answer, not even to report what you think a line number is. If you need to point at something, name it by function name, table row id, or column name instead.
- Fill in every cell of every table. Do not answer a whole row with only "yes" or "no": state the concrete sector numbers, or the concrete sentence, you computed.
- All the facts you need are given below. Do not guess at anything not stated below; if a table gives you the two source excerpts for a row, your job is to compare exactly those two excerpts, not to go look for more context.

Part one, label Z5: comparing two ways of reading a sparse in-memory disk image.

Background, given as fact, not to be re-derived: a test harness models a disk as a sparse image. The underlying storage is a sorted map from sector number (a non-negative integer, sector 0, 1, 2, and so on) to a 512-byte record. An entry exists in the map only for a sector that was written at least once. The map never holds an entry for a sector that was never written. The constant SECTOR_BYTES equals 512 (bytes per sector). All offsets and lengths below are already multiples of 512.

Old version, pseudocode, called old_read:

    function old_read(offset_bytes, length_bytes):
        assert offset_bytes is a multiple of 512
        assert length_bytes is a multiple of 512
        output = a new byte array of length_bytes bytes, every byte initialized to 0
        first_sector = offset_bytes / 512
        sector_count = length_bytes / 512
        for index from 0 up to but not including sector_count:
            sector_number = first_sector + index
            look up sector_number in the sparse map
            if the map has an entry for sector_number:
                copy that entry's 512 bytes into output, starting at byte position (index * 512)
            else:
                leave output's bytes at that position as 0 (they already are)
        return output

New version, pseudocode, called read_into, plus a thin wrapper new_read that has the same calling shape as old_read:

    function read_into(offset_bytes, buffer):
        assert offset_bytes is a multiple of 512
        assert the length of buffer is a multiple of 512
        set every byte of buffer to 0
        first_sector = offset_bytes / 512
        sector_count = (length of buffer) / 512
        for each (sector_number, bytes) pair that is actually present in the sparse map and whose sector_number satisfies first_sector <= sector_number < first_sector + sector_count:
            index = sector_number - first_sector
            copy bytes (512 bytes) into buffer, starting at byte position (index * 512)
        (nothing is returned; buffer has been filled in place)

    function new_read(offset_bytes, length_bytes):
        buffer = a new byte array of length_bytes bytes, every byte initialized to 0
        call read_into(offset_bytes, buffer)
        return buffer

Note on how the map iteration works, given as fact: "for each (sector_number, bytes) pair that is actually present in the sparse map and whose sector_number satisfies ..." only visits sector numbers that have an entry in the map; it never visits a sector number that was never written, and it visits them in increasing order of sector_number. This is the only way read_into looks at the map; it never calls a single-sector lookup the way old_read's loop does.

Fact table, Z5. Every row already fixes offset_bytes, length_bytes, and the exact set of sector numbers that have an entry in the sparse map (that is, sector numbers that were written at some point before this read). Sector numbers not listed in the "written sector numbers" column have no entry in the map. This table is authoritative; do not change any of its numbers.

Row id | offset_bytes | length_bytes | written sector numbers (global sector numbers, i.e. entries present in the sparse map)
Z5-1 | 5120 | 1536 | { 10 }
Z5-2 | 5120 | 1536 | { 12 }
Z5-3 | 5120 | 1536 | { 13 }
Z5-4 | 5120 | 1536 | { 9 }
Z5-5 | 5120 | 1536 | { } (empty, no sector has an entry)
Z5-6 | 5120 | 1536 | { 10, 11, 12 }
Z5-7 | 5120 | 0 | { 10 }
Z5-8 | 5120 | 512000 | { 10, 1009 }

For every row Z5-1 through Z5-8, answer these four things:
(a) Which sector numbers (inside the range [offset_bytes/512, offset_bytes/512 + length_bytes/512) ) does old_read fill with the map's stored 512-byte content, as opposed to leaving as 512 zero bytes? List the sector numbers explicitly, or state "none" if none.
(b) Which sector numbers does new_read (via read_into) fill with the map's stored content, as opposed to leaving as zero? List the sector numbers explicitly, or state "none" if none.
(c) Is the full output buffer old_read returns byte-for-byte identical to the full output buffer new_read returns, for this row? Answer yes or no.
(d) The "Would be wrong if:" falsification line for this row.

Part two, label Z1: comparing a documented precondition against an enforced precondition, for seven entry points of a file-system library.

Background: this library has seven public entry points that a test harness's random-history generator calls. For each entry point, two independent pieces of source text are quoted below, translated from the original source comments into English, word for word as closely as English allows, with no content added beyond what is stated:
- "Doc text" is the entry point's own documentation comment, taken from where that function is defined.
- "Generator text" is the comment and the runtime check that the random-history generator performs immediately before it decides to call that entry point.
Both are given to you as fixed fact. Do not look for more context; use exactly what is quoted.

Row Z1-A, entry point publish_first_file.
Doc text: "This function publishes the first file version. Regarding errors: the checkpoint number and the journal sequence number are hardcoded to 3 inside this function. This entry point only accepts being called on the version whose checkpoint number is 2 and whose journal sequence number is 2, that is, the version that exists right after the second warm-up publish. If the previous record's bytes cannot be parsed into a record belonging to this pool, or if that record's checkpoint number or journal sequence number is not 2, the call returns the error FirstFileVersionNotRightAfterTheSecondWarmUp and writes zero bytes to the device. This check is against the previous journal record, not against the pool's genesis root: within the same process that formatted the pool, this function is passed the generation-0 root produced by formatting, but that root is only used to copy an instance-table pointer from it, not as the thing being checked."
Generator text: "Before calling publish_first_file, the generator checks exactly one thing: that a writable session is currently open in memory. If no session is open, the step is marked as not applicable, with the reason recorded as NoWritableSession. The generator does not check, before this call, whether the current in-memory version already has a file or not. The generator's table of operation weights assigns this operation a nonzero chance of being drawn even in the state where a file already exists (the operation weights document literally lists it at 4 percent of draws in that state)."

Row Z1-B, entry point publish_overwrite.
Doc text: "This function publishes a new version of the same file, right after the previous publish inside the same instance: the checkpoint number, the journal sequence number, and the transaction number each increase by one; the file object's birth generation and its container identity do not change; the change-count field is set to this publish's checkpoint number; the previous version's eight storage placements are released through the mapping layer, into the deferred-release queue. Regarding errors: if the release-determination path cannot find one of the previous version's units (a family of errors named ReleaseNotInMapping), or if there is not enough space, or the content does not fit, or the block device reports an error, that error is returned unchanged. The function's parameter for the previous version is typed as a TransactionOutput, which is the type produced by a publish that carries a file."
Generator text: "Before calling publish_overwrite, the generator checks two things: first, that a writable session is open (otherwise the step is marked not applicable, reason NoWritableSession); second, that the current in-memory version is the variant that carries an existing file, as opposed to the variant whose tree table has zero entries (otherwise the step is marked not applicable, reason CurrentVersionWithoutFile). Only when both checks pass does the generator call publish_overwrite, passing that existing-file version as the previous version."

Row Z1-C, entry point publish_without_units.
Doc text: "This function performs a zero-unit publish: a barrier, then an empty journal record, then a barrier, then an FUA write of the root slot, then rotation of the superblock slot. The empty record names no unit, has transaction number 0 and commit flag 1; the new root's segment copies the previous version's root, and the new root record copies the previous version's root except that the checkpoint number, the instance generation, and the rollback floor change. Both the first writable mount's warm-up publishes and a writable mount on a pool that has only ever been formatted (never published a file) go through this function. Regarding errors: this function's documentation states only that whatever the block device reports is returned unchanged; its documentation does not state, and does not list as an error, what happens if it is called while the current version already carries a file. Elsewhere in the source, one settled decision item is cited by name as the justification for when this function is meant to be used: that when the tree table has zero entries, a publish writes zero units."
Generator text: "Before calling publish_without_units, the generator checks two things: first, that a writable session is open (otherwise not applicable, reason NoWritableSession); second, that the current in-memory version is the variant whose tree table has zero entries, as opposed to the variant that carries a file (otherwise not applicable, reason CurrentVersionWithFile). A comment at this second check states, in the generator's own words: this precondition, that the function is only to be called on a version whose tree table has zero entries, is not written out explicitly in any settled decision item; it is the generator author's own reading, taken from this entry point's documentation comment, of when the function is meant to be used."

Row Z1-D, entry point mount_writable.
Doc text: "This function performs a writable mount: recover, then rebuild the previous version and the allocator, then acquire a journal sequence number, then publish the row-writing publish, then warm up until this instance's root covers every device. A pool that has only ever been formatted (the chosen root's tree table has zero entries and the journal ring holds no records at all) goes through the same code path: it acquires number 1, does not write the row-writing publish, and instead pushes zero-unit publishes until this instance's root covers every device; the first file version, if one is published later, follows right after that. Regarding errors: recovery failure; the chosen root carries a file but the journal ring does not hold a single record; the instance table cannot be parsed; admission before number acquisition does not pass (the allocation-record tree, the accounting tree, or the one instance-table page does not fit); number acquisition fails; publish fails. Nothing in this function's documentation states a precondition about what in-memory session state the caller must be in before calling it."
Generator text: "Before calling mount_writable, the generator sets the in-memory session to closed unconditionally, increments a mount-attempt counter, and then calls mount_writable with no precondition check at all beforehand. This call is always applicable, regardless of what the in-memory session state was immediately before it."

Row Z1-E, entry point mount_rollback.
Doc text: "This function performs an administrator rollback, described as an explicit exception to the normal journal-replay rule: out of band, the caller picks an old root, called R_old, from the rollback candidate set, and no record after R_old is applied; a new instance generation is acquired; a rollback row and an intermediate-instance row are written onto the instance table that R_old points to; the rollback row, this publish's units, and the first new root are all part of the same publish. Regarding errors: the target root is not in the root ring; the target root is not in the rollback candidate set; the target root's own journal record cannot be read; admission before number acquisition does not pass (counted the same way as for a writable mount, with the instance table counted as of R_old's version); number acquisition fails; publish fails. Note that 'not in the root ring' and 'not in the candidate set' are listed as two separate error conditions, not one: a root can be in the ring yet not in the candidate set."
Generator text: "Before calling mount_rollback, the generator sets the in-memory session to closed, then reads the ring's roots, newest first, off the on-disk image; if there is not a single readable root in the ring, the step is marked not applicable, reason NoReadableRootInRing. Otherwise the generator picks a rollback target using one of three modes, chosen at random: about two fifths of the time, an index from 0 up to 3 counted back from the newest root (this usually lands inside the candidate set); about two fifths of the time, an index drawn from the full width of the ring, computed modulo the total number of roots currently in the ring (this can land on a root that is in the ring but not in the candidate set, for instance a root below the current rollback floor, or a root belonging to an abandoned instance); the remaining time, a target whose checkpoint number is the newest root's checkpoint number, plus 1, plus a value from 0 to 2, which by construction is a checkpoint number that does not belong to any root currently in the ring at all. In every one of the three modes, the generator calls mount_rollback with that target without first checking, itself, whether the target is in the ring, whether it is in the candidate set, or whether its own journal record can be read; the generator leaves all of that to mount_rollback, and treats whatever error mount_rollback returns (if any) as an accepted, legitimate outcome of the step."

Row Z1-F, entry point raise_rollback_floor.
Doc text: "This function raises the rollback floor, called F, to a caller-supplied target value. Its documentation states that the normal trigger for this is admission not passing, and that this entry point itself is a forced entry point that exists only for testing purposes. It pushes empty publishes until every device carries a root with the new F (that is the point at which the new floor takes effect), then reclaims released placements whose release generation is at most the new F. Regarding errors: the current version has no rewritten instance-table unit; the new floor exceeds a ceiling (the ceiling being, in general terms, bounded by the newest valid root on each device, which is itself at most the current checkpoint number); publish fails."
Generator text: "Before calling raise_rollback_floor, the generator checks that a writable session is open (otherwise not applicable, reason NoWritableSession), and that the current version is the variant that carries a file (otherwise not applicable, reason CurrentVersionWithoutFile). It then computes a candidate new floor as: the current version's floor value, plus a random nonnegative integer taken modulo the quantity (current checkpoint number minus current floor, plus 3). Note that this formula's result can be as large as (current floor) plus (current checkpoint number minus current floor plus 2), which simplifies to (current checkpoint number plus 2). A comment attached to this computation states, in the generator author's own words: the generator's own precondition is that the target is only ever taken at or above the current floor, never below it; this precondition is not written in any settled decision item, it is read from this entry point's own documentation comment. The same comment also states that no decision item says whether a target below the current floor would be accepted by this entry point, or whether accepting one would even be a legitimate outcome. The same comment also states, separately, that because the formula's maximum output (current checkpoint number plus 2) can exceed the ceiling described in the entry point's own error list (which is bounded by the newest valid root on each device, itself at most the current checkpoint number), a target above the ceiling is reachable by construction from this formula."

Row Z1-G, entry point recover (the cold-start recovery path).
Doc text: "This function's entire documentation comment reads: this is the whole recovery path; the root value reported in its result is always the chosen root; after journal records are applied, what is actually used internally is the rebuilt root. This function does not return a Result type the way the other six entry points above do; it returns a plain report value. Its documentation comment has no separate errors section, and states no precondition on caller state at all."
Generator text: "Before calling recover, the generator sets the in-memory session to closed, then calls recover unconditionally. There is no precondition check of any kind before this call. The outcome is always recorded as an applied step, never as a not-applicable step and never as a refusal, regardless of what state the pool was in beforehand."

For every row Z1-A through Z1-G, fill in a three-column answer:
(1) One sentence stating the precondition that row's Doc text documents for calling that entry point (if the Doc text states no such precondition at all, say so plainly instead of inventing one).
(2) One sentence stating the precondition that row's Generator text enforces before calling that entry point (if the Generator text states no check at all, say so plainly).
(3) One sentence stating the difference between columns (1) and (2): is the generator's guard narrower than, wider than, exactly matching, or simply answering a different question than, the documented precondition? Point at the specific words in the two quoted texts that make this so.
(4) The "Would be wrong if:" falsification line for this row.
