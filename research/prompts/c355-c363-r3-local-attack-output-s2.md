D5ROW.ONECOPY no the row is already marked as not carrying device dimension with pool-wide commitment which correctly indicates it is a single scalar for the entire pool regardless of whether the value represents one copy or sum of copies the description does not need to change because it does not specify the convention only the dimensionality this would be refuted by the row description explicitly stating one-copy or sum-of-copies in its current text

D5ROW.SUMCOPIES no the row is already marked as not carrying device dimension with pool-wide commitment which correctly indicates it is a single scalar for the entire pool regardless of whether the value represents one copy or sum of copies the description does not need to change because it does not specify the convention only the dimensionality this would be refuted by the row description explicitly stating one-copy or sum-of-copies in its current text

D5ROW.BREAKS neither change causes any other sentence to become false because no changes are required for this row so there is no change to cause breakage this would be refuted by a change to D5ROW being necessary for either convention which is not the case

GATE51.ONECOPY no the gate only checks that every formula term has a matching entry in the cross-reference table and does not validate the convention of the values the gate rules do not depend on whether the statistic is one copy or sum of copies this would be refuted by the gate script requiring validation of the convention of the terms which it does not

GATE51.SUMCOPIES no the gate only checks that every formula term has a matching entry in the cross-reference table and does not validate the convention of the values the gate rules do not depend on whether the statistic is one copy or sum of copies this would be refuted by the gate script requiring validation of the convention of the terms which it does not

GATE51.BREAKS neither change causes any other sentence to become false because no changes are required for this row so there is no change to cause breakage this would be refuted by a change to GATE51 being necessary for either convention which is not the case

CRATES.ONECOPY yes the code must be adjusted to compute pending-delete occupancy and committed reservation as logical size counted once per object instead of sum of physical replicas because currently the code only initializes these statistics to zero and does not compute them correctly for either convention this would be refuted by the code already correctly computing these values according to the one-copy convention which it does not

CRATES.SUMCOPIES yes the code must be adjusted to compute pending-delete occupancy and committed reservation as the sum of physical bytes across all replicas because currently the code only initializes these statistics to zero and does not compute them correctly for either convention this would be refuted by the code already correctly computing these values according to the sum-of-copies convention which it does not

CRATES.BREAKS changing to one-copy convention would cause the admission cross-check described in FACT 5 to fail because the values would be incorrect for the admission formula's expected convention making the cross-check's described behavior false this would be refuted by the admission cross-check still functioning correctly when using one-copy convention which it would not if the formula expects sum-of-copies

INVARIANT.ONECOPY no the invariant I-3.4 describes deducting pending-delete occupancy but does not specify the convention and is not implemented so no change is needed to the invariant text this would be refuted by the invariant definition explicitly requiring a specific convention which it does not

INVARIANT.SUMCOPIES no the invariant I-3.4 describes deducting pending-delete occupancy but does not specify the convention and is not implemented so no change is needed to the invariant text this would be refuted by the invariant definition explicitly requiring a specific convention which it does not

INVARIANT.BREAKS neither change causes any other sentence to become false because no changes are required for this row so there is no change to cause breakage this would be refuted by a change to INVARIANT being necessary for either convention which is not the case

SUMMARY.ONECOPY CRATES requires a change D5ROW GATE51 and INVARIANT require no change this would be refuted by any of D5ROW GATE51 or INVARIANT requiring a change for one-copy convention which they do not

SUMMARY.SUMCOPIES CRATES requires a change D5ROW GATE51 and INVARIANT require no change this would be refuted by any of D5ROW GATE51 or INVARIANT requiring a change for sum-of-copies convention which they do not
