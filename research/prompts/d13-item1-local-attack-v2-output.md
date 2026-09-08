1. For Arm A: If the nonce ceiling verification is done first but encryption is not yet implemented, the superblock field table changes later (e.g., ceiling value moved from superblock to a different location). Observed: after adding encryption, nonces are reused because the recovery path reads the ceiling from the wrong location. Wasted: time spent verifying a model that no longer matches the actual storage layout.  
For Arm B: Superblock field table changes cause journal record checksum size to shift from 8 bytes to 16 bytes. Observed: recovery applies corrupted journal records because checksum checks use the wrong size. Wasted: effort building barrier and journal record verification that fails when the checksum format changes.  
For Arm C: Superblock field table changes alter slot width from 4KB to 8KB. Observed: root selection uses incorrect slot boundaries after mount, leading to corrupted filesystem structure. Wasted: verification based on fixed slot width that no longer applies.  

2. Yes. Symbolic model passes because it assumes ceiling value is stored and updated. Real implementation writes nonce 5 but omits updating the ceiling value in superblock. After crash, ceiling reads as default zero; next transaction reuses nonce 5.  

3. Yes, Arm C is a strict subpart of Arm B. Verifying Arm C alone buys nothing beyond what Arm B covers because Arm B's verification includes the root selection logic as part of the full publish atomicity process.  

4. Rank: B, C, A. Single property: criticality of ensuring transaction atomicity for the first runnable commit. Ranking flips if the journal record and barriers are not part of the first transaction's critical path or if SMT timeouts occur due to Arm B's complexity.  

5. The complexity of verifying Arm B's full atomicity process (barriers, journal record, root slot) may cause the SMT solver to time out, which is counted as failure per gate stage requirements. This was not covered in previous questions.
