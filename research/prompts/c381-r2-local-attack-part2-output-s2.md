1.  
P5  
C1: inside scope  
justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."  
C2: inside scope  
justification: "the real discriminant remains the probe write."  
C3: outside scope  
justification: "the switch's own first step is itself a write that can fail: a switch must first take a new instance number, and taking that number requires writing to every system config copy that this mount exclusively opened successfully, all or nothing."  
C4: inside scope  
justification: "Write failure is guarded against by the probe write and by the number taking write's all or nothing rule."  
C5: inside scope  
justification: "When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending."  
C6: inside scope  
justification: "An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint."  

P6  
C1: inside scope  
justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."  
C2: inside scope  
justification: "the real discriminant remains the probe write."  
C3: outside scope  
justification: "the switch's own first step is itself a write that can fail: a switch must first take a new instance number, and taking that number requires writing to every system config copy that this mount exclusively opened successfully, all or nothing."  
C4: inside scope  
justification: "Write failure is guarded against by the probe write and by the number taking write's all or nothing rule."  
C5: outside scope  
justification: "When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending."  
C6: inside scope  
justification: "An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint."  

P7  
C1: outside scope  
justification: fact 4 describes failure handling for publish attempts and P7 occurs during ordinary mount number taking which is not part of a publish attempt.  
C2: outside scope  
justification: fact 5 references switch reservation and consecutive switches which do not apply to ordinary mount number taking.  
C3: outside scope  
justification: fact 6 refers to switch's own number taking which is distinct from ordinary mount number taking.  
C4: inside scope  
justification: "Write failure is guarded against by the probe write and by the number taking write's all or nothing rule."  
C5: outside scope  
justification: "When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending."  
C6: outside scope  
justification: "An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint."  

P8  
C1: inside scope  
justification: "when a failure occurs, first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient, and the response is an instance switch. If that probe write fails, the failure is classified as persistent, and the response is going read only until the next mount."  
C2: inside scope  
justification: "the real discriminant remains the probe write."  
C3: inside scope  
justification: "the switch's own first step is itself a write that can fail: a switch must first take a new instance number, and taking that number requires writing to every system config copy that this mount exclusively opened successfully, all or nothing."  
C4: inside scope  
justification: "Write failure is guarded against by the probe write and by the number taking write's all or nothing rule."  
C5: outside scope  
justification: "When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending."  
C6: inside scope  
justification: "An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint."  

This would be refuted by: any observation where the text explicitly states fact 4's core rule applies only to publish failures and excludes instance switch failures.  

2.  
Fact 5's N_switch clause by itself does not force a definite verdict for whether P8 falls inside the scope of clause C1; the verdict depends on whether fact 4's core rule applies to failures during an instance switch's number taking.  
This would be refuted by: if the text explicitly states that fact 4's core rule applies only to publish failures and not to instance switch failures.
