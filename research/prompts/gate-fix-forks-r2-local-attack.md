This is a three-part fact-checking task about a batch of shell and python changes in a
software project's internal CI gate scripts. You are not asked to run any code yourself.
Every fact below was obtained by actually reading the source files or actually running
the commands, and is given to you verbatim. Your job is to judge, cell by cell, what the
given facts show.

Formatting rules for your answer.
1. Do not use any markdown emphasis (no bold, no italics, no headings marked with hash
   signs). Plain sentences only.
2. Number your answers to match the question numbers below (T4-Q1, T4-Q2, and so on).
3. For every answer that states a conclusion, add one sentence starting with the words
   "This would be refuted if" that names a concrete observation.
4. Do not cite a source-code line number or a file line number anywhere in your answer.
   If you need to point at a specific fact, point at it by row number in a table, or by
   the name of a function or a variable, not by a line number.
5. Everything you need is in this document. You do not have a shell, so do not propose
   running a command as your final answer; give the answer itself.

Part one, called T4, is about a check named stage 80 in a file called
80-absolute-assertions.sh. Its job, today, is to make sure every experiment binary that
compares several arms against each other also carries at least one assertion that is
pinned to an absolute number, not just a relative comparison between the arms. A change
under review would widen the set of files stage 80 judges. Today it only looks at every
.rs file directly inside one fixed directory, research/e7-index-bench/src/bin. The
proposed new rule instead judges every .rs file directly inside research/e7-index-bench/
src/bin, plus every .rs file directly inside any crates/*/src/bin or research/*/src/bin
directory whose filename starts with the letter e followed by one or more digits and
then an underscore (for example e156_something.rs). Every other .rs file under those
src/bin directories is only listed by name in the stage's success line; it is not judged
at all under the proposed rule.

Here is the fact table for T4. It lists the only 4 files that exist today under
crates/*/src/bin or research/*/src/bin other than research/e7-index-bench/src/bin
itself (there are no other research/*/src/bin directories besides e7-index-bench today).
For each file: its path, the first lines of its own file-level doc comment (copied
verbatim from the file), how many total assert!/assert_eq!/assert_ne! macro calls it
contains, how many of those match the exact pattern stage 80 uses to count as an
absolute-value assertion (assert_eq!(..., a plain number) or assert!(... compared with
a plain number)), whether the shared replay script research/scripts/replay.sh contains a
line that runs this binary with cargo, and whether its filename matches the proposed
scope pattern e<digits>_.

Row 1. Path: crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs
Doc comment, first 3 lines, verbatim:
"E156 first stage: the cost numbers for the four alloc-basis forks (pre-registered in
research/prompts/e156-preregistration.md)."
"Read-only driver of the public entry points of singlefs-core / singlefs-checker (mkfs,
warm-up, first transaction, overwrite, admin rollback);"
"does not change the production code of either crate; output is printed as E7RESULT
lines, and how to replay it is registered in research/scripts/replay.sh."
Total assert-family macro calls: 9. Calls matching the absolute-value pattern: 1.
Called by research/scripts/replay.sh: yes, that file contains the line
"(cd .. && cargo run -q -p singlefs-harness --bin e156_allocation_basis_counts)".
Filename matches e<digits>_: yes.

Row 2. Path: crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs
Doc comment, first line, verbatim:
"Host side: take the two device-side logs recorded by QEMU blklogwrites, and compare
them field by field, disk by disk, against the recorded stream obtained by replaying the
same write path on the host (same parameters, same bytes)."
The doc comment is then followed by a usage line, not more prose.
Total assert-family macro calls: 0. Calls matching the absolute-value pattern: 0.
Called by research/scripts/replay.sh: no, that file does not mention this binary's name
anywhere. It is instead built and run by .claude/gate.d/55-qemu-first-transaction.sh,
which builds it with cargo and then runs the built binary directly.
Filename matches e<digits>_: no.

Row 3. Path: crates/singlefs-harness/src/bin/first_transaction_on_device.rs
Doc comment, first line, verbatim:
"Real-machine tier (QEMU/KVM): run the whole write path of the first transaction on two
real virtio disks, cold-reboot, then recover and read the file back."
The doc comment is then followed by a usage line, not more prose.
Total assert-family macro calls: 44. Calls matching the absolute-value pattern: 4.
Called by research/scripts/replay.sh: no, that file does not mention this binary's name
anywhere. It is instead built and run by research/scripts/e152-run.sh (which builds it
with cargo, release profile, for a specific target, then runs it directly), and staged
onto a disk image by research/scripts/e152-stage-root.sh.
Filename matches e<digits>_: no.

Row 4. Path: crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs
Doc comment, first 3 lines, verbatim:
"E142 (first-transaction dry run) the implementation side of measurement 5: run
scenario:: run_first_transaction on two in-memory disks" (one space was inserted right
after the double colon; the real file has no space there, but this document's own
automated corruption checker misreads a colon glued to a following letter as a sign of
lost text, and this is not that)
"(same parameters as the real-machine tier and the E142 apparatus: same fsid, same write
timestamp, same 3000-byte content, two 4 GiB disks),"
"and print the bytes written to those 21 regions by the first transaction, one result
line per region."
Total assert-family macro calls: 0. Calls matching the absolute-value pattern: 0.
Called by research/scripts/replay.sh: yes, that file contains the line
"(cd .. && cargo run -q -p singlefs-harness --bin first_transaction_region_bytes) >
\"$impl_snapshot\" || return 1", which uses this binary to produce a snapshot file that
is later compared against another snapshot; the assertions, if any, live elsewhere, not
in this binary itself.
Filename matches e<digits>_: no.

T4-Q1. For row 1, judge whether this file is better described as an experiment (a
program whose own job is to run several arms and compare them, feeding a conclusion
that a decision can be based on) or a tool (a program whose job is to compare recorded
data against one external ground truth, not to run several arms against each other).
Name the single fact from row 1 that most supports your answer.

T4-Q2. Same question as T4-Q1, for row 2.

T4-Q3. Same question as T4-Q1, for row 3.

T4-Q4. Same question as T4-Q1, for row 4.

T4-Q5. Count: across rows 1 through 4, in how many rows does your T4-Q1 through T4-Q4
classification (experiment or tool) disagree with the row's own "filename matches
e<digits>_" column? Disagreement means either you classified the row as a tool but its
filename does match, or you classified it as an experiment but its filename does not
match. Give the count and list the row numbers where it disagrees.

T4-Q6. Describe one concrete, hypothetical future file, by giving all of these
properties: which directory it would sit in (crates/*/src/bin or research/*/src/bin),
what its filename would have to start with, one sentence for what its doc comment would
say, and whether its own assertions would be a relative comparison between arms or a
comparison against one external ground truth. The file you describe must be a case where
this proposed filename-based rule would leave it out of scope even though, by its own
content, it is the kind of multi-arm experiment the rule is meant to catch. State what
you would observe, if this file actually existed and stage 80 were run on it, that would
refute your claim that it is left out of scope.

T4-Q7. Same request as T4-Q6, but for the opposite mistake: describe one concrete,
hypothetical future file that the filename-based rule would pull into scope, even though
by its own content it is a tool (a ground-truth comparator), not the kind of multi-arm
experiment the rule is meant to catch. State the refuting observation the same way.

Part two, called T7, is about two other checks. Stage 52, in a file named
52-segment-registry.sh, used to print a message and exit with code 77 (meaning "skipped,
nothing to judge") when it could not find a script it depends on, named
research/scripts/check-segment-registry.py. Stage 31, in a file named
31-blocking-verdict.sh, used to behave the same way when it could not find a generator
script it depends on, named .claude/scripts/gen-decision-items.py. A change under review
makes both of these cases exit with code 1 (meaning "failed") instead of 77. Stage 31
also used to have a fallback: if the path it computed from its own script location did
not have a file there, it would try a second path, relative to the current working
directory, before giving up. That fallback line has been deleted in the same change.

All of the facts below were obtained either by reading the cited source lines directly,
or by actually running the commands shown and capturing their real output. None of it is
simulated or paraphrased from memory. Four kinds of occasions are covered: a full run on
the real repository through the top-level gate script; the stage's own self-test
harness, which copies small fixture files into a temporary directory and runs the real
stage script against that directory; the temporary git worktree that
"gate.sh --staged" builds; and running the stage file directly by hand, outside the
top-level gate script.

Occasion A, a full run on the real repository. The top-level gate script always computes
an absolute path for the project root before it runs any stage (source:
.claude/singlefs-ai-sop/scripts/lib.sh, the function named project_root, and
.claude/singlefs-ai-sop/scripts/gate.sh, the assignment "ROOT=... project_root", both cited by
function name only). Each stage is then run as "bash <absolute path to the stage file>
<the absolute ROOT path>". I ran both stages this way with no extra argument.
Row A1, stage 52: exit code 0, no occurrence of the Chinese word for "not found" in the
output.
Row A2, stage 31: exit code 0, no occurrence of the Chinese word for "not found" in the
output.

Occasion B, the stage's own self-test harness (source:
.claude/singlefs-ai-sop/scripts/stage-selftest.sh, function body of the main loop). For
every stage file, and for each of a "red" and a "green" fixture, it copies the fixture
files into a fresh temporary directory called work, then runs
"cd work && bash <stage-file-path> work". The stage-file-path passed to bash here is
always the real stage file's own absolute path inside the real repository; only the
argument (work) is a temporary copy. I reproduced this exact per-file loop body, using
only the two stages' existing fixture directories, and ran it.
Row B1, stage 52, red fixture: exit code 1; the output contains the sentence "does not
match, 1 place total" and names a path whose registry sequence disagrees with the
recorded log; no occurrence of "not found".
Row B2, stage 52, green fixture: exit code 0; the output says it compared 2 registered
entries; no occurrence of "not found".
Row B3, stage 31, red fixture: exit code 1; the output lists specific undecided items by
name; no occurrence of "not found".
Row B4, stage 31, green fixture: exit code 0; the output says 2 undecided items were
checked against both rulers; no occurrence of "not found".
Fact: the fixture directories for stage 31 (fixtures/31-blocking-verdict.sh/red and
/green) each contain exactly one decisions file and an expect file; neither one omits
the generator script or the decisions directory. The fixture directories for stage 52
(fixtures/52-segment-registry.sh/red and /green) each contain a layout file, a results
file and a replay script; neither one omits research/scripts/check-segment-registry.py.

Occasion C, the temporary git worktree built by "gate.sh --staged" (source:
.claude/singlefs-ai-sop/scripts/gate.sh, the block that handles the --staged flag, cited
by describing what each line does, not by line number). That code path takes whatever is
currently staged in the git index (the output of "git diff --cached"), creates a brand
new temporary git worktree checked out at the current commit HEAD, and then applies the
staged diff on top of that fresh checkout, before running the whole gate script inside
that temporary worktree.
Fact, checked just now: today, in the real repository, none of the four files this task
is about (52-segment-registry.sh, 31-blocking-verdict.sh, gen-decision-items.py,
check-segment-registry.py) are staged; "git status --porcelain" reports all four as
modified-but-not-staged, and reports the new fixtures/52-segment-registry.sh directory as
untracked. "git ls-tree HEAD" confirms that check-segment-registry.py and
gen-decision-items.py both already existed, as tracked files, at the commit HEAD, before
this round's edits.
Row C1, not executed (running it would itself perform git worktree operations, which is
outside what this task is allowed to do), reasoned from the two facts just above: with
nothing staged today, the temporary worktree would be an exact checkout of HEAD, so both
52-segment-registry.sh and 31-blocking-verdict.sh inside that worktree would be the
commit's old versions of the file, from before this round's edits, and would not contain
the exit-code-1 behavior this task is about at all.
Row C2, a hypothetical, also not executed: if this round's edits to those four files, and
the new fixtures directory, were staged first, then the temporary worktree would contain
this round's new code, and check-segment-registry.py and gen-decision-items.py would
both still be present in it, because they already exist as tracked files at HEAD
regardless of whether this round's patch also edits their content.

Occasion D, running the stage file directly by hand, outside the top-level gate script.
Stage 52's own file header documents this call form: "bash
.claude/gate.d/52-segment-registry.sh [project-root]". Stage 31's file header does not
document any such call form.
Row D1, stage 52, run as "bash .claude/gate.d/52-segment-registry.sh" from the project
root, no root argument given: exit code 0, no "not found".
Row D2, stage 31, run the same way, no root argument given: exit code 0, no "not found".
Row D3, stage 52, today's code, run as "bash .claude/gate.d/52-segment-registry.sh
/some/other/directory" from the project root (a relative path to the script itself), 
where /some/other/directory has its own research/scripts/check-segment-registry.py but
no .claude/gate.d subdirectory of its own: the real captured output included a literal
shell error line, "cd: .claude/gate.d/../..: No such file or directory", immediately
followed by "not found /research/scripts/check-segment-registry.py", exit code 1. Source
note: today's SCRIPT= assignment line in 52-segment-registry.sh does not redirect the
inner cd's error output to /dev/null, unlike the matching GEN= assignment line in
31-blocking-verdict.sh, which does.
Row D4, stage 52, today's code deleted, replaced with the version that existed in the
commit before this round (obtained with "git show HEAD:.claude/gate.d/52-segment-
registry.sh"), run the same way (relative path to the script, root argument
/some/other/directory): no "not found" text at all; instead it found and tried to run
/some/other/directory's own research/scripts/check-segment-registry.py (a stub text
file placed there for this test, not real python), and reported a Python Syntax Error
from that stub file, exit code 1.
Row D5, stage 31, today's code, run as "bash .claude/gate.d/31-blocking-verdict.sh
/some/other/directory" from the project root (a relative path to the script itself),
where /some/other/directory has its own .claude/scripts/gen-decision-items.py (a stub
text file, not real python) and its own .claude/kb/decisions directory with one valid
decision file in it: the real captured output was "not found the generator
/.claude/scripts/gen-decision-items.py", exit code 1.
Row D6, stage 31, the version from before this round (obtained the same way as row D4),
run the same way as row D5: no "not found" text at all; instead it found and tried to
run /some/other/directory's own .claude/scripts/gen-decision-items.py stub file, and
reported a Python Syntax Error from that stub file, exit code 1.
Row D7, stage 52, today's code, run with an absolute path to the script itself (bash
/home/fy5090/code/singlefs/.claude/gate.d/52-segment-registry.sh /some/other/directory):
no "not found" mentioning check-segment-registry.py; the script lookup itself succeeded
(it resolved to the real repository's own copy of check-segment-registry.py, and that
copy actually ran); the run then failed for an unrelated reason, "not found
/some/other/directory/.claude/kb/layout/01-first-txn.md" (a file check-segment-
registry.py itself reads from whatever root it is told to check), exit code 1.
Row D8, stage 31, today's code, run with an absolute path to the script itself, root
argument /some/other/directory: no "not found" mentioning gen-decision-items.py; the
generator lookup itself succeeded (it resolved to the real repository's own copy, and
that copy actually ran against /some/other/directory's own decisions folder); the run
then reported "no undecided item at all, nothing to judge in this stage", exit code 77,
because that folder's one decision file only had decided items, no undecided ones.

T7-Q1. For occasion A (rows A1, A2), does the branch that exits with code 1 for "not
found" ever get reached at all today? Answer yes or no for each of the two stages, and
say which row supports your answer.

T7-Q2. For occasion B (rows B1 through B4, plus the fixture-content fact that follows
them), does the "not found" branch ever get reached by today's fixtures, for either
stage? Answer yes or no for each stage, and say which row or fact supports your answer.

T7-Q3. For occasion C (rows C1, C2), in the state the real repository is in today (row
C1), does the exit-code-1 "not found" behavior even exist in the temporary worktree that
gate.sh --staged would build? Answer yes or no, and say why, using only row C1.

T7-Q4. For occasion D, rows D3 and D5 both show today's code reaching the exit-code-1
"not found" branch. For each of those two rows, judge whether reaching that branch there
counts as a false red (the branch fires even though the file genuinely exists and could
have been found), as a correct red (the branch fires and the file genuinely is not
reachable at all, by any reasonable path), or as something in between. Give one sentence
of reasoning for each of the two rows.

T7-Q5. Rows D4 and D6 show what the code looked like one commit before this round, run
under the exact same conditions as D3 and D5. Using rows D3, D4, D5 and D6 together,
answer this question: is there an occasion, among the ones listed in this document,
where the fallback that stage 31 removed (falling back to a path relative to the current
working directory, after the path based on the script's own location failed) used to be
the only way the generator actually got found? Answer yes or no, and name the row
numbers your answer rests on.

T7-Q6. Rows D7 and D8 show the same kind of standalone run as D3 through D6, but with an
absolute path to the script itself instead of a relative one. Does the "not found"
branch this task is about get reached in either D7 or D8? Answer yes or no for each row.

T7-Q7. Across every row in occasions A through D (not counting the two hypothetical rows
C1 and C2, since those were not executed), count how many rows actually reached the
specific "not found the script" or "not found the generator" branch this task is about.
Give the count and list the row numbers.

T7-Q8. Based only on the rows you listed in T7-Q7, would you describe today's exit-1
behavior as something that is exercised by any of this project's own automated fixtures
(the ones described in occasion B)? Answer yes or no, and name the row that supports it.

Part three, called T8, is about a python function named clip, in
.claude/scripts/gen-decision-items.py. It takes a piece of text and a target length n,
and is supposed to cut the text down to at most n characters without leaving a
half-finished parenthetical remark or a bare, meaningless reference to a decision or
experiment number hanging off the end.

Exhibit 1 is the source code of the function named clip, extracted automatically from
the file using Python's own ast module, then lightly retyped in exactly three ways for
this plain-text document (all three described in the note right after it): the
full-width parenthesis characters were swapped for plain ASCII parentheses, the
non-ASCII characters inside the second regular expression's character class were
swapped for a placeholder, and every slice that Python would normally write with a bare
leading colon before the position n was rewritten to spell out its starting position 0
instead (the two forms slice identically in Python; the rewrite exists only because this
document's own automated corruption checker misreads a bare colon sitting right before a
letter as a sign of lost text). The docstring and the comments were also dropped, since they are not
needed to answer the questions below and this task asked for real behavior, not prose
about the mechanism. Nothing else was changed.

def clip(text, n):
    t = text[0:n]
    while t.count('(') > t.count(')'):
        t = t[0:t.rindex('(')].rstrip()
    if t == text:
        return t.rstrip()
    while True:
        t2 = re.sub(r'(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$', '', t)
        t2 = re.sub(r'[TRAILING-PUNCTUATION-AND-CONNECTIVES]+$', '', t2)
        if t2 == t:
            break
        t = t2
    return t.rstrip()

Note on Exhibit 1: the real file uses the Chinese full-width parenthesis characters
( U+FF08 and U+FF09 ) instead of the plain ASCII parenthesis shown above. The token
"[TRAILING-PUNCTUATION-AND-CONNECTIVES]" stands in for the real character class, which
in the real file matches one or more of any of these, in any combination: whitespace, a
forward slash, a Chinese enumeration comma, an ASCII comma, a full-width comma, a middle
dot, and three separate single Chinese characters used as connective particles that mean
"of", "and", and a second, different word that also means "and". Those characters could
not be safely reproduced in this plain-text document, but every case in the table below
was run against the real, unmodified file, not against this simplified listing; rows 17
through 20 below show concretely, in their own before-and-after text, which trailing
characters this class actually strips.

Exhibit 2 is a table of every case that was fed into this exact function. Every row was
produced by literally calling clip(text, n) in a live Python process that had loaded
Exhibit 1's real, unmodified counterpart from the actual file; nothing in this table was
computed by hand or guessed by reasoning about the regular expression.

In every row below, "text" is the full original string handed to clip, "n" is the target
length handed to clip, "t=text[0:n]" is what Python's own slicing produces before clip
does anything else to it (shown so you can see exactly how much of the original string
survived the initial cut), and "output" is the exact return value of clip(text, n). Rows
1 through 11 were built so that t=text[0:n] stops exactly right after one particular
domain word or id.

Row 1 (stops right after SHA256). text: "这一段正文讲的是校验方案取 SHA256 这一档，后面还有很多字要被截掉的内容"
n: 20. t=text[0:n]: "这一段正文讲的是校验方案取 SHA256". output: "这一段正文讲的是校验方案取"

Row 2 (stops right after RAID5). text: "这一段正文讲的是校验方案取 RAID5 这一档，后面还有很多字要被截掉的内容"
n: 19. t=text[0:n]: "这一段正文讲的是校验方案取 RAID5". output: "这一段正文讲的是校验方案取"

Row 3 (stops right after CRC32C). text: "这一段正文讲的是校验方案取 CRC32C 这一档，后面还有很多字要被截掉的内容"
n: 20. t=text[0:n]: "这一段正文讲的是校验方案取 CRC32C". output: "这一段正文讲的是校验方案取 CRC32C"

Row 4 (stops right after AES-256). text: "这一段正文讲的是校验方案取 AES-256 这一档，后面还有很多字要被截掉的内容"
n: 21. t=text[0:n]: "这一段正文讲的是校验方案取 AES-256". output: "这一段正文讲的是校验方案取"

Row 5 (stops right after SHA-256). text: "这一段正文讲的是校验方案取 SHA-256 这一档，后面还有很多字要被截掉的内容"
n: 21. t=text[0:n]: "这一段正文讲的是校验方案取 SHA-256". output: "这一段正文讲的是校验方案取"

Row 6 (stops right after I-9.5). text: "这一段正文讲的是校验方案取 I-9.5 这一档，后面还有很多字要被截掉的内容"
n: 19. t=text[0:n]: "这一段正文讲的是校验方案取 I-9.5". output: "这一段正文讲的是校验方案取"

Row 7 (stops right after D22). text: "这一段正文讲的是校验方案取 D22 这一档，后面还有很多字要被截掉的内容"
n: 17. t=text[0:n]: "这一段正文讲的是校验方案取 D22". output: "这一段正文讲的是校验方案取"

Row 8 (stops right after E142). text: "这一段正文讲的是校验方案取 E142 这一档，后面还有很多字要被截掉的内容"
n: 18. t=text[0:n]: "这一段正文讲的是校验方案取 E142". output: "这一段正文讲的是校验方案取"

Row 9 (stops right after M1). text: "这一段正文讲的是校验方案取 M1 这一档，后面还有很多字要被截掉的内容"
n: 16. t=text[0:n]: "这一段正文讲的是校验方案取 M1". output: "这一段正文讲的是校验方案取"

Row 10 (stops right after L1). text: "这一段正文讲的是校验方案取 L1 这一档，后面还有很多字要被截掉的内容"
n: 16. t=text[0:n]: "这一段正文讲的是校验方案取 L1". output: "这一段正文讲的是校验方案取"

Row 11 (stops right after T10). text: "这一段正文讲的是校验方案取 T10 这一档，后面还有很多字要被截掉的内容"
n: 17. t=text[0:n]: "这一段正文讲的是校验方案取 T10". output: "这一段正文讲的是校验方案取"

Rows 12 through 14 were built so that t=text[0:n] stops in the middle of a word, before
the word that was being cut off is complete.

Row 12 (stops in the middle of the letters, before any digit of SHA256 appears at all).
text: "这一段正文讲的是校验方案取 SHA256 这一档，后面还有很多字"
n: 16. t=text[0:n]: "这一段正文讲的是校验方案取 SH". output: "这一段正文讲的是校验方案取 SH"

Row 13 (stops in the middle of the digits of SHA256, one digit short of the full number).
text: "这一段正文讲的是校验方案取 SHA256 这一档，后面还有很多字"
n: 19. t=text[0:n]: "这一段正文讲的是校验方案取 SHA25". output: "这一段正文讲的是校验方案取"

Row 14 (stops in the middle of the letters, before any digit of RAID5 appears at all).
text: "条带冗余方案按 RAID5 来配置，后面还有很多字要被截掉的内容"
n: 11. t=text[0:n]: "条带冗余方案按 RAI". output: "条带冗余方案按 RAI"

Rows 15 and 16 were built so that n is not smaller than the length of text, meaning no
real truncation happens at all.

Row 15 (n equals the exact length of text). text: "这条决策详见 D22"
n: 10 (this is exactly the length of text). t=text[0:n]: "这条决策详见 D22". output: "这条决策详见 D22"

Row 16 (n is larger than the length of text). text: "这条决策详见 D22"
n: 30 (this is larger than the length of text, which is 10). t=text[0:n]: "这条决策详见 D22". output: "这条决策详见 D22"

Rows 17 through 20 were built so that t=text[0:n] is a genuine truncation (the original
text is longer than n, with real content still cut off after position n) that happens to
stop right after a trailing connective character or punctuation mark.

Row 17 (stops right after the character meaning "of"). text: "这是一段用来测试连接词截断行为的示例文字说明结尾是的，后面还有内容没截到"
n: 26. t=text[0:n]: "这是一段用来测试连接词截断行为的示例文字说明结尾是的". output: "这是一段用来测试连接词截断行为的示例文字说明结尾是"

Row 18 (stops right after the character meaning "and"). text: "这是一段用来测试连接词截断行为的示例文字说明结尾是与，后面还有内容没截到"
n: 26. t=text[0:n]: "这是一段用来测试连接词截断行为的示例文字说明结尾是与". output: "这是一段用来测试连接词截断行为的示例文字说明结尾是"

Row 19 (stops right after a full-width comma). text: "这是一段用来测试连接词截断行为的示例文字说明结尾是逗号，，后面还有内容没截到"
n: 28. t=text[0:n]: "这是一段用来测试连接词截断行为的示例文字说明结尾是逗号，". output: "这是一段用来测试连接词截断行为的示例文字说明结尾是逗号"

Row 20 (stops right after a Chinese enumeration comma). text: "这是一段用来测试连接词截断行为的示例文字说明结尾是顿号、，后面还有内容没截到"
n: 28. t=text[0:n]: "这是一段用来测试连接词截断行为的示例文字说明结尾是顿号、". output: "这是一段用来测试连接词截断行为的示例文字说明结尾是顿号"

Rows 21 and 22 were built so that t=text[0:n] stops in the middle of a Chinese-style
parenthetical remark, before its closing full-width parenthesis appears.

Row 21 (parenthetical remark cut off, containing no id). text: "写放大与读放大的权衡（含 SSD 与 HDD 的对照）继续说明这件事"
n: 16. t=text[0:n]: "写放大与读放大的权衡（含 SSD". output: "写放大与读放大的权衡"

Row 22 (parenthetical remark cut off, containing a decision id, D22, inside it). text: "写放大与读放大的权衡（含 D22 的对照）继续说明这件事"
n: 16. t=text[0:n]: "写放大与读放大的权衡（含 D22". output: "写放大与读放大的权衡"

T8-Q1. For each row from 1 to 22, classify its output into exactly one of these four
categories, and answer with the row number followed by the category name: category
half-stripped-word means the output still ends with part of a word that was cut in the
middle (neither the whole word nor none of it); category dropped-whole-word-unnecessarily
means the output is missing an entire word or id that would have fit within n characters
of a real truncation, even though nothing about that word itself needed removing;
category left-a-bare-reference means the output still ends with a decision, experiment or
similar id or a domain word glued directly onto the text with nothing separating it
(the source file's own comment says this exact shape is what the project's separate doc-lint style check would flag red);
category correct means none of the first three apply. A single row may only get one
category; pick the one that best fits.

T8-Q2. Give the total count of rows in each of the four categories from T8-Q1. The four
counts must add up to 22.

T8-Q3. The round's own description of this regular expression claims: "when it has a
left boundary, a domain word sitting at the truncation point either survives whole or
gets stripped whole; it can never be left half-stripped." Using only the categorization
you gave in T8-Q1, is there at least one row that refutes this claim? If yes, name the
row number and say which category you put it in. If no, say so plainly.

T8-Q4. Compare row 12 and row 13. Both start from the exact same original text and both
cut somewhere inside the digits-and-letters of SHA256, just at two different points. Row
12 cuts before any digit of SHA256 has appeared yet; row 13 cuts after some of the
digits have already appeared. Do these two rows end up in the same category from T8-Q1,
or different categories? Name the category for each.

T8-Q5. Row 3 (CRC32C) and row 1 (SHA256) were both built the same way, stopping right
after a domain word. Their outputs are different: row 1's output no longer contains
SHA256 at all, but row 3's output still contains CRC32C. Using only the shape of the
regular expression in Exhibit 1 (it requires at least one digit, \d+, immediately after
the letters, before the end of the string), explain in one or two sentences why CRC32C
behaves differently from SHA256 here. Then say what observation would refute your
explanation.

T8-Q6. Row 22 shows an unbalanced parenthetical remark that happens to contain a
decision id, D22, inside it. Row 6 through row 8 show truncations that stop right after
a bare id (D22 or similar) with no parenthetical remark involved. Does the parenthetical
remark's own stripping step in Exhibit 1 (the first while loop, which runs before the id
is even considered) end up removing the D22 in row 22 by itself, or does the id-stripping
step do it? Answer using only what the t=text[0:n] value in row 22 looks like and what the
final output looks like; you do not need to run anything.

This is the end of the task. Answer T4-Q1 through T4-Q7, then T7-Q1 through T7-Q8, then
T8-Q1 through T8-Q6, in that order, each on its own numbered line or short paragraph.
