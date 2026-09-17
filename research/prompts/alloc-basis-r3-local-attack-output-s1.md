1 21 yes 32
This would be refuted by: if Q's admission reading were less than 16, such as if in_flight_approved were higher or gate1 were lower.

2 37 5 definition (i)
This would be refuted by: if df were reported as 5 when gate1 is 37, for example.

3 5
This would be refuted by: if a request for 5 blocks failed or for 6 blocks succeeded.

4 yes s=6 df=37 >=6 but admission reading=5<6
This would be refuted by: if for s=6, the write succeeded or df was less than 6.

5 0
This would be refuted by: if F_effective were 9 or higher.

6 5
This would be refuted by: if gate1 were 9 or higher, for example if defer_pending was not increased on one device.

7 ENOSPC gate1=5 <16
This would be refuted by: if gate1 was at least 16 or if the write succeeded.

8 fails with ENOSPC every single time forever absence of rule to handle inconsistent state
This would be refuted by: if the system resolved the inconsistency and succeeded, or if a rule prevented the inconsistency.

9 SA yes s=6 df=37 >=6 but admission reading=5<6 SB no
This would be refuted for SA by: if for s=6, the write succeeded or df was less than 6. For SB by: if there was a case where df >= s but write failed, e.g., if df reported 10 and write of 5 blocks failed.

10 no delete in SA so requirement does not apply
This would be refuted by: if a delete occurred in Section SA.

11 yes
This would be refuted by: if the write-row publish did not carry user data.

12 no
This would be refuted by: if the bound applied to plain new writes.
