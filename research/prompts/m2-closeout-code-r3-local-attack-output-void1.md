**Fact Set 1**  
| Row | Length | Kind               | Two-state term | Three-state term |  
|-----|--------|--------------------|----------------|------------------|  
| 1   | 2      | system-configuration | 3              | 8                |  
| 2   | 2      | record             | 3              | 3                |  
| 3   | 1      | root               | 1              | 1                |  
| 4   | 2      | system-configuration | 3              | 8                |  
| 5   | 2      | record             | 3              | 3                |  
| 6   | 1      | root               | 1              | 1                |  
| 7   | 2      | system-configuration | 3              | 8                |  
| 8   | 24     | unit               | 16,777,215     | 16,777,215       |  
| 9   | 2      | record             | 3              | 3                |  
| 10  | 1      | root               | 1              | 1                |  
| 11  | 2      | system-configuration | 3              | 8                |  
Sum of two-state terms: 24 + 16,777,215 = 16,777,239. Add 1: **16,777,240** (matches pinned value).  
Sum of three-state terms: 32 + 12 + 16,777,215 = 16,777,259. Add 1: **16,777,260** (matches pinned value).  
*Observation showing derivation wrong: If the sum of two-state terms +1 ≠ 16,777,240 or three-state terms +1 ≠ 16,777,260.*  

**Fact Set 2**  
Two-state sum:  
9×3 = 27, 5×1 = 5, 2²⁴−1 = 16,777,215, 2²⁸−1 = 268,435,455, 2³⁰−1 = 1,073,741,823, 15 + 63 = 78.  
Total: 27 + 5 + 16,777,215 + 268,435,455 + 1,073,741,823 + 78 = 1,358,954,603. Add 1: **1,358,954,604** (matches pinned value).  
Three-state sum:  
6×8 = 48, 3×3 = 9, 5×1 = 5, 2²⁴−1 = 16,777,215, 2²⁸−1 = 268,435,455, 2³⁰−1 = 1,073,741,823, 15 + 63 = 78.  
Total: 48 + 9 + 5 + 16,777,215 + 268,435,455 + 1,073,741,823 + 78 = 1,358,954,633. Add 1: **1,358,954,634** (matches pinned value).  
Difference: 1,358,954,634 − 1,358,954,604 = **30** (equals 6×(8−3) = 30).  
*Observation showing derivation wrong: If the two-state sum +1 ≠ 1,358,954,604 or three-state sum +1 ≠ 1,358,954,634 or difference ≠ 30.*  

**Fact Set 3**  
Per-split reduction formula: 3×2ⁿ − 3.  
n=24: 3×16,777,216 − 3 = 50,331,645. Three splits: 3×50,331,645 = 150,994,935.  
n=16: 3×65,536 − 3 = 196,605. Two splits: 2×196,605 = 393,210.  
Total reduction: 150,994,935 + 393,210 = **151,388,145** (matches pinned value).  
Old value − reduction: 202,113,066 − 151,388,145 = **50,724,921** (matches pinned value).  
*Observation showing derivation wrong: If total reduction ≠ 151,388,145 or old − reduction ≠ 50,724,921.*  

**Fact Set 4**  
2¹⁸ − 1 = 262,144 − 1 = **262,143** (matches).  
Two-state closed form: 1 + 3 + 262,143 = **262,147**.  
Three-state closed form: 1 + (9 − 1) + 262,143 = 1 + 8 + 262,143 = **262,152**.  
Difference: 262,152 − 262,147 = **5** (equals (9−1) − (4−1) = 8−3 = 5).  
*Observation showing derivation wrong: If 2¹⁸−1 ≠ 262,143 or difference ≠ 5.*  

**Fact Set 5**  
2¹⁶ = **65,536** (matches pinned value).  
2¹⁸ = **262,144** (matches pinned value).  
Old write count (18) − new (16) = **2** (matches).  
*Observation showing derivation wrong: If 2¹⁶ ≠ 65,536 or 2¹⁸ ≠ 262,144 or difference ≠ 2.*  

**Fact Set 6**  
First pass: 1 + ceil(46/16)=3 + 3 = **7**.  
Second pass: 1 + 1 + 46 + 3 = **51**.  
Total lines: 7 + 51 = **58** (matches proposed value).  
Progress file: 46 + 1 = **47** (matches proposed value).  
*Observation showing derivation wrong: If total lines ≠ 58 or progress file ≠ 47.*  

**Fact Set 7**  
Left 12 + 1 = **13** (matches right value).  
Left 16 + 1 = **17** (matches right value).  
Both pairs consistent with +1.  
*Observation showing derivation wrong: If left + 1 ≠ right for either pair.*  

**Fact Set 8**  
Wholewhole-line replacements (7) + appended lines (9) = **16**.  
Appended sub-counts: 2 + 3 + 1 + 3 = **9** (matches appended total).  
Proven red: **3** (crash-injection lines).  
Not proven red: 16 − 3 = **13**.  
*Observation showing derivation wrong: If appended sub-counts ≠ 9 or red count ≠ 3.*