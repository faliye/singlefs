What a copy-on-write file system should do when the commit fixed point cannot get the blocks it needs, and whether rewriting freed allocation records makes the reserve estimate unbounded.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. Every publish writes a fixed point: new copies of the allocation record tree, the central mapping tree, the accounting tree and the tree table unit; each new block needs an allocation record, and allocating it may dirty more nodes. Each metadata block has one copy on each of two disks.
F2. A reserve for that fixed point is subtracted at admission: ckpt_cost = sum of record tree heights + accounting nodes per publish, recomputed at each publish. Rounds one and two found it undercounts: the tree table unit, one path per disk in the allocation record tree, and a release chain.
F3. The release chain: when a block is freed, its allocation record is not deleted but rewritten as freed with a free generation, in the same publish as the copy-on-write that frees it. The old placement's record sits in the leaf where that block was placed long ago, so rewriting it dirties an old leaf, whose own old placement sits in an even older leaf, and so on until an already dirty leaf is reached.
F4. Space that df reports as free must be writable; there must be no false ENOSPC. Deleted space becomes usable within a bounded number of publishes.
F5. Rollback promise: the most recent 4 states that changed user-visible data can be rolled back to; freed blocks become reusable only after their free generation passes a floor tied to those states.
F6. When admission is short, the file system may first push empty publishes to raise the floor, at most 8 in one admission, before reporting ENOSPC.

Proposed change (b): accept that admission cannot compute a tight bound; define what happens when the fixed point fails to allocate.

Questions:
1. Is the release chain in F3 unavoidable under F3's wording, or can the implementation sort all new and rewritten records by key in the first round of the fixed point and merge them into the fewest leaves? If it can, is the chain then bounded?
2. For each way to handle a failed fixed-point allocation, say whether it breaks correctness (the last acknowledged version not read back after a crash), F4, F5, or causes a publish that can never complete: borrow from the reserve, borrow freed blocks whose free generation has not passed the floor, push an empty publish and retry, switch the file system to read-only.
3. Which way would you pick, and what single observation would change your pick? Do not decide by bytes saved.
