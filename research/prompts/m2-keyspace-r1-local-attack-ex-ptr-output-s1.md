leaf_cap floor((16384 - 163) / 112) = floor(16221 / 112) = 144. This would be refuted by a different value for the node size from FACT 1, header size from FACT 2, or leaf-entry width from FACT 3.

fanout floor((16384 - 163) / 86) = floor(16221 / 86) = 188. This would be refuted by a different value for the node size from FACT 1, header size from FACT 2, or internal-entry width from FACT 6.

h_1mib L(1) = ceil(33 / 144) = 1. Height is 1. This would be refuted by a different leaf capacity from FACT 7 or data-unit count for the 1 MiB file from FACT 9.

n_1mib sum of L(1) to L(h) = 1. This would be refuted by a different leaf capacity from FACT 7 or data-unit count for the 1 MiB file from FACT 9.

h_1gib L(1) = ceil(32903 / 144) = 229, L(2) = ceil(229 / 188) = 2, L(3) = ceil(2 / 188) = 1. Height is 3. This would be refuted by a different leaf capacity from FACT 7, fanout from FACT 8, or data-unit count for the 1 GiB file from FACT 9.

n_1gib 229 + 2 + 1 = 232. This would be refuted by a different leaf capacity from FACT 7, fanout from FACT 8, or data-unit count for the 1 GiB file from FACT 9.

h_1tib L(1) = ceil(33692212 / 144) = 233974, L(2) = ceil(233974 / 188) = 1245, L(3) = ceil(1245 / 188) = 7, L(4) = ceil(7 / 188) = 1. Height is 4. This would be refuted by a different leaf capacity from FACT 7, fanout from FACT 8, or data-unit count for the 1 TiB file from FACT 9.

n_1tib 233974 + 1245 + 7 + 1 = 235227. This would be refuted by a different leaf capacity from FACT 7, fanout from FACT 8, or data-unit count for the 1 TiB file from FACT 9.

wc_k1 Level 1: min(1, 233974) = 1, Level 2: min(1, 1245) = 1, Level 3: min(1, 7) = 1, Level 4: 1. Total: 1 + 1 + 1 + 1 = 4. This would be refuted by a different level chain for the 1 TiB file from FACT 10.

wc_k10 Level 1: min(10, 233974) = 10, Level 2: min(10, 1245) = 10, Level 3: min(10, 7) = 7, Level 4: 1. Total: 10 + 10 + 7 + 1 = 28. This would be refuted by a different level chain for the 1 TiB file from FACT 10.

wc_k100 Level 1: min(100, 233974) = 100, Level 2: min(100, 1245) = 100, Level 3: min(100, 7) = 7, Level 4: 1. Total: 100 + 100 + 7 + 1 = 208. This would be refuted by a different level chain for the 1 TiB file from FACT 10.
