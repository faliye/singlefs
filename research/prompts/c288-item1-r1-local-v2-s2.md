1. For encryption:
   - Candidate A: In a type 2 unit with tree id 4 bytes and level 1 byte, the write sequence starts at offset 63. Placing the birth sequence after the write sequence at offset 67 places it within the plaintext header range (0-69), making it unencrypted.
   - Candidate B: No issue found.
   - Candidate C: No issue found.
   - Candidate D: In a type 2 unit with tree id 4 bytes and level 1 byte, the birth generation starts at offset 47. Placing the birth sequence after the birth generation at offset 55 places it within the plaintext header range (0-69), making it unencrypted.

2. For scan rebuild:
   - All layouts allow the five key components to be read from the header alone when the scan rebuild code is designed for the specific layout. No case found where components cannot be read or are read from wrong bytes.

3. For hidden ordering constraints:
   - F3 requires fixed field order for type 2: tree id, level, key range, birth generation, fsid, write sequence, payload CRC, reserved. Candidate A violates this by inserting birth sequence between write sequence and payload CRC. Candidate B violates this by appending birth sequence after reserved, making reserved not last. Candidate C violates this by inserting birth sequence between payload CRC and reserved. Candidate D violates this by inserting birth sequence between birth generation and fsid.

4. For allocation rule gap:
   - The birth sequence offset is independent of the allocation rule (initial value, step, renumbering). The offset is a fixed header position regardless of how the birth sequence value is generated.

5. For future fields:
   - Candidate A: Adding the 1-byte type tag copy moves all fields after the insertion point. Widening the reserved area does not move the birth sequence.
   - Candidate B: Adding the 1-byte type tag copy moves all fields. Widening the reserved area moves the birth sequence.
   - Candidate C: Adding the 1-byte type tag copy moves all fields after the insertion point. Widening the reserved area moves the birth sequence.
   - Candidate D: Adding the 1-byte type tag copy moves fields after the birth sequence. Widening the reserved area does not move the birth sequence.
