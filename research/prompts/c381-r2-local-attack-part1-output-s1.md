1
inside scope. Fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device'.
inside scope. Fact 5 states 'If the count of writable devices falls below a floor called w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch'.
outside scope. Fact 6 specifies 'the switch's own first step is itself a write that can fail', which applies only to instance switch steps, not publish failures.
inside scope. Fact 7 states 'Write failure is guarded against by the probe write and by the number taking write's all or nothing rule'.
text does not decide. The text would need to specify handling for failures before root slot write, but Fact 9 only addresses root slot write failures.
inside scope. Fact 10 defines 'An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint'.
inside scope. Fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device'.
inside scope. Fact 5 states 'If the count of writable devices falls below a floor called w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch'.
outside scope. Fact 6 specifies 'the switch's own first step is itself a write that can fail', which applies only to instance switch steps, not publish failures.
inside scope. Fact 7 states 'Write failure is guarded against by the probe write and by the number taking write's all or nothing rule'.
text does not decide. The text would need to specify handling for failures before root slot write, but Fact 9 only addresses root slot write failures.
inside scope. Fact 10 defines 'An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint'.
inside scope. Fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device'.
inside scope. Fact 5 states 'If the count of writable devices falls below a floor called w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch'.
outside scope. Fact 6 specifies 'the switch's own first step is itself a write that can fail', which applies only to instance switch steps, not publish failures.
inside scope. Fact 7 states 'Write failure is guarded against by the probe write and by the number taking write's all or nothing rule'.
text does not decide. The text would need to specify handling for failures before root slot write, but Fact 9 only addresses root slot write failures.
inside scope. Fact 10 defines 'An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint'.
inside scope. Fact 4 states 'when a failure occurs, first perform one probe write to a fixed location on the target device'.
inside scope. Fact 5 states 'If the count of writable devices falls below a floor called w, or if the switch's own reservation cannot be obtained, the response also goes straight to the read only branch'.
outside scope. Fact 6 specifies 'the switch's own first step is itself a write that can fail', which applies only to instance switch steps, not publish failures.
inside scope. Fact 7 states 'Write failure is guarded against by the probe write and by the number taking write's all or nothing rule'.
text does not decide. The text would need to specify handling for failures before root slot write, but Fact 9 only addresses root slot write failures.
inside scope. Fact 10 defines 'An instance switch is a recovery performed within the current mount: it takes a new instance number, writes a row, and resends the in flight checkpoint'.
This would be refuted by Fact 9 explicitly describing handling for failures during unit writes or barriers.
