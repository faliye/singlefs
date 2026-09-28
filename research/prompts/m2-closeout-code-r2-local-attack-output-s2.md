**Fact Set A**  
Table of segment lengths and (2^n - 1) values:  

| Row | Old Length | Old (2^n - 1) | New Length | New (2^n - 1) |  
|-----|------------|---------------|------------|---------------|  
| 1   | 2          | 3             | 2          | 3             |  
| 2   | 2          | 3             | 2          | 3             |  
| 3   | 1          | 1             | 1          | 1             |  
| 4   | 2          | 3             | 2          | 3             |  
| 5   | 2          | 3             | 2          | 3             |  
| 6   | 1          | 1             | 1          | 1             |  
| 7   | 26         | 67108863      | 2          | 3             |  
| 8   | 2          | 3             | 24         | 16777215      |  
| 9   | 1          | 1             | 2          | 3             |  
| 10  | 2          | 3             | 1          | 1             |  

Old sum of (2^n - 1): 3+3+1+3+3+1+67108863+3+1+3 = 67108884.  
Closed form: 67108884 + 1 = **67108885** (matches pinned value).  

New sum of (2^n - 1): 3+3+1+3+3+1+3+16777215+3+1 = 16777236.  
Closed form: 16777236 + 1 = **16777237** (differs from pinned 16777240 by -3).  
Gap arises because the new sequence requires an 11th segment (length 2, contributing 3) not included in the 10-row table.  

*Observation that would show derivation wrong: If the actual new sequence has only 10 segments and the closed form does not equal 16777240.*  

---

**Fact Set B**  
First-transaction path: 33 + 2 = **35** (matches pinned 35).  
Warm-up path: 18 + 2 = **20** (matches pinned 20).  

*Observation that would show derivation wrong: If the step counts after the barrier change do not increase by exactly 2 for either path.*  

---

**Fact Set C**  
2^18 = 262144 (pinned old value).  
2^16 = 65536 (pinned new value).  

*Observation that would show derivation wrong: If the computed values of 2^18 or 2^16 differ from 262144 or 65536, respectively.*  

---

**Fact Set D**  
- 768 MiB: 1024 + 768×64 = 1024 + 49152 = **50176** (matches). 50176 mod 64 = **0** (satisfies boundary).  
- 6 MiB: 1024 + 6×64 = 1024 + 384 = **1408** (matches). 1408 mod 64 = **0** (satisfies boundary).  
- 159 MiB: 1024 + 159×64 = 1024 + 10176 = **11200** (matches). 11200 mod 64 = **0** (satisfies boundary).  
lcm(812, 64): gcd(812,64)=4 → (812×64)/4 = 12992 (matches).  
50176 - 3×12992 = 50176 - 38976 = **11200** (matches).  

*Observation that would show derivation wrong: If any start value, mod 64 result, lcm, or 50176-3×12992 calculation does not match the pinned values.*  

---

**Fact Set E**  
1048576 / 4096 = **256** (remainder 0).  
256 / 3 = **85** (quotient), remainder **1** (floor division).  
Final floored quotient: **85** (matches pinned value).  
"85 admitted, 86 rejected" is consistent (85 is the maximum admitted count).  

*Observation that would show derivation wrong: If the computed floor value differs from 85, or if 86 records are admitted.*  

---

**Fact Set F**  
1 MiB - 4096 bytes = 1048576 - 4096 = **1044480**.  
For ring_bytes=12288: floor(12288/4096/3) = floor(3/3) = **1** (≥≥ ≥1).  
For ring_bytes=12287: floor(12287/4096) = 2 → floor(2/3) = **0** (<1).  
Smallest passing ring_bytes: **12288**.  

*Observation that would show derivation wrong: If the computed slot spacing upper bound ≠ differs from 1044480, or if 12288 does not satisfy the inequality while 12287 does.*
