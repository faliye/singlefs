1. For K1-A, the central-mapping-tree (t7) remains second-to-last and tree-table (t8) last; for K1-B, t8 becomes second-to-last and instance-table last; for K1-C, instance-table becomes second-to-last and t8 last. The three candidates disagree on which units occupy the second-to-last and last positions. This would be overturned if any candidate's complete write sequence shows different second-to-last or last units than specified.

2. For K1-A, the instance-table must be classified as commit-generated (forced); for K1-B, it is compatible with both commit-generated and user-data classifications (permitted); for K1-C, it must be classified as commit-generated (forced). The three candidates disagree on the required classification. This would be overturned if the instance-table can be allocated via lowest-numbered slot without violating the write order in K1-A or K1-C.

3. For all candidates, the clause’s list does not include the instance-table, so it is written during warm-up regardless; the "zero entries" statement applies only to listed categories, so total units written are the instance-table plus zero. All candidates agree on this consequence. This would be overturned if the instance-table is not written during warm-up or the "zero entries" statement affects its count.

4. For all candidates, the warm-up publish count (2) and first-transaction txg (3) are unaffected by the instance-table’s position, as they are derived solely from disk-ownership geometry. All candidates agree on this consequence. This would be overturned if changing the instance-table’s position alters the warm-up count or txg value.

5. Under both K3-1 and K3-2, the admission formula does not read any clean-shutdown marker bit, so the outcome is identical for the same on-disk state. This would be overturned if the admission outcome differs based on the presence or validity of a clean-shutdown marker.

6. Under K3-1, the warm-up requirement applies unconditionally; under K3-2, a valid clean-shutdown marker could permit skipping warm-up, violating the clause’s "no exception" wording. This would be overturned if warm-up is still required even with a valid marker under K3-2.

7. Under both K3-1 and K3-2, the rollback candidate set is computed identically from on-disk state (root ring and instance table), regardless of marker presence. This would be overturned if the candidate set computation differs based on the clean-shutdown marker.

8. Under K3-1, F is computed after full recovery; under K3-2, if recovery is skipped (due to valid marker), the method for computing F would need to change (e.g., using current state directly). This would be overturned if F computation remains identical even when recovery is skipped under K3-2.
