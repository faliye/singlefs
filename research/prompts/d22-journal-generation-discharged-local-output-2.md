No counterexample can be constructed. The design ensures that any stale journal record is rejected by either the instance-id and checkpoint_txg lower-bound filter (D23 item 14) or the back-chain hash verification (D23 items 8, 10, 11). Specifically:

- A stale record from a previous instance has a lower instance-id than the current root's instance-id, so the (instance-id, checkpoint_txg) comparison in D23 item 14 rejects it.
- A stale record from the same instance has a lower checkpoint_txg than the root's checkpoint_txg, so the (instance-id, checkpoint_txg) comparison rejects it.
- If a stale record somehow had higher instance-id or checkpoint_txg (which is impossible due to monotonic increases), the back-chain hash would fail because the previous record's header would be overwritten or invalid, violating D23 items 8, 10, and 11.

The single clause that blocks every attempted counterexample is D23 item 14: "重放的下界（2026-09-02，用户定案）：由所选根给出——只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录". This clause ensures only records strictly newer than the root are applied, and no stale record can satisfy this condition while remaining valid.
