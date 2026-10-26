## E51 Number of Reverse Chain Collision Opportunities — Run (2026-08-30, N=5 Rounds Byte-Identical, 12 Unit Tests / 7 Mutation Rules Caught): **D-Traversal Denominator of E49 (Reverse Chain Width: 32 vs 64) Was Overcounted**; Under Mechanism Reading, 32-Bit Is Sufficient Across All 12 Tiers

[中文](51-反向链的碰撞机会有多少次.md) · **English** · [日本語](51-逆方向チェーンの衝突機会数.ja.md)

**It re-examines the `D-traversal` denominator measurement basis of E49 (Reverse Chain Width: 32 vs 64).**
E49 (Reverse Chain Width: 32 vs 64) counted every time the record checker compares a pair of adjacent records as one collision opportunity,
judging on this basis that 32-bit is sufficient in only 4 out of 12 tiers. **That measurement basis may have been overcounted**, and it was the sole basis for "whether 32-bit is sufficient".

⚠️ **Separate number established instead of modifying E49 (Reverse Chain Width: 32 vs 64)**: what changed is the model, not parameters; per
`.claude/singlefs-ai-sop/rules/test-discipline.md`, this is a new experiment. Artifacts and conclusions of E49 (Reverse Chain Width: 32 vs 64) remain untouched.

### What to Measure

**When does a chain value comparison constitute a collision opportunity.** Both readings modeled, both run:

| Reading | Does Every Comparison Count as Opportunity | Lifetime Opportunities |
|---|---|---|
| **A (used by E49 (Reverse Chain Width: 32 vs 64))**: Every pair of adjacent records compared = one opportunity | All count | Records in ring × lifetime passes |
| **B (mechanism reading)**: Only counts when "the two compared records are actually from different sources" | Only seams count | Incident count × seams per incident + times residual scanned before being overwritten |

**What distinguishes the two is a decidable matter, not preference**:
Chain values of two records from the **same source** are obtained by **the same hash function acting on the same input**
⇒ **constructively identical, probability is 1, containing no randomness** ⇒ impossible to "collide".
Collision is meaningful only when the two compared records **should have been different**.

⇒ E51 (Number of Reverse Chain Collision Opportunities) makes this decidable: **builds a chain checking model, counting two counters: "comparison count" and "comparisons where two sides differ in source"**,
observing whether "comparisons where two sides differ in source" increments only at seams.

**How many times a residual can be scanned** must also be calculated: residual records lie on disk until the ring wraps around to overwrite them
⇒ expected to be scanned by checker `checking frequency × one lap duration` times. One lap duration = records in ring ÷ fsync rate
(fsync rate takes E44 (Host Testing and Cost of Sequence Number Bit Width) host measurement 2785/sec).

### Criteria (Hardcoded Before Run)

| # | Criterion |
|---|---|
| 1 | **Collision opportunity count for same-source comparisons must be constantly 0**—if model outputs non-zero, it treated constructive equality as a probabilistic event |
| 2 | **Different-source comparison count = seam count**, must be **0** on clean logs without incidents |
| 3 | Lifetime opportunities for both readings **given separately**, judging 32 / 64-bit per E49 (Reverse Chain Width: 32 vs 64) rule "sufficient = at least once in lifetime < 1%" |
| 4 | Residual survival time = one lap duration, converted by host fsync rate, **giving absolute values** |

### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: Forge a truly different-source record (residual) ⇒ different-source count must +1 | Does not increment ⇒ model not distinguishing source, **entire round aborted** |
| **Negative control**: Clean log (no holes, no residuals) ⇒ different-source count must be constantly 0 | Non-zero ⇒ **entire round aborted** |
| Both readings yield identical opportunity count | ⇒ This dimension is dead, **entire round aborted** |
| **Under Reading B, 32-bit remains insufficient** | **Faithfully record**. E51 (Number of Reverse Chain Collision Opportunities) does not presuppose 32-bit is sufficient |

### The Half It Cannot Answer

1. **Which fields `previous_hash` covers**—same limitation as E49 (Reverse Chain Width: 32 vs 64).
   **False acceptance rate is 100% when prerequisite is unsatisfied, regardless of bit width or measurement basis.**
2. **Incident frequency** remains a scenario point (10 crashes/day × 10 years = 36,500); this repo lacks telemetry.
3. **The checker does not yet exist**; "it scans full ring each pass" and "how often it runs" are both assumptions.

### Prerequisites

None. Pure arithmetic + minimal model of chain checking.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e51_chain_chances.rs`
  (`cargo run --release --bin e51_chain_chances`),
  raw output `e51-chain-chances-2026-08-30.out`.
- Mutation table `scripts/mutation/tables/e51_chain_chances.tsv`.

### Actual Measurement (2026-08-30, N=5 Rounds Byte-Identical)

#### Quantities 1 & 2: Same-Source Comparisons Do Not Constitute Collision Opportunities; Exactly One Seam

| Log | Comparisons | **Comparisons With Different Sources Across Sides** |
|---|---|---|
| Clean 10 / 100 / 20480 records | 9 / 99 / 20479 | **0 / 0 / 0** |
| 100 records + residual 1 / 6 / 20 records | 100 / 105 / 119 | **1 / 1 / 1** |

⇒ **No matter how long residuals concatenate, comparison across timelines occurs only once**—residuals are internally from the same source, chain values constructively identical.
⇒ **Collision opportunities = seam count, not comparison count.**

#### Quantity 3: One Lap Duration Is Second-Scale; Residuals Do Not Survive Until Next Check

| Ring | Disk Footprint Per Record | Records in Ring | **One Lap Duration** |
|---|---|---|---|
| 10 MiB | 512 B | 20 480 | **7.35 sec** |
| 2 GiB | 4096 B | 524 288 | **188.25 sec** |

(fsync rate takes E44 (Host Testing and Cost of Sequence Number Bit Width) host measurement 2785 times/sec.)
⇒ Weekly checker runs at **1.654×10⁻⁶ times/sec**; expected scans of residual on largest ring is
**3.11×10⁻⁴**—**checker almost never encounters the residual before it is overwritten by wraparound.**

#### Quantity 4: Two Readings Differ by 293×; Opposite Conclusions

Verdict for 32-bit under 12 tiers (ring 3 × record disk footprint 2 × frequency 2):

| Reading | Lifetime Opportunities | 32-Bit Sufficient in How Many Tiers |
|---|---|---|
| **A (used by E49 (Reverse Chain Width: 32 vs 64))**: Every pair compared counts | 1.34×10⁶ ~ 1.53×10¹⁰ | **4 / 12** |
| **B (mechanism reading)**: Only seams + times residual scanned count | **3.650×10⁴ ~ 3.714×10⁴** (almost identically equal to incident count)| **12 / 12**, lifetime at least once **8.50×10⁻⁶** |

⇒ **Under Reading B, 32-bit is sufficient in every tier, with three orders of magnitude margin.**
⚠️ 16-bit remains insufficient under both readings (under Reading B, lifetime at least once **42.7%**)—**that conclusion is unaffected by E51 (Number of Reverse Chain Collision Opportunities).**

### Conclusions

1. **`D-traversal` denominator of E49 (Reverse Chain Width: 32 vs 64) was overcounted**: it treated every adjacent comparison as a collision opportunity,
   whereas **chain values of same-source comparisons are constructively identical without randomness**. Correct denominator is **seam count**.
2. ⇒ **32-bit is sufficient** (lifetime at least once 8.50×10⁻⁶ = 0.00085%, criterion threshold is 1%),
   **the margin bought by 64-bit is unneeded for either responsibility**.
3. ⚠️ **16-bit remains insufficient**, both readings agree.
4. ⚠️ **All three are conditioned on `previous_hash` covering a field that "guarantees divergence between two timelines"**—
   when unsatisfied, false acceptance rate is 100%, independent of bit width or measurement basis.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e51_chain_chances.rs`
  (`cargo run --release --bin e51_chain_chances`),
  raw output `e51-chain-chances-2026-08-30.out`.
  Mutation table `scripts/mutation/tables/e51_chain_chances.tsv` (7 rules, all caught).
- ⚠️ **Mutation testing hit three blind spots, all patched**: chain value missing previous record, formula of Reading B, formula of one lap duration
  (first two in untested locations, third in `main`), plus whether truncation takes effect. **This is analogous to C49 (Universal Assertions Only Verified at Sample Points)**.
- **Incident frequency remains a scenario point** (10 crashes/day × 10 years = 36,500); this repo lacks telemetry.
- **"One incident, one seam" is a model assumption**: if one recovery leaves multiple breakpoints (e.g. recovery itself crashes), seam count > 1.
  E51 (Number of Reverse Chain Collision Opportunities) did not model such nested incidents.
- **Checker does not yet exist**; "how often it runs" and "whether it walks full ring or only deltas" are both assumptions.
- fsync rate takes host consumer NVMe measurement; **enterprise drives with PLP will be 1–2 orders of magnitude higher**
  ⇒ one lap duration shortens proportionally, which only makes Reading B denominator **smaller**, leaving conclusion direction unchanged.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 8 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 11 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30

- Run completed. Not a single criterion modified. Mutation testing revealed three blind spots (chain value depending on previous record, formula of Reading B, formula of one lap duration);
  all 7 caught after extracting functions + adding assertions.
- Initiated. Basis: `D-traversal` denominator of E49 (Reverse Chain Width: 32 vs 64) counted every adjacent comparison as collision opportunity,
  whereas chain values of same-source comparisons are constructively identical; this denominator is the sole basis for "whether 32-bit is sufficient", measurement basis must be audited first.
