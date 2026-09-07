1. Design A: 100000 * 32768 = 3276800000 bytes. Design B: 100000 / 31 ≈ 3225.806 → 3226 containers, each using 32768 bytes → 3226 * 32768 = 105730048 bytes. Ratio A/B = 3276800000 / 105730048 ≈ 31.0. Files per container: (32768 - 103) / (16 + 1024) = 32665 / 1040 ≈ 31.408 → 31 files. Assumption: container must fit in one unit; this affects container capacity calculation.

2. Design A: 100000 * 32768 = 3276800000 bytes. Design B: (100000 / m) * (103 + 1040m). For m=1: 100000 * 1143 = 114300000. m=8: (100000/8) * 8423 = 105287500. m=31: (100000/31) * 32343 ≈ 104332258. No m ≥ 1 where B equals A, since solving 103/m + 1040 = 32768 gives m ≈ 0.003 < 1. Assumption: device writes actual container size, not unit size; this is critical for B's write count.

3. Design B occupied bytes: (3226 + 100000f) * 32768. Set equal to Design A (100000 * 32768): 3226 + 100000f = 100000 → f = 0.96774. This means 96.77% of files must be overwritten for B to match A's space usage. Assumption: overwritten files create new units and dead slots remain unreclaimed; this directly impacts the space calculation.

4. Design A: 1 file lost per unit. Design B: 31 files lost per unit (average container size). Ratio B/A = 31. Expected files lost per unreadable unit: A=1, B=31. Assumption: containers uniformlyave uniform size; the last container's size has negligible effect on expectation for large datasets.
