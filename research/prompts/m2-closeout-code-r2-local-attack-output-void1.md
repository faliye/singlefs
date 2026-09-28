### Fact Set A
| Row | Old Segment Length | Old (2^n - 1) | New Segment Length | New (2^n - 1) |
|-----|---------------------|---------------|---------------------|---------------|
| 1   | 2                   | 3             | 2                   | 3             |
| 2   | 2                   | 3             | 2                   | 3             |
| 3   | 1                   | 1             | 1                   | 1             |
| 4   | 2                   | 3             | 2                   | 3             |
| 5   | 2                   | 3             | 2                   | 3             |
| 6   | 1                   | 1             | 1                   | 1             |
| 7   | 26                  | 67108863      | 2                   | 3             |
| 8   | 2                   | 3             | 24                  | 16777215      |
| 9   | 1                   | 1             | 2                   | 3             |
| 10  | 2                   | 3             | 1                   | 1             |

**Old sum**: (2^n - 1):**  
3 + 3 + 1 + 3 + 3 + 1 + 67108863 + 3 + 1 + 3 = 67108884  
Closed form: 67108884 + 1 = **67108885** (matches pinned value).  

**New sum (2^n - 1):**  
3 + 3 + 1 + 3 + 3 + 1 + 3 + 16777215 + 3 + 1 = 16777236  
Closed form: 16777236 + 1 = **16777237** (does not match pinned 16777240; gap of 3).  

*Observation showing derivation is wrong:* If the actual closed-form calculation for the new sequence does not yield 16777240 when summing all segment contributions (including all segments), the arithmetic is incorrect.  

---

### Fact Set B  
- First-transaction path: 33 + 2 = **35** (matches pinned value).  
- Warm-up path: 18 + 2 = **20** (matches pinned value).  

*Observation showing derivation is wrong:* If the actual step count after the barrier change does not equal the old count plus 2 for either path, the rule is invalid.  

---

### Fact Set C  
- $2^{18} = 262144$ (pinned old value).  
- $2^{16} = 65536$ (pinned new value).  

*Observation showing derivation is wrong:* If $2^{18} \neq 262144$ or $2^{16} \neq 65536$, the exponentiation is incorrect.  

---

### Fact Set D  
| Ring Size | Start Calculation                     | Pinned Value | Match? | Start mod 64 | Match? |
|-----------|---------------------------------------|--------------|--------|--------------|--------|
| 768 MiB   | 1024 + (768 × 64) = 1024 + 49152 = 50176 | 50176        | Yes    | 0            | Yes    |
| 6 MiB     | 1024 + (6 × 64) = 1024 + 384 = 1408     | 1408         | Yes    | 0            | Yes    |
| 159 MiB   | 1024 + (159 × 64) = 1024 + 10176 = 11200 | 11200        | Yes    | 0            | Yes    |

- $\text{lcm}(812, 64)$:  
  Prime factors: $812 = 2^2 \times 7 \times 29$, $64 = 2^6$ → $\text{lcm} = 2^6 \times 7 \times 29 = 64 \times 203 = 12992$ (matches).  
- $50176 - (3 \times 12992) = 50176 - 38976 = 11200$ (matches).  

*Observation showing derivation is wrong:* If any start value does not match its pinned value, or start mod 64 is nonzero, or $\(812,64) \neq 12992$, or $50176 - 3 \times 12992 \neq 11200$, the arithmetic is incorrect.  

---

### Fact Set E  
- $1048576 / 4096 = 256$ (no remainder).  
- $256 / 3 = 85$ (floor division), remainder $256 - (85 \times 3) = 1$.  
- Final floored quotient: **85** (matches pinned value).  
- "85 admitted, 86 rejected" is consistent with 85 being the maximum admitted count.  

*Observation showing derivation is wrong:* If $1048576 / 4096$ has a remainder, or $256 / 3$ floor division ≠ not 85, or the admitted/rejected counts do not align with the floor result, the calculation is incorrect.  

---

### Fact Set F  
- Fixed structure slot spacing upper bound: $1048576 - 4096 = 1044480$.  
- For $ring\_bytes = 12288$:  
  $12288 / 4096 = 3$, $3 / 3 = 1$ (floor = 1 ≥ 1 → valid).  
- For $ring\_bytes = 12287$:  
  $12287 / 4096 = 2.999...$ (floor = 2), $2 / 3 = 0$ (floor = 0 < 1 → invalid).  
- Smallest passing value: **12288**.  

*Observation showing derivation is wrong:* If $1048576 - 4096 \neq 1044480$, or $12288 / 4096 / 3$ floor is not 1, or $12287 / 4096 / 3$ floor is ≥ 1, the arithmetic is incorrect.