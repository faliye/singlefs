You are the counterexample leg of a design review for a from-scratch copy on write
filesystem. Find concrete failures. Do not use any markdown emphasis. No bold, no
italics, no asterisks. Answer in English. Number your findings.

SETTING. Every unit on disk is exactly 32768 bytes including its header, so a 512 byte
object occupies a whole unit today. A background compactor moves small objects into
shared 32768 byte containers. Each slot inside a container carries the object plus 43
bytes of self description, of which 33 bytes are the object's logical identity five
tuple. That five tuple is also the key of the central mapping, which is the single
entry point for dereferencing an object and for deciding whether space can be freed.
The central mapping value is a location entry of 14 bytes: device 4, physical offset 6
encoded as a 16 KiB slot number, ciphertext checksum 4.

To read a packed object: look up the five tuple in the central mapping, get the location
entry, read the whole 32768 byte container, then find the slot whose five tuple matches.

TWO HARD RULES the owner has just set:
R1. A slot is copy on write. No in place modification of any slot in a container that
has already been written. If any part of the design requires in place modification,
the whole feature is abandoned.
R2. The feature exists only to reclaim the space wasted by many consecutive small
objects. It must not change the dereference chain and must not introduce a more
complex ring or tree. Any new tree or index must first be proven necessary.

Consequences already accepted: a container is immutable once written. Modifying a
packed object writes a brand new independent unit through the unchanged write path and
leaves the old slot dead in place. Reclaiming dead slots means writing a whole new
container and releasing the old one; never compacting in place.

THE OPEN QUESTION. Under R1 and R2, is any new structure needed at all?
Candidate B says no: there is no container index and no live slot count. When the
compactor next walks the central mapping in key order, it discovers which slots are
still referenced and rewrites containers accordingly.

YOUR TASK. Give concrete operation sequences and on disk states, not opinions.

Q1. Under candidate B, construct a sequence where a container ends up with zero live
slots and no entry anywhere on disk refers to it. Say exactly what still records that
those 32768 bytes are allocated, and how they are ever returned to free space. If they
cannot be, that is a space leak: state its size and rate for a workload you specify.

Q2. Under candidate B, construct a crash point during compaction where the disk holds
both the old container and the new container, and say how a scanner reading raw disk
decides which one is current. State what information it uses and where that information
physically lives.

Q3. Encryption is on. A container holds 58 objects. The message authentication code for
a block lives in the pointer header, one per logical extent. There are 58 central
mapping entries pointing at one container. Construct the concrete mismatch that results,
and say what would have to change to avoid it.

Q4. Argue whether the step "read the container then find the slot whose five tuple
matches" is an extra step in the dereference chain compared to reading a plain unit.
Give the concrete operations in both cases side by side and count them.

Q5. Name every construction you tried that did not work and say why it failed.
