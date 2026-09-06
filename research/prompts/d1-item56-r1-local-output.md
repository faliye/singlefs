For shape 1: Before crash, root ring has R1 (txg=10), R2 (txg=20), R3 (txg=30) with K=2. U0 is referenced only by R1 and reallocated for U_new with birth generation 10. After crash, K=3. U1 has a stack entry pointing to U0 with stored birth generation 10. When reading U1, gate 1 passes (U_new block referenced by R2 in K window), gate 2 passes (birth generation 10 matches), and reader sees U_new content misidentified as U0. Relies on F2, F5, F6, F10(a).

For shape 2: During filesystem recovery after crash, K is not yet accessible (e.g., K storage location requires reading a block that is not yet loaded, causing dependency loop). Reader attempts to dereference a stack entry but cannot evaluate gate 1 due to missing K value. Relies on F5, F11.

For shape 3: Cannot construct a counterexample as no expiry does not destroy anything required to survive; F6 conditions and deadlist mechanism ensure referenced units survive until snapshot destruction, and F7 requires witness for judgment but not survival.

For shape 4: With depth 3, a unit modified 4 times (U0 to U4) has U0 missing from stack (only U3, U2, U1 stored), whereas depth 5 would retain U0. Attempting to access U0 fails, losing capability to view older versions beyond 3 steps. Relies on F3.

For shape 5: Stack entries in extension point point to core-layer units (e.g., previous versions), violating F8 hard constraint 5 which requires pointers to only reference extension point quota, not core-layer objects. Proposal does not address this conflict. Relies on F8.
