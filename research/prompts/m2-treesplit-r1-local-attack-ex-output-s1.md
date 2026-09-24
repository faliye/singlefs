EX.leaf_cap: floor((16384 - 163) / 112) = floor(16221 / 112) = 144. This would be refuted by a different header size or leaf entry width.

EX.fanout: floor((16384 - 163) / 110) = floor(16221 / 110) = 147. This would be refuted by a different header size or internal entry width.

EX.h3: For h=1, capacity is 144 which is less than 1000; for h=2, capacity is 144 * 147 = 21168 which is greater than or equal to 1000. So h=2. This would be refuted by a different leaf capacity or internal fanout.

EX.h4: For h=1, capacity is 144 which is less than 10000; for h=2, capacity is 144 * 147 = 21168 which is greater than or equal to 10000. So h=2. This would be refuted by a different leaf capacity or internal fanout.

EX.h5: For h=1, capacity is 144 which is less than 100000; for h=2, capacity is 144 * 147 = 21168 which is less than 100000; for h=3, capacity is 144 * (147 * 147) = 3111696 which is greater than or equal to 100000. So h=3. This would be refuted by a different leaf capacity or internal fanout.

EX.h6: For h=1, capacity is 144 which is less than 1000000; for h=2, capacity is 144 * 147 = 21168 which is less than 1000000; for h=3, capacity is 144 * (147 * 147) = 3111696 which is greater than or equal to 1000000. So h=3. This would be refuted by a different leaf capacity or internal fanout.

EX.n3: nodes_written(N) equals height(N) which is 2. This would be refuted by a different height(N) value.

EX.n4: nodes_written(N) equals height(N) which is 2. This would be refuted by a different height(N) value.

EX.n5: nodes_written(N) equals height(N) which is 3. This would be refuted by a different height(N) value.

EX.n6: nodes_written(N) equals height(N) which is 3. This would be refuted by a different height(N) value.

EX.s_k1: s(1) = ceil(1 / 144) = 1; s(2) = ceil(1 / 147) = 1; s(3) = ceil(1 / 147) = 1; total split count is 1 + 1 + 1 = 3. The tree height grows by 1. This would be refuted by a different leaf capacity, internal fanout, or tree height.

EX.s_k10: s(1) = ceil(10 / 144) = 1; s(2) = ceil(1 / 147) = 1; s(3) = ceil(1 / 147) = 1; total split count is 1 + 1 + 1 = 3. The tree height grows by 1. This would be refuted by a different leaf capacity, internal fanout, or tree height.

EX.s_k100: s(1) = ceil(100 / 144) = 1; s(2) = ceil(1 / 147) = 1; s(3) = ceil(1 / 147) = 1; total split count is 1 + 1 + 1 = 3. The tree height grows by 1. This would be refuted by a different leaf capacity, internal fanout, or tree height.
