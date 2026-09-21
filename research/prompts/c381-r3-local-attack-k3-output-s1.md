1. P1: specific device offset of the unit being written  
P2: no specific device offset (barrier performs no write of bytes to any offset per fact 2)  
P3: device 0's journal record offset  
P4: device 1's journal record offset  
P5: no specific device offset (barrier performs no write of bytes to any offset per fact 2)  
P6: specific device offset for root slot determined by txg mod 3 and slot number per fact 3  
P7: device 0's system config slot fixed offset per fact 10  
P8: device 1's system config slot fixed offset per fact 10  

2. P1: undetermined (no fact specifies previous content of unit blocks)  
P2: not applicable  
P3: previous journal record data (journal ring is a ring buffer where slots are reused per fact 9)  
P4: previous journal record data (journal ring is a ring buffer where slots are reused per fact 9)  
P5: not applicable  
P6: previous root record data (root ring slots are reused for different checkpoints per fact 3)  
P7: previous system config data for device 0 per fact 10  
P8: previous system config data for device 1 per fact 10  

3. P1: unsafe. Probe write would overwrite the allocated block which may contain data still referenced by the filesystem. This would be refuted by: the block being a fresh allocation with no prior data and free blocks being zeroed before reuse.  
P2: safe. Barriers have no target offset so no data is written. This would be refuted by: the barrier failure actually involving a specific offset not stated in facts.  
P3: unsafe. Overwriting journal slot may destroy data required for replay. This would be refuted by: the journal slot being written to being outside the replay window (older than 65536 records).  
P4: unsafe. Overwriting journal slot may destroy data required for replay. This would be refuted by: the journal slot being written to being outside the replay window (older than 65536 records).  
P5: safe. Barriers have no target offset so no data is written. This would be refuted by: the barrier failure actually involving a specific offset not stated in facts.  
P6: unsafe. Overwriting root slot may destroy previous root record needed for recovery. This would be refuted by: the root slot's previous data being no longer referenced by any valid checkpoint.  
P7: unsafe. Overwriting system config slot may destroy previous config data required for recovery. This would be refuted by: the system config slot's previous data not being needed for current system state.  
P8: unsafe. Overwriting system config slot may destroy previous config data required for recovery. This would be refuted by: the system config slot's previous data not being needed for current system state.
