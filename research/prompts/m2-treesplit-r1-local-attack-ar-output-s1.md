AR.leaf_cap: floor((16384 - 135) / 20) = floor(16249 / 20) = 812. This would be refuted by a different header size or leaf entry width for the allocation-record tree.
AR.fanout: floor((16384 - 135) / 96) = floor(16249 / 96) = 169. This would be refuted by a different header size or internal entry width for the allocation-record tree.
AR.h3: For N=1000, h=1: 812 < 1000; h=2: 812 * 169 = 137228 ≥ 1000. So h=2. This would be refuted by a different leaf capacity or internal fanout.
AR.h4: For N=10000, h=1: 812 < 10000; h=2: 812 * 169 = 137228 ≥ 10000. So h=2. This would be refuted by a different leaf capacity or internal fanout.
AR.h5: For N=100000, h=1: 812 < 100000; h=2: 812 * 169 = 137228 ≥ 100000. So h=2. This would be refuted by a different leaf capacity or internal fanout.
AR.h6: For N=1000000, h=1: 812 < 1000000; h=2: 812 * 169 = 137228 < 1000000; h=3: 812 * (169 * 169) = 812 * 28561 = 23191532 ≥ 1000000. So h=3. This would be refuted by a different leaf capacity or internal fanout.
AR.n3: nodes_written(N) equals height(N), so for N=1000, height is 2. This would be refuted by a different height(N) calculation.
AR.n4: nodes_written(N) equals height(N), so for N=10000, height is 2. This would be refuted by a different height(N) calculation.
AR.n5: nodes_written(N) equals height(N), so for N=100000, height is 2. This would be refuted by a different height(N) calculation.
AR.n6: nodes_written(N) equals height(N), so for N=1000000, height is 3. This would be refuted by a different height(N) calculation.
AR.s_k1: k=1, h=3; s(1)=ceil(1 / 812)=1; s(2)=ceil(1 / 169)=1; s(3)=ceil(1 / 169)=1; sum=3. Tree height increases. This would be refuted by a different k, leaf capacity, or internal fanout.
AR.s_k10: k=10, h=3; s(1)=ceil(10 / 812)=1; s(2)=ceil(1 / 169)=1; s(3)=ceil(1 / 169)=1; sum=3. Tree height increases. This would be refuted by a different k, leaf capacity, or internal fanout.
AR.s_k100: k=100, h=3; s(1)=ceil(100 / 812)=1; s(2)=ceil(1 / 169)=1; s(3)=ceil(1 / 169)=1; sum=3. Tree height increases. This would be refuted by a different k, leaf capacity, or internal fanout.
