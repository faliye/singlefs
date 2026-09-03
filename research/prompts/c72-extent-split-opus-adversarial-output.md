# C72 切分收口·opus-adversarial 原样输出（2026-09-03）

## Verdict

**Concession on the primary target (a): I could not find a reachable v1 break.** After a genuine hunt across every operation on the list, the load-bearing settled text **F2** (D4 item 3) forecloses v1 splitting directly, and no other v1-scope operation reintroduces it. F11 survives. My strongest attempted break and why it fails, then the sub-questions.

**Strongest attempt — cross-snapshot partial overwrite (via F8).** F8 says D6 *assumes splitting is the mechanism* that expresses a partial overwrite shared across snapshots — this is the one settled text that reads as if splitting is live. I tried to walk it into a reachable state: S0 snapshots unit U; live version overwrites U's middle. For this to break F11 it must yield one physical unit with two pointers at **different** offsets.

It fails on **F2**: a sub-unit write does whole-unit RMW — *read all of U, re-checksum the WHOLE unit, write the WHOLE unit to a NEW location*. Result: live points at fresh U′ (extent_off 0), snapshot keeps U (extent_off 0). Two independent whole units, **both at offset 0** = sharing-nothing, not splitting. F2 even explicitly forbids the splitting mechanic: splitting would re-checksum only the changed middle and keep old head/tail referenced at a nonzero offset, whereas F2 mandates re-checksumming and rewriting the *whole* unit. So splitting is not merely absent — it is **prohibited** for the partial-overwrite path.

This means F8 and F2 are in **latent contradiction** in the current kb: same operation (cross-snapshot partial overwrite), two different mechanisms (share-and-split vs. copy-whole-unit). That is a real defect — but it is a stale-text defect that **F11 fixes**, not one that breaks F11. (I am *inferring* the supersession direction from dating: F1 is 2026-09-01, newest; F2 2026-08-31; the F6/F7/F8 "will split" texts are older, and F10 shows D4 item 7 consciously re-scoping "the split question." So the newest decisions removed the splitting mechanism; F8's wording lagged.)

**Second attempt — partial-range reflink / sub-unit dedup.** These are the *only* operations that genuinely would mint extent_off != 0 (a pointer into the middle of a shared unit). But (i) nothing in the F-facts places reflink or dedup in the first runnable version — v1 scope is "commit one transaction," and (ii) F4 already rejects cross-file unit sharing to protect D20 unit self-containment, and F2's copy-on-partial-write philosophy means an unaligned share would resolve by copy, not by a sub-unit reference. So these produce splitting only as **future** features — exactly F11's "reserved, disabled" bucket. Concede: out of v1 scope.

I also closed the "is partial overwrite even in v1" branch: if v1 does sub-unit writes, F2 governs them → no split; if v1 does only whole-unit writes → no partial overwrite → no split. **Both branches yield no v1 splitting**, so F11 is robust to that uncertainty.

---

**(a) Any v1 operation producing extent_off != 0?** No, within v1 scope.
- partial overwrite / cross-snapshot partial overwrite → **F2** forces whole-unit RMW to new location, offset 0. Blocked.
- full-unit reflink / full-unit dedup → sharing at offset 0, **not** splitting (splitting requires *different* offsets).
- RAID rebuild/resilver, journal replay, defrag/restripe → relocate or reproduce units, preserve pointer topology; no new offsets minted.
- snapshot delete → removes pointers, no merge in v1.
- tombstone packing → index/metadata, out of scope (question is data extents).
- partial-range reflink / sub-unit dedup → would mint extent_off != 0, but not in v1 scope. Also a partially-*filled* unit (declared len < 32768, **F1**) has ONE pointer at offset 0 — filling is not splitting.

**(b) Does "reserved, disabled in v1" contradict settled text, or is "never exists" forced?** Neither. "Disabled in v1" is consistent with F1/F2/F3. And "never exists" is *contradicted* by F5+F6+F9: compression (F5, day-1 reserved, packing multiple logical extents into one compressed unit) is a concrete future consumer of extent_off, and F9 reserves the width unconditionally. So F11's "reserved feature, off in v1" is the correct framing — same posture as encryption/compression. The one required edit: **F8's D6 text must be reworded** from "splitting is the mechanism" to "v1 pays this cost via whole-unit COW; splitting is a reserved optimization" — otherwise the F8/F2 contradiction persists (kb-discipline: one authoritative record).

**(c) Does re-grounding F7 leave AAD reasoning broken?** The ruling (extent_off out of AAD) is sound, but F11's re-grounding is **understated and carries a wording hazard**. Note encryption itself is not in v1 (F5 references D9 item 10), so there is no AAD in v1 at all — the ruling is *entirely* forward-looking. Grounding it on "extent_off is 0 today" is dangerous: a future editor could read "the value is 0, so including it is harmless/deterministic" and put it back in the AAD, which then breaks splitting-enablement (F7's N-1-fail returns the moment splitting ships). The reason must **lead with the format-contract argument**: AAD composition is a permanent day-1 contract fixed before encryption ships; excluding extent_off is what keeps encryption *and* splitting independently enablable later without an AAD-format break. Not a break of F11, but the re-grounded reason needs that emphasis or it loses its teeth.

**(d) Reserved bits: justified or dead weight?** Justified, and not actually F11's call to make — **F9** (D19 item 3, fixed-width pointers) already settles it, and **F10** confirms reserving them does not change v1's bytes. Real future consumer exists (F5/F6). Removing them would be the risky move (a format break to re-add). Standard forward-compat reservation; keep.

---

**reachable breaks:** none found within v1 scope.

**costs only (edits F11 must carry, not breaks):**
- F8/D6 text reworded to remove the stale "splitting is the mechanism" claim, resolving its latent contradiction with F2.
- F7/D9-item-4 re-grounding must lead with the permanent-AAD-format-contract argument, not "extent_off is 0 today," or a future editor can re-derive "including 0 is harmless" and break splitting-enablement.
- F6/F9 wording changed from "after splitting…" (present-tense mechanism) to "if splitting is enabled…" (reserved feature).

**Single cheapest check that would falsify me:** read the settled D-texts for **reflink/clone_file_range and dedup** — does either land in the first-runnable-version scope AND permit **sub-unit granularity** (a pointer with extent_off != 0)? If yes to both for either one, a v1 operation mints real splitting and F11 is false. (I am inferring from the F-facts that neither is in v1; this is the one place that inference could be wrong, so it is the cheapest place to look.)
