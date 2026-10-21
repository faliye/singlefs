# Pitfalls checklist

[中文](pitfalls.md) · **English** · [日本語](pitfalls.ja.md)

**Design missteps already proven on existing COW file systems. Usage: review this table every time a design decision is made.**

Items are primarily taken from btrfs—not to single it out, but because its failure modes are the most thoroughly documented,
its community discussions are the most public, and the facts are verifiable. Lessons from other implementations are included just the same.

Descriptions of pitfalls come from public sources and respective official documentation, **not verified in this project**.
The "Countermeasure" column links to corresponding decisions in [decisions.md](decisions.md).

| # | Pitfall | Root cause | Countermeasure |
|---|---|---|---|
| 1 | Absurd ENOSPC (`df` shows free space but writes fail; deleting files also reports out of space) | Two-level allocation + rigid chunk classification | ✅ D3 (space allocation) does not classify; ✅ spurious ENOSPC is forbidden, deleted space must be usable within bounded steps (settled item 9 of D3 (space allocation), settled by user on 2026-09-11), landing form follows settled item 1 of D16 (publish semantics) |
| 2 | RAID5/6 write hole, not recommended for production | Fixed-width stripes require read-modify-write | ✅ D2 (RAID striping policy) variable-width full stripes |
| 3 | extent tree + backref is a complexity black hole and bug hotspot | Reverse index grew organically rather than by design | ✅ D1 (data mobility and reverse index) day-1 design |
| 4 | fsync uses specialized log tree, independent bug source | Second transaction mechanism | ✅ Unified transaction layer (.claude/rules/fs-design.md) |
| 5 | All data crammed into the same tree, same keyspace, same lock | Single index structure | ✅ D8 (core index structure) settled: one btree implementation + multiple independent keyspaces + write buffer / key cache two front-ends |
| 6 | RAID profile can only be set per whole disk, not per file | Profile tied to chunk | ⏸ Candidate: pointer-inline multi-replica |
| 7 | qgroup is slow and chronically inaccurate (+76% commit time / +1347% txn wait / −25% write throughput) | Accounting was retrofitted and relies on backref traversals | ✅ D5 (snapshots and space accounting mechanism) birth txg incremental accounting |
| 8 | `nodatacow` is an escape hatch, not a solution (disabling COW = losing checksums and snapshot consistency) | No true answer to random write fragmentation | ✅ D26 (background compaction and placement reclamation) resident compaction + anti-aging structure stacking, solved in-house without betting on device |
| 9 | Degraded mount writes single-replica chunks, worsening the situation | Degraded behavior not designed | ⏸ Open: degraded semantics need explicit definition |
| 10 | `btrfs check --repair` community itself advises against using it | Recoverability was not a design goal | ✅ Self-describing blocks + checker day-1 |
| 11 | No native encryption | Never entered format, cannot be squeezed in | ✅ D9 (encryption) settled: whole-volume AEAD, MAC / nonce / algorithm type / extent offset reserved day-1 in pointer, encryption itself excluded from v1 (settled item 10) |
| 12 | `df` output is meaningless to users | Implementation details leaked to users | ⏸ Open: space semantics must be user-facing |
| 13 | Native encryption and send/recv replication paths clash chronically (openzfs/zfs#12014 open since 2021-05, still being fixed in 2025) | Authentication structure distorted for raw send: MAC cannot form a Merkle tree, switched to aggregate MAC + dual roots at objset layer | ⏸ No clause governs it (D9 (encryption) fully settled, no item filed for ciphertext replication / raw send): ciphertext replication semantics must be settled at format layer, not as an afterthought |
| 14 | Nonce bound to physical location → moving a block requires re-encryption, background compaction requires keys, multi-replicas have different bytes | Encryption and data mobility were not designed together | ✅ Hard requirement 1 of settled item 13 of D9 (encryption): nonce follows logical identity and lives in pointer |
| 15 | Per-directory encryption forces file size in plaintext (for fsck), leaving gaps in filenames / xattr / inline extent | Encryption granularity finer than checksum granularity | ✅ D9 (encryption): whole-volume encryption, no per-directory |
| 16 | Changing user key does not invalidate old wrapped master key, recoverable via forensics | Key rotation only re-wraps master key without re-encrypting data | ⏸ No clause governs it (D9 (encryption) fully settled, no item filed for key rotation): guarantees of key rotation must be stated to users, cannot let people assume it equals re-encryption |

## Advantages

**Also drawn from existing implementations, but these are proven effective and worthy of direct adoption.**

| Item | Description |
|---|---|
| Self-describing metadata node headers | Node header carries its logical address, generation, level, owning tree, and fsid. This is the sole reason btrfs-rec can rebuild via full disk scan when the tree is corrupted |
| Three-tier feature bits | `incompat` / `compat_ro` / `compat`, evolution path correctly designed |
| Identical code paths for single and multi-device | No special cases for single disk, eliminating a whole category of bugs that only surface under multi-disk |

---
