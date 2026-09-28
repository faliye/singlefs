This is a fact checking and counting task about a plan to merge a set of software project
internal CI gate scripts into a smaller set of files. You are not asked to run any code.
Every fact below was read out of a fixed inventory table that a separate reviewer already
built by reading the project's own gate scripts; you do not need to re-read any source
file yourself, you only need to work from the facts given below.

Formatting rules for your answer, follow all six of them.

One. Do not use any markdown emphasis anywhere in your answer: no bold, no italics, no
headings marked with hash signs, no backtick quoted spans. Plain sentences only.

Two. Number your answers to match the question numbers given below, for example T1-Q1,
T1-Q2, TB-Q1.

Three. For every answer that states a conclusion, add one sentence that starts with the
words this would be refuted if, and names a concrete, checkable observation.

Four. Do not cite a source code line number or a file line number anywhere in your answer.
If you need to point at a specific fact, point at it by the row number given below, or by
the exact name of a gate script or a cell, not by a line number.

Five. Everything you need is in this document. You do not have a shell, so do not propose
running a command as part of your final answer, give the answer itself.

Six. Always put a space after a colon, a semicolon, or a comma when one of those marks is
immediately followed by a letter. Do not write two colons glued directly to a letter with
no space, even when quoting a piece of this document; if you must quote such a thing,
insert a single space right after the punctuation mark and say in words that you inserted
it.

Background. A team wants to merge 23 existing gate scripts down to 15 scripts, grouped
into a small number of categories, without changing what any individual check judges. The
merge plan groups the 23 old scripts into 10 groups. Below is the reference table for
those 10 groups. Each group states the group's target name, and the exact list of old gate
script file names that the plan says get merged into it.

Group 1. Target name: doc-kb. Old gate script file names merged into it:
10-references-and-invariants.sh, 20-doc-decision-documents.sh,
30-decision-history-entries.sh, 32-doc-field-and-layout-registry.sh,
43-checks-owed-and-closeout.sh. The plan states this group should end up with a total cell
count of 37.

Group 2. Target name: doc-experiments. Old gate script file names merged into it:
34-doc-experiment-pages-and-products.sh. The plan states this group should end up with a
total cell count of 10.

Group 3. Target name: doc-process-records. Old gate script file names merged into it:
11-review-and-sync-records.sh. The plan states this group should end up with a total cell
count of 7.

Group 4. Target name: doc-text. Old gate script file names merged into it:
12-doc-forbidden-notations-and-old-terms.sh. The plan states this group should end up with
a total cell count of 3.

Group 5. Target name: code-source-discipline. Old gate script file names merged into it:
13-code-vague-names-and-test-file-names.sh, 27-code-constants-enums-bits-match-kb.sh,
33-code-experiment-and-mutation-source-discipline.sh. The plan states this group should
end up with a total cell count of 8.

Group 6. Target name: code-tooling. Old gate script file names merged into it:
47-code-tooling-selftests-and-registries.sh. The plan states this group should end up with
a total cell count of 8.

Group 7. Target name: code-research-build. Old gate script file names merged into it:
15-research-build.sh. The plan states this group should end up with a total cell count of
1.

Group 8. Target name: harness-tests. Old gate script file names merged into it:
14-harness-one-scenario-per-test.sh, 74-model-differential.sh. The plan states this group
should end up with a total cell count of 2.

Group 9. Target name: checker-independence-and-sync. Old gate script file names merged
into it: 92-layout-checker-sync.sh, 94-checker-implementation-disjoint.sh. The plan states
this group should end up with a total cell count of 2.

Group 10. Target name: kept separate, one merged file per old file, not actually combined.
Old gate script file names merged into it: 54-layer0-replay.sh, 55-qemu-device-streams.sh,
57-lkmm.sh, 59-crates-mutation-replay.sh, 87-replay.sh, 77-test-environment.sh. The plan
states this group should end up with a total cell count of 6.

These ten groups' stated cell counts, added together, are 37 plus 10 plus 7 plus 3
plus 8 plus 8 plus 1 plus 2 plus 2 plus 6. Separately, the inventory used below has 84
rows in total. Whether those two numbers actually match is one of the counting questions
asked at the end of this document; it is not asserted here as already true.

Now here is the inventory itself, split into 14 tables of 6 rows each, 84 rows in total.
Every row gives three facts taken straight from the inventory: the old gate script file
name the cell lives in today, the cell's own name, and what that cell judges, copied
faithfully from the inventory's own description column.

Table T1.

Row 1. Old gate script: 10-references-and-invariants.sh. Cell name: experiment-refs. What
it judges: Every experiment number cited anywhere in kb or in the project rules must have
a matching definition heading inside the experiments directory of kb.

Row 2. Old gate script: 10-references-and-invariants.sh. Cell name: decision-refs. What it
judges: Every decision number cited anywhere in kb or in the project rules must have a
matching definition heading inside the decisions directory of kb.

Row 3. Old gate script: 10-references-and-invariants.sh. Cell name: declared-counts. What
it judges: In the invariants file, the line right after the invariant-count marker,
stating the current number of invariants in use, must equal the actual number of in-use
rows in the table; the owed-checks table's counts of open items and paid-off items must
also come out computable.

Row 4. Old gate script: 10-references-and-invariants.sh. Cell name: governance-refs. What
it judges: In governance documents, every reference of the form gate number N must have a
matching stage; every backtick-quoted in-repo path must exist; every reference of the form
file name plus a quoted section name must be findable inside that file.

Row 5. Old gate script: 10-references-and-invariants.sh. Cell name:
invariant-count-elsewhere. What it judges: Anywhere outside the invariants file and the
two change-history files, wherever a kb file states a sentence like N invariants in use,
that N must equal the real in-use count.

Row 6. Old gate script: 10-references-and-invariants.sh. Cell name: citations. What it
judges: Load bearing external line by line citations inside kb must still be verifiable
today. This is judged by calling a citation verification script and reading its exit code;
if that script itself is missing, this cell fails.

T1-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T1-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T2.

Row 1. Old gate script: 10-references-and-invariants.sh. Cell name: invariant-anchors.
What it judges: Any invariant line newly written or rewritten in a given change must cite
at least one settled item of some decision.

Row 2. Old gate script: 11-review-and-sync-records.sh. Cell name: batch-scope. What it
judges: Every file in the not yet committed change set that matches a fixed trigger table
must be registered in a batch scope file, carrying a reason and written from the
repository root.

Row 3. Old gate script: 11-review-and-sync-records.sh. Cell name:
verdict-names-local-samples. What it judges: A newly written round level main verification
file must cite, by exact file name, every sample produced by that round's local leg,
including void samples; zero byte samples are exempt.

Row 4. Old gate script: 11-review-and-sync-records.sh. Cell name:
crates-adversarial-review. What it judges: Every Rust source file under crates touched by
a given change must appear, by path, inside some newly written main verification file for
that round.

Row 5. Old gate script: 11-review-and-sync-records.sh. Cell name: implementation-premise.
What it judges: Any three way body file dated on or after a fixed cutoff date must contain
the literal word crates. It also checks that a line naming the local leg, attack or
defense, carries a reason, and that a round with background material has an opening
snapshot.

Row 6. Old gate script: 11-review-and-sync-records.sh. Cell name: abandoned-rounds. What
it judges: Every three way round that dispatched at least one leg must either have a
verdict, meaning a verdict file exists, or the same kb section both cites the material and
carries a verdict marker, or else be registered in an abandoned rounds registry file; that
registry file must not be malformed.

T2-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T2-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T3.

Row 1. Old gate script: 11-review-and-sync-records.sh. Cell name: knowledge-sync. What it
judges: Every file in the change scope that matches the trigger table must appear, by
path, inside some sync record file marked with a knowledge sync marker, within that same
change scope; that record must carry a search section, a hit-handling section, and a
raw-evidence section, each in valid shape.

Row 2. Old gate script: 11-review-and-sync-records.sh. Cell name:
agent-def-adversarial-review. What it judges: Every agent definition file, the shared
conventions file, and the main agent file touched by a given change must appear by path in
a newly written verdict, or be registered by content hash in an exemption file.

Row 3. Old gate script: 12-doc-forbidden-notations-and-old-terms.sh. Cell name:
prime-marks. What it judges: No prime mark style superscript character, of five specific
Unicode code points, may appear anywhere in the repository's text.

Row 4. Old gate script: 12-doc-forbidden-notations-and-old-terms.sh. Cell name:
clock-times. What it judges: Clock times, timezone words, and timestamps in prose text are
forbidden: ISO style timestamps, the machine generated times inside diff headers and stat
output, a bare hour colon minute not sitting next to a date, and bare timezone words; a
line inside a shell, python, or rust file carrying an explicit allow marker is exempted.

Row 5. Old gate script: 12-doc-forbidden-notations-and-old-terms.sh. Cell name:
term-renames. What it judges: A sweep script checks, per a registered table of renamed
terms, that the old name no longer appears anywhere in the repository; an exclusion table
entry that points at a path which no longer exists is itself also a failure.

Row 6. Old gate script: 13-code-vague-names-and-test-file-names.sh. Cell name:
vague-names. What it judges: For every function name and every struct, enum, type or trait
name declared inside the Rust sources under crates and under one fixed research benchmark
directory, if every word obtained by splitting that name is found in a registered list of
vague words, the name is a violation; the exclusion list may only shrink over time, never
grow.

T3-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T3-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T4.

Row 1. Old gate script: 13-code-vague-names-and-test-file-names.sh. Cell name:
test-file-names. What it judges: File names under either of the two test tiers directories
must not carry a milestone ordinal, a step number, an addendum number, a parallel track
number, or an owed check number.

Row 2. Old gate script: 14-harness-one-scenario-per-test.sh. Cell name: one-scenario. What
it judges: For test functions inside the harness tier tests directory, a for loop that
builds a brand new pool on every iteration, calling one of four specific pool constructor
names, build_pool, build_through, a wildcard of that same prefix, format_pool, or
MemoryPool with_devices, is a failure; a line carrying an explicit granularity marker with
a stated reason is exempted.

Row 3. Old gate script: 15-research-build.sh. Cell name: the whole gate. What it judges:
The research workspace's own release mode test run, executed through the memory wrapper,
must have completed at least one batch; in addition, five specific scripts, oov-check,
stage-mine, check-staged, relabel-item, and claim-experiment, must each pass their own
self test.

Row 4. Old gate script: 20-doc-decision-documents.sh. Cell name: kb-shape. What it judges:
Several sub checks bundled under one cell: only one spelling is allowed for open item
versus settled item; a kb file must never link back to itself; upstream rule paths must
carry the shared rules directory prefix; a decision's title numbering must carry a status
word, its two item sections must not bleed into each other or reuse numbers, and the
title's open item count must match its own list; the index page's status column counts
must match the body.

Row 5. Old gate script: 20-doc-decision-documents.sh. Cell name: item-ref-status. What it
judges: Every occurrence of a reference naming a decision and a settled item number, or an
open item number, must have its stated status match that decision's own settled items
index or open items index.

Row 6. Old gate script: 20-doc-decision-documents.sh. Cell name: status-redundancy. What
it judges: A subsection title must not carry a trailing dash plus the word settled; the
same item must not carry both a dash suffix and a separate status colon line; a sub item
index row's status colon must agree with the section it belongs to.

T4-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T4-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T5.

Row 1. Old gate script: 20-doc-decision-documents.sh. Cell name: cross-decision-status.
What it judges: If kb body text writes the word open immediately next to a decision
reference while that decision's own status line is not pending, that is a failure; quoted
text and history sections are skipped.

Row 2. Old gate script: 20-doc-decision-documents.sh. Cell name: settled-item-self-open.
What it judges: Inside a settled items section, if the text says of itself, or of another
settled sub item in the same decision, that it is still open, not yet decided, or not yet
settled, that is a failure.

Row 3. Old gate script: 20-doc-decision-documents.sh. Cell name: settled-ref-says-open.
What it judges: If a reference names a settled item number, the item it points to is
indeed settled, but the text immediately following still says open, not decided,
undecidable, pending, or blank, that is a failure; the two change history files and the
records directory are not scanned.

Row 4. Old gate script: 20-doc-decision-documents.sh. Cell name: stale-open-items. What it
judges: A still open item names another decision, and that other decision's own status
line has changed again, per git history, since this open item's most recent edit or
review.

Row 5. Old gate script: 20-doc-decision-documents.sh. Cell name: settled-same-file. What
it judges: A given change adds a brand new settled subsection to a decision, while the
still open items in the same file, and the index page's pending discussion section, are
not touched at all, per the diff.

Row 6. Old gate script: 20-doc-decision-documents.sh. Cell name: freeze-layer-membership.
What it judges: A freeze layer membership registry must have a row for every layer, every
component, both tiers of every tree, and every unit class code; any row marked exited must
be a derived state; the sub item it cites as its basis must exist and must be settled.

T5-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T5-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T6.

Row 1. Old gate script: 20-doc-decision-documents.sh. Cell name: decision-items-sync. What
it judges: The decisions file's generated sub item list block, and the index table's
status column, must be byte for byte identical to the output of a fixed generator script;
the write mode of this cell reruns only this one cell.

Row 2. Old gate script: 20-doc-decision-documents.sh. Cell name: blocking-verdict. What it
judges: Every open item's registry row must carry two separate measures, one about whether
it changes bytes of a newly created file in a new pool, with a yes, no, or no such object
plus a date, and one about whether it touches the on disk format; a mismatch between the
counted rows and the generator's own count is also a failure.

Row 3. Old gate script: 20-doc-decision-documents.sh. Cell name: user-verdict-owed. What
it judges: Any decision body item marked pending user review must have a matching unpaid
entry, carrying no dated payoff marker, inside the owed checks file, citing that decision
by its full number.

Row 4. Old gate script: 20-doc-decision-documents.sh. Cell name: decision-summary-width.
What it judges: Every row's conclusion column in the decision index table must not exceed
a character count cap; that cap is itself read from a sentence inside the decisions file.

Row 5. Old gate script: 27-code-constants-enums-bits-match-kb.sh. Cell name:
format-constants. What it judges: A kb marker naming a constant and a value, and possibly
a stale marker with an old value, must have its stated value equal the integer literal of
the same named constant in source code; the stale value string must no longer appear
anywhere in kb body text or in experiment source; an unreadable marker, or the same name
registered twice, is a failure.

Row 6. Old gate script: 27-code-constants-enums-bits-match-kb.sh. Cell name: clause-enums.
What it judges: Checked pair by pair against a fixed table: every enum variant must be
registered; every variant listed in the table must actually exist in code; the table's
name must be verbatim one of the items listed with a slash separator in that sub item's
own body text; every item listed in that body text must have a matching table row.

T6-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T6-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T7.

Row 1. Old gate script: 27-code-constants-enums-bits-match-kb.sh. Cell name: feature-bits.
What it judges: A feature bits document and a settled item's own registry table must match
row by row on category and bit number, including matching meaning; every bitmap's bit
numbers must run from zero upward without any gap; every relevant constant's bit number in
crates source code must appear in the ledger; the same bit number must never be registered
on two separate rows.

Row 2. Old gate script: 30-decision-history-entries.sh. Cell name: entry-added. What it
judges: If a decision's body text is changed, meaning the total of added and removed lines
exceeds four, or a status line is touched, the change history file must contain a newly
added entry, counted by the number of newly added subheadings, or if none, by the number
of newly added dated headings.

Row 3. Old gate script: 30-decision-history-entries.sh. Cell name: shape. What it judges:
Only newly added entries are judged. A dated heading line must not be jammed together with
a parenthetical remark on one physical line; in both change history files, a dated block
must live inside its own decision or experiment section; the same date must never appear
twice inside the same section.

Row 4. Old gate script: 30-decision-history-entries.sh. Cell name: status-sync. What it
judges: In both change history files, the current status line sitting at the top of every
decision or experiment section must match that same row's status column, or brief
conclusion column, in the decisions index table or the experiments index table; sections
and index rows must correspond one to one; the write mode of this cell rewrites only that
one status line.

Row 5. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: field-refs. What
it judges: Every points to column entry inside the first transaction layout document,
written as a decision name plus a settled or open item number, must genuinely exist inside
that decision's own settled items or open items index.

Row 6. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: field-projection.
What it judges: For any sub item that has been projected by the first transaction layout
document, every field name in that sub item's own field width table must also appear
inside the table rows of the layout section that cites that sub item.

T7-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T7-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T8.

Row 1. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: field-table-sums.
What it judges: A field table's trailing total byte count line, the total width and delta
stated in the sentence preceding the table, and the single format constant marker inside
one sub item, must all equal the sum of that field table; any recognizable struct total
width appearing inside a byte layout table must have a matching format constant
registration.

Row 2. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: first-txn-hooks.
What it judges: An open item judged yes and still open must be cited exactly once in the
byte layout table and once in the milestone document; the blank table only accepts items
judged yes; every section of the byte layout table has its own milestone and step
reference; every milestone step's bytes written line names an existing section.

Row 3. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: admission-terms.
What it judges: Every term appearing in the admission inequality formula must have an
attribution inside a matching cross reference table, either an item number in a statistics
table, or an explicitly stated exception; every copy of that formula appearing under the
decisions directory must be verbatim identical to the one authoritative line.

Row 4. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: segment-registry.
What it judges: Every row's segment sequence and step kind multiset inside the root slot
write path segment sequence registry table must be compared, byte for byte, against a
fixed experiment artifact and a pinned test case, via a dedicated checking script.

Row 5. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name:
format-const-placeholders. What it judges: For every placeholder marker inside the format
crate's source, the decision number it names must appear inside that decision's two index
tables, and the owed check number it names must have a registered row inside the owed
checks file.

Row 6. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name: second-txn-hooks.
What it judges: Every row of the second transaction layout table must have six cells; the
step it names must exist, and that step's own bytes written line must name this row back;
the layout section or layout row named by a milestone must exist; the pinned test's file
and function must exist; the step marked by the segment sequence registry table, and the
step that in turn cites it, must match on both sides.

T8-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T8-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T9.

Row 1. Old gate script: 32-doc-field-and-layout-registry.sh. Cell name:
tree-table-reserve. What it judges: For a reservation table marked with a fixed total
value, every width in it must be a positive integer, the sum must not exceed that total,
every row's provenance must carry an existing owed check or decision number, and exactly
one such marked table may exist.

Row 2. Old gate script: 33-code-experiment-and-mutation-source-discipline.sh. Cell name:
mutation-tables. What it judges: Every experiment binary source file must have a matching
well formed table inside a registered mutations directory, whose anchor is hit exactly
once; a shared mutation table for the crates workspace is checked by a static checker for
shape, anchor, test name, escaping, and copy range; it also flags citing an ignored test
without the matching flag, a filter word that matches nothing, and duplicate rows; a
relative path literal that escapes the intended copy range is also flagged.

Row 3. Old gate script: 33-code-experiment-and-mutation-source-discipline.sh. Cell name:
absolute-assertions. What it judges: Every experiment binary must carry at least one
assertion that pins a measured quantity against a bare literal number, recognized across
multiple source lines by balancing the statement's own parentheses.

Row 4. Old gate script: 33-code-experiment-and-mutation-source-discipline.sh. Cell name:
reading-discipline. What it judges: Two specific owed check patterns in experiment source:
an identifier whose name contains the word seed immediately followed by a bitwise or with
one, or a bitwise and with not one; and an integer struct field that never appears on the
write side while every one of its initializations is the literal zero; two exemption
tables must each be pinned to an open owed check, must match a genuine violation, and may
only shrink per file, never grow.

Row 5. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name: index-sync.
What it judges: An experiment page's own title state, meaning abandoned, not run, partly
run, or sufficient to judge, must match its own row in the experiments index; the number
in that index row's conclusion column must be findable somewhere in the page body; the two
sides must correspond one to one.

Row 6. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name: results-cited.
What it judges: Every artifact file sitting under the results directory must be cited by
name either in the experiments index or in some experiment page; an experiment marked as
already run must cite an artifact; any artifact newly cited by a given change must be
findable either in the current tree or in git history.

T9-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T9-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T10.

Row 1. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
evidence-in-repo. What it judges: Any experiment apparatus or mutation table touched by a
given change must have a matching artifact under the results directory that is not older
than itself, or whose input fingerprint matches; kb, and any prompt file newly written
this round, must never cite a temporary directory path as evidence.

Row 2. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
decision-links. What it judges: Nine separate sub checks bundled under one cell, in order:
the shape of the affected decisions table; the decisions and sub items named in that table
must exist; support and rebuttal must be registered in both directions; a scheduled review
back must not have expired; any review back due because of this change has actually been
done; the list of items still to be filled back in may only shrink, never grow; a slimmed
down table shape; a decision may only exist if a matching experiment exists; every
experiment must correspond to some decision.

Row 3. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
verdict-false-named. What it judges: For every field marked false inside a verdict line of
a replay script registered artifact, that same field must be cited on the same line,
either in the corresponding experiment page body or in that page's most recent history
entry.

Row 4. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name: repro-command.
What it judges: Any experiment page that cites an artifact under the results directory
must state a cargo run command for a specific binary, or must cite one of two named replay
scripts; pages that explicitly state the raw output was not retained are exempted.

Row 5. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
experiment-orphans. What it judges: Anything under the research directory, excluding build
output, named with the experiment number pattern, and any matching binary source file
under the checker tier crate, must have a matching same numbered body page inside the
experiments section of kb.

Row 6. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
quoted-result-lines. What it judges: Any line inside kb or research body text that, after
trimming leading and trailing blank space, starts with a fixed result line marker, must be
found verbatim inside a replay script registered artifact, or inside archived history;
only lines newly added or rewritten by a given change are judged.

T10-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T10-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T11.

Row 1. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
archive-past-rounds. What it judges: Experiment records from the previous round and
earlier, sitting under the prompts and results directories, must already be archived,
meaning deleted, while keeping verdicts, the abandoned rounds registry, and any artifact
still used as input by code.

Row 2. Old gate script: 34-doc-experiment-pages-and-products.sh. Cell name:
multipath-registry. What it judges: An experiment page carrying a path and conclusion
registry section: its header must be verbatim, must have five cells, its source landing
file must exist or state no implementation plus a reason, and the shared item cell must
never be empty or say none; a separate lag table has two columns, must point at an
existing page that has not yet written its own registry section, and may only shrink over
time, never grow.

Row 3. Old gate script: 43-checks-owed-and-closeout.sh. Cell name: table-shape. What it
judges: Both registry tables inside the owed checks file must have every row's cell count
match its own header, six columns for the open table and four columns for the paid table;
the same number must never be registered once in each of the two tables; the open table
must never keep a row whose start already reads paid, written off, or void; every numbered
row must be preceded by its own header row.

Row 4. Old gate script: 43-checks-owed-and-closeout.sh. Cell name: closeout-collects-open.
What it judges: Inside a milestone file marked with a closeout table marker, every still
open owed check number cited anywhere in the full text must either be inside the closeout
table, or have an explicit not closing out line with a reason right after the table; the
closeout table's first column allows only decimal sequence numbers or a fixed set of
circled numerals.

Row 5. Old gate script: 43-checks-owed-and-closeout.sh. Cell name: paid-cited-tests. What
it judges: A backtick wrapped identifier of three or more underscore separated segments,
sitting inside the paid off table, must be found as a whole word inside non comment code
somewhere in the repository's shell, python, or rust files.

Row 6. Old gate script: 43-checks-owed-and-closeout.sh. Cell name: audit-contradictions.
What it judges: In the most recent full audit record's disposition section, every row
whose disposition column starts with the word changed and quotes some original text, must
have that quoted text findable, today, inside the cited kb file's own body, or must
instead cite an open owed check number; a separate pending audit rows file must have zero
rows.

T11-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T11-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T12.

Row 1. Old gate script: 43-checks-owed-and-closeout.sh. Cell name: row27-preconditions.
What it judges: For the closeout table's row twenty seven, a set of owed items marked
unreachable today, each one has its own verbatim probe made of a file, an original text,
and today's hit count; if every probe matches, this cell exits with the not run today
code; if any probe mismatches, this cell fails; it also separately reports any item for
which no probe could be constructed at all.

Row 2. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
research-script-selftests. What it judges: All forty five self test entries inside a fixed
runner table must pass; any script under the research scripts directory whose own text
contains the string self test flag must either have some gate stage that calls it, or be
registered inside a fixed not run here list.

Row 3. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
rules-manifest. What it judges: The set of files sitting directly under the shared rules
directory must equal, item for item, the set of rule references written inside the
project's own top level guidance file; an at sign reference sitting inside a code fence or
inline code span does not count.

Row 4. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
stage-owners. What it judges: Every top level script under the gate stages directory must
have exactly one row inside a stage owners table; every stage named in that table must
exist, every owner named in it must be a defined agent, and all three columns must be
present; every stage treats its own first argument as the project root.

Row 5. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
agent-write-scope. What it judges: A long list of hooks, write guard, bash command
detector, runner dispatch guard, continuation guard, session start, heavy test guard,
handback guard, ask user claim guard, and one shared upstream hook, must each be
registered on the matcher they require, and each one's own self test must pass; the write
scope table and the agent definitions must be consistent with each other in both
directions; every agent definition must carry the shared conventions opt out marker and a
read first line, and its declared model value must be a recognized one; a function shared
by two modules must not be duplicated into a second copy.

Row 6. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
change-range-single-source. What it judges: Top level scripts under the gate stages
directory must never compute their own diff baseline by themselves; any git invocation
that lists paths must carry a fixed quoting flag; every call to the two shared path helper
functions must check its own exit code.

T12-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T12-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T13.

Row 1. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
research-gate-lint. What it judges: The research scripts directory, the hooks directory,
and the shared scripts directory are all handed to a shared upstream lint checker and a
shared shell lint checker; shell scripts inside the first two of those directories must
carry the executable bit; together with the gate stages directory, any process termination
call anywhere in those directories must name exactly one target process.

Row 2. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
fixture-claims. What it judges: Four sub checks: if a stage's own header states a fixtures
directory belonging to itself, that directory must exist; the fixtures directory as a
whole must contain no orphan subdirectories; every sample directory must contain at least
a red case or a green case, and every subdirectory inside it must contain an expect file;
no file name anywhere in the repository, and no date written inside a fixtures markdown
body, may fall inside an impossible date range.

Row 3. Old gate script: 47-code-tooling-selftests-and-registries.sh. Cell name:
kb-registry. What it judges: Every markdown file directly under the kb root, and every
subdirectory of it, must be registered by path in the first cell of some row inside the
project's own local facts table; every kb path cited anywhere in that table must actually
exist.

Row 4. Old gate script: 54-layer0-replay.sh. Cell name: the whole gate. What it judges: In
its fast tier, every non ignored test inside the checker tier package is run in release
mode; then, row by row, it checks a fixed crash case marker table for an all green marker,
reporting not run this time for any row that does not count; before running, it checks
placement and module declarations through a dedicated checker script; a full run mode is
triggered only by explicit user request or at night.

Row 5. Old gate script: 55-qemu-device-streams.sh. Cell name: the whole gate. What it
judges: Six virtual machine tiers are run in parallel across two virtio disks: the direct
tier's device side log must equal, item by item, the program's own recorded stream; the
flush count and the barriers must match; the segment sequence must be byte identical to a
fixed experiment artifact; two control tiers, one missing a barrier and one going through
the page cache, must both be judged as failures; the remaining three tiers perform a cold
reboot and then read the file back.

Row 6. Old gate script: 57-lkmm.sh. Cell name: the whole gate. What it judges: Every never
assertion under the litmus tests directory must have a matching control group, must be
bound to actual code, and the herd7 tool's own verdict must match the declared assertion.

T13-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T13-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Table T14.

Row 1. Old gate script: 59-crates-mutation-replay.sh. Cell name: the whole gate. What it
judges: Every row of the shared crates mutation table breaks exactly one spot in the code,
and the test named by that row must then fail; before dispatching any of this, it checks
shape, anchor, test name, escaping, copy range, and baseline, meaning the named test still
passes on the unmutated copy.

Row 2. Old gate script: 74-model-differential.sh. Cell name: the whole gate. What it
judges: The harness crate's own random histories routine is run alone, in release mode;
every section inside a fixed list of sections must print a model differential line whose
own step count is greater than zero.

Row 3. Old gate script: 77-test-environment.sh. Cell name: the whole gate. What it judges:
A dedicated environment check script checks ten categories: leftover loop or device mapper
devices, leftover mounts, any qemu process whose command line mentions the project name,
leftover temporary directory files, a host disk that is read only, remaining free space,
an ext4 error counter, kernel log block device errors, and drive health status when
requested; either an explicit failing judgment, or a category that could not be checked at
all, counts as a failure.

Row 4. Old gate script: 87-replay.sh. Cell name: the whole gate. What it judges: Rows
judged exact inside a shared replay registry, minus a separate slow list, are rerun and
byte compared against artifacts already sitting under the results directory; any row that
was not actually rerun is listed by name individually.

Row 5. Old gate script: 92-layout-checker-sync.sh. Cell name: the whole gate. What it
judges: Every row of a layouts registry table must have all four of its columns, every
path it names must exist, and every incompatible feature bit number it names must appear
inside a fixed decision's own registry table with a verbatim matching name; if a given
change alters some layout's own mapping from constant name to value while the checker's
own judging code path was not touched at all, that is a failure.

Row 6. Old gate script: 94-checker-implementation-disjoint.sh. Cell name: the whole gate.
What it judges: Four sub checks: the transitive dependency closure intersection of the
checker crate and the core crate, minus the format crate, must be empty; the checker
crate's source code must have zero references to the core crate; the format crate's own
body code, excluding test only code, must contain no branches and no loops; the harness
crate's dependency closure must not contain the checker tier crate, and its source code
must have zero references to it.

T14-Q1. For each of the 6 rows above, state which one of the 10 groups listed in the
background section this cell lands in, according to the group reference table (identify
the group by its target name, for example doc-kb, not by group number). Answer once per
row, six short statements.

T14-Q2. For each of the 6 rows above, state whether the merge plan, as described in the
background section, gives you any reason to think this cell's own judging criteria, the
text given in what it judges above, would become wider, narrower, or would stop being
judged at all, once it lands inside its target group. If you see no such reason, say so
plainly; if you do see a reason, name the specific word or phrase in the background
section that gives you that reason.

Now three counting questions that use all 84 rows together, across all 14 tables above.

Q-COUNT-1. Count how many of the 84 rows have an old gate script file name that does not
appear in the old-file list of any of the 10 groups in the background section. List the
row numbers, using the table number and row number together, for example T3 row 2.

Q-COUNT-2. Count how many distinct old gate script file names, among all the ones that
appear anywhere across the 84 rows, appear in the old-file list of more than one of the 10
groups. Name any such file name you find.

Q-COUNT-3. For each of the 10 groups, count how many of the 84 rows have an old gate
script file name belonging to that group's old-file list. Compare your count against the
cell count the plan states for that group in the background section. List every group
where your count and the plan's stated count disagree, and give both numbers.

Part two, called TB, is about three specific cells the plan proposes to delete or to
shrink, rather than to merge. For each one, you are given, as a fact, what the inventory's
own dependency column says about who else, if anyone, already judges the same matter.

TB row 1. Cell: row27-preconditions, inside 43-checks-owed-and-closeout.sh. What it
judges: for the closeout table's row twenty seven, a set of owed items marked unreachable
today, each one has its own verbatim probe made of a file, an original text, and today's
hit count; if every probe matches, this cell exits with the not run today code; if any
probe mismatches, this cell fails. What the inventory's dependency column says: nothing
else duplicates this cell; the inventory explicitly says none, noting only that it has the
same shape as a different cell's anchor counting method but is not the same matter. What
the plan proposes: delete this cell entirely, once a specific milestone has been
submitted, because its only reason for existing was to track a single milestone document's
row 27, and that milestone document today already states that row 27 has a destination and
is no longer waiting on any precondition.

TB row 2. Cell: agent-write-scope, inside 47-code-tooling-selftests-and-registries.sh.
What it judges: a long list of hooks must each be registered on the matcher they require
and each one's own self test must pass; the write scope table and the agent definitions
must be consistent with each other in both directions; every agent definition must carry a
required opt out marker and a required first line, and its declared model value must be
recognized; a function shared by two modules must not be duplicated into a second copy.
What the inventory's dependency column says: partial overlap. A separate, shared gate
stage already judges, for every hook under the project's own hooks directory and under a
shared upstream hooks directory, that the hook is registered and that its own self test,
if it has one, passes. This cell's own sub-checks that ask is it registered, did its self
test pass, for ten specific numbered sub-checks out of this cell's full numbered list (the
inventory identifies them by circled numerals, the highest one it names being numeral 16,
so the cell has at least 16 numbered sub-checks in total), are the same matter as what that
separate shared stage already judges; across a full run of every gate stage, those
particular self tests each get run twice. For four specific hooks, namely write-guard, heavy-test-guard, ask-user-claim-guard,
and session-start, the shared stage only judges half of what this cell judges, because
those four hooks' own matcher declarations are only checked by this cell and by no other
stage. The write scope table check, the
definition header check, and the shared module non duplication check are three things
nothing else duplicates. What the plan proposes: delete the part of this cell that
duplicates the separate shared stage's hook registration and self test check, keeping only
the write scope table consistency check, the definition header check, and the shared
module check.

TB row 3. Cell: table-shape, inside 43-checks-owed-and-closeout.sh. What it judges: both
registry tables inside the owed checks file must have every row's cell count match its own
header, six columns for the open table and four columns for the paid table; the same
number must never be registered once in each of the two tables; the open table must never
keep a row whose start already reads paid, written off, or void; every numbered row must
be preceded by its own header row. What the inventory's dependency column says: partial
overlap. This cell's own rule that the same number must never be registered once in each
of the two tables judges the same matter as a separate, shared documentation linting
stage's own rule F, which says a given number having more than one registration position
anywhere in the project is a failure; both of the owed checks file's two tables carry a
marker that hands them to that shared linting stage for exactly this same duplicate
registration check. The inventory notes that the cell count check, the paid off row shape
check, and the header presence check are three things the shared linting stage does not
judge. What the plan proposes: delete the half of this cell that duplicates the shared
linting stage's duplicate registration check, keeping the half the shared linting stage
does not judge.

TB-Q1. For TB row 1, after the plan's proposed deletion, is there anything left that would
judge a future new row being added to the closeout table's row 27 region, or would nobody
judge it. Name the thing that would judge it if you say something would, quoting only from
the facts given above, not from outside knowledge.

TB-Q2. For TB row 2, after the plan's proposed deletion, is there anything left that would
judge a future new hook being registered without a passing self test, for one of the four
specific hooks the facts above say only this cell judges, or would nobody judge it. Name
the thing that would judge it if you say something would.

TB-Q3. For TB row 3, after the plan's proposed deletion, is there anything left that would
judge the same number being registered once in each of the owed checks file's two tables,
or would nobody judge it. Name the thing that would judge it if you say something would.

TB-Q4. Across all three rows in part TB, count how many of the three would end up with
nobody judging the matter described in TB-Q1 through TB-Q3, after the plan's proposed
deletion. Give the count and list which rows, by row number.
