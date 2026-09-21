1
T1: The caller receives an error. This would be refuted by: the fsync caller receiving a success status after a T1 failure.
T2: The caller receives an error. This would be refuted by: the fsync caller receiving a success status after a T2 failure.
T3: The caller receives an error. This would be refuted by: the fsync caller receiving a success status after a T3 failure.
T4: The caller receives an error. This would be refuted by: the mount attempt returning a success status after a T4 failure.

2
T1: Not durable. This would be refuted by: a subsequent mount or recovery treating the publish's data as committed.
T2: Not durable. This would be refuted by: a subsequent mount or recovery treating the publish's data as committed.
T3: Durable. This would be refuted by: a subsequent mount or recovery not selecting the root record as valid.
T4: Not durable. This would be refuted by: the new instance number appearing in system config slots after a T4 failure.

3
T1: Overstates uncertainty. This would be refuted by: after a T1 failure, the data appears in a subsequent mount.
T2: Overstates uncertainty. This would be refuted by: after a T2 failure, the data appears in a subsequent mount.
T3: Consistent. This would be refuted by: after a T3 failure, the data never appears in a subsequent mount.
T4: Overstates uncertainty. This would be refuted by: the new instance number appearing in system config slots after a T4 failure.
