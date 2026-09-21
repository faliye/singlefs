1. Table 3:

Arm STOP, Stage T1:  
Caller receives: the original device error is handed back to the caller unchanged (fact 13).  
Recovery: definitely not taking effect because journal record was not written.  
Match: yes because caller error matches disk state of no effect.  
Fact: fact 3: one publish is applied as a whole, or not applied at all; there is no partial application of a single publish.  
This would be refuted by: the caller receiving a success while the journal record was not written.  

Arm STOP, Stage T2:  
Caller receives: the original device error is handed back to the caller unchanged (fact 13).  
Recovery: definitely not taking effect because root slot was not persisted, so journal record not replayed.  
Match: yes because caller error matches disk state of no effect.  
Fact: fact 1: the fsync call that triggers this publish does not return until the root slot write is persisted.  
This would be refuted by: the fsync call returning successfully while the root slot write was not persisted.  

Arm STOP, Stage T3:  
Caller receives: the original device error is handed back to the caller unchanged (fact 13).  
Recovery: definitely taking effect because root slot was persisted and journal record was written.  
Match: no because caller received error but disk state shows effect.  
Fact: fact 1: the fsync call that triggers this publish does not return until the root slot write is persisted.  
This would be refuted by: the fsync call not returning until the system config slot write is persisted.  

Arm STOP, Stage T4:  
Caller receives: taking the new instance number rolls back on its own write errors (fact 12).  
Recovery: definitely not taking effect because mount initialization failed before publish.  
Match: yes because caller error matches disk state of no effect.  
Fact: fact 12: rollback here only covers write errors from the number taking writes themselves, it does not cover a publish failure that happens after the number has already been taken.  
This would be refuted by: a number taking write error not triggering rollback.  

Arm TABLE, Stage T1:  
Caller receives: if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
Recovery: definitely not taking effect because journal record was not written.  
Match: depends on whether the probe write succeeds.  
Fact: none.  
This would be refuted by: the probe write result not affecting the caller's received response.  

Arm TABLE, Stage T2:  
Caller receives: if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
Recovery: definitely not taking effect because root slot was not persisted.  
Match: depends on whether the probe write succeeds.  
Fact: none.  
This would be refuted by: the probe write result not affecting the caller's received response.  

Arm TABLE, Stage T3:  
Caller receives: if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only.  
Recovery: definitely taking effect because root slot was persisted and journal record was written.  
Match: depends on whether the probe write succeeds.  
Fact: none.  
This would be refuted by: the probe write result not affecting the caller's received response.  

Arm TABLE, Stage T4:  
Caller receives: taking the new instance number rolls back on its own write errors (fact 12).  
Recovery: definitely not taking effect because mount initialization failed before publish.  
Match: yes because caller error matches disk state of no effect.  
Fact: fact 12: rollback here only covers write errors from the number taking writes themselves, it does not cover a publish failure that happens after the number has already been taken.  
This would be refuted by: a number taking write error not triggering rollback.
