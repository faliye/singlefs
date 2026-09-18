W1.start0.pub1 checkpoint_txg 1 jsn 1 tail 1
This would be refuted by: checkpoint_txg not equal to 1 for publish index 1.

W1.start0.pub2 checkpoint_txg 2 jsn 2 tail 2
This would be refuted by: jsn counter not equal to 2 for publish index 2.

W1.start1.pub1 checkpoint_txg 1 jsn 2 tail 2
This would be refuted by: jsn counter not equal to 2 for publish index 1.

W1.start1.pub2 checkpoint_txg 2 jsn 3 tail 3
This would be refuted by: tail not equal to jsn counter for publish index 2.

W1.start40.pub1 checkpoint_txg 1 jsn 41 tail 41
This would be refuted by: jsn counter not equal to 41 for publish index 1.

W1.start40.pub2 checkpoint_txg 2 jsn 42 tail 42
This would be refuted by: tail not equal to jsn counter for publish index 2.

W2.devices1 accounting row count 9 max entries 477 admission passes
The quoted decision text gives no directly checkable number for this device count; the answer relies only on the formula given in this section.

W2.devices2 accounting row count 15 max entries 477 admission passes
This would be refuted by: accounting row count not equal to 15 for device count 2.

W2.devices78 accounting row count 471 max entries 477 admission passes
The quoted decision text gives no directly checkable number for this device count; the answer relies only on the formula given in this section.

W2.devices79 accounting row count 477 max entries 477 admission passes
The quoted decision text gives no directly checkable number for this device count; the answer relies only on the formula given in this section.

W2.devices80 accounting row count 483 max entries 477 admission AccountingEntriesExceedOneNode
The quoted decision text gives no directly checkable number for this device count; the answer relies only on the formula given in this section.

W3.four_in_a_row 0 1 2 3
This would be refuted by: the first number not being 0 when the same triple is called for the first time in a publish.

W3.next_publish_first_value 0
This would be refuted by: the first call in a new publish returning a number other than 0.
