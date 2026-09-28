1.1 V1-Today: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 0, reread: not done, selected version: A (last record 3), new instance's write E is overridden in final reopen
facts: T25, T22, T01, T11, T12, T13, T18, T17
assumption: none
overturned if: if in the final reopen, the root choice does not pick C

1.2 V1-Jia: REFUSE (writable mount for instance 3)
mount: new-instance mount for instance 3
numbers: c_witness first pass: unreadable -> true, reread: unreadable -> refuse, selected version: none
facts: T01, T02, T11, T12, T13
assumption: none
overturned if: if the reread pass can read the system configuration slots

1.3 V1-Yi: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 0, reread: not done, selected version: A, E overridden by C
facts: T25, T22, T01, T11, T12, T13, T18, T17
assumption: for Yi, "only when both slots of one device are unreadable is it treated as undecided" means that device is considered to have no slot and the max is taken over the readable slots of the other devices
overturned if: if the final reopen does not override E

1.4 V1-Bing: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 0, reread: not done, selected version: A, E overridden by C
facts: T25, T22, T01, T11, T12, T13, T18, T17
assumption: for Bing, "merge the two passes: the reread pass's c_witness is the maximum of what the two passes read" means the max of the two c_witness values
overturned if: if the final reopen does not override E

1.5 V1-Ding: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 0, reread: not done, selected version: A, E overridden by C
facts: T25, T22, T01, T11, T12, T13, T18, T17
assumption: "keep today" means the current behavior is unchanged
overturned if: if the final reopen does not override E

1.6 V2-Today: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 7, reread: 0, selected version: A, E overridden by C
facts: T26, T22, T01, T11, T12, T13, T18, T17
assumption: none
overturned if: if the final reopen does not override E

1.7 V2-Jia: REFUSE (writable mount for instance 3)
mount: new-instance mount for instance 3
numbers: c_witness first pass: 7, reread: unreadable -> refuse, selected version: none
facts: T01, T02, T11, T12, T13
assumption: none
overturned if: if the reread pass can read the system configuration slots

1.8 V2-Yi: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 7, reread: 0, selected version: A, E overridden by C
facts: T26, T22, T01, T11, T12, T13, T18, T17
assumption: for Yi, "only when both slots of one device are unreadable is it treated as undecided" means that device is considered to have no slot and the max is taken over the readable slots of the other devices
overturned if: if the final reopen does not override E

1.9 V2-Bing: REFUSE (writable mount for instance 3)
mount: new-instance mount for instance 3
numbers: c_witness first pass: 7, reread: 0, merged c_witness: 7, selected version: none
facts: T01, T02, T11, T12, T13, T14
assumption: for Bing, "merge the two passes: the reread pass's c_witness is the maximum of what the two passes read" means the max of the two c_witness values
overturned if: if the reread pass has a readable slot

1.10 V2-Ding: WRITABLE-LOSS (new instance's)
mount: new-instance mount and final reopen
numbers: c_witness first pass: 7, reread: 0, selected version: A, E overridden by C
facts: T26, T22, T01, T11, T12, T13, T18, T17
assumption: "keep today" means the current behavior is unchanged
overturned if: if the final reopen does not override E

1.11 V3-Today: WRITABLE-LOSS (older instance's)
mount: new-instance mount for instance 3
numbers: c_witness first pass: 6, reread: not done, selected version: B (last record 6), C is lost
facts: T27, T22, T01, T11, T12, T13, T17
assumption: none
overturned if: if C's root slot is not overwritten

1.12 V3-Jia: REFUSE (writable mount for instance 3)
mount: new-instance mount for instance 3
numbers: c_witness first pass: unreadable -> true, reread: unreadable -> refuse, selected version: none
facts: T01, T02, T11, T12, T13
assumption: none
overturned if: if the reread pass can read the system configuration slots

1.13 V3-Yi: WRITABLE-LOSS (older instance's)
mount: new-instance mount for instance 3
numbers: c_witness first pass: 6, reread: not done, selected version: B, C lost
facts: T27, T22, T01, T11, T12, T13, T17
assumption: for Yi, "only when both slots of one device are unreadable is it treated as undecided" means that device is considered to have no slot and the max is taken over the readable slots of the other devices
overturned if: if C's root slot is not overwritten

1.14 V3-Bing: WRITABLE-LOSS (older instance's)
mount: new-instance mount for instance 3
numbers: c_witness first pass: 6, reread: not done, selected version: B, C lost
facts: T27, T22, T01, T11, T12, T13, T17
assumption: for Bing, "merge the two passes: the reread pass's c_witness is the maximum of what the two passes read" means the max of the two c_witness values
overturned if: if C's root slot is not overwritten

1.15 V3-Ding: WRITABLE-LOSS (older instance's)
mount: new-instance mount for instance 3
numbers: c_witness first pass: 6, reread: not done, selected version: B, C lost
facts: T27, T22, T01, T11, T12, T13, T17
assumption: "keep today" means the current behavior is unchanged
overturned if: if C's root slot is not overwritten

1.16 V4-Today: WRITABLE-LOSS (older instance's)
mount: mount k+1
numbers: at mount k: c_witness=6, acquisition write writes tail 6; at mount k+1: c_witness=6, mount from B, C lost
facts: T28, T22, T01, T11, T12, T13, T14, T15, T17
assumption: none
overturned if: if at mount k+1, C is not lost

1.17 V4-Jia: REFUSE (mount k)
mount: mount k
numbers: at mount k: first pass unreadable -> true, reread unreadable -> refuse
facts: T01, T02, T11, T12, T13, T14, T15, T16
assumption: none
overturned if: if at mount k, the reread pass can read the system configuration slots

1.18 V4-Yi: WRITABLE-LOSS (older instance's)
mount: mount k+1
numbers: at mount k: c_witness=6, acquisition write writes tail 6; at mount k+1: c_witness=6, mount from B, C lost
facts: T28, T22, T01, T11, T12, T13, T14, T15, T17
assumption: for Yi, "only when both slots of one device are unreadable is it treated as undecided" means that device is considered to have no slot and the max is taken over the readable slots of the other devices
overturned if: if at mount k+1, C is not lost

1.19 V4-Bing: WRITABLE-LOSS (older instance's)
mount: mount k+1
numbers: at mount k: c_witness=6, acquisition write writes tail 6; at mount k+1: c_witness=6, mount from B, C lost
facts: T28, T22, T01, T11, T12, T13, T14, T15, T17
assumption: for Bing, "merge the two passes: the reread pass's c_witness is the maximum of what the two passes read" means the max of the two c_witness values
overturned if: if at mount k+1, C is not lost

1.20 V4-Ding: WRITABLE-LOSS (older instance's)
mount: mount k+1
numbers: at mount k: c_witness=6, acquisition write writes tail 6; at mount k+1: c_witness=6, mount from B, C lost
facts: T28, T22, T01, T11, T12, T13, T14, T15, T17
assumption: "keep today" means the current behavior is unchanged
overturned if: if at mount k+1, C is not lost

2.1 R1-Today: isolation 14 / 14, counter 0, none through txg 31
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the isolation is not 14/14

2.2 R1-(a): isolation 14 / 14, counter 0, none through txg 31
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the isolation is not 14/14

2.3 R1-(d): isolation 14 / 14, counter 0, none through txg 31
facts: T35, T32, T33, T34, T35
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the isolation is not 14/14

2.4 R2-Today: isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse does not happen at txg 14

2.5 R2-(a): mount refused
facts: T35, T32, T33, T34, T35, T02, T19
assumption: candidate (a) does exactly one reread (two reads total) for the account
overturned if: if after one reread the allocation record tree becomes readable

2.6 R2-(d): isolation 14 / 14, counter 1, none through txg 31
facts: T35, T32, T33, T34, T35, T09, T10
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the allocator reuses a slot of instance 2 before the root leaves the root ring

2.7 R3-Today: isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse does not happen at txg 14

2.8 R3-(a): mount refused
facts: T35, T32, T33, T34, T35, T02, T19
assumption: candidate (a) does exactly one reread (two reads total) for the account
overturned if: if after one reread the allocation record tree becomes readable

2.9 R3-(d): isolation 14 / 14, counter 1, none through txg 31
facts: T35, T32, T33, T34, T35, T09, T10
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the allocator reuses a slot of instance 2 before the root leaves the root ring

2.10 R4-Today: isolation 14 / 14, counter 0, none through txg 31
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse happens before txg 31

2.11 R4-(a): isolation 14 / 14, counter 0, none through txg 31
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse happens before txg 31

2.12 R4-(d): isolation 14 / 14, counter 0, none through txg 31
facts: T35, T32, T33, T34, T35, T09, T10
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the reuse happens before txg 31

2.13 R5-Today: isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse does not happen at txg 14

2.14 R5-(a): mount refused
facts: T35, T32, T33, T34, T35, T02, T19
assumption: candidate (a) does exactly one reread (two reads total) for the account
overturned if: if after one reread the tree table becomes readable

2.15 R5-(d): isolation 14 / 14, counter 1, none through txg 31
facts: T35, T32, T33, T34, T35, T09, T10
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the allocator reuses a slot of instance 2 before the root leaves the root ring

2.16 R6-Today: isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse does not happen at txg 14

2.17 R6-(a): mount refused
facts: T35, T32, T33, T34, T35, T02, T19
assumption: candidate (a) does exactly one reread (two reads total) for the account
overturned if: if after one reread the tree table becomes readable

2.18 R6-(d): isolation 14 / 14, counter 1, none through txg 31
facts: T35, T32, T33, T34, T35, T09, T10
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the allocator reuses a slot of instance 2 before the root leaves the root ring

2.19 R7-Today: isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse does not happen at txg 14

2.20 R7-(a): isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35
assumption: none
overturned if: if the reuse does not happen at txg 14

2.21 R7-(d): isolation 0 / 0, counter 1, reused at txg 14
facts: T35, T32, T33, T34, T35, T09, T10
assumption: "slots written by the instance" means all slots written by instance 2
overturned if: if the reuse does not happen at txg 14

3.1 Today: (i) 6 (ii) no, device0: 1 self-verified slot, device1: 1 self-verified slot (iii) no
overturned if: if the acquisition write writes a different value or if T05 fires or if C is not lost

3.2 Jia: (i) not written (ii) no, device0: 1 self-verified slot, device1: 1 self-verified slot (iii) yes
overturned if: if after refusal at mount k, C is still lost

3.3 Yi: (i) 6 (ii) no, device0: 1 self-verified slot, device1: 1 self-verified slot (iii) no
overturned if: if the acquisition write writes a different value or if C is not lost

3.4 Bing: (i) 6 (ii) no, device0: 1 self-verified slot, device1: 1 self-verified slot (iii) no
overturned if: if the acquisition write writes a different value or if C is not lost

3.5 Ding: (i) 6 (ii) no, device0: 1 self-verified slot, device1: 1 self-verified slot (iii) no
overturned if: if the acquisition write writes a different value or if C is not lost

4.1 V1-Yi: missing definition for Yi: how "treated as undecided" affects the witness calculation
4.2 V1-Bing: missing definition for Bing: how to merge the two passes for c_witness
4.3 V1-Ding: missing definition for Ding: what "keep today" means exactly
4.4 V2-Yi: missing definition for Yi: how "treated as undecided" affects the witness calculation
4.5 V2-Bing: missing definition for Bing: how to merge the two passes for c_witness
4.6 V2-Ding: missing definition for Ding: what "keep today" means exactly
4.7 V3-Yi: missing definition for Yi: how "treated as undecided" affects the witness calculation
4.8 V3-Bing: missing definition for Bing: how to merge the two passes for c_witness
4.9 V3-Ding: missing definition for Ding: what "keep today" means exactly
4.10 V4-Yi: missing definition for Yi: how "treated as undecided" affects the witness calculation
4.11 V4-Bing: missing definition for Bing: how to merge the two passes for c_witness
4.12 V4-Ding: missing definition for Ding: what "keep today" means exactly
4.13 R2-(d): missing definition for (d): what "slots written by the instance" means
4.14 R3-(d): missing definition for (d): what "slots written by the instance" means
4.15 R4-(d): missing definition for (d): what "slots written by the instance" means
4.16 R5-(d): missing definition for (d): what "slots written by the instance" means
4.17 R6-(d): missing definition for (d): what "slots written by the instance" means
4.18 R7-(d): missing definition for (d): what "slots written by the instance" means
4.19 R2-(a): missing definition for (a): how many reads are done for the account
4.20 R3-(a): missing definition for (a): how many reads are done for the account
4.21 R5-(a): missing definition for (a): how many reads are done for the account
4.22 R6-(a): missing definition for (a): how many reads are done for the account
