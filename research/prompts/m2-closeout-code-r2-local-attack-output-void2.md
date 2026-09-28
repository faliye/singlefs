### Fact Set A Table
| Row | Old Segment Length | Old (2^n - 1) | New Segment Length | New (2^n - 1) |
|-----|---------------------|----------------|---------------------|----------------|
| 1   | 2                   | 3              | 2                   | 3              |
| 2   | 2                   | 3              | 2                   | 3              |
| 3   | 1                   | 1              | 1                   | 1              |
| 4   | 2                   | 3              | 2                   | 3              |
| 5   | 2                   | 3              | 2                   | 3              |
| 6   | 1                   | 1              | 1                   | 1              |
| 7   | 26                  | 67108863       | 2                   | 3              |
| 8   | 2                   | 3              | 24                  | 16777215       |
| 9   | 1                   | 1              | 2                   | 3              |
| 10  | 2                   | 3              | 1                   | 1              |

**Sum of Old Column (2^n - 1):**  
3+3+1+3+3+1+67108863+3+1+3 = 67108884  
**Closed Form (Old):** 67108884 + 1 = 67108885 (matches pinned value)  

**Sum of New Column (2^n - 1):**  
3+3+1+3+3+1+3+16777215+3+1 = 16777236  
**Closed Form (New):** 16777236 + 1 = 16777237  
**Mismatch:** 16777240 - 16777237 = 3 (gap of 3; likely due to the 10th segment being 1 instead of 2 in the new sequence)  

*Observation showing derivation wrong:* If the sum of (2^n - 1) for the new sequence does not equal 16777239 (since 16777239 + 1 = 16777240), the derivationderivation is incorrect.

---

### Fact Set B
**First-transaction path:** 33 + 2 = 35  
**Warm-up path:** 18 + 2 = 20  
Both match the pinned values (35 and 20).  

*Observation showing derivation wrong:* If the actual steps added per publish are not exactly 2 (e.g., if only 1 barrier is added instead of 2), the counts would not match.

---

### Fact Set C
**2^18:** 262144  
**2^16:** 65536  
Pinned old value: 262144; pinned new value: 65536.  

*Observation showing derivation wrong:* If 2^18 ≠ 262144 or 2^16 ≠ 65536 (e.g., due to incorrect exponentiation).

---

### Fact Set D
| Ring Size | Start (1024 + MiB × 64) | Matches Pinned? | Start mod 64 | On Segment Boundary? |
|-----------|--------------------------|-----------------|--------------|----------------------|
| 768 MiB   | 1024 + 768×64 = 50176    | Yes             | 0            | Yes                  |
| 6 MiB     | 1024 + 6×64 = 1408       | Yes             | 0            | Yes                  |
| 159 MiB   | 1024 + 159×64 = 11200    | Yes             | 0            | Yes                  |

**lcm(812, 64):**  
812 = 2² × 7 × 29; 64 = 2⁶ → lcm = 2⁶ × 7 × 29 = 64 × 203 = 12992 (matches 12992)  
**50176 - 3 × 12992:** 50176 - 38976 = 11200 (matches 11200)  

*Observation showing derivation wrong:* If lcm(812, 64) ≠ 12992 or 50176 - 3×12992 ≠ 11200.

---

### Fact Set E
**1048576 / 4096:** 256 (no remainder)  
**256 / 3 (floor division):** 85 (quotient), remainder 1  
**Final floored quotient:** 85 (matches pinned value)  
**Consistency with "85 admitted, 86 rejected":** Yes (85 is the maximum admitted count).  

*Observation showing derivation wrong:* If 1048576 / 4096 has a remainder or the floor division of 256 / 3 is 85.

---

### Fact Set F
**1 MiB - 4096 bytes:** 1048576 - 4096 = 1044480  
**Smallest ring_bytes where floor(ring_bytes / 4096 / 3) ≥ 1:**  
- 12288 / (4096 × 3) = 12288 / 12288 = 1 → floor = 1 (passes)  
- 12287 / 12288 = 0.999… → floor = 0 (fails)  
**Smallest passing value:** 12288  

*Observation showing derivation wrong:* If 1048576 - 4096 ≠ 1044480 or 12288 does not pass the inequality while 12287 does.