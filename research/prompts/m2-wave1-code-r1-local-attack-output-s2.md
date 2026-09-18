W1.start0.pub1: checkpoint_txg 1 jsn 1 tail 1 This would be refuted by checkpoint_txg not equal to 1 or jsn not equal to 1 or tail not equal to 1
W1.start0.pub2: checkpoint_txg 2 jsn 2 tail 2 This would be refuted by checkpoint_txg not equal to 2 or jsn not equal to 2 or tail not equal to 2
W1.start1.pub1: checkpoint_txg 1 jsn 2 tail 2 This would be refuted by checkpoint_txg not equal to 1 or jsn not equal to 2 or tail not equal to 2
W1.start1.pub2: checkpoint_txg 2 jsn 3 tail 3 This would be refuted by checkpoint_txg not equal to 2 or jsn not equal to 3 or tail not equal to 3
W1.start40.pub1: checkpoint_txg 1 jsn 41 tail 41 This would be refuted by checkpoint_txg not equal to 1 or jsn not equal to 41 or tail not equal to 41
W1.start40.pub2: checkpoint_txg 2 jsn 42 tail 42 This would be refuted by checkpoint_txg not equal to 2 or jsn not equal to 42 or tail not equal to 42
W2.devices1: accounting row count 9 max entries 477 admission verdict passes For device count 1 D5 item 8 provides no directly checkable number this relies on the formula This would be refuted by row count not 9 or max entries not 477 or admission verdict not passes
W2.devices2: accounting row count 15 max entries 477 admission verdict passes The row count of 15 matches D5 item 8's anchor for a 2-device pool This would be refuted by row count not 15 or max entries not 477 or admission verdict not passes
W2.devices78: accounting row count 471 max entries 477 admission verdict passes For device count 78 D5 item 8 provides no directly checkable number this relies on the formula This would be refuted by row count not 471 or max entries not 477 or admission verdict not passes
W2.devices79: accounting row count 477 max entries 477 admission verdict passes For device count 79 D5 item 8 provides no directly checkable number this relies on the formula This would be refuted by row count not 477 or max entries not 477 or admission verdict not passes
W2.devices80: accounting row count 483 max entries 477 admission verdict AccountingEntriesExceedOneNode For device count 80 D5 item 8 provides no directly checkable number this relies on the formula This would be refuted by row count not 483 or max entries not 477 or admission verdict not AccountingEntriesExceedOneNode
W3.four_in_a_row: 0 1 2 3 This would be refuted by any returned number not matching the sequence 0 1 2 3
W3.next_publish_first_value: 0 This would be refuted by the first call in a new publish returning a number other than 0
