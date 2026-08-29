# Background: authenticating the superblock and the root in a from-scratch COW filesystem

A copy-on-write filesystem is being designed from scratch (format-design phase, no code).
Everything below is a settled decision of that project.

## Settled: encryption

| # | Rule |
|---|---|
| A1 | **Whole-volume AEAD.** The checksum field inlined in the parent pointer *becomes* the MAC when encryption is on. |
| A2 | Everything is encrypted **except**: the superblock, and a plaintext "logical identity → physical address" map plus a plaintext reverse index. |
| A3 | The MAC is a full 128 bits, never truncated. |
| A4 | The nonce **must not be derived from physical position**; it travels with the logical identity and is stored in the pointer. |
| A5 | The AEAD's associated data is `(fsid, unit-type tag, tree id, object id, object birth generation, anchor offset)`. Physical position never goes in. |
| A6 | **Every AAD field's expected value may come only from the reader's lookup path, or from a ciphertext-side field already authenticated by the level above. Never from the plaintext side.** |
| A7 | The superblock must, from day one, carry: a KDF identifier, a master-key slot, and an encryption-type field. |

## Settled: the root

| # | Rule |
|---|---|
| B1 | Publication is checkpoint-based. The **root ring is a fixed set of K slots** outside the journal; publishing a root overwrites one slot. |
| B2 | **Every fsync publishes a root.** |
| B3 | Root-ring depth K **equals** the block-reuse delay — the same number. K must be a runtime policy, never a format constant. |
| B4 | A root must attest to itself: **whole-unit checksum** (it is complete) + **generation number** (it is this generation) + **slot rotation** (never overwrite the only copy). |
| B5 | Reading order is: **scan every slot, verify each, then pick the newest by generation.** Never "pick the newest by generation first, then verify." |

## Settled: nonce discipline

| # | Rule |
|---|---|
| C1 | No two blocks ever written (whether or not still referenced) may share a `(key id, nonce)` pair. |
| C2 | The **recorded nonce watermark must be strictly greater than the largest nonce anywhere on disk** — this is the decidable form of "never reuse after a crash". |
| C3 | The **nonce's id part must live in the plaintext header**, readable without the key. |
| C4 | C1's check is an **anti-bug check, not an anti-attacker check**: it enumerates written blocks by a plaintext magic number, and an attacker can erase the magic to make a block vanish from the scan. |

## Threat model (do not change it)

The attacker can read and write **every byte of the raw device** (offline, or online bypassing
the filesystem) but **does not have the master key**. A legitimate user then mounts and uses the
filesystem normally. Single key for the whole volume.

## Two positions already reached, which you should attack

- **Position P1**: the superblock must be authenticated, because rule A6 says AAD expected values
  may never come from the plaintext side, while the `fsid` field's expected value is documented as
  coming from "the superblock at mount time" — and the superblock is plaintext by A2. So either the
  superblock gets authenticated, or one of those two settled rules has to go.
- **Position P2**: a MAC proves *integrity*, not *freshness*. An attacker who replays a genuine
  **old** superblock passes the MAC check. So authenticating the superblock does not stop rollback
  of a watermark stored in it.

---

# Your role: find counterexamples

Do not summarise. Do not agree. Attack. Produce these, in order:

1. **Attack P1.** Find a way to satisfy A6 without authenticating the superblock. Think about
   where else an `fsid`-like value could come from, and what "the reader's lookup path" could be
   made to mean. If you conclude it is impossible, say so and give the argument.

2. **Attack P2 from the other side.** P2 says MAC does not give freshness. Construct a scheme that
   gets freshness **using only things already on the disk** — no external hardware, no user-carried
   state. Take the root ring (B1–B5) seriously as raw material. If you conclude it is impossible,
   give the impossibility argument, and state precisely what assumption makes it impossible.

3. **Attack the "reserve before use" idea.** One proposal is: persist `nonce_reserved_upto =
   current + B` *before* handing out any nonce in `[current, reserved_upto)`, so that after a crash
   you restart from `reserved_upto` and simply waste the unused tail. Find what breaks. Consider at
   least: what happens when the persisted reservation record itself is rolled back; what happens
   with two mounts; what it costs on the write path.

   Verified fact you may use (checked in the Linux v6.17 source this session, not from memory):
   fscrypt does **not** draw fresh key material per mount. `fscrypt_prepare_new_inode()` calls
   `get_random_bytes(nonce, FSCRYPT_FILE_NONCE_SIZE)` exactly once, when the inode is created, and
   the value is stored on disk in `struct fscrypt_context_v2 { ...; u8 nonce[16]; }`. So the
   per-file key is derived from persistent, per-file material — not from anything per-mount.

4. **Attack the "fresh salt per mount" idea.** Another proposal is: at every mount, draw a fresh
   128-bit random salt, derive this mount's data key from it, and keep the nonce counter only in
   memory — so rolling back any on-disk counter can no longer cause keystream reuse. Find what
   breaks. Consider at least: blocks written by *earlier* mounts, background work that moves or
   re-reads old blocks without the key, and what happens when two machines mount the same volume.

5. **Construct the worst single-byte edit.** Of every byte in the superblock, which one edit gives
   an attacker the most, and why? Be specific about the mechanism.

6. Finally: **which one sentence in this whole picture is most likely to be wrong, and what
   observation would show it?**

Write in English. Be structured and direct. No preamble.
