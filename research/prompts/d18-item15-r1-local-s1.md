1. For A:  
   In checkpoint txg 100, a key range [10-20] is split into two nodes [10-15] and [16-20]. During the fixed-point loop, the [10-15] node is rewritten again. Two class 2 nodes exist for tree ID X, level 2, key range [10-15], both with birth generation 100. A root R with txg 100 references one of them. The rule for A picks both nodes as candidates since they have the same birth <= 100 and cover the key range.  

2. For B:  
   In checkpoint txg 100, a node with node number 5 is written in fixed-point round 1 (birth 100, instance id 1, round 1). Later in the same checkpoint, the same node number 5 is written again in round 2 (birth 100, instance id 1, round 2). A root R with txg 100 references the round 1 version. The rule for B picks the round 2 version because it has a larger fixed-point round, which is not the version referenced by R.  

3. For both:  
   During the fixed-point loop of checkpoint txg 100, a class 2 node is written with birth 100. The instance's T_pub has not been set yet, so the node is unpublished. A full scan of on-disk nodes includes this node. The rule for A or B picks it because birth <= 100, but no root references it since the checkpoint is not yet published.  

4. For both:  
   A root R with txg 100 references a node with birth 100. However, the instance's T_pub has not been set during the fixed-point loop, so F7 states the node is not published. This violates F7 because the root references an unpublished node.
