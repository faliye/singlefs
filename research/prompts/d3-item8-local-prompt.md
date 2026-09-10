You are attacking a filesystem design choice. Find concrete counterexamples. Do not summarize,
do not agree, do not restate the proposal back at me.

Do not use any markdown emphasis anywhere in your answer. No bold, no italics, no asterisks.
Write plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem must decide where a newly written user data block goes on
the device. Today no rule exists at all: one settled clause says that blocks produced by the
commit itself (tree nodes, accounting nodes) must be allocated from a contiguous bump segment,
and that same clause says in so many words that user data blocks are not bound by it. So user
data placement is a blank, and so is the fallback question: when the bump segment has no room
left, where does this allocation go?

WHAT IS ALREADY MEASURED

A counting model was run with 8192 objects, 10240 device slots (80 percent full), segment length
64 slots, 64 objects dirtied per checkpoint, 2000 rounds, 5 seeds, median reported. The
fragmentation metric is runs divided by object count, where runs is the number of physically
contiguous stretches when walking all objects in key order. Higher runs is worse.

Policy first-fit means take the lowest numbered free slot. Policy bump-seg means sort this
checkpoint's dirty objects by key and write them contiguously into a bump segment, falling back
to first-fit slot by slot when no empty segment is available.

Under a uniform random rewrite load: first-fit gives 8186 runs, bump-seg gives 8186 runs. They
are identical, and 8186 out of 8192 is 99.9 percent, meaning almost every object sits alone.

Under a segmented rewrite load (runs of 8 adjacent keys dirtied together): first-fit gives 7997
runs (97.6 percent), bump-seg gives 7312 runs (89.3 percent).

Neither policy costs any extra writes.

First-fit under uniform random reaches 8184 runs at round 500, which is about 3.9 rewrites per
object, and then stays flat.

Bump-seg falls back to first-fit for 91.8 to 93.8 percent of allocations across the five seeds,
because scattered frees almost never add up to a fully empty segment. Adding a background
defragmenter with a budget of 32, 128 or 256 objects per checkpoint pushes that fallback rate
down to roughly 88, 73 and 50 percent respectively.

The experiment states plainly that its two loads are endpoints and not a distribution, and that
there is no evidence about where a real aging workload falls between them. It also states that
objects are fixed width in the model and variable length extents are not modelled.

OTHER SETTLED FACTS THAT BITE

The allocation granularity is 16384 bytes, and one 32768-byte data unit occupies two slots.

A background defragmenter exists. It starts when any of three watermarks is crossed: number of
fully empty bump segments, fragmentation runs over object count, and unreclaimable bytes. It
stops when the number of fully empty bump segments has grown by at least one since the moment
the defragmentation intent was written. The region being cleared may not be an allocation target
while that intent is open.

An admission control inequality counts unreclaimable space against free space, so placement that
produces unreclaimable holes can turn into a premature out-of-space error.

Objects of 4096 bytes and under are moved into shared containers by that same background
defragmenter after they are written, walking in the key order of a central mapping. So for that
size class this placement decision only governs the window before the packer catches up, and the
length of that window is set by a hysteresis constant that has no number anywhere.

THE FOUR CANDIDATES

Candidate A. First-fit: lowest numbered free slot, always.

Candidate B. Bump-seg: sort this checkpoint's dirty objects by key, write them contiguously into
a bump segment, fall back to candidate A slot by slot when no empty segment exists.

Candidate C. Arrival order: objects of one publish (or arriving consecutively) go to adjacent
slots. No sorting, no reading any on-disk field.

Candidate D. By directory: objects under one parent directory are placed together.

YOUR TASK

1. Attack candidate B. Give a concrete sequence in which sorting by key before placement produces
   a worse outcome than not sorting. Name the step where it goes wrong.

2. Candidate B falls back to candidate A between 91.8 and 93.8 percent of the time. State
   plainly: under that fallback rate, is B a different policy from A, or is it A with extra
   bookkeeping? Give the arithmetic that settles it.

3. Attack candidate A. Its defenders say it needs no clause at all because it is what a naive
   allocator does anyway. Construct a case where two implementations both calling themselves
   first-fit produce different behaviour, so that the rule does have to be written down.

4. The defragmenter stops when the count of fully empty segments has grown by at least one.
   Construct a placement policy and a workload under which that stop condition is never satisfied,
   so the defragmenter runs forever. Say which of the four candidates is closest to your
   construction.

5. Candidate C reads no on-disk field and sorts nothing. Find the case where that costs more than
   candidates A and B. Be concrete about the workload.

6. Say which one candidate you would attack next and why, in one sentence.

Answer in English. Be concrete. Numbers and step names, not adjectives.
