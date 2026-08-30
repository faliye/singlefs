You are attacking a design session for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Do not use any markdown emphasis in your answer. Write plain English. Keep it under 700 words.

Your attack angle is: what was never examined at all. Not errors, but omissions. Two other
attackers cover the repairs made after the first attack round and the methodology, so do not spend
words on those. A first attack round already covered over-extrapolation, internal contradictions,
and which conclusion is most fragile, so do not repeat those either.

What this session did:

1. Ran three pure-arithmetic experiments. One on how many bytes an extension point inside each
on-disk unit may occupy. One on the geometry of a rotating ring of root slots. One on how many bits
a ciphertext checksum needs.
2. Recorded two owner rulings: saving space is never a criterion, and unit geometry is computed
once at mount time and never per operation, each check returning mount or refuse-to-mount.
3. Settled that disks in a pool need not be equal in size, on the narrow ground that pointers carry
a device identity so addressing stays correct when capacities differ.
4. Rejected a candidate root slot width of 256 bytes, because on a 512 byte atomic write width one
write endangers two slots. The rule became: slot width at least the atomic width probed at mount.
5. Reproduced an existing derivation that 32 bits suffice for the ciphertext checksum, and measured
that the margin is only 3.37 times.
6. Corrected an invariant: ring depth is an upper bound on block reuse delay, not the same number.
7. After a first attack round, retracted one number, completed one broken chain of citations across
four files, changed one check from a pool-wide value to a per-device one, narrowed one settled item
to a sub-problem, and reworded one overstated claim.

Everything above is arithmetic and modelling. There is no code for the filesystem itself, no
checker, no crash-point replay harness, and no test on real hardware other than reading this
machine's reported block sizes.

Your task, in this order:

1. Name the questions this session should have asked and did not. Not questions that were asked and
answered wrongly, but ones that never came up at all. Give at least five, each in one or two
sentences, and say what makes each one load bearing.
2. For each of the three experiments, name one input the model treats as fixed that in a real
filesystem would be chosen by someone, and say who chooses it and when.
3. This session spent its effort on geometry and byte widths. Name what class of design question
gets systematically neglected by that focus, and give a concrete example of something that could be
badly wrong right now without any of this session's work noticing.
4. Name one thing this session did that will be expensive to undo later, and say why.
