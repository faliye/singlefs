leaf_cap: floor((16384 - 163) / 112) = floor(16221 / 112) = 144. This would be refuted by: if the header size from FACT 2 or the leaf-entry width from FACT 3 were different.

fanout: floor((16384 - 163) / 110) = floor(16221 / 110) = 147. This would be refuted by: if the header size from FACT 2 or the internal-entry width from FACT 6 were different.

h_1mib: L(1) = ceiling(33 / 144) = 1. Height is 1. This would be refuted by: if leaf capacity from FACT 7 or N for 1 MiB file size were different.

n_1mib: sum of L(1) = 1. This would be refuted by: if leaf capacity from FACT 7 or N for 1 MiB file size were different.

h_1gib: L(1) = ceiling(32903 / 144) = 229, L(2) = ceiling(229 / 147) = 2, L(3) = ceiling(2 / 147) = 1. Height is 3. This would be refuted by: if leaf capacity from FACT 7, internal fanout from FACT 8, or N for 1 GiB file size were different.

n_1gib: sum of L(1) + L(2) + L(3) = 229 + 2 + 1 = 232. This would be refuted by: if leaf capacity from FACT 7, internal fanout from FACT 8, or N for 1 GiB file size were different.

h_1tib: L(1) = ceiling(33692212 / 144) = 233974, L(2) = ceiling(233974 / 147) = 1592, L(3) = ceiling(1592 / 147) = 11, L(4) = ceiling(11 / 147) = 1. Height is 4. This would be refuted by: if leaf capacity from FACT 7, internal fanout from FACT 8, or N for 1 TiB file size were different.

n_1tib: sum of L(1) + L(2) + L(3) + L(4) = 233974 + 1592 + 11 + 1 = 235578. This would be refuted by: if leaf capacity from FACT 7, internal fanout from FACT 8, or N for 1 TiB file size were different.

wc_k1: level 1 rewritten count is min(1, 233974) = 1, level 2 rewritten count is min(1, 1592) = 1, level 3 rewritten count is min(1, 11) = 1, level 4 rewritten count is 1. Total is 1 + 1 + 1 + 1 = 4. This would be refuted by: if the level chain from FACT 10 for 1 TiB file size or k were different.

wc_k10: level 1 rewritten count is min(10, 233974) = 10, level 2 rewritten count is min(10, 1592) = 10, level 3 rewritten count is min(10, 11) = 10, level 4 rewritten count is 1. Total is 10 + 10 + 10 + 1 = 31. This would be refuted by: if the level chain from FACT 10 for 1 TiB file size or k were different.

wc_k100: level 1 rewritten count is min(100, 233974) = 100, level 2 rewritten count is min(100, 1592) = 100, level 3 rewritten count is min(100, 11) = 11, level 4 rewritten count is 1. Total is 100 + 100 + 11 + 1 = 212. This would be refuted by: if the level chain from FACT 10 for 1 TiB file size or k were different.
