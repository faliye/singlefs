1. For A-S:  
   - Deleted data: If a unit has a wrong locality_id (as a hint), MAC verification fails due to locality_id being in associated data, so data is not found.  
   - Orphans: Same issue; wrong locality_id causes MAC failure during rebuild.  
   - Packed objects re-keyed: Not applicable as slots don't carry locality_id, but shared object re-keying by two clones causes MAC failure when read by the other clone (different locality_ids in MAC).  
   - Two clone heads: Shared object re-keyed by both clones; each has different locality_id in MAC, so one clone cannot verify data from the other.  

   For B-prime:  
   - Two clone heads: Shared object re-keyed; each clone uses its own locality_id as key prefix. Data written with clone A's locality_id is not found when read by clone B (key prefix mismatch).  
   - Deleted data/orphans: Locality_id not in MAC, so data is found. Packed objects are unaffected.  

   For C-prime-record and C-prime-S1:  
   - No locality_id; keys are (tree ID, inode, offset). Deleted data, orphans, packed objects, and clone heads work correctly. No scenarios found where data is missed or duplicated.  

2. A-S conflicts with F5 (locality_id in associated data violates "must never be part of associated data") and F17 (S6 ignores locality_id in grouping, but F17 requires all identity fields for grouping). A-S does not conflict with F15 (packing slot does not carry locality_id). C-prime-S1 does not conflict with F14 (rebuild process respects superblock state and does not repair in place before promotion).  

3. Cost:  
   - A-S: 41 bytes/unit, 41 first-transaction bytes. Rename: no change. Background moves: re-encrypt data when moving due to locality_id in MAC. Re-clustering: requires re-keying with locality_id changes. Rebuild: multi-value prefixes, read locality_id from header.  
   - B-prime: 41 bytes/unit, 41 first-transaction bytes. Rename: no change. Background moves: no re-encryption (locality_id not in MAC), just key prefix update. Re-clustering: physical placement only, no header changes. Rebuild: prefix from surviving key, record, or 0.  
   - C-prime: 33 bytes/unit, 33 first-transaction bytes. Rename: no effect. Background moves: no locality_id, so no key prefix changes. Re-clustering: no action needed. Rebuild: simpler, no locality_id handling.  

4. S10 can turn red on a legal image only if one side was not derived from the other but the check misinterprets it. For S6: Check is grouping by object and anchor offset ignoring locality_id; it turns red if two units have same object, offset, and write order but different locality_id (violating F17's same key must have different total-order keys). For S7: Check is multi-value prefixes must include all data units' locality_ids; it turns red if a data unit's locality_id is missing from the set (e.g., rebuild process skipped it). For S8: Check is other key must be dropped during copy-then-cut; it turns red if the key remains (e.g., failure to drop).  

5. Pick C-prime. Single observation that would change pick: If empirical evidence shows locality_id significantly improves performance for small cache sizes (e.g., >2x speedup) and the risks are mitigated, but current design (F4) shows benefit disappears with full cache, so unlikely.
