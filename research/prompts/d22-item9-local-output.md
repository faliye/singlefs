2

1. The 512-byte slot width only occurs if all disks in the pool have 512-byte physical block sizes. Modern disks typically use 4K blocks, so the slot width would be 4K in most cases. The conclusion should state that the merged arm is necessary only if the slot width is 512 bytes, not as an unconditional requirement.

2. One of the 11 undecided fields could be "used blocks," which is not strictly required for filesystem operation and can be derived from other metadata structures. Removing it reduces the total size by at least 4 bytes, potentially allowing the non-merging arm to fit within 512 bytes.

3. The merged arm does not create a bootstrap cycle. The merged directory unit's size can be stored in a fixed header (e.g., first 4 bytes), allowing the system to read the header first to determine the unit's size before reading the full unit. This breaks any potential cycle.

4. Yes, a third shape exists by moving some fields out of the superblock entirely. For example, the "last mount time" field could be stored in a separate journal file instead of the superblock. This reduces the required items below 31, allowing the non-merging arm to fit within 512 bytes without merging.

5. Mounting requires one extra read to fetch the merged directory unit. Each publish that modifies the directory unit requires one extra write to update it. For the non-merging arm, updates to the four pointers are written directly in the superblock slot without additional reads or writes.
