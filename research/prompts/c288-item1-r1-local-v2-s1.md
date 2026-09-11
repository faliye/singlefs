1. Encryption  
Layout A: For type 2, birth sequence is placed after write sequence. With encryption on, plaintext part is 0-69 bytes. Birth sequence is at offset 74-77, which is beyond 69, so encrypted. No plaintext piece between ciphertext or checksum range error. For type 3, birth sequence at 103-106, beyond 69, encrypted.  
Layout B: For type 2, birth sequence appended after reserved bytes. Placed at 80-83, beyond 69, encrypted. No issues. For type 3, birth sequence at 103-106, encrypted.  
Layout C: For type 2, birth sequence after payload CRC and before reserved. Placed at 78-81, beyond 69, encrypted. No issues. For type 3, birth sequence at 103-106, encrypted.  
Layout D: For type 2, birth sequence after birth generation. Placed at 62-65. This is within 0-69 plaintext range. Birth sequence is in plaintext, but class identity segment must be encrypted per F5. This violates encryption rules.  

2. Scan rebuild  
No counterexamples found for any layout. All five key components (type tag at offset 6, birth tree at fixed offset, birth txg at fixed offset, instance id from write sequence start, birth sequence at candidate offset) can be read from the header alone for all layouts.  

3. Hidden ordering constraints  
Layout B is ruled out. F3 specifies fixed field order for type 2: tree id, level, key range, birth generation, fsid, write sequence, payload CRC, reserved. Reserved must be the last field. Layout B appends birth sequence after reserved, making reserved no longer last. This violates field order.  

4. Allocation rule gap  
No case exists. The birth sequence offset in the header is fixed and independent of allocation rules (initial value, step, renumbering). F9 describes value generation rules, not storage location. Offset is determined by layout, not allocation rules.  

5. Future fields  
Layout A:  
- Adding F7 (1-byte type tag for type 2): Fields after insertion point move (e.g., if inserted before birth sequence, birth sequence, payload CRC, reserved move).  
- Widening reserved: Reserved is last field, so no fields move.  
Layout B:  
- Adding F7: Fields after insertion point move.  
- Widening reserved: Birth sequence moves since it is after reserved.  
Layout C:  
- Adding F7: Fields after insertion point move.  
- Widening reserved: Reserved is last field, so no fields move.  
Layout D:  
- Adding F7: Fields after insertion point move.  
- Widening reserved: Reserved is last field, so no fields move.
