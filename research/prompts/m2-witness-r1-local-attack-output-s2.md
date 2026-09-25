1. Candidate A  
width_A = 1 + 16 * 4 = 65  
occupied_A = 481 + 65 = 546  
past_512_A: 546 > 512 → yes  
margin_512_A = 512 - 546 = -34  
past_4096_A: 546 > 4096 → no  
margin_4096_A = 4096 - 546 = 3550  
occupied_A_with_B = 481 + 8 + 65 = 554  
margin_512_with_B_A = 512 - 554 = -42  
extra_writes_per_rollback_A = 0 (by F11)  
extra_reads_per_mount_A = 0 (by F12)  
extra_disk_bytes_A = 65 * 2 * 2 = 260  

width_A = 1 + 16 * 3 = 49  
occupied_A = 481 + 49 = 530  
past_512_A: 530 > 512 → yes  
margin_512_A = 512 - 530 = -18  
past_4096_A: 530 > 4096 → no  
margin_4096_A = 4096 - 530 = 3566  
occupied_A_with_B = 481 + 8 + 49 = 538  
margin_512_with_B_A = 512 - 538 = -26  
extra_writes_per_rollback_A = 0 (by F11)  
extra_reads_per_mount_A = 0 (by F12)  
extra_disk_bytes_A = 49 * 2 * 2 = 196  

width_A = 1 + 12 * 3 = 37  
occupied_A = 481 + 37 = 518  
past_512_A: 518 > 512 → yes  
margin_512_A = 512 - 518 = -6  
past_4096_A: 518 > 4096 → no  
margin_4096_A = 4096 - 518 = 3578  
occupied_A_with_B = 481 + 8 + 37 = 526  
margin_512_with_B_A = 512 - 526 = -14  
extra_writes_per_rollback_A = 0 (by F11)  
extra_reads_per_mount_A = 0 (by F12)  
extra_disk_bytes_A = 37 * 2 * 2 = 148  

width_A = 1 + 12 * 4 = 49  
occupied_A = 481 + 49 = 530  
past_512_A: 530 > 512 → yes  
margin_512_A = 512 - 530 = -18  
past_4096_A: 530 > 4096 → no  
margin_4096_A = 4096 - 530 = 3566  
occupied_A_with_B = 481 + 8 + 49 = 538  
margin_512_with_B_A = 512 - 538 = -26  
extra_writes_per_rollback_A = 0 (by F11)  
extra_reads_per_mount_A = 0 (by F12)  
extra_disk_bytes_A = 49 * 2 * 2 = 196  

If the maximum number of concurrently-alive rollback witnesses (F5) was higher than 4, e.g., 5, the numbers for rows with n=4 would change.

2. Candidate B  
width_B = 40 + (1 + 16 * 4) = 40 + 65 = 105  
past_512_B: 105 > 512 → no  
margin_512_B = 512 - 105 = 407  
past_4096_B: 105 > 4096 → no  
margin_4096_B = 4096 - 105 = 3991  
margin_512_with_B_B: not applicable  
extra_writes_per_rollback_B = 2 * 2 = 4  
extra_reads_per_mount_B = 2 * 2 = 4  
extra_disk_bytes_B at PB=512: ceil(105/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_B at PB=4096: ceil(105/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

width_B = 40 + (1 + 16 * 3) = 40 + 49 = 89  
past_512_B: 89 > 512 → no  
margin_512_B = 512 - 89 = 423  
past_4096_B: 89 > 4096 → no  
margin_4096_B = 4096 - 89 = 4007  
margin_512_with_B_B: not applicable  
extra_writes_per_rollback_B = 2 * 2 = 4  
extra_reads_per_mount_B = 2 * 2 = 4  
extra_disk_bytes_B at PB=512: ceil(89/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_B at PB=4096: ceil(89/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

width_B = 40 + (1 + 12 * 3) = 40 + 37 = 77  
past_512_B: 77 > 512 → no  
margin_512_B = 512 - 77 = 435  
past_4096_B: 77 > 4096 → no  
margin_4096_B = 4096 - 77 = 4019  
margin_512_with_B_B: not applicable  
extra_writes_per_rollback_B = 2 * 2 = 4  
extra_reads_per_mount_B = 2 * 2 = 4  
extra_disk_bytes_B at PB=512: ceil(77/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_B at PB=4096: ceil(77/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

width_B = 40 + (1 + 12 * 4) = 40 + 49 = 89  
past_512_B: 89 > 512 → no  
margin_512_B = 512 - 89 = 423  
past_4096_B: 89 > 4096 → no  
margin_4096_B = 4096 - 89 = 4007  
margin_512_with_B_B: not applicable  
extra_writes_per_rollback_B = 2 * 2 = 4  
extra_reads_per_mount_B = 2 * 2 = 4  
extra_disk_bytes_B at PB=512: ceil(89/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_B at PB=4096: ceil(89/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

If the self-certifying overhead (F9) was different (e.g., 32 bytes instead of 40), the width_B and extra_disk_bytes_B would change.

3. Candidate C  
entries_per_piece(w=16) = floor(471 / 16) = 29  
pieces_needed(w=16, n=4) = ceil(4 / 29) = 1  
width_C = 41 * 1 + 16 * 4 = 41 + 64 = 105  
margin_512_C = 512 - 105 = 407  
past_4096_C: 105 > 4096 → no  
margin_4096_C = 4096 - 105 = 3991  
margin_512_with_B_C: not applicable  
extra_writes_per_rollback_C = 1 * 2 * 2 = 4  
extra_reads_per_mount_C = 1 * 2 * 2 = 4  
extra_disk_bytes_C at PB=512: ceil(105/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_C at PB=4096: ceil(105/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

entries_per_piece(w=16) = floor(471 / 16) = 29  
pieces_needed(w=16, n=3) = ceil(3 / 29) = 1  
width_C = 41 * 1 + 16 * 3 = 41 + 48 = 89  
margin_512_C = 512 - 89 = 423  
past_4096_C: 89 > 4096 → no  
margin_4096_C = 4096 - 89 = 4007  
margin_512_with_B_C: not applicable  
extra_writes_per_rollback_C = 1 * 2 * 2 = 4  
extra_reads_per_mount_C = 1 * 2 * 2 = 4  
extra_disk_bytes_C at PB=512: ceil(89/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_C at PB=4096: ceil(89/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

entries_per_piece(w=12) = floor(471 / 12) = 39  
pieces_needed(w=12, n=3) = ceil(3 / 39) = 1  
width_C = 41 * 1 + 12 * 3 = 41 + 36 = 77  
margin_512_C = 512 - 77 = 435  
past_4096_C: 77 > 4096 → no  
margin_4096_C = 4096 - 77 = 4019  
margin_512_with_B_C: not applicable  
extra_writes_per_rollback_C = 1 * 2 * 2 = 4  
extra_reads_per_mount_C = 1 * 2 * 2 = 4  
extra_disk_bytes_C at PB=512: ceil(77/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_C at PB=4096: ceil(77/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

entries_per_piece(w=12) = floor(471 / 12) = 39  
pieces_needed(w=12, n=4) = ceil(4 / 39) = 1  
width_C = 41 * 1 + 12 * 4 = 41 + 48 = 89  
margin_512_C = 512 - 89 = 423  
past_4096_C: 89 > 4096 → no  
margin_4096_C = 4096 - 89 = 4007  
margin_512_with_B_C: not applicable  
extra_writes_per_rollback_C = 1 * 2 * 2 = 4  
extra_reads_per_mount_C = 1 * 2 * 2 = 4  
extra_disk_bytes_C at PB=512: ceil(89/512) * 512 * 2 * 2 = 1 * 512 * 4 = 2048  
extra_disk_bytes_C at PB=4096: ceil(89/4096) * 4096 * 2 * 2 = 1 * 4096 * 4 = 16384  

If the self-certifying overhead (F9) was different (e.g., 24 bytes instead of 40), the entries_per_piece and width_C would change.
