### Fact Set A Table

| Row | Segment Length (n) | Classification | Full-Enumeration State Count | Quick-Tier State Count |
|-----|---------------------|----------------|----------------------------|------------------------|
| 1   | 2                   | System-configuration (m=2, c=0) | $3^2 \times 2^{0} - 1 = 9 - 1 = 8$ | $3^2 \times 2^{0} \times 1 - 1 = 8$ |
| 2   | 2                   | System-configuration (m=2, c=0) | $3^2 \times 2^{0} - 1 = 8$ | $3^2 \times 2^{0} \times 1 - 1 = 8$ |
| 3   | 1                   | Ordinary (m=0, c=0) | $2^1 - 1 = 1$ | $2^1 - 1 = 1$ |
| 4   | 2                   | System-configuration (m=2, c=0) | $3^2 \times 2^{0} - 1 = 8$ | $3^2 \times 2^{0} \times 1 - 1 = 8$ |
| 5   | 2                   | System-configuration (m=2, c=0) | $3^2 \times 2^{0} - 1 = 8$ | $3^2 \times 2^{0} \times 1 - 1 = 8$ |
| 6   | 1                   | Ordinary (m=0, c=0) | $2^1 - 1 = 1$ | $2^1 - 1 = 1$ |
| 7   | 26                  | Segment A (m=2, c=24) | $3^2 \times 2^{24} - 1 = 9 \times 16777216 - 1 = 150994943$ | $3^2 \times 2^{0} \times 2 - 1 = 17$ |
| 8   | 2                   | Ordinary (m=0, c=0) | $2^2 - 1 = 3$ | $2^2 - 1 = 3$ |
| 9   | 1                   | Ordinary (m=0, c=0) | $2^1 - 1 = 1$ | $2^1 - 1 = 1$ |
| 10  | 2                   | Ordinary (m=0, c=0) | $2^2 - 1 = 3$ | $2^2 - 1 = 3$ |

**Sum of Full-Enumeration Counts (segments only):**  
$8 + 8 + 1 + 8 + 8 + 1 + 150994943 + 3 + 1 + 3 = 150994984$  
**Total Full-Enumeration (add 1):** $150994984 + 1 = 150994985$  
Pinned value: $150994980$ → **Difference: $+5$**  

**Sum of Quick-Tier Counts (segments only):**  
$8 + 8 + 1 + 8 + 8 + 1 + 17 + 3 + 1 + 3 = 58$  
**Total Quick-Tier (add 1):** $58 + 1 = 59$  
Pinned value: $54$ → **Difference: $+5$**  

**Observation for mismatch:** If the actual pinned value for Full-Enumeration was $150994985$, or Quick-Tier was $59$, the derivation would be correct. The current discrepancy of $+5$ suggests either the pinned values are incorrect or the system-configuration write count is misclassified (e.g., segment A may not have 2 torn writes).

---

### Fact Set B Table

| Row | Segment Length (n) | Classification | Full-Enumeration State Count | Quick-Tier State Count |
|-----|---------------------|----------------|----------------------------|------------------------|
| 1   | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 2   | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 3   | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 4   | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 5   | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 6   | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 7   | 26                  | System-configuration (m=2, c=24) | $3^2 \times 2^{24} - 1 = 150994943$ | $17$ |
| 8   | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 9   | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 10  | 26                  | System-configuration (m=2, c=24) | $150994943$ | $17$ |
| 11  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 12  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 13  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 14  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 15  | 18                  | System-configuration (m=2, c=16) | $3^2 \times 2^{16} - 1 = 589823$ | $17$ |
| 16  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 17  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 18  | 18                  | System-configuration (m=2, c=16) | $589823$ | $17$ |
| 19  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 20  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 21  | 18                  | System-configuration (m=2, c=16) | $589823$ | $17$ |
| 22  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 23  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 24  | 26                  | System-configuration (m=2, c=24) | $150994943$ | $17$ |
| 25  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 26  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 27  | 22                  | System-configuration (m=2, c=20) | $3^2 \times 2^{20} - 1 = 9437183$ | $17$ |
| 28  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 29  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 30  | 30                  | System-configuration (m=2, c=28) | $3^2 \times 2^{28} - 1 = 2415919103$ | $17$ |
| 31  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 32  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 33  | 30                  | System-configuration (m=2, c=28) | $2415919103$ | $17$ |
| 34  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 35  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 36  | 30                  | System-configuration (m=2, c=28) | $2415919103$ | $17$ |
| 37  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 38  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 39  | 30                  | System-configuration (m=2, c=28) | $2415919103$ | $17$ |
| 40  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 41  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 42  | 30                  | System-configuration (m=2, c=28) | $2415919103$ | $17$ |
| 43  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 44  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 45  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 46  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 47  | 16                  | System-configuration (m=2, c=14) | $3^2 \times 2^{14} - 1 = 147455$ | $17$ |
| 48  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 49  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 50  | 18                  | System-configuration (m=2, c=16) | $589823$ | $17$ |
| 51  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 52  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 53  | 30                  | System-configuration (m=2, c=28) | $2415919103$ | $17$ |
| 54  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 55  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 56  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 57  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 58  | 16                  | System-configuration (m=2, c=14) | $147455$ | $17$ |
| 59  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 60  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 61  | 18                  | System-configuration (m=2, c=16) | $589823$ | $17$ |
| 62  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |
| 63  | 1                   | Ordinary (m=0, c=0) | $1$ | $1$ |
| 64  | 2                   | Ambiguous (m=2 or m=0) | $8$ or $3$ | $8$ or $3$ |

**Sum of Full-Enumeration Counts (segments only):**  
Calculated as $14961180788$ (using known m=2 segments and assuming 6 length-2 segments as m=2).  
**Total Full-Enumeration (add 1):** $14961180789$  
Pinned value: $14960689284$ → **Difference: $+491505$**  

**Sum of Quick-Tier Counts (segments only):**  
Calculated as $422$ (using known m=2 segments and assuming 6 length-2 segments as m=2).  
**Total Quick-Tier (add 1):** $423$  
Pinned value: $390$ → **Difference: $+33$**  

**Ambiguous rows:** All 28 rows with segment length 2 (rows 1, 2, 4, 5, 8, 11, 13, 14, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 45, 46, 48, 51, 54, 56, 57, 59, 62, 64). These rows could plausibly have $m=2$ or $m=0$, directly affecting the sum.

---

### Fact Set C Table

| Row Name             | Allocation-Record-Tree Term (Formula) | Allocation-Record-Tree Term (alloc= Field) | Sum (alloc + map + acc + tt) | ckpt= Value | Gap |
|----------------------|---------------------------------------|--------------------------------------------|-----------------------------------------------------------|-------------|-----|
| two-4GiB             | $2 \times 2 \times (3 - 1) + 1 = 9$   | $2 + 2 + 1 = 5$                           | $5 + 1 + 1 + 1 = 8$          | $12$        | $-4$ |
| two-1GiB             | $2 \times 2 \times (2 - 1) + 1 = 5$   | $1 + 1 + 1 = 3$                           | $3 + 1 + 1 + 1 = 6$          | $8$         | $-2$ |
| two-4GiB-mapping-4-8 | $2 \times 2 \times (3 - 1) + 1 = 9$   | $2 + 2 + 1 = 5$                           | $5 + 4 + 1 + 1 = 11$         | $24$        | $-13$ |

**Observation for mismatch:** If the allocation-record-tree term matched the `alloc=` field (e.g., $5$ instead of $9$), the sum would align with `ckpt=` for "two-4GiB" (8 vs 12) but still mismatch. The gap likely stems from the formula not accounting for actual node counts in the allocation-record tree.

---

### Fact Set D Table

| Key Width | Computed Value (86 + 2×key_width + 29) | Pinned Value | Match? |
|-----------|----------------------------------------|--------------|--------|
| 8         | $86 + 2 \times 8 + 29 = 131$          | $131$        | Yes    |
| 24        | $86 + 2 \times 24 + 29 = 163$         | $163$        | Yes    |

**Observation for mismatch:** If either computed value differed from the pinned constant (e.g., 131 ≠ 131), the derivation formula would be incorrect. The current match confirms the formula is applied correctly.

---

### Fact Set E Table

| Calculation | Quotient | Remainder | Pinned Value | Match? |
|-------------|----------|-----------|--------------|--------|
| $(4096 - 311) / 56$ | $3785 \div 56 = 67$ | $3785 - 56 \times 67 = 33$ | $67$ | Yes |

**Observation for mismatch:** If the quotient was not $67$ (e.g., $68$), the floor division would be incorrect. The current match confirms the calculation is correct.
