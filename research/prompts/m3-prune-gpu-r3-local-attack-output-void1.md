| Q1 | no | This would be falsified if the identity hash string changed despite the flow definition being unchanged. |
| Q2 | H1: 3038380458, 3038380458 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T1. |
| Q2 | H2: 3038380458, 3038380458 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T1. |
| Q2 | H3: 3038380458, 3038380458 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T1. |
| Q3 | H1: 1645871305, 1645871305 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T5. |
| Q3 | H2: 1645871305, 1645871305 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T5. |
| Q3 | H3: 1645871305, 1645871305 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T5. |
| Q4 | H1: 268566570, 268566570 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T18. |
| Q4 | H2: 268566570, 268566570 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T18. |
| Q4 | H3: 268566570, 268566570 | This would be falsified if re-recording or re-checking counts differed between hash candidates for a core change at T18. |
| Q5 | 0, 3038380458 | This would be falsified if re-recording count was non-zero or re-checking count differed from the total states. |
| Q6 | R1: 0, 0 | This would be falsified if re-recording or re-checking occurred for R1 when only the file name changed. |
| Q6 | R2: 1662648564, 1662648564 | This would be falsified if re-recording or re-checking count did not match P2's total states for R2. |
| Q7 | RTX 5090, B1: 53, 645111653, 9844, 1989 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5090, B2: 7, 4886473215, 74540, 4395 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5060 Ti (home), B1: 53, 322279549, 4903, 2401 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5060 Ti (home), B2: 7, 2442502415, 37271, 14159 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5080, B1: 53, 322176036, 4902, 1504 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5080, B2: 7, 2442048273, 37261, 32217 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5060 Ti (second, first card), B1: 53, 322279549, 4903, 2401 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5060 Ti (second, first card), B2: 7, 2442502415, 37271, 14159 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5060 Ti (second, second card), B1: 53, 322279549, 4903, 2401 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q7 | RTX 5060 Ti (second, second card), B2: 7, 2442502415, 37271, 14159 | This would be falsified if the computed batch size or blocks didn't match VRAM and byte calculations. |
| Q8 | first_segment_head: 228191, 1080464, 4.735 | This would be falsified if the computed ns per state or ratio didn't match the given elapsed times. |
| Q8 | second_segment_head: 513282, 4161250, 8.107 | This would be falsified if the computed ns per state or ratio didn't match the given elapsed times. |
| Q9 | 331000, 8 | This would be falsified if the computed time to fill a batch or backlog cap didn't align with the recording rate and checking ratio. |
| Q10 | none | This would be falsified if any node in sections 1-3 had zero or undefined count. |