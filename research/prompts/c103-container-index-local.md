You are reviewing a design for a copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled and not open for debate. Small records are grouped into packed container units. A container is named by four fields: the tree it was born in, the packed record type, a container number allocated as one more than the current maximum, and a birth generation. Until today, a separate container index held the physical pointer of every packed container, and every other tree referred to containers by name rather than by physical pointer. One packed record type, the instance table, is the exception: its physical pointer is held directly by the root record. Today a central mapping was made the single entry point for dereferencing and for release decisions: every unit is looked up there by a thirty three byte identity made of a one byte kind tag, a tree id, an object id, an object birth generation, and an anchor offset. Also today, a stripe membership table was defined whose leaves are packed containers and which is ordered by physical slot.

The design under review. The separate container index is abolished. A packed container is treated as an ordinary object: its four naming fields are folded into the thirty three byte identity, with the packed record type folded into the kind tag byte, the container number placed in the object id field, and the anchor offset fixed at zero. Its physical pointer then lives in the central mapping like any other unit's.

Your task:

1. The central mapping is itself a tree made of units, and every unit is supposed to be found through the central mapping. Ask what holds the physical pointers of the central mapping's own nodes. Construct the shortest cycle you can, and say at which level it must be broken and by what.

2. The instance table's physical pointer is held by the root record, not by the central mapping. Construct a history where a reader follows the rule that the central mapping is the single entry point, looks up an instance table container there, finds nothing, and does the wrong thing. Say what the reader should have done instead and how it could know.

3. Rebuilding the central mapping after losing it must read each unit's own header to recover its identity. Ask whether a packed container's header carries all four of its naming fields. If any is missing, say which one and what the rebuild produces in its place.

For each item give the history as numbered steps, say which rule it breaks, and give the smallest change that would close it. If you cannot construct one, say so and say exactly which step blocked you.
