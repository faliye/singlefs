You are reviewing a design for a copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled and not open for debate. Every storage unit is immutable: changing one byte means writing a new unit. Each unit carries a fixed length plaintext header naming exactly one birth owner, and nothing else about who refers to it. A pointer inside a tree holds the physical location of the unit it points at, and that pointer is the authoritative record of where the unit lives. Snapshots are multiple writable heads, each with its own tree. Reflink is a day one promise and shares whole units. Authoritative state is exactly three things: units, accounting, and roots; everything else is derived and must be rebuildable, and losing a derived thing may only cost time, never data. Background defragmentation is resident and automatic and moves units to new physical locations. Creating a snapshot must be a bounded, cheap operation.

Three candidate designs for recording who refers to a shared unit:

Path one. Give every unit two integer labels, a preorder and a postorder number, so that asking whether any live reference remains inside a subtree becomes an interval containment test rather than a traversal.

Path two. Stop treating the pointer as the authoritative location. Put the authoritative location in one indirection layer keyed by physical slot, so that moving a unit touches exactly one entry and nobody ever needs to know who refers to it.

Path three. Record each sharing relationship as its own unit, with the same durability and crash recovery as any other unit, discoverable by a linear scan.

Your task:

1. Attack path one. Construct an insertion sequence that forces the preorder and postorder labels to be re-assigned, and say how many already published immutable units must be rewritten as a result. If the labels are stored somewhere other than the units, say where, and what that does to the requirement that the thing be rebuildable.

2. Attack path two. A reader following a pointer now finds only a hint, and must consult the indirection layer for the true location. Construct a history involving a snapshot read after a defragmentation move where the reader silently gets the wrong version of the data rather than an error. State the smallest extra field that would make it detectable.

3. Attack path three. Work out how many new units must be written at the moment a snapshot of a large tree is created, if every sharing relationship is its own unit. Give the arithmetic in terms of the size of the data set, and say whether that is compatible with snapshot creation being a bounded, cheap operation.

For each item give the history as numbered steps, say which rule it breaks, and give the smallest change that would close it. If you cannot construct one, say so and say exactly which step blocked you.
