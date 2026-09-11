1. For C1: Tried a code 2 index node written twice in the same checkpoint round. The header stores birth sequence (4 bytes) which increments per write. Scanning the unit header yields birth tree, checkpoint, instance id, and birth sequence. The key built from these matches the referrer's key. No counter found. Tried relocating a code 3 packed container unit. Header is copied unchanged. Scanning the unit header provides correct birth tree, checkpoint, instance id, and birth sequence. Key matches referrers. No counterexample.

2. For C2: Created a reflink between two code 1 data units sharing the same physical block. Both referrers use the same key built from the header's birth tree, checkpoint, and write order. Relocated the shared block. Header unchanged. Referrers still use identical key. No counterexample.

3. For C3: Relocated a code 2 index node. Header copied byte for byte. Birth tree, checkpoint, instance id, and birth sequence remain unchanged. All referrers use the same key. No changes needed to referrers. No counterexample.

4. For C4: Checked dereference path. Map key includes class tag. No extra read needed. Checked snapshot free decision path. Map key provides class tag directly. Checked relocation path. Map key provides class tag. Checked checker path. Map key provides class tag. No extra reads or lookups required. No counterexample.

5. For C5: The current key requires adding a birth sequence field to code 3 headers. This forces the code 3 identity segment table to be updated to store birth sequence. However, a dominated candidate using transaction number (already present in code 3 headers per G2) would not need to update the code 3 identity segment table. Thus, the current key must reopen the code 3 identity segment table while the dominated candidate does not, contradicting C5's claim that the current key's set is a strict subset.

6. For C6: The text required for C6 to fail would be explicit statement that the map key is used as part of the AAD identity or authoritative tree key. The provided facts do not contain such text. Encryption is off in version one and the map is derived state. No counterexample exists.
