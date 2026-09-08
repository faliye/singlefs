You are reviewing a design gap in a copy-on-write filesystem being built from scratch in Rust.
Your assigned stance: find counterexamples. Do not argue that the plan is good. Try to break
each option with concrete scenarios.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Facts are labeled F1 to F13 and were verified against the repository by the requester.
P1 to P5 are candidate placements under review, not facts.

--- BACKGROUND START ---
F1. Two different device quantities. physical_block_size is the width the device promises to
    write atomically across a power failure. io_min is the smallest write width that does not
    make the device do an internal read-modify-write. On Linux physical_block_size is
    min(preferred_write_granularity, atomic_write_unit_power_fail) and io_min is the preferred
    write granularity, so physical_block_size is always less than or equal to io_min. On a
    md raid5 array io_min is the stripe chunk size, commonly 64 KiB, while
    physical_block_size can be 512.

F2. Settled decision D2 has two hard requirements. One: never issue a write smaller than the
    device physical mapping unit, which on Linux is io_min, not physical_block_size. Two:
    never let two objects with different lifetimes share one physical mapping unit. The
    reason for both is that a sub-unit write makes the device do an internal
    read-modify-write, and a power failure during that can corrupt already persisted
    neighbour data inside the same mapping unit.

F3. Settled decision D22 item 2 fixes the root ring geometry. Three regions, placed at prime
    stride, slot order rotates across regions with region equal to txg modulo 3, slot width
    equal to the physical_block_size probed at mount time, no reserved slots. Each root slot
    holds one self-certifying root record. The ring exists so that if the newest root is torn
    the previous generation is still readable.

F4. Experiment E34, arithmetic half. With io_min 64 KiB and physical_block_size 512, all 16
    slots of one region land inside a single io_min unit. In 18 of 48 parameter cells one
    write knocks out two or more slots. Changing the slot width to max(physical_block_size,
    io_min) drops all 48 cells to one slot at risk.

F5. Experiment E34, real device half, on a real md raid5 array, ten rounds with identical
    verdicts. The 128 times gap between io_min and physical_block_size was measured and is
    real. But the 16 slots do not fail together. Raid5 parity is computed per sector row, so a
    torn write only pollutes the row it lands in. Neighbour slots in the same chunk survived
    ten out of ten, and the two other regions survived ten out of ten. So the exposed area is
    not the lost area, and the 16 is an upper bound. This covers only the raid5 write hole
    mechanism. SSD internal read-modify-write over a whole mapping unit cannot be injected on
    loop devices and remains untested.

F6. The D22 body text contains this unanswered sentence: whether different slots inside the
    same root ring region count as two objects with different lifetimes has never been
    written down anywhere in the repository. The body then says the slot width is not being
    changed for now.

F7. That sentence has no landing spot. It is not one of the D22 open items, which are only
    numbers 6 and 9. There is no entry for it among the 195 entries of the owed-checks file.
    The first transaction layout table marks the root ring row as settled.

F8. Two gate stages exist that would otherwise track it. Stage 31 requires every open item to
    have a verdict on whether it changes the bytes of the first transaction. Stage 60 checks
    whether an open item was silently settled elsewhere. Both only see items that are
    registered as open items.

F9. An existing owed check C82 proposes a mount gate: refuse a writable mount unless io_min is
    at most 32768 and 32768 is a multiple of io_min. The requester computed that io_min equal
    to 32768 with physical_block_size equal to 512 passes that gate and still puts 64 slots
    inside one mapping unit. So C82 landing does not close this question.

F10. D2 is fully settled, all 14 of its items.

F11. Raising the slot width from 512 to 64 KiB multiplies the root ring footprint by 128. The
     stated reason in D22 for the current slot width is that anything larger buys no safety
     and only costs bytes. That sentence was written before the io_min dimension existed.

F12. Two project rules apply. A pitfall must be turned into a check that fails, not into a
     reminder sentence. Format level branches are permanent because every future reader must
     support every historical layout, while behaviour branches can be deleted.

F13. Every owed check entry must state what the check checks and must state a self proof of
     discriminating power, that is, an injected fault that the check must catch. An entry
     that cannot state those two things is a fake entry.

--- PROPOSALS ---
P1. Register it as a new open item of decision D22, and give it the mandatory verdict on
    whether it changes the bytes of the first transaction.
P2. Register it as a new owed check entry only, leaving the decisions untouched.
P3. Reopen decision D2, on the ground that the missing definition is the phrase objects with
    different lifetimes, which belongs to D2, not to D22.
P4. Register nothing. The real device half of E34 already narrowed the loss surface, so what
    remains is a known residual risk and the prose is enough.
P5. Some combination, for example the missing definition goes to a decision and the missing
    check goes to an owed check entry.

--- BACKGROUND END ---

Answer these four questions.

1. Construct a concrete scenario in which choosing P4 leads to a wrong outcome. Give concrete
   device parameters, a concrete sequence of writes and failures, and say exactly what goes
   wrong and who would have caught it if the question had been registered. If you cannot
   construct one, say so plainly.

2. For each of P1, P2, P3 and P5, give the strongest concrete reason it is the wrong place for
   this question. One paragraph each.

3. Is there a sixth placement the requester did not list? If yes, describe it and say what it
   buys that P1 to P5 do not.

4. Is the question itself malformed? In particular, consider whether the phrase objects with
   different lifetimes is already pinned down by something in F1 to F13, so that the answer
   for root slots follows without any new decision. If so, state the answer it forces and the
   facts it follows from.
