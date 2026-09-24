CM.leaf_cap floor((16384 - 169) / 55) = floor(16215 / 55) = 294. This would be refuted by any change to FACT 1, FACT 2, or FACT 3.
CM.fanout floor((16384 - 169) / 113) = floor(16215 / 113) = 143. This would be refuted by any change to FACT 1, FACT 2, or FACT 6.
CM.h3 For h=1: 294 < 1000; h=2: 294 * 143 = 42042 >= 1000. Height is 2. This would be refuted by any change to leaf capacity or internal fanout.
CM.h4 For h=1: 294 < 10000; h=2: 294 * 143 = 42042 >= 10000. Height is 2. This would be refuted by any change to leaf capacity or internal fanout.
CM.h5 For h=1: 294 < 100000; h=2: 294 * 143 = 42042 < 100000; h=3: 294 * 143^2 = 6012006 >= 100000. Height is 3. This would be refuted by any change to leaf capacity or internal fanout.
CM.h6 For h=1: 294 < 1000000; h=2: 294 * 143 = 42042 < 1000000; h=3: 294 * 143^2 = 6012006 >= 1000000. Height is 3. This would be refuted by any change to leaf capacity or internal fanout.
CM.n3 nodes_written(N) = h3 = 2. This would be refuted by any change to height(N).
CM.n4 nodes_written(N) = h4 = 2. This would be refuted by any change to height(N).
CM.n5 nodes_written(N) = h5 = 3. This would be refuted by any change to height(N).
CM.n6 nodes_written(N) = h6 = 3. This would be refuted by any change to height(N).
CM.s_k1 s(1) = ceiling(1 / 294) = 1; s(2) = ceiling(1 / 143) = 1; s(3) = ceiling(1 / 143) = 1; total = 3. Tree height grows by 1. This would be refuted by any change to FACT 1, FACT 2, FACT 3, or FACT 6.
CM.s_k10 s(1) = ceiling(10 / 294) = 1; s(2) = ceiling(1 / 143) = 1; s(3) = ceiling(1 / 143) = 1; total = 3. Tree height grows by 1. This would be refuted by any change to FACT 1, FACT 2, FACT 3, or FACT 6.
CM.s_k100 s(1) = ceiling(100 / 294) = 1; s(2) = ceiling(1 / 143) = 1; s(3) = ceiling(1 / 143) = 1; total = 3. Tree height grows by 1. This would be refuted by any change to FACT 1, FACT 2, FACT 3, or FACT 6.
