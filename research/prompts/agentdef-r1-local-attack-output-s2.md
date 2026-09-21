R1a. FOUND; LABEL_TXN followed by VAL_TXN_NO, LPAREN, DATE sequence present
R1b. FOUND; LABEL_FMT followed by VAL_FMT_WILL, LPAREN, DATE sequence present
R1c. GREEN; both TXN and FMT found so neither list is non-empty
R1d. NO; separate labels and brackets with no shared elements; if they shared a bracket or used wrong values it would change to YES
R1e. NO; neither Doc-15 default rule applies since FMT value is explicitly WILL and TXN has no defaults
R1f. NONE-NEEDED; if any blurring or default disagreement occurred it would overturn NONE-NEEDED
R2a. FOUND; LABEL_TXN followed by VAL_TXN_NO, LPAREN, DATE sequence present
R2b. NOT-FOUND; LABEL_FMT never occurs in item-block
R2c. RED-missing-FMT; FMT not found so second list non-empty while TXN found
R2d. NO; FMT entirely absent with no blurring of requirements; if FMT label was present but misused it would change to YES
R2e. YES; Doc-15 unwritten-defaults-to-WILL rule applies since no FMT judgment written, but checker flags missing-FMT
R2f. Check that absence of LABEL_FMT is acceptable per Doc-15 default; if LABEL_FMT were required to be present it would overturn NONE-NEEDED
R3a. NOT-FOUND; VAL_TXN_NO followed by SPACE then LABEL_FMT breaks TXN sequence (must have only STARs between value and LPAREN)
R3b. FOUND; LABEL_FMT followed by VAL_FMT_WILL, LPAREN, DATE sequence present
R3c. RED-missing-TXN; TXN not found so first list non-empty while FMT found
R3d. YES; shared single LPAREN between TXN and FMT judgments; if each had separate LPAREN it would change to NO
R3e. NO; TXN has no defaults and FMT is explicitly present so no Doc-15 default conflict
R3f. Check that each judgment's value is immediately followed by LPAREN without shared brackets; if brackets were separate it would overturn NEED
R4a. NOT-FOUND; VAL_FMT_WILL is not a valid TXN value (must be VAL_TXN_YES/NO/MOOT)
R4b. NOT-FOUND; LABEL_FMT never occurs in item-block
R4c. RED-missing-both; both TXN and FMT not found so both lists non-empty
R4d. YES; VAL_FMT_WILL used under LABEL_TXN (wrong vocabulary); if correct TXN value was used it would change to NO
R4e. YES; Doc-15 unwritten-defaults-to-WILL rule applies for missing FMT, but checker flags missing-FMT
R4f. Check that values under LABEL_TXN are exclusively VAL_TXN_YES/NO/MOOT; if wrong values were allowed it would overturn NEED
R5a. FOUND; LABEL_TXN followed by VAL_TXN_NO, LPAREN, DATE sequence present
R5b. FOUND; LABEL_FMT followed by VAL_FMT_UNCLEAR, LPAREN, DATE sequence present
R5c. GREEN; both TXN and FMT found so neither list is non-empty
R5d. NO; separate labels and correct values with no blurring; if they shared brackets or used wrong values it would change to YES
R5e. NO; Doc-15 UNCLEAR-folds-into-WILL applies but checker sees FMT present so no conflict
R5f. NONE-NEEDED; if any blurring or default disagreement occurred it would overturn NONE-NEEDED
R6a. FOUND; LABEL_TXN followed by VAL_TXN_NO, LPAREN, DATE sequence present
R6b. FOUND; LABEL_FMT followed by VAL_FMT_WONT, LPAREN, DATE sequence present
R6c. GREEN; both TXN and FMT found so neither list is non-empty
R6d. NO; separate labels with proper brackets and values; if they shared brackets it would change to YES
R6e. NO; VAL_FMT_WONT explicitly written so Doc-15 treats as WONT with no default conflict
R6f. NONE-NEEDED; if any blurring or default disagreement occurred it would overturn NONE-NEEDED
R7a. FOUND; LABEL_TXN sequence present despite FMT appearing first
R7b. FOUND; LABEL_FMT sequence present despite appearing first
R7c. GREEN; both TXN and FMT found so neither list is non-empty
R7d. NO; order of judgments doesn't affect finding per RULE THREE; if labels were swapped within sequence it would change to YES
R7e. NO; both judgments present with no default conflicts
R7f. NONE-NEEDED; if any blurring or default disagreement occurred it would overturn NONE-NEEDED
R8a. FOUND; example sentence contains LABEL_TXN followed by VAL_TXN_NO, LPAREN, DATE sequence
R8b. FOUND; LABEL_FMT followed by VAL_FMT_WONT, LPAREN, DATE sequence present
R8c. GREEN; both TXN and FMT found so neither list is non-empty
R8d. YES; TXN sequence is part of illustrative example not real judgment; if example was clearly marked as such it would change to NO
R8e. NO; FMT explicitly written as VAL_FMT_WONT so Doc-15 treats as WONT with no default conflict
R8f. Check if TXN sequence is inside a quoted example or marked as illustrative; if example was unmarked it would overturn NONE-NEEDED
Q1. R3 shows shared bracket between judgments; R4 shows swapping vocabulary under wrong label; R8 shows illustrative example as stand-in answer; if R3 had separate brackets, R4 used correct TXN value, or R8 example was clearly marked it would overturn the list
Q2. R2: check absence of LABEL_FMT is acceptable per Doc-15 default; R4: check values under LABEL_TXN are exclusively VAL_TXN_YES/NO/MOOT; R8: check if TXN sequence is inside a quoted example or marked as illustrative; if all rows had correct judgments it would overturn this list
Q3. No; checker's GREEN verdict requires FMT judgment present, so Doc-15 default (which applies only when no judgment) never applies; if a row had missing FMT but checker said GREEN it would overturn this answer
