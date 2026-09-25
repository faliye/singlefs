Question 1  
Scenario | Floor-raising target used | Formula or fact you used | Number of valid, non-empty, in-the-candidate-set roots left in the ring | Number of retreat steps a crash-time root-selection fallback still has available  
Today, steady state, admission has never been tight enough to raise F | not computable from the given facts; missing txg value for F and whether it is raised | D16-1-c, D23-14-b | not computable from the given facts; missing number of non-empty valid roots | not computable from the given facts; missing number of non-empty valid roots  
Today, worst case, F has been raised as far as D16-1-d's ceiling formula allows | not computable from the given facts; missing txg values of newest and 4th-newest non-empty valid roots | D16-1-d, D16-1-f, D23-14-a, D23-14-b | not computable from the given facts; missing number of non-empty valid roots | not computable from the given facts; missing number of non-empty valid roots  
Under semantics B, immediately after a completed clean unmount, at the moment the machine is then powered off | not computable from the given facts; missing txg value of newest root | Part 1 description of semantics B, D16-1-c, D16-1-d, D16-1-f | not computable from the given facts; missing whether newest root is non-empty | not computable from the given facts; missing whether newest root is non-empty  

Semantics B's description of raising F all the way up to the newest root is not consistent with the ceiling formula in D16-1-d because the ceiling formula sets F to the minimum of the newest valid root and the 4th-newest non-empty valid root (or oldest if fewer than 4), which is less than the newest root when there are at least 4 non-empty valid roots. This would be falsified by observing that F is raised to the newest root when there are at least 4 non-empty valid roots, contradicting D16-1-d.

Question 2  
k | Which tree you are walking and why | Tree height and fan-out numbers you are using, with their source label | Your upper bound formula in terms of k, height, and fan-out | The resulting number for the two-4-GiB-disk worked example in D8-14-a  
1 | Allocation-record tree; to find blocks with birth generation greater than rollback generation for release based on D5-13 | Height 3, fan-out 169 for internal nodes (D8-14-a) | F^H (k not used in bound) | 4826809  
2 | Allocation-record tree; to find blocks with birth generation greater than rollback generation for release based on D5-13 | Height 3, fan-out 169 for internal nodes (D8-14-a) | F^H (k not used in bound) | 4826809  
3 | Allocation-record tree; to find blocks with birth generation greater than rollback generation for release based on D5-13 | Height 3, fan-out 169 for internal nodes (D8-14-a) | F^H (k not used in bound) | 4826809  
4 | Allocation-record tree; to find blocks with birth generation greater than rollback generation for release based on D5-13 | Height 3, fan-out 169 for internal nodes (D8-14-a) | F^H (k not used in bound) | 4826809  

Genesis-generation-pruned walk means walking the tree to identify blocks with birth generation greater than the rollback generation for direct release, based on D5-13's rule that such blocks are released directly. For the birth-generation field in node headers (D18-18), the gap in whether it is refreshed on rewrite is handled by assuming the worst case where birth-generation must be read from each node, so no pruning occurs and all nodes are read regardless of the field's behavior. This would be falsified by observing that for a specific tree, birth-generation is reduces the number of nodes read below the bound of F^H.

Question 3  
Row | Upper bound expression | Which facts it is built from | Numeric value, or "not computable from the given facts" and what is missing  
number of publishes | min(R*S, B_adm) but B_adm dominates 8 is smaller | D16-1-h for B_adm=8, D22-2-a for R=3, D22-2-b for S range, D23-14-b for ring depth; assuming B_adm cap applies to unmount for upper bound | 8  
number of units per publish | ckpt_cost from D28-4 | D16-9, D28-4 | not computable from the given facts; missing height of allocation-record tree, height of central-mapping tree, and accounting tree's nodes rewritten per publish  
number of units total | (number of publishes) * (units per publish) | As above for both components | not computable from the given facts; as above for units per publish  

This would be falsified by a specific system where the checkpoint reserve pool cost is known and the units per publish is less than the bound, but since it is not computable, the falsification would require measured values contradicting the assumption that B_adm=8 applies to unmount or that the tree heights are high.
