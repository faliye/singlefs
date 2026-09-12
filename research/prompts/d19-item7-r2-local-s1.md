1. For A: no associated-data field is read back from the pointer. The identity reference birth tree is part of the node's data authenticated by the parent's MAC, so it is allowed. For B: the birth tree field is read back from the pointer.

2. For A, record type 0 with identity reference birth tree equal to pointer birth tree: no conflict case found. For A, record type 0 with identity reference birth tree 0: no conflict case found. F3 does not check birth tree for type 0, and no other checks are violated in either scenario.

3. For both options: child header birth tree set to 5 (should be 10), pointer birth tree 5, record type 2, container id and generation correct. Identity reference birth tree 5 for A (or absent for B). CRC passes. F3 passes for A (all fields match), F5 passes (pointer equals child), F6 passes. For B, F3 passes (only record type, container id, generation checked), F5 passes, F6 passes. All checks pass despite wrong birth tree.

4. For clone sharing: clone pointer birth tree is 100 (clone txg), child header birth tree is 50 (origin txg). Option A: F3 fails because identity reference birth tree 100 ≠does not equal child header birth tree 50. Option B: F3 passes because birth tree is not checked in identity reference. For rebuild: option A internal node stores birth tree in identity reference, option B does not, producing different bytes.
