## E9 Key Encoding Impact on Traversal Locality — Run (2026-08-28 Rerun Finalized)

[中文](09-key编码对遍历局部性的影响.md) · **English** · [日本語](09-走査局所性に対するキー符号化の影響.ja.md)

**Background**: [decisions.md](../decisions.md) D8 (Core Index Structure) settled key = `(locality_id, inode, offset)`,
and `locality_id` **is inherited from the parent directory at creation, intentionally not updated on rename**. E9 (Key Encoding Impact on Traversal Locality) measures how much this choice buys.

**What is measured**: How many device reads a single recursive traversal issues. Four arms share the same directory tree and LRU cache:
`FullPath` (packing level-by-level by path components) / `Inode` (pure creation order) /
`Locality` (`(locality << 40) | inode`) / `Hashed` (**positive control**, hashed scatter).

### Conclusions

**1. The three real arms diverge only under "interleaved creation + small node cache".**
Under depth-first creation (tar extraction pattern), `Inode` and `Locality` are **bit-for-bit identical**—
the 24 files of each directory receive consecutive inodes, and `locality` is `path[0]`,
which under depth-first creation is a monotonic function of inode, so **ordering across both arms is completely identical**.
⇒ Observing a difference requires `interleave` (scrambling directory creation order, proxy for aging).

**2. The ratio depends heavily on node cache size and is not a constant** (`interleave`, 0 renames, median of 5 seeds):

| `cache_leaves` | FullPath | Inode | Locality | Hashed | Inode/Locality |
|---|---|---|---|---|---|
| 4 | 122 | 1364 | 1158 | 28367 | 1.178 |
| **8** | 122 | 1317 | 901 | 28367 | **1.462** |
| 16 | 122 | 1231 | 409 | 28267 | 3.010 |
| 32 | 122 | 1058 | 123 | 23265 | **8.602** |
| 64 | 122 | 738 | 122 | 15073 | 6.049 |
| 128 / 256 | 122 | 122 | 122 | 122 | **1.000** |

⇒ **When the cache fits the entire tree, all four arms are equal (122)**, and the effect vanishes.
**Reporting "1.46×" without reporting the cache tier is equivalent to reporting a single point on a curve.**

**3. The benefit of `locality_id` is a new-tree benefit, which renames consume entirely** (`cache_leaves=8`, median of 5 seeds):

| Rename Count | FullPath | Inode | Locality | Hashed | Inode/Locality | Range |
|---|---|---|---|---|---|---|
| 0 | 122 | 1317 | 901 | 28367 | **1.462** | 3.0% |
| 500 | 122 | 1316 | 1217 | 28367 | 1.081 | 2.8% |
| 2000 | 122 | 1315 | 1288 | 28367 | 1.021 | 0.9% |
| 5000 | 122 | 1311 | 1316 | 28367 | **0.996** | 2.5% |
| 20000 | 122 | 1315 | 1312 | 28367 | 1.002 | 1.9% |

⇒ **After 2000 subtree renames the advantage essentially drops to zero, and after 5000 it is slightly reversed.**
This provides numerical backing to the statement already recorded in D8 (Core Index Structure) that "the benefit of `locality_id` is a new-tree benefit, not a steady-state benefit"—
prior to E9 (Key Encoding Impact on Traversal Locality), that statement had no figures supporting it.

**4. `FullPath` remains constant at 122 across all tiers**, meaning "a single traversal reads only the leaves of that subtree".
It represents the upper bound on traversal locality. ⚠️ However, D8 (Core Index Structure) settled against adopting full-path keys, on the grounds that renaming requires rewriting keys of all descendants—
**E9 (Key Encoding Impact on Traversal Locality) does not measure that cost**, only the traversal side. Using E9 (Key Encoding Impact on Traversal Locality) to argue that "full path is better" is a misuse.

### Positive Control

`Hashed` is far worse than the three real arms in every cell (28367 vs 122–1364, up to 232×).
⇒ The metric indeed measures traversal locality. **If it could not detect the difference between hashed scatter and the other three arms, the first three conclusions would be void.**

⚠️ **The positive control fails at the `cache_leaves ≥ 128` tiers** (all four arms 122)—
that is not "hashing is also great", but **the entire tree fits in cache, so the metric lacks discriminative power at that tier**.
Per `.claude/singlefs-ai-sop/rules/test-discipline.md` "Negative results must be separable from 'code was not reached'",
those two tiers can only be read as "unmeasurable", not "no difference".

### Measurement Basis

Directory tree of 4 levels × 6 subdirectories per level = 1296 leaf directories, 24 files per directory, total 31104 objects;
255 (key,val) slots per leaf; `O_DIRECT`, image placed in temp directory (512 MiB, not committed to repository).
5 seeds (3/7/11/13/17) taking median.
Code `scripts/experiment-apparatus/e9_keylayout.rs`,
invoking `cargo run --release --bin e9-keylayout -- <image> <seed> interleave <cache_leaves> <rename_count>`;
output `e9-keylayout-2026-08-28.out` (25 rounds, 25 completion markers).

⚠️ **Those 25 rounds were assembled by a loop that was not written into this document prior to 2026-08-29**—
anyone rerunning had to infer parameters from the config lines of the artifacts. Added verbatim (`cache_leaves` constant 8):

```bash
truncate -s 512M "$IMG"                       # Image lands in temp directory, not committed to repo
for r in 0 500 2000 5000 20000; do            # Rename count, outer loop
  for s in 3 7 11 13 17; do                   # Seed, inner loop
    cargo run --release --bin e9-keylayout -- "$IMG" "$s" interleave 8 "$r"
  done
done
```

Rerun on 2026-08-29 following this loop yielded **byte-for-byte identical** output to repository artifacts (150 lines, 25 completion markers matching completely).
It is driven by `driver_e9` in `scripts/experiment-apparatus/replay.sh`;
rerunning simply requires passing the identifier of E9 (Key Encoding Impact on Traversal Locality): `bash scripts/experiment-apparatus/replay.sh <E9 (Key Encoding Impact on Traversal Locality) ID>`.

### ⚠️ E9 (Key Encoding Impact on Traversal Locality) Previously Had Zero Absolute Assertions (Added During Adversarial Verification on 2026-08-29)

The seven unit tests pinned only **structural properties** (tree shape, key encoding, rename semantics, `interleave` genuinely changing order),
**none pinning the measured quantity itself**. ⇒ If leaf assignment or geometry were wrong across the board, **all four arms would be wrong together**,
while the ratio `Inode/Locality = 1.46×` would still "hold"—
**and that ratio is precisely the figure cited by D8 (Core Index Structure) when adopting locality prefix.**
This is exactly the pattern called out by `.claude/singlefs-ai-sop/rules/test-discipline.md`
"Only comparing multiple arms against each other cannot detect all arms being wrong together".

Four absolute assertions were added, **each forced out by mutation testing**:

| Assertion | Independent Arithmetic | Mutation It Blocked |
|---|---|---|
| Total leaves == **122** | Slots `(4096−16)/16 = 255`; objects `6⁴×24 = 31104`; `⌈31104/255⌉ = 122` | Slot count changed 16→17 |
| Union of leaves touched by four arms == 122 | `⌈31104/255⌉ = 122` | **Blocked none**—remained unchanged under both `pos/(SLOTS+1)` and "no sorting" corruptions |
| **A leaf cannot hold more than `SLOTS` objects** | 255 is slot count, not fitted value | Production `sorted_position / SLOTS` → `sorted_position / (SLOTS+1)` |
| **Under full path, each directory spans at most 2 leaves** (paired with hashed arm > 10 negative control) | 24 consecutive objects cut at most once by a leaf boundary | Removed `ks.sort_unstable()` |

⚠️ **The first two assertions were added first, but mutation testing proved they could not catch those two mutations**—
changing production `sorted_position / SLOTS` to `sorted_position / (SLOTS+1)` and deleting `keys_with_object_index.sort_unstable()`,
under these corruptions total leaves and union size **happened to remain unchanged**. Only the latter two held firm. **"Added absolute assertions" is not equal to "added correct absolute assertions"; only mutation testing distinguishes.**

⚠️ **Also caught an error made by myself**: the first version of the new assertions **reimplemented leaf assignment inside the test**,
so breaking that production line turned zero tests red—the exact same mistake recorded in this kb under E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units)
("that test computed wrong itself and bypassed the tested code").
The fix was extracting leaf assignment into `assign_leaves()`, shared between test and production.
The mutation table resides in `scripts/mutation/tables/e9_keylayout.tsv`, with all three mutations killed.

**E9 (Key Encoding Impact on Traversal Locality) has 11 unit tests**, pinning these properties (each proved to fail red by mutation testing):
Tree shape matches declaration, full path key keeps each directory contiguous, high bits of locality key are locality,
hash key breaks directory contiguity (the mechanism of positive control itself), rename modifies only path leaving inode/locality untouched,
rename preserves tree shape (directory count unchanged), `interleave` genuinely changes creation order.

⚠️ **One of the tests was rewritten under pressure from mutation testing**: original implementation tested "difference between hash keys of adjacent inodes",
which failed to catch removing hash's first multiplication—because the truly load-bearing property is **breaking directory contiguity**,
not distance between adjacent keys. Rewriting symmetrically to full-path test caught it.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) Settled Item 3 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E9 (Key Encoding Impact on Traversal Locality) are gathered in [experiments-history.md](../experiments-history.md).
