Z4A.narrow.txg10
U1 not isolated, U2 not isolated, U3 not isolated
34
0
release-generation 9, reclaim-floor 0, not reclaimable
nothing available

This would be refuted by: if U1 were isolated or reclaimable at txg10 under narrow reading.

Z4A.narrow.txg16
U1 not isolated, U2 not isolated, U3 not isolated
34
0
release-generation 9, reclaim-floor 11, reclaimable
50176-50177

This would be refuted by: if U1 were isolated or not reclaimable at txg16 under narrow reading.

Z4A.narrow.txg17
U1 not isolated, U2 not isolated, U3 not isolated
34
0
release-generation 9, reclaim-floor 11, reclaimable
50176-50177

This would be refuted by: if U1 were isolated or not reclaimable at txg17 under narrow reading.

Z4A.conservative.txg10
U1 isolated, U2 not isolated, U3 not isolated
36
+2
release-generation 9, reclaim-floor 0, not reclaimable
nothing available

This would be refuted by: if U1 were not isolated or isolated count not 36 at txg10 under conservative reading.

Z4A.conservative.txg16
U1 isolated, U2 not isolated, U3 not isolated
36
+2
release-generation 9, reclaim-floor 11, reclaimable
50178-50179

This would be refuted by: if U1 were not isolated or lowest-issuable pair not 50178-50179 at txg16 under conservative reading.

Z4A.conservative.txg17
U1 isolated, U2 not isolated, U3 not isolated
36
+2
release-generation 9, reclaim-floor 11, reclaimable
50178-50179

This would be refuted by: if U1 were not isolated or lowest-issuable pair not 50178-50179 at txg17 under conservative reading.

Z4A.G5.txg10
U1 not isolated, U2 not isolated, U3 not isolated
34
0
release-generation 9, reclaim-floor 0, not reclaimable
nothing available

This would be refuted by: if U1 were isolated or reclaimable at txg10 under G5 reading.

Z4A.G5.txg16
U1 isolated, U2 not isolated, U3 not isolated
36
+2
release-generation 9, reclaim-floor 11, reclaimable
50178-50179

This would be refuted by: if U1 were not isolated or lowest-issuable pair not 50178-50179 at txg16 under G5 reading.

Z4A.G5.txg17
U1 isolated, U2 not isolated, U3 not isolated
36
+2
release-generation 9, reclaim-floor 11, reclaimable
50178-50179

This would be refuted by: if U1 were not isolated or lowest-issuable pair not 50178-50179 at txg17 under G5 reading.

Z4B
Not equivalent; first timepoint txg16, narrow and conservative differ; narrow gives 50176-50177, conservative gives 50178-50179

This would be refuted by: if at txg16 all readings gave the same lowest-issuable pair.

Z4C
For narrow: lowest-issuable pair 50176-50177 referenced by abandoned root B txg4, and handing out violates main clause; for conservative: lowest-issuable pair 50178-50179 not referenced by any abandoned root, so no violation; for G5: same as conservative, no violation

This would be refuted by: if for narrow the pair was not referenced by an abandoned root or violation did not occur, or for conservative or G5 the pair was referenced by an abandoned root.

Z4D
Could not happen in this script; the 2-slot difference is for U1 which is referenced by abandoned root B, so it must be isolated to satisfy the main clause; conservative correctly isolates it and reports less free space which is accurate, so when ENOSPC is reported it is correct; narrow incorrectly does not isolate U1 and may allow allocation causing corruption but does not report ENOSPC; with pool size 211968 slots, the 2-slot difference cannot cause false ENOSPC as the blocking is justified and ENOSPC reports are accurate for conservative.

This would be refuted by: if a write of size s bytes could succeed under narrow but fail under conservative with free space reported >= s for conservative.

Z4E
Root A is not in checker's candidate set at txg17 because txg=3 < F_effective=11; for I-2.1: would not detect corruption because A not in candidate set; for I-3.1: would not detect corruption because A not in candidate set

This would be refuted by: if root A were in candidate set or if either invariant detected the corruption when U1 was overlapped.
