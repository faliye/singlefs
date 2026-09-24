leaf_cap: floor((16384 minus 163) divided by 112) equals floor(16221 divided by 112) equals 144. This would be refuted by a different header size or leaf-entry width.

fanout: floor((16384 minus 163) divided by 110) equals floor(16221 divided by 110) equals 147. This would be refuted by a different header size or internal-entry width.

h_1mib: L(1) equals ceiling(33 divided by 144) equals 1. Since L(1) equals 1, the height h_1mib is 1. n_1mib: sum of L(1) equals 1. This would be refuted by a different leaf capacity or data-unit count for a 1 MiB file.

h_1gib: L(1) equals ceiling(32903 divided by 144) equals 229. L(2) equals ceiling(229 divided by 147) equals 2. L(3) equals ceiling(2 divided by 147) equals 1. Height h_1gib is 3. n_1gib equals 229 plus 2 plus 1 equals 232. This would be refuted by a different leaf capacity, internal fanout, or data-unit count for a 1 GiB file.

h_1tib: L(1) equals ceiling(33692212 divided by 144) equals 233974. L(2) equals ceiling(233974 divided by 147) equals 1592. L(3) equals ceiling(1592 divided by 147) equals 11. L(4) equals ceiling(11 divided by 147) equals 1. Height h_1tib is 4. n_1tib equals 233974 plus 1592 plus 11 plus 1 equals 235578. This would be refuted by a different leaf capacity, internal fanout, or data-unit count for a 1 TiB file.

wc_k1: For k equals 1, level 1 rewritten nodes equals min(1, 233974) equals 1. Level 2 rewritten nodes equals min(1, 1592) equals 1. Level 3 rewritten nodes equals min(1, 11) equals 1. Level 4 rewritten nodes equals 1. Total rewritten nodes equals 1 plus 1 plus 1 plus 1 equals 4. This would be refuted by a different level chain for the 1 TiB tree or a different k.

wc_k10: For k equals 10, level 1 rewritten nodes equals min(10, 233974) equals 10. Level 2 rewritten nodes equals min(10, 1592) equals 10. Level 3 rewritten nodes equals min(10, 11) equals 10. Level 4 rewritten nodes equals 1. Total rewritten nodes equals 10 plus 10 plus 10 plus 1 equals 31. This would be refuted by a different level chain for the 1 TiB tree or a different k.

wc_k100: For k equals 100, level 1 rewritten nodes equals min(100, 233974) equals 100. Level 2 rewritten nodes equals min(100, 1592) equals 100. Level 3 rewritten nodes equals min(100, 11) equals 11. Level 4 rewritten nodes equals 1. Total rewritten nodes equals 100 plus 100 plus 11 plus 1 equals 212. This would be refuted by a different level chain for the 1 TiB tree or a different k.
