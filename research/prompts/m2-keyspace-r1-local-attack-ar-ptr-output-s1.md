leaf_cap: floor((16384 - 135) / 20) = floor(16249 / 20) = 812. This would be refuted by: if the header size from FACT 2 or leaf-entry width from FACT 3 were different.

fanout: floor((16384 - 135) / 86) = floor(16249 / 86) = 188. This would be refuted by: if the internal-entry width from FACT 6 or header size from FACT 2 were different.

h_4gib: L(1) = ceil(211968 / 812) = 261, L(2) = ceil(261 / 188) = 2, L(3) = ceil(2 / 188) = 1, height is 3. This would be refuted by: if the unit-area slot count for 4GiB device was different, or leaf capacity or internal fanout changed.

n_4gib: L(1) = 261, L(2) = 2, L(3) = 1, total node count = 261 + 2 + 1 = 264. This would be refuted by: if the unit-area slot count for 4GiB device was different, or leaf capacity or internal fanout changed.

h_64gib: L(1) = ceil(4144128 / 812) = 5104, L(2) = ceil(5104 / 188) = 28, L(3) = ceil(28 / 188) = 1, height is 3. This would be refuted by: if the unit-area slot count for 64GiB device was different, or leaf capacity or internal fanout changed.

n_64gib: L(1) = 5104, L(2) = 28, L(3) = 1, total node count = 5104 + 28 + 1 = 5133. This would be refuted by: if the unit-area slot count for 64GiB device was different, or leaf capacity or internal fanout changed.

h_1tib: L(1) = ceil(67058688 / 812) = 82585, L(2) = ceil(82585 / 188) = 440, L(3) = ceil(440 / 188) = 3, L(4) = ceil(3 / 188) = 1, height is 4. This would be refuted by: if the unit-area slot count for 1TiB device was different, or leaf capacity or internal fanout changed.

n_1tib: L(1) = 82585, L(2) = 440, L(3) = 3, L(4) = 1, total node count = 82585 + 440 + 3 + 1 = 83029. This would be refuted by: if the unit-area slot count for 1TiB device was different, or leaf capacity or internal fanout changed.

h_16tib: L(1) = ceil(1073691648 / 812) = 1322281, L(2) = ceil(1322281 / 188) = 7034, L(3) = ceil(7034 / 188) = 38, L(4) = ceil(38 / 188) = 1, height is 4. This would be refuted by: if the unit-area slot count for 16TiB device was different, or leaf capacity or internal fanout changed.

n_16tib: L(1) = 1322281, L(2) = 7034, L(3) = 38, L(4) = 1, total node count = 1322281 + 7034 + 38 + 1 = 1329354. This would be refuted by: if the unit-area slot count for 16TiB device was different, or leaf capacity or internal fanout changed.

wc_k1: level 1 rewritten count = min(1, 1322281) = 1, level 2 = min(1, 7034) = 1, level 3 = min(1, 38) = 1, level 4 = 1, total = 1 + 1 + 1 + 1 = 4. This would be refuted by: if the level chain for the 16 TiB device size changed.

wc_k10: level 1 rewritten count = min(10, 1322281) = 10, level 2 = min(10, 7034) = 10, level 3 = min(10, 38) = 10, level 4 = 1, total = 10 + 10 + 10 + 1 = 31. This would be refuted by: if the level chain for the 16 TiB device size changed.

wc_k100: level 1 rewritten count = min(100, 1322281) = 100, level 2 = min(100, 7034) = 100, level 3 = min(100, 38) = 38, level 4 = 1, total = 100 + 100 + 38 + 1 = 239. This would be refuted by: if the level chain for the 16 TiB device size changed.
