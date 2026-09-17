1
(a) U1 not isolated U2 not isolated U3 not isolated (b) 34 (c) 0 (d) release-generation 9 reclaim-floor 0 not reclaimable (e) nothing currently available
This would be refuted by: if slot 50179 and slot 50180 were both reclaimable at txg10

2
(a) U1 not isolated U2 not isolated U3 not isolated (b) 34 (c) 0 (d) release-generation 9 reclaim-floor 11 reclaimable (e) 50176-50177
This would be refuted by: if U1 was not reclaimable at txg16

3
(a) U1 not isolated U2 not isolated U3 not isolated (b) 34 (c) 0 (d) release-generation 9 reclaim-floor 11 reclaimable (e) 50176-50177
This would be refuted by: if U1 was isolated at txg17

4
(a) U1 isolated U2 not isolated U3 not isolated (b) 36 (c) +2 (d) release-generation 9 reclaim-floor 0 not reclaimable (e) nothing currently available
This would be refuted by: if U1 was not isolated under conservative reading at txg10

5
(a) U1 isolated U2 not isolated U3 not isolated (b) 36 (c) +2 (d) release-generation 9 reclaim-floor 11 reclaimable (e) 50178-50179
This would be refuted by: if U1 was not isolated at txg16

6
(a) U1 isolated U2 not isolated U3 not isolated (b) 36 (c) +2 (d) release-generation 9 reclaim-floor 11 reclaimable (e) 50178-50179
This would be refuted by: if U1 was not isolated at txg17

7
(a) U1 not isolated U2 not isolated U3 not isolated (b) 34 (c) 0 (d) release-generation 9 reclaim-floor 0 not reclaimable (e) nothing currently available
This would be refuted by: if U1 was isolated under G5 reading at txg10

8
(a) U1 isolated U2 not isolated U3 not isolated (b) 36 (c) +2 (d) release-generation 9 reclaim-floor 11 reclaimable (e) 50178-50179
This would be refuted by: if U1 was not isolated at txg16 under G5 reading

9
(a) U1 isolated U2 not isolated U3 not isolated (b) 36 (c) +2 (d) release-generation 9 reclaim-floor 11 reclaimable (e) 50178-50179
This would be refuted by: if U1 was not isolated at txg17 under G5 reading

10
not equivalent; first timepoint txg16 narrow and conservative differ (narrow: 50176-50177 conservative: 50178-50179)
This would be refuted by: if narrow and conservative produced the same lowest-issuable pair at txg16

11
narrow: referenced by B and violation; conservative: no reference and no violation; G5: no reference and no violation
This would be refuted by: if for narrow the lowest-issuable pair was not referenced by an abandoned root or if for conservative or G5 the lowest-issuable pair was referenced by an abandoned root

12
it could not happen in this particular script as given because there are other free slots available (e.g. 50178-50179 and 50180-50181) that can be allocated for writes even with the extra 2 slots isolated
This would be refuted by: if the only free slots were exactly U1 and no other slots were available for allocation

13
I-2.1 would not detect I-3.1 would not detect
This would be refuted by: if invariant I-2.1 or I-3.1 detected the corruption when U1 was handed out to E
