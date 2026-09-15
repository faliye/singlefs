Should a locality hint be added to the self-describing header of every data unit in a copy-on-write filesystem design?

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. Every data unit carries a plaintext self-describing header with five fields: unit type tag 1 byte, tree ID 8, object ID 8, object birth generation 8, anchor offset 8, 33 bytes in total. The same five fields are the associated data of the AEAD when encryption is enabled. The design prefers writing more self-describing bytes over saving space.
F2. A pending proposal (since 2026-09-03) is to add locality_id as a sixth field. Reason: the first segment of the extent key is locality_id and it has only one witness on disk; if an inode record is lost, a scan finds the data units but cannot put them back, because the header gives the object ID and anchor offset but not the key's first segment. The five fields are also the associated-data set, whose composition is a permanent day-one contract, so changing it is a permanent contract change.
F3. Extent key = (locality_id, inode, offset). locality_id is inherited from the parent directory at creation and is deliberately not updated on rename. It is a hint, not part of correctness: it may be wrong or stale. The design also says: being a hint permits not trusting its meaning, not omitting its bits; as the first segment of an address its value is exact and defining. The inode record must store a copy of locality_id; otherwise after a rename the data could only be found by scanning, and a scan rebuild could not re-encode keys.
F4. Benefit: keying by inode alone is 1.46 times slower than with the locality prefix, in one setting only (interleaved creation with an 8-leaf node cache); when the cache holds the whole tree the effect disappears; the benefit decays to about 1.0 as renames accumulate.
F5. A separate settled rule says locality_id must never be part of the associated data, because a field declared "may be wrong" must not be made to carry weight.
F6. In the first version locality_id is always 0, because there are no directories. With encryption on, a plaintext locality_id would leak a semantic category of user objects for the first time. Today locality_id has no plaintext on-disk form in unit headers.
F7. Key encodings, including locality_id, belong to freeze layer 3; nothing is frozen yet.
F8. Invariant: an inode record's locality_id equals the first segment of every extent key of that object. Before an extent-tree rebuild the check reports red or green; after a rebuild both sides come from the same source, so it must report "undecidable", never "pass".
F9. First transaction: the inode record's locality_id is 0; the extent key is (0, 1, 0).
F10. The project rule "do not trade self-containment for space" is not a license to add fields: whether to add a field still needs its own justification.

Candidates:
A. Add locality_id to the five-field set, so it becomes a sixth associated-data field (header 33 to 41 bytes). A rebuild reads the key's first segment from the header. This requires revoking the rule in F5 and changes the first transaction's bytes.
B. Add locality_id to the header as a plaintext hint outside the associated-data set (header plus 8 bytes). A rebuild reads it as a hint. It leaks the semantic category when encryption is on. It changes the first transaction's bytes.
C. Do not add it. When an inode record is lost, the rebuild gives the rebuilt record and that object's extent keys the same locality_id: the first segment of any surviving extent key of the object, or 0 if none survives. A wrong value only costs locality. The invariant in F8 reports "undecidable" after a rebuild. Zero bytes, no contract change.

Questions:
1. Rebuild: for each candidate, build a rebuild history (which units or records are lost, in what order keys are re-encoded, how a reader finds the data) covering: inode record lost; extent tree also rebuilt; both lost; units shared by several writable heads or clones. Can a reader fail to find data, or can one object end up under two different key first segments?
2. Contracts: does F3's sentence about "exact and defining" undercut the reason given in F5? Does candidate A conflict with F5 in a way that cannot be repaired? How serious is B's plaintext leak?
3. Cost: bytes per data unit, first-transaction bytes, and what rename, background relocation and rebuild must do under each candidate.
4. What can the invariant in F8 decide under each candidate, and how would you prove the check can fail?
5. Which candidate would you pick, and what single observation would change your pick?
