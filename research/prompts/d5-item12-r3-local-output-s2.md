1. For C-interval with upper bound: a crash between writing the intent and passing prev to the next newer side causes blocks with birth between prev(S).txg and S.txg to be freed even if referenced by a snapshot in that range. For B with ordering: no known scenario where a referenced block is freed or a block that should be freed is not; cascade intents run in order and deadlist number swaps handle crashes safely. For C-prime with reserve: same as C-interval; crash between intent write and passing prev leads to freeing referenced blocks.

2. C-interval: 43 bytes per entry, 0 tree table bytes; lookups by head tree ID + epoch range. B with ordering: 35 bytes per entry, 8 tree table bytes for deadlist number; lookups by deadlist number + mapping key. C-prime with reserve: 43 bytes per entry, 8 tree table bytes for head's latest snapshot tree ID; lookups by head tree ID + epoch (tree ID of previous snapshot).

3. C-interval: 24 reserved bytes fit lineage labels (16) and children counter (8). B with ordering: 16 remaining bytes fit lineage labels (16). C-prime with reserve: 16 remaining bytes fit lineage labels (16).

4. For B with ordering: check that for any block in the deadlist being freed, birth(b) > all snapshot txgs that reference it. If a block is freed but birth(b) <= some snapshot's txg, the check turns red. Proof: the intent processes in order and deadlist number swaps ensure only unreferenced blocks are freed.

5. Pick B with ordering. The single observation that would change the pick is if the deadlist number swap operation becomes prohibitively expensive for large deadlists.
