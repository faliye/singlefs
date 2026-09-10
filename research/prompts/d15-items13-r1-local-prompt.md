You are attacking two decisions about the format-freeze machinery of a filesystem project. Find
concrete counterexamples. Do not summarize, do not agree, do not restate the proposals back at me.

Do not use any markdown emphasis anywhere in your answer. No bold, no italics, no asterisks.
Write plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem is in the format design stage. There is no code yet and no
on-disk format yet. Design decisions are numbered and live in a knowledge base of markdown files.
Each decision has sub-items, and each sub-item is either settled or open.

Freezing the format is defined as committing a file that names a component, a version, a commit id,
and a spec hash. When a commit touches that directory the gate switches to a stricter set of
pre-conditions and refuses the commit unless all of them hold. A human decides whether to file a
freeze request; the machine decides whether the request is admissible.

There is a pre-freeze checklist with seven entries. Entry seven says: six named design decisions
must each either become settled, or be explicitly recorded as "version 1 does not contain this
feature, the bytes are left blank with no semantics assigned, deferred to version 2 behind an
incompatible feature bit". If bytes are reserved, the width must also be decided.

The format is frozen in four layers with a strict dependency order: layer 1 superblock and block
header, layer 2 pointer field layout, layer 3 key encodings, layer 4 index node internal layout.

QUESTION ONE. What is the disposition of checklist entry seven?

Facts about the history of this open item, all verified today:

1. The item has been re-checked four times in five weeks. Every re-check consisted of a human
   reading the current status of six other decisions and copying a summary of them into this item's
   text.
2. One of those copies copied a value that had been overturned earlier the same day: it recorded an
   inline threshold as 512 bytes when the value had already been changed to 0. Both places were
   labelled settled. That incident created a standing unpaid check in the project's owed-checks
   list, about settled values forking across the knowledge base.
3. The item's own enumeration disagreed with the checklist it is supposed to serve: the checklist
   named six decisions, the item listed four, for an unknown length of time. That was repaired
   twenty three days later.
4. Five of the six cells are now answered. One is still open: the form of a structural anti-aging
   mechanism, which would change index node layout. A user ruling three days ago said that whole
   item waits for an experiment that has not been run.
5. A separate settled decision says the index (the trees) is derived state, can always be rebuilt
   from authoritative state, and does not participate in format freezing at all. That decision
   explicitly filed a request that the four-layer freeze diagram be rearranged so that derived state
   leaves the freeze entirely. The four-layer diagram has never been changed. This contradiction is
   a known unpaid debt in the owed-checks list.

The candidate answer, arm B: delete the list of six from the freeze policy. Replace it with a rule.
The rule is: at the moment a freeze request is filed, every still-open sub-item anywhere in the
knowledge base must carry an explicit verdict of the form "does this touch the layers being frozen
in this request". If any open sub-item carries "yes", or carries no verdict at all, the freeze
request is refused. Separately, the default reserved width is zero, and the burden of proof is on
whoever wants a nonzero width: they must show that the reserved bytes convert "used it wrongly"
into "cannot mount". A reserved but never set bit prevents no misuse, because a version 2 that uses
the feature will change the layout anyway and will carry its own incompatible bit, so a version 1
reader could not have mounted it regardless.

Competing arms: arm A is to decide the last cell now by fiat, which overrules the user ruling.
Arm C is to keep the item open and keep doing manual re-checks. Arm D is arm B but the list of six
stays in the text as navigation, marked non-authoritative.

QUESTION TWO. What artifact should the spec hash be computed over?

Facts, all verified today:

6. The current text says: for now the spec is a machine-readable toml file per frozen component;
   once there is code, switch to hashing the definition of the corresponding Rust struct. The only
   thing recorded as open is when to switch.
7. A later user ruling settled that the checker and the implementation share exactly one artifact: a
   constants module generated from the field tables in the knowledge base. The generator reads from
   the knowledge base. Neither side may hand-edit it. Everything else, including format parsing and
   checksum implementations, is written twice independently, on purpose, so that the auditor and the
   audited do not share code.
8. Another settled decision says the conformance suite for third-party pipelines is published as
   source code plus a judging program, so that a third party can check the judgment itself rather
   than being told it is compliant.
9. Two freeze components exist that are outside the four layers. Neither has a spec file today. One
   of them freezes at the instant the first image containing a certain container type is written.

The candidate answer, arm B: never switch. The spec hash is always computed over the spec artifact
generated from the knowledge base field tables. When code appears, what gets added is not a change
of hash input but a projection check plus the golden-image regression.

Competing arms: arm A is the original plan, switch to hashing Rust struct definitions.
Arm C is to keep two hashes, one over the spec and one over the code, and require both to change
together. Arm D is to anchor the hash on the bytes of an archived golden image instead of on any
declaration.

YOUR TASK

1. Attack arm B of question one. Construct a concrete scenario in which replacing the list of six
   with the rule lets something through the freeze that the list would have caught. Be specific
   about which object escapes and why the rule does not see it.

2. Attack the default-width-zero rule. Give a concrete class of format element for which reserving
   zero bytes today is provably worse than reserving some, and explain what the reserved bytes buy
   that is not merely performance.

3. Attack arm B of question two. Give a concrete way in which the spec hash stays constant while
   the bytes actually written to disk change. Then say whether the projection check and the
   golden-image regression close that hole or not, and where exactly they fail.

4. Make the strongest possible case FOR arm A of question two, hashing the Rust struct definitions.
   Do not hedge. If there is a class of error that only a code-side hash catches, name it.

5. Point out any contradiction among the numbered facts above, or any place where a fact is being
   used to support a conclusion it cannot reach.

Answer in English.
