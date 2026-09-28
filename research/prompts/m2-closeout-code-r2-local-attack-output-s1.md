### Fact Set A Table
| Row | Old Segment Length | Old (2^n - 1) | New Segment Length | New (2^n - 1) |
|-----|--------------------------------------|---------------|--------------------|---------------|
| 1   | 2                  | 3             | 2                  | 3             |
| 2   | 2                  | 3             | 2                  | 3             |
| 3   | 1                  | 1             | 1                  | 1             |
| 4   | 2                  | 3             | 2                  | 3             |
| 5   | 2                  | 3             | 2                  | 3             |
| 6   | 1                  | 1             | 1                  | 1             |
| 7   | 26                 | 67108863      | 2                  | 3             |
| 8   | 2                  | 3             | 24                 | 16777215      |
| 9   | 1                  | 1             | 2                  | 3             |
| 10  | 2                  | 3             | 1                  | 1             |

**Sum of old column (2^n - 1):**  
3 + 3 + 1 + 3 + 3 + 1 + 67108863 + 3 + 1 + 3 = 67108884  
**Closed-form (sum + 1):** 67108884 + 1 = 67108885 (matches pinned value)  

**Sum of new column (2^n - 1):**  
3 + 3 + 1 + 3 + 3 + 1 + 3 + 16777215 + 3 + 1 = 16777239  
**Closed-form (sum + 1):** 16777239 + 1 = 16777240 (matches pinned value)  

**Observation to show derivation wrong:** If the sum of (2^n - 1) for any segment does not yield the computed values when recalculated, or if the closed-form sum + 1 deviates from the pinned values.

---

### Fact Set B
- First-transaction path: 33 + 2 = 35 (matches pinned 35)  
- Warm-up path: 18 + 2 = 20 (matches pinned 20)  

**Observation to show derivation wrong:** If the step count after the barrier change is not exactly 2 more than the old count for either path.

---

### Fact Set C
- $ 2^{18} = 262144 $  
- $ 2^{16} = 65536 $  
- Pinned old value: 262144 (2^18), new value: 65536 (2^16)  

**Observation to show derivation wrong:** If $ 2^{18} \neq 262144 $ or $ 2^{16} \neq 65536 $.

---

### Fact Set D
| Ring Size | Computed Start | Matches Pinned | Start mod 64 | Is 0? |
|-----------|----------------|----------------|--------------|-------|
| 768 MiB   | 1024 + 768×64 = 50176 | Yes | 50176 % 64 = 0 | Yes |
| 6 MiB     | 1024 + 6×64 = 1408 | Yes | 1408 % 64 = 0 | Yes |
| 159 MiB   | 1024 + 159×64 = 11200 | Yes | 11200 % 64 = 0 | Yes |

- $ \text{lcm}(812, 64) $:  
  Prime factors: 812 = $ 2^2 \times 7 \times 29 $, 64 = $ 2^6 $ → lcm = $ 2^6 \times 7 \times 29 = 64 \times 203 = 12992 $ (matches)  
- $ 50176 - 3 \times 12992 = 50176 - 38976 = 11200 $ (matches)  

**Observation to show derivation wrong:** If any ring’s start value mod 64 is nonzero, or if lcm(812, 64) ≠ 12992, or if $ 50176 - 3 \times 12992 \neq 11200 $.

---

### Fact Set E
- $ 1048576 / 4096 = 256 $ (no remainder)  
- $ 256 / 3 = 85 $ (quotient), remainder $ 256 - (85 \times 3) = 1 $  
- Final floored quotient: 85 (matches pinned 85)  
- "85 admitted, 86 rejected" is consistent because the floor value (85) is the maximum admitted count.  

**Observation to show derivation wrong:** If the division $ 1048576 / 4096 $ has a remainder, or if $ \text{floor}(256 / 3) \neq 85 $, or if 86 records are not rejected.

---

### Fact Set F
- $ 1 \text{ MiB} - 4096 \text{ bytes} = 1048576 - 4096 = 1044480 $  
- For $ \text{ring\_bytes} = 12288 $:  
  $ 12288 / 4096 = 3 $, $ 3 / 3 = 1 $ → $ \text{floor}(1) = 1 $ (satisfies ≥1)  
- For $ \text{ring\_bytes} = 12287 $:  
  $ 12287 / 4096 = 2.999... $, $ \text{floor}(2.999...) = 2 $, $ 2 / 3 = 0.666... $, $ \text{floor}(0.666...) = 0 $ (fails ≥1)  
- Smallest passing ring_bytes: 12288  

**Observation to show derivation wrong:** If $ 1048576 - 4096 \neq 1044480 $, or if $ \text{floor}(12288 / 4096 / 3) \neq 1 $, or if $ \text{floor}(12287 / 4096 / 3) \neq 0 $.
