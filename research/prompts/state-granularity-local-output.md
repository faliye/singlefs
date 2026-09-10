Consider a field named "last_access_timestamp" in an index tree node. This field records the last time the node was accessed for optimization purposes but is not part of the filesystem's data integrity or structure. Authoritative state (units, accounting, roots) does not include historical access times, so this field cannot be recomputed from authoritative state alone.  

Under the candidate's field-granularity rule:  
- The "last_access_timestamp" field is present but not in the recomputation list.  
- The candidate would declare the entire node structure invalid for derived state because not all fields are recomputable.  
- Thus, the statement "this whole structure can be rebuilt" would be false for the node.  

However, at node granularity:  
- The node's essential fields (keys, pointers, checksums) *can* be recomputed from authoritative state.  
- The "last_access_timestamp" is a non-essential cache; losing it does not prevent rebuilding the node's functional structure.  
- The node remains valid derived state because the critical data for reconstruction is recoverable.  

This contradicts the candidate's requirement. The project owner's decision (G1) that the allocation-record tree can be rebuilt from authoritative state would be overturned if the allocation-record tree included such a field, but the decision was correct because non-essential fields do not affect rebuildability.  

"Field" is not well-defined for composite values. For example, an interval field (start, end) would require recursion into its components. The candidate does not specify where recursion stops, leading to ambiguity.  

A concrete observation falsifying the candidate: If a filesystem node contains a "last_access_timestamp" field and is lost, a full-disk scan rebuilds the index tree correctly (keys, pointers, checksums verify), with the timestamp missing but no data corruption. The system operates normally, proving the structure is rebuildable despite the non-recomputable field. The candidate incorrectly flags this as an error.
