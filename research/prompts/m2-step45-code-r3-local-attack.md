SINGLEFS SECOND TRANSACTION MILESTONE: ROLLBACK SHADOW LEDGER AND ALLOCATION RECORD TREE GUARD

Background

Singlefs is a copy on write filesystem under design. It keeps a fixed size ring of root records on each device. Each root record names a checkpoint transaction generation number, called txg, and an instance generation number. An administrator can roll back the mount to an older root, called R old, chosen from a rollback candidate set. After a rollback, every root in the ring that is newer than R old and belongs to the instance line that got rolled back becomes an abandoned root. Abandoned roots stay physically present in the root ring, they are not erased, until the ring eventually rotates a new root into the same slot and overwrites them.

Definition, rollback floor and effective floor

Each root record carries a rollback floor value, written here as F. Raising F narrows the rollback candidate set by dropping roots whose txg is below the new F. Once every surviving device has a persisted root that carries the new F, the new F is said to take effect. The quantity effective floor is defined as the minimum, taken over all surviving devices, of the maximum rollback floor value F carried by that device's most recently persisted root. If one device's most recently persisted root still carries an older, lower F, effective floor stays at that lower value even though other devices already carry the new F.

Definition, candidate set

A root belongs to the rollback candidate set if and only if all three of the following hold: it is readable, it is valid according to the newest instance table (a root belonging to instance i is valid unless the instance table has a row for instance i, call it row (i, Ti, Wi), with that root's checkpoint_txg greater than Ti; a root whose instance has no row in the table at all is always valid), and its checkpoint_txg is greater than or equal to effective floor. A root that fails any of these three conditions is abandoned.

Definition, the shadow ledger mechanism, called G5 in what follows

The governing rule says that before an abandoned root leaves the root ring, the storage units it references must not be reallocated and must not have their headers erased. The mechanism that enforces this, call it G5, runs on every mount, and again every time the floor F is raised. It computes a set of storage slots to isolate, meaning marked so the allocator will never hand them out, as follows.

a. For every abandoned root whose allocation record ledger can be read, take every slot that ledger still shows as allocated, not released.
b. From that collection, remove every slot that is also still shown as allocated in any candidate root's own ledger. A root only gets credit for exempting the slots that its own ledger still shows as allocated; a slot that a root's ledger shows as already released does not count as referenced by that root.
c. From what remains, also remove every slot that the current, live version's own ledger still shows as allocated.
d. Whatever is left after steps a, b and c gets isolated.

If an abandoned root's ledger cannot be read, because its tree table or its allocation record tree is unreadable or malformed, that root is simply skipped. It is counted, as unreadable, the mount is not rejected, and none of that root's slots get isolated through this mechanism.
If a candidate root's ledger cannot be read, that candidate root is treated as if it exempted nothing at all. So on that kind of failure, isolation can only end up including more slots than it otherwise would, never fewer.
Because the isolation marking lives independently of the ordinary allocated or free bookkeeping, shrinking the candidate set, for example by raising F, and then recomputing G5 can only add newly isolated slots. Nothing that was already isolated ever needs to be revoked.

Definition, the ninth admission term, called the abandoned root exclusive quantity

A separate, decision level formula for admission control lists an upper bound on how many slots are exclusively tied up by an abandoned root. In one place, that upper bound is defined as the abandoned root's own allocated statistic total minus R old's own allocated statistic total. Each of those two totals is read directly from the accounting tree root reachable from that root, without walking the whole tree and without adding any new on disk field. This upper bound is computed once, at the moment R old is chosen during a rollback, kept only in memory, computed separately per device, and reset to zero once the abandoned root in question is overwritten by ring rotation. The same source states that this quantity is maintained by the rollback code path.

A separate, tracked issue elsewhere in the project's records restates the same intended upper bound using different words, as the amount allocated by the abandoned timeline since R old, and adds in parentheses that this is computed the same way, as a difference of the allocated statistics carried by root records, without walking the tree. That same tracked issue separately warns that the literal difference of two totals formula can go negative once the abandoned timeline has had net releases, meaning more space freed than allocated after R old, that this negative sign is the opposite of what the quantity is supposed to mean, since it is supposed to represent an amount of space that is tied up and unavailable, that no clamping to zero is specified anywhere, and that these two ways of stating the intended quantity are explicitly flagged, in the project's own records, as two different specifications for what is supposed to be the same underlying number.

As of today, nothing in the running code computes or subtracts this ninth term in any admission check anywhere. The only mechanism that actually keeps those slots from being handed out is the physical isolation marking described above, G5.

Facts about a fixed test script

A fixed test script runs a sequence of steps: creating the filesystem, several publishes under one instance, a rollback to an earlier root called A, more publishes under the new instance, and further steps up to a point labeled E.

Right after the rollback publish, G5's isolation count is 34 slots per device. These are slots referenced only by root B, by the row writing publish, by the warm up publishes that follow it, and by root C. None of these 34 slots are also referenced by root A, which is still a candidate at that point.

Later, the floor F is raised so that effective floor becomes checkpoint_txg 11. Before the corresponding reclaim of freed space happens, G5 recomputes and isolates 2 more slots per device. These 2 slots hold an instance table written at filesystem creation time. They had previously been exempted because root A was still a candidate, but once effective floor reaches 11, root A's checkpoint_txg falls below the new floor, so root A stops being a candidate, and those 2 slots are now referenced only by the abandoned root B. The isolation count is now 36 slots per device.

At step E in this scenario, the space that gets reclaimed and then handed out again lands on one particular slot, which is legitimate at that point: an earlier version of that slot, belonging to root A, has already been released, and the slot is otherwise referenced only by the generation zero root.

Facts about the allocation record tree reader

There is a function, used both to read a root's own ledger and to decide whether that root's slots should be exempted or isolated, that works as follows given a root record. It reads that root's tree table, finds the entry in the tree table whose kind marks it as the allocation record tree, and reads the node that entry points to. It requires that node to be at level 0, meaning a single node that directly holds allocation record entries, not the root of a taller, multi level tree with internal pointer nodes underneath it. If the node's level is anything other than 0, the function returns a format error, and does not attempt to interpret that node's contents as allocation records.

A comment attached to this code explicitly states that in this first version, the allocation record tree is assumed to fit in a single node, at level 0, that the code path for a taller, multi level tree has not been exercised yet, and that this is tracked as unfinished work for a later milestone step.

Each node is 16384 bytes. This exact number appears directly in the code. Each allocation record occupies 20 bytes. The exact per node header overhead for this kind of node is not stated here, so the precise maximum number of records that fit in one node cannot be computed exactly from the facts given here. It can only be loosely bounded: no more than 16384 divided by 20, which is 819, and somewhat fewer once header overhead is subtracted.

Questions about the ninth admission term and G5

1. Is G5's isolation count, by construction, mathematically the same operation as taking the abandoned root's allocated statistic total and subtracting R old's allocated statistic total? Walk through what each computation actually counts, slot by slot versus total by total, and state explicitly whether they are the same operation, or two different operations that could coincidentally produce the same number in some specific cases.

2. At the step right after the rollback, where the isolation count is 34 slots per device, and at the step right after the floor is raised to checkpoint_txg 11, where the isolation count is 36 slots per device, would the formula abandoned root's allocated statistic total minus R old's allocated statistic total necessarily also produce 34 and then 36? If you cannot compute the exact totals from the facts given here, explain the mechanism that would make the two numbers equal, or explain the mechanism that would make them differ, and name what additional fact you would need in order to compute the exact totals yourself.

3. If your answer to question 2 is that the two numbers come out the same, is that a general property that always holds given how the two computations are defined, or is it a coincidence specific to this particular fixed script, for example because the only other candidate root besides R old happens to have an allocated set that does not matter here? If your answer is that the two numbers can differ, name precisely which step of G5's computation, a, b, c or d as listed above, is responsible for the difference.

4. The project's own records describe the ninth term's intended meaning in two different ways: first, as the abandoned root's allocated statistic total minus R old's allocated statistic total, and second, as the amount allocated by the abandoned timeline since R old, while also noting that both are meant to be computed as a difference of allocated statistics, without walking the tree. A separate note flags these two descriptions as two different specifications of what is supposed to be the same quantity, and separately observes that the difference formula can go negative once the abandoned timeline has had net releases. Do these two descriptions compute the same number whenever there have been no releases at all on the abandoned timeline after R old, and only start to diverge once there have been releases? Or can they diverge even when there have been no releases? Explain your reasoning, and say whether the goes negative on net release problem described for the first description also applies to the second one.

5. Today, nothing in the running code computes or subtracts the ninth term in any admission check, and the only thing that actually prevents those slots from being handed out is G5's physical isolation marking. Given that the admission inequality is not wired up to this ninth term at all yet, does it matter in practice whether the ninth term's formula matches G5's isolation count exactly? Is there any concrete harm today from a mismatch between the two, if one exists, or is this purely an inconsistency between two decision documents with no effect on running behavior until someone actually wires the ninth term into a real admission check?

Questions about the single level guard on the allocation record tree

6. Given that each allocation record is 20 bytes, that each node is 16384 bytes, and that the counts you have seen in this fixed script are small, isolation counts around 34 to 36 slots and a handful of slots being reclaimed and reallocated, does anything in the facts given here suggest that this fixed script, or the first version of this filesystem in general, could ever produce a root whose live allocation record count is large enough to require more than one node? Answer using only what can be derived or bounded from the facts given here. If you cannot bound the node's exact capacity from the given facts, say so explicitly rather than assuming a specific number.

7. Assuming the number of live allocation records under any one root stays far below whatever a single node can hold, is it accurate to describe today's single level guard as tracked debt only, with no live impact today, rather than as a bug that is already being triggered somewhere in the fixed script? What would have to grow, in terms of pool size, number of live objects, or accumulated allocation record history, before this guard would start rejecting a tree that grew that tall for entirely legitimate reasons, as opposed to a tree that is genuinely corrupted?

8. After the format error is raised, two different callers handle it in two different ways. When reading an abandoned root's ledger, the caller skips that root entirely: it is counted, the mount is not rejected, and none of that root's slots get isolated through this mechanism. When reading a candidate root's ledger, the caller treats that candidate root as exempting nothing at all, so isolation can only grow, never shrink, as a result of this kind of failure. Are both of these two directions safe, with respect to the rule that a candidate root's own slots must never be isolated out from under it while that root is still valid and in use? Specifically, could treating an unreadable candidate root as exempting nothing at all ever cause a slot that some other still valid, in use root needs to be marked isolated, purely because that in use root's own ledger happened to be unreadable at that particular moment?

9. The guard cannot distinguish between two different situations that produce the exact same format error: a root whose allocation record tree has legitimately grown past a single node because there are simply many live allocation records under it, versus a root whose allocation record tree root is genuinely corrupted, for example by a bit flip that changes the stored level field, or by a stray pointer. Does this milestone's claim of correctly handling a torn or corrupted abandoned root's ledger depend on being able to tell these two situations apart? Or is it acceptable, for now, to treat both situations as the same failure with the same handling, and defer telling them apart to later work?

Instructions

Answer all nine questions, in order, using the question numbers. For each answer, add a short paragraph that states what observation or fact, if it turned out to be true, would refute or overturn that answer. Write in plain English prose and plain numbered lists only. Do not use any markdown emphasis, such as bold text, italic text, or backtick code formatting, anywhere in your answer.
