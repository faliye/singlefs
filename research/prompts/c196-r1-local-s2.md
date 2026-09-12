1. Rollback sequence: publish 5 generations, roll back to generation 3, then dereference a block only in generation 2.  
   - A: Reader follows map root from tree table entry in generation 3 tree table. Entry exists.  
   - B: Reader follows map root from root record's third field (generation 3 value). Entry exists.  
   - C: Reader follows map root from superblock (generation 5 value). Block not found. Entry missing.  

2. Publication order:  
   - A: Map root pointer durable during units step (tree table unit written). No fourth step.  
   - B: Map root pointer durable during root slot step. No fourth step.  
   - C: Map root pointer durable during superblock write step. Superblock write outside publication order requires fourth step.  

3. Mount chain:  
   - A:  
     - Root record to tree table pointer: authoritative (bootstrap exemption)  
     - Tree table unit to map tree id 5 root pointer: authoritative (tree kind 5 rule)  
   - B:  
     - Root record to map pointer field: authoritative (bootstrap exemption)  
   - Tree table entry root pointer is authoritative for map (id 5) and hint for others. Tree kind 2 bytes sufficient.  

4. Tree table rules for map under A:  
   - Old reader skips unknown kinds: Reader without map entry if unknown tree kind 5 → filesystem unmountable. Breaks rule.  
   - Every snapshot is one entry: Map not a snapshot → okay.  
   - Clone writes watermark row: Map pool-wide → no watermark needed → okay.  
   - Destroy head deletes by prefix: Map not head-specific → okay.  
   - Previous_snapshot_txg per head: Map has 0 → okay.  
   - Enumerating live snapshots: Map not a snapshot → skipped → okay.  

5. Tree id under B and C:  
   - Map class 2 node headers carry tree id 5. Watermark issues it. Tree id 0 (instance table) distinct → no collision.  

6. Missing candidates:  
   - Store map pointer in instance table unit as pool-wide field. Cost: 83 bytes added to instance table unit. New rule: instance table unit contains map pointer. Mixes per-instance data with pool-wide metadata.  
   - Store map pointer in allocation record tree data. Cost: 83 bytes in allocation record tree. New rule: allocation record tree contains map pointer. Mixes allocation data with map metadata.
