1. For arm B:
   - Create file with index node (type 2) and data block (type 1) at txg=1.
   - Take snapshot S1 at txg=2.
   - Delete the file. Both blocks added to deadlist.
   - Move index node. New index node written at new location (birth=3), old location freed. Parent pointers in S1's tree not updated because S1 is read-only.
   - Destroy S1. When processing deadlist, system tries to free index node using version key but cannot find current location in central map (arm B only maps type 1). Block remains unfreed, causing leak.
   For arm A:
   - Same sequence. Central map has type 2 entries. After moving index node, map updated. When destroying S1, deadlist entry for index node is resolved via map, block freed correctly. No leak.

2. For arm B:
   - Original snapshot S (txg=2) references index node C (birth=1) and root R (birth=1).
   - Move C to new location. Write new C' (birth=3) and new R' (birth=3) pointing to C'.
   - When S is destroyed (txg=4), F1 requires birth(b) <= S.txg (2) < death(b) for S to reference b. For C', birth=3 > 2, so S does not reference C' per F1. However, S's tree now points to C', violating F1.

3. F8 states index nodes are not restored from their own headers during scan rebuild; trees are rebuilt from leaves. Therefore, no scan-rebuild path requires reading a key component from an old type 2 header. It is impossible.

4. For arm B rule: type 2/3 units referenced by any snapshot never moved. 
   - Disk failure occurs. Failed disk contains type 2 block C referenced by snapshot S. Rebuild process cannot move C to new disk due to rule. Rebuild fails for C, causing data loss.
   - Compaction cannot move C to defragment disk. C remains fragmented, reducing performance. C can still be freed when S is destroyed, but compaction is blocked indefinitely.
