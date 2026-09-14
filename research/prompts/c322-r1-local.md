You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 6.

Settled facts (do not dispute them; use them):

F1. A pool has two devices. Each device has two superblock slots. A slot carries a generation number that starts at 1 and grows by one on every superblock write; the next write goes to slot (generation mod 2); within one device the reader picks the slot whose checksum is valid and whose generation is largest. Nothing in the rules says how to pick among the superblocks of different devices.

F2. Format writes instance code 0 into every superblock, meaning "no instance yet". The first writable mount takes the new instance code as max(superblock, root ring) + 1 = 1 and writes it into every superblock with one superblock slot write per device (generation becomes 2), and only after that touches any other structure. The rule text does not say whether a flush barrier sits between those two superblock writes, or after them.

F3. An invariant says: among the superblocks of the pool, the instance codes are equal, and they are not lower than the instance code of any root record in the root ring. Its note calls the instance acquisition "all or nothing".

F4. Every publish, including the two empty warm-up publishes that follow instance acquisition, persists in this order: data and tree units, flush barrier, journal record, flush barrier, root slot written with force-unit-access, then the superblock slots rotate. Each warm-up publish starts with a flush barrier before its journal record. A flush barrier makes every earlier write on every device durable before any later write is issued.

F5. The implementation writes the two acquisition superblock writes with no barrier between or after them; the opening barrier of the first warm-up publish closes that segment. A crash-state enumerator cuts the write stream at barriers and force-unit-access writes, takes every subset of the writes inside the current segment, and treats all earlier segments as fully durable. Over the 262165 crash states of the first transaction, an independent checker found exactly 2 states that violate the invariant in F3: the two states where only one of the two acquisition superblock writes is durable. In both, the root ring holds only the format-time root (instance 0) and the journal holds no record of instance 1; recovery picks the format-time root and reports that no file exists, which the correctness oracle accepts.

F6. The recovery code picks, per device, the valid slot with the largest generation, checks only that the devices agree on filesystem id and device count, and returns the first device's superblock. The mount-time instance acquisition is not implemented yet, so nothing decides which number the next acquisition starts from when the devices disagree.

F7. The open work item asks for a rule for this state: "one device got the acquisition write, the other did not, and the warm-up record is already durable". It also asks for a self-test: changing the superblock choice to "compare generations only, ignore instance codes" must turn the check from green to red.

F8. Three candidate fixes. A: add clauses only, no new barrier. The opening barrier of the first warm-up publish is the durability point for "written into every superblock before touching anything else"; unequal instance codes mean an unfinished acquisition, the reader picks across devices the superblock with the largest generation, and the next acquisition takes max(all superblocks, root ring) + 1. The invariant in F3 becomes: instance codes are equal, or, if they differ, no root record and no journal record in the pool carries the largest of them. B: write device 0's superblock, barrier, device 1's superblock, barrier, and keep the invariant unchanged. C (reference arm): make the instance code authoritative only in one atomic write, such as a root record that changes only the instance code, and demote the superblock copy to a hint; this changes the format.

Answer these six items:

1. Under the ordering in F4 and F5, can the state quoted in F7 ("one device got the acquisition write, the other did not, and the warm-up record is already durable") be reached at all? Give the reasoning from the barrier rules. Are the two states in F5 the only states in which instance codes differ?

2. For each of A, B and C, is there a crash state after which the next acquisition would hand out an instance code that some durable root record, journal record or unit already carries? If yes, describe the durable write set of that state.

3. The note in F3 calls acquisition "all or nothing". With two independent device writes, can it be all or nothing physically under A or B? If A rewrites the invariant, can you construct a pool image that the rewritten invariant accepts but that is actually wrong? Describe it.

4. Under the rules in F1 and F4, can the two devices ever carry superblocks whose generation order disagrees with their instance-code order in a reachable crash state? If never, then the self-test in F7 is equivalent to the original rule and cannot turn red; say what self-test would work instead.

5. Count the cost of each candidate: extra flush barriers per acquisition, format changes, and whether the recorded segment sequence of the first transaction changes.

6. Which candidate do you recommend, and what single observation would make you change that recommendation?
