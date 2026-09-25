1. width_A = 1 + 16 * 4 = 65  
   occupied_A = 481 + 65 = 546  
   past_512_A: yes (546 > 512)  
   margin_512_A = 512 - 546 = -34  
   past_4096_A: no (546 <= 4096)  
   margin_4096_A = 4096 - 546 = 3550  
   occupied_A_with_B = 481 + 8 + 65 = 554  
   margin_512_with_B_A = 512 - 554 = -42  
   extra_writes_per_rollback_A = 0 (by F11)  
   extra_reads_per_mount_A = 0 (by F12)  
   extra_disk_bytes_A = 65 * 2 * 2 = 260  

   width_A = 1 + 16 * 3 = 49  
   occupied_A = 481 + 49 = 530  
   past_512_A: yes (530 > 512)  
   margin_512_A = 512 - 530 = -18  
   past_4096_A: no (530 <= 4096)  
   margin_4096_A = 4096 - 530 = 3566  
   occupied_A_with_B = 481 + 8 + 49 = 538  
   margin_512_with_B_A = 512 - 538 = -26  
   extra_writes_per_rollback_A = 0 (by F11)  
   extra_reads_per_mount_A = 0 (by F12)  
   extra_disk_bytes_A = 49 * 2 * 2 = 196  

   width_A = 1 + 12 * 3 = 37  
   occupied_A = 481 + 37 = 518  
   past_512_A: yes (518 > 512)  
   margin_512_A = 512 - 518 = -6  
   past_4096_A: no (518 <= 4096)  
   margin_4096_A = 4096 - 518 = 3578  
   occupied_A_with_B = 481 + 8 + 37 = 526  
   margin_512_with_B_A = 512 - 526 = -14  
   extra_writes_per_rollback_A = 0 (by F11)  
   extra_reads_per_mount_A = 0 (by F12)  
   extra_disk_bytes_A = 37 * 2 * 2 = 148  

   width_A = 1 + 12 * 4 = 49  
   occupied_A = 481 + 49 = 530  
   past_512_A: yes (530 > 512)  
   margin_512_A = 512 - 530 = -18  
   past_4096_A: no (530 <= 4096)  
   margin_4096_A = 4096 - 530 = 3566  
   occupied_A_with_B = 481 + 8 + 49 = 538  
   margin_512_with_B_A = 512 - 538 = -26  
   extra_writes_per_rollback_A = 0 (by F11)  
   extra_reads_per_mount_A = 0 (by F12)  
   extra_disk_bytes_A = 49 * 2 * 2 = 196  
   A different system-configuration field table size (F1), system-configuration slot size (F2), witness entry width (F3), checkpoint-txg field width (F6), device count (F10), or maximum number of concurrently-alive rollbacks (F5) would overturn these numbers.

2. width_B = 40 + (1 + 16 * 4) = 105  
   past_512_B: no (105 <= 512)  
   margin_512_B = 512 - 105 = 407  
   past_4096_B: no (105 <= 4096)  
   margin_4096_B = 4096 - 105 = 3991  
   margin_512_with_B_B: not applicable  
   extra_writes_per_rollback_B = 2 * 2 = 4  
   extra_reads_per_mount_B = 2 * 2 = 4  
   extra_disk_bytes_B at PB=512: ceil(105 / 512) * 512 * 2 * 2 = 2048  
   extra_disk_bytes_B at PB=4096: ceil(105 / 4096) * 4096 * 2 * 2 = 16384  

   width_B = 40 + (1 + 16 * 3) = 89  
   past_512_B: no (89 <= 512)  
   margin_512_B = 512 - 89 = 423  
   past_4096_B: no (89 <= 4096)  
   margin_4096_B = 4096 - 89 = 4007  
   margin_512_with_B_B: not applicable  
   extra_writes_per_rollback_B = 4  
   extra_reads_per_mount_B = 4  
   extra_disk_bytes_B at PB=512: ceil(89 / 512) * 512 * 4 = 2048  
   extra_disk_bytes_B at PB=4096: ceil(89 / 4096) * 4096 * 4 = 16384  

   width_B = 40 + (1 + 12 * 3) = 77  
   past_512_B: no (77 <= 512)  
   margin_512_B = 512 - 77 = 435  
   past_4096_B: no (77 <= 4096)  
   margin_4096_B = 4096 - 77 = 4019  
   margin_512_with_B_B: not applicable  
   extra_writes_per_rollback_B = 4  
   extra_reads_per_mount_B = 4  
   extra_disk_bytes_B at PB=512: ceil(77 / 512) * 512 * 4 = 2048  
   extra_disk_bytes_B at PB=4096: ceil(77 / 4096) * 4096 * 4 = 16384  

   width_B = 40 + (1 + 12 * 4) = 89  
   past_512_B: no (89 <= 512)  
   margin_512_B = 512 - 89 = 423  
   past_4096_B: no (89 <= 4096)  
   margin_4096_B = 4096 - 89 = 4007  
   margin_512_with_B_B: not applicable  
   extra_writes_per_rollback_B = 4  
   extra_reads_per_mount_B = 4  
   extra_disk_bytes_B at PB=512: ceil(89 / 512) * 512 * 4 = 2048  
   extra_disk_bytes_B at PB=4096: ceil(89 / 4096) * 4096 * 4 = 16384  
   A different generation number or checksum width (F9), witness entry width (F3), maximum number of concurrently-alive rollbacks (F5), device count (F10), or physical block size (F7) would overturn these numbers.

3. entries_per_piece(w=16) = floor(471 / 16) = 29  
   pieces_needed = ceil(4 / 29) = 1  
   width_C = 41 * 1 + 16 * 4 = 105  
   margin_512_C = 512 - (41 + 16 * 4) = 407  
   past_4096_C: no (105 <= 4096)  
   margin_4096_C = 4096 - 105 = 3991  
   margin_512_with_B_C: not applicable  
   extra_writes_per_rollback_C = 1 * 2 * 2 = 4  
   extra_reads_per_mount_C = 4  
   extra_disk_bytes_C at PB=512: ceil(105 / 512) * 512 * 2 * 2 = 2048  
   extra_disk_bytes_C at PB=4096: ceil(105 / 4096) * 4096 * 2 * 2 = 16384  

   entries_per_piece(w=16) = floor(471 / 16) = 29  
   pieces_needed = ceil(3 / 29) = 1  
   width_C = 41 * 1 + 16 * 3 = 89  
   margin_512_C = 512 - (41 + 16 * 3) = 423  
   past_4096_C: no (89 <= 4096)  
   margin_4096_C = 4096 - 89 = 4007  
   margin_512_with_B_C: not applicable  
   extra_writes_per_rollback_C = 4  
   extra_reads_per_mount_C = 4  
   extra_disk_bytes_C at PB=512: ceil(89 / 512) * 512 * 4 = 2048  
   extra_disk_bytes_C at PB=4096: ceil(89 / 4096) * 4096 * 4 = 16384  

   entries_per_piece(w=12) = floor(471 / 12) = 39  
   pieces_needed = ceil(3 / 39) = 1  
   width_C = 41 * 1 + 12 * 3 = 77  
   margin_512_C = 512 - (41 + 12 * 3) = 435  
   past_4096_C: no (77 <= 4096)  
   margin_4096_C = 4096 - 77 = 4019  
   margin_512_with_B_C: not applicable  
   extra_writes_per_rollback_C = 4  
   extra_reads_per_mount_C = 4  
   extra_disk_bytes_C at PB=512: ceil(77 / 512) * 512 * 4 = 2048  
   extra_disk_bytes_C at PB=4096: ceil(77 / 4096) * 4096 * 4 = 16384  

   entries_per_piece(w=12) = floor(471 / 12) = 39  
   pieces_needed = ceil(4 / 39) = 1  
   width_C = 41 * 1 + 12 * 4 = 89  
   margin_512_C = 512 - (41 + 12 * 4) = 423  
   past_4096_C: no (89 <= 4096)  
   margin_4096_C = 4096 - 89 = 4007  
   margin_512_with_B_C: not applicable  
   extra_writes_per_rollback_C = 4  
   extra_reads_per_mount_C = 4  
   extra_disk_bytes_C at PB=512: ceil(89 / 512) * 512 * 4 = 2048  
   extra_disk_bytes_C at PB=4096: ceil(89 / 4096) * 4096 * 4 = 16384  
   A different physical block size (F7), generation/checksum width (F9), witness entry width (F3), maximum number of concurrently-alive rollbacks (F5), or device count (F10) would overturn these numbers.
