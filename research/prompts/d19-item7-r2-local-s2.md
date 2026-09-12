1. For A: no associated-data field is read back from the pointer. For B: the birth tree field is read back from the pointer.

2. For A, choice where identity reference birth tree equals pointer's birth tree: no conflict found after testing scenarios including clone sharing, rebuild, and F5 checks. For A, choice where identity reference birth tree is 0: child header birth tree is 5, pointer birth tree is 5, identity reference birth tree is 0. If associated data for encryption uses identity reference birth tree (0) but child header birth tree is 5, decryption MAC verification fails during decryption.

3. For A: child header birth tree is 10 (wrong), pointer birth tree is 10, identity reference birth tree is 10. F3 checks all match (10=10), F5 checks pointer equals child header (10=10), CRC passes. For B: child header birth tree is 10 (wrong), pointer birth tree is 10. Identity reference has no birth tree field. F3 check for record type 2 now only verifies record type, container id, and container birth generation (which pass), F5 checks pointer equals child header (10=10), CRC passes.

4. When rebuilding an internal node with a type 0 entry: option A includes 8 bytes for birth tree in identity reference (set to 0), while option B omits the birth tree field entirely. This results in different byte sequences for the internal node structure.
