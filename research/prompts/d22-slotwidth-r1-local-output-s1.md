1. The worst crash state is a partially written slot where the checksum fails due to torn writes across multiple 512-byte blocks, but the other slot remains intact with a valid checksum and higher generation. The reader can still pick the valid slot. No state exists where both slots are unreadable.

2. No. Candidate B's slot width exceeds physical_block_size, so the torn-write detection resolution (whole-slot checksum) would be larger than physical_block_size, violating the rule.

3. Yes. The mount rule in F7 refuses the mount when live io_min exceeds recorded spacing, preventing filesystem operation and ensuring no writes smaller than io_min are issued, which is acceptable.

4. Zero extra disk bytes. The bytes between slots were already unused and reserved in the device layout; candidate B repurposes this existing unused space without additional allocation.

5. 3615 bytes remain under candidate B on the local machine. A device descriptor pointer of 86 bytes does not fit under candidate A (only 31 bytes remain) but fits under candidate B.

6. Candidate B contradicts F4. Candidate A contradicts none.
