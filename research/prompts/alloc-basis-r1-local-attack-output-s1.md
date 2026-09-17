Y2.A.Serial.1 no. This would be refuted by: a scenario where df reports 10 and a write of 10 bytes succeeds despite Gate D16's allocatable being less than 10.

Y2.A.Serial.2 yes. This would be refuted by: after deleting 10 bytes, a write of 10 bytes fails after 3 user-visible publishes.

Y2.A.Merged.1 no. This would be refuted by: a scenario where df reports 7 and a write of 7 bytes succeeds despite Gate D16's allocatable being less than 7.

Y2.A.Merged.2 yes. This would be refuted by: after deleting 10 bytes, a write of 10 bytes fails after 3 user-visible publishes.

Y2.B.Serial.1 no. This would be refuted by: a scenario where df reports 10 and a write of 10 bytes succeeds despite Gate D16's allocatable being less than 10.

Y2.B.Serial.2 yes. This would be refuted by: after deleting 10 bytes, a write of 10 bytes fails after 3 user-visible publishes.

Y2.B.Merged.1 no. This would be refuted by: a scenario where df reports 7 and a write of 7 bytes succeeds despite Gate D16's allocatable being less than 7.

Y2.B.Merged.2 yes. This would be refuted by: after deleting 10 bytes, a write of 10 bytes fails after 3 user-visible publishes.

Y2.C.Serial.1 no. This would be refuted by: a scenario where df reports 10 and a write of 10 bytes succeeds despite Gate D16's allocatable being less than 10.

Y2.C.Serial.2 no. This would be refuted by: after deleting 10 bytes, a write of 10 bytes succeeds within 3 user-visible publishes.

Y2.C.Merged.1 no. This would be refuted by: a scenario where df reports 7 and a write of 7 bytes succeeds despite Gate D16's allocatable being less than 7.

Y2.C.Merged.2 no. This would be refuted by: after deleting 10 bytes, a write of 10 bytes succeeds within 3 user-visible publishes.

Y5.A.Scenario1 I-3.1 violated I-5.2 holds. This would be refuted by: item 1 equals the sum of references over the rollback candidate set and free equals capacity minus allocated.

Y5.A.Scenario2 I-3.1 violated I-5.2 holds. This would be refuted by: item 1 equals the sum of references over the rollback candidate set and free equals capacity minus allocated.

Y5.B.Scenario1 I-3.1 violated I-5.2 holds. This would be refuted by: item 1 equals the sum of references from the single latest valid root and free equals capacity minus allocated.

Y5.B.Scenario2 I-3.1 violated I-5.2 holds. This would be refuted by: item 1 equals the sum of references from the single latest valid root and free equals capacity minus allocated.

Y5.C.Scenario1 I-3.1 violated I-5.2 holds. This would be refuted by: item 1 equals the sum of references over all self-verifying roots and free equals capacity minus allocated.

Y5.C.Scenario2 I-3.1 violated I-5.2 holds. This would be refuted by: item 1 equals the sum of references over all self-verifying roots and free equals capacity minus allocated.
