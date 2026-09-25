Q1.1  
SHORTEST POSSIBLE HEIGHT: 1  
DERIVATION: n=1 fits within a single leaf node (capacity 477), so it is stored in a level-0 node; height = level + 1 = 0 + 1 = 1.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: no  
This would be refuted by observing a tree with exactly 1 entry and height greater than 1.  

Q1.2  
SHORTEST POSSIBLE HEIGHT: 1  
DERIVATION: n=477 equals leaf capacity, so it fits in a single level-0 node; height = 0 + 1 = 1.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: yes  
This would be refuted by observing that for n=477, the height cannot exceed 1.  

Q1.3  
SHORTEST POSSIBLE HEIGHT: 2  
DERIVATION: n=478 exceeds leaf capacity (477), requiring at least one split; the minimal structure is a root level-1 node with two leaves (height 2).  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: yes  
This would be refuted by observing that for n=478, the height cannot exceed 2.  

Q1.4  
SHORTEST POSSIBLE HEIGHT: 2  
DERIVATION: n=71550 equals 150 * 477 (max entries for height 2), stored in a root level-1 node with 150 leaves; height = 1 + 1 = 2.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: yes  
This would be refuted by observing that for n=71550, the height cannot exceed 2.  

Q1.5  
SHORTEST POSSIBLE HEIGHT: 3  
DERIVATION: n=71551 exceeds 150 * 477, causing the root level-1 node to split into two internal nodes; the new root is level-2, so height = 2 + 1 = 3.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: no  
This would be refuted by observing a tree with n=71551 and height greater than 3.  

Q2.1  
SHORTEST POSSIBLE HEIGHT: 1  
DERIVATION: n=1 fits within a single leaf node (capacity 294), so it is stored in a level-0 node; height = 0 + 1 = 1.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: no  
This would be refuted by observing a tree with exactly 1 entry and height greater than 1.  

Q2.2  
SHORTEST POSSIBLE HEIGHT: 1  
DERIVATION: n=294 equals leaf capacity, so it fits in a single level-0 node; height = 0 + 1 = 1.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: yes  
This would be refuted by observing that for n=294, the height cannot exceed 1.  

Q2.3  
SHORTEST POSSIBLE HEIGHT: 2  
DERIVATION: n=295 exceeds leaf capacity (294), requiring at least one split; the minimal structure is a root level-1 node with two leaves (height 2).  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: yes  
This would be refuted by observing that for n=295, the height cannot exceed 2.  

Q2.4  
SHORTEST POSSIBLE HEIGHT: 2  
DERIVATION: n=42042 equals 143 * 294 (max entries for height 2), stored in a root level-1 node with 143 leaves; height = 1 + 1 = 2.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: yes  
This would be refuted by observing that for n=42042, the height cannot exceed 2.  

Q2.5  
SHORTEST POSSIBLE HEIGHT: 3  
DERIVATION: n=42043 exceeds 143 * 294, causing the root level-1 node to split into two internal nodes; the new root is level-2, so height = 2 + 1 = 3.  
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: no  
This would be refuted by observing a tree with n=42043 and height greater than 3.  

Q3a.1  
VALID STATE UNDER TODAY'S CODE: yes  
HEIGHT IF VALID: 1  
DERIVATION: n=1 is within the single node capacity (812); the tree is always level-0, so height = 0 + 1 = 1.  
This would be refuted by observing a valid state for n=1 with height not equal to 1.  

Q3a.2  
VALID STATE UNDER TODAY'S CODE: yes  
HEIGHT IF VALID: 1  
DERIVATION: n=812 equals the single node capacity; the tree is level-0, so height = 0 + 1 = 1.  
This would be refuted by observing a valid state for n=812 with height not equal to 1.  

Q3a.3  
VALID STATE UNDER TODAY'S CODE: no  
HEIGHT IF VALID: invalid  
DERIVATION: FACT D3 states that exceeding 812 records refuses the publish, so no valid on-disk state exists.  
This would be refuted by observing a valid state for n=813 under today's code.  

Q3b  
Can the height be computed as a function of entry count alone? No. Additional information needed: total device slot count and leaf width W (or fan-out of upper levels), as the tree structure depends on the slot range covered (not entry count), per FACT E1 and E5.  
This would be refuted by observing that the height for a given entry count can be determined without knowing the slot count or W.  

REGIME | ACCOUNTING HEIGHT (FROM Q1) | MAPPING HEIGHT (FROM Q2) | ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED") | TREE TABLE HEIGHT (FROM FACT F1)  
1 | 1 | 1 | 1 | 1  
2 | 1 | 1 | 1 | 1  
3 | 2 | 2 | UNDEFINED | 1  
4 | 2 | 2 | UNDEFINED | 1  
5 | 3 | 3 | UNDEFINED | 1  

REGIME | TWO-TREE SUM (accounting height plus mapping height) | THREE-TREE SUM (TWO-TREE SUM plus allocation height) | FOUR-TREE SUM (THREE-TREE SUM plus tree table height) | CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term) | THIS WOULD BE REFUTED BY  
1 | 2 | 3 | 4 | 2 + accounting_tree_nodes_rewritten for two-tree, 3 + accounting_tree_nodes_rewritten for three-tree, 4 + accounting_tree_nodes_rewritten for four-tree | observing any tree height used in the sums differs from the values in this regime's calculation  
2 | 2 | 3 | 4 | 2 + accounting_tree_nodes_rewritten for two-tree, 3 + accounting_tree_nodes_rewritten for three-tree, 4 + accounting_tree_nodes_rewritten for four-tree | observing any tree height used in the sums differs from the values in this regime's calculation  
3 | 4 | UNDEFINED | UNDEFINED | 4 + accounting_tree_nodes_rewritten for two-tree; three-tree and four-tree sums are undefined due to allocation height being undefined | observing any tree height used in the sums differs from the values in this regime's calculation  
4 | 4 | UNDEFINED | UNDEFINED | 4 + accounting_tree_nodes_rewritten for two-tree; three-tree and four-tree sums are undefined due to allocation height being undefined | observing any tree height used in the sums differs from the values in this regime's calculation  
5 | 6 | UNDEFINED | UNDEFINED | 6 + accounting_tree_nodes_rewritten for two-tree; three-tree and four-tree sums are undefined due to allocation height being undefined | observing any tree height used in the sums differs from the values in this regime's calculation
