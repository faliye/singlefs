QUESTION 1  
(a) For "layer0-first-stream" (19 bytes name): 1 + 32 + 2 + 19 + 32 + 8 = 94 bytes; for "layer0-second-stream" (20 bytes name): 1 + 32 + 2 + 20 + 32 + 8 = 95 bytes; for "layer0-multi-record-publish-stream" (34 bytes name): 1 + 32 + 2 + 34 + 32 + 8 = 109 bytes.  
This would be refuted by measuring the actual key length for any of these example names and finding it differs from the sum of the fields as per fact 2.1.  

(b) For "layer0-first-stream": 94 + 8 = 102 bytes; for "layer0-second-stream": 95 + 8 = 103 bytes; for "layer0-multi-record-publish-stream": 109 + 8 = 117 bytes.  
This would be refuted by measuring the actual violations table key length for any of these example names and finding it differs from the verdict block key length plus 8 bytes.  

(c) Using a 32-byte digest instead of 16-byte increases the key by 32 - 16 = 16 bytes.  
This would be refuted by adding a 32-byte digest to the key and measuring the total key length, finding the increase is not exactly 16 bytes.  

(d) Shortest possible: 1 + 32 + 2 + 1 + 32 + 8 = 76 bytes; maximum: 1 + 32 + 2 + 65535 + 32 + 8 = 65610 bytes.  
This would be refuted by finding a verdict-block key shorter than 76 bytes or longer than 65610 bytes under fact 2.1's rules.  

(e) The stream-or-node name field is the only one whose width is not fixed by the store's code.  
This would be refuted by finding that another field in the key has a variable width not determined by the 2-byte length field or other fixed fields.  

QUESTION 2  
Target A:  
(a) 2^24 - 1 = 16777215 states.  
(b) 16777260 - 16777215 = 45 states.  
(c) 16777260 states.  
(d) 16777215 states.  
P7 scenarios: no improvement = 16777260; one per segment = 1 + 45 = 46.  
This would be refuted by measuring the actual state counts for Target A's data-unit segment or non-data-unit segments and finding them different from the given values.  

Target B:  
(a) 1 state (data-unit segment always fully persisted).  
(b) 25 - 1 = 24 states.  
(c) 25 states.  
(d) 1 state.  
P7 scenarios: no improvement = 25; one per segment = 1 + 24 = 25.  
This would be refuted by measuring the actual state counts for Target B's data-unit segment or total grand total and finding them different.  

Target C:  
(a) 3*(2^24-1) + (2^18-1) + 6*(2^16-1) + (2^20-1) + 6*(2^28-1) = 50331645 + 262143 + 393210 + 1048575 + 1610612730 = 1662648303 states.  
(b) 1662648564 - 1662648303 = 261 states.  
(c) 1662648564 states.  
(d) 1662648303 states.  
P7 scenarios: no improvement = 1662648564; one per segment = 17 + 261 = 278.  
This would be refuted by measuring the actual state counts for Target C's data-unit segments or grand total and finding them different.  

Target D:  
(a) (2^24-1) + (2^28-1) + (2^30-1) = 16777215 + 268435455 + 1073741823 = 1358954493 states.  
(b) 1358954634 - 1358954493 = 141 states.  
(c) 1358954634 states.  
(d) 1358954493 states.  
P7 scenarios: no improvement = 1358954634; one per segment = 3 + 141 = 144.  
This would be refuted by measuring the actual state counts for Target D's data-unit segments or grand total and finding them different.  

QUESTION 3  
Scheme 3.1 total bytes:  
Target A: 16777260; Target B: 25; Target C: 1662648564; Target D: 1358954634.  

Scheme 3.2 using (c) figure:  
Target A: 16777260 * 5 = 83886300; Target B: 25 * 5 = 125; Target C: 1662648564 * 5 = 8313242820; Target D: 1358954634 * 5 = 6794773170.  
Ratio (3.1 / 3.2 using (c)): Target A: 0.2; Target B: 0.2; Target C: 0.2; Target D: 0.2.  

Scheme 3.2 using (d) figure:  
Target A: 16777215 * 5 = 83886075; Target B: 1 * 5 = 5; Target C: 1662648303 * 5 = 8313241515; Target D: 1358954493 * 5 = 6794772465.  
Ratio (3.1 / 3.2 using (d)): Target A: 0.2; Target B: 5.0; Target C: 0.2; Target D: 0.2.  

Scheme "violations only": 0 bytes for all targets (0 violations recorded). General formula: 9 * number of violating states.  

Block-unit choice (i) "slice":  
Target A: ceil(16777260 / 257) = 65282 blocks; Target C: ceil(1662648564 / 25371) = 65536 blocks; Target D: ceil(1358954634 / 20737) = 65536 blocks.  

Block-unit choice (ii) "fixed 65536":  
Target B: single block holds 65536 states, oversized by 65536 - 25 = 65511 states.  

Block-unit choice (iii) "node": No established count of states per node exists since no node tree exists in the real code.  
This would be refuted by measuring the actual block counts for any target and finding them different from the calculated values.  

QUESTION 4  
(a) Idealized card (34359738368 bytes):  
1 byte: 34359738368 states; 7 bytes: 4908534052 states; 66 bytes: 520602096 states; 3.419 bytes: 10050000000 states; 10.26 bytes: 3348902375 states.  

(b) Real card (32607 * 1048576 = 34188955648 bytes):  
1 byte: 34188955648 states; 7 bytes: 4884136521 states; 66 bytes: 517999328 states; 3.419 bytes: 10000000000 states; 10.26 bytes: 3332256885 states.  

For 1-byte cost, batches needed:  
Target A: ceil(16777260 / 34188955648) = 1; Target B: ceil(25 / 34188955648) = 1; Target C: ceil(1662648564 / 34188955648) = 1; Target D: ceil(1358954634 / 34188955648) = 1.  

Fixed 65536 block fits in wgpu default limit (134217728 bytes):  
1 byte: 65536 * 1 = 65536 < 134217728; 7 bytes: 65536 * 7 = 458752 < 134217728; 66 bytes: 65536 * 66 = 4325376 < 134217728; 3.419 bytes: 65536 * 3.419 ≈ 223800 < 134217728; 10.26 bytes: 65536 * 10.26 ≈ 672399 < 134217728. All fit.  
This would be refuted by measuring the actual number of states that fit in the idealized or real card and finding it different from the calculated values, or by measuring the storage buffer limit and finding it different from 134217728 bytes.
