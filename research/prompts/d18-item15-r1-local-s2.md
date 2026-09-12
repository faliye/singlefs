1. For A: During a single checkpoint with txg 10, two nodes exist for tree ID 1, level 2, key range [0-100]. Both have birth generation 10 and instance id 1. A root R with txg 10 references one of them. The rule selects both nodes as candidates since they have the same birth generation and key range, failing to pick exactly one.

2. For B: Node number 5 is reused. A node with number 5 has birth 10 and another with number 5 has birth 12. A root R with txg 15 references the birth 10 node. The rule selects the birth 12 node (largest birth <=15) for node number 5, but R references the birth 10 node.

3. For both: During txg 10 fixed-point loop, three nodes for tree ID 1, level 2, key range [0-50] are written with birth 10. Root R references the third node. The first and second nodes are not referenced by any root. The rule selects one of the first or second nodes, which no root references.

4. For both: A node has birth 10. The instance's T_pub is 5. A root R has txg 10. The rule selects this node (birth 10 <=10), but F7 states it is not published because birth > T_pub.
