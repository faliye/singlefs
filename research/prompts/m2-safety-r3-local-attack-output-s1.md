Question 1. Rule T's available(d) on each disk is 21 slots, and demand(d) is 6 slots. This would be wrong if the av and d fields in fact A1 did not match the values stated in fact A4.

Question 2. Rule P2's available(d) per disk is 53.5 slots (computed by adding the pool-wide deferred-pending-release of 65 slots divided by R=2 to rule T's available(d) of 21), and demand(d) is 14 slots (sum of all roles rewritten, including both Demand and CheckpointReservePool roles). This would be wrong if the total deferred-pending-release was not 65 slots or R was not 2.

Question 3. Cannot be determined from the given facts. This would be wrong if the number of non-empty valid roots in the root ring immediately before the write in fact A1 was known to be sufficient to determine whether pushing 8 empty publishes would evict a valid root.

Question 4.  
20 | 1 | 1  
50 | 8 (26 slots short) | 1  
100 | 8 (76 slots short) | 8 (39 slots short)  
This would be wrong if the reclaimed slots per push for rule P1 or P3 did not match facts B3 and B4.

Question 5. The 20/50/100 numbers represent the deficit (demand(d) minus available(d)) needed for admission. To compute the real demand(d) for the B3/B4 scenario, we would need a fact stating the available(d) and demand(d) for that specific write, such as "E4 240 arm=T delete_write available=80 demand=100". This would be wrong if the deficit was not defined as demand(d) minus available(d) for the pending write.
