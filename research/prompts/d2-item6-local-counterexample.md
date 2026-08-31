You are reviewing a design inference for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 500 words.

Settled decisions you must not argue with.
S1. Stripe width is variable. Every write is a full stripe write. Read-modify-write never happens.
S2. Writes smaller than the device preferred write granularity are never issued.
S3. A pointer location entry lists every physical location one by one, each carrying a device identity.
S4. Disks in a pool need not be the same size.
S5. Each stripe has exactly one parity cell in the model under discussion.

Measured data from this project, eight devices, twenty thousand writes, five seeds.
Physical cells per data cell, and the probability that a stripe loses data when two arbitrary
devices fail at the same time:
always take full width: 2.00 and 1.000 at fifty percent small writes, 4.34 and 1.000 at ninety percent.
width fitted to the write size: 1.52 and 0.688 at fifty percent, 1.71 and 0.215 at ninety percent.
fixed width three: 1.68 and 0.107, and 2.21 and 0.107.
batch first then full width: 1.18 and 1.000, and 1.26 and 1.000.

The inference under attack, call it U: the upper bound on stripe width should be four, that is three
data cells plus one parity cell. The reasoning is marginal. With eight devices, the chance that two
simultaneous device failures destroy a given stripe is the number of pairs inside the stripe divided
by the number of pairs in the pool. Going from width three to width four saves 16.7 points of space
overhead and adds 10.7 points of that risk, so it is worth it. Going from four to five saves 8.3
points and adds 14.3 points, so it is not. The general rule would be: the bound is one plus the
largest v such that v squared times v minus one is at most the number of pairs in the pool.

Your task, in this order:

1. Recompute the arithmetic yourself and say whether the crossover really falls at four.
2. Attack the weighting. The argument treats one point of space overhead as equal to one point of
probability of losing data. Name what changes if that weighting is wrong, and say what bound you get
under two other defensible weightings.
3. Give concrete counterexamples to U. A counterexample is a specific situation where a bound of four
produces a worse outcome than a larger or smaller bound, for reasons other than speed. Do not argue
from throughput; this project forbids using speed as a criterion.
4. State what happens to U if each stripe has two parity cells instead of one.
5. State whether a bound that is a fixed constant four, independent of how many devices the pool has,
is better or worse than a bound computed from the device count, and why.
6. Accept U, reject U, or accept a narrowed version, and give the narrowed version if that is your
answer.
