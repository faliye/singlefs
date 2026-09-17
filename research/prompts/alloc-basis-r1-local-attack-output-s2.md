Y2.A.Serial.1: No  
This would be refuted by: a write request that passes D28's available check but fails D16's allocatable check despite df reporting sufficient space.  
Y2.A.Serial.2: No  
This would be refuted by: a deletion followed by 3 user-visible publishes where the freed space remains unreclaimable and the write fails.  
Y2.A.Merged.1: Yes  
This would be refuted by: df reporting sufficient space but the write failing due to insufficient allocatable space after merging.  
Y2.A.Merged.2: Yes  
This would be refuted by: a deletion followed by 3 user-visible publishes where the freed space is not available for allocation.  
Y2.B.Serial.1: No  
This would be refuted by: a write request that passes D28's available check but fails D16's allocatable check despite df reporting sufficient space.  
Y2.B.Serial.2: No  
This would be refuted by: a deletion followed by 3 user-visible publishes where the freed space remains unreclaimable and the write fails.  
Y2.B.Merged.1: Yes  
This would be refuted by: df reporting sufficient space but the write failing due to insufficient allocatable space after merging.  
Y2.B.Merged.2: No  
This would be refuted by: a deletion followed by 3 user-visible publishes where the freed space remains unreclaimable and the write fails.  
Y2.C.Serial.1: No  
This would be refuted by: a write request that passes D28's available check but fails D16's allocatable check despite df reporting sufficient space.  
Y2.C.Serial.2: No  
This would be refuted by: a deletion followed by 3 user-visible publishes where the freed space remains unreclaimable and the write fails.  
Y2.C.Merged.1: Yes  
This would be refuted by: df reporting sufficient space but the write failing due to insufficient allocatable space after merging.  
Y2.C.Merged.2: Yes  
This would be refuted by: a deletion followed by 3 user-visible publishes where the freed space is not available for allocation.  
Y5.A.Scenario1: I-3.1 violated and I-5.2 violated  
This would be refuted by: allocated statistic matching the sum of references from the rollback candidate set and free plus allocated equaling capacity.  
Y5.A.Scenario2: I-3.1 violated and I-5.2 violated  
This would be refuted by: allocated statistic matching the sum of references from the rollback candidate set and free plus allocated equaling capacity.  
Y5.B.Scenario1: I-3.1 holds and I-5.2 holds  
This would be refuted by: allocated statistic not matching the sum of references from the latest valid root or free plus allocated not equaling capacity.  
Y5.B.Scenario2: I-3.1 holds and I-5.2 holds  
This would be refuted by: allocated statistic not matching the sum of references from the latest valid root or free plus allocated not equaling capacity.  
Y5.C.Scenario1: I-3.1 violated and I-5.2 violated  
This would be refuted by: allocated statistic matching the sum of references from all self-verifying roots and free plus allocated equaling capacity.  
Y5.C.Scenario2: I-3.1 violated and I-5.2 violated  
This would be refuted by: allocated statistic matching the sum of references from all self-verifying roots and free plus allocated equaling capacity.
