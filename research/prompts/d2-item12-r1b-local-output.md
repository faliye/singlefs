1. Steps:  
1. Devices 1, 2, 3 exist.  
2. Create stripe S1 with w=3: members (1,0), (2,0), (3,0). Stripe record container for S1 is stored at (1,1).  
3. Device 1 fails.  
4. Allocator releases (2,0). Lookup for stripe record fails (container on failed device). Allocator assumes no stripe membership and reuses (2,0) for new data.  
5. (3,0) remains live as part of unrepaired S1. Reusing (2,0) overwrites S1 data.  
Rule broken: "after a member of a w at least 3 stripe is released, that grid and the whole stripe including parity may not be reused until repair completes."  
Smallest change: Store stripe record containers in a dedicated w=2 metadata stripe, not within data stripes.  

2. Steps:  
1. Devices 1, 2, 3.  
2. Create stripe S1 with w=3: members (1,0), (2,0), (3,0). Stripe birth checkpoint 10. Record stored at (1,1).  
3. Device 1 fails. Repair creates new stripe S2 with members (2,0), (3,0), (4,0). Due to bug, S2 birth checkpoint set to 10. Record stored at (2,1).  
4. Full index scan finds two class 3 containers for (2,0): S1 record (checkpoint 10) and S2 record (checkpoint 10). Scan arbitrarily picks S1 record.  
Rule broken: Rebuild must correctly identify current stripe for each location. Same checkpoint prevents disishing old/new records.  
Smallest change: Use strictly monotonically increasing checkpoint for new stripes; add stripe generation counter to records.  

3. Steps:  
1. Devices 1, 2, 3.  
2. Create stripe S with w=3: members (1,0), (2,0), (3,0). Stripe record container stored at (1,1).  
3. Devices 1 and 2 fail. Only (3,0) remains.  
4. Rebuild fails: stripe record container is lost (only on failed devices). No way to determine member table or parity.  
Rule broken: "After a member of a w at least 3 stripe is released, that grid and the whole stripe including parity may not be reused until repair completes." (Rebuild impossible due to missing record.)  
Smallest change: Store stripe record containers in a dedicated w=2 metadata stripe, not within data stripes.
