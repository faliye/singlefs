1. Create a parent directory with locality_id 0. Create two child directories D1 and D2 under it, each inheriting locality_id 0. Write 32 files of 512 bytes each to D1 and 32 files of 512 bytes each to D2. Delete all files in D1. At the deletion step, the freed space is fragmented because D2's files are interleaved in the same contiguous segment as D1's files, preventing a full segment from being freed. This is worse than if D1 and D2 had separate locality_ids where deleting D1 would free a full contiguous segment.

2. No such operation exists.

3. Create 100 directories each with 32 files of 512 bytes. Pack all files into containers where each container holds 58 files. Delete one directory's 32 files, which are spread across 32 different containers. Each of these containers now has 57 files, leaving 3584 bytes free per container. The largest contiguous free block is 3584 bytes. When attempting to write a new 32768 byte file, the system fails due to no contiguous block despite 114688 total free bytes. Under a strategy grouping small files by directory, deleting one directory would free a full 32768 byte container, allowing allocation. The cost of the allocation failure outweighs the space savings from packing.

4. No such reason exists.

5. The project loses both traversal performance (1317 leaves read with inode keys vs 901 with locality keys for 8-leaf cache) and reclamation efficiency (frees none when deleting a directory under arrival order vs two segments under grouped placement).

6. Attack arm E because grouping small files by locality_id could pack unrelated directories' files together, causing fragmented reclamation when deleting a single directory despite container packing.
