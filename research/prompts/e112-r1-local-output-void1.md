1. The arithmetic checks out for all rows:
   - Ignore + Carry: 4 unknown trees × 1000 records = 4000 misinterpreted records. Accounting rows for unknown trees are carried (no loss). Rows written per generation: 11 pool-wide + 6 known writable heads' per-tree rows + 2 unknown writable heads' per-tree rows = 19.
   - Ignore + NoCarry: Same 4000 misinterpreted records. Since Ignore treats trees as known, accounting rows for unknown trees are not considered "belonging to unknown trees" → 0 rows lost. Rows written per generation = 19 (same as above).
   - Skip + Carry: Skips tree entries → 0 misinterpreted records. Accounting rows for unknown trees are carried → 0 rows lost. Rows written per generation: 11 pool-wide + 6 known writable heads' rows = 17 + 2 unknown writable heads' rows = 19.
   - Skip + NoCarry: Skips tree entries → 0 misinterpreted records. Accounting rows for unknown trees not carried → 2 rows lost (the two unknown writable heads' per-tree rows in current generation). Rows written per generation: 11 + 6 = 17.
   - Refuse: Does not mount → no rows written or processed.

2. Choosing by fewest quiet outcomes is not sound. It fails to price in the severity or impact of silent errors. For example, a silent data loss might be worse than a silent misinterpretation of non-critical data, but the quiet measure treats all silent errors equally. A better measure would weight silent errors by their potential impact (e.g., data corruption vs. minor statistic miscount) and prioritize minimizing high-impact silent errors.

3. Arguments for maintaining rows under Ignore: Preserves data for future versions to use, avoids permanent loss. Arguments against: Misinterpreting data could silently corrupt critical operations (e.g., space accounting errors leading to over-allocation or data loss). I believe it is worse than losing them because silent corruption is more dangerous than data loss; the latter can be detected and recovered from, while silent corruption may cause irreversible damage without warning.

4. To rewrite a row, the writer needs: 
   - Statistic tag (raw bytes can be copied; yes, writer has it).
   - Dimension tuple (raw bytes can be copied; yes, writer has it).
   - Generation number (must know current generation to update; yes, writer has it).
   - Value data (raw bytes can be copied; yes, writer has it).
   A writer that does not recognize the statistic tag can assume it has all raw data components since it only needs to copy bytes without understanding their meaning.

5. The strongest objection not covered: Skipping tree entries means the actual file data in those trees is inaccessible inaccessible, even if accounting rows are preserved. This renders the filesystem incomplete and unusable for critical operations, which is a far more severe issue than accounting row handling.
