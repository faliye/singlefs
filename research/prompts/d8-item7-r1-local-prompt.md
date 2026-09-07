You are auditing a filesystem design question. Answer in English only.
Do not use any markdown emphasis. Plain text and plain numbered lists only.

SETTING

A copy-on-write filesystem keeps space accounting in a btree. An entry is written as a complete
current value, never as a delta. The btree key has four fixed width parts:
  statistic tag 2 bytes, tree id 8 bytes, device 4 bytes, generation 8 bytes.
The generation is the publish counter, so it advances on every publish.

Within one publish window the same four part key is written many times, because a complete value
is rewritten whenever the quantity changes. A front end buffer collects these writes in memory
and folds them before they reach the btree. The folding rule is last writer wins.

The buffer is flushed by first sorting the collected entries by key, which destroys arrival
order. So the folding needs some way to tell which of two entries with equal keys came later.

Three candidates for where that ordering information lives:
  A. Extend the btree key with an ordinal, making it a five part key.
  B. Keep the key as is and put the ordinal in the stored value.
  C. Do not store it at all. Keep the ordinal only in memory, as the position of the entry in
     the buffer array, and make the sort key the pair (btree key, array position) so the sort
     is stable and the last one for each key is the winner.

Additional facts:
  - Only the most recent K generations of accounting rows are retained; older ones are discarded.
    K is not yet chosen.
  - The key encoding is frozen at a specific format layer. The value encoding is not in that layer.
  - A checker verifies accounting independently and is required not to share code with the
    writer, so any rule the writer uses to decide a winner must also be expressible to the checker.
  - Index nodes carry message buffers, so a write may sit in an interior node for a while before
    reaching a leaf.

QUESTIONS

1. Under C, name every situation in which two entries with the same four part key could be
   present on the medium at the same time. For each, say what a reader would have to do to
   decide which is current, and whether it can do so without a stored ordinal.

2. Under A, discarding old generations means removing every row whose generation part is older
   than a cutoff. Compare how that removal works with the four part key versus the five part key.
   Be concrete about what the remover must know before it starts.

3. Under B, once the front end has folded the entries, at most one entry per key reaches the
   btree. Is the stored ordinal then dead weight, or does some reader still need it? Name the
   readers you can think of and say for each.

4. Rank A, B and C. State the single property that decides your ranking, and name what would
   have to be true for the ranking to change.

5. Name the strongest objection to your top choice that questions 1 through 4 did not cover.
