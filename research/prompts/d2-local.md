You are reviewing a design inference for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 600 words.

Two rulings by the project owner. Do not argue with them. Reason on top of them.
Ruling one: being cheaper in space than other filesystems is not a criterion. Saving space is not a goal.
Ruling two: unit geometry is computed dynamically, but computed once at mount time. It must never be
recomputed per operation. Each geometry check returns mount or refuse-to-mount, never a slowdown.
The inputs to that computation must be quantities that cannot change during an operation: values
declared in the superblock, unit and node sizes fixed by the format, and the physical block size
probed at mount.

Existing decision about striping, already settled: stripe width is variable, every write is a full
stripe write, and read-modify-write never happens, so there is no write hole.

Two open questions in that striping decision, both never written down:

Open question one, the granularity of variable width. Quoting it: does every single write possibly
use a different width, or is the width fixed inside one allocation group and only changes between
groups? The two readings force different on-disk formats. The first requires every physical location
to be listed one by one, each carrying a device identity. The second can use a logical address plus
a chunk mapping table, and then pointers do not need a device identity.

Open question two, the assumption about device heterogeneity: must all disks be the same size? The
decision notes that a five-year silent data corruption bug in Linux md raid0 was triggered exactly by
disks of unequal size.

A related settled decision: pointer location entries do carry a device identity. That decision also
records that carrying a device identity works under both granularity readings, so it did not settle
open question one.

The inference under attack, call it S: ruling two settles both open questions.

Your task, in this order:

1. Give concrete counterexamples to S for open question one. A counterexample is a specific
situation where the stripe width must depend on something that is not known at mount time, or where
mount-time geometry leaves the granularity question exactly as open as before.
2. Give concrete counterexamples to S for open question two.
3. Name every input to a stripe width decision that can change while the filesystem is mounted. For
each, say what happens to a geometry computed once at mount when that input changes.
4. State whether ruling two is even compatible with the settled part of the striping decision, that
every write is a full stripe write with variable width.
5. Accept S, reject S, or accept a narrowed version, and give the narrowed version if that is your
answer.
