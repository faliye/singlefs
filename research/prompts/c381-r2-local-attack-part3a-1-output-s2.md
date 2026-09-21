1.  
Arm STOP, P1:  
outside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', but Arm STOP does not perform any probe write as per fact 13.  
T1  
Justification: P1 is a unit write failure occurring before any journal record is written, matching T1's definition of unit write partway done with no journal record sent.  
At stage T1: the allocator rolls back to its pre attempt state, the write path may keep publishing as before, and, per fact 13, the call whose own write failed at this stage receives the original device error.  
not applicable  
Justification: Arm STOP's handling does not involve the probe write discriminant specified in fact 4, making fact 4's core rule inapplicable to this scenario.  

Arm STOP, P2:  
outside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', but Arm STOP does not perform any probe write as per fact 13.  
T1  
Justification: P2 is a first barrier failure after unit writes but before journal record write, matching T1's definition of no journal record sent.  
At stage T1: the allocator rolls back to its pre attempt state, the write path may keep publishing as before, and, per fact 13, the call whose own write failed at this stage receives the original device error.  
not applicable  
Justification: Arm STOP's handling does not involve the probe write discriminant specified in fact 4, making fact 4's core rule inapplicable to this scenario.  

Arm STOP, P3:  
outside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', but Arm STOP does not perform any probe write as per fact 13.  
T2  
Justification: P3 is a journal record write failure after unit writes and first barrier but before root slot FUA, matching T2's definition of journal record already sent but root slot FUA not succeeded.  
At stage T2: the publish call that failed reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk.  
not applicable  
Justification: Arm STOP's handling does not involve the probe write discriminant specified in fact 4, making fact 4's core rule inapplicable to this scenario.  

Arm STOP, P4:  
outside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', but Arm STOP does not perform any probe write as per fact 13.  
T2  
Justification: P4 is a second barrier failure after journal record write but before root slot FUA, matching T2's definition of journal record already sent but root slot FUA not succeeded.  
At stage T2: the publish call that failed reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk.  
not applicable  
Justification: Arm STOP's handling does not involve the probe write discriminant specified in fact 4, making fact 4's core rule inapplicable to this scenario.  

Arm TABLE, P1:  
inside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', which applies to all failures including P1.  
T1  
Justification: P1 is a unit write failure before any journal record is written, matching T1's definition of unit write partway done with no journal record sent.  
At stage T1: follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if that probe write succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
consistent  
Justification: Arm TABLE follows the probe write discriminant specified in fact 4 for this failure point.  

Arm TABLE, P2:  
inside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', which applies to all failures including P2.  
T1  
Justification: P2 is a first barrier failure after unit writes but before journal record write, matching T1's definition of no journal record sent.  
At stage T2: same handling as arm TABLE's stage T1.  
consistent  
Justification: Arm TABLE follows the probe write discriminant specified in fact 4 for this failure point.  

Arm TABLE, P3:  
inside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', which applies to all failures including P3.  
T2  
Justification: P3 is a journal record write failure after unit writes and first barrier but before root slot FUA, matching T2's definition of journal record already sent but root slot FUA not succeeded.  
At stage T2: same handling as arm TABLE's stage T1.  
consistent  
Justification: Arm TABLE follows the probe write discriminant specified in fact 4 for this failure point.  

Arm TABLE, P4:  
inside scope  
Justification: fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device', which applies to all failures including P4.  
T2  
Justification: P4 is a second barrier failure after journal record write but before root slot FUA, matching T2's definition of journal record already sent but root slot FUA not succeeded.  
At stage T2: same handling as arm TABLE's stage T1.  
consistent  
Justification: Arm TABLE follows the probe write discriminant specified in fact 4 for this failure point.  

2.  
no such case  
This would be refuted by: finding a row where Arm TABLE at one failure point is marked consistent and another row where Arm TABLE at a different failure point is marked conflict while using identical reasoning about fact 4's probe write discriminant.
