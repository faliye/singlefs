You are the defense leg of a three-way adversarial review for a from-scratch
copy-on-write filesystem. Two earlier attack rounds recorded hits against one design
verdict. Your job is to defend: for each hit below, try to show it does not land, or
that it lands equally on the alternatives it was used to separate. Do not argue which
option is better overall. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A central map is the only entry point for dereference and for the free decision. Unit
classes: code 1 data units, code 2 index nodes, code 3 packed record units. Small objects
can be packed into slots of a shared container unit, which is a new unit class with its
own code c, not 1.

Candidates for the map key (only these three matter here):
- K (class-reuse write-order key): code 1 key = class tag + birth tree + birth checkpoint
  + write order (instance id 4 + transaction number 6); code 2 and code 3 key = class tag
  + birth tree + birth checkpoint + instance id + birth sequence 4. The birth sequence is
  a new 4 byte header field in code 2 and code 3 units, issued in memory from 0 per
  (tree, checkpoint number, instance id), used only as a key segment and never to choose
  the newest version. Code 1 headers get no new field.
- P (birth placement key): the unit's birth location (device + slot) frozen as the key,
  plus birth checkpoint and birth tree; every unit header, in all three classes, gets a
  new 10 byte field recording its birth location.
- R (random 16 byte id key): every unit header, in all three classes, gets a new 16 byte
  random identifier; its entropy source must not be reproducible.

Settled texts you can rely on (translated from the repository):
T1. A settled decision rejected a lineage rewrite sequence number in unit headers,
    because "what it would buy has already been bought" by a per-location predicate.
    Its range note says: "The decision rejects a sequence number meant for telling stale
    copies from current ones. If another use appears in the future (for example, if the
    question of how it coexists with the settled checkpoint number really stands on its
    own), reopen it if you must, but whoever reopens it must first explain that what it
    buys is not the thing already bought."
T2. A later range note on the same decision, added when a write order field (instance id
    plus transaction number) was put into unit headers: "The unit header's write order
    buys whether this unit was published and which timeline it belongs to. It is not
    meant for telling stale copies apart. The field the decision rejected is still not
    done."
T3. The settled criteria for any header field, all three required: 1) its update cycle
    is bound to the content's change cycle; 2) without it, some specific recovery or
    judgement goes from decidable to undecidable; 3) its net gain over existing
    mechanisms; a field overlapping an existing mechanism has near-zero marginal gain.
T4. The verdict's own reasoning for code 1: adding one more sequence number on code 1
    buys what has already been bought (citing the range note in T1), because the code 1
    write order is already unique per data unit.

THE HITS UNDER REVIEW

H-A (round 1). Because K adds a birth sequence to code 2 and code 3 headers for a new
use, K must go through T1 (reopen it, or add a range note like T2). P and R add no
sequence number, so they never touch T1. So the set of settled texts K must change is
not a subset of what P must change, nor of what R must change; K no longer dominates
P and R.

H-B (round 1). A packed container slot must carry the object's key. If the class tag of
a key is read from the unit header, a packed object lives in the container unit whose
header class is c, not 1, so the slot key would say c while referrers hold 1. Tightening
adopted: the class tag is the class of the named object; slot objects are always code 1.

H-C (round 2). The code 1 half of K relies on "one transaction writes at most one data
unit's user data". If preallocated zero units are read as not user data, one transaction
can write several code 1 units with identical key fields. Tightening adopted: read that
sentence as covering every code 1 unit.

TASK

For H-A: try to show it does not land. Either show that T1 and T2 do not cover a birth
sequence that is only a key segment, or show that P or R must also change T1 or another
settled text of the same rank that K does not change (for example through T3). Give the
reading and why it is the better reading of the quoted texts.
For H-B and H-C: try to show each tightening is not needed (the weakest reading is not
permitted by the texts), or that the same problem also hits P and R.

If you cannot defend a hit, say exactly what you tried and why it failed. Do not stop
after the first attempt.
