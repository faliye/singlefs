## E2 Key Layout Impact on Recursive Traversal —— Answered by E9 (Key Encoding Impact on Traversal Locality), No Separate Experiment

[中文](02-key布局对递归遍历的影响.md) · **English** · [日本語](02-key布局对递归遍历的影响.ja.md)

**The three tiers that E2 (Key Layout Impact on Recursive Traversal) was to measure are precisely the three real arms of E9 (Key Encoding Impact on Traversal Locality)**: encoding keys by inode, encoding keys by inode + locality prefix,
and encoding keys by full path—on the same directory tree and the same LRU, with E9 (Key Encoding Impact on Traversal Locality) adding one positive control (hash scattering).
⇒ **For conclusions, see E9 (Key Encoding Impact on Traversal Locality); this number is retained and not reclaimed.**

The original failure clause in E2 (Key Layout Impact on Recursive Traversal), "if all three tiers show no difference, mostly the dataset is too small or cache completely hits",
has its answer in E9 (Key Encoding Impact on Traversal Locality): when `cache_leaves ≥ 128`, all four arms are identical (122 device reads each),
and E9 (Key Encoding Impact on Traversal Locality) explicitly marked those two tiers as "untestable" rather than "no difference".

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 3 | Preparation | 2026-09-20 unaffected: the three tiers E2 (Key Layout Impact on Recursive Traversal) was to measure are the three real candidates for key encoding in D8 (Core Index Structure) settled item 3; since E2 (Key Layout Impact on Recursive Traversal) does not stand as a separate experiment and its numbers are provided by E9 (Key Encoding Impact on Traversal Locality), D8 (Core Index Structure) also references E9 (Key Encoding Impact on Traversal Locality) |

## History

E2 (Key Layout Impact on Recursive Traversal) history entries are consolidated in [experiments-history.md](../experiments-history.md).
