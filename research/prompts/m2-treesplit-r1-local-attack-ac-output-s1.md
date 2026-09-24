AC.leaf_cap leaf capacity equals floor((16384 minus 159) divided by 34) which is floor(16225 divided by 34) equals 477. This would be refuted by a change in the header size or leaf entry width.

AC.fanout internal fanout equals floor((16384 minus 159) divided by 108) which is floor(16225 divided by 108) equals 150. This would be refuted by a change in the header size or internal entry width.

AC.h3 height(N) at N equals 1000: h=1 gives 477 * 150^0 = 477 <1000; h=2 gives 477 * 150^1 = 71550 >=1000. So height is 2. This would be refuted by a change in leaf capacity or internal fanout.

AC.h4 height(N) at N equals 10000: h=1 gives 477 <10000; h=2 gives 71550 >=10000. So height is 2. This would be refuted by a change in leaf capacity or internal fanout.

AC.h5 height(N) at N equals 100000: h=1 gives 477 <100000; h=2 gives 71550 <100000; h=3 gives 477 * 150^2 = 10732500 >=100000. So height is 3. This would be refuted by a change in leaf capacity or internal fanout.

AC.h6 height(N) at N equals 1000000: h=1 gives 477 <1000000; h=2 gives 71550 <1000000; h=3 gives 10732500 >=1000000. So height is 3. This would be refuted by a change in leaf capacity or internal fanout.

AC.n3 nodes_written(N) equals height(N) which is 2. This would be refuted by a change in height(N).

AC.n4 nodes_written(N) equals height(N) which is 2. This would be refuted by a change in height(N).

AC.n5 nodes_written(N) equals height(N) which is 3. This would be refuted by a change in height(N).

AC.n6 nodes_written(N) equals height(N) which is 3. This would be refuted by a change in height(N).

AC.s_k1 for k=1 and tree height 3: s(1)=ceiling(1 divided by 477)=1; s(2)=ceiling(1 divided by 150)=1; s(3)=ceiling(1 divided by 150)=1; total splits=1+1+1=3. Tree height grows. This would be refuted by a change in k, leaf capacity, internal fanout, or tree height.

AC.s_k10 for k=10 and tree height 3: s(1)=ceiling(10 divided by 477)=1; s(2)=ceiling(1 divided by 150)=1; s(3)=ceiling(1 divided by 150)=1; total splits=3. Tree height grows. This would be refuted by a change in k, leaf capacity, internal fanout, or tree height.

AC.s_k100 for k=100 and tree height 3: s(1)=ceiling(100 divided by 477)=1; s(2)=ceiling(1 divided by 150)=1; s(3)=ceiling(1 divided by 150)=1; total splits=3. Tree height grows. This would be refuted by a change in k, leaf capacity, internal fanout, or tree height.
