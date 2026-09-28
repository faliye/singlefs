### Fact Set 1  
**Segment table (row, length, kind, two-state term, three-state term):**  
- Row 1: length=2, kind=acquiring-instance-number, two-state term=2²-1=4-1=3, three-state term=3²-1=9-1=8  
- Row 2: length=2, kind=record, two-state term=2²-1=3, three-state term=2²-1=3  
- Row 3: length=1, kind=root, two-state term=2¹-1=1, three-state term=2¹-1=1  
- Row 4: length=2, kind=system-configuration-slot, two-state term=3, three-state term=8  
- Row 5: length=2, kind=record, two-state term=3, three-state term=3  
- Row 6: length=1, kind=root, two-state term=1, three-state term=1  
- Row 7: length=2, kind=system-configuration-slot, two-state term=3, three-state term=8  
- Row 8: length=24, kind=unit, two-state term=2²⁴-1=16,777,215, three-state term=16,777,215  
- Row 9: length=2, kind=record, two-state term=3, three-state term=3  
- Row 10: length=1, kind=root, two-state term=1, three-state term=1  
- Row 11: length=2, kind=system-configuration-slot, two-state term=3, three-state term=8  

**Sum of two-state terms:** 3+3+1+3+3+1+3+16,777,215+3+1+3=16,777,239  
Two-state closed form: 16,777,239+1=16,777,240 (matches pinned value)  
**Sum of three-state terms:** 8+3+1+8+3+1+8+16,777,215+3+1+8=16,777,259  
Three-state closed form: 16,777,259+1=16,777,260 (matches pinned value)  

*Observation showing derivation wrong: If the sum of two-state terms plus 1 does not equal 16,777,240 or the sum of three-state terms plus 1 does not equal 16,777,260.*  

---

### Fact Set 2  
**Two-state sum step-by-step:**  
- 9 segments of length 2: 9×(2²-1)=9×3=27  
- 5 segments of length 1: 5×(2¹-1)=5×1=5  
- 1 segment of length 24: 2²⁴-1=16,777,215  
- 1 segment of length 28: 2²⁸-1=268,435,455  
- 1 segment of length 4: 2⁴-1=15  
- 1 segment of length 30: 2³⁰-1=1,073,741,823  
- 1 segment of length 6: 2⁶-1=63  
- Total sum of terms: 27+5+16,777,215+268,435,455+15+1,073,741,823+63=1,358,954,603  
- Two-state closed form: 1,358,954,603+1=1,358,954,604 (matches pinned value)  

**Three-state sum step-by-step:**  
- 6 system-configuration segments (length 2): 6×(3²-1)=6×8=48  
- 3 record segments (length 2): 3×(2²-1)=3×3=9  
- 5 segments of length 1: 5×1=5  
- 1 segment of length 24: 16,777,215  
- 1 segment of length 28: 268,435,455  
- 1 segment of length 4: 15  
- 1 segment of length 30: 1,073,741,823  
- 1 segment of length 6: 63  
- Total sum of terms: 48+9+5+16,777,215+268,435,455+15+1,073,741,823+63=1,358,954,633  
- Three-state closed form: 1,358,954,633+1=1,358,954,634 (matches pinned value)  
- Difference: 1,358,954,634-1,358,954,604=30  
- 6×((3²-1)-(2²-1))=6×(8-3)=30 (matches)  

*Observation showing derivation wrong: If the computed three-state total minus two-state total does not equal 30 or any individual term sum does not match the pinned values.*  

---

### Fact Set 3  
**Per-split reduction formula:** (2^(n+2)-1) - ((2²-1)+(2^n-1)) = 3×2ⁿ - 3  
- For n=24: 3×2²⁴ - 3=3×16,777,216-3=50,331,648-3=50,331,645  
- Three splits (n=24): 50,331,645×3=150,994,935  
- For n=16: 3×2¹⁶ - 3=3×65,536-3=196,608-3=196,605  
- Two splits (n=16): 196,605×2=393,210  
- Total reduction: 150,994,935+393,210=151,388,145 (matches pinned value)  
- New value: 202,113,066-151,388,145=50,724,921 (matches pinned value)  

*Observation showing derivation wrong: If the total reduction does not equal 151,388,145 or the new value does not match 50,724,921.*  

---

### Fact Set 4  
- 2¹⁸-1=262,144-1=262,143 (matches)  
- Two-state closed form: 1+(2²-1)+(2¹⁸-1)=1+3+262,143=262,147  
- Three-state closed form: 1+(3²-1)+(2¹⁸-1)=1+8+262,143=262,152  
- Difference: 262,152-262,147=5  
- (3²-1)-(2²-1)=8-3=5 (matches)  

*Observation showing derivation wrong: If 2¹⁸-1 does not equal 262,143 or the difference between three-state and two-state results does not equal 5.*  

---

### Fact Set 5  
- 2¹⁶=65,536 (matches pinned value)  
- 2¹⁸=262,144 (matches pinned old value)  
- Difference between old (18) and new (16) write counts: 18-16=2 (matches)  

*Observation showing derivation wrong: If 2¹⁶ does not equal 65,536 or 2¹⁸ does not equal 262,144.*  

---

### Fact Set 6  
- First pass: 1 (LAYER0_PARALLEL_START) + ceil(46/16)=3 (LAYER0_PROGRESS) + 3 (LAYER0_FINDINGS, LAYER0_PARALLEL_FINISHED, GOLDEN_TALLY) = 1+3+3=7  
- Second pass: 1 (LAYER0_RESUME) + 1 (LAYER0_PARALLEL_START) + 46 (LAYER0_PROGRESS) + 3 (LAYER0_FINDINGS, LAYER0_PARALLEL_FINISHED, GOLDEN_TALLY) = 1+1+46+3=51  
- Total printed lines: 7+51=58 (matches proposed new value)  
- Progress-file lines: 46 (slice lines) + 1 (header) = 47 (matches proposed value)  

*Observation showing derivation wrong: If ceil(46/16) is not 3 or the total printed lines do not equal 58.*  

---

### Fact Set 7  
- First pair: left=12, right=13; 12+1=13 (matches)  
- Second pair: left=16, right=17; 16+1=17 (matches)  
- Both pairs consistent with additive constant of 1.  

*Observation showing derivation wrong: If left+1 does not equal right for either pair.*  

---

### Fact Set 8  
- Whole-line replacements: 7  
- Appended lines: 9  
- Grand total: 7+9=16  
- Appended sub-counts: 2 (P1) + 3 (Q7) + 1 (Q4) + 3 (crash-injection) = 9 (matches appended total)  
- Proven red lines: 3 (crash-injection lines)  
- Not proven red lines: 16-3=13  

*Observation showing derivation wrong: If the appended sub-counts do not sum to 9 or the count of proven red lines is not 3.*
