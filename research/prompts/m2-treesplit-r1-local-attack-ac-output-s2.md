AC.leaf_cap: floor((16384 - 159) / 34) = floor(16225 / 34) = 477. This would be refuted by: a different header size or leaf-entry width.
AC.fanout: floor((16384 - 159) / 108) = floor(16225 / 108) = 150. This would be refuted by: a different header size or internal-entry width.
AC.h3: For h=1, capacity = 477 * 150^0 = 477 < 1000; for h=2, capacity = 477 * 150^1 = 71550 >= 1000. So h=2. This would be refuted by: different leaf capacity or internal fanout.
AC.h4: For h=1, capacity = 477 < 10000; for h=2, capacity = 477 * 150 = 71550 >= 10000. So h=2. This would be refuted by: different leaf capacity or internal fanout.
AC.h5: For h=1, capacity = 477 < 100000; for h=2, capacity = 477 * 150 = 71550 < 100000; for h=3, capacity = 477 * 150^2 = 10732500 >= 100000. So h=3. This would be refuted by: different leaf capacity or internal fanout.
AC.h6: For h=1, capacity = 477 < 1000000; for h=2, capacity = 477 * 150 = 71550 < 1000000; for h=3, capacity = 477 * 150^2 = 10732500 >= 1000000. So h=3. This would be refuted by: different leaf capacity or internal fanout.
AC.n3: nodes_written(N) = height(N) = 2. This would be refuted by: different height(N).
AC.n4: nodes_written(N) = height(N) = 2. This would be refuted by: different height(N).
AC.n5: nodes_written(N) = height(N) = 3. This would be refuted by: different height(N).
AC.n6: nodes_written(N) = height(N) = 3. This would be refuted by: different height(N).
AC.s_k1: s(1) = ceiling(1 / 477) = 1; s(2) = ceiling(1 / 150) = 1; s(3) = ceiling(1 / 150) = 1; total = 1 + 1 + 1 = 3. This would be refuted by: different leaf capacity, internal fanout, or tree height.
AC.s_k10: s(1) = ceiling(10 / 477) = 1; s(2) = ceiling(1 / 150) = 1; s(3) = ceiling(1 / 150) = 1; total = 1 + 1 + 1 = 3. This would be refuted by: different leaf capacity, internal fanout, or tree height.
AC.s_k100: s(1) = ceiling(100 / 477) = 1; s(2) = ceiling(1 / 150) = 1; s(3) = ceiling(1 / 150) = 1; total = 1 + 1 + 1 = 3. This would be refuted by: different leaf capacity, internal fanout, or tree height.
