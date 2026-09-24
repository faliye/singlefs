Judgment 1 verdict: the instance number invariant becomes not applicable and fact 7's stated reason is unsound  
Justification: fact 7 states the instance number invariant reports not applicable when per-device computation yields nothing, which occurs for a device with invalid system configuration slots (fact 8), and fact 6 excludes such devices from its filesystem identifier check list, meaning fact 6 never checks membership for invalid devices as fact 7 claims.  
This would be refuted by: evidence showing fact 6 processes devices without valid system configurations for membership determination.  

Judgment 2 verdict: both candidate images make the checker's early exit test and choose_system_configuration disagree about whether the pool can be mounted  
Justification: for candidate image A and B, choose_system_configuration returns second failure kind due to disagreement (facts 3 4 5), but the checker's early exit test does not trigger since valid configurations exist (fact 1), so the checker proceeds while recovery fails to mount.  
This would be refuted by: evidence that choose_system_configuration returns success for either candidate image.  

Judgment 3 verdict: matches word for word  
Justification: fact 10 implements green when either at least one strictly lower generation exists or all records are zero, exactly matching fact 9's "at least one record carrying a strictly earlier generation number must still exist" and "when every self certifying root record present is generation zero, judge it green".  
This would be refuted by: fact 10 adding or omitting any condition from fact 9's exception clause.  

Judgment 4 verdict: correctly falls under only the first rule  
Justification: fact 12 sets expected back chain to zero when previous record has different instance number, which aligns with fact 11's first rule that the first record of an instance always has back chain zero, and the second rule about not judging across boundaries does not apply to this specific case's handling.  
This would be refuted by: evidence that the implementation treats this case as involving the second rule or both rules.  

Judgment 5 verdict: the implementation's merge key omits a field that facts 15 and 16 count as belonging to the class identity segment  
Justification: facts 15 and 16 explicitly list payload CRC and payload checksum as part of their respective class identity segments, but fact 14 removes these fields from the merge key despite fact 13 requiring all class identity segment fields.  
This would be refuted by: evidence that facts 15 and 16 do not include payload related fields in the class identity segment.
