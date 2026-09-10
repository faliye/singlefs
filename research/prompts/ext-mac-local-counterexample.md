Role: you are the counterexample leg of a three-way design review. Your job is to
construct a concrete counterexample, not to agree. Answer in English. Do not use any
markdown emphasis in your answer.

Context. This is a from-scratch copy-on-write filesystem. Data lives in self-describing
units. Every unit may carry a small reserved region called the extension point, which a
third-party medium maintainer may use for its own purposes. The core filesystem does not
understand its contents.

Established rules, quoted:

R1. The extension point must fall inside the checksum and authentication coverage. If it
falls outside, that region is a corruption blind spot, and when encryption is on it is
unauthenticated bytes, which is directly an attack surface.

R2. The checker must not make any judgement about the contents of the extension point.
It is opaque to the core filesystem.

R3. The size of the extension point is declared by the format, in the superblock of that
line. General tools, namely scrub, scan-rebuild and rescue, must be able to skip over it.

R4. A pointer stored inside the extension point must target space that goes through this
project's allocation and accounting. The checker must be able to verify that the range
targeted by the extension point is recorded as allocated, even though it does not
understand the contents.

R5. A pointer stored inside the extension point may not point at core-layer objects. It
may only point inside the extension point's own quota.

R6. Each block pointer carries two integrity fields: a MAC, which needs the key and
provides authentication, and a separate ciphertext checksum of 4 bytes, which needs no key
and detects corruption. The ciphertext checksum is stored once per physical location,
alongside a 4 byte device id and a 6 byte physical offset.

R7. Decided 2026-09-07 by the project owner: when encryption is on, the keyless side is
read-only and detect-only. It cannot relocate, cannot evacuate, cannot reclaim. Detection
is always possible because the pointer carries the keyless ciphertext checksum. This is
the only capability the keyless side retains.

R8. When encryption is on, the plaintext header whitelist is exactly: magic, format
version, flags, nonce id, length, MAC. It does not include the unit's own logical address,
generation, tree id, or owning object. Therefore a keyless party cannot rebuild the index;
it can only scrub.

History of the question. On 2026-09-06 the following rationale was withdrawn: "the whole
unit's checksum and authentication already cover the extension point, therefore the
extension point needs no integrity field of its own". The stated reason for withdrawing it
was that it is mutually exclusive with "the keyless side must be able to relocate blocks",
because when encryption is on, authentication means the MAC, the MAC covers ciphertext,
and being inside the coverage means the extension point is ciphertext, which a keyless
party cannot read. On 2026-09-07 rule R7 was decided, which removes the keyless
relocation requirement entirely.

Candidate conclusion under test: the withdrawal is void. R1 stands again as the rationale,
so the extension point does not need its own 4 byte ciphertext checksum. Its integrity is
covered by the whole-unit MAC on the keyed side and by the per-location 4 byte ciphertext
checksum on the keyless side, because the extension point sits inside the unit ciphertext
and that 4 byte checksum covers the whole ciphertext at that physical location.

Your task. Find a counterexample. Concretely:

1. Name a reader or a scenario that still needs to read or verify the extension point
   bytes without the key, after R7 removed keyless relocation. For each candidate reader
   you consider, say whether it survives and whether it must touch those bytes.
2. R3 says general tools must be able to skip over the extension point. Skipping requires
   knowing its length. State where that length comes from and whether obtaining it
   requires reading anything that is encrypted. If skipping is possible without the key
   but verifying is not, say whether that asymmetry breaks anything.
3. R4 says the checker must verify that the range targeted by the extension point is
   recorded as allocated. When encryption is on and the checker has no key, can it do
   this? If not, is that a pre-existing hole or one created by the candidate conclusion?
4. Consider partial corruption. The per-location ciphertext checksum is 4 bytes and covers
   the whole ciphertext at that location. Does covering the extension point only at
   whole-unit granularity lose anything that a dedicated per-extension-point checksum
   would have provided? Give a concrete failure scenario or say there is none.

If you cannot construct a counterexample, say so explicitly and list which classes of
reader you searched. Do not invent facts that are not in this prompt.
