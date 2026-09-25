Item 1  
Row 1. 50432  
Row 2. 53  
Row 3. 0  
Row 4. 0  
Row 5. rows0=2, pages_of_chain=1, chain_rewrite=2, warm_up=15, one_switch=17  
Row 6. 0  
Row 7. 0  
Row 8. 0  
Row 9. allocation-record tree height=2, central-mapping tree height=1, accounting tree node count=1, tree table=1  
Row 10. 5  
Row 11. 50306  
Row 12. 2 (user data unit) + 1 (extent-tree node below root) + 1 (extent-tree root) + 2 (inode leaf container) + 1 (inode-tree root) = 7  
Row 13. yes, by 50299 slots  
This would be refuted by allocated(d) = 50400 slots, which would make available(d) = -41, less than demand(d) of 7.  

Item 2  
Row 1. 50560  
Row 2. 113  
Row 3. 0  
Row 4. 0  
Row 5. rows0=2, pages_of_chain=1, chain_rewrite=2, warm_up=15, one_switch=17  
Row 6. 0  
Row 7. 0  
Row 8. 0  
Row 9. allocation-record tree height=2, central-mapping tree height=1, accounting tree node count=1, tree table=1  
Row 10. 5  
Row 11. 50374  
Row 12. 0 (plain mount, no write)  
Row 13. yes, by 50374 slots  
This would be refuted by allocated(d) = 50560 slots, which would make available(d) = -186, less than demand(d) of 0.  

Item 3  
Row 1. 3  
Row 2. 1  
Row 3. 1  
Row 4. 1  
Row 5. 6  
Row 6. 98304  
Included allocation-record tree height, central-mapping tree height, accounting tree node count, and tree table; excluded instance-table chain because Fact 6 explicitly states it is not part of ckpt_cost and is handled separately as instance_switch_reserve.  
This would be refuted by including instance-table chain in ckpt_cost, which would make row 5 higher than 6.  

Item 4  
allocated(d) = 13 + 10N for N from 0 to 12.  
Row 3: fixed (Fact 3)  
Row 4: fixed (no deletions)  
Row 5: fixed (rows0=2 for each mount)  
Row 6: fixed (Fact 4)  
Row 7: fixed (Fact 3)  
Row 8: fixed (Fact 3)  
Row 9: fixed (ckpt_cost=5)  
Row 10: fixed (ckpt_cost=5)  
available(d) = 50474 - 10N  
Smallest N where refused: 5048  
This would be refuted by if the actual growth rate of allocated(d) is less than 10 per overwrite, such as 5 slots per overwrite, which would delay the refusal to N=10095.
