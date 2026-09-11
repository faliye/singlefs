For C1:
- A code 2 unit is written with birth tree id 5, checkpoint number 10, instance id 2, and birth sequence 3.
- The unit header contains only birth tree id (8 bytes), checkpoint number (8 bytes), and instance id (4 bytes).
- Birth sequence (4 bytes) is not stored in the unit header.
- During scan rebuild, the system reads the header and sees birth tree id 5, checkpoint 10, instance id 2.
- The system cannot determine the birth sequence value 3 because it is not in the header.
- The rebuilt key lacks the birth sequence field, so it does not match the map entry which includes birth sequence 3.
- Thus, the key cannot be rebuilt from the unit alone.

For C2:
- Tried to find a case where two referrers to the same physical unit have different keys.
- Considered reflreflink between tree A and tree B to a unit with birth tree id 7, checkpoint 15, instance id 1, birth sequence 0.
- Both referrers point to the same physical unit.
- The key for both referrers is class tag + 7 + 15 + 1 + 0.
- Tried unit relocation, multiple writes in same checkpoint, and different instance mounts.
- In all cases, the unit's header fields (birth tree, checkpoint, instance id, birth sequence) are fixed for the physical unit.
- No case found where referrers to same physical unit have different keys.
- C2 holds.

For C3:
- Tried relocation of a code 3 packed container unit.
- Original unit has header with birth tree 3, checkpoint 4, instance id 5, birth sequence 2.
- Relocation copies the unit byte for byte to new location.
- Unit header remains unchanged.
- Map entry for the new location uses the same key: class tag + 3 + 4 + 5 + 2.
- Referrers point to the new location but the key remains identical.
- Triedreflink and instance switch scenarios also showed key unchanged.
- No case found where relocation changes the key or requires referrer updates.
- C3 holds.

For C4:
- A dereference operation for a code 1 pointer requires the class tag.
- The pointer contains 26 bytes: birth tree (8), checkpoint (8), write order (10).
- Class tag is not part of the pointer.
- To obtain the class tag, the system must read the unit header.
- Example: pointer to code 1 unit at location X.
- System reads unit header at X to get the class tag (1 byte).
- This is an extra read not accounted for by the claim.
- Similarly, for code 2/3 pointers, class tag must be read from header.
- Thus, class tag is not available without extra read.

For C5:
- The current key requires code 2 headers to store birth sequence (4 bytes).
- A dominated candidate key for code 2 could be class tag (1) + birth tree (8) + checkpoint (8) + instance id (4) = 21 bytes.
- This candidate does not require storing birth sequence in the code 2 header.
- The current key must reopen the code 2 header field table to add birth sequence.
- The dominated candidate would not need to reopen the code 2 header field table.
- Thus, the current key must reopen a clause (code 2 header change) that the dominated candidate would not.
- C5 claim fails because the settled clauses are not a strict subset.

For C6:
- The facts do not contain text stating the map key is used as AAD identity.
- For C6 to fail, facts would need to include "the map key is used as AAD identity in encryption" or similar.
- G9 states reverse index entry identity, AAD identity, and authoritative tree key are one definition.
- The map is derived state and not named in G9.
- No text in facts links map key to AAD identity.
- Thus, C6 holds.