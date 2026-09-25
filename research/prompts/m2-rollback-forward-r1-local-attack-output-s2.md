Question 1  
| Scenario | Floor-raising target used | Formula or fact you used | Number of valid, non-empty, in-the-candidate-set roots left in the ring | Number of retreat steps a crash-time root-selection fallback still has available |  
| Today, steady state, admission has never been tight enough to raise F | current F (not raised) | D16-1-c and D23-14-b | not computable from the given facts (slots per region S is not fixed, only bounded between 4 and 16) | not computable from the given facts (same reason) |  
| Today, worst case, F has been raised as far as D16-1-d's ceiling formula allows | min(newest persistent valid root, 4th-newest non-empty persistent valid root) | D16-1-d, D16-1-f, D23-14-a, D23-14-b | 4 | 4 |  
| Under semantics B, immediately after a completed clean unmount, at the moment the machine is then powered off | semantics B's "raises F all the way up to the newest root" is inconsistent with D16-1-d's ceiling (which limits F to min(newest, 4th-newest non-empty)) | Part 1 description of semantics B and D16-1-d | 4 (assuming ≥4 non-empty roots exist) | 4 |  

The analysis shows that in steady state, the exact number of roots cannot be determined due to missing slot count details. When F is raised to the ceiling, the candidate set size is 4. Under semantics B, raising F to the newest root conflicts with the ceiling constraint, resulting in a candidate set of 4 roots. This would be falsified by observing that after a clean unmount under semantics B, the rollback floor F exceeds the 4th-newest non-empty root's txg, which violates D16-1-d's ceiling constraint.  

Question 2  
| k | Which tree you are walking and why | Tree height and fan-out numbers you are using, with their source label | Your upper bound formula in terms of k, height, and fan-out | The resulting number for the two-4-GiB-disk worked example in D8-14-a |  
| 1 | allocation-record tree (D5-13's deadlist management relates to it; blocks are released based on birth generation vs previous_snapshot_txg) | height=3 (D8-14-a worked example), fan-out=169 (D8-14-a internal entries) | sum_{i=0}^{h-1} (fan-out)^i | 28731 |  
| 2 | allocation-record tree (D5-13's deadlist management relates to it; blocks are released based on birth generation vs previous_snapshot_txg) | height=3 (D8-14-a worked example), fan-out=169 (D8-14-a internal entries) | sum_{i=0}^{h-1} (fan-out)^i | 28731 |  
| 3 | allocation-record tree (D5-13's deadlist management relates to it; blocks are released based on birth generation vs previous_snapshot_txg) | height=3 (D8-14-a worked example), fan-out=169 (D8-14-a internal entries) | sum_{i=0}^{h-1} (fan-out)^i | 28731 |  
| 4 | allocation-record tree (D5-13's deadlist management relates to it; blocks are released based on birth generation vs previous_snapshot_txg) | height=3 (D8-14-a worked example), fan-out=169 (D8-14-a internal entries) | sum_{i=0}^{h-1} (fan-out)^i | 28731 |  

A genesis-generation-pruned walk skips nodes whose birth generation exceeds the rollback target's txg, using D18-18's birth-generation field (assumed to be the creation txg, as refresh rules are unspecified). The allocation-record tree is walked because D5-13's deadlist rule directly involves block release decisions tied to its structure. The upper bound is the total tree nodes, independent of k. This would be falsified by observing that the number of nodes read during a rollback depends on k, contradicting the independence from k.  

Question 3  
|  | Upper bound expression | Which facts it is built from | Numeric value, or "not computable from the given facts" and what is missing |  
| number of publishes | min(8, ring_slots) | D16-1-h (B_adm=8 for admission-triggered raises), D16-1-d (ceiling), D23-14-b (rollback depth entire ring) | 8 |  
| number of units per publish | ckpt_cost from D28-4 | D16-9, D28-4 | not computable from the given facts (central-mapping tree height and accounting tree nodes rewritten per publish are missing) |  
| number of units total | publishes × units per publish | previous rows | not computable from the given facts (units per publish not computable) |  

The number of publishes is upper-bounded by 8 based on D16-1-h's admission cap, applied to unmount's raise-F string. Units per publish cannot be computed due to missing components in D28-4's formula. Total units also cannot be computed. This would be falsified by observing that the units per publish during an empty publish is a fixed number, contradicting the missing data in D28-4's formula.
