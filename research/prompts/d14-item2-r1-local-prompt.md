You are attacking a filesystem design choice. Find concrete counterexamples. Do not summarize,
do not agree, do not restate the proposal back at me.

Do not use any markdown emphasis anywhere in your answer. No bold, no italics, no asterisks.
Write plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem is choosing what rule its block allocator should use for
two hints: put the blocks of small files near each other, and put data that is probably
short-lived into a region that is expected to be freed all at once.

The hint layer is claimed to owe nothing, on these grounds: it never reaches the disk, guessing
wrong only costs locality, every allocation may re-judge it, and it uses zero format bits.

The project has five hard requirements for any branch. A branch rule must be explicit and
computable and written down. Every branch must be forcible in a test through a switch that
ignores the rule. Every branch must be verified on its own, including crash-point replay. Every
branch must be observable at run time, because taking the wrong branch is a performance cliff and
cliffs do not raise errors. And the branch variable must not change in the middle of an operation.

One candidate is already dead. File size was measured: a 4 KiB file written in eight appends
lands on one side of a size threshold and the same file written in one shot lands on the other,
so file size changes mid-operation and violates the fifth requirement. Do not propose file size.

Two settled clauses will bite. First, when encryption is on, file names, separator keys and any
path information may never be left in plaintext. Second, the inline threshold is zero: a small
file is never inlined, it always gets its own data extent padded out to a full 32768-byte unit.

One more fact: the physical arrangement of units on the device is visible to someone reading the
raw disk without the key, and it is already known that units written in one commit are clustered.

THE FOUR CANDIDATE ARMS

Arm A. A suffix table in the superblock, the way F2FS does it: an administrator configures a list
like so, db, jpg, and the allocator matches a file's name suffix at allocation time.

Arm B. The caller supplies the hint: an open flag, a posix_fadvise call, or an agreed xattr. The
filesystem never guesses on its own.

Arm C. By directory: objects under one parent directory are placed together, and "probably
short-lived" is annotated per directory by the administrator.

Arm D. Do not implement hints in the first version at all. Build one slow correct path and leave
the hint layer for the day there is a rule.

YOUR TASK

1. Attack arm A. Give a concrete operation sequence in which the suffix rule takes the wrong
   branch and the cost is more than lost locality. Then say whether the fifth requirement
   (the branch variable must not change mid-operation) is really satisfied, given that a file can
   be renamed, hard-linked into a second name, or created with a temporary name and renamed into
   its final name afterwards.

2. Attack arm B. The caller supplies the hint, so it is explicit. Construct a case where a hostile
   or merely careless caller uses the hint to damage the filesystem or another tenant: think about
   what happens if every caller claims its data is short-lived, or if nobody ever sets the flag.

3. Attack arm C. Construct a case where placing everything in one directory together is worse
   than not grouping at all.

4. The hint layer claims to use zero format bits. Attack that claim. To satisfy the requirement
   that every branch is observable at run time and forcible in a test, what has to exist, and can
   all of it really live outside the on-disk format? Be concrete about where each piece lives.

5. Under encryption, arms A and C group objects by a property the filesystem inferred. Someone
   reading the raw disk without the key sees the physical arrangement. Construct the inference
   chain by which that grouping leaks something the design elsewhere works hard to hide.

6. Name the single assumption across all four arms you think is most likely to be false, and give
   the observation that would reveal it.

Give constructions, not opinions.
