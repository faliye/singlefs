1. A lost node X has two different child versions: version A has location entries set to [1,2,3] and data D, version B has location entries [4,5,6] and same data D. When computing logical checksum with location entries zeroed, both versions produce identical plaintext (D with zeros in location fields). If parent pointer stores checksum for version A, but rebuild uses version B, the gate passes even though location entries are wrong. 4-byte checksum makes collisions more likely than 8-byte, but both can have such cases.

2. No such counterexample exists for unencrypted or encrypted volumes. Any corrupted location entry in an intact node would cause the node's checksum to fail, so the node cannot be intact if its location entries are corrupted.

3. For extent internal node X at birth checkpoint b_X: buffer state 1 has pending updates "add 5 to key K" and buffer state 2 has pending updates "add 3 then add 2 to key K". Both states, when drained, produce identical children with key K value increased by 5. Both states are consistent with same authoritative state and children.

4. No. Draining buffers first empties the buffer before rebuilding, so the rebuilt node has no buffer while original X had a buffer. The nodes cannot be equal.

5. Yes. Two different extent node contents with identical logical checksum (e.g., data differences that cancel out when location entries are zeroed). Gate passes but content is wrong. Also, if system incorrectly computes logical checksum without zeroing location entries during normal operation, gate fails.

6. Nothing stops the rule from picking the older X. The rebuild rule selects versions born at or before b_X not covered by later versions in the same range. If current X is lost and only older X exists, it is selected.
