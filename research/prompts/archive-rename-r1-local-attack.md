SINGLEFS ARCHIVE AND RENAME REVIEW - LOCAL ATTACK LEG - ROUND archive-rename-r1

You are one of three independent reviewers checking a set of engineering
changes for singlefs, a copy on write filesystem being designed from
scratch. You are given a self contained set of facts below. Do not assume
anything not stated here. Read carefully, every qualifier in this text is
load bearing; a definition with one word removed can flip an answer. Do
not use any markdown emphasis anywhere in your answer: no bold text, no
italic text, no backtick code formatting, no asterisk bullets. Number
your answers to match the question numbers given at the end. For every
numbered answer you give, add one sentence starting with "This would be
refuted by:" describing the specific observation that would prove that
answer wrong. Where a question asks you to fill in a table, give the
table with an explicit value and an explicit one sentence justification
for every single cell, not just "yes" or "no" for the whole table, and do
not merge cells together even where you believe several rows behave
identically; each cell must carry its own explicit verdict and its own
justification. When your answer needs to point at a source, point at it
by its fact number given below, or by naming the scenario letter and
column name of the table you are answering. Do not write any source file
name together with a line number, and do not write any line number at
all, anywhere in your answer, because the underlying files may have moved
to different line numbers by the time anyone reads your answer; refer to
a function or script by its name, and refer to a table row by its
scenario letter or row number as given here.

This document asks two independent groups of questions: group A about a
replay script's status labels, and group B about an exemption list used
by a terminology sweep tool. The two groups do not share any fact except
the shared background paragraph below; do not let an answer to one group
depend on a fact given only in the other group.

Shared background. Singlefs keeps two kinds of records checked into its
repository: source code, and a set of retained experiment artifacts under
a directory called results. A separate policy, adopted by the project,
states that every commit deletes the experiment artifacts kept by the
previous round and keeps only the current round's artifacts; anything
deleted this way can still be found in the version control history, by
looking up the commit that deleted it. Separately, the project just
renamed a term project wide: the word "superblock" (and its case and
spelling variants, such as SuperBlock, super_block, SUPERBLOCK, and
abbreviations such as sb) was renamed to "system configuration" (and
matching variants) across the repository; the new spelling took effect
for English identifiers starting 2026-09-20, and the renaming was
completed across the rest of the repository by 2026-09-21. An automatic
sweep tool
performs this renaming across the repository, except in a short list of
exempted paths and files, given in full in group B below.

GROUP A. A script called replay.sh reruns a fixed list of experiments and
compares each freshly produced output against a retained artifact checked
into the repository, printing one status label per experiment row.

Fact 1. The script's comparison logic for one experiment row runs the
following steps in this fixed order, stopping at the first step whose
condition is true.

Step 1: test whether a file at a specific path, referred to below as the
registered path for that row, exists on disk right now, using the shell
test "the registered path is not a regular file". If this test is true
(the file does not exist as a regular file at that exact path right now),
the script prints the status label "archived", together with a message
stating that the registered path is not currently in the tree and that
this run was still able to produce a fresh output for it, and it does not
perform any byte level comparison for this row.

Step 2: if step 1's test was false (a regular file does exist at the
registered path), compare the fresh output against that file byte for
byte. If they are identical, print the status label "byte identical".

Step 3: if they are not byte for byte identical, strip out substrings
related to timing (such as elapsed time numbers) from both the fresh
output and the registered file, and compare again. If the two are
identical after stripping timing substrings: when the row's declared kind
is "timing", print the status label "timing only differs"; when the row's
declared kind is anything other than "timing", print the status label
"criteria written wrong" instead.

Step 4: if the two are still different even after stripping timing
substrings, print the status label "mismatch", together with a count of
differing lines.

Fact 2. The script's own inline comment attached to step 1 states the
reasoning for step 1 in these terms: once a retained artifact has already
been removed from the working tree by the deletion policy described in
the shared background above, this comparison tier is not applied to it;
the reason given is that reporting it as "mismatch" would look exactly
identical to the case where the apparatus has genuinely regressed and a
rerun now produces different bytes, which would make a reader believe the
experiment is broken when it is not.

Fact 3. When the count of rows labeled "archived" is not zero, the
script's summary section prints an additional explanatory block,
noting again that the byte for byte comparison tier has no counterpart
to compare against for these rows. That block states: this is not a
failure; every one of these rows could still
be produced by this run, and separate conclusion interval assertions are
still checked as usual for these rows; to look at the artifact as it
existed at the time, use the version control history to find the commit
that deleted it and read its content back from that commit; what is read
back that way is the data as of that past time, not today's conclusion,
and using it to support any new conclusion requires rerunning the
experiment first.

Fact 4. Step 1's test looks only at whether a regular file currently
exists at the registered path string. It does not check whether that
exact path string was ever tracked by version control, it does not check
whether the path string itself was constructed correctly by the script,
and it does not check whether some other artifact with the intended
content exists under a different path.

Fact 5. In the shell language this script is written in, the specific
test used in step 1 also evaluates to "not a regular file" (making step
1's condition true) in each of the following cases, in addition to the
case where the file was genuinely deleted by the policy in the shared
background: when the registered path string itself is wrong due to any
clerical mistake (such as a typo, a wrong directory segment, or a
leftover unresolved placeholder) so that nothing exists at that exact
string even though a file with the intended content exists on disk under
a different, correct path; and when the path is a symbolic link whose
target has been deleted, even if the symbolic link entry itself is still
tracked by version control.

Now the scenarios. For each scenario below, assume the row's declared
kind is not "timing", unless the scenario says otherwise.

Scenario a. The row's registered path was genuinely deleted by the shared
background's deletion policy after a past round; no file of any kind
exists at that exact path today; the true, intended content, if it were
rerun, would match what is now produced.

Scenario b. The row's registered path string contains a clerical mistake
(such as a typo or a leftover placeholder) introduced by an unrelated
editing step; nothing exists at that exact, slightly wrong string;
however, a file with the intended content, produced by an earlier correct
run, still exists on disk right now under the correct, intended path, a
path different from the registered path string; nobody's comparison step
reads that correct path for this row.

Scenario c. The row's registered path is a symbolic link that is still
tracked by version control; the symbolic link's target file has been
deleted; the symbolic link entry itself is still physically present in
the working tree.

Scenario d. The row's registered path exists as an ordinary regular
file, and that file's content is different, byte for byte, from what a
correct rerun of the experiment now produces; this difference reflects
an actual regression in the code under test, not a timing artifact.

Scenario e. The row's registered path exists as an ordinary regular
file with zero bytes of content, distinct from what a correct rerun of
the experiment now produces.

QUESTIONS FOR GROUP A.

Question 1. For each of scenario a, b, c, d, and e, fill in a row of a
table with these four columns: column 1, does step 1's test evaluate
true or false for this scenario; column 2, which status label does the
script print for this row (archived, byte identical, timing only
differs, criteria written wrong, or mismatch); column 3, does the
printed label, by itself, correctly tell a reader whether the underlying
apparatus and its output are known to be fine, known to be regressed, or
genuinely unknown either way; column 4, one sentence justification for
column 3, naming the specific fact number that supports it. After the
table, add one sentence starting with "This would be refuted by:"
describing what observation about the script's actual behavior would
show any single cell in your table to be wrong.

Question 2. Based only on the table you just filled in, does the
"archived" status label have the discriminating power to tell scenario a
apart from scenario b, when a reader sees only the printed label and the
message text quoted in fact 1's step 1 and fact 3, without independently
rereading the script's source code. Answer with the words "can
discriminate" or "cannot discriminate", followed by one sentence of
justification naming the specific scenario letters that would print the
identical label and message despite having different underlying truth,
or explaining why no such collision exists. Add the required "This would
be refuted by:" sentence.

Question 3. Does the explanatory block described in fact 3 state,
anywhere in the wording given, a distinction between scenario a on one
hand and scenario b or scenario c on the other hand. Answer with the
word "yes" or "no", followed by one sentence pointing at the specific
clause in fact 3 that would have to state such a distinction if it
existed. Add the required "This would be refuted by:" sentence.

Question 4. Consider scenario d by itself, without comparing it to any
other scenario. Is scenario d, by the logic given in fact 1, ever capable
of being printed with the status label "archived" instead of "mismatch".
Answer with "yes" or "no" and one sentence justification citing the
relevant step number from fact 1. Add the required "This would be
refuted by:" sentence.

GROUP B. A separate exemption list, used by the automatic sweep tool
described in the shared background, names six paths or files that the
sweep tool skips. For each exempted path or file, the reason the
exemption exists is written next to it. The six rows, given here in
full and in the order they appear in the list, are as follows.

Row 1. Path: a file of research notes about designs used by other
organizations. Reason given: this file discusses designs from other
organizations (three named systems are given as examples), containing
those organizations' own terminology and direct quotations from their
original documentation; renaming their word for this concept to this
project's new word would turn the quotation into something false; this
exemption was a specific user decision dated 2026-09-21.

Row 2. Path: a directory of warning records. Reason given: warning
records in this directory are frozen by the date they were written;
what such a record documents is exactly the fact that a particular word
was renamed on a particular date; if the old word were itself swapped
out for the new word inside these records, the sentence describing the
renaming event would lose its point.

Row 3. Path: a local directory holding a copy of a shared process
document, called an SOP, maintained upstream, outside this project.
Reason given: this directory is a local copy of a document that can
only be edited in the upstream location; editing this local copy in
place would be overwritten and lost the next time this project
synchronizes with the upstream copy.

Row 4. Path: the script file that implements the automatic sweep tool
itself. Reason given: this script's own internal replacement table and
its own detection regular expressions must literally contain the
spelled out old word, in order to know what to look for and what to
replace it with; sweeping this file would remove the tool's own ability
to ever rename anything again.

Row 5. Path: the exemption list file itself, the file containing these
six rows. Reason given: an exemption reason has to be able to spell out
the old word being exempted, in order to say clearly, in words, what is
being exempted from the sweep.

Row 6. Path: a single file recording the authoritative old to new
mapping for every term ever renamed in this project. Reason given:
writing the sentence "the old word was renamed to the new word"
requires being able to write the old word somewhere; the old word needs
at least one place in the project where it can still be looked up, so
that historical artifacts and old commits, which still contain the old
word, can be matched back to what they mean today.

Fact 6. The renaming event that row 1 through row 6 above all refer to
is the single project wide renaming of "superblock" to "system
configuration" described in the shared background.

QUESTIONS FOR GROUP B.

Question 5. For each of row 1 through row 6, classify it into exactly
one of these three categories, and give one sentence of justification
for the classification, referring only to the reason given for that row
above.

Category X: the old word being preserved at this row's path belongs to
another organization's own terminology, or is part of a direct
quotation of another organization's original documentation; it is not a
word this project chose for its own design.

Category Y: the old word being preserved at this row's path appears
only inside a tool, script, or table whose subject matter is the
renaming mechanism itself (what word maps to what word, or how the
sweep works); it is not a word used in an ordinary descriptive sentence
about this project's own filesystem design.

Category Z: the old word being preserved at this row's path is this
project's own design terminology, appearing in an ordinary descriptive
sentence about this project's own filesystem, a sentence that is
neither about another organization's system nor about the renaming
mechanism itself.

Give your answer as six lines, one per row, each stating the row
number, the single category letter chosen, and the one sentence
justification.

Question 6. List every row number, if any, that you placed in category
Z in question 5. If you placed no row in category Z, answer with the
word "none". Add the required "This would be refuted by:" sentence,
describing what a reviewer would have to find in that row's reason text,
quoted above, to change your classification.

Question 7. Take whichever row or rows you named in question 6, if any.
For a row named there, does exempting that path from the automatic
sweep mean that any other ordinary descriptive sentence about this
project's own design, located anywhere else within that same exempted
path, would also be skipped by the sweep indefinitely, with no separate
mechanism described anywhere in the facts above that would ever catch
it and force it to be updated. Answer "yes" or "no" for each row you
named, with one sentence of justification citing which fact above does
or does not describe such a separate mechanism. Add the required "This
would be refuted by:" sentence. If you named no row in question 6,
answer this question with the single word "none".

END OF DOCUMENT.
