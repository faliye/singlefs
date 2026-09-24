SINGLEFS GATE-FIX-FORKS - LOCAL ATTACK LEG - ROUND gate-fix-forks-r1

You are one of three independent reviewers checking a batch of automated
repository checks (called gate stages below) for singlefs, a copy on write
filesystem being designed from scratch. You are given a self contained set
of facts below, gathered today by directly reading the source files named
in each fact. Do not assume anything not stated here. Read carefully,
every qualifier in this text is load bearing, a definition with one word
removed can flip an answer. Do not use any markdown emphasis anywhere in
your answer: no bold text, no italic text, no backtick code formatting, no
asterisk bullets. Number your answers to match the question numbers given
at the end. For every numbered answer you give, add one sentence starting
with "This would be refuted by:" describing the specific observation that
would prove that answer wrong. Where a question asks you to fill in a
table, give the table with an explicit value and an explicit one sentence
justification for every single cell, not just "yes" or "no" for the whole
table, and do not merge cells together even where you believe several
rows behave identically: each cell must carry its own explicit verdict
and its own quoted or explained justification. When your answer needs to
point at a source, point at it by the fact label given below (for example
fact 4-2), by the row label given below (for example R7), or by naming
the constant name or binary name given below. Do not write any source
file name together with a line number, and do not write any line number
at all, anywhere in your answer, whether the number came from this prompt
or from your own count of something. If you need to refer to a specific
place in a file, name the function, the constant, or the binary instead.

This prompt covers five independent items, called item T4, item T7, item
T8, item T9, and item T10. They do not share any fact or any table; treat
each one as a separate task and do not let a conclusion from one item
carry over into another.

ITEM T4.

Background for item T4. One gate stage, called stage 80 below, checks
whether experiment binaries carry an assertion that pins a value to a
literal number, rather than only comparing several computed values
against each other. The shared project rule this stage exists to
enforce, quoted verbatim in translation, in full, section title
included: "Only letting multiple arms compare against each other cannot
catch all the arms being wrong together. Arm to arm equality, as a test,
is a very weak one: it can only rule out one single arm being wrong on
its own; it cannot rule out all the arms sharing the same formula, with
that formula itself being wrong. In a cross comparison, the second case
looks exactly like everything being correct: every arm still comes out
equal. So: every time you write an assertion that compares arms against
each other, place beside it an assertion that pins an absolute value.
'The three arms' cost is equal' is not enough; you also need 'the cost
equals exactly N', where N is computed separately. In practice: after
writing a cross comparison assertion, ask yourself one question, if all
three arms were wrong together, who would notice. If you cannot answer
that, add an absolute value assertion. This rule, and a separate rule
that positive controls must run against every single arm, are two sides
of one thing: that separate rule governs every arm having to pass
through the gate; this rule governs the gate itself not being allowed to
be only relative." A binary counts as covered, for stage 80's purposes,
when at least one line in its own source file matches this exact pattern
(given here as a regular expression, not as prose):
assert_eq!\(text, -?digits\) OR assert!\(text comparison-operator -?digits\).

Fact 4-1. Stage 80's own scope line, read directly from its source, sets
BINS to research/e7-index-bench/src/bin. Right after checking that
directory, stage 80 only checks directories named crates/*/src/bin and
research/*/src/bin to print, on its own success line, how many .rs files
each such directory holds; it does not check any assertion inside those
files at all. Today, exactly two such directories exist: crates/*/src/bin
resolves to one directory, crates/singlefs-harness/src/bin, holding 4 .rs
files; research/*/src/bin resolves to one directory,
research/e7-index-bench/src/bin, holding 145 .rs files (this is the same
directory already fully covered by stage 80's main check, so it is not
printed a second time).

Fact 4-2. The four .rs files under crates/singlefs-harness/src/bin are
named, and referred to below by these short names: binary B1 is
e156_allocation_basis_counts.rs; binary B2 is
first_transaction_device_log_check.rs; binary B3 is
first_transaction_on_device.rs; binary B4 is
first_transaction_region_bytes.rs.

Fact 4-3. The first sentence of each binary's own module level doc
comment, translated:
B1: "E156 stage one: the cost numbers for the four forks of alloc-basis
(pre-registered before the run)."
B2: "Host side: take the two device side logs that a QEMU tool recorded,
and compare them item by item, in device order, against the record
stream obtained by replaying the same write path on the host with the
same parameters and the same bytes."
B3: "VM tier: run the entire write path of the first transaction on two
real virtio disks, then cold reboot and recover, reading the file back."
B4: "The implementation side of one measurement's dry run: run a fixed
scenario function on two in memory disks with the same parameters as the
VM tier, and print, as one result line per region, the bytes that the
first transaction wrote to 21 fixed regions."

Fact 4-4. Whether a driver function inside a shared replay script
(research/scripts/replay.sh) invokes each binary directly by name:
B1: yes, a function named driver_e156 runs this binary directly through
cargo, with no companion binary in any other directory.
B2: no driver function in that script names this binary anywhere; the
binary's own doc comment describes it as meant to be run by hand, taking
two log file paths and several device size numbers as command line
arguments.
B3: no driver function in that script names this binary; two other shell
scripts (one that stages a VM root filesystem, one that runs the VM)
install and invoke this binary inside a virtual machine, for a milestone
that has not been run yet this round.
B4: a function named driver_e142 runs this binary directly through cargo,
captures its stdout into a temporary file, then passes that temporary
file's path as the first command line argument to a second, separately
compiled binary that lives under research/e7-index-bench/src/bin (the
directory already covered by stage 80's main check today).

Fact 4-5. Whether result files tied to each binary currently exist under
research/results/, and stage 80's own current count of assertions
matching the pattern given in the background paragraph above, counted
today directly against each binary's own source file:
B1: three result files exist, produced by driver_e156, holding this
binary's own direct output. Current assertion count: 1.
B2: no result file under research/results/ is tied to this binary's own
output under any name. Current assertion count: 0.
B3: no result file under research/results/ is tied to this binary's own
output under any name; the milestone that would produce one has not been
run yet this round. Current assertion count: 4.
B4: no result file under research/results/ holds this binary's own raw
output under its own name; two result files exist that were produced by
the second, separately compiled binary named in fact 4-4, and those two
files begin by printing this binary's own output verbatim, then add
further lines produced by that second binary comparing it against a real
device. Current assertion count: 0.

Fact 4-6. Two candidate policies for stage 80's scope.
Candidate KEEP: leave stage 80 exactly as described in fact 4-1: only
research/e7-index-bench/src/bin is checked for the assertion pattern;
every other directory holding experiment binaries is only named and
counted on the success line, never checked for the assertion pattern.
Candidate EXTEND: widen stage 80 so that it also checks the assertion
pattern inside crates/*/src/bin and research/*/src/bin; add alongside it
an exemption table, one row per binary that is deliberately excluded from
this check, each row required to give a reason and to name a path that
must exist; a row naming a path that does not exist makes that row itself
fail the check.

Table task for item T4. For each of B1, B2, B3, B4, do three things.
First, state whether you judge it to be an experiment binary or an
apparatus and tooling component, as those two roles are used in the
background paragraph's quoted rule and in facts 4-3 and 4-4, and give one
sentence of justification grounded only in facts 4-2 through 4-5. Second,
state whether, on the basis of the assertion count given in fact 4-5
alone, this binary would make stage 80 fail red today if candidate EXTEND
were adopted with zero exemption table rows filed for it. Third, given
your own answer to the first part of this row, state whether this binary
would need its own exemption table row under candidate EXTEND, or whether
your classification already places it outside candidate EXTEND's subject
matter entirely; name which of these two you mean.

Questions for item T4.

1. Give the filled table for B1 through B4, following the table task
above exactly, one block per binary.

2. Counting only from the assertion counts given in fact 4-5, how many of
B1 through B4 would make stage 80 fail red today under candidate EXTEND
with zero exemption table rows filed? Give the number and name which
binaries by their short name.

3. Do facts 4-1 through 4-5 already decide, by themselves, whether B2 and
B4 should be classified as an experiment binary or as an apparatus and
tooling component, or is this classification a judgment call that the
facts leave open? If you say the facts decide it, quote the exact phrase
from the relevant fact that your verdict rests on, for each of B2 and B4
separately. If you say it is left open, state explicitly what additional
fact would need to exist, but does not exist here, for you to decide.

ITEM T7.

Background for item T7. singlefs has roughly ninety gate stages, run in
sequence by a shared runner script. That runner's contract, unchanged by
this round: exit code 0 means the stage passed; exit code 77 means "not
run this time" and counts neither as a pass nor as coverage of anything;
any other exit code means the stage failed. Every stage that exits 77 has
its line in the final summary printed as the exact same fixed sentence:
"not run this time: this stage reported there was nothing to judge this
round (exit code 77), see its own output above for why."

Fact 7-1, quoted verbatim in translation from the header comment of one
particular gate stage (called stage 70 below, which re-verifies citations
against external source trees such as kernel source and other
filesystems' source): "Even when the source tree is missing, this stage
still fails red; it must not be treated as a skip, because skipping is
exactly the behavior that lets an earlier batch of citations evaporate
without a trace." The same stage's own comment states its scope directly
above this sentence: it is talking about the external source trees it
reads citations against, not about its own supporting scripts.

Fact 7-2, a table of twenty rows, each row describing one place in the
gate stages where something is missing on disk today, grouped into four
categories already assigned by the person who prepared this material for
you; this categorization was not derived from any of the facts above and
should be checked, not trusted, when you answer the table task below.
Columns are: row label, which gate stage, what is missing, the assigned
category, the exit code this row's stage currently returns when that
thing is missing, and a translated quote of what the stage itself prints
when that thing is missing.

Category "the same generator script is missing" (this category has two
rows, and both rows name the exact same file path, confirmed identical by
reading both stages' own source):
R1: stage 21. Missing: a Python script under .claude/scripts/ that
generates a decision item listing. Exit code: 1. Quote: "can't find the
generator $GEN" together with "it lives under .claude/scripts/ in this
same repository, right alongside the gate stages; without it this stage
can't judge anything at all, recover it from git".
R2: stage 31. Missing: the exact same script path as R1. Exit code: 77.
Quote: "can't find $GEN, this stage is skipped".

Category "a script that this stage's own checking logic lives inside is
missing" (five rows):
R3: stage 57. Missing: a shell script under .claude/scripts/ holding this
stage's own memory model checking logic. Exit code: 1. Quote: "can't find
$LKMM" together with "it travels with the repository (under
.claude/scripts/); if it was deleted, recover it from git".
R4: stage 70. Missing: a shell script under research/scripts/ that this
stage calls to verify citations one by one. Exit code: 1. Quote: "can't
find $S" together with "this stage relies on it to check every citation
one by one; if it moved, change the path written into this stage; if it
was never written, write it first, because without it not one citation
has ever actually been checked".
R5: stage 73. Missing: a shared lint script that this stage forwards to.
Exit code: 1. Quote: "can't find the shared script $script" together with
"the shared copy of project rules is not installed here, fetch it and run
its own install script".
R6: stage 77, as of this round's own uncommitted edit to it (fact 7-4
below gives the exact edit). Missing: a Python script under
research/scripts/ that checks whether the test machine is left clean
(and no recorded fallback log file either). Exit code: 1 as of this
round's uncommitted edit; it was 77 immediately before this round's edit.
Quote (this round's new wording): "there is no $SCRIPT: it travels with
the repository, its absence means it was deleted or moved, and nobody is
checking whether the machine is clean this round" together with "recover
it from git; if it moved, change the path written into this stage".
R7: stage 52. Missing: a Python script under research/scripts/ that this
stage forwards essentially its entire check to. Exit code: 77. Quote:
"can't find research/scripts/check-segment-registry.py, this stage is
skipped".

Category "a kb directory or a kb index file is missing" (eleven rows, all
exit code 77):
R8: stage 21, its second check (a different check inside the same file as
R1). Missing: the decisions index file under .claude/kb/. Quote: "can't
find $IDX, this stage has nothing to judge".
R9: stage 24. Missing: a decisions directory under .claude/kb/. Quote:
"can't find $DEC, this stage is skipped".
R10: stage 27. Missing: the .claude/kb directory itself. Quote: "can't
find kb, this stage is skipped".
R11: stage 34. Missing: an index file, or an experiments directory, under
.claude/kb/ (either one missing triggers this same row). Quote: "can't
find $IDX or $EXP, this stage is skipped".
R12: stage 37. Missing: the decisions index file under .claude/kb/.
Quote: "can't find $IDX, this stage is skipped".
R13: stage 40. Missing: one experiment's body file under .claude/kb/ (an
empty file also counts as missing here). Quote: "can't find the
experiment body, this stage is skipped".
R14: stage 42. Missing: any one of three specific kb paths. Quote: "can't
find $FTL / $MS / $DEC, this stage is skipped".
R15: stage 76. Missing: any one of three specific kb paths. Quote: "can't
find $L1 / $L2 / $MS, this stage has nothing to judge".
R16: stage 85. Missing: an experiments directory under .claude/kb/.
Quote: "can't find $EXP_DIR, this stage is skipped".
R17: stage 86. Missing: an experiments directory under .claude/kb/.
Quote: "can't find $EXP_DIR, this stage is skipped".
R18: stage 88. Missing: the .claude/kb directory itself. Quote: "can't
find .claude/kb, this stage has nothing to judge".

Category "not inside a git repository at all" (two rows, both exit 77):
R19: stage 60. Quote: "not inside a git repository, this stage is
skipped".
R20: stage 61. Quote: "not inside a git repository, this stage is
skipped".

Fact 7-3. R1 and R2, read directly from both stages' own source today,
construct the exact same absolute path for the file each one calls
missing: both build it as the repository root joined with
".claude/scripts/gen-decision-items.py". This is not an approximation;
both source lines were read and compared character by character today.

Fact 7-4, the exact diff this round has already made, uncommitted, to
stage 77, translated. Before this round's edit, on the thing named in R6
being missing, the stage printed "no $SCRIPT, this stage is skipped" and
returned exit code 77. This round's uncommitted edit changed the printed
message to the R6 quote given above, changed the exit code to 1, and
separately added one new header comment line to the same stage, reading,
translated: "with the script or the recorded fallback log both missing:
fail red, do not treat it as a skip, because it travels with the
repository, and its absence is exactly a case of something that should
be in the repository not being there, the same rule as stages 57, 70, and
73."

Two candidate conventions for item T7.
Candidate UNIFORM-1: change every one of R1 through R20 so that its stage
returns exit code 1 when the thing named in that row is missing.
Candidate UNIFORM-77: change every one of R1 through R20 so that its
stage returns exit code 77 when the thing named in that row is missing.

Table task for item T7. For each of R1 through R20, do two things. First,
classify it as either MISSING-FROM-REPO, meaning the missing thing is
something that should be present in this repository, or NOTHING-TO-JUDGE,
meaning the missing thing genuinely means there is nothing for this stage
to judge this particular round; give one sentence of justification
grounded in fact 7-1, fact 7-2's own description of that row, and fact
7-4, and say explicitly if the row's own description alone does not let
you decide between the two, rather than guessing. Second, only for rows
you classified as MISSING-FROM-REPO, state whether that row's current exit
code (as given in fact 7-2) is consistent with, or in conflict with, the
literal wording of fact 7-1; for rows classified as NOTHING-TO-JUDGE,
answer "not applicable" for this second part.

Questions for item T7.

4. Give the filled table for R1 through R20, following the table task
above exactly, one block per row.

5. Under candidate UNIFORM-1, how many of the twenty rows change away
from their current exit code as given in fact 7-2? Give the number, then
for each row you classified MISSING-FROM-REPO whose exit code changes
under this candidate, state whether its new exit code (1) is consistent
with or in conflict with fact 7-1, and list those rows by row label.

6. Under candidate UNIFORM-77, how many of the twenty rows change away
from their current exit code as given in fact 7-2? Give the number, then
for each row you classified MISSING-FROM-REPO whose exit code changes
under this candidate, state whether its new exit code (77) is consistent
with or in conflict with fact 7-1, and list those rows by row label.

7. R1 and R2 name the exact same missing file, confirmed identical in
fact 7-3, yet carry two different exit codes today, 1 and 77. Using only
facts 7-1 through 7-4, does the text already decide which of the two
exit codes is the correct one for this one dependency, or does it leave
this open? If the text decides it, quote the exact phrase your verdict
rests on. If it does not decide it, state explicitly what fact would need
to exist, but does not exist here, for you to decide.

ITEM T8.

Background for item T8. A Python generator script builds a listing of
decision items from source markdown files. When a decision item's name
would print too long, a function called clip cuts it down. This item is
about a bug in an earlier version of clip, a fix already applied this
round to the generator script's own source code, and a candidate for
going further than that fix.

Fact 8-1, the clip function's logic, read directly from its current,
already fixed, source code today. clip(text, n) first cuts text down to
at most n characters. It then keeps trimming characters off the end,
back to the position of the last opening full width parenthesis
character, for as long as the result contains more opening full width
parentheses than closing full width parentheses (this prevents a cut
landing inside a parenthetical annotation and leaving it unclosed).
After that parenthesis balancing step, the function compares its current
result against the original, untouched, input text. If they are equal,
this means the character limit cut, and the parenthesis balancing step,
neither one actually removed anything: no real truncation happened, and
the function returns immediately, only trimming trailing whitespace,
without running the next step described below at all. Only when the
result differs from the original text, meaning real truncation did
happen, does the function go on to repeatedly apply two more patterns to
the result, alternating between them, until neither changes anything:
first, a pattern matching a trailing bare reference anchored to the end
of the string (one capital letter, an optional hyphen, one or more
digits, an optional decimal part, and optional trailing whitespace);
second, a pattern matching one or
more trailing characters, anchored to the end of the string, drawn from a
fixed set that includes whitespace, a forward slash, any one of three
different comma characters (an ASCII half width comma, a full width
Chinese comma, and the separate ideographic enumeration comma), a middle
dot, and three specific Chinese connector characters that function like
the English words "of", "and", and "with" attached to the end of a
Chinese noun phrase.

Fact 8-2, an already committed, real example of the bug that fact 8-1's
fix addresses, reproduced today by running both the old and the new
version of clip against the same real input string taken verbatim from a
committed decision file. The real input string, an item name, is twelve
characters long, far under any length limit this generator ever uses (the
generator's own limit for this kind of name is 46 characters). Because
the input is so far under the limit, the character limit cut in the first
step of clip changes nothing, and the parenthesis balancing step also
changes nothing, so under the fixed version of clip, described in fact
8-1, the function detects that no real truncation happened and returns
the input completely unchanged. Under the old, unfixed version of clip,
which ran the two alternating patterns unconditionally regardless of
whether truncation had actually happened, the second pattern's trailing
connector character class matched and removed the input's own last
character, a single Chinese particle functioning as a possessive or
nominalizing suffix, that was not part of any reference or citation at
all. This is not a hypothetical: the file committed in the repository
today, generated by the old, unfixed version of clip before this round's
fix was written, still shows the result with that one trailing character
missing; the source markdown file that the generator reads from still
has that trailing character present. The generator has not been rerun to
regenerate that already committed file since this round's fix was
written.

Fact 8-3, a second, separately reproduced case, this time testing whether
the fix described in fact 8-1 also protects a domain specific abbreviated
term, such as the name of a hash algorithm, from being damaged, when that
term sits right at the character limit boundary. A synthetic item name
was built today so that the plain character limit cut in the very first
step of clip, before either of the two alternating patterns ever runs,
lands in the middle of a six character algorithm name, cutting it down to
its first two characters. This was verified today by actually running the
fixed clip function, described in fact 8-1, against this synthetic input:
the six character algorithm name in the input came out as only its first
two characters in the output. The reason is that the character limit cut
itself, which always runs unconditionally as clip's very first step, does
not know where any word boundary is; it can land inside any word,
including inside a domain specific abbreviated term, before either
alternating pattern in fact 8-1 ever gets a chance to run, and neither
alternating pattern's job is to detect or repair a cut that already
landed in the middle of a term.

Fact 8-4. Across the kb directory today, one particular marker line
appears in twenty one different files; the marker's own format, quoted
verbatim from one file that carries it: "<!-- doc-lint:not-numbers L1 L2
L3 AES-256 AES256 SHA256 SHA-256 SHA512 CRC32 CRC32C CRC64 RAID5 RAID6
OCFS2 Z3 T10 M1 M2 M3 M4 M5 M6 M7 M8 M9 M12 M13 M25 M26 F41 -->". This
marker line exists today for a separate checking tool, unrelated to the
clip function, that would otherwise mistake these listed tokens for
numbered cross references.

Two candidates for item T8.
Candidate KEEP: leave clip exactly as described in fact 8-1; do not add
any vocabulary list.
Candidate VOCAB: in addition to fact 8-1's logic, also read the kb's
doc-lint:not-numbers marker lines (fact 8-4) as a vocabulary list; when
the text clip is about to strip from the end matches one of these listed
tokens exactly, skip stripping it, even though real truncation did
happen. This candidate would make clip's correctness depend on the
doc-lint marker line's own format continuing to hold.

Table task for item T8. Consider the two reproduced cases, fact 8-2 and
fact 8-3, as two separate rows. For each row, and for each of the two
candidates, KEEP and VOCAB, state whether that candidate would have
prevented that row's specific character loss, and give one sentence of
justification grounded only in facts 8-1 through 8-4; if your answer
depends on whether the exact damaged word happens to appear, verbatim, as
one of the twenty one files' listed tokens today, say so explicitly
instead of assuming it does or does not.

Questions for item T8.

8. Give the filled table for fact 8-2's row and fact 8-3's row, crossed
with candidates KEEP and VOCAB, four cells in total, following the table
task above exactly.

9. Fact 8-3 describes a character loss that happens inside clip's very
first step, before either of the two alternating patterns described in
fact 8-1 ever runs. Does candidate VOCAB, exactly as defined above, act
on anything before or during that first step, or does it only act on the
alternating patterns that come after it? Answer using only the definition
of candidate VOCAB given above, and add the required "This would be
refuted by:" sentence.

10. Fact 8-4 states this marker line appears in twenty one files today.
Does this number, by itself, tell you anything about how completely
candidate VOCAB's vocabulary list would cover every domain specific term
that could be damaged by the failure mode shown in fact 8-3, across the
whole kb directory, or does it only tell you how many files currently use
this marker for the unrelated tool named in fact 8-4? Answer using only
facts 8-1 through 8-4, and add the required "This would be refuted by:"
sentence.

ITEM T9.

Background for item T9. singlefs keeps a change history for its design
decisions. Each change history entry lives under a dated file, and names,
in its own text, which decision or decisions it changed. A generator
script reads every change history entry, and, for each decision, collects
every entry that names it, to build a summary page. This item is about
what happens to an entry that names a decision whose own body markdown
file cannot currently be found.

Fact 9-1. The generator script first builds a list of currently valid
decision numbers, by reading, for every markdown file under one fixed
decisions directory, only that file's own very first line, and keeping
the file's number only if that first line matches this exact pattern
(given here as a regular expression): ^## D(\d+) (.+?) —— (this requires
a literal two dash separator right after the decision's short name,
immediately following the number and the name, on the file's first
line). Read fresh today, this list contains the decision numbers 1
through 28, every one of them, with no gaps and no numbers outside this
range.

Fact 9-2. Separately, for each change history entry, the generator
extracts every decision number the entry names, by scanning the entry's
own text (its own title, or, when it has no title, its own quick note
for what changed, plus its own quick notes for what came before and what
came after) for this exact
pattern (given here as a regular expression): (?<![\w-])D(\d+)（ (this
requires the digits to be immediately followed by a literal full width
opening parenthesis character, with no digit, letter, or hyphen
immediately before the D). This was tested today against four example
strings. The string "see D8（core index structure） settled item 3"
matches, extracting the number 8. The string "see D8: core index
structure" (a colon instead of a full width parenthesis right after the
digit) does not match at all, extracting nothing. The string "see D08
（core index structure）" (a leading zero, still followed immediately by
a full width parenthesis) matches, extracting the number 8, because the
generator converts the matched digits to a plain integer before using
them. The string "D8 already has this" (a half width space instead of a
full width parenthesis right after the digit) does not match at all,
extracting nothing.

Fact 9-3, the generator's own current placement logic, read directly from
its source code today. For every decision number found in fact 9-1's
list, the generator collects every change history entry whose own set of
extracted numbers, per fact 9-2, contains that decision number, and marks
each such entry as placed. Separately, the generator collects, into a
section titled "not attached to any decision", every entry whose own
extracted number set, per fact 9-2, is completely empty, and marks each
such entry as placed too. After both of these passes, if the total count
of entries marked placed is not equal to the total count of all entries
that exist, the generator raises an exception and the check fails red,
printing every entry that ended up in neither category. The generator's
own printed guidance in that case, translated: "check the number against
the source text: if the quick note or the title names the wrong decision,
fix it and rerun; if the number was not written wrong, and that
decision's body genuinely is not under the decisions directory (its first
line is unreadable, it was deleted, or it was merged into another
decision), find out first where the body went and report that upward;
there is not yet any rule for which section such an entry should land in,
do not edit history entries just to make this check pass."

Fact 9-4. The generator's logic, as described in facts 9-1 through 9-3,
has no memory of which decision numbers existed on any day other than
today: fact 9-1's list is rebuilt fresh, every run, only from whatever
decision body files currently exist under the fixed decisions directory
today. Nothing in facts 9-1 through 9-3 reads or consults any change
history file, any git log, or any other record of which decision numbers
were ever valid in the past.

Four example change history entries for item T9, each considered on its
own, one entry at a time, each with a single extracted-number outcome
already worked out for you from fact 9-2's pattern.
S1: an entry whose text names D8. Per fact 9-2, its extracted number set
is {8}. Per fact 9-1, 8 is in today's valid list.
S2: an entry whose text names D99. Per fact 9-2, its extracted number set
is {99}. Per fact 9-1, 99 is not in today's valid list (today's valid
list runs 1 through 28 only).
S3: an entry whose text names no decision at all, using only prose with
no D-number pattern anywhere in it. Per fact 9-2, its extracted number
set is the empty set.
S4: an entry whose text mentions the decision using the exact string "D8"
followed immediately by a colon, not by a full width parenthesis (as in
fact 9-2's third tested example). Per fact 9-2, its extracted number set
is the empty set, because the pattern requires the full width parenthesis
immediately after the digits.

Two candidates for item T9, for the specific situation shown by S2: an
entry names a decision number that is not in today's valid list.
Candidate FAIL-LOUD: this is fact 9-3 exactly as already described: such
an entry ends up in neither category, the total counts do not match, and
the whole check fails red, printing that entry by name.
Candidate NEW-SECTION: instead of failing red, place such an entry into a
new section, titled something like "decisions whose body is no longer
under the decisions directory", and let the check pass.

Table task for item T9. For each of S1 through S4, and for each of the
two candidates, FAIL-LOUD and NEW-SECTION, state what that candidate does
with that entry: which of the three destinations it reaches (a normal
per-decision section naming a specific number, the "not attached to any
decision" section, or, only for NEW-SECTION, the proposed new section),
or, if it instead makes the whole check fail red, say so instead of
naming a destination. Give one sentence of justification grounded only in
facts 9-1 through 9-3 and this entry's own extracted number set as given
above. Note that FAIL-LOUD and NEW-SECTION are only defined above to
differ from each other on S2; for S1, S3, and S4, state what fact 9-3's
logic, unchanged, does with that entry, and then state explicitly whether
NEW-SECTION's own definition, as given above, gives you any reason to
expect a different outcome for that particular entry.

Questions for item T9.

11. Give the filled table for S1 through S4, crossed with candidates
FAIL-LOUD and NEW-SECTION, eight cells in total, following the table task
above exactly.

12. Consider two different real world causes that could both produce
exactly the same entry text as S2: first, a person genuinely mistyped a
number that was never valid, while meaning to type a different, valid
number; second, a decision that genuinely existed under some number in
the past was later renumbered, merged into another decision, or deleted,
so that number is no longer valid today, even though the entry naming it
was written correctly at the time. Using only facts 9-1 through 9-4,
does the generator's own logic, as described, have access to any
information that would let it tell these two causes apart, for one given
entry, working only from that entry's own text and today's list of valid
decision numbers? Answer using only the facts given above, and if you
say no such information is available, name the specific fact whose
wording establishes that. Add the required "This would be refuted by:"
sentence.

13. Does candidate NEW-SECTION change what happens to S1, S3, or S4,
compared to candidate FAIL-LOUD, or does it only change what happens to
S2? Answer for each of S1, S3, and S4 separately, grounding each answer
in your own table cells from question 11. Add the required "This would be
refuted by:" sentence.

ITEM T10.

Background for item T10. Three gate stages (called stage 27, stage 39,
and stage 92 below) share one Python library for reading a Rust source
file's top level public const declarations. That shared library's own
module docstring, quoted verbatim in translation: "how the value is read
is chosen explicitly by the caller: integer_literal reads an integer
literal (digits, optionally with underscore separators and an integer
type suffix); a value that cannot be read this way, for example an
expression like 16 * 1024, becomes the value None. normalized_text reads
the original text after normalizing whitespace, and is meant only as a
change detector: it never asks what the value equals."

Fact 10-1. Stage 27's own call into this shared library does not pass any
explicit value for this reading choice at all, so it uses the library's
own default, which is integer_literal, per the docstring quoted above.
Stage 27's own call also passes a separate flag requesting only scalar
typed declarations (a Rust type written as a single bare identifier, such
as u64, rather than a compound type such as an array type); it does not
request the separate "only top level pub" flag, so it also picks up const
declarations that are not top level pub. Stage 27 applies this call
across every .rs file found under research/ and crates/, and only checks
a found declaration's value against a separately maintained marker
comment when that declaration's own name already appears among the
marker comments; a declaration whose name never appears in any marker
comment is never looked at by stage 27 at all.

Fact 10-2. Stage 39 does not call this shared library's Rust reading
function at all. It calls a different function in the same shared
library, one that reads a different kind of marker comment embedded
directly inside kb markdown files (not inside any .rs file), and compares
those marker values against numbers added up from field tables written in
the same kb markdown files. The reading choice named in fact 10-1's
docstring (integer_literal versus normalized_text) governs only the
function stage 27 and stage 92 call; it is not a parameter of the
function stage 39 calls, and nothing in stage 39's own source sets it.

Fact 10-3, quoted verbatim in translation from two places in stage 92's
own source. From its header comment: "the .rs side's value is compared as
its whitespace normalized original text (reading choice: normalized
text), unlike stage 27, which requires an integer literal: the format
constant module has derived constants computed from expressions like
DATA_UNIT_HEADER_BYTES + ...; this stage only asks whether that text
changed, not what it equals." From an inline comment right next to where
this reading choice is set in code: "stage 27 reads the .rs value
requiring an integer literal; this stage instead keeps the old
convention of only normalizing whitespace; whether the two should be
unified into one convention is not yet decided; if unified, change this
one setting." Stage 92's own call into the shared library requests only
top level pub declarations, but does not request the separate
"only scalar typed" flag that stage 27 requests, so stage 92's own call
also picks up top level pub declarations whose type is a compound type,
such as an array type, not only single identifier scalar types.

Fact 10-4, counted today by actually running stage 92's exact function
call (top level pub declarations, no scalar-only restriction) against the
one Rust source file this stage's registered layouts point at
(crates/singlefs-format/src/lib.rs), under both reading choices. Under
normalized_text, every one of this call's declarations returns some
non-empty text value; none of them return the value None. Under
integer_literal, this call returns 72 declarations in total; 15 of these
72 return the value None, because they cannot be read as an integer
literal. Of these 15: 14 of them have a Rust type written as a single
scalar identifier (u64 in every one of these 14 cases), and their own
value text, read directly from the source today, is either an expression
naming one or more other const declarations from the same file (for
example, one of the 14 has the value text "DATA_UNIT_HEADER_BYTES +
NONCE_MAC_ALGORITHM_RESERVED_BYTES", naming two other consts by name; one
of the 14 has the value text "768 * 1024 * 1024", a pure arithmetic
expression naming no other const); the remaining 1 of these 15 has a
compound type, an array of 3 unsigned 32 bit integers, and its own value
text, read directly from the source today, is "[0, 1, 0]", a literal
array, not an arithmetic expression naming any other const at all. All 72
of these declarations, including all 15 that fail under integer_literal,
already return some result today, in the sense that stage 92, using its
own actual current reading choice, normalized_text, successfully reads a
comparable text value for every single one of them.

A candidate for item T10.
Candidate UNIFY: change stage 92's reading choice from normalized_text to
integer_literal, matching stage 27's default, so both stages read Rust
const values the same way.

Table task for item T10. Consider the 15 declarations described in fact
10-4 that return None under integer_literal, as 15 rows (you do not need
their individual names; refer to them by an index you assign, 1 through
15, in the order you discuss them, and state, for each one, whether its
own value text, as characterized in fact 10-4, is an arithmetic or
naming expression referring to other consts, or is a literal compound
value such as an array). For each of these 15 rows, state what stage 92
would do with that declaration today, under its actual current reading
choice (normalized_text), and what stage 92 would do with the same
declaration if candidate UNIFY were adopted, grounding both parts only in
facts 10-3 and 10-4 and the shared library's docstring quoted in the
background paragraph above.

Questions for item T10.

14. Give the filled table for all 15 rows described in fact 10-4, one
block per row, following the table task above exactly.

15. If candidate UNIFY were adopted today, would stage 92 fail red purely
because of these 15 declarations, with no change to any layout's format
definition and no change to any checker code at all? Answer using only
fact 10-4, and add the required "This would be refuted by:" sentence.

16. The shared library's docstring, quoted in the background paragraph,
states that normalized_text is "meant only as a change detector: it
never asks what the value equals." Using only fact 10-3's two quotes and
fact 10-4, is stage 92, as it is actually written and called today, best
described as a change detector, a value validator, or something that
tries to be both at once? Give one sentence of justification that quotes
the exact phrase your verdict rests on from fact 10-3. Add the required
"This would be refuted by:" sentence.

17. Fact 10-1 states that stage 27 only checks a declaration's value
against a marker comment when that declaration's name already appears
among the marker comments, and that a declaration whose name never
appears in any marker comment is never looked at by stage 27 at all.
Using only facts 10-1 and 10-4, can you tell, from the facts given here
alone, how many of the 15 declarations in fact 10-4 have a name that
currently appears in any marker comment anywhere, and are therefore
actually subject to stage 27's own check today? Answer using only the
facts given above; if you cannot tell from these facts alone, say so
explicitly instead of guessing, and state what additional fact would be
needed.

END OF PROMPT. Answer questions 1 through 17, in order, following every
instruction given at the top of this prompt.
