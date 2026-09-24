EX.leaf_cap floor((16384 - 163) / 112) = floor(16221 / 112) = 144. This would be refuted by a different header size from FACT 2 or leaf entry width from FACT 3.

EX.fanout floor((16384 - 163) / 110) = floor(16221 / 110) = 147. This would be refuted by a different header size from FACT 2 or internal entry width from FACT 6.

EX.h3 For N=1000: h=1 capacity=144 <1000; h=2 capacity=144*147=21168 >=1000. This would be refuted by a different leaf capacity or internal fanout.

EX.h4 For N=10000: h=1 capacity=144 <10000; h=2 capacity=21168 >=10000. This would be refuted by a different leaf capacity or internal fanout.

EX.h5 For N=100000: h=1 capacity=144; h=2 capacity=21168; h=3 capacity=144*147*147=3111696 >=100000. This would be refuted by a different leaf capacity or internal fanout.

EX.h6 For N=1000000: h=1 capacity=144; h=2 capacity=21168; h=3 capacity=3111696 >=1000000. This would be refuted by a different leaf capacity or internal fanout.

EX.n3 nodes_written(N) = height(N) = 2. This would be refuted by a different height(N) value.

EX.n4 nodes_written(N) = height(N) = 2. This would be refuted by a different height(N) value.

EX.n5 nodes_written(N) = height(N) = 3. This would be refuted by a different height(N) value.

EX.n6 nodes_written(N) = height(N) = 3. This would be refuted by a different height(N) value.

EX.s_k1 s(1)=ceil(1/144)=1; s(2)=ceil(1/147)=1; s(3)=ceil(1/147)=1; total=3; height grows. This would be refuted by a different leaf capacity from FACT 3, internal fanout from FACT 8, or tree height from EX.h6.

EX.s_k10 s(1)=ceil(10/144)=1; s(2)=ceil(1/147)=1; s(3)=ceil(1/147)=1; total=3; height grows. This would be refuted by a different leaf capacity from FACT 3, internal fanout from FACT 8, or tree height from EX.h6.

EX.s_k100 s(1)=ceil(100/144)=1; s(2)=ceil(1/147)=1; s(3)=ceil(1/147)=1; total=3; height grows. This would be refuted by a different leaf capacity from FACT 3, internal fanout from FACT 8, or tree height from EX.h6.
