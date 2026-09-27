1  
(a) Disk 0: corrupt 1 slot (newest root record with nonzero F). Disk 1: corrupt 1 slot (newest root record with nonzero F). Total: 2.  
(b) Disk 0: 16 slots (all root slots, as all have nonzero F). Disk 1: 8 slots (all root slots, as all have nonzero F). Total: 24.  
(c) Part (a)'s total answers the question. The "least number that must be corrupted" refers to the minimal sufficient number in a favorable scenario, not the worst-case forced number.  
This would be refuted by misreading the modeling assumption's favorable case to include multiple nonzero F slots on a disk when it states only the newest root has nonzero F and the next older has 0.  

2  
Design MAX+HOLD's F effective formula is textually identical to design MAX's, as fact 5 states "for the purpose of computing F effective, this design is identical to design MAX".  
(a) Disk 0: 1 slot. Disk 1: 1 slot. Total: 2.  
(b) Disk 0: 16 slots. Disk 1: 8 slots. Total: 24.  
(c) Part (a)'s total answers the question.  
The slot holding behavior does not affect F sub d calculation because F sub d depends solely on the newest valid root record, regardless of slot holding. Thus, numeric answers are identical to item 1.  
This would be refuted by misreading fact 5 to believe MAX+HOLD has a different F effective formula or misinterpreting slot holding as affecting F sub d.  

3  
(a) Disk 0: 1 slot. Disk 1: 1 slot. Total: 2. Disk 0: 16 slots. Disk 1: 8 slots. Total: 24. Part (a)'s total answers the question.  
(b) Favorable case: disk 0 corrupt 1 system config slot, disk 1 corrupt 1 system config slot. Total: 2. Unfavorable case: disk 0 corrupt 2 system config slots, disk 1 corrupt 2 system config slots. Total: 4.  
(c) Smallest combined total: 4 (root part 2 + config part 2). Largest combined total: 28 (root part 24 + config part 4).  
This would be refuted by misreading fact 6's system config slot count or modeling assumption's favorable case description for system config slots.
