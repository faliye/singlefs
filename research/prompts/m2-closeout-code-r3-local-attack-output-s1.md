**Fact Set 1**  
| Row | Segment Length | Kind                | Two-State Term | Three-State Term |  
|-----|----------------|---------------------|----------------|------------------|  
| 1   | 2              | system-configuration | 3              | 8                |  
| 2   | 2              | record              | 3              | 3                |  
| 3   | 1              | root                | 1              | 1                |  
| 4   | 2              | system-configuration | 3              | 8                |  
| 5   | 2              | record              | 3              | 3                |  
| 6   | 1              | root                | 1              | 1                |  
| 7   | 2              | system-configuration | 3              | 8                |  
| 8   | 24             | unit                | 16,777,215     | 16,777,215       |  
| 9   | 2              | record              | 3              | 3                |  
| 10  | 1              | root                | 1              | 1                |  
| 11  | 2              | system-configuration | 3              | 8                |  
**Two-state sum**: 3+3+1+3+3+1+3+16,777,215+3+1+3 = 16,777,239. Add 1 → **16,777,240**.  
**Three-state sum**: 8+3+1+8+3+1+8+16,777,215+3+1+8 = 16,777,259. Add 1 → **16,777,260**.  
Both sums match pinned values.  
*Observation showing derivation wrong: If the sum of two-state terms plus 1 does not equal 16,777,240 or three-state terms plus 1 does not equal 16,777,260.*

---

**Fact Set 2**  
**Two-state sum**:  
- 9×3 = 27 (length-2 segments)  
- 5×1 = 5 (length-1 segments)  
- 2⁴−1 = 15 (length-4 segment)  
- 2⁶−1 = 63 (length-6 segment)  
- 2²⁴−1 = 16,777,215 (length-24 segment)  
- 2²⁸−1 = 268,435,455 (length-28 segment)  
- 2³⁰−1 = 1,073,741,823 (length-30 segment)  
Sum: 27+5+15+63+16,777,215+268,435,455+1,073,741,823 = 1,358,954,603. Add 1 → **1,358,954,604**.  
**Three-state sum**:  
- 6×(3²−1) = 48 (system-configuration length-2 segments)  
- 3×(2²−1) = 9 (record length-2 segments)  
- 5×1 = 5 (length-1 segments)  
- 15 (length-4), 63 (length-6), 16,777,215 (length-24), 268,435,455 (length-28), 1,073,741,823 (length-30)  
Sum: 48+9+5+15+63+16,777,215+268,435,455+1,073,741,823 = 1,358,954,633. Add 1 → **1,358,954,634**.  
Difference: 1,358,954,634 − 1,358,954,604 = **30**. 6×(8−3) = 30 → matches.  
*Observation showing derivation wrong: If the computed two-state or three-state total does not match the pinned values, or the difference does not equal 30.*

---

**Fact Set 3**  
Per split reduction: 3×2ⁿ − 3.  
- n=24: 3×16,777,216 − 3 = 50,331,645. Three splits: 3×50,331,645 = 150,994,935.  
- n=16: 3×65,536 − 3 = 196,605. Two splits: 2×196,605 = 393,210.  
Total reduction: 150,994,935 + 393,210 = **151,388,145**.  
New value: 202,113,066 − 151,388,145 = **50,724,921**.  
Both match pinned values.  
*Observation showing derivation wrong: If the computed total reduction does not equal 151,388,145 or the new value does not equal 50,724,921.*

---

**Fact Set 4**  
- 2¹⁸ − 1 = 262,144 − 1 = **262,143** (matches).  
- Two-state closed form: 1 + (2²−1) + 262,143 = 1 + 3 + 262,143 = **262,147**.  
- Three-state closed form: 1 + (3²−1) + 262,143 = 1 + 8 + 262,143 = **262,152**.  
- Difference: 262,152 − 262,147 = **5**. (3²−1) − (2²−1) = 8 − 3 = 5 → matches.  
*Observation showing derivation wrong: If 2¹⁸−1 ≠ 262,143 or the difference does not equal 5.*

---

**Fact Set 5**  
- 2¹⁶ = **65,536** (matches pinned value).  
- 2¹⁸ = **262,144** (matches pinned old value).  
- Difference: 18 − 16 = **2** (matches the two system-configuration writes moved out).  
*Observation showing derivation wrong: If 2¹⁶ ≠ 65,536 or 2¹⁸ ≠ 262,144.*

---

**Fact Set 6**  
- First pass: 1 (PARALLEL_START) + ceil(46/16)=3 (PROGRESS) + 3 (FINDINGS, FINISHED, TALLY) = **7 lines**.  
- Second pass: 1 (RESUME) + 1 (PARALLEL_START) + 46 (PROGRESS) + 3 (FINDINGS, FINISHED, TALLY) = **51 lines**.  
- Total: 7 + 51 = **58 lines** (matches proposed new value in 6b).  
- Progress file: 46 + 1 = **47 lines** (matches proposed value).  
*Observation showing derivation wrong: If ceil(46/16) ≠ 3 or the total lines or progress file count do not match.*

---

**Fact Set 7**  
- First pair: 12 + 1 = **13** (matches right value).  
- Second pair: 16 + 1 = **17** (matches right value).  
Both pairs consistent with constant 1.  
*Observation showing derivation wrong: If left + 1 ≠ right for either pair.*

---

**Fact Set 8**  
- Total lines: 7 (replacements) + 9 (appended) = **16**.  
- Appended sub-counts: 2 + 3 + 1 + 3 = **9** (matches appended total).  
- Proven red: **3** (crash-injection lines).  
- Not proven red: 16 − 3 = **13**.  
*Observation showing derivation wrong: If appended sub-counts do not sum to 9 or the red count does not equal 3.*
