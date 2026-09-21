1
P1 C1 inside scope when a failure occurs, first perform one probe write to a fixed location on the target device
P1 C2 inside scope if the count of writable devices falls below w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch
P1 C3 outside scope the switch's own first step is itself a write that can fail
P1 C4 outside scope The reservation guards against the allocation failure path, not the write failure path
P1 C5 outside scope When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending
P1 C6 outside scope it takes a new instance number, writes a row, and resends the in flight checkpoint
P2 C1 inside scope when a failure occurs, first perform one probe write to a fixed location on the target device
P2 C2 inside scope if the count of writable devices falls below w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch
P2 C3 outside scope the switch's own first step is itself a write that can fail
P2 C4 outside scope The reservation guards against the allocation failure path, not the write failure path
P2 C5 outside scope When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending
P2 C6 outside scope it takes a new instance number, writes a row, and resends the in flight checkpoint
P3 C1 inside scope when a failure occurs, first perform one probe write to a fixed location on the target device
P3 C2 inside scope if the count of writable devices falls below w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch
P3 C3 outside scope the switch's own first step is itself a write that can fail
P3 C4 outside scope The reservation guards against the allocation failure path, not the write failure path
P3 C5 outside scope When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending
P3 C6 outside scope it takes a new instance number, writes a row, and resends the in flight checkpoint
P4 C1 inside scope when a failure occurs, first perform one probe write to a fixed location on the target device
P4 C2 inside scope if the count of writable devices falls below w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch
P4 C3 outside scope the switch's own first step is itself a write that can fail
P4 C4 outside scope The reservation guards against the allocation failure path, not the write failure path
P4 C5 outside scope When a root slot write fails and the publish is resent, the checkpoint's txg number is advanced by one before resending
P4 C6 outside scope it takes a new instance number, writes a row, and resends the in flight checkpoint
This would be refuted by: fact 4 explicitly stating that probe write is not performed for unit write failures
