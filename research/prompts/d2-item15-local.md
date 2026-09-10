You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize. Find cases that break things.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A settled hard requirement from 2026-08-28 says: do not let two objects with
different lifetimes share the same physical mapping unit. The reason given is that
writing a piece smaller than the device's minimum io size makes the device do an
internal read-modify-write, and a power cut during it may damage neighbouring data
already persisted in the same mapping unit.

What that clause never defined is what "different lifetimes" means. Two consumers
are stuck on it.

Consumer A, tombstones. A settled decision packs tombstone records by generation.
An experiment states verbatim that a tombstone record's lifetime is the death
queue it belongs to, that is, the snapshot generation. So same-generation
tombstones share a unit, and that is considered fine.

Consumer B, root slots. The root ring is R = 3 regions, regions placed at prime
strides, slot order rotating across regions, and slot width equals the runtime
probed physical block size. On a device where the minimum io size is larger than
the physical block size, several slots land inside one physical mapping unit. Each
slot holds a different generation of the root. Nobody has decided whether "two
slots in the same region" counts as different lifetimes.

Measured facts:
- On an md RAID5 array with 4 legs and 64 KiB chunk, minimum io size is 65536 and
  physical block size is 512, so they differ by 128 times. One mapping unit covers
  a whole region: with 16 slots per region, one write knocks out all 16.
- Across regions the prime stride placement means one mapping unit cannot cover
  two regions, and rotation across regions bounds the worst case to falling back
  one generation. That was measured, five rounds, on a real array.
- Measured 2026-09-10 on real devices: whether a neighbour is actually damaged is
  mechanism dependent. On a thin provisioning target with 64 KiB blocks, a
  512 byte write to a shared block makes the device write 129 sectors, a real
  internal read-modify-write. But when that copy is torn precisely in half, the
  neighbour is byte identical in all five rounds and the writer gets an error.
  The copy is transactional: allocate a new block, copy, and only then switch the
  mapping. By contrast the RAID5 write hole does damage the neighbour.

YOUR TASK

Construct concrete counterexamples, with specific operations and numbers.

Q1. Propose a precise definition of "different lifetimes" and then construct a
    case where your own definition gives the wrong answer for one of the two
    consumers. State the definition first, then break it.

Q2. Construct a case where defining it as "the two objects die at different times"
    and defining it as "the failure of one can damage the other" give opposite
    answers. Be specific about the objects and the device.

Q3. Construct a case where the root ring's prime stride placement and cross region
    rotation do NOT bound the damage to one generation. State the device geometry,
    the region count, the slots per region, and the write that does it.

Q4. Construct a case where the transactional behaviour observed on the thin
    provisioning target does not generalise, that is, a device class where an
    internal read-modify-write is not transactional and a neighbour is silently
    damaged. Name the mechanism.

Q5. Construct a case where some object pair in a filesystem of this design shares
    a physical mapping unit and nobody has noticed, beyond tombstones and root
    slots. Name the pair and why they end up in one unit.

For each answer, state the operation sequence, the concrete values, and what
exactly goes wrong. If you cannot construct a counterexample for a question, say
so in one sentence and move to the next.
