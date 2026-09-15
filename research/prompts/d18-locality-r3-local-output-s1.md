1. For A-S: Packed objects re-keyed fail MAC verification because container MAC cannot bind multiple locality_id values. Two clone heads re-keying same shared object cause old data units to have mismatched locality_id in associated data, leading to MAC failure and data not found. During rebuild, S7 causes one object to have multiple key prefixes (e.g., some keys with prefix X, others with Y), so reader cannot find data if checking only one prefix.  
For B-prime: No scenario found where data not found, object under two prefixes, or wrong data seen. Packed objects re-keyed work correctly (locality_id not in MAC). Two clone heads share object under different prefixes but in separate trees, which is correct.  
For C-prime-record: No issues. All keys for an object have same prefix. Deleted/truncated data remains unlinked; orphans not found; packed objects work; clone heads use different inode numbers.  
For C-prime-S1: Same as C-prime-record. No issues.  

2. A-S conflicts with F5 (locality_id in associated data violates "must never be part"), F15 (container MAC cannot handle multiple locality_id values for slots), and F17 (S6 ignores locality_id in grouping, violating "all fields of class identity section"). C-prime-S1 does not conflict with F14; F14 restricts in-place repair during rebuild, but C-prime-S1's prefix assignment is part of read-only rebuild process.  

3. A-S: 41 bytes per unit; packing slots same size; first-transaction bytes increase; rename no change; background moves require re-keying; re-clustering requires re-keying; rebuild reads locality_id from header but fails for packed objects.  
B-prime: 41 bytes per unit; packing slots same size; first-transaction bytes increase; rename no change; background moves physical placement only; re-clustering physical placement only; rebuild uses S3 prefix order.  
C-prime: 33 bytes per unit; packing slots same size; first-transaction bytes unchanged; rename no change; background moves physical placement only; re-clustering physical placement only; rebuild uses S9 prefix order.  

4. S10 cannot turn red on legal image (it reports "undecidable" when one side derived from other, which is correct). For A-S:  
- S6: Check for two units with same object ID, anchor offset, payload CRC, but different locality_id. Grouped together (S6 ignores locality_id), but I-1.8 requires different write orders. If same write order, it turns red.  
- S7: Check for object with multiple key prefixes. I-9.9 requires record's locality_id to match all extent keys, but record has multiple prefixes. Turns red.  
- S8: Check for copy-then-cut where key is dropped but data unit remains. Rebuild finds data unit but no key, so I-9.9 fails as no extent key to match. Turns red.  

5. Pick C-prime. Single observation that would change pick: if performance tests show locality_id prefix provides significant read speedup (e.g., >2x) for small cache sizes in real workloads, and the 8-byte overhead is acceptable for the use case.
