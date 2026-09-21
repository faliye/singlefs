1.  
Arm STOP, P1  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T1  
Justification: P1 is a unit write failure occurring before journal record writing.  
Action: At stage T1: the allocator rolls back to its pre attempt state, the write path may keep publishing as before, and, per fact 13, the call whose own write failed at this stage receives the original device error.  
Consistency: conflict  
Justification: fact 4's core rule requires performing a probe write to classify the failure, but Arm STOP does not perform this step and instead directly rolls back the allocator.  
Arm STOP, P2  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T1  
Justification: P2 is a first barrier failure occurring after unit writes but before journal record writing.  
Action: At stage T1: the allocator rolls back to its pre attempt state, the write path may keep publishing as before, and, per fact 13, the call whose own write failed at this stage receives the original device error.  
Consistency: conflict  
Justification: fact 4's core rule requires performing a probe write to classify the failure, but Arm STOP does not perform this step and instead directly rolls back the allocator.  
Arm STOP, P3  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T1  
Justification: P3 is a journal record write failure occurring before the second barrier.  
Action: At stage T1: the allocator rolls back to its pre attempt state, the write path may keep publishing as before, and, per fact 13, the call whose own write failed at this stage receives the original device error.  
Consistency: conflict  
Justification: fact 4's core rule requires performing a probe write to classify the failure, but Arm STOP does not perform this step and instead directly rolls back the allocator.  
Arm STOP, P4  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T2  
Justification: P4 is a second barrier failure occurring after journal record writing but before root slot write.  
Action: At stage T2: the publish call that failed reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk.  
Consistency: conflict  
Justification: fact 4's core rule requires performing a probe write to classify the failure, but Arm STOP's action does not follow this procedure.  
Arm TABLE, P1  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T1  
Justification: P1 is a unit write failure occurring before journal record writing.  
Action: At stage T1: follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
Consistency: consistent  
Justification: Arm TABLE's action directly implements fact 4's core rule as described.  
Arm TABLE, P2  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T1  
Justification: P2 is a first barrier failure occurring after unit writes but before journal record writing.  
Action: At stage T1: follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
Consistency: consistent  
Justification: Arm TABLE's action directly implements fact 4's core rule as described.  
Arm TABLE, P3  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T1  
Justification: P3 is a journal record write failure occurring before the second barrier.  
Action: At stage T1: follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
Consistency: consistent  
Justification: Arm TABLE's action directly implements fact 4's core rule as described.  
Arm TABLE, P4  
Scope: inside scope  
Justification: The handling of a failure splits into two branches according to the nature of the failure.  
Stage: T2  
Justification: P4 is a second barrier failure occurring after journal record writing but before root slot write.  
Action: At stage T2: same handling as arm TABLE's stage T1.  
Consistency: consistent  
Justification: Arm TABLE's action for stage T2 follows the failure table as per its definition.  
This would be refuted by: any row in the table where the scope, stage, action, or consistency does not match the provided facts and definitions.  

2.  
no  
This would be refuted by: finding a row for Arm STOP where one failure point was marked consistent and another conflict, or for Arm TABLE where one was conflict and another consistent.
