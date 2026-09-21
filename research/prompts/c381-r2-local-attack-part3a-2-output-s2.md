1
Arm STOP
P5
Inside scope
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."
Stage: T2
Justification: root slot force unit access failure occurs after journal record write and before system config slot rotation, matching stage T2's definition of journal record having been sent but root slot force unit access not yet succeeded
Action: "the publish call that failed reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk"
Consistency: conflict
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device" but arm STOP does not perform a probe write before handling the failure
P6
Inside scope
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."
Stage: T3
Justification: system config slot rotation failure occurs after root slot force unit access succeeded, matching stage T3's definition of root slot force unit access having succeeded and failure during system config slot rotation
Action: "same handling as arm STOP's stage T2"
Consistency: conflict
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device" but arm STOP does not perform a probe write before handling the failure
P7
Outside scope
Justification: fact 4's core rule describes handling for publish failures, but P7 occurs during mount initialization before any publish attempt, so the text does not decide this case as it is outside the scope of publish failures
Stage: T4
Justification: system config slot writes for number taking during mount, matching stage T4's definition of failure during system config slot writes used to take a new instance number before that mount's own row writing publish
Action: "taking the new instance number rolls back on its own write errors; this is today's running code, per fact 12"
Consistency: not applicable
Justification: fact 4's core rule does not address mount initialization failures, so the rule is not applicable to P7
P8
Inside scope
Justification: "the switch's own first step is itself a write that can fail: a switch must first take a new instance number, and taking that number requires writing to every system config copy that this mount exclusively opened successfully, all or nothing. Because of this, a fault of the kind where a write cannot go through lets a switch advance at most one further step: the probe write catches it first, and even if a switch happens to get past the probe write, the all or nothing number taking write immediately judges it a failure."
Stage: T4
Justification: number taking and row writing during switch involves system config slot writes for new instance number, matching stage T4's definition of failure during system config slot writes used to take a new instance number
Action: "taking the new instance number rolls back on its own write errors; this is today's running code, per fact 12"
Consistency: conflict
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device" but arm STOP does not perform a probe write before handling the failure
Arm TABLE
P5
Inside scope
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."
Stage: T2
Justification: root slot force unit access failure occurs after journal record write and before system config slot rotation, matching stage T2's definition of journal record having been sent but root slot force unit access not yet succeeded
Action: "follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only"
Consistency: consistent
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device" and arm TABLE follows this by performing the probe write
P6
Inside scope
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."
Stage: T3
Justification: system config slot rotation failure occurs after root slot force unit access succeeded, matching stage T3's definition of root slot force unit access having succeeded and failure during system config slot rotation
Action: "same handling as arm TABLE's stage T1"
Consistency: consistent
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device" and arm TABLE follows this by performing the probe write
P7
Outside scope
Justification: fact 4's core rule describes handling for publish failures, but P7 occurs during mount initialization before any publish attempt, so the text does not decide this case as it is outside the scope of publish failures
Stage: T4
Justification: system config slot writes for number taking during mount, matching stage T4's definition of failure during system config slot writes used to take a new instance number before that mount's own row writing publish
Action: "same handling as arm STOP's stage T4"
Consistency: not applicable
Justification: fact 4's core rule does not address mount initialization failures, so the rule is not applicable to P7
P8
Inside scope
Justification: "the switch's own first step is itself a write that can fail: a switch must first take a new instance number, and taking that number requires writing to every system config copy that this mount exclusively opened successfully, all or nothing. Because of this, a fault of the kind where a write cannot go through lets a switch advance at most one further step: the probe write catches it first, and even if a switch happens to get past the probe write, the all or nothing number taking write immediately judges it a failure."
Stage: T4
Justification: number taking and row writing during switch involves system config slot writes for new instance number, matching stage T4's definition of failure during system config slot writes used to take a new instance number
Action: "same handling as arm STOP's stage T4"
Consistency: conflict
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device" but arm TABLE's T4 action does not perform a probe write
This would be refuted by: evidence that fact 4's core rule does not apply to root slot write failures during publish, but fact 1 explicitly states root slot is part of the publish sequence

2
Yes, there is such a case. The two rows are Arm TABLE at failure point P5 (marked consistent) and Arm TABLE at failure point P8 (marked conflict). For P5 (stage T2), arm TABLE follows fact 4's probe write requirement, but for P8 (stage T4), arm TABLE's action does not perform a probe write despite fact 4's rule applying to both failure points.
This would be refuted by: if arm TABLE's stage T4 action was defined to perform a probe write as per fact 4, eliminating the inconsistency.
