Question 1  
G8 does not violate Quote 1's second prohibition because the checker and runtime implement the predicate in separate code paths. The runtime uses the predicate during block reclamation operations, while the checker computes it independently during audit by walking the allocation-record tree. The predicate's logical definition is shared, but the code implementing it is distinct: the runtime's version is embedded in allocation/reclamation logic, and the checker's version is part of the audit program's traversal code. Quote 1 forbids "the same piece of code," not identical logic, and G8's checker uses its own standalone implementation.  

What would refute this: If the checker's code imports and directly uses the runtime's function for the reclaim-threshold calculation, such as calling `runtime.get_threshold()` in the checker's code.  

Question 2  
G8 is permitted because it uses two independent implementations of the predicate, not shared code. The decision explicitly allows separate implementations for the same logical definition (e.g., checksums or format parsing), as long as each side writes its own code. The threshold formula is not a scalar constant (it involves branches and arithmetic), but the decision permits independent implementations of non-constant logic like format parsing. G8's checker and runtime each write their own code for `max(F_effective, oldest_valid_root)`, satisfying the "format parsing" exception clause where each side independently implements the logic.  

What would refute this: If the checker's code and runtime's code both reference the same compiled function or shared module for the threshold calculation, such as a single C file compiled into both binaries.  

Question 3  
I-3.1 cannot catch a wrong reclaim-threshold error because it sums allocated bytes from disk, which is unaffected by defer-state bookkeeping errors. I-7.4 cannot catch it because it checks block reassignment of active references (defer blocks are unreferenced). I-4.8 cannot catch it because it verifies checksums along root-linked trees (defer blocks are not part of these trees). All three checks focus on physical disk consistency or referenced data integrity, not the arithmetic count of defer-state blocks.  

What would refute this: If I-3.1's disk walk included defer-state blocks in its allocated count, but the project defines allocated bytes as only active references, excluding defer blocks.  

Question 4  
No such check can be constructed because the necessary quantity "unreclaimable" is not tracked as an independently maintained statistic. The alternative formula requires capacity = allocated + free + unreclaimable + defer, but unreclaimable blocks are not measured or stored separately in the project's statistics. Without an independent source for unreclaimable (e.g., a statistic tracked via a separate algorithm), the equation cannot be verified without relying on runtime logic that might share the same threshold bug.  

What would refute this: If the project already tracks "unreclaimable" as a distinct statistic with its own independent maintenance logic, not derived from the reclaim-threshold predicate.
