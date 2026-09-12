1. Yes. Sequence: Instance 1 with root R1 (txg 100), journal records up to counter 50. Instance switch to instance 2, journal records for txg 101 (jsn=2, counter=0), root R2 (txg 101) written. Crash after fsync. Recovery sees R2 but replay stops at first gap (jsn 1→2 discontinuous). Data from txg 101 lost despite readable root and intact journal. One fault (crash after fsync).

2. Yes. Journal ring contains instance 0, counter 6. Instance 1 records up to counter 5. Instance 2 starts. Candidate B continues counter (5+1=6). A stale instance 0, counter 6 record is applied as next in sequence, even though it belongs to abandoned instance 0. This violates timeline integrity.

3. Impossible. Candidate C requires handover records matching prev instance and counter. Stale records without valid handover are ignored. No sequence exists where invalid records are applied; handover validation prevents this.

4. A: F2 must change (current rule stops at gap, but needs to allow instance transitions). B: F4 must change (current claim replay never crosses boundary is false). C: F2 must change (jsn not strictly consecutive; handover records required).

5. No. Journal record format is frozen (F9). First transaction’s bytes written (mkfs + publication) remain unchanged regardless of candidate. Candidates affect replay logic, not on-disk format.

6. Recommend C. A loses data during instance switches, B applies stale records from abandoned timelines, C safely handles transitions via validated handover records without data loss or corruption.
