# Find a counterexample: fixed group rewrite versus round robin compaction

Answer in English. Do not use any markdown emphasis of any kind in your answer.

This is a counting model of index node aging in a from scratch copy on write filesystem.
No real filesystem exists yet. Everything below is the model, stated in full.

## The model

There are L equals 8192 objects, keyed 0 through 8191, and S equals 10240 physical slots,
so the device is 80 percent full. Initially object k lives in slot k, so the layout is perfectly
contiguous in key order.

The metric is runs, the number of physically contiguous stretches when you walk the objects in
key order. Perfect layout gives runs equals 1. Fully shredded gives runs equals 8192.
Lower runs is better.

Each checkpoint, D equals 64 objects are dirtied and must be rewritten copy on write to new
slots. Their old slots are freed but cannot be reused until the next checkpoint. Two workloads:

- uniform: the 64 dirty objects are drawn uniformly at random from all 8192.
- runs8: dirty objects are drawn as contiguous stretches of 8 consecutive keys, until 64 are
  collected.

Each arm is also given a fixed extra write budget per checkpoint, either 32, 128, or 256 extra
object writes. Every arm spends the same budget. The run is 2000 checkpoints, 5 seeds, and the
reported number is the median final runs.

## The arms

Policy compact. Spend the budget walking a round robin cursor over key space, rewriting that
many objects in key order, placed as one contiguous stretch.

Policy neighbor. Spend the budget extending each dirty stretch outward by one key on each side,
then two, and so on until the budget runs out, then place each resulting stretch contiguously.

Format group with parameter N. Key space is partitioned into fixed groups of N consecutive keys.
If any object in a group is dirty, the entire group is rewritten and placed as one contiguous
stretch, charging the clean members against the budget. Leftover budget goes to rewriting whole
untouched groups under a round robin cursor.

Format gensep. Objects are bucketed by how many checkpoints have passed since they were last
rewritten, into three buckets. Segments are typed by bucket, hot and cold never share a segment.
The budget is spent compacting only objects currently sitting in the hot bucket.

## The measured result

Format group won every cell. On runs8 it more than halved the best policy arm at all three
budgets: 1595 versus 5146 at budget 32, 904 versus 3922 at budget 128, 276 versus 1339 at
budget 256. On uniform it won but never halved: 6128 versus 7031, 3966 versus 5463, 3414 versus
4791. The winning group size was 8 or 32 on runs8 and 2 on uniform. Format gensep lost every
cell, its best result being 1343 against the policy arm's 1339.

## Your task

Your job is to break the claim that fixed group rewrite is genuinely better. Construct concrete
scenarios, with numbers, where the format group arm loses badly or where its advantage is an
artifact of how the comparison was set up. For each construction give the parameters, what you
expect each arm to do, and why.

Directions worth probing, not a closed list. You are expected to find things not on this list.

- The runs8 workload has stretch length 8 and the winning group size is 8. What happens when the
  stretch length and the group size are deliberately mismatched, for example stretch length 8
  with group size 5, or stretch length 100 with group size 8.
- Group boundaries are fixed at multiples of N while the workload stretches start at arbitrary
  offsets, so a stretch usually straddles two groups. Does that help or hurt, and at which N.
- Occupancy is 80 percent. What happens at 95 percent or 99 percent, where contiguous free space
  is scarce and a whole group may not fit anywhere.
- The group arm rewrites clean neighbours it was not asked to rewrite. Is there a workload where
  that is pure waste, and how much does it cost.
- A skewed workload where a small hot set is dirtied over and over. Does grouping amplify the
  write cost of the cold members that share a group with a hot object.
- The budget is charged per object write. Is there any way the group arm gets more useful work
  per unit of budget purely because it writes in smaller contiguous chunks than policy compact.

Finally, answer one separate question, even if you find no counterexample. In this model a group
is just the key range from N times i to N times i plus N, so the placer can compute a group from
a key with no stored field at all. Under what circumstances would a group need an identifier
actually stored on disk in the node header. Be concrete about what changes.
