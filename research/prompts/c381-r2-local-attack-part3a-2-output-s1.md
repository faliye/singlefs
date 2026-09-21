1  
Arm STOP, P5:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T2  
Reasoning: the root slot force unit access write is the fifth step in fact 1's order, after the journal record write and second barrier, so it falls within stage T2 where the journal record has been sent but root slot has not succeeded.  
publish call reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk  
conflict  
Justification: "the response is an instance switch" if probe succeeds, "going read only" if fails, but the action does not include performing a probe write or acting on its result.  
This would be refuted by: observing that the STOP arm's action at P5 includes performing a probe write followed by instance switch or read only as per fact 4's core rule.  

Arm STOP, P6:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T3  
Reasoning: the system config slot rotation occurs after the root slot force unit access write succeeds, so failure during this step falls within stage T3.  
publish call reports an error to its caller, the write path becomes void, no further publish is allowed for the rest of this mount, the caller must remount, and recovery alone decides which version ends up being the one on disk  
conflict  
Justification: "the response is an instance switch" if probe succeeds, "going read only" if fails, but the action does not include performing a probe write or acting on its result.  
This would be refuted by: observing that the STOP arm's action at P6 includes performing a probe write followed by instance switch or read only as per fact 4's core rule.  

Arm STOP, P7:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T4  
Reasoning: the failure during number taking for an ordinary mount involves system config slot writes used to take a new instance number, which matches the T4 stage definition.  
taking the new instance number rolls back on its own write errors; this is today's running code, per fact 12  
conflict  
Justification: "the response is an instance switch" if probe succeeds, "going read only" if fails, but the action does not include performing a probe write or acting on its result.  
This would be refuted by: observing that the STOP arm's action at P7 includes performing a probe write followed by instance switch or read only as per fact 4's core rule.  

Arm STOP, P8:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T4  
Reasoning: the failure during the switch's own number taking and row writing involves system config slot writes used to take a new instance number, which matches the T4 stage definition.  
taking the new instance number rolls back on its own write errors; this is today's running code, per fact 12  
conflict  
Justification: "the response is an instance switch" if probe succeeds, "going read only" if fails, but the action does not include performing a probe write or acting on its result.  
This would be refuted by: observing that the STOP arm's action at P8 includes performing a probe write followed by instance switch or read only as per fact 4's core rule.  

Arm TABLE, P5:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T2  
Reasoning: the root slot force unit access write is the fifth step in fact 1's order, after the journal record write and second barrier, so it falls within stage T2 where the journal record has been sent but root slot has not succeeded.  
follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only  
consistent  
Justification: "first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."  
This would be refuted by: observing that the TABLE arm's action at P5 does not perform a probe write or act based on its result as per fact 4's core rule.  

Arm TABLE, P6:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T3  
Reasoning: the system config slot rotation occurs after the root slot force unit access write succeeds, so failure during this step falls within stage T3.  
follow the failure table, facts 4 through 6, 9, and 10: first perform a probe write to a fixed location on the target device; if it succeeds, go to an instance switch as defined in fact 10; if it fails, go read only until the next mount; if the count of consecutive switches exceeds N_switch, also go read only  
consistent  
Justification: "first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."  
This would be refuted by: observing that the TABLE arm's action at P6 does not perform a probe write or act based on its result as per fact 4's core rule.  

Arm TABLE, P7:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T4  
Reasoning: the failure during number taking for an ordinary mount involves system config slot writes used to take a new instance number, which matches the T4 stage definition.  
same as arm STOP's stage T4  
conflict  
Justification: "the response is an instance switch" if probe succeeds, "going read only" if fails, but the action does not include performing a probe write or acting on its result.  
This would be refuted by: observing that the TABLE arm's action at P7 includes performing a probe write followed by instance switch or read only as per fact 4's core rule.  

Arm TABLE, P8:  
inside scope  
Justification: "when a failure occurs, first perform one probe write to a fixed location on the target device."  
T4  
Reasoning: the failure during the switch's own number taking and row writing involves system config slot writes used to take a new instance number, which matches the T4 stage definition.  
same as arm STOP's stage T4  
conflict  
Justification: "the response is an instance switch" if probe succeeds, "going read only" if fails, but the action does not include performing a probe write or acting on its result.  
This would be refuted by: observing that the TABLE arm's action at P8 includes performing a probe write followed by instance switch or read only as per fact 4's core rule.  

2  
no such case  
This would be refuted by: identifying a specific row where for the same arm at two different failure points, one is marked consistent and one conflict due to inconsistent application of the same reasoning.
