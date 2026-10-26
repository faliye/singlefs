## D9 Encryption —— Settled

[中文](09-加密.md) · **English** · [日本語](09-暗号化.ja.md)

D9 (Encryption) governs encryption in this project: whole-volume AEAD direction and plaintext exemption scope, format bits that must be reserved day-1 for encryption, MAC / nonce / algorithm / AAD forms, three hard requirements independent of algorithm selection, capabilities of the keyless side, authentication and rollback protection for system configuration and root, and whether encryption enters the first runnable version. Encryption is an optional operation, not an optional format (.claude/rules/fs-design.md "encryption is an optional operation, not an optional format"): when encryption is disabled, MAC and AAD are not computed; bits reserved for encryption remain present across all volumes, all-zero, and not repurposed for other fields. What it does not govern: field widths and offsets—pointers belong to D19 (Block Pointer Structure & Width Budget) Settled Item 11, root record belongs to D22 (Synthesizing Unit Atomicity) Settled Item 7, system configuration belongs to D22 (Synthesizing Unit Atomicity) Settled Item 9, journal record header belongs to D23 (Journal Role & Format) Settled Item 4, nonce / MAC / algorithm type reserved bits in unit headers belong to D18 (Information Carried in a Block) Settled Item 14 / Settled Item 16; unit type tag domain belongs to registry in D18 (Information Carried in a Block) Settled Item 11. Survey in [prior-art.md](../prior-art.md) section 6. There is no AEAD encryption/decryption operation. Writers use the unencrypted format for reserved fields; readers validate the legal values of that format and reject encryption-enabled configurations. Reserved fields and value validation do not implement encryption.

### Settled Items

| # | Item | Decision |
|---|---|---|
| 1 | **Whether pointers extra store non-keyed ciphertext checksum** | Yes: pointers have two integrity fields: MAC and ciphertext checksum; ciphertext checksum requires no key and is used on all read paths **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **MAC length: 80-bit truncation vs 128-bit** | Full 128 bits, no truncation **Status: Settled.** **Progress: Unimplemented.** |
| 3 | **Algorithm: ChaCha20-Poly1305 vs AES-256-GCM** | Both implemented, algorithm type recorded on disk, runtime selects default by CPU features, not pinned in format; encryption path must be parallelizable **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Nonce carrier: compute on the fly or store per extent** | Store 96-bit nonce per extent, do not compute on the fly **Status: Settled.** **Progress: Unimplemented.** |
| 5 | **Extent of keyless maintenance** | Scrub only: when encryption enabled, keyless side is read-only and detection-only, no moves, no evacuations, no reclamation **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **What goes into AEAD associated data** | Binds five logical identity items, physical location strictly excluded; five items are blind and silent to snapshot dimension, field table owes one reopening—citing these five items must cite this point together **Status: Settled.** **Progress: Unimplemented.** Identity fields have format encoding; AEAD/AAD computation is unimplemented. |
| 7 | **Withdrawn · Number no longer used** | Original issue "which structure keyless reclamation modifies"; after Settled Item 5 settled on scrub-only, keyless side does not move, no code object **Status: Settled.** **Progress: No code object.** |
| 8 | **Authentication and rollback protection for system configuration and root** | fsid enters KDF, system configuration must be authenticated due to nonce watermark, whole-volume rollback unprevented **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **Withdrawn · Number no longer used** | Original issue "fields carried in plaintext mapping layer and its integrity"; after Settled Item 5 settled on scrub-only, no plaintext mapping layer established, no code object **Status: Settled.** **Progress: No code object.** |
| 10 | **Whether encryption enters first runnable version** | Does not enter, only reserves format fields **Status: Settled.** **Progress: Implemented.** |
| 11 | **Direction: whole-volume AEAD** | Whole-volume AEAD, encryption boundary coincides with checksum boundary; all encrypted except system configuration, metadata retains minimal plaintext header, no exceptions **Status: Settled.** **Progress: Unimplemented.** |
| 12 | **Four items format must reserve day-1** | MAC + nonce space (including self-proving units' own), extent offset from original start, system configuration KDF identifier + master key slot + encryption type, per-write algorithm type field **Status: Settled.** **Progress: Implemented.** |
| 13 | **Three hard requirements independent of final selection** | Nonce must not derive from physical location; fixed order: compression -> encryption -> MAC; if MAC truncated, read retry upper bound hardcoded and checked **Status: Settled.** **Progress: Unimplemented.** |
| 14 | **Registry of compression algorithm codes** | 0 = uncompressed, first version writes 0 with compressed length 0; new algorithms append without reusing codes, unrecognized code judges pointer unresolvable **Status: Settled.** **Progress: Implemented.** |
| 15 | **Registry of prerequisite numbers (A series)** | Sole registry for A2 (system configuration is plaintext), A6 (AAD expected values not from plaintext side), A7 (meaning pending inquiry) **Status: Settled.** **Progress: No code object.** |
| 16 | **Source of expected value for each AAD field** | Must only come from reader lookup path or ciphertext-side fields authenticated by higher-level MAC; forbidden from plaintext side or reading back from pointer **Status: Settled.** **Progress: Unimplemented.** |
| 17 | **Linkage with other decisions** | Nonce does not bind physical location; parity computed over ciphertext, replicas are identical ciphertext; birth txg cannot be nonce; fast paths bypassing COW must not bypass encryption **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Whether pointers extra store non-keyed ciphertext checksum

**Settled**: Pointers contain two integrity fields: MAC (requires key, authentication) + ciphertext checksum (no key required, damage detection). All read paths use ciphertext checksum, not only scrub.

**Scope**: Keyless operation only detects, without deciding which copy is good—attackers can craft blocks with internally consistent checksums to deceive the keyless side (Settled Item 6 Proposition 1 does not hold for keyless scrub, which is the same fact). Cost is larger pointers. Where ciphertext checksum resides and its width belong to D19 (Block Pointer Structure & Width Budget) Settled Item 4 (4 bytes in position entry); its decidable form is I-6.8 (ciphertext checksum matches), requiring no key.

**Basis**:

- No experiments: whether to have this field is capability tradeoff without measurable costs—keyless side only performs scrub (Settled Item 5), detecting corruption and reporting bad blocks relying entirely on this keyless field; without it, operation is completely blind when locked.
- ZFS identical form: MAC occupies half of checksum space, other half used to detect on-disk damage whether keys loaded or not (`include/sys/spa.h` comment "MAC occupies half", original in changelog).

**Owed**: None (I-6.8 (ciphertext checksum matches) checker status in [invariants/08-加密不变量.md](../../../crates/singlefs-checker/invariants/08-加密不变量.md), encryption excluded from first version).

#### Settled Item 2: MAC length: 80-bit truncation vs 128-bit

**Settled**: MAC takes full 128 bits (16 bytes), no truncation.

**Scope**: Responsibility of MAC reduces strictly to authentication—keyless corruption detection is borne independently by ciphertext checksum in Settled Item 1, and the 8 bytes saved by truncation buy zero detection capability. Governs MAC in all pointers and self-proving units' own MAC (Settled Item 12 Item 1). Settled Item 13 Hard Requirement 3 (read retry upper bound hardcoded if MAC truncated) carries no load; criterion "MAC length depends on read retry upper bound" does not hold, and C3 (read retries lack hard upper bound) is settled because prerequisite measurement basis was flawed, not because check enforced. When citing dangers of truncation, extrapolating Ferguson 2005 short tag disaster to Poly1305 is forbidden: root cause of that attack was GHASH linearity over GF(2¹²⁸), whereas Poly1305 operates over prime field 2¹³⁰−5 with modular addition rather than XOR, lacking any bit-forcing counterpart (this statement is deductive reasoning without authoritative citation stating immunity).

**Basis**:

- No experiments: value is settled by hard constraints of standards and kernel interfaces, with no measurable costs to compare between sides.
- Three-way argument unanimous (process in changelog).
- SP 800-38D §5.2.1.2 verbatim: "t may be any one of the following five values: 128, 120, 112, 104, or 96", 80 not among them; Linux `chachapoly_setauthsize` requires authsize exactly 16 bytes, `crypto_gcm_check_authsize` rejects 10 bytes ([prior-art.md](../prior-art.md) 6.7).
- RFC 8439 §4: "Tag truncation MUST NOT be done." Conservative prohibition without analysis, not universally enforced (BoringSSL and OpenSSL allow down to 1 byte), treated as moderate evidence; details in changelog.
- Zero production implementations pin cross-request forgery attempt limits (dm-integrity, dm-verity, bcachefs inspected, whole kernel `MaxInvalids` 0 hits); `MaxInvalids` originates in SP 800-38B, while SP 800-38D Appendix C counts all decryption calls.
- Sole production data point supporting 80 bits is bcachefs default: margin squeezed from `bch_extent_crc64` bitfield via manual truncation bypassing kernel AEAD interface, while its own metadata constantly uses 128 bits.
- Poly1305 truncation danger is independent of RFC ban: Wegman-Carter forgery bound is `fℓ/2^τ`; single successful forgery in Poly1305 exposes r (DJB *Cryptography in NaCl* §9 verbatim); secure truncation requires two-stage hashing or extra subkeys, neither present in Poly1305.
- OpenZFS preserves full 128-bit MAC against same conflict (`include/sys/zio.h` `ZIO_DATA_MAC_LEN 16`), preferring to pay "encrypted blocks at most 2 copies".

**Owed**: None.

#### Settled Item 3: Selected at runtime by CPU features, not pinned in format

**Settled**: Algorithm type recorded on disk, runtime selects default depending on AES-NI availability. Both ChaCha20-Poly1305 and AES-256-GCM algorithms must be implemented. Algorithm type field registry: 0 unencrypted, 1 ChaCha20-Poly1305, 2 AES-256-GCM; new algorithms append without reusing codes; readers fail explicitly on unregistered codes without checking against other algorithms (.claude/rules/fs-design.md "per-write algorithm type field" cell; D22 (Synthesizing Unit Atomicity) Settled Item 17 points registry here). Encryption path must be parallelizable.

**Scope**: Determines how defaults are selected, without format additions. Parallelism clause: without AES-NI, AES-256-GCM single core delivers only 244.74 MiB/s, below sequential bandwidth of any modern NVMe (local nvme0n1 3.6 T, sequential bandwidth unbenchmarked individually—deduced by media class). Single-core encryption is not flush critical path; critical path is tree insertion I/O during flush (comparison set correction at end of E17 (Write Buffer Merge Limit)). Superposition of parallel encryption path and D13 (Verification Route) Settled Item 2 (serial commit first) has not been evaluated. Read paths in `crates/` skip algorithm type byte (`PointerHead::read_from`, `JournalRecord::parse`, `RootRecord::parse_slot`), and "explicit failure on unregistered codes" has no evaluating code.

**Basis**:

- E6 (Encryption Algorithm Selection): AES vs ChaCha speed reversal under two CPU tiers, full eight-cell sweep spans multiples—pinning one into format is inevitably wrong on other machines; with AES-NI masked, AES single-core throughput is below NVMe sequential bandwidth, requiring encryption paths to be parallelizable.
- Registry: user settlement, original words in changelog.

**Owed**: C405 (nobody reconciled encryption's need for parallelism with serial-first commit): combining parallelism with serial commit; C406 (the read side does not check the algorithm type and the three compression bytes): pointer header three checks landed in `crates/singlefs-core/src/pointer.rs` `PointerHeadAsRead::judged`; still owes "explicit failure on unregistered codes" check on algorithm type byte at journal record header and root record offset 437.

#### Settled Item 4: Nonce stored per extent, not computed on the fly

**Settled**: Pointer header stores a 96-bit nonce for each logical extent. Does not adopt bcachefs approach of computing on the fly from `bversion`.

**Scope**:

- Buys: ① decryption does not depend on fields **elsewhere** remaining correct => unit self-containment (D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment)) holds even when encryption is enabled—holding verbatim via D18 (Information Carried in a Block) Settled Item 14: full 96-bit nonce has another copy in unit plaintext header, while pointer copy is retained; ② I-6.1 (nonce not reused) becomes a property **decidable against image**—scanning all pointer headers suffices. Pays: **12 bytes / logical extent**; reserved when unencrypted (D19 (Block Pointer Structure & Width Budget) Settled Item 3).
- Does not resolve extent splitting cell: Settled Item 12 Item 2 remains necessary—**if** splitting enabled, N sub-extents after split **share identical ciphertext, identical nonce, identical MAC**, where stored nonce belongs to "original extent", found by sub-extents via offset. **Storing nonce does not mean storing one per sub-extent.** First version lacks this cell: splitting is reserved feature disabled in first version, `extent_off` constantly 0, each unit constantly one nonce and MAC (D4 (Checksum Location) Settled Item 7).

**Basis**:

- No experiments: store vs compute is self-containment policy, carrying no measurable costs—nonce is something pinned the moment it is written; computing on the fly turns "which nonce matches this ciphertext" into a derivation, where every input (bversion, per-crc offset, extent splitting history) is subject to future change, whereas storing it is write-once, read-only thereafter. Parallel basis is .claude/rules/fs-design.md **never sacrifice self-containment to save space**.
- User settlement, original words in changelog.

**Owed**: C184 (reconstruction path cannot acquire full nonce): gate comparison verifying nonce source accessibility on scan rebuild path.

#### Settled Item 5: Scrub only, security first

**Settled**: **When encryption is enabled, the keyless side is read-only and detection-only—no moves, no evacuations, no reclamation.** "Authoritative record point of a block's physical address" **remains inside encrypted structure** (position entry in pointer, D19 (Block Pointer Structure & Width Budget) Settled Item 4), with no plaintext mapping layer and no plaintext reverse index. What keyless operation can achieve is detecting corruption and reporting bad blocks; unable to move anything, fragmentation and bad disk evacuations still require keys.

**Scope**:

- **Knowingly accepted cost**: Hard commitment of D3 (Space Allocation) for "resident, automatic background compaction without user awareness" **does not hold when encryption is enabled**; compaction and evacuation completely stall during locked period, and disk swaps / shrink on nearly-full volumes cannot be executed.
- Two other candidates rejected: "retaining plaintext in index internal nodes" (enables moves and evacuations, but internal nodes carry separator keys, where dirent tree separator keys are file names—leakage surface vastly larger than ZFS, whose indirect blocks are pure pointer arrays without keys), "plaintext reverse index + bucket identifiers" (enables moves, leaks physical layout and reference relationships). Third candidate taken before withdrawal and its history in changelog.
- What structures keyless scrub must read lacks a repository inventory (C303 (the basis of D2 boundary ③ lapsed with the withdrawal of D9 settled item 7)).

**Basis**:

- No experiments: not a preference shift, but **"movable" side proved unreachable**, with no measurable costs to compare. Four settled clauses cannot be simultaneously true: moving a unit means issuing a new stripe (D2 (RAID Stripe Policy) Settled Item 6 verbatim "`w >= 2` is hard lower bound (zero-redundancy stripes forbidden)"), new stripe requires member record per slot (D2 (RAID Stripe Policy) Settled Item 12), and member table verbatim "**must reside in unit payload**", which is ciphertext => **keyless side cannot write member table** => moves cannot succeed; `w >= 2` is hard lower bound => holds for **any** pool. Plaintext mapping layer is cost paid for "movable" (D3 (Space Allocation) Settled Item 16); with goal gone, cost need not be paid.
- Exact bottleneck is not MAC position, but **"whether authoritative record point of block physical address resides in ciphertext"**: moving MAC to block's own plaintext header (as bcachefs does for btree nodes, `extents_format.h` comment `/* Btree pointers don't carry around checksums: */`) relaxes only one point—metadata blocks can be moved verbatim (MAC covers block's own bytes, nonce not derived from physical location), but **address fields** in parent pointers still must change, and parent node payload is ciphertext.
- Three-way argument: six rounds on Settled Item 9 broke candidates on "movable" side one by one (`research/prompts/d9-item9-r1-*` to `d9-item9-r6-*`, `records/2026-09-06-D9项9三轮对抗.md`); user settlement, original words in changelog.
- bcachefs "keyless background compaction paralyzed" is deduction, not quotation: primary sources contain no explicit prose, deduction resting on five verifiable facts (retrieval scope and five facts in changelog).

**Owed**: C303 (the basis of D2 boundary ③ lapsed with the withdrawal of D9 settled item 7): list all structures read by keyless scrub; C23 (keyless scrub cannot tell live slots from stale slots): scanning self-consistent unit cannot tell if it is live; C32 (no gate on whether the keyless side can write bytes): "read-only" has no gate; C33 (no clause on what to do after scrub detects): where reports land, who consumes after unlock; C134 (the scrub watermark itself has no placement or protection); C136 (nobody has marked which invariants can be verified without a key); C166 (Whether Format Reservations Retained After Plaintext Side Cancellation); C167 (Repo-Wide Rescan of Plaintext Mapping Layer References Unperformed).

#### Settled Item 6: AEAD associated data binds logical identity, physical location strictly excluded

**Implementation boundary**: `crates/singlefs-core/src/unit.rs` encodes the five logical identity fields as format preparation only; AEAD/AAD authentication is absent. `crates/singlefs-core/src/recovery.rs` rejects system configurations with encryption enabled. AAD is not computed with encryption disabled, so identity fields or CRC checks do not establish MAC authentication of those fields; the first-version scope remains format reservation only under settled item 10.

**Settled**: **AAD = `(unit_type_tag, tree_id, object_id, object_birth_gen, anchor_offset)`, canonical encoding with fixed length and order. Physical placement, device ID, and any plaintext-side fields strictly excluded.** AAD is an operation computed only when encryption is enabled, uncomputed when disabled (.claude/rules/fs-design.md "encryption is an optional operation, not an optional format"). Source of expected values for each field belongs to Settled Item 16.

| Field | Why It Must Be Inside | Who Provides Expected Value |
|---|---|---|
| **Unit type tag** (value domain is registry of D18 (Information Carried in a Block) Settled Item 11—data extent = Code 1, btree nodes and accounting tree nodes = Code 2, packed record units = Code 3, root record = Code 16; first byte of AAD canonical encoding) | Field sets differ across unit classes. Without domain separation, AAD for two unit classes can serialize to identical byte strings => **type confusion becomes valid MAC pass** | Reader lookup path |
| **Tree ID** | D8 (Core Index Structure) has multiple keyspaces; object numbers assigned independently per keyspace, **inevitably colliding** | Lookup path |
| **Object ID (inode)** | In same tree, offset 0 of file A and offset 0 of file B are identical except object number. Pointer attached to wrong object => read A gets B | Lookup path |
| **Object birth gen (inode generation)** | Preserved per day-1 contract of 5-tuple / AAD (D18 (Information Carried in a Block) Settled Item 3 / Settled Item 11). Not defense-in-depth, but uniqueness itself: D8 (Core Index Structure) Settled Item 6 settles "published (inode ID, object birth gen) unreusable"; numbers reissued by design upon rollback, separated at AAD / nonce layers by this field; second gate when I-6.1 (nonce not reused) fails (crash window / nonce watermark rollback) | Ciphertext side |
| **Anchor offset** = starting offset within object when extent first written, reader computes `key.offset - ptr.extent_off` on the fly | Swapping two extents in same file causes real corruption for database files / VM images | Lookup path + ciphertext-side `extent_off` |

Fields that must never enter AAD, and what each violates:

| Field | What It Violates | Mechanism |
|---|---|---|
| **Physical placement / device ID** | **A7 (meaning pending inquiry) + D19 (Block Pointer Structure & Width Budget)** | D2 (RAID Stripe Policy) settled "N replicas write identical ciphertext N times sharing one nonce", D19 (Block Pointer Structure & Width Budget) settled "MAC once per logical extent, placement once per physical location" => placement enters AAD => N replicas yield N AADs => requires N MACs => overturns segmented pointer structure of D19 (Block Pointer Structure & Width Budget) |
| **Extent offset from original start** | **Settled Item 12 Item 2** | **AAD field composition is permanent day-1 contract**: must be pinned before encryption enabled, modifying later breaks format => criterion judged by "can this field vary per pointer in the future", not "what value it holds today". If splitting enabled, N sub-extents share ciphertext, nonce, MAC while field varies per pointer => N AADs against 1 MAC => **N-1 sub-extents fail verification**, mismatch becoming inevitable under normal operations => excluded. ⚠️ **Re-arguing via 'constantly 0 in first version so harmless to include' forbidden** (trap identified by backward leg): constantly 0 is runtime fact of first version, not format contract; re-including breaks immediately upon enabling splitting |
| **`locality_id`** | **D8 (Core Index Structure) Settled Item 3 verbatim**: "it is a hint, not part of correctness—can be wrong, can be stale" | Burdening field declared as "allowed to be wrong"; proposal in D18 (Information Carried in a Block) Settled Item 3 to add `locality_id` to block header rejected by user, maintaining this row |
| **Any plaintext-side field** (plaintext header flags / length / nonce generation, ciphertext checksum in position entry) | Expected value source criterion (Settled Item 16) | Expected value supplied by read object itself; attacker can modify both sides simultaneously, AEAD provides zero protection |
| **Snapshot ID reader currently resides in** | COW sharing | Block reachable from N snapshots; binding reader snapshot => block verifies in only one snapshot (exclusion holds, gap in snapshot dimension detailed in scope "AAD lacks snapshot dimension") |
| **Reference count / accounting quantity / deadlist membership / current txg** | D5 (Snapshot / Space Accounting Mechanism) | All set properties; modify without block rewriting => requires re-encryption on every change |
| **Parent node physical address** | D18 (Information Carried in a Block) Settled Item 23 | Temporally impossible: COW writes child before parent; value nonexistent when child block written |

Three items format must provide day-1 (missing any prevents checker from judging I-6.3 (MAC matches ciphertext) and I-6.9 (ciphertext passes but MAC fails judged as identity mismatch)):
① **Unit type tag explicit on authoritative side**, forbidden from inferring "which tree it resides in";
② **Reverse index entry identity measurement basis == AAD measurement basis == authoritative tree key measurement basis, identical definition** (three measurement bases mean three definitions);
③ **Logical identity of btree nodes defined**—AAD field list covers data extents only; D18 (Information Carried in a Block) Settled Item 1 settled not implementing lineage rewrite sequences; AAD field set of index nodes must align with Code 2 identity section (tree ID + level + key range), owed in C107 (no placement for index node AAD identity).

**Scope**:

- fsid is not in AAD, serving as input to key derivation (Settled Item 8): cross-volume binding occurs at key layer; ciphertext from another volume under current volume's derived key **cannot be decrypted at all**, earlier and more complete than per-unit AAD mismatches; **not being a field, exempt from A6 (AAD expected values not from plaintext side) constraints**.
- Two unnecessary but zero-cost items: algorithm type, key identifier. Can be placed, but registered as "zero-cost redundancy": residing on ciphertext side, unalterable by attacker, mismatched key inevitably fails. Algorithm type cannot reach most dangerous cell: when field marks "unencrypted", reader does not execute MAC, while AAD is inside MAC; defense line there is I-6.4 (no unencrypted blocks) plus encryption declaration in system configuration. Unit length need not enter AAD: AEAD construction implicitly authenticates it (`crypto/chacha20poly1305.c` feeds `tail.assoclen` / `tail.cryptlen` at conclusion).
- **AAD lacks snapshot dimension**: These five items are blind and silent to snapshot dimension—on snapshot key mismatch, reading data from another writable head leaves MAC green, and "snapshot key rewrite" is one of three implementation bug classes cited in Chain B. Rationale excluding "reader current snapshot ID" holds; what fails is leap "excluding reader snapshot => snapshot dimension omitted entirely from AAD". Only resolution is binding birth snapshot—folding snapshot dimension into on-the-fly nonce is alternative in Settled Item 4, which settled "stored per extent, not computed on the fly". Disposition: "settlement unchanged, gap recorded", field table owes reopening (C63 (Settled AAD Blind to Snapshot Dimension)). Feasible form under tree-per-head: birth (tree, txg) enters AAD, pointer carries declaration, reader verifies against **clone ancestor table** (declared tree on ancestor chain, birth txg <= clone point) before entering MAC; locations for those two fields provided by D19 (Block Pointer Structure & Width Budget) Settled Item 7 (birth tree 8 + birth txg 8), clone ancestor table residing in authenticated structure accessible to reader (C110 (reading block-header consistency for cross-head sharing and metadata classes after encryption)); version swaps within ancestor verification interval unbenchmarked. **Citing these five items must cite this point together.**
- What link AAD actually buys—not "MAC will mismatch":

```
Logical identity q  ──Chain B──  Pointer (head: nonce / MAC)  ──Chain A──  On-disk ciphertext C
```

| Chain | Guaranteed By | Primary Threat |
|---|---|---|
| **Chain A: Pointer <-> Ciphertext block** | **Nonce** (in pointer header on ciphertext side, I-6.1 (nonce not reused) guarantees disk-wide uniqueness; tag is function of nonce) => **No AAD required** | Attacker |
| **Chain B: Logical identity <-> Pointer** | **AAD alone**. Nonce merely asserts "this block pairs with this pointer", not "this pointer belongs under which key" | **Project in-house implementation bugs**—btree split/merge, write buffer seq deduplication, snapshot key rewrite |

"MAC will mismatch" answers only Chain A; value of AAD resides almost entirely in Chain B, whose primary threat is in-house code (E17 (Write Buffer Merge Limit) measured omission of `seq` as this class).

- Scope of applicability for three propositions:

| Proposition | Verdict | Narrowed Scope |
|---|---|---|
| 1 Cannot fool checker | **Holds for keyed checker** | **Fails completely for keyless scrub**: ciphertext checksum resides on plaintext side; attacker moving block alters checksum to be self-consistent. Verbatim accepted scope of Settled Item 1 |
| 2 Cannot escalate privileges | **Holds only after two channels closed individually** | ① "Local repair": if MAC failure with valid ciphertext checksum routes to replica repair, attack converts to data destruction—closed by I-6.9 (ciphertext passes but MAC fails judged as identity mismatch); ② **Nonce watermark in plaintext system configuration can rollback** (A2 (system configuration is plaintext)) => writer reuses nonce => two plaintexts XORed directly under ChaCha20, **AAD provides zero effect**—closed by Settled Item 8 sub-issue 2 |
| 3 Cannot read others' stale data | **Holds within scope of single volume, non-root rollback, non-block-level clone** | Main barrier against reordering is **ciphertext-side nonce, not AAD**; net gain of AAD is degraded protection when I-6.1 (nonce not reused) fails, and catching Chain B implementation bugs. Cross-volume cell transferred to key derivation layer (Settled Item 8) |

**Two cells AAD cannot mend**: **whole-root rollback** (each (nonce, AAD, ciphertext) triple in stale root rolled back to is genuine—any verdict asserting "this is previous version" requires witness outside AEAD); **block-level cloning** (cloned volume shares fsid, key, and nonce space with source volume from same point => both volumes may issue identical nonce for different content post-fork, **representing nonce reuse event incurable by anything inside AAD**, requiring key domain swap during mkfs / clone). "Unable to prevent whole-volume rollback" is not unique to this project, but must be stated openly like fscrypt and dm-verity.

- Corresponding invariants documented in [invariants/08-加密不变量.md](../../../crates/singlefs-checker/invariants/08-加密不变量.md), with checker statuses unimplemented: each field of associated data in I-6.3 (MAC matches ciphertext) computed on the fly by checker from authoritative tree keys, forbidden from reading back from pointers or plaintext side; I-6.9 (ciphertext passes but MAC fails judged as identity mismatch) forbidden from categorizing as damage or triggering replica repair; whitelist in I-6.2 (plaintext header field allowlist) means "allowed in plaintext header", not "usable for verdicts"; I-6.1 (nonce not reused) is anti-bug check, not anti-attack check.

**Basis**:

- E31 (AAD Lacks Snapshot Dimension): Five-item AAD silently passes snapshot key mismatches, binding reader snapshot breaks shared reads, binding birth snapshot and folding into computed nonce pass both criteria—excluding reader snapshot holds, gap and resolution provided.
- E90 (Tree ID into AAD and Cross-Head Sharing): Under tree-per-head, reader taking tree dimension from lookup path rejects legitimate inherited reads, while birth (tree, txg) in AAD plus clone ancestor verification passes both criteria—feasible form for binding birth snapshot.
- Three-way verification: three legs unanimously judge three propositions hold under proposal, narrowing conditions provided by two cloud legs; forward leg caught extent offset from original start ineligible for AAD (material `_d9q6-background.md`, status and four external facts verified by main agent in changelog).
- ext4 inode checksum seed is 3-tuple (fsid, object ID, object birth gen), but items enter distinct layers ([prior-art.md](../prior-art.md) 6.5): intersection is "which three items", layered reading supports Settled Item 8 form; folding into seed verifies guesses without decoding identity, failing to mend load-bearing hole in D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) (data block self-describing behavior relies on explicit fields in D18 (Information Carried in a Block) Settled Item 3).
- fscrypt explicitly disclaims confidentiality and authentication after offline tampering ([prior-art.md](../prior-art.md) 6.1).

**Owed**: C63 (Settled AAD Blind to Snapshot Dimension): reopening AAD field table (binding birth snapshot) and checker mismatch verdicts; C107 (no placement for index node AAD identity); C110 (reading block-header consistency for cross-head sharing and metadata classes after encryption): clone ancestor table placement.

#### Settled Item 7: Withdrawn · Number no longer used

**Settled**: Withdrawn, this number no longer used. Original issue: conflict between Settled Item 5 clause "keyless GC can move or reclaim blocks" and D21 (Boundary Between Authoritative and Derived States) (accounting is authoritative state and encrypted => keyless cannot write accounting), asking which structure keyless "reclamation" modifies. After Settled Item 5 changed to scrub-only, keyless side does not move; sole purpose of Cession Area A was allocating space for keyless side **to move into**, so A **has no code object**; questions of `|A|` size and issuance criteria become objectless. Consequentially invalidated: C135 (structures the keyless side must read are not on the plaintext exemption list), C137 (relocation source and destination indistinguishable on disk), C150 (handover-area membership has no on-disk placement), C151 (keyless relocation necessarily writes an authoritative table under encryption)—all four were offshoots of premise "keyless side must move".

**Scope**: Pre-withdrawal arguments and settlement (Cession Area A U E, Proposition 5, four clauses of that reopening) remain only in changelog, serving as starting point if keyless moves reopened in future, not current design; invalidation of withdrawal reasons does not resurrect withdrawn conclusions (.claude/singlefs-ai-sop/rules/evidence-discipline.md "invalidation of retraction reason does not resurrect retracted conclusions").

**Basis**:

- No experiments: withdrawal is consequence of Settled Item 5 adopting scrub-only, carrying no measurable quantities.
- User settlement, original words in changelog; material `_d9q7-background.md` from three-way verification before withdrawal.

**Owed**: None—five debts originally under this item rewritten 2026-09-20 per keyless scrub form and reassigned under Settled Item 5, details in [checks-owed.md](../checks-owed.md) historical version section on 2026-09-20.

#### Settled Item 8: fsid enters KDF, system configuration authenticated due to nonce watermark, whole-volume rollback announced unprevented

**Settled**:

| Sub-issue | Settlement |
|---|---|
| 1 Is system configuration authenticated | **Must be authenticated, but reason is not fsid.** Once fsid moves into KDF, contradiction with A6 (AAD expected values not from plaintext side) disappears; need for authentication forced by nonce watermark |
| 2 Where nonce watermark resides | **Must not reside in bare plaintext system configuration**, must land on field covered by master-key-derived MAC—keyless side forbidden from writing it |
| 3 Whole-volume rollback | **Announced as known unprevented scope**, matching fscrypt measurement basis. Prevention requires witness outside AEAD, not introduced in this project |

**Scope**:

- Sub-issue 1 solution is not adding authentication to system configuration to rescue fsid, but moving fsid out of AAD: moving ciphertext from another volume into current volume => derived keys differ => **cannot decrypt**; attacker modifying plaintext fsid => whole volume undecryptable, causing denial of service rather than confidentiality breach, **immediately visible**. KDF exists only when encryption enabled, leaving first version without this cross-volume defense line (C74 (No Defense Against Cross-Volume Misdirected Writes in the Period Without Encryption)).
- Sub-issue 2: if watermark of I-6.5 (nonce watermark monotonic) resides in bare plaintext system configuration, rolling it back **bypasses key** (writer reuses nonce, plaintexts XOR directly under ChaCha20, incurable by anything in AAD).
- Sub-issue 3: D22 (Synthesizing Unit Atomicity) noted "any verdict that 'this is previous version' requires external witness, by definition, not an engineering limitation"; here stated explicitly for system configuration: **MAC provides integrity, not freshness**—replayed, genuine historical system configuration has every bit legal and passes signature verification.
- Which fields system configuration MAC covers, algorithm and key specifics are format details landing in D22 (Synthesizing Unit Atomicity) Settled Item 9 (system configuration field table): table contains system configuration own MAC 16, nonce watermark 12, KDF identifier 4, encryption type 1, MAC length declaration 1, master key slot 80 (inlined); coverage scope clause unwritten.

**Basis**:

- No experiments: three sub-issues derived from threat models and settled clauses, carrying no measurable costs.
- Production precedents: ext4 folds `s_uuid` into `s_csum_seed` (computed once at mount), per-object checksum feeds `inum` and `gen`—"volume identity in derivation layer, object identity in per-unit layer" is standard practice ([prior-art.md](../prior-art.md) 6.5); dm-integrity `recalc_sector` resides in plaintext system configuration, where rollback bypasses security, disabled by default upstream when keyed (`drivers/md/dm-integrity.c` `dm_integrity_disable_recalculate`, [prior-art.md](../prior-art.md) 6.6)—this project adopts identical shape.
- Three-way verification: three legs gave three distinct answers; local leg uniquely identified "fsid belongs in KDF rather than AAD"; divergence resolved by examining where ext4 places uuid (material `_d9q8-background.md`, status and four discarded local rounds in changelog).

**Owed**: C74 (No Defense Against Cross-Volume Misdirected Writes in the Period Without Encryption).

#### Settled Item 9: Withdrawn · Number no longer used

**Settled**: Withdrawn, this number no longer used. Original issue: which fields plaintext mapping layer carries and how its integrity is maintained (outside MAC). Settled Item 5 adopted scrub-only: authoritative record point of physical addresses **remains inside encrypted structure**, establishing no plaintext mapping layer and no plaintext reverse index => **no code object**. Consequentially invalidated: C34 (three-way clash on plain text mapping layer states), C129 (plaintext mapping layer during the locked period is authenticated by no one), C138 (plaintext-side leakage surface exceeds what the settled item declares); I-6.10 (deactivated · the plaintext mapping layer no longer exists) similarly objectless. Modifying new-pool file creation bytes: no.

**Scope**: Records of six three-way argument rounds prior to withdrawal (where two candidates collapsed, physical keys for plaintext layer, Candidates D and E, three mutually exclusive points of Cession Area A, leakage deduction chain) remain only in changelog and `records/2026-09-06-D9项9三轮对抗.md`, not current design. Conditional settlement prior to withdrawal "if physical key efficiency and cost-effectiveness higher, settle physical key, else logical key" had condition quantified in E108 (Cost Crossover Point of Plaintext Layer); numbers archived without consumers. E57 (Authoritative Placement for Plaintext-Side Field Set) "split by field" remains valid, but authority for physical address row shifts from plaintext mapping layer to position entry within encrypted structures (D19 (Block Pointer Structure & Width Budget) Settled Item 4). C34 (three-way clash on plain text mapping layer states) was cited as blocker in three places—D19 (Block Pointer Structure & Width Budget), D21 (Boundary Between Authoritative and Derived States) "referrers list" half, D26 (Background Compaction and Placement Reclamation) Settled Item 5 location authority three-way choice; what remains blocking each has been reviewed individually in owed checks table: reviews complete, gate half still owed.

**Basis**:

- No experiments: withdrawal is consequence of Settled Item 5 adopting scrub-only, carrying no measurable quantities.
- User settlement, original words in changelog.

**Owed**: C177 (Three unreviewed sites unlocked post-C34 invalidation).

#### Settled Item 10: Encryption excluded from first runnable version, reserving format fields only

**Settled**: Encryption does not enter the first runnable version, reserving format fields only. Implementation deferred until transaction layer and checker run successfully (.claude/rules/fs-design.md: start from transactions, not features).

**Scope**: Reserved fields are four items of Settled Item 12; when encryption disabled, all-zero, preserved across all volumes, not repurposed (D19 (Block Pointer Structure & Width Budget) Settled Item 3). `crates/` contains no encryption code: write paths write zeroes to these bits (`PointerHead::write_to`, `RootRecord` and `JournalRecord` writes, `SystemConfiguration::to_slot`, unit header 29-byte reservation); core read paths verify unit header 29 bytes all zero (I-2.4 (header checksum coverage)), system configuration encryption type constantly 0, two reader locations (multi-level Code 2 tree internal entries, journal new root segment pointers) verify pointer header MAC / nonce constantly 0, judging non-zero as corruption (Audit A3a); pointer readers outside these two sites, remaining system configuration constants, and journal record own algorithm type / nonce / MAC reservations continue to be skipped by read paths. Pool-level checker verifies unit header 29 bytes all zero and format versions recognized across three unit codes, and verifies traversed pointer headers have MAC / nonce all zero, categorized under I-2.4 (header checksum coverage); system configuration format version and encryption type exceeding reader supported scope categorized under newly established I-7.13 (system configuration pool-level fields within the range the reader accepts).

**Basis**:

- No experiments: sequencing is direct application of .claude/rules/fs-design.md "start from transactions, not features", carrying no measurable quantities.

**Owed**: C166 (Whether Format Reservations Retained After Plaintext Side Cancellation): whether bits reserved for canceled plaintext mapping layer / plaintext reverse index are retained or reclaimed.

#### Settled Item 11: Direction is whole-volume AEAD, encryption boundary coincides with checksum boundary

**Settled**: **Whole-volume AEAD, encryption boundary coincides with checksum boundary: field inlined into pointers in D4 (Checksum Location) is MAC when encryption enabled. All encrypted except system configuration, metadata retains minimal plaintext header.** No exceptions to this statement: journal, accounting tree, allocation records, stripe membership tables **all on ciphertext side**, without asking individually. No per-directory / per-subvolume encryption; metadata not moved to plaintext to enable "keyless background maintenance".

**Scope**: Plaintext header allowed fields divided into two whitelists by unit class (I-6.2 (plaintext header field allowlist); D18 (Information Carried in a Block) Settled Item 13 subsumes Code 1 identity section and declared length into ciphertext); system configuration unencrypted but authenticated (Settled Item 8). Remaining plaintext leakage surface when encryption enabled tracked in comparison table (D18 (Information Carried in a Block) Settled Item 13 "remains in plaintext / what is leaked / verbatim basis"), gate comparison still owed (C173 (plaintext header leakage surface left undefended after the plaintext side was cancelled)).

**Basis**:

- No experiments: placement of encryption boundary is tradeoff between threat model and format, carrying no measurable costs.
- Space for AEAD cleared by D4 (Checksum Location): block-layer encryption (LUKS) cannot authenticate because it has no place for MAC and nonce without breaking block alignment; fscrypt avoids authentication citing ciphertext expansion.
- No per-directory / per-subvolume encryption: bcachefs rejection rationale weighs heavier in this project—fsck requires file size => size must be plaintext; while this project's checker verifies accounting via disk-wide traversal (I-3.1 (Allocated Statistics Match)), and locked subvolumes prevent accounting verification for those portions.
- Metadata not moved to plaintext for keyless background maintenance: ZFS chose that path at cost of indirect blocks entirely in plaintext (leaking pool structure, file sizes, file holes, snapshot layout), authentication degrading to SHA512 aggregation rather than MAC tree, maintaining dual portable / local root MACs at objset layer—series of send/recv corruption bugs in [prior-art.md](../prior-art.md) 6.4 arose from this complexity. Suppresses second candidate of Settled Item 5 (plaintext index internal nodes): same path, worse in this project because internal node separator keys carry semantics.

**Owed**: C173 (plaintext header leakage surface left undefended after the plaintext side was cancelled): gate comparison between plaintext field table and leakage list in D18 (Information Carried in a Block) Settled Item 13, any extra item triggers red.

#### Settled Item 12: Four items format must reserve day-1

**Settled**:

| # | Field | What Happens If Not Reserved |
|---|---|---|
| 1 | MAC + nonce space in pointers; **MAC + nonce + algorithm type on self-proving units themselves** | Adding later modifies pointer width, requiring format restart. **Self-proving units (root record, journal record header, system configuration slots) lack parent pointers**, carrying MAC / nonce / algorithm type on themselves: end of root record has algorithm type 1 + nonce 12 + MAC 16 (D22 (Synthesizing Unit Atomicity) Settled Item 7), journal record header has MAC 16 after fsid, nonce 12 and algorithm type 1 among ten fields (D23 (Journal Role & Format) Settled Item 4), system configuration has MAC 16 / nonce watermark 12 / encryption type 1 in field table (D22 (Synthesizing Unit Atomicity) Settled Item 9). ⚠️ **None of these three enter I-6 (encryption invariants)**: without reservations, enabling encryption leaves two paths—entire path takes plaintext exemption (root record leaks `checkpoint_txg`, tree table pointers, mapping root pointer, tree ID watermark; journal record header leaks `checkpoint_txg`, new root segment pointers, named item positions), or layout change. |
| 2 | Extent "offset from original extent start" | **If** splitting enabled, rewritten split extents cannot avoid both nonce reuse and recomputation ([prior-art.md](../prior-art.md) 6.3) => bits must be reserved day-1. ⚠️ **Splitting is reserved feature disabled in first version, field constantly 0 in first version** (D4 (Checksum Location) Settled Item 7)—reservation rationale is D19 (Block Pointer Structure & Width Budget) Settled Item 3 fixed-width policy reserving bits even when unencrypted, not "used in first version". Width: domain capped by **maximum extent size**, not file size (extent constantly = unit = 32768 => 15 bits suffice), D19 (Block Pointer Structure & Width Budget) Settled Item 11 offset table gives **2 bytes** (offset 32) |
| 3 | System configuration KDF identifier + master key slot + encryption type | Inability to change KDF or enable encryption on existing file system. Lands in D22 (Synthesizing Unit Atomicity) Settled Item 9: KDF identifier 4 bytes, master key slot inlined into system configuration slot 80 bytes, encryption type 1 byte, all zero when unencrypted; internal structure of master key slot settled when encryption enabled |
| 4 | Per-write algorithm type field for data/metadata | Inability to change algorithms; bcachefs treats encryption as checksum type, new data using new algorithms, old data decoded per respective field |

**Scope**: Whether these four are shared across layouts is authoritatively recorded in D12 (Target Media) Settled Item 2: Items 1, 2, 3 **must be shared across all layouts**, Item 4 **may differ per layout**. Not duplicated in D9 (Encryption). When encryption disabled, bits reserved across all volumes, all-zero, not repurposed (.claude/rules/fs-design.md "encryption is an optional operation, not an optional format"); 1 byte algorithm type in journal record header belongs to Item 1 (self-proving unit's own copy).

**Basis**:

- No experiments: what to reserve is format judgment that cannot be added post-hoc, carrying no measurable costs; expanding Item 1 scope to self-proving units was user settlement, original words in changelog.

**Owed**: None (C360 (algorithm type on the byte table's journal row points to the wrong reserved entry) resolved, see owed checks resolved table).

#### Settled Item 13: Three hard requirements independent of final selection

**Settled**:

1. **Nonce must not derive from physical location**, otherwise balance / copygc moving a block requires re-encryption (modifying MAC, requiring keys), multi-replicas would diverge in bytes, and D2 (RAID Stripe Policy) parity could not be computed. **Nonce follows logical identity and resides in pointer.**
2. **Fixed order: compression -> encryption -> MAC.** Reverse is impossible (ciphertext incompressible). Hence ciphertext length leaks plaintext compressibility (Kelsey, FSE 2002), documented in known leakage list rather than ignored. Format carrier: pointer header carries **compression algorithm code 1 byte (offset 29) + compressed length 2 bytes (offset 30)** after algorithm type (offset 28) (D19 (Block Pointer Structure & Width Budget) Settled Item 7 / Settled Item 11), with code registry in Settled Item 14.
3. **If truncated MAC adopted, read retry upper bound must be small hardcoded constant, with checks.** bcachefs 80-bit MAC security argument rested entirely on "MAC mismatch triggers device failure classification and rapid deactivation"—**an implementation behavior, not cryptographic property.** (Settled Item 2 took full 128 bits, so this clause carries no load.)

**Scope**: Three requirements independent of final selection. Leakage list documented in D18 (Information Carried in a Block) Settled Item 13 ("remains in plaintext / what is leaked / verbatim basis"), C173 (plaintext header leakage surface left undefended after the plaintext side was cancelled) still owes gate comparison. Compression fields write 0 in first version (Settled Item 14).

**Basis**:

- No experiments: all three constraints derived from data movement, cryptographic sequencing, and Settled Item 2, carrying no measurable quantities.
- Compression fields: user settlement selected retention (main agent recommended exclusion, compression being optional operation and .claude/rules/fs-design.md requiring parsimony in format branches, while "bits reserved for encryption stay across all volumes" applies only to encryption, not compression), original words in changelog; unverified by three-way, overturnable.

**Owed**: C173 (plaintext header leakage surface left undefended after the plaintext side was cancelled): gate comparison against leakage list (table in D18 (Information Carried in a Block) Settled Item 13).

#### Settled Item 14: Registry of compression algorithm codes

**Settled**:

| Code | Algorithm | First Version |
|---|---|---|
| 0 | Uncompressed | **Writes 0**, compressed length writes 0 |

New algorithms append only, never reusing codes; readers encountering unrecognized codes judge pointer unresolvable (identical policy to algorithm type field, D22 (Synthesizing Unit Atomicity) Settled Item 17). **Compressed length** takes 2 bytes, sufficient for `[0, 32768)` (D4 (Checksum Location) Settled Item 7 settles extent disk space constantly = unit = 32768); on Code 0 constantly 0, non-zero judged pointer corruption.

**Scope**: Sole registry site here (3 bytes for compression in D19 (Block Pointer Structure & Width Budget) Settled Item 7 point here). Governs values of code and compressed length, not write path timing (sequencing belongs to Settled Item 13 Hard Requirement 2). `crates/` writes 0 (`PointerHead::write_to`); read-side checks "unrecognized code judges unresolvable" and "Code 0 with non-zero length judges corruption" landed in `PointerHeadAsRead::judged` (`crates/singlefs-core/src/pointer.rs`); algorithm type bytes in journal record header and root record offset 437 remain unjudged (C406 (the read side does not check the algorithm type and the three compression bytes)).

**Basis**:

- No experiments: registry is convention, carrying no measurable quantities; user decision, original words in changelog.

**Owed**: C406 (the read side does not check the algorithm type and the three compression bytes): read-side compression code and compressed length checks landed in `crates/singlefs-core/src/pointer.rs` `PointerHeadAsRead::judged`; algorithm type bytes in journal record header and root record offset 437 remain unjudged.

#### Settled Item 15: Registry of prerequisite numbers (A series)

**Settled**: A series prerequisite numbers have no definition site in this repository, existing only as references (grep across repository finds zero definition lines); short names deduced from verbatim citations at reference sites, with unresolvable ones explicitly marked as pending inquiry without guessing. This table is their sole registry.

<!-- doc-lint:registry name-col=2 -->

| Prerequisite | Short Name | Content (Source: Verbatim Citation at D9 (Encryption) Reference Site) |
|---|---|---|
| A2 | System configuration is plaintext | Citation verbatim: system configuration is plaintext |
| A6 | AAD expected values not from plaintext side | Expected values for each AAD field strictly forbidden from plaintext side |
| A7 | Meaning pending inquiry | Known only that "physical placement / device ID cannot enter AAD" violates it; original text unfound |

**Scope**: Registers numbers and short names, not rules: active rule for A6 (AAD expected values not from plaintext side) resides in Settled Item 16; A2 (system configuration is plaintext) states system configuration unencrypted, while remaining authenticated (Settled Item 8).

**Basis**:

- No experiments: registry is naming convention, carrying no measurable quantities.

**Owed**: None (original text of A7 (meaning pending inquiry) unfound, registered as "pending inquiry").

#### Settled Item 16: Expected values for each AAD field must only come from lookup path or authenticated ciphertext side

**Settled**:

> **Expected values for each AAD field must only come from ① reader lookup path, or ② ciphertext-side fields authenticated by higher-level MAC.
> Strictly forbidden from plaintext side, and strictly forbidden from reading back from pointer.**

Mechanism: taking expected value from plaintext side => attacker controls AAD input, converting AAD from defense line to free variable; reading back from pointer => checker tests only "pointer internal consistency", leaving **logical identity <-> pointer** link completely unchecked.

**Scope**: Governs each segment of AAD in Settled Item 6; fsid is not AAD field (Settled Item 8), exempt from constraint. Its decidable form is I-6.3 (MAC matches ciphertext): each field of associated data computed on the fly by checker from authoritative tree keys, forbidden from reading back from pointer or plaintext side. Two edge cases lack clauses: whether key section in pointer (carrying birth (tree, txg) per D19 (Block Pointer Structure & Width Budget) Settled Item 7) counts as "lookup path" or "reading back from pointer" (C295 (whether key segment in pointer counts as AAD lookup path)); object ID and anchor offset lack expected value sources after cross-file reflink (C275 (two AAD segments under cross-file reflink lack expected value source)).

**Basis**:

- E90 (Tree ID into AAD and Cross-Head Sharing): Arm reading tree dimension back from pointer passed three remounts silently, failing to detect even forged birth txg—executable proof of expected value source criterion (forbidden from reading back from pointer).
- Three-way verification: two legs reached this independently from different angles, both judging "if this clause fails, all three propositions collapse" (Settled Item 6 round, material `_d9q6-background.md`).

**Owed**: C295 (whether key segment in pointer counts as AAD lookup path); C275 (two AAD segments under cross-file reflink lack expected value source).

#### Settled Item 17: Linkage with other decisions

**Settled**:

| Decision | Linkage Point |
|---|---|
| D1 (Data Mobility / Reverse Index) | Data mobility is hard requirement -> nonce must not bind physical location (Settled Item 13 Hard Requirement 1). **Bucket generation governs invalidation only, not movement** |
| D2 (RAID Stripe Policy) | Parity computed over **ciphertext**; multi-replicas must be byte-identical, writing identical ciphertext N times rather than encrypting N times |
| D3 (Space Allocation) | Keyless can verify but cannot move: scrub viable, moving requires reading/writing index, see Settled Item 5 |
| D4 (Checksum Location) | When encryption enabled, field becomes MAC; both are two values of same field |
| D5 (Snapshot / Space Accounting Mechanism) | **birth txg cannot be nonce**—blocks in same transaction share it, causing nonce reuse |
| D8 (Core Index Structure) | Extent key must reserve version number and offset fields; plaintext header field set of btree nodes must be settled together |
| D10 (True Answer to Random Writes / Fragmentation) | Any fast path bypassing COW **must not bypass encryption**; in-place overwrite causes direct nonce reuse |

**Scope**: Lists only constraints D9 (Encryption) imposes on other decisions; fulfillment on that side belongs to respective decisions. Version number in D8 (Core Index Structure) row does not exist in D8 (Core Index Structure) extent key (its Settled Item 3 is `(locality_id, inode, offset)`), while Settled Item 4 settled storing nonce per extent rather than computing on the fly from key; whether this row should be retained has not been judged.

**Basis**:

- No experiments: each row derived from D9 (Encryption) settled items (nonce follows logical identity, MAC and checksum share field, scrub-only), carrying no measurable quantities.

**Owed**: None.

## Historical Versions

Historical entries for D9 (Encryption) are collected in [decisions-history.md](../decisions-history.md).
