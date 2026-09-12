1. A lost node X where the correct children produce logical checksum 0x12345678, but a different set of children with altered data also computes to 0x12345678. For 4-byte checksums this is possible due to higher collision probability, while 8-byte reduces but does not eliminate risk.

2. For unencrypted volume: parent node has location entry pointing to slot 100 with data matching checksum of correct node, but map points to slot 50. Reader uses slot 100 data, checksum passes, but data is wrong. For encrypted volume: same scenario but 4-byte MAC collision allows wrong ciphertext to pass MAC check.

3. Two buffer states for extent node X at b_X: one with update key 5 to 10 followed by key 10 to 20, another with key 10 to 20 followed by key 5 to 10. Both result in same authoritative state and children, but buffer order differs. Serialization of buffer may produce identical bytes if ordered by key.

4. Yes. For an inode tree internal node (which has no buffer), draining does nothing. The rebuild process returns the original node exactly.

5. Gate fails in normal operation if location entries are not zeroed during checksum computation. For example, a correct node with location entries present causes checksum mismatch because the gate expects them zeroed.

6. Nothing stops the rule from picking the older X. The rebuild rule selects any child version born at or before b_X not covered by a later version in the same key range, which includes the older version.
