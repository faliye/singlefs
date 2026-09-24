leaf_cap floor((16384 - 135) / 20) = floor(16249 / 20) = floor(812.45) = 812. This would be refuted by a different header size or a different leaf-entry width.

fanout floor((16384 - 135) / 96) = floor(16249 / 96) = floor(169.2604) = 169. This would be refuted by a different header size or a different internal-entry width.

h_4gib L(1) = ceil(211968 / 812) = ceil(261.0443) = 261; L(2) = ceil(261 / 169) = ceil(1.5443) = 2; L(3) = ceil(2 / 169) = ceil(0.0118) = 1. Height is 3. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

n_4gib L(1) = 261; L(2) = 2; L(3) = 1; sum = 261 + 2 + 1 = 264. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

h_64gib L(1) = ceil(4144128 / 812) = ceil(5103.606) = 5104; L(2) = ceil(5104 / 169) = ceil(30.201) = 31; L(3) = ceil(31 / 169) = ceil(0.183) = 1. Height is 3. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

n_64gib L(1) = 5104; L(2) = 31; L(3) = 1; sum = 5104 + 31 + 1 = 5136. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

h_1tib L(1) = ceil(67058688 / 812) = ceil(82584.59) = 82585; L(2) = ceil(82585 / 169) = ceil(488.668) = 489; L(3) = ceil(489 / 169) = ceil(2.893) = 3; L(4) = ceil(3 / 169) = ceil(0.0178) = 1. Height is 4. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

n_1tib L(1) = 82585; L(2) = 489; L(3) = 3; L(4) = 1; sum = 82585 + 489 + 3 + 1 = 83078. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

h_16tib L(1) = ceil(1073691648 / 812) = ceil(1322280.355) = 1322281; L(2) = ceil(1322281 / 169) = ceil(7824.148) = 7825; L(3) = ceil(7825 / 169) = ceil(46.301) = 47; L(4) = ceil(47 / 169) = ceil(0.278) = 1. Height is 4. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

n_16tib L(1) = 1322281; L(2) = 7825; L(3) = 47; L(4) = 1; sum = 1322281 + 7825 + 47 + 1 = 1330154. This would be refuted by a different leaf capacity or internal fanout for the allocation-record tree.

wc_k1 For the 16 TiB device size, height h=4. Level 1: min(1, 1322281) = 1; Level 2: min(1, 7825) = 1; Level 3: min(1, 47) = 1; Level 4: 1. Total = 1 + 1 + 1 + 1 = 4. This would be refuted by a different level chain for the 16 TiB device size.

wc_k10 For the 16 TiB device size, height h=4. Level 1: min(10, 1322281) = 10; Level 2: min(10, 7825) = 10; Level 3: min(10, 47) = 10; Level 4: 1. Total = 10 + 10 + 10 + 1 = 31. This would be refuted by a different level chain for the 16 TiB device size.

wc_k100 For the 16 TiB device size, height h=4. Level 1: min(100, 1322281) = 100; Level 2: min(100, 7825) = 100; Level 3: min(100, 47) = 47; Level 4: 1. Total = 100 + 100 + 47 + 1 = 248. This would be refuted by a different level chain for the 16 TiB device size.
