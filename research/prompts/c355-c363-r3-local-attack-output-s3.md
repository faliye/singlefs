D5ROW.ONECOPY no, because the row's current description only specifies the dimension (pool-wide) and does not depend on the convention of one-copy or sum-of-copies; the convention affects the value calculation but not the row's metadata. This would be refuted by: a change to the row's name or description that explicitly states the convention being used.

D5ROW.SUMCOPIES no, for the same reason as ONECOPY. This would be refuted by: a change to the row's name or description that explicitly states the convention being used.

D5ROW.BREAKS neither change causes any other sentence in FACT 1 through FACT 7 to become false, because no changes are required for either convention at this row. This would be refuted by: evidence that modifying the row's current description for one convention causes another fact to become false.

GATE51.ONECOPY no, because the gate check only verifies term existence and matching to the cross-reference table, not the convention of the statistic's value. This would be refuted by: the gate script failing due to a change in the convention affecting term matching.

GATE51.SUMCOPIES no, for the same reason as ONECOPY. This would be refuted by: the gate script failing due to a change in the convention affecting term matching.

GATE51.BREAKS neither change causes any other sentence in FACT 1 through FACT 7 to become false, because no changes are required for either convention at this row. This would be refuted by: evidence that modifying the gate's check for one convention causes another fact to become false.

CRATES.ONECOPY no, because the code currently sets both statistics to zero with no other code that computes or assigns values, so the convention does not affect existing code behavior. This would be refuted by: evidence of code that computes or assigns values to these statistics which would be incorrect under the one-copy convention.

CRATES.SUMCOPIES no, for the same reason as ONECOPY. This would be refuted by: evidence of code that computes or assigns values to these statistics which would be incorrect under the sum-of-copies convention.

CRATES.BREAKS neither change causes any other sentence in FACT 1 through FACT 7 to become false, because no changes are required for either convention at this row. This would be refuted by: evidence that modifying the code for one convention causes another fact to become false.

INVARIANT.ONECOPY no, because the invariant's text does not specify the convention and only refers to deducting the statistic, which is handled correctly based on the convention's value. This would be refuted by: evidence that the invariant's text explicitly requires a specific convention which would be violated.

INVARIANT.SUMCOPIES no, for the same reason as ONECOPY. This would be refuted by: evidence that the invariant's text explicitly requires a specific convention which would be violated.

INVARIANT.BREAKS neither change causes any other sentence in FACT 1 through FACT 7 to become false, because no changes are required for either convention at this row. This would be refuted by: evidence that modifying the invariant's text for one convention causes another fact to become false.

SUMMARY.ONECOPY all four rows (D5ROW GATE51 CRATES INVARIANT) require no change for the one-copy convention. This would be refuted by: evidence that one of the four rows requires a change for the one-copy convention.

SUMMARY.SUMCOPIES all four rows (D5ROW GATE51 CRATES INVARIANT) require no change for the sum-of-copies convention. This would be refuted by: evidence that one of the four rows requires a change for the sum-of-copies convention.
