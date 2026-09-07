1. For the six rows:
   - Ignore + Carry: Matches parameters. 4000 records misinterpreted (4 unknown trees × 1000 records each), 0 rows lost (Carry policy rewrites all unknown rows), 19 rows written per generation (17 known rows: 6 for per-tree statistic on known writable heads + 11 pool-wide, plus 2 for unknown writable heads per-tree statistic).
   - Ignore + NoCarry: Does not match. Should have 17 rows written (only known rows) and at least 2 rows lost (for oldest generation unknown rows), but reported 19 written and 0 lost.
   - Skip + Carry: Does not match. Should have 17 rows written (only known rows) and 2 rows lost (for oldest generation unknown rows), but reported 19 written and 0 lost.
   - Skip + NoCarry: Matches parameters. 0 misinterpreted (trees skipped), 2 rows lost (oldest generation rows for two unknown writable heads), 17 rows written per generation (only known rows).
   - Refuse + either: Matches parameters. Does not mount, nothing happens, 0 rows written.
   - Two rows cannot be reproduced: Ignore + NoCarry and Skip + Carry.

2. Choosing by fewest quiet outcomes is not sound. It fails to price in the severity or impact of errors. For example, silent data corruption (quiet outcome) is worse than a reported error like missing trees, but the quiet measure treats both equally by only counting whether an error is reported, not how harmful it is. A better measure is expected harm, which considers the probability and consequence of errors, such as potential system instability or data loss.

3. Maintaining misinterpreted accounting rows under Ignore is worse than losing them. On one hand, losing rows causes an error that can be reported and handled, preventing further issues. On the other hand, maintaining misinterpreted rows leads to silent corruption that may go undetected, causing system instability or data loss later. Silent corruption is more dangerous because it can propagate errors without warning, so it is worse than losing rows.

4. To rewrite a row, the writer needs:
   - Statistic tag: Can be assumed to have it as opaque data.
   - Dimension tuple: Can be assumed to have it as opaque data.
   - Current generation number: Has it for the row.
   - Value: Has it.
   - New generation number: Can be computed or known for the update.
   All can be assumed to have it, as the writer can store keys and values as raw bytes regardless of understanding.

5. The strongest objection not covered is that Skip plus Carry results in critical data inaccessibility without recovery options. Skipping unknown trees makes their data permanently unavailable to version one, and if the system does not report this, it causes silent failure. This is more severe than misinterpretation because it removes data entirely, whereas misinterpretation retains data that might be corrected later.
