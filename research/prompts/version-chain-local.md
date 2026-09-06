You are reviewing a design decision for a copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled in this project. Every unit is immutable; a write allocates a new unit and the tree pointer is repointed. A pointer to a child carries a checksum over the whole child unit and the child's birth checkpoint number. A ring of the last three roots is kept on disk. Snapshots exist and are the supported way to read an older state; each snapshot has its own root. A deadlist records which blocks are referenced only by snapshots. Every unit header carries its own birth checkpoint number and a write sequence. There is a rejection list in this project that already rejected storing a pointer from a unit to its own previous version, for three reasons: it overlaps with the birth checkpoint plus deadlist mechanism; a previous-version pointer becomes misleading once that block is reused; and it needs a new invariant to guard the hazard it creates itself.

The proposal now under review would add exactly that: each unit's extension area holds a stack of up to five pointers to its previous versions, each entry carrying the target's birth checkpoint so a reader can detect expiry, and the pointers explicitly do not count as references, so garbage collection ignores them.

The claim being tested is that this buys nothing: the only consumer in this filesystem that asks whether something is a previous version is stale detection, and stale detection is already answered by the parent pointer's checksum and birth number; fetching an older state is done through snapshots.

Your task:

1. Name a consumer inside a filesystem that needs to reach the previous version of one specific unit, and that cannot be served either by walking one of the three retained old roots or by a snapshot. Be concrete about who asks, when, and what they do with the answer. If you cannot name one, say so and list what you considered.

2. Attack the claim that the parent pointer's checksum plus birth number already detects staleness. Construct a history in which the parent pointer itself is stale, so that parent and child are mutually consistent and both checks pass, and say what does detect it in that case.

3. Consider repair. When a unit fails its checksum and redundancy rebuild also fails, is falling back to a previous version a sensible recovery path for a filesystem, or does it produce a state the rest of the system cannot reason about? Answer for a system where the tree, the accounting and the free space records were all updated on the assumption that the new version is the live one.

For each item give the reasoning, the rule text it breaks if any, and what would change your answer. If you cannot construct an answer, say so and say which step blocked you.
