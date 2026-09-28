QUESTION 0  
The total form alpha states are calculated as follows: sum of all five streams (FACT 0B.4) is 3,117,351,422. Subtract the shared prefix overcount (FACT 0B.3) of 33,554,520 to get 3,083,796,902. Add the 0C estimates: 0C.1 (2 × 268,435,467 = 536,870,934), 0C.2 (1,879,048,236), and 0C.3 (1,000,000,000) for 3,415,919,170. Total form alpha is 3,083,796,902 + 3,415,919,170 = 6,499,716,072. This exceeds FACT 0.4 (3,038,380,458) by 3,461,335,614. For 2^136, decimal digits are floor(136 × log10(2)) + 1 = floor(40.936) + 1 = 41. For 2^300, floor(300 × 0.3010) + 1 = floor(90.3) + 1 = 91. The 2^300 contribution dominates all other states combined (66.5 billion) by approximately 80 orders of magnitude (10^90.3 vs 10^9.8). This would be refuted by: the actual decimal digits of 2^300 not being 91 or the combined total of all states in FACT TABLE 0, 0B, and 0C exceeding 10^89.  

QUESTION 0-PRIME  
For by segment: stream "first" smallest nonzero is 1, largest is 150,994,943; shape is mostly tiny units with one enormous outlier. Stream "second" smallest nonzero is 1, largest is 150,994,943; shape is mostly tiny units with multiple large outliers. For by publish: stream "first" smallest nonzero is 8, largest is 150,994,947; shape is mostly tiny units with one enormous outlier. Stream "second" smallest nonzero is 262,163, largest is 150,994,947; shape is mostly zero publishes with a few large non-zero values. For by operation: smallest given is 8 (FACT 0F.1), largest given is 2^136-1 (FACT 0F.4); shape is highly skewed with very small and astronomically large values, but FACT TABLE 0F is only a sample of operations and does not fully represent the distribution. This would be refuted by: the smallest nonzero unit size in segment grouping for stream "first" not being 1 or the largest unit size in publish grouping for stream "second" not being 150,994,947 or the by-operation smallest not being 8 or the largest not being 2^136-1.  

QUESTION 1  
For FACT 1B.1: (a) 37 bytes, (b) 5 × 5 = 25 bytes, (c) 37 / 25 = 1.5. This would be refuted by: the actual state count of Sample A not being 37 or narrow classes not being 5 or "only store classes" byte count not being 25.  
For FACT 1B.2: (a) 37 bytes, (b) 5 × 13 = 65 bytes, (c) 37 / 65 = 0.6. This would be refuted by: the actual state count of Sample A not being 37 or wider classes not being 13 or "only store classes" byte count not being 65.  
For FACT 1B.3: (a) 37 bytes, (b) 5 × 37 = 185 bytes, (c) 37 / 185 = 0.2. This would be refuted by: the actual state count of Sample A not being 37 or maximal classes not being 37 or "only store classes" byte count not being 185.  
For FACT 1B.4: (a) 84 bytes, (b) 5 × 7 = 35 bytes, (c) 84 / 35 = 2.4. This would be refuted by: the actual state count of Sample B not being 84 or narrow classes not being 7 or "only store classes" byte count not being 35.  
For FACT 1B.5: (a) 84 bytes, (b) 5 × 16 = 80 bytes, (c) 84 / 80 = 1.1. This would be refuted by: the actual state count of Sample B not being 84 or wider classes not being 16 or "only store classes" byte count not being 80.  
For FACT 1B.6: (a) 84 bytes, (b) 5 × 84 = 420 bytes, (c) 84 / 420 = 0.2. This would be refuted by: the actual state count of Sample B not being 84 or maximal classes not being 84 or "only store classes" byte count not being 420.  
The "only store classes" scheme's advantage depends more on key definition, as narrow definitions yield higher ratios (more savings) while wider or maximal definitions reduce savings. This would be refuted by: the ratio for narrow key definitions not being higher than for wider or maximal definitions.  

QUESTION 2  
For 2A.1: (a) 4 × (43.5 + 37.3 + 5.7) = 346.0 bytes, (b) floor(34359738368 / 346.0) = 99305602, (c) floor(268435456 / 346.0) = 775825.  
For 2A.2: (a) 4 × (24.3 + 20.6 + 8.0) = 211.6 bytes, (b) floor(34359738368 / 211.6) = 162356992, (c) floor(268435456 / 211.6) = 1268584.  
For 2A.3: (a) 4 × (39.7 + 36.4 + 6.4) = 330.0 bytes, (b) floor(34359738368 / 330.0) = 104120419, (c) floor(268435456 / 330.0) = 813440.  
For 2A.4: (a) 4 × (108.3 + 94.6 + 12.0) = 859.6 bytes, (b) floor(34359738368 / 859.6) = 39960131, (c) floor(268435456 / 859.6) = 312313.  
For 2A.5: (a) 4 × (227.0 + 198.7 + 18.4) = 1776.4 bytes, (b) floor(34359738368 / 1776.4) = 19340205, (c) floor(268435456 / 1776.4) = 151112.  
For 2B.4 segment (2415919103 states) using 2A.4 (c) 312313 states per batch: ceil(2415919103 / 312313) = 7736 batches. This uses average read counts, not worst-case. A state with more read positions than average would refute the batch-count estimate. This would be refuted by: the average read counts for 2A.4 not being 108.3, 94.6, 12.0 or a state with more read positions causing the batch size to be smaller than estimated.  

QUESTION 3  
For 16 writes: ceil(16/64) = 1 word × 8 bytes = 8 bytes; measured (FACT 3B).  
For 24 writes: ceil(24/64) = 1 word × 8 bytes = 8 bytes; not measured (FACT 3B).  
For 26 writes: ceil(26/64) = 1 word × 8 bytes = 8 bytes; measured (FACT 3B).  
For 28 writes: ceil(28/64) = 1 word × 8 bytes = 8 bytes; not measured (FACT 3B).  
This would be refuted by: the actual bitmap size for a 16-write segment not being 8 bytes or for a 24-write segment not being 8 bytes or for a 26-write segment not being 8 bytes or for a 28-write segment not being 8 bytes.  

QUESTION 4  
For 4.1: (a) (64510703 / 37) / 1000 = 1743.5 μs, (b) (35012682 / 37) / 1000 = 946.3 μs, (c) 946.3 / 1743.5 = 0.543, (d) (946.3 × 18 / 37) / 1743.5 = 0.264.  
For 4.2: (a) (3967962616 / 4096) / 1000 = 968.7 μs, (b) (2512991021 / 4096) / 1000 = 613.5 μs, (c) 613.5 / 968.7 = 0.633, (d) (613.5 × 9 / 4096) / 968.7 = 0.014.  
For 4.3: (a) (81293816 / 54) / 1000 = 1505.4 μs, (b) (51945813 / 54) / 1000 = 962.0 μs, (c) 962.0 / 1505.4 = 0.639, (d) (962.0 × 18 / 54) / 1505.4 = 0.214.  
For 4.4: (a) (16482335779 / 4096) / 1000 = 4023.5 μs, (b) (8635287703 / 4096) / 1000 = 2108.2 μs, (c) 2108.2 / 4023.5 = 0.524, (d) (2108.2 × 23 / 4096) / 4023.5 = 0.029.  
For 4.5: (a) (686332409 / 84) / 1000 = 8171.0 μs, (b) (363700795 / 84) / 1000 = 4330.0 μs, (c) 4330.0 / 8171.0 = 0.530, (d) (4330.0 × 58 / 84) / 8171.0 = 0.371.  
(d) is only an upper bound because it assumes every state in a content-dedup class costs exactly the same to check; this would be refuted by two states sharing the same deduplicated unit content but requiring different numbers of checker passes for non-content-dependent reasons. This would be refuted by: the actual distinct unit contents not matching the given counts or states in the same content class having different checker costs.
