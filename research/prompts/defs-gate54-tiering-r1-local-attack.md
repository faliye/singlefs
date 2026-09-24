SINGLEFS THREE DEFINITION CHANGES - LOCAL ATTACK LEG - ROUND defs-gate54-tiering-r1

You are one of several independent reviewers checking a batch of three small
definition changes in singlefs, a copy on write filesystem being designed
from scratch. The changes affect how automated agent definitions and one
gate script in this project's own tooling behave. You are given a self
contained set of facts below. Do not assume anything not stated here. Read
every fact carefully, every qualifier is load bearing, a definition with one
word removed can flip an answer. Do not use any markdown emphasis anywhere
in your answer, no bold text, no italic text, no backtick code formatting,
no asterisk bullets, no pipe tables, no heading marks. Answer entirely in
English. Do not cite, invent, or guess at any source file line number or
code line number anywhere in your answer, including any you might work out
for yourself while reasoning; refer only to the fact numbers given below
(FACT 1 through FACT 11), the row labels given below, gate stages by their
project assigned number or name, and plain descriptive names for functions
(for example "the row building function" is fine, a line number is not).

This round has several independent reviewers, each covering a different,
non-overlapping part of this same batch of three changes. You are covering
only the two rows groups given below, about two of the three changes. A
third change, and questions about cross session and cross workspace history
around all three changes, are being handled by other reviewers using
separate material. Do not answer anything about that third change; if you
find yourself reasoning about it, stop and return to the two changes
actually asked about here.

BACKGROUND

This project already has an automated project wide gate check called the
write scope gate. Every one of this project's agent definitions declares,
in its own definition file, a list of file path patterns it is allowed to
write to. The write scope gate is a hook that runs automatically every time
one of these agents calls a Write or Edit tool; it looks up the calling
agent's registered list of path patterns in one shared table, and allows
the write only if the target file's path matches at least one of that
agent's own registered patterns.

This project also has a numbered sequence of independent automated gate
stages, each one a separate script, each one checking one specific thing.
Gate stage 33 and gate stage 59 both read the same file, a table of source
code mutations named crates/mutations.tsv, but they check different things
about it and run under different conditions, described in the facts below.
Gate stage 54 is unrelated to mutations; it replays crash points against
two data streams and writes a marker file when it passes in its most
thorough mode, described in facts 10 and 11 below.

FACT 1. As part of the first change under review, the write scope section
of the experiment runner agent's own definition file now includes, among
its other entries, this new entry: the mutation rows of the in repo device,
meaning the end of the file crates/mutations.tsv; that entry is scoped to
only appending this experiment's own new rows there, and explicitly to not
modifying any row already present in that same file.

FACT 2. As the second half of that same first change, a new row was added
to this project's write scope table, the shared table the write scope gate
described above reads. The new row grants the experiment runner agent the
path crates/mutations.tsv, with an attached note reading: the mutation rows
of the in repo device are only ever appended at the end, per step 2 and the
write scope section of experiment runner's own definition. That same note
also records a real incident dated 2026-09-23, experiment E158, where the
write scope gate rejected an attempted append to this same file; because of
that rejection, three rows meant for crates/mutations.tsv instead ended up
written to a different location, inside research/mutations/, a location
that gate 59's replay, described in fact 5 below, does not read from.

FACT 3. The write scope gate itself works as follows, unrelated to this
specific change and unchanged by it. When an agent defined by one of this
project's agent definitions calls the Write or Edit tool, the gate looks up
that agent's registered list of path patterns in the write scope table (the
same table fact 2 added a row to), normalizes the target file path, and
allows the write only if that path matches at least one of the agent's
registered patterns; an agent with no row at all in that table has every
one of its writes refused outright. Two other situations make this same
gate a no-op instead of a check: a write made without going through any of
this project's own agent definitions at all, and a write made through an
agent type this project has no definition file for; in both of those two
situations the gate allows the write through without checking any pattern
at all. This matching is purely a check of
which file path is being written to. The gate has no way to inspect what
part of a file's existing content a given write changes; it cannot tell
apart a write that only appends new lines at the end of a file from a
write that also rewrites or deletes lines already present in that same
file, because both kinds of write target the exact same file path and
therefore match the exact same registered pattern. The gate also only
intercepts the Write and Edit tools; it does not intercept or see any
write made to a file through a Bash shell command.

FACT 4. A separate, independent automated gate stage, numbered 33, performs
its own check of the file crates/mutations.tsv every time it runs, and it
runs unconditionally as part of every full pass of this project's whole
gate sequence, with no skip condition of its own. Every row of that file is
six tab separated fields: a mutation name, a file path, an original text, a
replacement text, cargo test arguments, and the name of a test that must
fail. For every row, gate 33 rereads the file path named in that row, and
counts how many times that row's own original text field occurs in that
file's current content; if that file does not exist at all, or if that
count is not exactly one, gate 33 reports that row as a failure and the
whole gate stage fails. Gate 33 separately
reports two rows as a failure whenever they share the exact same
combination of file path, original text, replacement text, and expected
failing test, and separately whenever two rows share the exact same
mutation name. Gate 33 never compiles or executes anything; it only
rereads file content and counts substring occurrences, so running it costs
on the order of milliseconds.

FACT 5. Another separate, independent automated gate stage, numbered 59,
actually replays every row of crates/mutations.tsv. For each row, it copies
the whole repository into a separate scratch directory, replaces that row's
original text with its replacement text inside that row's named file in
that copy, runs that row's named cargo test command in that copy, and
requires that row's named test to fail; it then discards that scratch copy
before moving on to the next row. If any row's original text is not found
exactly once in its named file, if any row's named file falls outside the
specific set of paths this stage copies into each scratch directory, or if
any row's named test does not fail, the whole stage fails. Before doing any
of this row by row work, gate 59 first checks whether the inputs it reads
have changed at all since the last time this project's whole gate sequence
passed completely; if nothing relevant changed, it exits early with a skip
status, without replaying any row that time. Running gate 59's full row by
row replay, when it is not skipped, takes on the order of hours, because it
recompiles and reruns tests for every single row; because of this cost, a
person implementing a change under crates/ is not expected to run gate 59
themselves before handing that change off, and gate 33's cheap check from
fact 4 exists specifically to catch the same kind of row level problem
cheaply, without waiting for gate 59 to eventually catch it during a full
gate pass.

FACT 6. Consider two separate, concrete workflow histories in which more
than one person or process ends up writing to the single shared
crates/mutations.tsv. History PATCHCOPY: several different people, each
implementing a different, unrelated change, each work in their own
separate, full copy of the repository, entirely isolated from one another;
each of them, following the rule from facts 1 and 2, appends only their own
new row to the end of their own copy's crates/mutations.tsv; each of them
then produces a patch, meaning a diff, from their own copy; those patches
are then applied one after another, in some order, to the single real,
shared crates/mutations.tsv. History BATCHRENAME: after all of that, a
separate, later session, whose job is fixing something gate 33 or gate 59
reported (for instance, two rows gate 33 reported as sharing the same
mutation name, per fact 4), performs one bulk, table driven rename: it
reads the whole current crates/mutations.tsv, computes a new name for
every row that needs one, and rewrites, in one single pass, every row of
the file whose name needs to change, not only the specific row or rows it
was asked to fix.

FACT 7. Separately from the second change under review, described in fact 8
below, the crash verifier agent's own definition already contains this
standing rule, unaffected by the current proposal: before starting any gate
stage, crash verifier checks machine load once, following the project's
shared conventions; and specifically, whenever some other, separate gate.sh
process is already in the middle of running gate stage 54, crash verifier
does not start its own run of gate stage 54, and instead waits for that
other run to finish first.

FACT 8. The second change under review edits the very next rule in that
same crash verifier definition, the one describing how crash verifier
actually runs each gate stage assigned to it. Before this change, whenever
the stage crash verifier was about to run was gate stage 54, it ran that
stage plainly, with no extra flag, which runs only gate 54's quick tier,
described in fact 10 below. After this change, whenever the stage crash
verifier is about to run is gate stage 54, it now always passes the flag
called full to it in addition, while still running its assigned stages one
at a time, one after another, exactly as it did before this change.

FACT 9. Separately from the change under review, and unaffected by it, this
project's shared conventions for all of these automated agents, crash
verifier included, already state a standing rule about long running work.
Such work may be waited on; it must never be given a timeout, and it must
never be killed partway through. Compiling, working with a mutation table,
and replaying a gate stage are the specific kinds of work this rule names
as examples of long running work. This kind of work is started using a
background launch option of this project's shell tool; once such a
background job is started, the current turn of work ends, and the agent
that started it is only notified later, when that background job finishes.
Ending the turn this way means that turn's own reply says only, in one
sentence, what is being waited for, and calls no further tool after saying
that. A command run directly in the foreground, without using that
background launch option, is separately capped at a timeout of 240000
milliseconds; that cap applies only to a foreground command, not to work
started through the background launch option.

FACT 10. Separately from the change under review, and separately from
crash verifier's own definition, this project's main agent definition
already describes one specific point in its own overall workflow: the end
of one stage of work, such as one milestone step, one round's verdict, one
experiment segment, or one batch of definition or script edits, once every
hand back for that whole batch has come in, and before that batch is
staged for a commit. At that specific point, and stated as happening only
there, if that batch of work touched anything under crates/, the main
agent itself, as distinct from crash verifier, starts a background run of
gate stage 54 using the flag called full. The same sentence describing
this point states explicitly: layer 0's full run is run only at this one
point in the whole workflow; when that run passes, it writes an all green
marker file; and the check gate stage 54 performs as part of every full
pass of the whole gate sequence only verifies that marker file, it does
not itself rerun the full layer 0 replay.

FACT 11. Separately from the change under review, and unaffected by it,
gate stage 54's own script already behaves as follows whenever it is run
with the flag called full. Unlike its own quick tier described in fact 10,
a run with the flag called full never checks whether it can be skipped or
reused; every single time it is invoked with that flag it runs the whole
thing again from the start, regardless of whether anything actually
changed since the last such run. At the very start of such a run, before
doing anything else, it deletes any all green marker file left over from a
previous run. If any check during that run fails, or if the run is
interrupted before reaching its own end for any reason, no marker file is
written or left behind at all, because the only marker file that existed
was already deleted at the very start, and nothing later in that run
writes a new marker file until the very last step. A marker file is only
ever written once, right at the very end of such a run, and only if every
check along the way passed, and a hash computed over that run's input
files at the end of the run matches the hash computed over those same
input files at the very start of that same run.

ROWS FOR THE FIRST CHANGE

The first change under review is the pair of facts 1 and 2 together, the
new write scope entry for the experiment runner agent and its matching row
in the write scope table. There are five rows to evaluate against that
first change. Each row is named by the row label used in the questions
below.

ROW K1SCOPEGATE pairs the first change with fact 3, the write scope gate's
own matching mechanism.

ROW K1PATCHCOPY pairs the first change with history PATCHCOPY from fact 6.

ROW K1GATE33 pairs the first change with fact 4, gate stage 33's cheap scan.

ROW K1GATE59 pairs the first change with fact 5, gate stage 59's full replay.

ROW K1BATCHRENAME pairs the first change with history BATCHRENAME from
fact 6.

ROWS FOR THE SECOND CHANGE

The second change under review is fact 8, crash verifier now always passing
the flag called full whenever it is about to run gate stage 54. There are
four rows to evaluate against that second change. Each row is named by the
row label used in the questions below.

ROW K2OWNRULE1 pairs the second change with fact 7, crash verifier's own
standing rule about waiting for another gate.sh already running stage 54.

ROW K2LONGWAIT pairs the second change with fact 9, the shared convention
that long running work may be waited on and must never be given a timeout
or killed partway through.

ROW K2MAINSYNC pairs the second change with fact 10, the main agent's own
stage sync point, where it separately runs gate stage 54 with the flag
called full whenever a batch touching crates/ finishes.

ROW K2REDMARKER pairs the second change with fact 11, gate stage 54's own
behavior of deleting its marker at the start of a full run and only writing
a new one at the very end, only if every check passed.

QUESTIONS FOR THE FIRST CHANGE ROWS

For each of the five rows K1SCOPEGATE, K1PATCHCOPY, K1GATE33, K1GATE59, and
K1BATCHRENAME, answer two labeled parts.

WHATHAPPENS. Trace concretely, step by step, what actually happens if the
new write scope rule from facts 1 and 2 is carried out exactly as written,
in the specific situation that row pairs it with. State plainly whether the
situation named by that row can even occur while the new rule is being
followed exactly as written; and if it can, describe concretely what state
crates/mutations.tsv, or the specific location that row concerns, ends up
in as a result.

GATE. Name every one of this project's numbered gate stages that, given the
WHATHAPPENS state you just described for that same row, would itself
produce a failing, non zero, non skip result because of that state, and say
concretely what about that resulting state is what makes that gate stage
fail. If no numbered gate stage described anywhere in the facts above would
fail, say so explicitly, and say specifically why not, rather than leaving
this part blank or vague.

QUESTIONS FOR THE SECOND CHANGE ROWS

For each of the four rows K2OWNRULE1, K2LONGWAIT, K2MAINSYNC, and
K2REDMARKER, answer two labeled parts.

WHATHAPPENS. Trace concretely what happens when crash verifier's changed
rule from fact 8 operates together with the fact that row pairs it with
(fact 7, fact 9, fact 10, or fact 11, respectively). State plainly whether
the two facts can become true, or come into play, at the exact same real
point in time or in some specific real sequence relative to each other, and
if so, describe the concrete resulting behavior in that case.

GATE. Name every one of this project's numbered gate stages that would flag
anything wrong in the resulting behavior you just described for that same
row, or that would prevent an incorrect state from being recorded as if it
had succeeded. If none would, say so explicitly, and say specifically why
not, rather than leaving this part blank or vague.

FORMAT RULES

Give exactly nine numbered answers, in this exact order, using exactly
these nine labels: K1SCOPEGATE, K1PATCHCOPY, K1GATE33, K1GATE59,
K1BATCHRENAME, K2OWNRULE1, K2LONGWAIT, K2MAINSYNC, K2REDMARKER. For each of
the nine labels, give both the WHATHAPPENS part and the GATE part, each
clearly labeled. Do not answer any part with only a single word or a short
phrase such as only yes, only no, or only nothing happens; always also give
the specific concrete reasoning or the specific concrete state that
supports it, referring by number to the facts it rests on. After the
WHATHAPPENS and GATE parts of each of the nine labels, add one further
sentence starting exactly with the words "This would be refuted by:"
describing one specific, concrete observation that would prove that row's
two answers wrong. Do not use any markdown emphasis anywhere in your
answer, no bold text, no italic text, no backtick code formatting, no
asterisk bullets, no pipe tables, no heading marks. Do not cite, invent, or
guess at any source file line number or code line number anywhere in your
answer; refer only to fact numbers, row labels, gate stage numbers or
names, or plain descriptive names for functions. Answer entirely in
English.

END OF FACTS. Answer K1SCOPEGATE through K2REDMARKER now, in the format
given above.
