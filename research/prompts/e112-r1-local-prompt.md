You are auditing a decision model for a copy-on-write filesystem. Answer in English only.
Do not use any markdown emphasis. Plain text and plain numbered lists only.

THE SITUATION

An on-disk table lists every btree in the pool, one entry per tree. Each entry carries a two
byte field of boolean flags. Bit zero means "this tree is a writable head". The other fifteen
bits are unused in version one.

A version one writer mounts a pool that a version two writer created. Version two set one of
the unused bits on some entries. Version one has three possible policies for such an entry:
  Ignore  - proceed as if the unknown bit were not set, interpreting the tree with the
            semantics it already knows.
  Skip    - treat it as a tree it does not understand and pass over the entry entirely.
  Refuse  - refuse to mount the pool at all.

Separately there is a space accounting btree. Each statistic is stored as a complete value,
never as a delta, keyed by (statistic tag, dimension tuple, generation number). Only the last
four generations are retained; anything older is discarded. Exactly one statistic carries a
per tree dimension today, and only for writable heads. So a tree that is not a writable head
has no per tree accounting rows at all.

The version one writer has two possible policies for accounting rows belonging to trees it
does not understand:
  Carry   - rewrite each such row into the new generation unchanged, same value, new
            generation number.
  NoCarry - do not rewrite them.

A discrete model of ten generations gives these results. Twelve known trees of which six are
writable heads; four unknown trees of which two are writable heads; one per tree statistic and
eleven pool wide ones; four generations retained; one thousand records per unknown tree.

  Ignore  + Carry    mounts, 4000 records misinterpreted, 0 rows lost, 19 rows written per generation
  Ignore  + NoCarry  mounts, 4000 records misinterpreted, 0 rows lost, 19 rows written per generation
  Skip    + Carry    mounts, 0 misinterpreted,            0 rows lost, 19 rows written per generation
  Skip    + NoCarry  mounts, 0 misinterpreted,            2 rows lost, 17 rows written per generation
  Refuse  + either   does not mount, nothing happens, 0 rows written

An outcome is called quiet when something went wrong and nothing in the system would report it.
By that measure Skip plus Carry is the only combination that mounts and has zero quiet outcomes,
so it was chosen.

QUESTIONS

1. Check the arithmetic. For each of the six rows above, derive the numbers yourself from the
   parameters and say whether you get the same. Name any row you cannot reproduce.

2. The quiet measure counts an outcome as bad only if something went wrong AND nothing reports
   it. Refuse scores zero quiet outcomes because nothing goes wrong. Is choosing by fewest quiet
   outcomes a sound way to pick among these six? If not, say exactly what it fails to price in,
   and propose a measure that does.

3. Under Ignore, the writer maintains the accounting rows of trees it misunderstands, so zero
   rows are lost. Is that actually a good outcome, or is it worse than losing them? Argue both
   directions, then say which you believe and why.

4. Under Carry, a writer rewrites a row whose statistic tag it does not recognise. To rewrite
   a row it must at minimum locate the row and write it back under a new generation number.
   Name every piece of information the writer needs in order to do that, and say for each
   whether a writer that does not recognise the statistic tag can be assumed to have it.

5. Name the strongest objection to choosing Skip plus Carry that questions 1 through 4 did not
   cover.
