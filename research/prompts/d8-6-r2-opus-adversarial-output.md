# D8 未定项 6 第二轮·反推攻击腿（opus）原样输出（2026-09-03）

## Verdict

**F23 survives as a *shape*; it does not survive as a *freeze candidate*.** The three structural choices — fixed-length + reserved tail (a), no extent pointers (c), separate attribute keyspace — hold up under attack. But the revision's *new* clauses (d), (e), (f) and the presence-bit promise in (a) each open a hole, and two of them are silent-failure holes. Six reachable breaks below.

---

## Strongest break — B1: `locality_id` is now the sole witness for an object's *addressability*, and F23 states no counterpart, no invariant, and no repair rule for it

I accept F14 (the record must store it) and am not re-deriving it. The break is in what F23 did **not** add alongside it.

After F23, the chain "inode → data" is: dirent → inode N → record → `locality_id` L → extent keys `(L, N, off)` (F1). F5 settles that the data unit's plaintext identity is `(type, tree id, object id, birth gen, anchor offset)` — **no locality**. So the count of witnesses for L is still exactly one (F14), and F23 makes that one witness load-bearing for whether *any* of the object's data can be named at all. Three consequences, none addressed:

1. **Extent-tree repair is impossible without a complete inode tree.** A scan finds units tagged `object_id = N` but cannot compute their key's leading component. If N's record is the lost block, those units are unplaceable — the repair must invent a prefix or drop them. This is precisely the last-resort path that F10(a) and F14 exist to support, and F23 leaves it undefined. *(Inference: repair-by-scan reconstructs keys; if unit identity lacks a key component, that component can only be invented.)*
2. **The counterpart F16 implies is the wrong one.** F16 justifies `blocks` against "sum of units found" — a **scan-based** count. A scan finds units on disk whether or not they are *reachable* under `(record.locality, N, *)`. So after any independent reconstruction of the two trees, `blocks = 128` and `units found = 128` agree, the checker is green, and every read returns a hole. The comparison that would catch it is reachability-based, and it is not stated. F23 adopts F16's rationale verbatim in (b) without refining the comparison.
3. **F6's settled words become a data-loss licence.** F6 says `locality_id` is "a hint, not part of correctness — may be wrong, may be stale." Under F23 a stale value silently redirects every lookup to an empty key range. F23 has therefore promoted a field that settled text calls non-correctness into the most correctness-critical field in the record, **without amending F6 and without flagging the amendment**. Any future re-locality / defrag / placement feature is licensed by F6 to move the hint while F1 forbids moving the extent keys.

Note the asymmetry that makes this the *revision's own* hole: (f) carefully supplies an authority rule for the one duplication it inherited from F22 (inode number: key vs record) and misses the duplication that (b)+(d) created (locality: record vs every extent key's prefix) — even though the consequence there is worse. Inode mismatch → EIO, loud. Locality mismatch → zeros, silent, with no unit-side witness by construction.

**Fix, and it must happen at this freeze:** F5 says the five-tuple's individual widths and content are *not* settled. Add `locality_id` to the unit identity (F10(b): "prefer writing it bigger; do not sacrifice self-containment to save space" — the generous choice is mandated here, and F10's space-comparison ban forecloses the only objection). Then state the three-way invariant: `blocks` vs extents reachable under `(record.locality, N, *)` vs units found by scan, with a verdict for each disagreement pattern.

---

## (a) Does storing `locality_id` close rename and rebuild?

**Rename: yes, conceded.** `mv a/f b/f` then `open("b/f")` now resolves: dirent → N → record → L_a → `(L_a, N, *)`. F14's hole is closed for the named path, and (d)'s anti-circularity reasoning is correct.

**Rebuild: no** — see B1. Witness count unchanged at one.

**Can the copies disagree, and what does (f) do?** They can (B1.3), and (f) does nothing: it is scoped to identity. If you naively extend "the KEY is authoritative" to locality, you get the wrong remedy in both directions — a stale record bricks the object with EIO after a defrag bug; not extending it returns zeros. Neither is stated.

Two further defects in (f) itself:

- **The authority named is unavailable exactly where the field was justified.** (b) cites F10(a) ("picked up alone… who am I") for the inode-number field, but F10(a)'s scenario is *scan with no tree*, where there **is no key**. (f) declares the record "a cross-check only" — i.e. non-authoritative — in the one situation where it is the sole witness. (f) needs mode-scoping: key wins when both are present; the record is the sole witness in rebuild.
- **For the corruption class it names, (f)'s remedy never fires.** A bit flip in a stored key does not relocate the record; a B-tree descent for inode 42 simply fails to land on it. Observable: **ENOENT (looks like "file deleted")**, not EIO. The cross-check only fires on a scan or on a search for the corrupted value. So (f) as written does not deliver what F22 demanded — key corruption still presents as a silently missing object. *(Inference: standard B-tree search semantics.)*
- **"must never silently repair" is right; "no repair path at all" is not.** As worded, one flipped bit renders an object permanently EIO even though the correct identity is sitting in the record. Split the rule: forbidden on the kernel I/O path, permitted and specified in the offline checker with logging.

## (b) Is `(inode)` alone the right inode-tree key?

**No — under-reserved, and (d) contradicts (g).**

F9's live candidate is multiple writable heads. Two heads can hold different `size`/`mtime`/`nlink` for the same inode; a keyed tree holds one value per key. So the inode tree needs a head discriminator in the key, or one inode tree per head (a structural change to D8's "one tree per data class", i.e. exactly the conceptual-model move F11 says must be slow). (g) gates the freeze on F9 resolving "at least to *out of v1, N bits reserved in key*" — **but (d) reserves zero bits in this key.** Either (g) means only the data key, in which case the inode key is settled in a way known-incompatible with the leading candidate, or (d) is not actually settled and should say so.

Worse, it is a self-inconsistency with the proposal's own doctrine: (a) invokes F4's "reserve, don't grow later" to justify the record's reserved tail, and (d) does the opposite for the key — the one place where growth is hardest, since key width fixes node layout, sort order, and the position of every existing record. `(head_id, inode)` with `head_id = 0` in v1 is the F4-consistent move.

Also missing: whether `inode_reuse_gen` participates in the key. Scan-time it must (see (e)); live-tree it must not.

The scan-locality argument (inode records of one directory's files scatter by number) is **inadmissible as an attack** — F4's judging rule bars "it's faster" from justifying a format branch, and the same bar applies to me using "it's slower" against one. Listed under costs only.

## (c) Is the presence-bit promise a cheque that can be cashed?

**It bounces in three ways, and the third is a security hole.**

1. **It requires an unstated rule about *unknown* bits.** A reader predating the attribute keyspace does not know the bit means "refuse" — it sees an unknown flag and masks it. So the cashable rule is "**any** unknown bit in `flags` is fatal", not "this bit means refuse". F23 doesn't say that. And once you say it, you can never add an ignorable per-object bit — so `flags` must be **tiered into fatal / read-only-fatal / ignorable ranges now**, before freeze (F12). One undifferentiated `u64` with a single defined bit is not a freezable design.
2. **One bit forces the most severe semantics onto the most benign class.** ACL, quota/project id, and crypto policy have opposite ignorability: ignoring a crypto policy is catastrophic, ignoring a project id is benign. A single all-or-nothing bit makes a quota-only object unreadable to every older reader. That is a reachable evolvability break created entirely by the revision.
3. **The bit is itself a duplicated fact with no authority rule — the disease (f) exists to cure.** Bit set / no entries → an object permanently refused with nothing to refuse over, and no repair path. Bit clear / entries exist → **silent ACL bypass**, i.e. exactly the failure (a) was written to prevent, and undetectable by anything that reads only the inode record. Catching it needs a cross-tree invariant F23 does not state.
4. Unstated dependents: the attribute keyspace's key must be derivable from the inode number alone — inheriting (d)'s head-bits and reuse-gen questions — and unlink has no reverse reference, so attribute entries orphan.

"Refuse the operation" also needs an enumerated operation set. Refuse `unlink` and an old reader cannot unstick the filesystem; refuse `stat` and `ls -l` dies on the whole directory; refuse neither and the bypass in (3) is live.

## (d) Do the v1 defaults survive?

**`locality_id = 0`: survives**, with one coupling — 0 is also what a zeroed region reads as, and the record carries no type tag or checksum (see B5), so "valid v1 record" and "blank" are byte-indistinguishable. Either make 0 the invalid sentinel and use a nonzero root constant, or fix B5.

**`nlink = 1` with the exemption: breaks, two ways, and the wording does not say which.** "an object with no dirent tree yet is exempt until the dirent tree exists" mixes an **object-scoped subject** with a **filesystem-scoped predicate**.

- *Per-object reading*: the invariant becomes "nlink equals dirent count, unless the object has no dirent entries" — **vacuous exactly for orphans**, the only objects it was meant to catch. This is the unfalsifiability the question feared, and it is a live reading of the sentence as written.
- *Whole-filesystem reading*: falsifiable (concede) — but it **expires globally and retroactively**. The moment the dirent tree ships, every v1 object has `nlink = 1` and zero dirents: checker red at best, reclaimed at worst. Every v1 image is condemned by the v2 checker, with nothing in the image marking it as pre-directory.
- Either reading: the switch is **on-disk state the same corruption can flip off**. Zero the dirent tree root and the invariant disables itself across the whole image — a self-disabling check.

Fix: a **monotone** switch (a superblock feature bit, once on always on) plus a per-object `anonymous` flag bit, or reserve a well-known inode for v1's object and define it as named-by-construction. The exemption must not be a predicate over mutable data.

## (e) Is `change_counter` sufficient?

**No. It orders, but it does not date.**

- **No commit anchor.** F17's stated asymmetry is that pointers are *self-dating* (F8: birth txg inline), so a scanner can reject anything newer than the superblock's committed txg. `change_counter` is a private ordinal with no such anchor: nothing on disk says counter 7 for inode 42 was ever committed. A crash after writing version 7 and before publishing leaves 6 and 7 on disk; a scan-rebuild picks 7 and resurrects an **uncommitted** version, potentially referencing blocks the allocator considers free. F17's asymmetry is only half-closed. Fix: order by `(txg, change_counter)` with txg comparable against the superblock, or store the record's txg.
  - *Caveat, honestly flagged*: this collapses if 16 KiB index nodes (F2) already carry a txg in their header **and** the node — not the record — is the scan unit. F23 never says which. See B5.
- **No composition rule with `inode_reuse_gen`.** Ordering must be lexicographic `(inode_reuse_gen, change_counter)`, and "resets to 0 on reuse" vs "never resets" must be stated. If it resets and a scan compares only `change_counter`, the rebuild prefers the **dead** object's record (large counter) over the new object's fresh one (0) — it attaches a deleted file's metadata to a live inode's data.
- **No tie rule.** Equal `(reuse_gen, change_counter)` with differing bytes is corruption and must be red, not "pick either".
- **"Every metadata change" is ambiguous** on the one case that matters operationally: does an atime-only update increment it? That decision determines whether reads dirty metadata.
- **Initial value unstated** (0 vs 1) — and 0 is what blank space reads as.
- **Wrap: conceded unreachable.** 2⁶⁴ at 10⁹/s is ~584 years. The *rule* is still owed pre-freeze (F12), but it is not a break.

## (f) What the first transaction still needs

Format-level items that must exist before any byte is written, none of which appear in F23:

1. **Byte order.**
2. **B5 — record framing.** The table has no type tag, no checksum, no version, no length. So either (i) a 160-byte record is **not findable or verifiable by scan** — which unsupports F14's rebuild story and makes (b)'s citation of F10(a) misattributed (the inode-number field is doing F22 cross-check work, not F10(a) self-description work) — or (ii) the 16 KiB node is the scan unit and carries type/tree-id/generation/checksum, in which case **say so**, because that single sentence also decides (e)'s commit-anchor question and half of B1's repair story. F23 must pick one; both branches change other clauses.
3. **Reserved-bytes rule.** Must-be-zero on write, and *non-zero reserved without a corresponding flags bit ⇒ refuse*. Without it the reserved tail is a silent-misinterpretation vector — the identical failure mode (a) invented the presence bit to prevent.
4. **How the single v1 file is located.** No dirent tree, no name, and the key is `(inode)` — nothing states which inode, so nothing states how a reader finds the one object. A well-known reserved inode or a superblock field. This also interacts with B2's fix.
5. **Unit of `blocks`** (512-byte POSIX units vs D4's 32 KiB units) and its mismatch verdict.
6. Initial `inode_reuse_gen`; inode number 0 reservation; `size` signedness; nanos-out-of-range verdict; `rdev` non-zero on a non-device file verdict; `btime` immutability.
7. **The gate list in (g) is incomplete.** (b) pins its identity widths to D18's five-tuple, which F5 says is unsettled — yet (g) gates the freeze only on F9. Under F12 the gate must include D18.
8. **(b)'s reserved-32 justification is self-contradictory**: its two named dependents are F9 and the attribute keyspace, and (g) routes F9's discriminator out of the record while (a) routes attributes out of the record. Keep the 32 bytes (F10(b), and space is never a criterion per F10) — but the stated reason is wrong, which means the size was not derived from anything.

---

**reachable breaks:**
- **B1** `locality_id` single-witness: unplaceable units on repair, scan-based `blocks` check green while reads return holes, F6's staleness licence becomes a data-loss licence — no authority rule, no invariant, no unit-side counterpart *(fix: add locality to F5's identity, now, while D18 is open)*
- **B2** `nlink` exemption: scope-ambiguous (per-object reading is vacuous for orphans), expires globally at v2 condemning every v1 image, and its switch is corruption-flippable
- **B3** attribute presence bit: needs an unstated "unknown flags are fatal" rule (⇒ `flags` must be tiered now); one bit forces crypto-grade severity on benign attributes; bit-clear/entries-exist is a silent ACL bypass with no authority rule
- **B4** `change_counter`: no commit anchor (scan can resurrect an uncommitted version); no composition rule with `inode_reuse_gen` (can prefer a dead object's record); no tie rule
- **B5** record has no type tag / checksum / version — dilemma: either not scan-findable (unsupports F14, misattributes F10(a)) or the node is the scan unit and F23 must say so
- **B6** (d) reserves zero key bits while (g) gates on "N bits reserved in key"; F4's reserve-don't-grow applied to the record but not to the key
- **(f)** as stated: ENOENT not EIO for the corruption class it names; authority undefined in rebuild mode where the record is the sole witness; no explicit-repair path
- first-transaction gaps: byte order, reserved-bytes rule, how v1's file is located, `blocks` unit, initial values

**costs only:**
- 64-bit `change_counter` wrap (~584 years; rule owed, not a break)
- inode-tree scan locality vs the extent tree's prefix (and F4 bars performance as a format-branch criterion in either direction)
- reserved-tail size, 160 not dividing 16384, sector straddle (moot if nodes are checksummed whole)
- `rdev`/nanos/`size`-signedness checker rules — cheap, but owed under F12
- (b)'s reserved-32 rationale naming two dependents that (a) and (g) exclude — consistency, not correctness

**Cheapest check that would falsify me:** read D18 settled items 3 and 7 verbatim and determine what **"tree id"** denotes. If it already encodes the keyspace/locality prefix rather than merely "which of the six trees", then the unit side *does* carry locality, the record's copy becomes cross-checkable, repair-by-scan can place units without the inode tree, and **B1 collapses from a break to a documentation defect** — which would also weaken my (a). One file, two items. *(Runner-up, nearly as cheap and it settles three findings at once: state whether the scan/rebuild unit is the 16 KiB node or the 160-byte record — that single sentence resolves B5 and the caveat under B4.)*
