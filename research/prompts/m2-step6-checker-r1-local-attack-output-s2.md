1. The comment's claim that only the warm up roots reference the genesis tree table unit is supported only if the first file publish relocated the tree table to a new location, causing A's tree table pointer to change. If the first file publish updated the existing genesis tree table unit in place (same location), A would still reference it. To settle this with certainty, the additional fact needed is whether the first file publish causes the tree table pointer in the new root record to point to a new location or retains the original location.

If it were true that the first file publish always updates the tree table in place (same location), the comment's claim would be false and A would reference the genesis unit. This would contradict the comment's assertion.

2. If A does not reference the genesis tree table unit, the newest walk (A) will not encounter the corrupted unit. Therefore, the newest walk's failures list remains empty and the I-2.1 violations count remains zero, so neither I-7.2 nor the I-4.8 judgement is affected. There is no indirect path for the corruption to affect the newest walk because the comment states only warm up roots reference the unit, and A's walk only follows its own pointers.

If A indirectly referenced the genesis unit through another structure (e.g., a chain of pointers), the newest walk would encounter the corruption, affecting the failures list and I-4.8 judgement. This would contradict the assumption that A does not reference the unit.

3. Both bad images cause a checksum mismatch during the unit read, triggering an I-2.1 violation. This increases the I-2.1 violations count and walk failures list length for the older candidate walk, causing both I-7.4 and I-4.8 to be flagged as violated in the same pass. Thus, they do not exercise independent failure modes. To isolate them, a bad image must cause I-7.4 violation (block reused) without checksum mismatch (e.g., reused block with identical checksum) and another causing I-4.8 violation (checksum mismatch) without reuse (e.g., corruption in non-reused block). The current checker cannot detect I-7.4 violations without checksum mismatches, so this isolation requires changing how I-7.4 is checked (e.g., checking allocation maps for reuse).

If a reused block had the same checksum as the original, the checksum check would pass, so I-2.1 would not fail. This would mean I-7.4 is violated but not flagged, while I-4.8 holds. This would show the current method cannot isolate I-7.4 without I-4.8.

4. Both mutations cause a checksum mismatch between the stored value in the location entry and the current bytes' computed checksum. The checker only compares these two values and ignores the unit's internal header consistency. Thus, both mutations are detected identically, triggering the same I-2.1 violation and subsequent flagging of both invariants. The difference in internal consistency does not change the checker's response.

If the first bad image's modified content accidentally had the same checksum as the original (e.g., due to hash collision), the checksum check would pass, and the mutation would not trigger a failure. This would contradict the assertion that both mutations are caught the same way.

5. Marking I-7.4 as not applicable is incorrect. The candidate set includes the newest root, so the invariant must be checked on it. The checker's current process only checks I-7.4 during the older candidate loop, but the newest root is part of the candidate set and should be evaluated. The candidate set definition explicitly includes the newest root and has no minimum size requirement, so the invariant applies to it.

If the checker's process were modified to check I-7.4 on the newest root during its initial walk (like I-4.8), marking it as not applicable would be wrong. This would refute the current behavior.

6. It is inconsistent for I-7.4 to be marked not applicable while I-4.8 is checked when only the newest root is in the candidate set. Both invariants specify "for every root in the rollback candidate set," which includes the newest root. The checker should evaluate both on the newest root. The difference in treatment arises from implementation flaws, not the wording of the invariants. Both should be checked identically for the newest root.

If the checker's process for I-7.4 were updated to include the newest root (e.g., during the initial walk), treating them differently would be unjustified. This would contradict the current inconsistent behavior.
