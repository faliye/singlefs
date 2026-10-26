# Prior Art Survey

[中文](prior-art.md) · **English** · [日本語](prior-art.ja.md)

**All obtained through document reading or line-by-line source code review; none verified in this project**, nor compiled or run from any project.
Star counts and commit counts reflect values displayed on pages at that time (2026-08-25).

**Source Pinning and Verification Mechanism**: Source code and literature under weight-bearing citations are pinned locally on this machine under `~/code/fs-refs/`—`linux-6.17/fs/bcachefs/` (`linux-6.17.tar.xz` from kernel.org, 260 files), `zfs/` (OpenZFS master, `git log -1` = `58d73c9`, 2026-08-27), and 7 papers under `docs/` (RFC 8439, NIST SP 800-38B / 800-38D, DJB *Cryptography in NaCl*, OSTEP Chapter 45, BetrFS FAST 2017 / 2018, NetApp FAST'20; retrieval methods and sha256 see `scripts/kb/fetch-refs.sh`, extractor `scripts/kb/pdf-text.py`).
**89 weight-bearing citations** are constructed as rerun-capable verbatim assertions (`scripts/kb/verify-citations.sh`, this count is reported by its own final line: `bash scripts/kb/verify-citations.sh` output "Result: N hits", do not count by hand—the lines matching `^(ck|ckn|ckdoc|ckdocn)` in source code also include 11 synthetic assertions run only under `--selftest`); among them, 38 citations check against the local host kernel tree rather than pinned trees (C44 (citations check against host tree instead of pinned tree)), hooked into gate stage "Can external citations still be verified" (`citations` cell in `.claude/gate.d/doc-registries.sh`)—**turning red if source tree is missing**, as "skipping" is precisely the behavior that silently evaporated previous batches of literature.
⚠️ **ZFS On-Disk Specification not collected**: It uses CID double-byte fonts, and `pdf-text.py` extracts garbled text ⇒ per that script's own rules, this counts as **extraction failure**, not "verbatim sentence absent from text"; several citations referencing it were switched to checking against the pinned OpenZFS source tree.
⚠️ For which citations check against non-pinned trees, see [checks-owed.md](checks-owed.md) C44.

## 1. Accounting Model: Pick One of Four

| Model | System | Mechanism | Cost |
|---|---|---|---|
| extent refcount + reverse index traversal | btrfs qgroup | Accurately calculates shared/exclusive | Commit time +76%, transaction wait +1347%, write throughput −25% |
| Attributed to original creator, no global refcount | btrfs squota | `OWNER_REF` inlined; accounting bound 1:1 to extent lifecycle, avoids reverse index | Only 3.4% slower than disabling quotas; cannot calculate shared/exclusive |
| birth txg + deadlist | ZFS | Blocks carry birth generation; on deletion, generation compared against "next older snapshot" → freed directly, or placed into that snapshot's deadlist; deadlists cascade-merged when snapshots destroyed | Accounting maintained incrementally, never traverses |
| accounting key in dedicated btree | bcachefs (2024 rewrite) | Counters themselves are bkeys, extensible via tagged union; percpu + eytzinger index in memory; updates proceed via normal transactions + write buffer | Rewrite broke disk format compatibility, forced upgrade in 6.11 |

**Measurement basis of the numbers**: Percentages for qgroup / squota come from comparative benchmarks published by btrfs officials in the LWN squota article; workloads and hardware were not fully disclosed in that article and **must not be taken as performance expectations for this project**, serving only as qualitative evidence that "this path is impassable."

**Key Takeaway: btrfs conceded defeat itself.** squota is official admission that accurate accounting is intractable.
That this path is impassable requires no further re-verification by this project.

**Three systems distinguish victory on the same point: whether accounting is a by-product of transactions, or post-hoc traversal.**
btrfs qgroup post-hoc traversal → dies; ZFS and rewritten bcachefs transactional incremental maintenance → survives.

Sources:
- [btrfs: simple quotas (LWN)](https://lwn.net/Articles/944371/)
- [bcachefs disk accounting rewrite (LWN)](https://lwn.net/Articles/963570/)
- [Advanced ZFS Dataset Management — Klara](https://klarasystems.com/articles/advanced-zfs-dataset-management/)
- [Disk Space Accounting for ZFS Snapshots — Oracle](https://docs.oracle.com/cd/E19253-01/819-5461/gbcxc/index.html)

## 2. Several Points in bcachefs Worth Studying

Sources: [bcachefs Snapshots Design Document](https://bcachefs.org/Snapshots/),
[Principles of Operation (2026-04-16, PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf).
**Principles of Operation is the newest and most authoritative bcachefs design document, not yet read in full.**

1. **bucket + generation number**. Disks sliced into buckets, each carrying a generation number; pointers store "expected generation", mismatch indicates block reclaimed → data invalidation requires no reverse index updates. Directly alleviates burden on D1 (Data Mobility / Reverse Index).
2. **Snapshot ID in low bits of key**. Snapshot IDs form an independent tree (root `U32_MAX`, allocated downward, parent ID strictly greater than child ID); lookups filtered by ancestor relationship, **without cloning any tree**; O(1) creation, supporting millions of snapshots.
   Cost: deleting snapshots requires traversing all btrees with snapshots (official notes mention desire for bloom filter optimization); read path incurs constant ancestor filtering overhead; write path must handle whiteouts and partial overwrite slicing across snapshots.
3. **Pointers inline multiple replicas**. Extent pointer directly lists N replica locations, enabling per-file replica policies—btrfs uses per-chunk profiles and cannot achieve this.
4. **B-tree nodes are internally log-structured** (large nodes, small updates appended, merged on read, compacted in background).
5. **copygc / rebalance are resident background daemons**, not manual administrative commands.
6. **Reverse indexing was added later**—a cautionary lesson proving how painful it is to retrofit.

**Do not copy engineering pacing**: bcachefs disk format took years of churn before stabilizing, and was removed from kernel mainline maintenance in 2025. Technical aspects serve as references; pacing does not.

## 3. Existing Wheels on the Rust Side

| Project | Language | Progress | Status |
|---|---|---|---|
| [rustutils/btrfsutils](https://github.com/rustutils/btrfsutils) | Rust | disk (incl. RAID5/6 Reed-Solomon) / transaction (write path) / fuse (read-only mount) / stream / mkfs / tune / cli; btrfs check 7 stages, rescue chunk-recover implemented | 0.13.0 (2026-05-14). 13 stars, 464 commits, **0 open issues, 20 open PRs**. Tests are unit + cargo insta snapshots, **no fuzz / xfstests / differential testing visible**. transaction carries warning "Do not use it on filesystems you care about" |
| [adam900710/btrfs-fuse](https://github.com/adam900710/btrfs-fuse) | C | Read-only FUSE, full RAID(0/1/10/5/6), full compression, 4 checksums | Author is btrfs kernel maintainer |
| [btrfs-rec](https://www.lukeshu.com/blog/btrfs-rec.html) | Go | Read-only + damaged tree / chunk mapping reconstruction | Strongest open-source implementation in recovery domain |
| [btrfscue](https://github.com/cblichmann/btrfscue) | Go | Data carving when all superblocks overwritten | Legacy project |
| [GodTamIt/btrfs-diskformat](https://github.com/GodTamIt/btrfs-diskformat) | Rust | Only struct definitions, cannot read disks | 22 commits, stagnant |
| [Dicklesworthstone/frankenfs](https://github.com/Dicklesworthstone/frankenfs) | Rust | Claims ext4+btrfs rewrite, 22 crates, zero unsafe | 21 stars / **8213 commits** / 0 issues. **Ratio strongly suggests agent batch generation**—inferred, code unread |

**Significance to this project**: btrfsutils proves that "building a Rust implementation following btrfs format" can produce a prototype in 3 months, but only because **the format was ready-made and `btrfs inspect-internal dump-tree` served as a free referee**.
Building from scratch, this project has neither—**the lack of an oracle is the largest hidden cost of building from zero**, and must be accounted for in budget; see `.claude/singlefs-ai-sop/rules/test-discipline.md` "Model Differential Testing".

## 4. Kernel Rust Filesystem Abstraction Layer

By 2026, Rust has entered Linux drivers, filesystems, and network components. The filesystem abstraction layer continues evolving, without conclusion yet on "general-purpose vs. serving only simple filesystems."

Source: [Rust for filesystems (LWN)](https://lwn.net/Articles/978738/).
**Little relevance to this project**—D7 (Merging into Linux Mainline) already settled not entering mainline in early years.

## 5. Index Structures (Special Investigation for D8)

**Measurement basis**: Section 5 comes from bcachefs *Principles of Operation* Rev 1.39.2 (2026-08-25 revision) original text, LWN, XFS documentation, and paper abstracts. **None verified in this project, nor cloned/compiled/run from any implementation.**

### 5.1 bcachefs: 28 trees, single btree implementation

> "The entire filesystem is built on two primitives: btrees and buckets.
> All filesystem state is a key-value pair in a btree. ... There are no separate
> inode tables, bitmap allocators, or per-inode extent trees."

It is **not** "switching data structures per access pattern", but **divided into 28 independent keyspaces sharing a single btree implementation**, using two frontends to solve two difficult access patterns:

| Frontend | Problem solved | Where used |
|---|---|---|
| **key cache** | Same batch of keys repeatedly read/written, whole tree lookup too expensive | alloc btree (extent updates modify parent bucket allocation info) |
| **write buffer** | High-frequency small updates, batch application more economical | backpointers / LRU / accounting |

Among the 28 trees, **10 are write-buffered**: `lru`, `need_discard`, `backpointers`, `deleted_inodes`, `accounting`, `stripe_backpointers`, and four `reconcile_*` trees.
⚠️ 28 / 10 are figures from Principles of Operation Rev 1.39.2; checking linux-6.17 source `BCH_BTREE_IDS()` on 2026-08-29 shows 21 trees with 5 using write buffers; text of D8 (Core Index Structure) takes precedence.

**Cost of write buffer (explicit in original text)**: Updates are **unordered and eventually consistent**—btree cannot observe pending updates before next flush; flush sorts by key position, **discarding temporal order**; batched insertion into btree node order after deduplication.
> "this sort-merge-sweep is inherently single-threaded, making its efficiency
> critical for multithreaded workloads."

Namely: write buffer suppresses write amplification, but **trades for a single-threaded flush bottleneck**.

### 5.2 Log-structured btree nodes

Nodes are 128K–256K (default 256K), **internally log-structured**: new keys appended, not inserted in sorted order.
A node contains multiple bsets—the one left by previous full rewrite plus subsequent appends.
Lookup merges results of all bsets; memory periodically re-sorts so a node **holds at most ~3 bsets** (currently written one ≤ 8×16K, remainder scaling geometrically).

Three notable properties:

1. **Writes are sequential** (beneficial for SSDs and HDDs).
2. **Nodes can be written to disk under read locks**—because writes are strictly append-only. This keeps lock hold times short, and writeback does not block readers. **A tangible concurrency benefit, not merely I/O.**
3. **Deletion relies strictly on whiteouts** (inserting tombstones)—items cannot be deleted from bsets already flushed to disk. A mandatory consequence of log-structured nodes permeating all deletion-related code.

Keys within bsets are **packed**: each bset has a format descriptor recording fields identical across all keys, storing compressed, **saving 30–50% metadata space**.

**Why others cannot use large nodes**: Other COW filesystems store metadata in Linux page cache, constrained to 4K nodes; bcachefs manages its own btree node cache, enabling 128K–256K nodes with independent eviction tuning.
> "At petabyte scale with spinning disks, deep btrees with 4K nodes mean more
> seeks per lookup and level-2 nodes too large to stay resident under page
> cache pressure."

**Significance to this project**: D7 settled not entering mainline, so **this constraint does not apply to us at all**—large nodes come free without paying costs beyond self-managed caching.

### 5.3 Bucket allocator: from scanning thread to btree

- Buckets are 512K–2M; buckets and data pointers both carry **generation numbers**; incrementing generation invalidates all old pointers, **avoiding locating or deleting any reverse references**.
- Original text notes generation value extends beyond cache invalidation: makes **bootstrapping allocation info** feasible during journal replay, and **makes certain previously unrecoverable corruptions repairable**. Directly echoes our "reconstructibility" design objective.
- Buckets are **never overwritten**; sealed once sequentially filled → copying GC mandatory.
  **copyGC reserves 8% (default, configurable 5%–20%)**—a ready-made baseline for D3 (Space Allocation) "reserves must not be excessive."
- A **fragmentation LRU btree** tracks bucket fullness; copygc targets the most fragmented buckets without scanning the entire alloc btree.
- **Allocator originally used memory bucket array + dedicated scanning thread** maintaining freelist/discard/eviction; later completely replaced with btrees (freespace / discard / LRU), **scanning thread entirely removed**.
  Reasoning: transactional btree code is "far easier to debug and reason about", and old threads **consumed significant CPU** when nearing capacity—edge cases vanished after switching to btrees.
- Buckets naturally map to zones on SMR / zoned devices.

### 5.4 Accounting (Independent confirmation for D5)

> "Accounting counters are maintained by triggers as keys are committed,
> not by walking the btrees."

`check_allocations` **rebuilds accounting via full traversal** to distinguish "counter stale" from "key genuinely present."
Shares identical design with I-3.1 (Allocated Statistics Match) in `crates/singlefs-checker/invariants/05-空间记账.md`: **runtime incremental maintenance, checker verifies via traversal**.

### 5.5 Erasure coding: second known solution to write hole

bcachefs **encodes across whole buckets in the background** rather than slicing foreground writes into stripes:
Foreground performs normal replicated writes → background collects candidate buckets → computes Reed-Solomon → writes parity → atomically updates all pointing extents (dropping redundant replicas, attaching parity pointers).

> "This approach avoids the write hole entirely: parity is computed once for
> immutable data, and the extent updates are atomic btree operations."

**The second known good solution alongside ZFS RAID-Z "variable-width stripes, full-stripe writes every time"**, with entirely different philosophy: one eliminates partial stripe writes, the other computes parity only once over immutable data.
Cost: while live data remains in a stripe, its buckets cannot be reused, requiring copygc to evacuate entire stripes.

### 5.6 Bε-tree / BetrFS: rename cost in full-path indexing reduced to subtree depth in 2018

Bε-tree is a B-tree variant where **internal nodes reserve ε fraction of space to buffer messages**; writes are flushed downward in amortized batches, yielding theoretical write I/O below B+tree. BetrFS was the first kernel filesystem using it.

BetrFS uses **full-path indexing** (key is full path), making scans extremely fast.
Renaming requires replacing the prefix of every key in the subtree; **naive implementation must rewrite the entire subtree**—FAST 2018 paper calls this the "Achilles' heel" of full-path indexing.

**BetrFS 0.4 (FAST 2018) uses lifted Bε-trees to reduce this cost from "subtree size" to "subtree depth."**
Paper verbatim: with key lifting, "the number of paths that need to be modified in a range rename also changes from being proportional to the size of the subtree to the depth of the subtree"; complexity section further states the number of nodes split or merged in tree surgery is "at most proportional to the height of the tree".

Two-step mechanism:

1. **Tree surgery**—Splitting nodes on left and right margins of subtree that mix keys to be moved and kept (reusing ordinary node split code, splitting by pivot key rather than midpoint), then moving pointers.
2. **Batched key update**—Replacing prefixes of all keys in subtree en masse.
   Accomplished **free by lifting invariant**: in lifted Bε-trees, key value is defined by "path traversed to reach node"; once pointer move lands, all keys in subtree have their prefixes implicitly replaced.

**Requirements of lifting on key encoding**: Keys must make "same subtree shares prefix" hold at tree structural level.
BetrFS 0.4 additionally adjusted key format so `memcmp` suffices for key comparisons.
A healing mechanism preserves the invariant of "internal nodes have 4–16 children" to bound tree height.

**Self-admitted costs in paper**: BetrFS 0.4 degrades relative to 0.3 when **node splitting and merging fall on critical path**, where extra lifting computation outweighs benefits; among all patterns, only **sequential write** was adversely affected.
Rename performance claimed as "competitive with indirection-based file systems for a range of sizes"—**competitive within a size range, not winning across all sizes**.

Measurement basis: FAST 2018 paper + TOS 2018 extended version, **unverified in this project**.
Impact on key layout see [decisions.md](decisions.md) D8.

Sources:
- [The Full Path to Full-Path Indexing (FAST 2018)](https://www.usenix.org/system/files/conference/fast18/fast18-zhan.pdf)
- [Efficient Directory Mutations in a Full-Path-Indexed File System (TOS 2018)](https://www.cs.unc.edu/~porter/pubs/tos18.pdf)

### 5.7 XFS reverse mapping: cost magnitude and another justification

rmapbt is a b+tree per AG recording `(physical block, owner, offset, block count)`.

**Space cost (single-source baseline, unverified)**: On a 50TB filesystem, bare `mkfs.xfs` takes ~52GB metadata (~1%); enabling `rmapbt=1` increases to 675GB (~1.35%); further enabling `reflink=1` reaches 981GB (~1.96%).
⚠️ Figures from personal blog mkfs output, **representing metadata scale at format time rather than actual usage**, serving only as magnitude reference.

**More importantly, provides the second justification for D1**: XFS documentation states reverse mapping "is an essential feature for repairing filesystems online because the secondary copy can be used to rebuild damaged primary metadata".

Namely: **reverse indexing is not merely for balance/defrag; it is the prerequisite making online repair possible**—when primary index corrupts, reverse index reconstructs it. Directly aligns with Pitfall 10 (repair tools nobody dares run).

Sources:
- [bcachefs: Principles of Operation, Rev 1.39.2 (2026-08-25)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)
- [bcachefs Allocator](https://bcachefs.org/Allocator/)
- [xfs: add reverse mapping support (LWN)](https://lwn.net/Articles/695290/)
- [mkfs.xfs(8)](https://www.man7.org/linux/man-pages/man8/mkfs.xfs.8.html)
- [Notes on XFS: metadata size (k1024.org)](https://k1024.org/posts/2019/2019-02-08-notes-on-xfs/)
- [BetrFS: A Right-Optimized Write-Optimized File System (FAST'15)](https://www.usenix.org/system/files/conference/fast15/fast15-paper-jannen_william.pdf)
- [The Full Path to Full-Path Indexing (FAST'18)](https://oscarlab.github.io/papers/fast18-betrfs.pdf)
- [Closing the B-tree vs LSM-tree Write Amplification Gap (FAST'22)](https://arxiv.org/abs/2107.13987)

## 6. Encryption (D9 (Encryption) Special Investigation)

**Measurement basis**: Sourced verbatim from bcachefs Encryption design document (page marked last edited 2025-11-02),
OpenZFS `module/os/linux/zfs/zio_crypt.c` top design comments (master branch, retrieved 2026-08-26),
Linux fscrypt kernel documentation, OpenZFS `zfs-load-key(8)` manual page, and OpenZFS public issues.
**All obtained from documentation and source code reading, unverified in this project, with none compiled or executed.**

### 6.1 Three encryption boundaries

| Boundary | What is encrypted | Tamper detection | Capabilities without key | Primary cost |
|---|---|---|---|---|
| Block layer (LUKS / dm-crypt) | Entire device | **No** (no place to store MAC) | Filesystem functions as usual | Cannot separate keys per file/subvolume; nowhere to place MAC and nonce |
| File level (fscrypt: ext4 / f2fs / UBIFS) | File content + filenames | **No** (uses XTS / CBC-CTS / Adiantum) | All metadata readable | Size, permissions, timestamps, xattrs, hole positions entirely plaintext |
| Pointer-inlined AEAD (ZFS / bcachefs) | See 6.2, scopes differ | Yes, MAC stored inside pointer | See 6.2, significant divergence | Format must reserve MAC / nonce fields from day one |

bcachefs documentation gives a structural reason sentencing block-layer encryption:
> "at the block level there's nowhere to store MACs or nonces without causing painful
> alignment problems."

fscrypt kernel documentation explicitly states it does not authenticate:
> "fscrypt is not guaranteed to protect confidentiality or authenticity if an attacker
> is able to manipulate the filesystem offline prior to an authorized user later
> accessing the filesystem."
The stated reason is "ciphertext expansion is difficult to handle"—ciphertext being longer than plaintext cannot fit in block-aligned filesystems.

**Significance to this project**: Having checksums accompany pointers ([decisions.md](decisions.md) D4 (Checksum Location)) inherently frees up space for the MAC and nonce. AEAD comes essentially free for this project, unlike block-aligned filesystems.

### 6.2 ZFS and bcachefs part ways on the same question: can maintenance still be done without the key?

| | OpenZFS | bcachefs |
|---|---|---|
| Encryption scope | **Only level 0 blocks**; indirect blocks (blkptr tree) unencrypted | Everything encrypted except system configuration |
| Authentication structure | Indirect layers aggregate with SHA512(set of lower MACs); objset layer has two 256-bit MACs (portable / local) | Poly1305 MAC per block, chain of trust reaching up to system configuration |
| Algorithms | AES-CCM / AES-GCM, 128/192/256-bit; authentication auxiliary uses SHA512-HMAC | ChaCha20 + Poly1305, using cryptographic primitives directly per RFC 7539 |
| MAC location | Trailing 128 bits of blk_cksum | Metadata: 128-bit field in plaintext header; Data: follows pointer |
| Nonce location | 96-bit IV: 64 bits in DVA[2] second word, 32 bits in blk_fill upper 32 bits | Metadata: sequence number in plaintext header; Data: 96-bit version number of key plus derived quantity (see 6.3) |
| Capabilities without key | **scrub / resilver / rename / delete / raw send** | Virtually none—metadata itself requires decryption to read |
| Information leaked | Pool structure, dataset & snapshot names, dataset properties, file sizes, file holes, dedup table | Only leaks "this location is a btree node/journal entry" and its length |

**The price ZFS pays for "scrub without key" is stated bluntly in source comments**:
The leading 128 bits of `blk_cksum` are reserved for a **truncated checksum of ciphertext** specifically for scrub, while only the trailing 128 bits store the MAC.
The reason the MAC tree is not a Merkle tree is that raw send does not transmit indirect blocks, and the receiver without the master key cannot recalculate:
> "Ideally, the cleanest solution would be to maintain a tree of authentication MACs
> going up the bp tree. However, this presents a problem for raw sends."
ZIL and dnodes each have sections that "must remain plaintext" (for claiming and scrubbing),
and these plaintext parts are authenticated via AEAD's AAD mechanism.

**ZFS key hierarchy**: Master key does not directly encrypt data; 64-bit salt + HKDF derives working keys,
**rotating salt every 400 million blocks**—comments calculate via birthday paradox that under a 96-bit IV space, keeping collision probability below 1/10^12 limits writes to 398,065,730 blocks. Master key is stored encrypted on disk with the user's wrapping key, so changing user passphrase does not require rewriting data.

**bcachefs justification for rejecting per-directory encryption** (verbatim): fsck needs file sizes, so sizes must be plaintext; otherwise that portion of fsck must be deferred until keys are provided; filenames, xattrs, inline extents each bring similar troubles.
> "With whole filesystem encryption, it's much easier to say what is and isn't encrypted."

**bcachefs MAC truncation argument** (noteworthy as it wagers security on an implementation constraint):
Data extent Poly1305 MAC is truncated to **80 bits** by default, saving 8 bytes/extent (single-replica extent 32 bytes vs 40 bytes). The rationale is that filesystem attacker forgery attempts are bounded—a MAC mismatch flags device failure and quickly halts it:
> "attacker gets a very small (on the order of 10) attempts to forge a particular extent"
Scenarios not accepting this premise (e.g. network-attached access, paths known to corrupt data) require 128-bit MACs.

**Swappable algorithms**: bcachefs structures encryption as "a new checksum type"; each data and metadata write carries an algorithm field, so switching algorithms does not require rewriting legacy data.

### 6.3 Nonces: the real requirement encryption places on COW formats

**This is the most useful section in the bcachefs design document for this project**, recording a field forcibly added to the format.

- Keys carry a 96-bit version number, enforced when encryption is enabled, with each new extent acquiring a new number.
- When an extent is partially overwritten or split and rewritten: **cannot change number** (version number carries other semantics), **nor reuse number** (no longer writing identical data).
- "Fast-forwarding keystream to the location of live data in original extent" fails with compression: dropping 4K before compression != output of compressing before dropping 4K.
- Final solution: **add an "offset from original extent start" field to extents**,
  with nonce synthesized from version number + offset + current size + compression algorithm:
  > "offset + current size uniquely determines the uncompressed data, so, offset +
  > current size + compression function will uniquely determine the compressed output."

**Nonce reuse after crash**: Data is written to disk with a version number before the extent is inserted into the btree; thus after an unclean shutdown, numbers exist that "were used but cannot be found in the btree". bcachefs documentation notes the strict solution is recording a high-water mark in the journal header (numbers below which may have been used), **marking it as not yet implemented**; current implementation scans the existing maximum number and takes the next number at 64k above it.

### 6.4 Encryption pitfalls encountered (public records)

| Pitfall | Record | Measurement Basis |
|---|---|---|
| ZFS native encryption + send/recv prolonged corruption | openzfs/zfs#12014 open since 2021-05; Phoronix reported 2024-02-12; openzfs-docs#494 proposed doc warning; 2025-05-31 claimed fixing two non-raw send bugs in 2.2.8 / 2.3.3 | Public issues and news reports, **unverified in this project**. Qualitative evidence that encryption permeates replication/snapshot paths, not performance or probability data |
| Compression before encryption inevitably leaks length | Kelsey, *Compression and Information Leakage of Plaintext*, FSE 2002 | Order cannot be inverted (ciphertext cannot be compressed); leakage must be accepted if compression is chosen |
| dedup + encryption leaks equality | ZFS uses **plaintext HMAC** as salt+IV (first 64 bits salt, next 96 bits IV) to ensure identical plaintext yields identical ciphertext | Source comments admit: attackers observe identical blocks, "but dedup already leaks this". Also: under encryption, dedup only functions within clone family |
| Changing user key != old key invalidated | `zfs-load-key(8)`: Old wrapped master key remains "accessible via forensic analysis for an indeterminate length of time" | Manual page verbatim |

Sources:
- [bcachefs Encryption Design Document](https://bcachefs.org/Encryption/)
- [OpenZFS `zio_crypt.c` Design Comments](https://github.com/openzfs/zfs/blob/master/module/os/linux/zfs/zio_crypt.c)
- [Linux fscrypt Kernel Documentation](https://www.kernel.org/doc/html/latest/filesystems/fscrypt.html)
- [zfs-load-key(8)](https://openzfs.github.io/openzfs-docs/man/master/8/zfs-load-key.8.html)
- [OpenZFS Encryption Documentation Warning Proposal openzfs-docs#494](https://github.com/openzfs/openzfs-docs/issues/494)
- [Phoronix: OpenZFS Native Encryption Use Raises Data Corruption Concerns (2024-02-12)](https://www.phoronix.com/news/OpenZFS-Encrypt-Corrupt)
- [Kelsey, Compression and Information Leakage of Plaintext (FSE 2002)](https://link.springer.com/chapter/10.1007/3-540-45661-9_21)

### 6.5 Where volume identity resides: both ext4 and fscrypt place it in the derivation layer, not per-unit input

**Inspected line by line in local kernel tree on 2026-08-29, used to finalize Item 8 of D9 (Encryption).**

| Implementation | Which layer volume and file identity enters | What is carried in per-unit input | Provenance source |
|---|---|---|---|
| ext4 checksum | s_uuid **folded once at mount time** into s_csum_seed | Only inum, gen, block content; s_uuid never appears | fs/ext4/super.c ext4_init_metadata_csum; fs/ext4/ialloc.c __ext4_new_inode; fs/ext4/bitmap.c ext4_inode_bitmap_csum_verify; fs/ext4/super.c ext4_group_desc_csum |
| fscrypt key | Per-file nonce **generates random bytes once at inode creation**, persisted in on-disk context | Derived per-file key | fs/crypto/keysetup.c fscrypt_prepare_new_inode get_random_bytes(nonce, FSCRYPT_FILE_NONCE_SIZE); fs/crypto/fscrypt_private.h struct fscrypt_context_v2 |

⚠️ **fscrypt is not "generating a new salt on every mount"**—this project previously paraphrased it this way in drafts, which was erroneous.
Nonce is generated once at **inode creation** and written into on-disk context; subsequent mounts read back the same value.
If "generate new salt every mount" were actually implemented, blocks written during the previous mount could never be decrypted.

**Significance to this project**: D9 (Encryption) moves fsid from AAD into KDF, matching the layering of both systems—volume/file identity enters derivation layer, while object identity enters per-unit layer. See [decisions.md](decisions.md) D9 (Encryption) Item 8.

### 6.6 High-water mark fields in plaintext system configuration can be rolled back: an active demonstration of failure modes

**Inspected in local kernel tree on 2026-08-29.** `recalc_sector` in dm-integrity lives inside plaintext system configuration; its documentation states:

> "legacy_recalculate — Allow recalculating of volumes with HMAC keys. This is disabled by
> default for security reasons - an attacker could modify the volume, set recalc_sector to
> zero, and the kernel would not detect the modification."
> (`Documentation/admin-guide/device-mapper/dm-integrity.rst`)

Upstream resolution was not "include high-water mark in MAC", but rather **disabling that path by default when keys are present**, requiring explicit `legacy_recalculate` to enable (`dm_integrity_disable_recalculate()` in `drivers/md/dm-integrity.c`).

**Significance to this project**: The nonce high-water mark in I-6.5 shares the same structure, with even harder consequences—rolling it back is not merely "missing one tamper detection", but **stream cipher nonce reuse => two plaintexts directly XORed**.
=> D9 (Encryption) Item 8 establishes: "high-water mark must be covered by MAC derived from master key; non-keyed side is forbidden from modifying it".

### 6.7 AEAD tag length: kernel permits truncation down to 32 bits

`crypto_gcm_check_authsize()` accepts 4 / 8 / 12 / 13 / 14 / 15 / 16 bytes
(`crypto_gcm_check_authsize()` in `include/crypto/gcm.h`, inspected 2026-08-29).
=> "MAC strictly 128-bit without truncation" is **this project's own design constraint, not imposed by cryptography or kernel**; the space cost it incurs (16 bytes per unit) should be accounted on this measurement basis rather than deemed "having no choice".

## 7. Survey of Academic Results in the Last Decade (2012–2026)

**"7. Survey of Academic Results in the Last Decade (2012–2026)" only collects results that "could overturn premises of existing decisions", not general literature reviews.** Each item notes which decision it impacts.
All sourced from published papers and project documentation, **unverified in this project**.

### 7.1 Small random writes: write-optimized indexing turns it into a non-issue (impacts D10)

Measurements of BetrFS 0.4 vs mainstream filesystems on identical small random write microbenchmark (FAST 2018 Paper Table 2):

| Filesystem | 256K 4-byte random writes duration (seconds, lower is better) |
|---|---|
| BetrFS 0.4 | 4.9 ± 0.3 |
| BetrFS 0.3 | 5.9 ± 0.1 |
| nilfs2 | 2013.1 ± 19.1 |
| btrfs | 2147.5 ± 7.4 |
| ext4 | 2776.0 ± 40.2 |
| xfs | 2835.7 ± 67.9 |
| zfs | 3288.9 ± 394.7 |

**Measurement basis**: 256K 4-byte random writes totaling only 1 MiB written—this is an **extreme microbenchmark**, not a realistic database or VM image workload. Paper self-reports "up to 600 times faster".

**⚠️ The durability model of these numbers differs from singlefs; must read this section before extrapolating (verified 2026-08-26).**
FAST 2018 Paper Sections 2.1 & 4.x verbatim:

> Crash consistency is ensured using **logical logging**, i.e., by logging the inserts,
> deletes, etc, performed on the tree. **Internal operations, such as node splits, flushes,
> etc, are not logged.** Nodes are written to disk using **copy-on-write**. At a periodic
> checkpoint (**every 5 seconds**), all dirty nodes are written to disk and the log can be trimmed.

> BetrFS ensures crash consistency by keeping a **redo log** of pending messages and applying
> messages to nodes copy-on-write. Crash recovery simply replays the redo log since the last checkpoint.

This yields three critical facts for this project:

| Fact | Meaning for singlefs |
|---|---|
| BetrFS **is** COW—nodes written using copy-on-write | "Those numbers come from a non-COW system" does not hold; cannot dismiss Bε on this basis |
| BetrFS durability relies on **redo log + 5-second periodic checkpoints** | That 5-second window amortizes vast amounts of COW node rewrites. ⚠️ **This project makes no commitment to "publish a new root every transaction"**—`litmus/commit-publish.litmus` only fixes the internal order of a single publication event (write block content -> smp_wmb -> write generation), without mentioning publication frequency; `decisions.md` / `CLAUDE.md` / `.claude/rules/fs-design.md` contain no such assertion (verified via grep 2026-08-26). Publication frequency is an **undecided** matter |
| Paper **contains no checksums**; no Merkle chain where parent stores child checksum | singlefs D4 inlined checksums + D9 AEAD MAC/nonce add fixed per-rewrite costs that BetrFS does not pay; crossover point will shift, magnitude unknown |

**What these numbers demonstrate**: Message buffering reduces "how many times a key is rewritten" from O(tree height) to O(tree height / B).
**What they do not demonstrate**: How much gain remains given Merkle checksum chains + AEAD, and at a specific publication frequency.
Remaining gain must be measured independently via [experiments.md](experiments.md) E7 (Offline Index Harness).

⚠️ **Publication frequency is a free variable here, not a constant.** If this project adopts checkpoint semantics (accumulating dirty nodes in memory, batching writes periodically), the benefit of "a small write needing not immediately rewrite the full leaf-to-root path" is **provided free by checkpoint batching**, without requiring a persistent message buffer in the format.
How much of BetrFS's number stems from message buffering vs 5-second checkpointing **was never isolated in the paper**.
E7 control groups must therefore separate two "no message buffer" baselines: zero batching, and checkpoint batching without format-level message buffers. Conflating the two makes message buffering appear more essential than it actually is.

Mechanism converts small writes into Bε-tree batched message flushes rather than in-place block updates or disabling COW.

Significance to D10: btrfs `nodatacow` trades COW for random write performance at the cost of losing checksums and snapshot consistency; this path absorbs random writes into the index message layer, leaving checksums and snapshots intact.
**This is the sole candidate direction for D10 backed by empirical measurement figures.**

**Another verified fact**: All versions of BetrFS use **two** Bε-trees—one for file data and one for file system metadata (Paper Section 2.2 verbatim: "All versions of BetrFS use two Bε-trees: one for file data and one for file system metadata").
This aligns with D8 "multiple independent keyspaces", merely with far fewer trees (bcachefs has 28).

### 7.2 Aging is not inevitable; can be eliminated at the structural level (impacts D3, D10)

FAST 2017 "File Systems Fated for Senescence? Nonsense, Says Science!" aged filesystems via **repeated git checkout of Linux kernel source**, benchmarking btrfs / ext4 / F2FS / XFS / ZFS and BetrFS.
Conclusion: **Aged BetrFS even outperformed unaged performance of other filesystems (except btrfs)**.
Another figure cited as motivation in FAST 2018: git version control workloads can degrade ext4 scan performance by up to **15x**.

For this project: D3 commits to resident automatic defragmentation, reasoned by "no classification => fragmentation pressure concentrates on a single allocator".
This result presents a second path: **aging can be absorbed by the index structure itself, rather than relying entirely on post-hoc defragmentation**.
Costs of both paths are measurable; criteria see [experiments.md](experiments.md) E10.

### 7.3 Index structure: parameter space is far larger than "switching to another tree" (impacts D8)

| Result | Conclusion | Measurement Basis |
|---|---|---|
| SplinterDB (ATC 2020) | STBε-tree = Bε-tree plus LSM size-tiering, structured as "tree of trees": trunk nodes point to B-tree branches, each with a quotient filter. Relative to RocksDB: 6–10x faster inserts, 2–2.6x faster point lookups, 2x write amplification reduction | Abstract figures, targeting NVMe SSD |
| FAST 2022 "Closing the B+-tree vs. LSM-tree Write Amplification Gap" | On hardware with built-in transparent compression, Bε-tree write amplification is 28 (8KB pages) / 36 (16KB pages); under identical conditions RocksDB is 38, WiredTiger 268 (8KB) / 530 (16KB) | 500GB dataset, 32B records, 4 client threads |
| WiscKey key-value separation | Large values moved out of index, keeping only pointers in index; compaction moves only keys and metadata, reducing write amplification by up to two orders of magnitude. Adopted industrially as RocksDB BlobDB, Titan, TerarkDB | Paper + industrial implementations |
| HashFS (FAST 2021 "Rethinking File Mapping for Persistent Memory") | File mapping alone accounts for **70%** of IO path; hash-based mapping improves LevelDB YCSB throughput by up to 45% over page cache extent trees | **Persistent memory context**; reference only if not targeting PM |
| TableFS (ATC 2013) | All metadata packed into LSM (LevelDB); metadata-intensive workloads run 50%–1000% faster than ext4 / XFS / btrfs | FUSE implementation, author admits inefficient implementation, still wins |

Significance to D8: D8 establishes "single btree implementation + multiple keyspaces + write buffer / key cache frontends".
These results do not negate that direction, but significantly broaden **tunable parameters within the same structural framework**:
Bε buffer ratio, size-tiering, key-value separation, filter types—all represent parameters within one btree, rather than "adding another tree type".
This does not conflict with D8 (Core Index Structure) "the accounting tree, central mapping tree, tree table, instance table and inode tree are still built on one btree"; rather it complements it (the allocation record tree and the extent tree have been changed to allow heterogeneous implementations per settled item 14 and are excluded).

### 7.4 Crash consistency verification: the "no oracle" premise needs reassessment (impacts verification plan)

[CLAUDE.md](../../CLAUDE.md) records that the largest hidden cost of this project is that "a design from scratch has no reference implementation to compare against".
The last decade has produced at least five categories of ready-made methods, each giving something different:

| Method | Exemplar | What it provides | Cost / Premise |
|---|---|---|---|
| Push-button formal verification | Yggdrasil (OSDI 2016, Best Paper) | **crash refinement** correctness: disk states an implementation produces (including crashes) must be a subset of specification-permitted states. Z3 automated verification, **no manual proof annotations** | Implementation must be written in symbolically executable form; paper reports verification under 1 minute after decoupling logical and physical representations |
| Bounded black-box crash testing | CrashMonkey + ACE (OSDI 2018) | Exhaustive parameter exploration of workloads, checking recovery after each crash point. 65 machines run for 2 days: reproduced 24 known bugs, found 10 new bugs (7 in kernel since 2014); **also discovered a crash-consistency bug in formally verified FSCQ** | Bounded space only; says nothing about outside bounds |
| PM specialized framework | Chipmunk (EuroSys 2023 Best Paper candidate) | Similar method applied to persistent memory filesystems, finding 23 new bugs across five persistent-memory (PM) filesystems, including mount failures and rename atomicity violations | Tailored to PM semantics |
| Model checking + differential testing | Metis (FAST 2024) | Combines model checking state exploration with differential testing, **requiring no abstract model**: uses another filesystem as reference, comparing, operation by operation, abstract states, syscall returns, and errno. Accompanied by **RefFS**—a small, fast reference filesystem designed to accelerate model checking and improve bug reproducibility | Requires trusted reference implementation |
| Machine-checked proofs for concurrency + crash | GoJournal (OSDI 2021) / DaisyNFS (OSDI 2022) | GoJournal is verified concurrent crash-safe journaling (Coq / Perennial 2.0), **uncovering a severe concurrency bug missed by extensive unit tests**; DaisyNFS sits atop GoTxn, verifying NFS operations **sequentially** using Dafny | High proof cost |

**Two most direct takeaways for this project**:

1. **RefFS in Metis demonstrates that a "reference implementation" need not be someone else's mature filesystem; it can be a small implementation written specifically for differential testing.**
   This matches the model differential testing in `.claude/singlefs-ai-sop/rules/test-discipline.md`, but Metis connects it with state-space enumeration rather than solely feeding random operation sequences.
2. **DaisyNFS layering is isomorphic to `.claude/rules/fs-design.md` "one transaction layer shared by all structures"**:
   The transaction layer resolves concurrency and crashes once, leaving upper layers to verify only sequential correctness.
   This provides an independent justification outside this project for that discipline.

### 7.5 Commit protocols: ordering doesn't necessarily have to be bought with flushes (impacts transaction layer design)

OptFS (SOSP 2013) transforms ext4 journaling into an optimistic protocol via three techniques:
**transaction checksums** (including data blocks in checksums, discarding whole transactions on recovery mismatch) replacing commit ordering;
**asynchronous durability notification** replacing blocking flushes;
separating ordering and durability into two APIs (`osync()` / `dsync()`). Paper reports up to an order-of-magnitude improvement on certain workloads.

NoFS (FAST 2012, "Consistency Without Ordering") goes further:
Each data block carries a **backpointer**; read paths verify consistency via backpointers, **writing completely without ordering**, with a formalized model.

For this project: `litmus/commit-publish.litmus` fixes the internal ordering of a publication event ("write block contents first, then system configuration"), independent of publication frequency.
These two results demonstrate alternative ordering forms with differing trade-offs.
⚠️ **Backpointers in NoFS differ from D1 reverse index**:
NoFS uses an on-block field for "each block self-authenticating ownership"; D1 is a queryable index of "who references this physical block".
This project's rule that "metadata blocks must be self-describing" (`.claude/rules/fs-design.md`) approaches NoFS's field.

### 7.6 Hardware interfaces: block interface is no longer the sole abstraction layer (impacts D2, D3, D8 node size)

| Interface | Status | Significance to this project |
|---|---|---|
| ZNS (Zoned Namespace) | Sequential writes, host-managed GC; adopted in scale deployments for cost efficiency | Zone size serves as input to D8 rule on "selecting node size per device characteristics" |
| NVMe FDP (Flexible Data Placement) | Merged Google SmartFTL and Meta Direct Placement Mode proposals, bridging cost/benefit gap between ZNS and standard SSDs. **Requires no sequential writes, backward-compatible, applications run unmodified** | More likely target layer for this project than ZNS: does not force sequential writes, avoiding conflict with COW allocation flexibility |

Measurement basis: NVM Express and SNIA public materials, 2025 arXiv papers, **unverified in this project**.
Common thread is delegating placement and GC to host—**overlapping with COW filesystems managing allocation themselves**.
Whether this overlap yields efficiency gains or double-management penalties requires empirical benchmarking.

### 7.7 Status of Rust in kernel file systems (impacts engineering route, not on-disk format)

| Fact | Source and Date |
|---|---|
| Bento (FAST 2021): Safe Rust kernel filesystem with errors sandboxed. Performance comparable to native VFS ext4, 7x (git clone) to 90x (Filebench) faster than FUSE; **supports live upgrades with ~10ms service interruption**, replacing running filesystem without application disruption | Paper and project page |
| Greg Kroah-Hartman statement (2026-07-15): Rust in kernel is no longer experimental but a permanent fixture; graphics subsystem will only accept new drivers in Rust | Secondary reporting, **original email unverified** |
| Rust binder driver merged into Linux 6.18; PuzzleFS listed in Rust for Linux project roster | Secondary reporting |

For D7 decision to remain out-of-tree: Bento's live upgrade capability is noteworthy—turning "whether filesystems belong in-kernel" from a binary choice into a triad (userspace / kernel C module / live-upgradable kernel Rust module).

### 7.8 bcachefs status update (reinforces D7 justification, does not alter conclusion)

| Fact | Date |
|---|---|
| bcachefs marked externally maintained in Linux 6.17; **code removed from mainline in 6.18** | 2025–2026, public reporting |
| Distributed as DKMS module, identical to OpenZFS on Linux; patches submitted to independent mailing list | Concurrent period |
| bcachefs-tools v1.38.6 **drops experimental tag** | 2026-06-17 |

For D7 decision to remain out-of-tree: this shifts evidence from "removal from mainline was a social rather than technical failure" to "**project continues regular development post-removal, shedding experimental tag afterwards**"—out-of-tree distribution is a viable path, not merely a fallback.

### 7.10 bcachefs official documentation addendum (inspected 2026-08-26, Principles of Operation Rev 1.39.2+2202696)

| Fact | Original text / Measurement basis | Significance to this project |
|---|---|---|
| **bucket gen scope restricted to cached data** | §1.5: "we can reuse a bucket with **cached data** in it without finding and deleting all the data pointers by incrementing the generation number" | [decisions.md](decisions.md) D1: Manages invalidation only, not relocation |
| **gen wraparound mechanism superseded by reverse index** | State list need_gc_gens: "Legacy state, retained for compatibility … **now effectively unused since the invalidate worker uses backpointers instead of generation bumping**" | Aligns with D1 selection of day-1 reverse index |
| **copygc reserves 8%, configurable 5–20%** | §1.5 & §9.1.10. Also "Normal writes cannot dip into this reserve", write latency rises above ~90% capacity | D3 space reservation via admission control has comparable magnitude: **others carve out 8% unusable**, this project uses dynamic admission; difference must be documented |
| **Encryption is all-or-nothing, only enabled at mkfs** | §2.1.2: "Encryption is all-or-nothing at the filesystem level: all data and metadata except the superblock is encrypted, and all data and metadata is authenticated … **Encryption can only be enabled at format time; it cannot be added to an existing filesystem**" | Corroborates D9 direction; note singlefs D9 reserves "system configuration KDF identifier + master key slot" aiming to **enable encryption on existing filesystems**, an intentional divergence |
| ⚠️ **Data written with nocow is stored unencrypted on encrypted filesystems** | §2.1.2: "Data written with the nocow option is stored **unencrypted**, even on an encrypted filesystem. **This is a hard design incompatibility, not a policy choice**: ChaCha20 requires a unique nonce per (key, plaintext), and bcachefs stores the nonce externally alongside each data pointer" | **Empirical evidence for D9 and D10 linkage**: This project mandates "any fast path bypassing COW must never bypass encryption; in-place overwrite causes immediate nonce reuse". bcachefs faced identical constraint and chose **leaving nocow data unencrypted**—singlefs rejects this trade-off, forbidding in-place overwrite for any D10 candidate |
| **Must unlock before mount** | §4.2.3: "the passphrase is requested once the devices have been found and **before any attempt to mount**"; FAQ: mounting locked returns Required key not available | One premise supporting D9 Item 5 derivation |

**What was not found (explicitly stated, not extrapolated)**: No primary documentation states in prose that "copygc / rebalance / scrub / fsck cannot run without key".
"Unkeyed background maintenance paralysis" is derived from the table facts above; **it is an inference, not a citation**, as noted in decisions.md D9.

Sources: [bcachefs Principles of Operation (PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf),
[bcachefs.org/Encryption](https://bcachefs.org/Encryption/),
[bcachefs.org/FAQ](https://bcachefs.org/FAQ/),
[The Programmer's Guide to bcache](https://bcache.evilpiepirate.org/BcacheGuide/),
[Kent Overstreet: Bcachefs - encryption, fsck, and more (LWN Mirror)](https://lwn.net/Articles/717378/),
[Bringing bcachefs to the mainline (LWN, LSFMM 2022)](https://lwn.net/Articles/895266/).

### 7.11 bcachefs accounting verification has a self-invalidation flaw (verified line-by-line in Linux v6.17 source, 2026-08-26)

**This section records an active invalidation bug, not speculation.**

bcachefs `bch2_check_allocations()` reconstructs accounting via full traversal and compares against runtime counters using `memcmp`—**identical in form to invariant I-3.1 in [invariants/05-空间记账.md](../../crates/singlefs-checker/invariants/05-空间记账.md)**. However, both paths share the exact same addition/subtraction code:

```c
this_cpu_add(e->v[gc][i], a.v->d[i]);        // fs/bcachefs/disk_accounting.h, bch2_accounting_mem_mod_locked()
```

`gc` is the sole differentiator—runtime writes `v[0]`, GC reconstruction writes `v[1]`.
Upstream `__trigger_extent()` similarly forks on a single `bool gc = flags & BTREE_TRIGGER_gc;`.
Comments in `btree_gc.c` admit the reconstruction side "is not idempotent; we'll calculate the wrong result if we run it multiple times"—**reconstruction is itself a chain of += operations**.

| What it catches | What it misses |
|---|---|
| Incremental application divergence: dropped update, concurrency loss, un-replayed crash | **Flawed addition/subtraction semantics**: sign reversed on a branch causes runtime and reconstruction to err to the exact same value, passing memcmp completely green |

**Significance to this project**: `.claude/singlefs-ai-sop/rules/evidence-discipline.md` mandates that "verification paths do not share the same code, sampling, or tools". This section records a concrete real-world instance where violating that rule produced blind spots. Remediation see [checks-owed.md](checks-owed.md) C12.

**Incidental accounting format facts** (verified concurrently): Accounting tree is bound at the **format level** to `BTREE_IS_write_buffer` in `bcachefs_format.h` `BCH_BTREE_IDS()`; `disk_accounting_format.h` verbatim: "Unlike with other key types, **updates are _deltas_**"; merge function is `dst->v.d[i] += src.v->d[i]`; write buffer explicitly forks accounting vs regular keys across insertion, flush deduplication, and btree writeback; because Δ is non-idempotent, it invents a **total bversion ordering** to determine whether a Δ has been applied, and strictly forbids accounting key flushes until journal replay completes.
**These items form the checklist of costs incurred by placing increments in buffers.**

Source: Linux mainline v6.17 `fs/bcachefs/` (`disk_accounting.{c,h}`, `disk_accounting_format.h`, `btree_write_buffer.{c,h}`, `btree_trans_commit.c`, `btree_gc.c`, `buckets.c`, `bcachefs_format.h`), fetched and compared line-by-line.

## 8. Root ring slot width and racing long-running consumers against publications: how active implementations handle them

**Inspected verbatim at local checkout `~/code/fs-refs` on 2026-09-08. Unverified in this project.**
These answer D2 (RAID Stripe Policy) Item 15 (general criterion for objects with differing lifecycles) and [checks-owed.md](checks-owed.md) C214 (Traversal-hop window margin is unguarded).

### 8.1 ZFS: root ring slot width scales with device granularity, capped at 8 KiB; widening slot width reduces slot count rather than taking more disk space

Source: `zfs/include/sys/vdev_impl.h` macros from `VDEV_UBERBLOCK_RING` to `VDEV_UBERBLOCK_SIZE(vd)` and `zfs/include/sys/uberblock_impl.h` `UBERBLOCK_SHIFT`, verbatim:

```c
#define	VDEV_UBERBLOCK_RING	(128 << 10)
/* The largest uberblock we support is 8k. */
#define	MAX_UBERBLOCK_SHIFT (13)
#define	VDEV_UBERBLOCK_SHIFT(vd)	\
	MIN(MAX((vd)->vdev_top->vdev_ashift, UBERBLOCK_SHIFT), \
	    MAX_UBERBLOCK_SHIFT)
#define	VDEV_UBERBLOCK_COUNT(vd)	\
	(VDEV_UBERBLOCK_RING >> VDEV_UBERBLOCK_SHIFT(vd))
```

`UBERBLOCK_SHIFT` is verbatim `10 /* up to 1K */`; `vdev_ashift` field comment is verbatim `/* block alignment shift */`.

**Three combined takeaways**:

| Takeaway | Verbatim basis |
|---|---|
| **Slot width scales with device granularity**, not a fixed constant | `MAX((vd)->vdev_top->vdev_ashift, UBERBLOCK_SHIFT)` |
| **Capped at 8 KiB**—does not balloon because array stripe chunks are large | `MAX_UBERBLOCK_SHIFT (13)` and preceding comment |
| **Total ring size fixed at 128 KiB => widening slot width reduces slot count**, rather than consuming more disk space | `VDEV_UBERBLOCK_COUNT = VDEV_UBERBLOCK_RING >> SHIFT` |

⚠️ **The metric used is ashift (alignment / minimum allocation unit), not Linux io_min (stripe chunk size on arrays).**
These two metrics diverge drastically on md arrays—E34 (Root Ring Slot Geometry) measured a 128x difference on real md/raid5.

⚠️ **Known difference with this project** (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` requires noting):
ZFS labels occupy 4 **fixed areas** per disk at 256 KiB each, with uberblock ring permanently taking 128 KiB;
in this project, the root ring distributes R regions across the disk with prime stride; **region size is not a format constant**—slots per region S resides in system configuration, and only the two edges of its interval (lower bound 4, upper bound 16) are format constants (D22 (Synthesizing Unit Atomicity) settled item 2).
=> "Widening slot width = reducing slot count" holds in this project only if "fixed region size" is established, which does not exist in the repository.

### 8.2 btrfs: relies on commit root + pausable traversal, excluded in this project by two independent gates

Source: `linux-6.17/fs/btrfs/`, inspected verbatim 2026-09-08:

| Location | Verbatim code |
|---|---|
| scrub.c scrub_setup_ctx (two locations) | `sctx->extent_path.search_commit_root = 1;` / `sctx->csum_path.search_commit_root = 1;` |
| scrub.c scrub_enumerate_chunks | `path->search_commit_root = 1;` |
| transaction.c switch_commit_roots | `down_write(&fs_info->commit_root_sem);` / `up_write(&fs_info->commit_root_sem);` |
| ctree.c finish_need_commit_sem_search | `lockdep_assert_held_read(&lowest->fs_info->commit_root_sem);` |
| transaction.c btrfs_commit_transaction | `btrfs_scrub_pause(fs_info);` |

⚠️ **This project cannot use this path; excluded by two independent gates (settled 2026-09-08)**:
① **Commit interval differs by 83,000x**—btrfs default commit interval is verbatim `#define BTRFS_DEFAULT_COMMIT_INTERVAL (30)` (`linux-6.17/fs/btrfs/fs.h`, 30 seconds), whereas publication interval under singlefs workload A is **0.36 milliseconds** (2785 publishes/sec). btrfs implicitly assumes "commit interval is far larger than a consumer scan slice"; transplanted here, scrub would be paused 2785 times per second, never completing.
**This implicit premise was never cited alongside that practice.**
② **Checker side structurally cannot acquire that lock**—D13 (Verification Route) settled item 5 verbatim: "share only a constants module generated from kb; newtypes, parsing, checksums, traversals, and accounting written independently" => independent checker cannot hold the tested implementation's `commit_root_sem`.

This triad is documented to clarify what is excluded, not as a candidate solution.

=> **The Triad**: scrub reads the **commit root** (previous committed transaction root) rather than live tree; commit paths acquire **write lock** on `commit_root_sem` while traversals acquire read lock, mutually exclusive; commit paths **explicitly pause scrub** (`btrfs_scrub_pause`), rather than allowing it to continue with an expiring root.

**This is a fundamentally different path from "lengthening protection window to cover scan duration"**:
It leaves generation windows untouched, instead making consumers **pausable and synchronized with commit points**.

⚠️ **Known difference with this project**: btrfs commit root spans only **one generation** (previous committed transaction), sustained by in-memory read-write locks, thus covering only the **runtime** half;
whereas I-7.4 verbatim states: "after crash, in-memory defer queues are lost, runtime K is taken over by root ring depth (cleaning admission gate: freed generation <= oldest root txg in ring), protection width shifts accordingly"—**locks have zero say over the post-crash half.**

<!-- doc-lint:not-numbers EXT4 NILFS2 -->

## 9. Per-object extension slots: how 5 active implementations place "data belonging to this object but not in main fields" (D21 Item 3 ③ external survey, inspected 2026-09-13)

D21 Item 3 ③ verbatim states: "User-requested external survey (how others implement per-object extension slots) not yet done". All five surveyed verbatim from primary sources (URLs in table), **unverified in this project**; per SOP rules, they supply hypotheses and paths to test, not proofs or counterproofs. Each notes a difference with this project.

| Implementation | Where slot lives | Shape and width | Overflow behavior | Source (inspected) | Difference with this project |
|---|---|---|---|---|---|
| ext4 | Space after fixed fields in inode record (i_extra_isize records "Size of this inode - 128"; default 256B inode has 160B fixed, remainder for xattrs) | Header ext4_xattr_ibody_header 4B (magic 0xEA020000) + entries ext4_xattr_entry (e_name_len, e_name_index, e_value_offs, e_value_inum, e_value_size, e_hash), values 4B aligned; inline data is xattr named system.data (EXT4_INLINE_DATA_FL) | Spills to external block pointed by i_file_acl ("not possible for this block to contain a pointer to a second extended attribute block"); larger values use EA inode (e_value_inum, EXT4_EA_INODE_FL) | https://www.kernel.org/doc/html/latest/filesystems/ext4/inodes.html , https://www.kernel.org/doc/html/latest/filesystems/ext4/attributes.html | Slots live inside inode record, holding arbitrary named key-values; singlefs extension point per D1 Item 3 "is either empty or a single reference", never inlining data |
| XFS | Inode literal area (XFS_LITINO = inode size - core) shared by data and attr forks; di_forkoff (attr fork offset, <<3 for 64b align) sets attr fork start; di_aformat selects format | Short form: xfs_attr_sf_hdr { totsize u16, count u8, padding u8 } + xfs_attr_sf_entry { namelen u8, valuelen u8, flags u8, nameval[] }, "packed as tightly as possible so as to fit into literal area" | Attr fork switches to extents / btree format pointing to leaf, node, remote value blocks | https://raw.githubusercontent.com/torvalds/linux/master/fs/xfs/libxfs/xfs_format.h , https://raw.githubusercontent.com/torvalds/linux/master/fs/xfs/libxfs/xfs_da_format.h | Fork offset is variable per-inode (di_forkoff); singlefs extension point offset is declared in system configuration N and implied by unit structure, no per-unit offset |
| btrfs | Entries in fs tree keyed by objectid: XATTR_ITEM (0x18, key carries name hash), DIR_ITEM (0x54), EXTENT_DATA (0x6c, when inlined "remaining item bytes are data bytes") | Variable-length entries in leaf: header \| item 0 \| item 1 \| … \| free space \| data N \| … \| data 0; max_inline default: min(2048, page size), ~3900B at 4KiB sectorsize | Same-hash xattrs must fit in one leaf; data exceeding max_inline uses external extent | https://btrfs.readthedocs.io/en/latest/dev/On-disk-format.html , https://btrfs.readthedocs.io/en/latest/Administration.html | Extension data is independent variable-length items in index tree; singlefs places it in fixed quota after self-describing unit header |
| ZFS | Bonus buffer following dnode core 64B: DN_BONUS_SIZE(dnsize) = dnsize - DNODE_CORE_SIZE - (1 << SPA_BLKPTRSHIFT) (320B at 512B dnode), dn_bonustype u8, dn_bonuslen u16 | Bonus holds System Attributes (SA); type declared by dn_bonustype | Spill blocks store system attribute data that does not fit in bonus buffer (DNODE_FLAG_SPILL_BLKPTR), spill pointer consumes trailing blkptr in bonus | https://raw.githubusercontent.com/openzfs/zfs/master/include/sys/dnode.h | Per-object slot width is dnode size minus core (reaches >1KiB); singlefs quota N declared per line in system config, 0 in first version |
| APFS | Variable-length extension fields at end of inode value j_inode_val: apfs_xf_blob { xf_num_exts u16, xf_used_data u16, xf_data[] }, each field apfs_x_field { x_type u8, x_flags u8, x_size u16 }; types like DSTREAM=8, NAME=4, FINDER_INFO=7 | 4B metadata + value per field; xattr values apfs_xattr_val { flags u16, xdata_len u16, xdata[] } | Embedded limit APFS_XATTR_MAX_EMBEDDED_SIZE = 3804 (XATTR_DATA_EMBEDDED=2), larger values use data stream (XATTR_DATA_STREAM=1) | https://raw.githubusercontent.com/linux-apfs/linux-apfs-rw/master/apfs_raw.h (Apple PDF used CID font unextractable via pdf-text.py; verified Linux driver header) | Type + length self-describing field list holding arbitrary data; singlefs extension point holds only a single reference |

**Common pattern across all 5 implementations**: All feature "bounded space within object record + external pointer on overflow", bounded at 320 bytes (ZFS 512B dnode) to ~3.9 KiB (btrfs leaf, APFS 3804). **Not a single implementation restricts extension slots to 'reference only, no inline data'**—this is unique to singlefs D1 Item 3. Per SOP evidence rules, "no active implementation follows path X" is a high-value external signal placing burden of proof on singlefs (justified in D1 Item 3 and D21 Hard Constraint 5).
**How 'empty' is encoded across implementations**: ext4 checks `e_value_size = 0` and entry count; XFS checks `count = 0`; btrfs relies on item absence; ZFS checks `dn_bonuslen = 0`; APFS checks `xf_num_exts = 0`—**all rely on 'count is zero' rather than 'all-zero pointer'**, aligning with D21 Item 3 ②: "all-zero cannot simply be treated as null".

## 10. Deadlist accounting, relocating shared blocks, and discard timing under snapshots

Retrieved on 2026-10-12. None of these items has been verified in this project; each is a clue, not evidence (`.claude/singlefs-ai-sop/rules/evidence-discipline.md`, "What other projects do is a clue, not evidence").

**Versions Checked and Retrieval Method**

| Target | Version / Commit | Method |
|---|---|---|
| OpenZFS master | 4542a317e2fa524e24c1c28b6fbca9cd3634ba56 (2026-10-09) | git clone --depth 1 --filter=blob:none --sparse, checked out module/zfs include/sys man cmd/zpool cmd/zfs |
| Linux Mainline | 24a19b953e2a3d0ea5a8c4951a0ab8f32720c907 (Linux 7.3-rc7, 2026-10-11) | --depth 1 --sparse, checked out fs/btrfs fs/f2fs fs/nilfs2 include/uapi/linux Documentation/filesystems |
| bcachefs | bcachefs-tools efec39c7e8d1dda11fb16bad0e1e75d86ead3105 (2026-10-10) fs/ directory | --depth 1. bcachefs is no longer in mainline: mainline HEAD git ls-tree fs/ \| grep -c bcachefs returns 0 |
| nilfs-utils | master 26efb5daff0757365101035145331b0a5a85d9d9 | raw.githubusercontent fetched lib/gc.c, man/nilfs_cleanerd.conf.5 |
| APFS | Apple File System Reference, PDF CreationDate 2020-06-22, 181 pages | Downloaded from developer.apple.com, text extracted with pypdf |
| btrfs Documentation | btrfs.readthedocs.io/en/latest Trim, btrfs-quota, Qgroups, btrfs-balance, btrfs-man5 | Fetched via curl |

Every file:line reference points to a commit in the "Versions inspected and how they were obtained" table. The preexisting local `/home/fy5090/code/fs-refs/zfs` (`58d73c90`, 2026-08-27) and `/home/fy5090/linux-bug-fix/linux` (v7.3-rc1, shallow clone) are older than mainline and were not used for citations.

The singlefs-side facts used to describe the differences are only two, both checked live: "settled item 5: the position entry is demoted to a hint, the central mapping is the sole entry point" and "settled item 7: pointer headers carry birth (tree, txg), 8 bytes each" in `.claude/kb/decisions/19-块指针的结构与宽度预算.md` (abbreviated as "singlefs uses central mapping" and "singlefs pointers carry birth txg").

### 10.1 ZFS: deadlist, livelist, why no BP rewrite, device removal indirect mapping

| Fact | Source (OpenZFS 4542a317) | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Memory structure: deadlist is an AVL tree ordered by dle_mintxg, each entry holding a bpobj; dl_cache caches by mintxg bytes; on-disk header dsl_deadlist_phys_t stores only used/comp/uncomp totals | include/sys/dsl_deadlist.h definitions; module/zfs/dsl_deadlist.c:106 (dsl_deadlist_compare compares dle_mintxg) | Source reading | Deadlist bucketed by birth txg: ZFS block pointers are physical DVAs without central mapping; singlefs uses central mapping with pointers carrying birth txg |
| Keys on disk: deadlist object is a ZAP, key is mintxg (decimal string), value is bpobj object ID | dsl_deadlist.c:534 (zap_add_int_key(... mintxg, obj ...)), :895-897 | Source reading | ZFS block pointers are physical DVAs; singlefs uses central mapping |
| Bucketing rule: killed blocks use BP_GET_BIRTH(bp) to find largest key strictly smaller than mintxg ("mintxg is not inclusive") | dsl_deadlist.c:477-491; :511-514 comments | Source reading | ZFS block pointers are physical DVAs; singlefs uses central mapping |
| Entry timing: dsl_dataset_block_kill directly frees blocks with birth > ds_prev_snap_txg; otherwise enters head deadlist; zio callbacks enter ds_pending_deadlist first, sync thread inserts later | module/zfs/dsl_dataset.c:304-349; :2397-2398 | Source reading | ZFS block pointers are physical DVAs; singlefs uses central mapping |
| Snapshot creation: snapshot inherits head's deadlist object; head receives clone (identical keys, empty bpobj) plus new key ds_prev_snap_txg | dsl_dataset.c:1787, :1842-1849; dsl_deadlist_clone at :700-730 | Source reading | ZFS block pointers are physical DVAs; singlefs uses central mapping |
| Concurrency: deadlist modified exclusively in syncing thread; non-insert operations require dp_config_rwlock write lock | dsl_deadlist.c:25-31 | Comment reading | Concurrency model differences serve only as clues |
| Snapshot deletion (new format): first use `dsl_deadlist_space_range` to adjust prev unique and snapused; `dsl_deadlist_move_bpobj` moves whole buckets from next's deadlist whose mintxg ≥ the deleted snapshot's `ds_prev_snap_txg` to the pool's `dp_free_bpobj` (these blocks were born after the deleted snapshot and died before next, so nothing references them). Then `dsl_deadlist_merge` merges the deleted snapshot's own deadlist by key into next's deadlist. Finally remove the deleted snapshot's creation_txg key from the head deadlist (`remove_key` merges that bucket into the previous bucket). | `module/zfs/dsl_destroy.c:403-439`, `:478-487`; `dsl_deadlist.c:920-979` (move_bpobj), `:855-918` (merge), `:539-567` (remove_key) | Source reading | ZFS block pointers are physical DVAs, with no central mapping. This project uses central mapping and pointers carry birth txg. This project plans to merge a deleted snapshot's deadlist into the newer side; ZFS merges into next, the newer side. |
| Merge and move attach whole bpobj sub-objects (bpobj_enqueue_subobj / dle_enqueue_subobj) without moving blocks individually; prefetches up to 128 sub-deadlists | dsl_deadlist.c:822, :954, :882-891, :942-949 | Source reading | ZFS block pointers are physical DVAs; singlefs uses central mapping |
| Old on-disk format (single bpobj, dl_oldfmt) runs process_old_deadlist: block-by-block comparison of birth vs ds_prev_snap_txg | dsl_destroy.c:127-151, :154-188 | Source reading | Legacy burden: singlefs has no legacy format |
| livelist definition: uses dsl_deadlist_t attached to clone dsl_dir, recording blocks born after clone creation, flagging FREE/ALLOC; clone deletion scans backward canceling pairs, freeing remainder; embedded blocks omitted | dsl_deadlist.c:45-63; dsl_dataset.c:291-302 | Comments & source | Singlefs plans read-only snapshots; ZFS livelist serves writable clones only |
| When livelist is used: allocated on clone creation when livelist feature active; disabled in sync when shared ratio <= zfs_livelist_min_percent_shared (default 75%); deleted on promote/swap; uses dsl_async_clone_destroy on clone destruction | dsl_dataset.c:1353-1365, :2169-2183, :2400-2404, :4019-4026, :4518-4524; dsl_destroy.c:1139-1146 | Source reading | Singlefs plans read-only snapshots; ZFS livelist serves writable clones only |
| livelist sub-tables: splits into new sub-table when entries exceed zfs_livelist_max_entries (default 500k, ~128M); grouped by birth txg so pairs fall in same sub-table; periodic condense thread | dsl_deadlist.c:64-75, :77-86; spa.c:3486, :3559 | Comments & source | Singlefs plans read-only snapshots; ZFS livelist serves writable clones only |
| livelist feature page: deletes faster than traditional methods under heavy random/sparse clone writes; active from clone creation until all clones deleted | man/man7/zpool-features.7:775-782 | Doc reading | Singlefs plans read-only snapshots; ZFS livelist serves writable clones only |

ZFS Group 2: No block pointer rewrite; device removal uses indirect mapping.

| Fact | Source & Date | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Self-stated rationale: indirect vdev corresponds to removed device, "since we cannot rewrite block pointers of snapshots, etc.", retaining old->new mapping checked on every DVA access | module/zfs/vdev_indirect.c:36-47 (4542a317) | Comment reading | Adding old->new mapping at SPA layer without altering pointers: ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| Mapping does not align with logical block boundaries; a DVA may span split blocks, so not every DVA can be replaced by an equivalent new DVA, providing segmented callback vdev_remap | vdev_indirect.c:41-58 | Comment reading | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| Designer public statement (Alex Reece, 2015-01-13, Delphix blog): per-block pointer mapping "quickly requires an unreasonably large data structure that must be stored both on disk and kept in memory"; "Even BP rewrite would require a huge temporary table"; hence SPA-layer contiguous extent mapping | Delphix blog 2020 web.archive.org snapshot, lines 104-107 | Archive doc reading | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| Removal workflow: stop allocation; a separate thread copies allocated segments in LBA order, and sync tasks update partial mappings. Frees during copying free both the old position and the new position already copied. Once complete, the vdev becomes indirect. | `module/zfs/vdev_removal.c:43-73` | Comment reading | ZFS normally has no central mapping; it adds mapping for just that vdev after removing a device. This project uses central mapping, so all pointers already go through mapping. |
| Restrictions: the normal allocation class must not contain top-level raidz/draid vdevs (segment copying ignores block boundaries, so it cannot calculate parity columns); all top-level vdevs must have the same ashift; keys for encrypted datasets must be loaded; all vdevs must be online and healthy. | `vdev_removal.c:75-92`; `man/man8/zpool-remove.8:50-52` | Comments & docs | ZFS normally has no central mapping; it adds mapping for just that vdev after removing a device. This project uses central mapping, so all pointers already go through mapping. |
| Unresolved issues: #9013 "device removal for RAID-Z" (ahrens, 2019-07-10 open, still open 2026-10-12); #9129 "generic device removal" open | GitHub issues 9013, 9129, checked 2026-10-12 | Issue reading | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| Memory cost: entire mapping loaded into memory (vmem_alloc(map_size) + dmu_read); zpool remove -n estimates memory; zpool status reports mapping memory | module/zfs/vdev_indirect_mapping.c:357-361; zpool-remove.8:78-83; cmd/zpool/zpool_main.c:10607 | Source & docs | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| On-disk mapping entry = vimep_src (u64, 63b source offset + 1b mark) + vimep_dst (dva_t); asize 24b, maps up to 8GB | include/sys/vdev_indirect_mapping.h:29-43 | Source reading; inferred 24 bytes (8 + 16) | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| User report magnitude: removing 6.94TB vdev consumed "39.8M memory used for removed device mappings" | issue #18930 (2026-08-15, TrueNAS, zfs-2.3.0) | User report, workload unverified | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| Condensing mapping: freed/remapped blocks become obsolete; when obsolete pct >= zfs_condense_indirect_obsolete_pct (default 25%) or obsolete spacemap > 1GB, rewrites new mapping containing referenced entries only; resumable; the obsolete_counts feature has no rollback once active | vdev_indirect.c:60-124, :126-171, :175-197; zpool-features.7:864-878 | Comments & docs | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| remap: writing indirect blocks updates DVAs pointing to indirect vdevs to concrete DVAs where possible; dedup/gang blocks not remapped; snapshot-referenced old DVAs enter ds_remap_deadlist | module/zfs/metaslab.c:5699-5730; vdev_indirect.c:100-123; dsl_destroy.c:280-302 | Source reading | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| zfs remap subcommand removed from master | grep '"remap"' cmd/zfs/zfs_main.c yields zero matches | grep verification | Legacy burden: singlefs has none |
| Reading split blocks with damaged replicas reconstructs via combinations, randomly sampling 100 if combinations exceed limit (default 4096) | vdev_indirect.c:207-215 | Comment reading | ZFS lacks central mapping normally; singlefs routes all pointers through central mapping |
| Recent indirect-related defects (titles and status only): #18930 "Kernel NULL pointer dereference in vdev_indirect_mapping_duplicate_adjacent_entries() … on a pool with an indirect vdev", open; #19120 "vdev_indirect: validate split-block copies before repair", closed (2026-09-16); #12200 "VERIFY3(0 == vdev_obsolete_sm_object(...)) failed", open | GitHub search API `q=repo:openzfs/zfs "indirect vdev"`, 292 hits in total, retrieved on 2026-10-12 | Read titles only; also read the beginning of #18930's body | ZFS normally has no central mapping; it adds mapping for just that vdev after removing a device. This project uses central mapping, so all pointers already go through mapping. |
| Related current practice: `zfs rewrite` rewrites file blocks unchanged to new locations. After rewriting blocks shared with snapshots, the old copies remain for the snapshots, increasing space use. `-S` skips snapshot-shared blocks, `-C` skips BRT clone blocks, and `-P` preserves logical birth (requires the `physical_rewrite` feature). | `man/man8/zfs-rewrite.8:31-57`, `:84-91`; `zpool-features.7:880-895` | Doc reading | ZFS normally has no central mapping; it adds mapping for just that vdev after removing a device. This project uses central mapping, so all pointers already go through mapping. |

Searched but not found: no design document in the OpenZFS tree explicitly explaining "why not BP rewrite" (running `grep -rn -i 'bp rewrite\|bprewrite\|block pointer rewrite\|rewrite block pointers\|bp_rewrite'` on the sparse checkout of `module include man cmd` found only the comment at `module/zfs/vdev_indirect.c:38`; directories not checked out, such as `tests` and `lib`, were not searched). The original Delphix blog page is offline, with only an archive remaining. No publicly measured latency cost was found for the indirect-mapping read path.

### 10.2 btrfs: shared block reference counts, snapshot deletion, balance/relocation

| Fact | Source (Linux 24a19b95, 7.3-rc7) | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Snapshot creation: btrfs_copy_root copies root node only, calls btrfs_inc_ref on new root (incrementing child refs by 1), sets last_snapshot to transid; non-root tree blocks untouched | fs/btrfs/ctree.c:238-296; transaction.c:1798, :1827 | Source reading | Sharing via extent tree refcounts and backrefs, lazily pushing refs during COW: btrfs parents store child logical addresses directly; singlefs uses central mapping |
| Determining sharing: non-shareable trees, root nodes, generation > last_snapshot without relocation deemed unshared without querying extent tree | ctree.c:312-347 | Source reading | btrfs parents store child logical addresses directly; singlefs uses central mapping |
| COW on potentially shared block (update_ref_for_cow): refs > 1 sets BTRFS_BLOCK_FLAG_FULL_BACKREF on old block and increments ref; refs == 1 with full backref increments new and decrements old | ctree.c:349-451 | Source reading | btrfs parents store child logical addresses directly; singlefs uses central mapping |
| btrfs_inc_ref / btrfs_dec_ref route to __btrfs_mod_ref: traverses child pointers generating delayed refs; parent = buf->start when full_backref, else parent = 0 | extent-tree.c:2553-2649 | Source reading | btrfs parents store child logical addresses directly; singlefs uses central mapping |
| Snapshot deletion, `btrfs_drop_snapshot`: walk the tree, freeing blocks referenced only by this tree. At a shared block, only decrement its reference without descending (the `do_walk_down` comment says "if the block is shared and there is no need to update back, this function drops the reference"). | `extent-tree.c:6247-6259`, `:5902-5913` | Comment reading | btrfs parent nodes store child-block logical addresses directly, without central mapping. This project uses central mapping and plans deadlists instead of walking reference counts. |
| Resumable snapshot drop: writes drop_progress (key) and drop_level to root_item; commits transaction when yielding to cleaner or transaction boundaries, returning -EAGAIN; resumes on mount | extent-tree.c:6318-6373, :6403-6433, :6514-6519 | Source reading | btrfs parents store child logical addresses directly; singlefs uses central mapping |
| Preventing duplicate dec_ref: check_ref_exists verifies reference exists before clearing restarted flag | extent-tree.c:5856-5869 | Source reading | btrfs parents store child logical addresses directly; singlefs uses central mapping |
| Deleting snapshot under qgroup: shared subtree with refs > 1 calls btrfs_qgroup_trace_subtree; subtree root level >= qgroup_drop_subtree_thres (default 3) marks qgroup inconsistent without calculating, to avoid stalling commit | extent-tree.c:5871-5887; qgroup.c:2647-2658; qgroup.h:127; sysfs.c:2447-2448 | Source reading | btrfs parents store child logical addresses directly; singlefs uses central mapping |
| Official docs on qgroup: "not recommended for daily usage unless there is no planned snapshot creation/deletion"; "core design of qgroup and snapshot are not compatible from day one"; snapshot deletion "can cause system hang" | btrfs-quota.html "STABILITY AND PERFORMANCE IMPLICATIONS" | Doc reading | Singlefs lacks qgroup accounting layer; informative clue |
| Documentation on squota: qgroup "must count up trees referring to an extent after ref change, slowing transaction commits with unacceptable latency as snapshots scale" | Qgroups.html "Simple quotas (squota)" | Doc reading | Singlefs lacks qgroup accounting layer; informative clue |
| Overall relocation workflow: make the block group read-only; record every extent in it; build a data reloc tree (one inode holds the newly moved data extents) and a reloc tree (a special snapshot of the source tree holding relocated tree blocks); swap the source tree and the reloc tree; finally clean up both reloc tree and data reloc tree (old extents remain referenced by the reloc tree, so the block group can be freed only after cleanup). | `fs/btrfs/relocation.c:42-87` | Comment reading | Classic relocation: backref walking, reloc tree, rewriting all caller pointers: btrfs rewrites addresses in every caller; singlefs uses central mapping |
| Backref walking: build_backref_tree ascends from backrefs finding upper referencing blocks recursively until tree root or cache hit | relocation.c:485-498 | Comment reading | btrfs rewrites addresses in every caller; singlefs uses central mapping |
| Two-phase data relocation: MOVE_DATA_EXTENTS copies data, flushes and waits on ordered extents before switching to UPDATE_DATA_PTRS to rewrite file extent items | relocation.c:5382-5431, :928-959 | Source reading | btrfs rewrites addresses in every caller; singlefs uses central mapping |
| Crash recovery: btrfs_recover_relocation merges reloc trees with fs trees, preserving tree block sharing; balance progress stored internally, resumed on mount | relocation.c:5614-5620; btrfs-balance.html | Comments & docs | btrfs rewrites addresses in every caller; singlefs uses central mapping |
| Balance docs: "Extent sharing preserved and reflinks unbroken"; balance without filters "takes a lot of time moving data/metadata from whole filesystem and updating all block pointers" | btrfs-balance.html lines 111-123 | Doc reading | btrfs rewrites addresses in every caller; singlefs uses central mapping |
| qgroup and relocation: swapping subtrees traces large numbers of tree blocks; qgroup_trace_extent_swap and btrfs_qgroup_add_swapped_blocks defer tracing | qgroup.c:2297-2330, :4751-4765 | Comment reading | btrfs rewrites addresses in every caller; singlefs uses central mapping |
| Evading backref walk costs: send falls back from clone to write for extents with refs > SEND_MAX_EXTENT_REFS (1024), citing "limitations of backreference walking code (taking too much memory/time)" | send.c:37-43, :1506-1514 | Source reading | btrfs rewrites addresses in every caller; singlefs uses central mapping |

btrfs Group 2: Mainline remap tree (relocation without rewriting caller pointers via address remapping layer).

| Fact | Source & Date | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Introduction: incompat bit BTRFS_FEATURE_INCOMPAT_REMAP_TREE, tree ID BTRFS_REMAP_TREE_OBJECTID 13, keys IDENTITY_REMAP(234)/REMAP(235)/REMAP_BACKREF(236), flags REMAPPED & METADATA_REMAP | include/uapi/linux/btrfs.h:339; btrfs_tree.h:80, :288-290, :1174-1175, :1348-1355 | Source reading | Block-group-level old->new address mapping translated on I/O: btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |
| First mainline tag: `INCOMPAT_REMAP_TREE` in `include/uapi/linux/btrfs.h` has count 0 in v6.19 and count 1 in v7.0-rc1. Introducing commit `ef6a31d035a1`, "btrfs: add definitions and constants for remap-tree", authored by Mark Harmstone on 2026-01-07 and committed by David Sterba on 2026-02-03. | git.kernel.org `plain/include/uapi/linux/btrfs.h?h=<tag>` and `commit/?id=ef6a31d…`, retrieved on 2026-10-12 | Measured (`curl` + `grep -c`) | btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |
| Experimental status: sysfs remap_tree property guarded by CONFIG_BTRFS_EXPERIMENTAL, comment "Remove once support is feature complete" | sysfs.c:297-304, :333-337 | Source reading | btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |
| Eligible block groups: non-SYSTEM, non-METADATA_REMAP with REMAP_TREE enabled; runs move_existing_remaps, start_block_group_remapping, do_remap_reloc; skips relocate_file_extent_cluster | relocation.h:15-24; relocation.c:5542-5559, :3747-3751; volumes.c:3745-3750 | Source reading | btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |
| I/O translation: btrfs_translate_remap queries remap tree for REMAPPED groups; IDENTITY_REMAP indicates unmoved, REMAP yields new start address | relocation.c:5256-5304; volumes.c:7258-7268 | Source reading | btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |
| Empty block group definition: REMAPPED groups empty when identity_remap_count == 0; regular groups empty when used == 0 && remap_bytes == 0 | discard.c:218-235 | Source reading | btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |
| Motivation in the author's RFC: "Our present system involves following backrefs to rewrite addresses in the metadata trees, which can be slow", and "has knock-on effects such as breaking nocow files". For SYSTEM and REMAP block groups, "relocation is as it is at the moment". | Mark Harmstone, "[RFC PATCH 00/10] btrfs: remap tree", linux-btrfs, 2025-05-15, https://lwn.net/Articles/1021452/ | Read the email through a WebFetch summary; quotations are the original sentences supplied by that summary and were not checked verbatim against the email | btrfs applies mapping only to relocated block groups; singlefs translates all pointers through central mapping |

Searched but not found: no publicly measured figure in btrfs mainline for "how much slower balance is with qgroup enabled", only qualitative statements in the documentation. Whether there is a known-defect list for remap tree in 7.x was not investigated (the linux-btrfs list and fixes in 7.0–7.3 were not reviewed). The source for qgroup accounting when remap tree relocates snapshot-shared extents was not read.

### 10.3 bcachefs: snapshot tree and key visibility, copygc moving shared data, backpointers

| Fact | Source (bcachefs-tools efec39c7) | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Four btrees carry snapshots (extents, inodes, dirents, xattrs); keys carry snapshot ID (bpos.snapshot); keys visible iff snapshot ID is ancestor/self without closer ancestor override; deletion inserts whiteouts | snapshots/snapshot.c:44-54 | Doc comments | Sharing via key visibility by snapshot ID, without refcounts or deadlist: bcachefs shares at key level in key space; singlefs uses deadlist bucketed by birth txg and central mapping |
| Ancestor determination: snapshot nodes store 128-bit ancestor bitmap (O(1) within 128 ID difference) plus 3-entry skip list | snapshots/snapshot.c:56-60; snapshots/types.h:14 | Comments & source | bcachefs shares at key level; singlefs uses deadlist and central mapping |
| Descending snapshot IDs: children smaller than parents, placing ancestors later and newest first; visibility filter is a forward scan | snapshots/snapshot.c:62-84 | Comment reading | bcachefs shares at key level; singlefs uses deadlist and central mapping |
| Snapshot creation: split the source subvolume's current snapshot node into two child nodes, one for the new snapshot and one replacing the source subvolume; no keys are copied. The comment explicitly contrasts this with btrfs, which "clones entire COW btrees on snapshot". | `snapshots/snapshot.c:86-98` | Comment reading | The sharing unit in bcachefs is a btree key, with the snapshot tree in key space. This project plans to record deadlists by birth txg and use central mapping. |
| Extent invariant: two extents in different snapshots either match exactly or do not overlap; write path splits ancestor extents for alignment | snapshots/snapshot.c:117-121 | Comment reading | bcachefs shares at key level; singlefs uses deadlist and central mapping |
| Snapshot deletion: mark first, then reclaim in one or more passes of the `delete_dead_snapshots` recovery pass (also run online). "Cost is proportional to the data in the deleted snapshot". Drop keys of nodes with no live descendants directly; an intermediate node with only one live child moves its keys down to that child (`live_child`); retain intermediate nodes with two live children. | `snapshots/snapshot.c:128-163` | Comment reading | The sharing unit in bcachefs is a btree key, with the snapshot tree in key space. This project plans to record deadlists by birth txg and use central mapping. This project plans to merge the deadlist into the newer side; bcachefs moves keys down to the sole live descendant. |
| Interrupt semantics: keys only shift downwards, inodes processed last; interrupts produce only content-shifted but inode-unmoved skew | snapshots/snapshot.c:215-218 | Comment reading | The sharing unit in bcachefs is a btree key, with the snapshot tree in key space. This project plans to record deadlists by birth txg and use central mapping. |
| copygc: buckets are append-only, and random overwrites leave half-empty buckets. copygc selects the most fragmented buckets, finds live data via backpointers, writes it into new buckets, and atomically updates extent pointers. `copygc_reserve` (default 8%, configurable from 5–20%) is reserved exclusively for copygc. | `data/copygc.c:6-28`; `doc/bcachefs-principles-of-operation.tex:308-321` | Doc comments | Extents are keyed by logical position with physical pointers in their values, and relocation changes that one key: bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. |
| Bucket selection: the fragmentation LRU index is `dirty_sectors / bucket_size`, scaled by 2^31. Less live data ranks earlier (greedy by live-data volume). Do not move buckets currently being written, buckets on unwritable devices, or buckets whose backpointer and bucket counts disagree. | `alloc/background.h:127-145`; `data/copygc.c:98-140` | Source reading | bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. |
| Relocation commit: `bch2_data_update` writes the new key at the same snapshot position as the original key (with `BTREE_UPDATE_internal_snapshot_node`, allowing direct modification of a key in an intermediate snapshot node). When a key is split, it calls `bch2_insert_snapshot_whiteouts` to add whiteouts for descendant snapshots that have already overwritten that position. | `data/update.c:431`, `:473-479`; `btree/update.h:89-110`; `btree/update.c:123-159` | Source reading | bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. |
| Is it true that "being keyed by logical address means pointers need not change"? The source shows relocation still changes the physical pointer in the extent key (one transaction for that same key). What need not change is each snapshot's separate reference, because sharing has only one key. | `data/copygc.c:25-27`; `data/update.c:473-479` | Read source; "no per-snapshot changes needed" is inferred from "sharing has only one key" | bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. |
| backpointers: every on-disk sector range containing data has a backpointer stored in a write-buffer btree, with key = (device, sector offset, discriminator). The value records the owner key's btree ID, level, position (including snapshot ID), data type, and bucket generation. Extent triggers add and remove these automatically. | `alloc/background.c:246-263`; structure at `bcachefs_format.h:568-577` | Comments & source | bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. A backpointer points to that one key, rather than one per snapshot (inferred from key sharing). |
| Consistency: equal backpointer-sector and bucket-sector counts are taken to mean the bucket is consistent; only mismatches trigger a full bidirectional check. Three recovery passes cross-check one another. | `alloc/background.c:284-306` | Comment reading | bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. |
| Operations using backpointers: copygc, device evacuation, scrub, reconcile | alloc/background.c:271-282 | Comment reading | bcachefs keys carry snapshot IDs; data shared by multiple snapshots has only one key in the ancestor snapshot, and relocation changes only that key. This project uses central mapping, and relocation changes the mapping entry. |

Searched but not found: no public measurements from bcachefs of "copygc or delete_dead_snapshots cost with many snapshots". The "Subvolumes and snapshots" section at line 722 of `doc/bcachefs-principles-of-operation.tex` was not read line by line; only the `DOC_LATEX(snapshots)` comment in the source was read.

### 10.4 F2FS and NILFS2: segment cleaning

F2FS (Linux 24a19b95):

| Fact | Source | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Three segment selection algorithms: GC_CB (cost-benefit), GC_GREEDY, GC_AT (age-threshold); two GC modes: BG_GC (background), FG_GC (foreground) | fs/f2fs/segment.h:132-154 | Source reading | Log-structured segment cleaning: F2FS has no snapshots, GC never handles shared blocks; singlefs GC relocates snapshot-shared blocks |
| Default mappings: BG_GC uses CB (AT if atgc enabled), FG_GC uses GREEDY; gc_urgent/gc_idle can force modes | fs/f2fs/gc.c:248-273; Documentation/filesystems/f2fs.rst:822-828 | Source & docs | F2FS has no snapshots; singlefs GC relocates snapshot-shared blocks |
| Greedy cost = valid blocks in segment; CB cost = UINT_MAX - (100*(100-u)*age)/(100+u), where u is valid block pct and age is normalized mtime | gc.c:390-410, :362-388 | Source reading | F2FS has no snapshots; singlefs GC relocates snapshot-shared blocks |
| Background GC records selected victims in victim_secmap, prioritized by foreground GC; search bound by max_victim_search | gc.c:343-360, :302-310 | Source reading | F2FS has no snapshots; singlefs GC relocates snapshot-shared blocks |
| Foreground trigger: insufficient free sections switches to FG_GC. If there are enough prefree segments, first write a checkpoint to turn prefree into free, which may avoid FG_GC. If free space remains insufficient after GC, another checkpoint reclaims prefree segments. | `gc.c:1986-2004`, `:2068-2075` | Source reading | F2FS has no snapshots and its GC never handles shared blocks. This project's GC must move blocks shared with snapshots. |
| Relocation atomicity: cleaned segments enter prefree; f2fs_clear_prefree_segments clears them and issues discard only after successful checkpoint | fs/f2fs/checkpoint.c:2039-2046; fs/f2fs/segment.c:2209-2282 | Source reading | F2FS has no snapshots; singlefs GC relocates snapshot-shared blocks |
| SSR allocation takes ckpt_valid_map \| cur_valid_map zero bits; blocks valid at last checkpoint not overwritten before next checkpoint | segment.c:3126-3140; segment.h:182-185 | Source reading | F2FS has no snapshots; singlefs GC relocates snapshot-shared blocks |
| Docs: "F2FS adopts greedy algorithm for on-demand cleaner, while background cleaner adopts cost-benefit algorithm" | Documentation/filesystems/f2fs.rst:817-828 | Doc reading | F2FS has no snapshots; singlefs GC relocates snapshot-shared blocks |

NILFS2 (Linux 24a19b95 kernel, nilfs-utils 26efb5da userspace):

| Fact | Source | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Checkpoints generated automatically every few seconds or on sync writes; checkpoints can be converted to snapshots; NILFS_IOCTL_CHANGE_CPMODE toggles | Documentation/filesystems/nilfs2.rst:14-18, :100-102 | Doc reading | Block numbers virtualized via DAT; GC modifies DAT without touching trees; liveness based on checkpoint intervals: both use indirection; NILFS2 snapshots whole volume and runs liveness in userspace |
| Skip snapshots when deleting checkpoints: range deletion counts encountered snapshots and ultimately returns `-EBUSY`; deleting a single snapshot returns `-EBUSY` directly. | `fs/nilfs2/cpfile.c:507-513`, `:558-559`, `:750-777` | Source reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |
| GC runs in userspace: `nilfs_cleanerd` selects segments and judges liveness, then passes the checkpoint ranges to delete, virtual block numbers to free, blocks to move, and segments to free to the kernel in one `NILFS_IOCTL_CLEAN_SEGMENTS` call. | `nilfs2.rst:138-141`; `fs/nilfs2/ioctl.c:850-860` (five argv sections) | Docs & source | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |
| Segment-selection policy: `selection_policy` supports only `timestamp` (reclaim oldest to newest); `nsegments_per_clean` defaults to 2; `protection_period` defaults to 3600 seconds: "NILFS2 GC never deletes checkpoints and their blocks whose duration time is less than the value". | nilfs-utils `man/nilfs_cleanerd.conf.5:22-26`, `:52-58` | Doc reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. F2FS uses greedy/CB, whereas NILFS2 uses time alone. |
| Liveness check (userspace): block lifetime [p_start, p_end) in checkpoint numbers; live if p_end is NILFS_CNO_MAX or within protection period, or binary search finds snapshot in range | nilfs-utils lib/gc.c:417-470 | Source reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |
| Kernel-side atomicity: `nilfs_clean_segments` first saves DAT in a shadow map and restores it from shadow if preparation fails. It then writes segments using `SC_LSEG_SR` (a complete log with super root). Segment release occurs in the SUFILE phase of that same segment construction (`nilfs_free_segments`); failed construction is undone with `nilfs_sufile_cancel_freev`. After success, discard is issued to freed segments only if discard is enabled. | `fs/nilfs2/segment.c:2546-2580`, `:1260-1265`, `:1566-1573`, `:1880-1886` | Source reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |
| Comments explain why preparation can safely abort: checkpoints "can be removed independently"; DAT "is updated atomically using a copy-on-write technique"; marking dirty is "nondestructive". | `fs/nilfs2/ioctl.c:797-834` | Comment reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |
| Moving blocks modifies DAT: nilfs_btree_assign_gc calls nilfs_dat_move updating virtual-to-physical mapping; on-disk binfo preserves virtual block number, leaving referencing trees unchanged | fs/nilfs2/btree.c:2354-2379; fs/nilfs2/dat.c:317-331 | Source reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |
| GC input blocks attached to dummy inode (gcinode), released after copy; current generation dirty blocks never overlap GC blocks | fs/nilfs2/gcinode.c:11-22 | Comment reading | NILFS2 DAT provides one layer mapping virtual block numbers to physical block numbers, and this project uses central mapping: both have one level of indirection. The difference is that NILFS2 snapshots are whole-volume checkpoints, and a userspace daemon judges liveness. |

Searched but not found: no dedicated logic for resuming after a mid-GC crash was found in either F2FS or NILFS2. F2FS returns to the last checkpoint because "blocks valid before the last checkpoint are not reused before the next checkpoint"; NILFS2 relies on "segment release and the new log being written in the same operation". Both statements are inferred from their source code; no verbatim documentation statement was found. No GC segment-selection code was found in the NILFS2 kernel: `grep -rn -i 'victim\|greedy\|cost.benefit' fs/nilfs2 | wc -l` returned `0`.

### 10.5 APFS: snapshots and space reclamation facts

| Fact | Source & Date | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| Creating snapshot moves current volume extent-reference tree to snapshot, replacing volume tree with empty tree | Apple File System Reference (2020-06-22) p.55 apfs_extentref_tree_oid | Official doc | One extent-reference tree per snapshot with reference counts on physical extents: APFS records reference counts per physical extent; this project plans deadlists by birth txg. APFS has an object map (omap) mapping virtual oids to physical addresses, but file-data extents use physical addresses directly (on page 102, the key of `j_phys_ext_key_t` is a physical block address), unlike this project where all pointers go through central mapping. |
| Snapshot metadata record `j_snap_metadata_val_t` contains `extentref_tree_oid`, `sblock_oid`, and other fields. The documentation says: "Snapshots are designed to be fast and inexpensive to create; however, deleting a snapshot involves more work". | Apple File System Reference (2020-06-22 edition), pp. 117–118 | Official doc | APFS records reference counts per physical extent; this project plans deadlists by birth txg. APFS has an object map (omap) mapping virtual oids to physical addresses, but file-data extents use physical addresses directly (on page 102, the key of `j_phys_ext_key_t` is a physical block address), unlike this project where all pointers go through central mapping. |
| Physical extent record `j_phys_ext_val_t` has `refcnt`: "The extent can be deleted when its reference count reaches zero"; "For a volume that has no snapshots, the kind is always APFS_KIND_NEW". | Apple File System Reference (2020-06-22 edition), pp. 102–103 | Official doc | APFS records reference counts per physical extent; this project plans deadlists by birth txg. APFS has an object map (omap) mapping virtual oids to physical addresses, but file-data extents use physical addresses directly (on page 102, the key of `j_phys_ext_key_t` is a physical block address), unlike this project where all pointers go through central mapping. |
| The object map has a snapshot tree (xid → `omap_snapshot_t`), accessed using xid as the key. | Apple File System Reference (2020-06-22 edition), pp. 17 and 47 | Official doc | APFS records reference counts per physical extent; this project plans deadlists by birth txg. APFS has an object map (omap) mapping virtual oids to physical addresses, but file-data extents use physical addresses directly (on page 102, the key of `j_phys_ext_key_t` is a physical block address), unlike this project where all pointers go through central mapping. |
| The reaper handles "deletion of objects that are too large to be deleted in the time between transactions", preserving deletion progress across multiple transactions. Snapshot deletion uses `apfs_reap_state_t` (`last_pbn`, `cur_snap_xid`, `phase`) and the two omap reap phases (map tree first, then snapshot tree). | Apple File System Reference (2020-06-22 edition), pp. 26, 50, and 166–167 | Read official documentation (`apfs_reap_state_t` says "No overview available") | APFS records reference counts per physical extent; this project plans deadlists by birth txg. APFS has an object map (omap) mapping virtual oids to physical addresses, but file-data extents use physical addresses directly (on page 102, the key of `j_phys_ext_key_t` is a physical block address), unlike this project where all pointers go through central mapping. The planned deadlist merge in this project has no reaper-like object for deletion across transactions. |
| Spaceman maintains free queue keyed by (xid, paddr), spaceman_free_queue_t holds sfq_oldest_xid; documentation lacks reallocation timing | Apple File System Reference p.159-160 | Official doc | Informative clue |
| Partition shrinking uses evict-mapping tree recording "where blocks are being moved to" | Apple File System Reference p.33, p.42-43 | Official doc | Informative clue |
| Secondary source: deleting small snapshot "may take around 0.2s, mainly cleaning up object map", executed by container Reaper | Howard Oakley, "APFS: Snapshots", eclecticlight.co, 2024-04-08 | Secondary article | Informative clue |

Searched but not found: `grep -c -i trim` over the entire Apple reference manual matched seven lines, all for `INODE_ACTIVE_FILE_TRIMMED` ("overprovisioning file that has been trimmed") and its index entries. None explains when TRIM is issued or its ordering relative to block reallocation. The manual does not explain how snapshot deletion decides which extents to release, or the difference in space released when deleting newer versus older snapshots. The original reverse-engineering paper by Hansen and Toolan (Digital Investigation, 2017) was not obtained this time. No other official document about snapshot-space reclamation was found on Apple's developer site (only one web search was performed).

### 10.6 discard/TRIM: timing and ordering relative to block reallocatability

| Fact | Source | Measurement Basis | Difference with singlefs |
|---|---|---|---|
| ZFS freed blocks enter ms_defer[txg % TXG_DEFER_SIZE] (TXG_DEFER_SIZE = 2); upon expiry, moved to ms_allocatable and added to ms_trim if autotrim active; un-deferred directly to ms_trim when low on slop, removing vdev, or expanding raidz | module/zfs/metaslab.c:4616-4667; include/sys/txg.h:39 | Source reading | Reallocatable first, trimmed later, removing range from pending trim on reallocation: ZFS trims per metaslab; singlefs produces fully cleared segments before discard |
| Allocation removes range from ms_trim, ensuring reallocated ranges are not re-trimmed | metaslab.c:4928-4929, :5945-5946 | Source reading | ZFS trims per metaslab; singlefs produces fully cleared segments before discard |
| autotrim thread (one per top-level vdev) disables metaslab allocation while trimming, swaps ms_trim for empty tree, issuing TRIM against swapped tree; visits metaslab every zfs_trim_txg_batch (default 32) txgs; skips ranges <= 32KiB | module/zfs/vdev_trim.c:1257-1305, :54-78; metaslab.c:6304-6311 | Source reading | ZFS trims per metaslab; singlefs produces fully cleared segments before discard |
| Manual zpool trim: one thread per leaf vdev, traverses unallocated ranges across metaslabs, blocking allocation during trim; progress persisted, resumable | vdev_trim.c:40-52, :71-72 | Comment reading | ZFS trims per metaslab; singlefs produces fully cleared segments before discard |
| autotrim property docs: "Automatic TRIM does not immediately reclaim blocks after a free", delaying to batch small ranges; default off | man/man7/zpoolprops.7:444-458 | Doc reading | ZFS trims per metaslab; singlefs produces fully cleared segments before discard |
| btrfs discard=sync: transaction completion btrfs_finish_extent_commit runs btrfs_discard_extent on pinned ranges before unpin_extent_range returns them to allocatable; synchronous discard occurs before reallocatability | fs/btrfs/extent-tree.c:2998-3013 | Source reading | btrfs manages free space via in-memory free space cache per block group; singlefs manages per segment |
| btrfs discard=async: discard state exists only in in-memory free space cache, not persisted; block groups prioritized by idle time LRU, initial delay BTRFS_DISCARD_DELAY 120s, empty groups 10s; max IOPS 1000 | fs/btrfs/discard.c:16-61 | Comments & source | btrfs manages free space via in-memory free space cache per block group; singlefs manages per segment |
| btrfs async ordering: unpinned ranges return to allocatable first, then processed by discard worker; worker calls unlink_free_space, discards, and re-adds as TRIMMED; allocation resets trimmed bitmap | extent-tree.c:3052-3054; free-space-cache.c:3768, :3794, :3650-3710; discard.c:44-52 | Source reading | btrfs manages free space via in-memory free space cache per block group; singlefs manages per segment |
| btrfs docs: sync discard "could have severe performance hit and is not recommended"; async batches in memory; "kernel 6.2 enables discard=async automatically where supported" | btrfs.readthedocs.io Trim.html | Doc reading | btrfs manages free space via in-memory free space cache per block group; singlefs manages per segment |
| F2FS: prefree segments convert to free only after checkpoint success, whereupon discard is issued; block allocation carves out unsubmitted discards or waits on inflight discards | fs/f2fs/segment.c:2223-2282, :1874-1917, :3933 | Source reading | Segment-based reclamation similar to singlefs; F2FS lacks snapshots |
| NILFS2: GC-freed segments issued discard only after log containing super root successfully commits | fs/nilfs2/segment.c:2560-2580 | Source reading | Segment-based reclamation similar to singlefs |
| bcachefs: emptied bucket triggers set need_discard in need_discard btree; discard worker issues TRIM before clearing flag, moving bucket to freespace btree; requires journal sequence <= flushed_seq_ondisk and < rewind_seq_ondisk | alloc/background.c:1291-1300, :56-76; alloc/discard.c:343-351 | Source reading | bcachefs enforces "discard first, reallocatable second", opposite to ZFS/btrfs async |
| APFS: Official reference contains no TRIM specifications | APFS manual lacks TRIM details (see 10.5 end) | Doc reading | Unclear |

Searched but not found: no dedicated description of handling "a crash before discard completes" was found for either ZFS or btrfs (the btrfs async comment only says everything is treated as undiscarded at mount). No discard latency or throughput was measured for either system.

### 10.7 Counterevidence checklist

| Counterevidence | Implementations investigated | How investigated |
|---|---|---|
| Across all investigated systems, not a single one combines "deadlist bucketed by birth txg" with "segment cleaning that relocates snapshot-shared blocks" in the same implementation | ZFS (has deadlist, no generic GC, device removal uses indirect mapping); btrfs (sharing uses extent refcounts, snapshot deletion traverses tree decrementing refs); bcachefs (sharing uses key visibility by snapshot ID, snapshot deletion moves keys); F2FS (no snapshots); NILFS2 (DAT records checkpoint ranges, liveness judged against snapshots); APFS (extentref tree per snapshot + physical extent refcounts, GC undocumented) | Inspected 10.1–10.5 sources and docs; "none" covers only these six implementations and commits |
| OpenZFS master lacks mechanism to rewrite snapshot-referenced block pointers: source comments state "we cannot rewrite block pointers of snapshots"; zfs rewrite duplicates shared blocks, leaving old replica to snapshot | ZFS exclusively | vdev_indirect.c:36-40; zfs-rewrite.8:52-57; sparse checkout grep bp rewrite yielded only that comment |
| Across investigated systems, relocating snapshot-shared data without rewriting callers always introduces indirection: NILFS2 modifies DAT (virtual->physical), ZFS device removal adds indirect mapping, btrfs remap tree adds block-group-level remapping, bcachefs shares via a single key; only btrfs classic relocation rewrites all caller pointers directly, noted in RFC as "can be slow" | ZFS, btrfs, bcachefs, NILFS2; F2FS not applicable; APFS unknown | 10.1–10.4 sources and RFC verbatim |
| ZFS and btrfs async make blocks reallocatable first and issue trim later, removing ranges from pending trim upon allocation; bcachefs does the inverse: discard first, freespace second | ZFS, btrfs, bcachefs; F2FS issues discard after checkpoint; NILFS2 issues after log commit | 10.6 sources |

Measurement basis note: Quotations from LWN remap tree RFC and eclecticlight APFS article are from WebFetch summaries without verbatim comparison to original emails; Delphix blog uses 2020 web.archive.org snapshot.

### 7.9 Sources for Section 7 ("Survey of Academic Results in the Last Decade (2012–2026)")

- [The Full Path to Full-Path Indexing (FAST 2018)](https://www.usenix.org/conference/fast18/presentation/zhan)
- [Copy-on-Abundant-Write for Nimble File System Clones (TOS 2021) / How to Copy Files (FAST 2020)](https://www.usenix.org/conference/fast20/presentation/zhan)
- [File Systems Fated for Senescence? Nonsense, Says Science! (FAST 2017)](https://www.usenix.org/conference/fast17/technical-sessions/presentation/conway)
- [SplinterDB: Closing the Bandwidth Gap for NVMe Key-Value Stores (ATC 2020)](https://www.usenix.org/conference/atc20/presentation/conway)
- [Closing the B+-tree vs. LSM-tree Write Amplification Gap (FAST 2022)](https://www.usenix.org/conference/fast22/presentation/qiao)
- [Rethinking File Mapping for Persistent Memory (FAST 2021)](https://www.usenix.org/conference/fast21/presentation/neal)
- [TABLEFS: Enhancing Metadata Efficiency in the Local File System (ATC 2013)](https://www.usenix.org/conference/atc13/technical-sessions/presentation/ren)
- [Push-Button Verification of File Systems via Crash Refinement (OSDI 2016)](https://www.usenix.org/conference/osdi16/technical-sessions/presentation/sigurbjarnarson)
- [Finding Crash-Consistency Bugs with Bounded Black-Box Crash Testing (OSDI 2018)](https://www.usenix.org/conference/osdi18/presentation/mohan)
- [Chipmunk: Investigating Crash-Consistency in Persistent-Memory File Systems (EuroSys 2023)](https://dl.acm.org/doi/10.1145/3552326.3567498)
- [Metis: File System Model Checking via Versatile Input and State Exploration (FAST 2024)](https://www.usenix.org/conference/fast24/presentation/liu-yifei)
- [GoJournal: a verified, concurrent, crash-safe journaling system (OSDI 2021)](https://www.usenix.org/conference/osdi21/presentation/chajed)
- [Verifying the DaisyNFS concurrent and crash-safe file system with sequential reasoning (OSDI 2022)](https://www.usenix.org/conference/osdi22/presentation/chajed)
- [Optimistic Crash Consistency (SOSP 2013)](https://research.cs.wisc.edu/adsl/Publications/optfs-sosp13.pdf)
- [Consistency Without Ordering (FAST 2012)](https://www.usenix.org/conference/fast12/consistency-without-ordering)
- [High Velocity Kernel File Systems with Bento (FAST 2021)](https://www.usenix.org/publications/loginonline/high-velocity-kernel-file-systems-bento)
- [NVMe Flexible Data Placement Overview (NVM Express, FMS 2023)](https://nvmexpress.org/wp-content/uploads/FMS-2023-Flexible-Data-Placement-FDP-Overview.pdf)
- [Bcachefs removed from the mainline kernel (LWN)](https://lwn.net/Articles/1040120/)

---
