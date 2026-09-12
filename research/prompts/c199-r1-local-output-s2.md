1. Yes. Instance 1 root R1 (txg=10) exists. Instance switches to 2; transaction T1 (fsync'd) writes journal records and durable root R2. Crash occurs. Recovery selects R2 but replay stops at instance boundary due to non-consecutive jsn (instance 1 counter 100 → instance 2 counter 1), skipping T1's records. One fault (crash) suffices.

2. Cannot construct sequence. Candidate B skips all records of instances with W=0 (per settled rule) and requires counter continuity. Stale records either have lower instance numbers (skipped due to jsn gap) or W=0 (explicitly skipped). No valid sequence applies incorrect records.

3. Cannot construct sequence. Candidate C requires a valid handover record (matching previous instance and counter) to cross boundaries. Mismatched handovers (e.g., wrong instance or counter) are rejected. Stale records cannot be applied without correct handover metadata.

4. For A: F2 must change to allow instance boundary crossing. For B: F2 (to permit counter continuation across instances) and F4 (to remove "replay never crosses boundary" assumption) must change. For C: F2 must change to enforce handover record validation for boundary crossing.

5. No. The first transaction (mkfs + publication) writes identical journal bytes regardless of candidate. Candidates only affect replay logic, not the physical journal record format or initial publication.

6. Recommend C. It explicitly validates instance transitions via handover records, preventing stale data application while ensuring continuity without relying on implicit counter assumptions or risking data loss like A or B.
