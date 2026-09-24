Row 12 (R4, K3):
Clause A: ACTION -- resend the in-flight checkpoint; W is the maximum transaction number in the checkpoint being resent
Clause B: ACTION -- W is the maximum transaction number in the checkpoint being resent
Clause C: ACTION -- Rule (2) applies: write (k, k's last published txg, W) where k is the instance of the selected root
Clause D: ACTION -- the published predicate rule as stated in Clause D
Clause E: UNDEFINED -- Clause E describes rollback, not instance switch
Clause F: UNDEFINED -- Clause F explicitly states the three things are open questions with no governing clause

1. The clauses do not specify whether the in-flight checkpoint is resent when the root chosen by re-reading disk is newer than the root recorded in memory. This is directly stated in Clause F part (i) as an open question with no governing clause. Observation that would falsify this is if any clause in A-F describes the behavior for this specific case.

2. The clauses do not specify whether the instance-table row and W value are written against the disk-chosen root or memory root. This is explicitly listed in Clause F part (ii) as an open question with no governing clause. Observation that would falsify this is if any clause in A-F clarifies which root is used for writing rows and W.

3. The clauses do not specify how the root chosen by re-reading disk corresponds to R_old during rollback. This is explicitly listed in Clause F part (iii) as an open question with no governing clause. Observation that would falsify this is if any clause in A-F describes the relationship between the disk-chosen root and R_old in rollback.
