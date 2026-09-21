1. Arm STOP Stage T1  
Caller receive: the original device error is handed back to the caller unchanged. This would be refuted by: the caller receiving success despite the unit write failure.  
Recovery: definitely not taking effect. This would be refuted by: recovery replaying the journal record when it was not written.  
Match: yes. This would be refuted by: the caller receiving success while recovery found the publish not taking effect.  
Fact: fact 13: After admission succeeds, the allocator's current state is cloned into a saved copy before any write for this publish is attempted. If the inner write step later returns any error, the allocator is unconditionally restored from that saved copy, regardless of which one of the on disk writing steps the error happened at. The on disk writing steps run, in order, inside one closure: writing the changed units, a barrier, writing the journal record, a second barrier, writing the root slot with force unit access, and rotating the system config slot. If any one of these steps returns an error, this attempt is recorded into a failed publish accounting bucket, and the original device error is handed back to the caller unchanged. The caller does not receive this attempt's resulting version object. This would be refuted by: the caller not receiving the original device error.  

Arm STOP Stage T2  
Caller receive: the publish call that failed reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk. This would be refuted by: the caller receiving success despite the publish not being fully written.  
Recovery: definitely not taking effect. This would be refuted by: recovery replaying the journal record when the root slot was not written.  
Match: yes. This would be refuted by: the caller receiving success while recovery found the publish not taking effect.  
Fact: fact 3: For the question of what applying one journal record does at the pointer layer, the choice taken between two options is that one publish is applied as a whole, or not applied at all; there is no partial application of a single publish. This would be refuted by: a partial application of a publish being observed.  

Arm STOP Stage T3  
Caller receive: the publish call that failed reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk. This would be refuted by: the caller receiving success despite the system config slot failure.  
Recovery: takes effect. This would be refuted by: recovery not replaying the journal record when the root slot was written.  
Match: no. This would be refuted by: the caller receiving error while recovery found the publish not taking effect.  
Fact: no fact requires this match. This would be refuted by: a fact stating that the caller's receive must match recovery's findings.  

Arm STOP Stage T4  
Caller receive: taking the new instance number rolls back on its own write errors; this is today's running code, per fact 12. This would be refuted by: the instance number not rolling back on write errors.  
Recovery: definitely not taking effect. This would be refuted by: recovery finding the new instance number applied despite the write error.  
Match: yes. This would be refuted by: the caller receiving error while recovery found the publish taking effect.  
Fact: fact 12: rollback here only covers write errors from the number taking writes themselves, it does not cover a publish failure that happens after the number has already been taken. This would be refuted by: rollback covering publish failures after the number was taken.  

Arm TABLE Stage T1  
Caller receive: the original device error is handed back to the caller unchanged. This would be refuted by: the caller receiving success despite the unit write failure.  
Recovery: definitely not taking effect. This would be refuted by: recovery replaying the journal record when it was not written.  
Match: yes. This would be refuted by: the caller receiving success while recovery found the publish not taking effect.  
Fact: fact 13: After admission succeeds, the allocator's current state is cloned into a saved copy before any write for this publish is attempted. If the inner write step later returns any error, the allocator is unconditionally restored from that saved copy, regardless of which one of the on disk writing steps the error happened at. The on disk writing steps run, in order, inside one closure: writing the changed units, a barrier, writing the journal record, a second barrier, writing the root slot with force unit access, and rotating the system config slot. If any one of these steps returns an error, this attempt is recorded into a failed publish accounting bucket, and the original device error is handed back to the caller unchanged. The caller does not receive this attempt's resulting version object. This would be refuted by: the caller not receiving the original device error.  

Arm TABLE Stage T2  
Caller receive: the original device error is handed back to the caller unchanged. This would be refuted by: the caller receiving success despite the journal record being partially written.  
Recovery: definitely not taking effect. This would be refuted by: recovery replaying the journal record when the root slot was not written.  
Match: yes. This would be refuted by: the caller receiving success while recovery found the publish not taking effect.  
Fact: fact 13: After admission succeeds, the allocator's current state is cloned into a saved copy before any write for this publish is attempted. If the inner write step later returns any error, the allocator is unconditionally restored from that saved copy, regardless of which one of the on disk writing steps the error happened at. The on disk writing steps run, in order, inside one closure: writing the changed units, a barrier, writing the journal record, a second barrier, writing the root slot with force unit access, and rotating the system config slot. If any one of these steps returns an error, this attempt is recorded into a failed publish accounting bucket, and the original device error is handed back to the caller unchanged. The caller does not receive this attempt's resulting version object. This would be refuted by: the caller not receiving the original device error.  

Arm TABLE Stage T3  
Caller receive: the original device error is handed back to the caller unchanged. This would be refuted by: the caller receiving success despite the system config slot failure.  
Recovery: takes effect. This would be refuted by: recovery not replaying the journal record when the root slot was written.  
Match: no. This would be refuted by: the caller receiving error while recovery found the publish taking effect.  
Fact: no fact requires this match. This would be refuted by: a fact stating that the caller's receive must match recovery's findings.  

Arm TABLE Stage T4  
Caller receive: taking the new instance number rolls back on its own write errors; this is today's running code, per fact 12. This would be refuted by: the instance number not rolling back on write errors.  
Recovery: definitely not taking effect. This would be refuted by: recovery finding the new instance number applied despite the write error.  
Match: yes. This would be refuted by: the caller receiving error while recovery found the publish taking effect.  
Fact: fact 12: rollback here only covers write errors from the number taking writes themselves, it does not cover a publish failure that happens after the number has already been taken. This would be refuted by: rollback covering publish failures after the number was taken.
