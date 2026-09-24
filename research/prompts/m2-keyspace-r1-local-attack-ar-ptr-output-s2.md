leaf_cap floor((16384 - 135) / 20) = floor(16249 / 20) = floor(812.45) = 812. This would be refuted by the header size not being exactly 135 bytes or the leaf entry width not being exactly 20 bytes.

fanout floor((16384 - 135) / 86) = floor(16249 / 86) = floor(188.94186) = 188. This would be refuted by the header size not being exactly 135 bytes or the internal-entry width not being exactly 86 bytes.

h_4gib ceiling(211968 / 812) = ceiling(261.04) = 261; ceiling(261 / 188) = ceiling(1.388) = 2; ceiling(2 / 188) = 1. Height is 3. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 4 GiB device not being exactly 211968.

n_4gib 261 + 2 + 1 = 264. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 4 GiB device not being exactly 211968.

h_64gib ceiling(4144128 / 812) = ceiling(5104.0) = 5104; ceiling(5104 / 188) = ceiling(27.1489) = 28; ceiling(28 / 188) = 1. Height is 3. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 64 GiB device not being exactly 4144128.

n_64gib 5104 + 28 + 1 = 5133. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 64 GiB device not being exactly 4144128.

h_1tib ceiling(67058688 / 812) = ceiling(82585.0) = 82585; ceiling(82585 / 188) = ceiling(439.2819) = 440; ceiling(440 / 188) = ceiling(2.3404) = 3; ceiling(3 / 188) = 1. Height is 4. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 1 TiB device not being exactly 67058688.

n_1tib 82585 + 440 + 3 + 1 = 83029. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 1 TiB device not being exactly 67058688.

h_16tib ceiling(1073691648 / 812) = ceiling(1322281.0) = 1322281; ceiling(1322281 / 188) = ceiling(7033.41) = 7034; ceiling(7034 / 188) = ceiling(37.4149) = 38; ceiling(38 / 188) = 1. Height is 4. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 16 TiB device not being exactly 1073691648.

n_16tib 1322281 + 7034 + 38 + 1 = 1329354. This would be refuted by the leaf capacity not being exactly 812, the fanout not being exactly 188, or the unit-area slot count for a 16 TiB device not being exactly 1073691648.

wc_k1 level 1: min(1, 1322281) = 1; level 2: min(1, 7034) = 1; level 3: min(1, 38) = 1; level 4: 1. Total = 1 + 1 + 1 + 1 = 4. This would be refuted by the height of the tree for the 16 TiB device not being exactly 4, the level chain for the 16 TiB device not matching the computed values, or k not being exactly 1.

wc_k10 level 1: min(10, 1322281) = 10; level 2: min(10, 7034) = 10; level 3: min(10, 38) = 10; level 4: 1. Total = 10 + 10 + 10 + 1 = 31. This would be refuted by the height of the tree for the 16 TiB device not being exactly 4, the level chain for the 16 TiB device not matching the computed values, or k not being exactly 10.

wc_k100 level 1: min(100, 1322281) = 100; level 2: min(100, 7034) = 100; level 3: min(100, 38) = 38; level 4: 1. Total = 100 + 100 + 38 + 1 = 239. This would be refuted by the height of the tree for the 16 TiB device not being exactly 4, the level chain for the 16 TiB device not matching the computed values, or k not being exactly 100.
