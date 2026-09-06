You are reviewing a design proposal for a copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled in this project. Space accounting lives in its own btree. Its key is a tuple of four parts: a statistic tag, a tree id, a device id, and a generation number, where the generation is the checkpoint number. The value is the complete current value of that statistic for that key, written idempotently on every publish, deduplicated by a per entry sequence number, merged through a write buffer front end. There are fourteen statistics. Twelve of them do not use the tree id part at all: bytes allocated, bytes free, bytes unreclaimable, bytes held by unfinished delete intents, bytes queued for deferred release, bytes of committed reservation, extension point quota used, a fragmentation run count, a count of fully empty cluster segments, the next free inode number, a sweep watermark, and the generation of the most recent root destruction. Only two use it: bytes exclusive to one tree and bytes shared by one tree. Their stated consumer is snapshot space usage. Snapshots are supported and there is no upper bound on how many may exist; every writable head also has its own tree.

The proposal is to stop maintaining those two per tree statistics at commit time, and instead compute them on demand in the background when someone asks how much space a snapshot uses. The tree id part would then be dropped from the accounting key entirely, making every entry eight bytes narrower and making the number of entries independent of the number of snapshots.

Admission control for new writes computes, before letting an operation in, a worst case of how much space that operation could need, and refuses it if the pool cannot cover it. A rule in this project says that runtime decision paths may not use whole pool traversal, because the answer must exist at the moment it is asked, and separately because if the runtime computed by traversal then the checker's traversal would be the same computation and the cross check would be worth nothing.

Your task:

1. Construct a concrete situation in which some decision the filesystem must make at run time genuinely needs to know how much space one particular snapshot occupies, or how much is shared between two snapshots. Say who asks, what they do with the answer, and what goes wrong if the answer takes a background scan to produce.

2. Attack the on demand computation itself. Given that blocks are shared between snapshots and that a block's liveness depends on which roots still reference it, say what a background pass would have to traverse to answer how much one snapshot exclusively occupies, and whether that pass can produce a correct answer while writes are still happening.

3. Suppose the answer to item 1 is that nothing needs it at run time, and it is only ever displayed to a human. Name what is lost by making it a background computation, and say whether anything in a filesystem breaks if that number is slightly stale or occasionally unavailable.

For each item give the reasoning and what would change your answer. If you cannot construct an answer, say so and say which step blocked you.
