SINGLEFS SECOND TRANSACTION MILESTONE: WARM-UP COUNTERS, ACCOUNTING-TREE ADMISSION, AND BIRTH SEQUENCE ARITHMETIC - LOCAL ATTACK LEG - ROUND m2-wave1-code-r1

You are one of several independent reviewers checking arithmetic in a design for singlefs, a copy-on-write filesystem being designed from scratch. Below are three self-contained sets of facts about three separate, unrelated small mechanisms. Do not assume anything not stated here. Do not invent any history, scenario, or publish sequence beyond what each section gives you. Answer only the three question sets given at the end (W1, W2, W3); do not discuss anything else about this filesystem.

Read carefully: every number and every qualifier below is load-bearing. Do not use any markdown emphasis (no bold text, no italics, no asterisk bullets, no backtick code formatting) anywhere in your answer. Number your answers to match the answer labels given at the end of each question set. Where a question asks for a table, give explicit numbers for every column; do not answer with just "yes" or "no". For every numbered answer, add one sentence starting with "This would be refuted by:" describing the specific observation that would prove that answer wrong.

Do not use any code line numbers or file line numbers anywhere in your answer. You have not been read any source file in this prompt, so any line number you might state would be one you invented yourself, not one you looked up. If you want to point to where a rule came from, point to it by name instead (for example "the warm-up formula in Section 2" or "row W2.devices79").

Section 1: shared vocabulary.

A "publish" is one commit of the filesystem. Every publish has a checkpoint transaction generation number, called checkpoint_txg (or txg for short). Every publish also writes one journal record, and that journal record carries a counter called jsn (journal sequence number). The filesystem's superblock has an 8-byte field called tail, which some publishes update.

Section 2: warm-up empty publishes.

There is a fixed constant used only in the first version of this filesystem, called WARM_UP_EMPTY_PUBLISHES, whose value is 2.

A procedure called warm-up takes one input, called the starting record number: the jsn counter value of the newest journal record already present in the journal ring before warm-up begins. Right after making a fresh filesystem, with an empty ring, this starting record number is conventionally 0, but warm-up can in principle be given any starting record number as its input.

Warm-up performs exactly WARM_UP_EMPTY_PUBLISHES empty publishes (that is, exactly 2 empty publishes), one after another, in a loop. The loop has a loop variable, called the publish index, which runs from 1 up to and including WARM_UP_EMPTY_PUBLISHES; so with WARM_UP_EMPTY_PUBLISHES equal to 2, the loop runs once with publish index 1, and then once more with publish index 2, and then stops.

For the empty publish at a given publish index, its checkpoint_txg equals the publish index itself. So the first empty publish in the loop (publish index 1) has checkpoint_txg 1, and the second empty publish in the loop (publish index 2) has checkpoint_txg 2. This checkpoint_txg value does not depend on the starting record number at all; it depends only on the publish index.

Before the loop starts, a running variable called the previous counter is set equal to the starting record number.

For each empty publish in the loop, in order: that publish's own jsn counter value equals the previous counter's current value, plus 1. Immediately after that publish happens, the running previous counter variable is updated to equal that publish's own jsn counter value (so if there is a next publish in the loop, it will add 1 to this newly updated previous counter, not to the original starting record number).

Every empty publish also writes the superblock's tail field. The tail value written by a given empty publish equals that same publish's own jsn counter value exactly (the identical number as its jsn counter, not a separately computed number).

Decision text D16 item 8 (quoted): "Before the new instance's own root -- once written and FUA-acknowledged -- covers both devices, no fsync may return and no rollback is confirmed to the administrator; the mechanism is pushing empty publishes back to back, at most R = 3 times under the first version's geometry." A follow-up note attached to the same decision item states: "The first version actually performs this 2 times, and it is a format constant, not a runtime predicate: in the first version, the device ownership of the root ring's three regions is fixed as 0, 1, 0, so checkpoint_txg 1 lands in region 1 (device 1) and checkpoint_txg 2 lands in region 2 (device 0); two times exactly covers both devices, so the count is uniquely determined by that ownership. This is registered as the format constant WARM_UP_EMPTY_PUBLISHES = 2."

Decision text D23 item 18 (quoted, the part about the tail field's encoding): "The superblock's 8-byte tail field stores jsn's 48-bit counter; it gets its redundancy from the superblock's own two-slot rotation, and no separate tail slot is set up. Record n lands at ring offset (counter minus 1) mod slot-count, times 4096."

Decision text D23 item 19, part 1 (quoted): "Transaction number 0 is reserved for records that carry no transaction (empty-publish records); the commit flag on such a record is written as 1; such a record does not enter the maximum used for W in the instance table, and it does not enter the judgment of whether a tail was dropped because a commit flag never showed up."

Question set W1. For each of three starting record numbers: 0, 1, and 40, and for each of the two empty publishes in warm-up (publish index 1, then publish index 2), give three numbers: the checkpoint_txg written by that publish, the jsn counter value written by that publish, and the superblock tail value written by that publish.

Give six answers: W1.start0.pub1, W1.start0.pub2, W1.start1.pub1, W1.start1.pub2, W1.start40.pub1, W1.start40.pub2. Each answer must give all three numbers (checkpoint_txg, jsn counter, tail), clearly labeled.

For each of these six answers, add one further sentence stating whether your three numbers are consistent with, or contradict, the quoted D16 item 8 and D23 item 18 text above. Name specifically which clause you are checking against (for example "checkpoint_txg walking 1 then 2 regardless of the starting record number" or "tail stores the identical value as jsn's own counter").

Section 3: accounting-tree row count and admission.

Every publish writes a fixed number of rows into a structure called the accounting tree. This number depends only on how many devices the storage pool has, called the device count below.

The formula is: accounting row count = 3, plus 6 times the device count.

The 3 fixed rows do not depend on the device count at all. In order, they are: (i) the next available inode number, a high-water mark; (ii) a count of bytes that are occupied by objects marked for deletion whose deletion has not yet completed; and (iii) an amount of space already committed as a reservation.

The 6 rows that get repeated once per device are, in order: allocated bytes, free bytes, non-reclaimable bytes, defer-queue bytes pending release, a count of fragment runs, and a count of fully-empty clustered segments.

Separately, there is a fixed-size on-disk structure called an index node, whose total size is 16384 bytes (16 KiB). An index node has a header, and the remainder of the node is filled with fixed-width entries. The header's size depends on something called the key width for that particular tree. The header-size formula is: header size = 86, plus 2 times the key width, plus 29.

For the accounting tree specifically, the key width is 22 bytes, so the header size is 86 + 2 times 22, plus 29, which is 159 bytes. This 159 does not depend on the device count and is the same for every row of the table below.

Each accounting-tree entry (one row of the accounting tree) has a fixed width: 22 (the key width) + 8 + 4 = 34 bytes. This 34 also does not depend on the device count.

The maximum number of fixed-width accounting entries that fit into one 16384-byte index node is computed as: (16384 minus the header size) divided by the entry width, using integer division that always rounds down toward zero (discarding any remainder), never rounding up.

The admission rule: when a publish is about to happen, the system computes the accounting row count for that publish, from the device count, using the formula above. If that row count is strictly greater than the maximum number of entries that fit in one node (computed above), the publish is rejected before anything is actually allocated, with a specific error named AccountingEntriesExceedOneNode. If the row count is less than or equal to that maximum, this particular check passes (other, unrelated checks may still apply afterward, but this specific check passes).

Decision text D5 item 8 (quoted, an anchor number given only for a 2-device pool): "The first transaction writes 15 rows" into the accounting tree.

Question set W2. For each of five device counts: 1, 2, 78, 79, and 80, give three items: (a) the accounting row count, computed from the formula above using that device count; (b) the maximum number of entries that fit in one 16384-byte node (state this number every time, even though it does not change across the five device counts); and (c) the admission verdict, which must be exactly one of two words: "passes" or "AccountingEntriesExceedOneNode", depending on whether (a) is strictly greater than (b).

Give five answers: W2.devices1, W2.devices2, W2.devices78, W2.devices79, W2.devices80. Each answer must give all three items (a), (b), (c), clearly labeled.

For each of these five answers, add one further sentence about whether your row-count number (a) is consistent with, or contradicts, the quoted D5 item 8 anchor. That anchor only gives a directly checkable number for the devices2 row (15 rows for a 2-device pool); for the other four device counts (1, 78, 79, 80), say explicitly that the quoted decision text gives no directly checkable number for that row, and that your answer instead relies only on the formula given in this section.

Section 4: birth sequence numbers.

Every unit of a certain kind (called a code-2 or code-3 unit) that gets written out during a publish is tagged with a small number called its birth sequence number.

Birth sequence numbers are handed out by something called a birth sequence allocator. This allocator keeps an internal table mapping a triple of (tree identifier, checkpoint txg, instance generation) to a counter value; the starting counter value for a triple that has never been seen before is 0.

Each time the allocator's "next" operation is called with a specific triple: it looks up (or creates, starting at 0, if this is the first time this exact triple has been seen) the counter for that exact triple; it remembers that counter's current value as the number to hand out this time; and then it immediately increases the stored counter for that same triple by 1, for next time. So: the first time "next" is called for a brand-new triple, it returns 0; the second time it is called for that exact same triple, it returns 1; the third time, 2; the fourth time, 3; and so on, one higher each time, for that same triple only.

Critically: a brand-new, completely empty birth sequence allocator, with no triples recorded in its table yet, is created from scratch at the very beginning of handling each individual publish. Nothing carries over from one publish's allocator to the next publish's allocator; the next publish always starts with a totally empty table, no matter what triples or counter values existed in the previous publish's allocator.

Decision text D19 item 9 (quoted): "The birth sequence starts at 0; within the same tree, it increases by 1 every time a code-2 or code-3 unit is written out (code-2 and code-3 share one counter); the scope resets to zero when moving to the next checkpoint; if the same node is rewritten a second time by a fixed point within the same checkpoint, it gets a new number."

Question set W3.

(a) Suppose that, during the handling of one single publish, the "next" operation gets called four times in a row, every single time with the exact same triple (the same tree identifier, the same checkpoint txg, and the same instance generation) as its argument. List the four numbers returned, in the exact order they are returned, as four separate numbers.

(b) Suppose that publish finishes being handled, and then a second, separate publish begins being handled, so a brand-new, empty birth sequence allocator is created for it, as stated above. The very first time the "next" operation is called during this second publish, no matter which triple is passed as its argument, what single number does it return?

Give two answers: W3.four_in_a_row (give all four numbers from part (a), in order) and W3.next_publish_first_value (give the single number from part (b)).

For each of these two answers, add one further sentence stating whether your numbers are consistent with, or contradict, the quoted D19 item 9 text above. For W3.four_in_a_row, name specifically the "starts at 0" clause. For W3.next_publish_first_value, name specifically the "the scope resets to zero when moving to the next checkpoint" clause.

Final instructions. Answer W1 (six answers), then W2 (five answers), then W3 (two answers), in that order, thirteen answers total. Use the exact answer labels given above (for example "W1.start0.pub1"). Do not renumber or relabel them. Do not add any additional commentary, summary, or recommendation beyond the answers and their required "This would be refuted by:" and consistency sentences.
