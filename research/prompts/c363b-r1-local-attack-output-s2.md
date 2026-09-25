Q1.1
SHORTEST POSSIBLE HEIGHT: 1
DERIVATION: A height of 1 can hold up to 477 entries, which is greater than or equal to 1, so the minimal height is 1.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: No
This would be refuted by: observing a tree with one entry having a root node with level 1 (height 2) that does not collapse to height 1 as per FACT G3.

Q1.2
SHORTEST POSSIBLE HEIGHT: 1
DERIVATION: 477 is exactly the leaf capacity, so height 1 suffices.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that a tree with 477 entries must always have height 1.

Q1.3
SHORTEST POSSIBLE HEIGHT: 2
DERIVATION: 478 exceeds leaf capacity of 477, so height 1 is insufficient; the maximum entries for height 1 is 477.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that all trees with 478 entries have height 2.

Q1.4
SHORTEST POSSIBLE HEIGHT: 2
DERIVATION: 71550 = 150 * 477, the maximum for height 2.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that a tree with 71550 entries cannot have height greater than 2.

Q1.5
SHORTEST POSSIBLE HEIGHT: 3
DERIVATION: 71551 exceeds the maximum entries for height 2 (71550), so minimal height is 3.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that a tree with 71551 entries has height less than 3.

Q2.1
SHORTEST POSSIBLE HEIGHT: 1
DERIVATION: 1 <= leaf capacity 294.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: No
This would be refuted by: observing a tree with one entry having height greater than 1.

Q2.2
SHORTEST POSSIBLE HEIGHT: 1
DERIVATION: 294 is leaf capacity.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that all trees with 294 entries have height 1.

Q2.3
SHORTEST POSSIBLE HEIGHT: 2
DERIVATION: 295 > 294, so height 1 insufficient.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that all trees with 295 entries have height 2.

Q2.4
SHORTEST POSSIBLE HEIGHT: 2
DERIVATION: 42042 = 143 * 294, max for height 2.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that a tree with 42042 entries cannot have height greater than 2.

Q2.5
SHORTEST POSSIBLE HEIGHT: 3
DERIVATION: 42043 > 42042, so height 3 needed.
TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n: Yes
This would be refuted by: observing that a tree with 42043 entries has height less than 3.

Q3a.1
VALID STATE UNDER TODAY'S CODE (YES OR NO): Yes
HEIGHT IF VALID: 1
DERIVATION: Today's code uses a single node (level 0), height = level +1 = 1. 1 <= 812 capacity.
This would be refuted by: observing that a single node cannot hold 1 entry under today's code.

Q3a.2
VALID STATE UNDER TODAY'S CODE (YES OR NO): Yes
HEIGHT IF VALID: 1
DERIVATION: 812 is the capacity of the single node, so valid.
This would be refuted by: observing that 812 entries exceed the node capacity.

Q3a.3
VALID STATE UNDER TODAY'S CODE (YES OR NO): No
HEIGHT IF VALID: 
DERIVATION: FACT D3 states that if entries exceed 812, publish is refused. 813 > 812, so no valid state.
This would be refuted by: observing a valid publish with 813 entries under today's code.

Q3b
No, the height cannot be computed as a function of entry count alone. Additional information needed: the total slot count of the storage device, the leaf width W, and the fan-out of each internal level.
This would be refuted by: observing that for the position-addressed design, a given entry count always results in the same height regardless of other factors.

REGIME 1
ACCOUNTING HEIGHT (FROM Q1): 1
MAPPING HEIGHT (FROM Q2): 1
ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED"): 1
TREE TABLE HEIGHT (FROM FACT F1): 1

REGIME 2
ACCOUNTING HEIGHT (FROM Q1): 1
MAPPING HEIGHT (FROM Q2): 1
ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED"): 1
TREE TABLE HEIGHT (FROM FACT F1): 1

REGIME 3
ACCOUNTING HEIGHT (FROM Q1): 2
MAPPING HEIGHT (FROM Q2): 2
ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED"): undefined
TREE TABLE HEIGHT (FROM FACT F1): 1

REGIME 4
ACCOUNTING HEIGHT (FROM Q1): 2
MAPPING HEIGHT (FROM Q2): 2
ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED"): undefined
TREE TABLE HEIGHT (FROM FACT F1): 1

REGIME 5
ACCOUNTING HEIGHT (FROM Q1): 3
MAPPING HEIGHT (FROM Q2): 3
ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED"): undefined
TREE TABLE HEIGHT (FROM FACT F1): 1

REGIME 1
TWO-TREE SUM (accounting height plus mapping height): 2
THREE-TREE SUM (TWO-TREE SUM plus allocation height): 3
FOUR-TREE SUM (THREE-TREE SUM plus tree table height): 4
CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term): 2 + accounting-tree nodes rewritten; 3 + accounting-tree nodes rewritten; 4 + accounting-tree nodes rewritten
This would be refuted by: observing that the accounting tree height is not 1, or the mapping tree height is not 1, or the allocation record tree height is not 1, or the tree table height is not 1.

REGIME 2
TWO-TREE SUM (accounting height plus mapping height): 2
THREE-TREE SUM (TWO-TREE SUM plus allocation height): 3
FOUR-TREE SUM (THREE-TREE SUM plus tree table height): 4
CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term): 2 + accounting-tree nodes rewritten; 3 + accounting-tree nodes rewritten; 4 + accounting-tree nodes rewritten
This would be refuted by: observing that the accounting tree height is not 1, or the mapping tree height is not 1, or the allocation record tree height is not 1, or the tree table height is not 1.

REGIME 3
TWO-TREE SUM (accounting height plus mapping height): 4
THREE-TREE SUM (TWO-TREE SUM plus allocation height): undefined
FOUR-TREE SUM (THREE-TREE SUM plus tree table height): undefined
CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term): 4 + accounting-tree nodes rewritten; undefined; undefined
This would be refuted by: observing that the accounting tree height is not 2, or the mapping tree height is not 2, or the allocation record tree has a valid height at 813 entries, or the tree table height is not 1.

REGIME 4
TWO-TREE SUM (accounting height plus mapping height): 4
THREE-TREE SUM (TWO-TREE SUM plus allocation height): undefined
FOUR-TREE SUM (THREE-TREE SUM plus tree table height): undefined
CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term): 4 + accounting-tree nodes rewritten; undefined; undefined
This would be refuted by: observing that the accounting tree height is not 2, or the mapping tree height is not 2, or the allocation record tree has a valid height at 813 entries, or the tree table height is not 1.

REGIME 5
TWO-TREE SUM (accounting height plus mapping height): 6
THREE-TREE SUM (TWO-TREE SUM plus allocation height): undefined
FOUR-TREE SUM (THREE-TREE SUM plus tree table height): undefined
CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term): 6 + accounting-tree nodes rewritten; undefined; undefined
This would be refuted by: observing that the accounting tree height is not 3, or the mapping tree height is not 3, or the allocation record tree has a valid height at 813 entries, or the tree table height is not 1.
