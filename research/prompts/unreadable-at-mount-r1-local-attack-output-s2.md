Q1.1 V1-Today: WRITABLE-LOSS
mount: new-instance mount
numbers: first pass c_witness 0, no reread. Selected version (1,3) last record 3
facts: T25, T12, T13
assumption: none
overturned if: final reopen does not override E

Q1.2 V1-Jia: REFUSE
mount: new-instance mount
numbers: first pass c_witness treated as true, reread still unreadable. Selected version not acquired
facts: T01, T02, T13
assumption: reread pass remains unreadable
overturned if: reread slots become readable

Q1.3 V1-Yi: REFUSE
mount: new-instance mount
numbers: each device has zero self-verified slots
facts: T05, T16
assumption: T05 applies when both slots of a device are unreadable
overturned if: a device has at least one self-verified slot

Q1.4 V1-Bing: WRITABLE-LOSS
mount: new-instance mount
numbers: merged c_witness 0, selected version (1,3) last record 3
facts: T12, T13
assumption: merged c_witness is 0
overturned if: merged c_witness > 3

Q1.5 V1-Ding: WRITABLE-LOSS
mount: new-instance mount
numbers: same as Today
facts: T25
assumption: none
overturned if: final reopen does not override E

Q1.6 V2-Today: WRITABLE-LOSS
mount: new-instance mount
numbers: first pass c_witness 7, reread 0. Selected version (1,3) last record 3
facts: T26
assumption: none
overturned if: E not overridden

Q1.7 V2-Jia: REFUSE
mount: new-instance mount
numbers: first pass true, reread still unreadable
facts: T01, T02, T13
assumption: reread pass remains unreadable
overturned if: reread slots become readable

Q1.8 V2-Yi: REFUSE
mount: new-instance mount
numbers: during reread, each device has zero self-verified slots
facts: T05, T16
assumption: T05 applies when both slots of a device are unreadable
overturned if: a device has at least one self-verified slot

Q1.9 V2-Bing: REFUSE
mount: new-instance mount
numbers: merged c_witness 7 > 3
facts: T12, T13
assumption: merged c_witness is 7
overturned if: merged c_witness <= 3

Q1.10 V2-Ding: WRITABLE-LOSS
mount: new-instance mount
numbers: same as Today
facts: T26
assumption: none
overturned if: E not overridden

Q1.11 V3-Today: WRITABLE-LOSS
mount: new-instance mount
numbers: c_witness 6 equals B's last record 6. Selected version (1,6)
facts: T27
assumption: none
overturned if: C not lost

Q1.12 V3-Jia: REFUSE
mount: new-instance mount
numbers: witness read has unreadable slots, reread still unreadable
facts: T01, T02, T13
assumption: reread pass remains unreadable
overturned if: reread slots become readable

Q1.13 V3-Yi: WRITABLE-LOSS
mount: new-instance mount
numbers: each device has at least one readable slot, c_witness 6
facts: T27, T16
assumption: other slots on devices are readable
overturned if: both slots of a device are unreadable

Q1.14 V3-Bing: WRITABLE-LOSS
mount: new-instance mount
numbers: merged c_witness 6 equals B's last record
facts: T27, T13
assumption: merged c_witness is 6
overturned if: merged c_witness > 6

Q1.15 V3-Ding: WRITABLE-LOSS
mount: new-instance mount
numbers: same as Today
facts: T27
assumption: none
overturned if: C not lost

Q1.16 V4-Today: WRITABLE-LOSS
mount: mount k+1
numbers: acquisition write tail 6 at mount k. c_witness 6 at mount k+1
facts: T28, T14
assumption: none
overturned if: C not lost

Q1.17 V4-Jia: REFUSE
mount: mount k
numbers: mount refused at acquisition
facts: T01, T02, T13
assumption: reread pass remains unreadable
overturned if: reread slots become readable

Q1.18 V4-Yi: WRITABLE-LOSS
mount: mount k+1
numbers: acquisition write tail 6, c_witness 6. Each device has at least one self-verified slot
facts: T28, T16, T05
assumption: each device has at least one readable slot
overturned if: both slots of a device are unreadable

Q1.19 V4-Bing: WRITABLE-LOSS
mount: mount k+1
numbers: merged c_witness 6
facts: T28, T13
assumption: merged c_witness is 6
overturned if: merged c_witness > B's last record

Q1.20 V4-Ding: WRITABLE-LOSS
mount: mount k+1
numbers: same as Today
facts: T28
assumption: none
overturned if: C not lost

Q2.1 R1-Today: WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: none
overturned if: reuse before txg 31

Q2.2 R1-(a): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T02
assumption: none
overturned if: reuse before txg 31

Q2.3 R1-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q2.4 R2-Today: WRITABLE-REUSE
numbers: isolation 0/0, counter 1, txg 14
facts: T35
assumption: none
overturned if: no reuse

Q2.5 R2-(a): REFUSE
numbers: isolation N/A, counter N/A, none through txg 31
facts: T02, T19
assumption: after one reread still unreadable
overturned if: reread makes it readable

Q2.6 R2-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q2.7 R3-Today: WRITABLE-REUSE
numbers: isolation 0/0, counter 1, txg 14
facts: T35
assumption: hook called 0 times
overturned if: no reuse

Q2.8 R3-(a): REFUSE
numbers: isolation N/A, counter N/A, none through txg 31
facts: T02, T19
assumption: hook called 0 times so reread does not lift fault
overturned if: hook called and lifts fault

Q2.9 R3-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q2.10 R4-Today: WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T36
assumption: second read succeeds
overturned if: reuse before txg 31

Q2.11 R4-(a): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T02, T36
assumption: second read succeeds
overturned if: reuse before txg 31

Q2.12 R4-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q2.13 R5-Today: WRITABLE-REUSE
numbers: isolation 0/0, counter 1, txg 14
facts: T35
assumption: none
overturned if: no reuse

Q2.14 R5-(a): REFUSE
numbers: isolation N/A, counter N/A, none through txg 31
facts: T02, T19
assumption: after one reread still unreadable
overturned if: reread makes it readable

Q2.15 R5-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q2.16 R6-Today: WRITABLE-REUSE
numbers: isolation 0/0, counter 1, txg 14
facts: T35
assumption: hook called 0 times
overturned if: no reuse

Q2.17 R6-(a): REFUSE
numbers: isolation N/A, counter N/A, none through txg 31
facts: T02, T19
assumption: hook called 0 times so reread does not lift fault
overturned if: hook called and lifts fault

Q2.18 R6-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q2.19 R7-Today: WRITABLE-REUSE
numbers: isolation 0/0, counter 1, txg 14
facts: T35, T36
assumption: second read succeeds for tree table
overturned if: no reuse

Q2.20 R7-(a): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T02, T36
assumption: second read succeeds
overturned if: reuse before txg 31

Q2.21 R7-(d): WRITABLE-NO-LOSS
numbers: isolation 14/14, counter 0, none through txg 31
facts: T35, T19
assumption: "slots written by the instance" means all slots allocated by instance 2
overturned if: slots reused

Q3.1 Today:
(i) 6
(ii) no refusal; each device has at least one self-verified slot
(iii) no, C is lost
facts: T28, T14, T05
overturned if: device has no self-verified slots

Q3.2 Jia:
(i) no acquisition write
(ii) refusal due to Jia's rule; each device has unreadable slots
(iii) yes, C is kept
facts: T01, T02, T13
overturned if: reread slots become readable

Q3.3 Yi:
(i) 6
(ii) no refusal; each device has at least one self-verified slot
(iii) no, C is lost
facts: T28, T16, T05
overturned if: device has no self-verified slots

Q3.4 Bing:
(i) 6
(ii) no refusal; each device has at least one self-verified slot
(iii) no, C is lost
facts: T28, T13
overturned if: merged c_witness > B's last record

Q3.5 Ding:
(i) 6
(ii) no refusal; each device has at least one self-verified slot
(iii) no, C is lost
facts: T28
overturned if: device has no self-verified slots

Q4.1 V1-Jia: missing definition for whether reread passes remain unreadable
Q4.2 V1-Yi: missing definition for T05 application to devices with both slots unreadable
Q4.3 V2-Jia: missing definition for whether reread passes remain unreadable
Q4.4 V2-Yi: missing definition for T05 application to devices with both slots unreadable
Q4.5 V3-Jia: missing definition for whether reread passes remain unreadable
Q4.6 V3-Yi: missing definition for whether other slots on devices are readable
Q4.7 V4-Jia: missing definition for whether reread passes remain unreadable
Q4.8 V4-Yi: missing definition for whether other slots on devices are readable
Q4.9 R2-(a): missing definition for whether reread makes it readable
Q4.10 R3-(a): missing definition for whether hook is called
Q4.11 R4-(a): missing definition for whether second read succeeds
Q4.12 R5-(a): missing definition for whether reread makes it readable
Q4.13 R6-(a): missing definition for whether hook is called
Q4.14 R7-(a): missing definition for whether second read succeeds
Q4.15 R2-(d): missing definition for "slots written by the instance" meaning
Q4.16 R3-(d): missing definition for "slots written by the instance" meaning
Q4.17 R4-(d): missing definition for "slots written by the instance" meaning
Q4.18 R5-(d): missing definition for "slots written by the instance" meaning
Q4.19 R6-(d): missing definition for "slots written by the instance" meaning
Q4.20 R7-(d): missing definition for "slots written by the instance" meaning
Q4.21 R1-(d): missing definition for "slots written by the instance" meaning
