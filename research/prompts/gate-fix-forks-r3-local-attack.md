This is a two-part fact-checking task about a software project's internal CI gate
scripts. You are not asked to run any code yourself. Every fact below was obtained by
actually reading the source files in the project's own checked-out working tree, or by
actually running the commands and capturing their real output, and is given to you
verbatim. Your job is to judge, cell by cell, what the given facts show.

Formatting rules for your answer.
1. Do not use any markdown emphasis anywhere in your answer: no bold, no italics, no
   headings marked with hash signs, no backtick-quoted spans. Plain sentences only.
2. Number your answers to match the question numbers below, for example T4-Q1, T8-Q1.
3. For every answer that states a conclusion, add one sentence that starts with the
   words "This would be refuted if" and names a concrete observation.
4. Do not cite a source-code line number or a file line number anywhere in your answer.
   If you need to point at a specific fact, point at it by row number in a table, or by
   the name of a function or a variable, not by a line number.
5. Everything you need is in this document. You do not have a shell, so do not propose
   running a command as part of your final answer; give the answer itself.
6. Always put a space after a colon, a semicolon, or a comma when one of those marks is
   immediately followed by a letter. Do not write two colons glued directly to a letter
   with no space, even when quoting a piece of program source; if you quote such a thing,
   insert a single space right after the punctuation and say in words that you inserted
   it.

Part one, called T4, is about a check named stage 80, in a file named
80-absolute-assertions.sh. Its job, today, is this: every experiment binary must carry at
least one assertion that is pinned to an absolute value, not only a relative comparison
between several arms. The check's own header comment states its scope this way: every
file directly inside one fixed directory, research/e7-index-bench/src/bin, plus, in any
other crates times src/bin or research times src/bin directory (crates/*/src/bin,
research/*/src/bin), every file whose name starts with the letter e, followed by one or
more digits, followed by an underscore (the written form for an experiment number,
registered in a project file named .claude/abbreviations as e<digits>); recognition goes by whether the file
is an experiment, not by which directory it sits in, so an experiment living under
crates/ follows the same rule as one under research/. The same header comment also says:
elsewhere, a file whose name does not start with that e<digits>_ pattern is treated as an
apparatus tool, for example one that compares a device log against ground truth field by
field; such a file is not judged at all, it is only listed by name on the stage's success
line, with that list computed fresh on every run.

Today, research/e7-index-bench/src/bin does not exist in the working tree being judged in
this task, so it contributes zero files. The only other src/bin directories that exist
under crates/ or research/ are singlefs-harness's, which contains exactly 5 files. Here is
the fact table for all 5.

Row 1. Path: crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs. Its own
first doc-comment sentence, copied verbatim (it runs across two source lines in the real
file): "E156 rerun, second time, first stage: the cost numbers for the four alloc-basis
forks, doing only fork 7 (the defer check) and its prerequisite S1 and anchor (pre
registered in research/prompts/e156-r2-prereg.md, part five, 5.7, first paragraph)."
Fact, stated separately from the doc comment: this row's source file was rewritten,
wholesale, by a different, unrelated working session, after this project's previous round
of three-way review of this same stage 80 question had already finished and handed back
its verdict; the numbers below describe the file exactly as it stands today, not as it
stood during that previous round. Total assert, assert_eq or assert_ne macro calls found
in the file: 28. Of those, calls that match the exact pattern stage 80 itself uses to
count as an absolute-value assertion (an assert_eq call whose second argument is a bare
number literal, or an assert call containing a comparison operator directly followed by a
bare number literal): 3. Whether research/scripts/replay.sh, the project's shared replay
script, contains a line that builds and runs this exact binary: yes; that file contains a
function named driver_e156, and its body runs the line "cargo run -q -p singlefs-harness
--bin e156_allocation_basis_counts". Filename matches the scope pattern e<digits>_: yes.

Row 2. Path: crates/singlefs-harness/src/bin/e158_root_choice_repair.rs. This file does
not exist at all in the copy of the working tree used for the project's previous round of
review of this same question; it is new since then. Its own first doc-comment sentence,
copied verbatim: "E158, root choice and repair, four forks, pre registration, first
stage: the apparatus to be checked into the results store." Total assert, assert_eq or
assert_ne macro calls found in the file: 9. Of those, calls matching stage 80's
absolute-value pattern: 3. Whether replay.sh contains a line that builds and runs this
exact binary: yes; that file contains a function named driver_e158, and its body runs the
line "cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- all".
Filename matches the scope pattern e<digits>_: yes.

Row 3. Path: crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs. Its
own first doc-comment sentence, copied verbatim: "Host side: take the two device-side
logs recorded by QEMU blklogwrites, and compare them field by field, disk by disk,
against the recorded stream obtained by replaying the same write path on the host, same
parameters, same bytes." That doc comment is then followed by a usage line for the
command's own arguments, not more prose. Total assert, assert_eq or assert_ne macro calls
found in the file: 0. Calls matching stage 80's absolute-value pattern: 0. Whether
replay.sh contains a line that builds and runs this exact binary: no; that file's text
does not mention this binary's name anywhere. This exact file's own doc comment does not
name any script that invokes it either; no other script was available to check in this
task. Fact drawn from a different file, row 4's file, not from this file itself: row 4's
own doc comment states that its binary, first_transaction_on_device, is fed into a
virtual machine, and names this row's binary, first_transaction_device_log_check, as the
one that later judges, on the host side, whether what actually reached the disk matches
what the program believed it sent. Filename matches the scope pattern e<digits>_: no.

Row 4. Path: crates/singlefs-harness/src/bin/first_transaction_on_device.rs. Its own
first doc-comment sentence, copied verbatim: "Virtual-machine tier, QEMU or KVM: run the
whole write path of the first transaction on two real virtio disks, cold-reboot, then
recover and read the file back." Total assert, assert_eq or assert_ne macro calls found
in the file: 44. Of those, calls matching stage 80's absolute-value pattern: 4. Whether
replay.sh contains a line that builds and runs this exact binary: no; that file's text
does not mention this binary's name anywhere. This file's own doc comment (a later line
than the first sentence quoted above) states, in its own words, that it is fed into a
virtual machine by a script named research/scripts/vm-bench.sh; that script was not
available to check in this task, so this claim is only a quote of the binary's own
comment, not independently re-confirmed this round. That same later line also states, in
its own words, that this binary only judges what it can judge by itself (recovering the
file correctly, and the sequence of on-disk segments), and that a separate question,
whether what actually arrived on disk matches what the program believed it sent, is left
to row 3's binary, first_transaction_device_log_check, to judge afterward on the host
side. Filename matches the scope pattern e<digits>_: no.

Row 5. Path: crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs. Its own
first doc-comment sentence, copied verbatim (it runs across three source lines in the
real file): "E142, first-transaction dry run, the implementation side of measurement 5:
run scenario:: run_first_transaction on two in-memory disks (one space was inserted right
after the double colon in the name just given; the real file has no space there, this
insertion follows formatting rule 6 above and is not a sign of any missing text), same
parameters as the virtual-machine tier and the E142 apparatus: same fsid, same write
timestamp, same 3000-byte content, two 4 GiB disks, and print the bytes written to those
21 regions by the first transaction, one result line per region." Total assert, assert_eq
or assert_ne macro calls found in the file: 0. Calls matching stage 80's absolute-value
pattern: 0. Whether replay.sh contains a line that builds and runs this exact binary:
yes, but only as one step inside a function named driver_e142, not as a dedicated
function of its own the way rows 1 and 2 have; driver_e142 runs "cargo run -q -p
singlefs-harness --bin first_transaction_region_bytes" and captures its output into a
temporary file, which is then handed as a command-line argument to a different binary
that performs the actual comparison and prints the assertions, if any; no assertion about
absolute values lives inside first_transaction_region_bytes.rs itself. Filename matches
the scope pattern e<digits>_: no.

T4-Q1. For row 1, judge whether this file is better described as an experiment (a
program whose own job is to run several arms and compare them, feeding a conclusion that
a decision can be based on) or a tool (a program whose job is to compare recorded data
against one external ground truth, not to run several arms against each other). Name the
single fact from row 1 that most supports your answer.

T4-Q2. Same question as T4-Q1, for row 2.

T4-Q3. Same question as T4-Q1, for row 3.

T4-Q4. Same question as T4-Q1, for row 4.

T4-Q5. Same question as T4-Q1, for row 5.

T4-Q6. Count: across rows 1 through 5, in how many rows does your T4-Q1 through T4-Q5
classification, experiment or tool, disagree with that row's own filename-matches-scope
fact? Disagreement means either you classified the row as a tool but its filename does
match the pattern, or you classified it as an experiment but its filename does not match.
Give the count and list the row numbers where it disagrees.

T4-Q7. Describe one concrete, hypothetical future file, by giving all of these
properties: which kind of directory it would sit in (crates/*/src/bin or
research/*/src/bin), what its filename would have to start with, one sentence for what
its doc comment would say, and whether its own assertions would be a relative comparison
between several arms or a comparison against one external ground truth. The file you
describe must be a case where today's filename-based scope rule would leave it out of
stage 80's scope even though, by its own content, it is the kind of multi-arm experiment
the rule is meant to catch. State what you would observe, if this file actually existed
and stage 80 were run on it, that would refute your claim that it is left out of scope.

T4-Q8. Same request as T4-Q7, but for the opposite mistake: describe one concrete,
hypothetical future file that today's filename-based scope rule would pull into stage
80's scope, even though by its own content it is a tool, a ground-truth comparator, not
the kind of multi-arm experiment the rule is meant to catch. State the refuting
observation the same way.

Part two, called T8, is about a Python function named clip, in a file named
gen-decision-items.py. It is handed a piece of text and a target length n, and its job is
to cut the text down to at most n characters without leaving a half-finished
parenthetical remark hanging open, and without leaving a bare, meaningless reference to a
decision, experiment or similar id number hanging off the end.

Exhibit 1 describes exactly what the real function does, in order, using the real
regular-expression shapes, but replacing the two full-width parenthesis characters used
in the real source with plain ASCII parentheses, and describing one small character class
in English instead of showing its raw characters (that class is explained fully, in
words, right where it is used below). Throughout, "ASCII word character" means exactly
one of: an uppercase or lowercase ASCII letter, an ASCII digit, a period, an underscore,
or a hyphen.

Step one. Let t be text sliced from position 0 up to position n (using Python's own
slicing, t = text[0:n]). If the text is longer than n, and the character of text sitting
right at position n is itself an ASCII word character, and the last character of t is
also an ASCII word character, then this truncation landed in the middle of an ASCII
token; in that case, remove the entire trailing run of ASCII word characters from the end
of t, then strip trailing whitespace.

Step two. While t contains more opening parentheses than closing parentheses, cut t back
to just before the last opening parenthesis, then strip trailing whitespace, and repeat.

Step three. If, after steps one and two, t is exactly equal to the original text
(meaning no real truncation happened at all), return t with trailing whitespace stripped,
and stop here; none of the remaining steps run.

Step four. Otherwise, repeat the following until it stops changing anything. Look at the
very end of t for a run that looks like an id: one or more uppercase ASCII letters,
optionally followed by a hyphen, then one or more ASCII digits, then zero or more repeats
of a period followed by one or more ASCII digits, sitting right at the end of t (trailing
whitespace is allowed after it), and only counted as a match at all if the character
sitting immediately before this run is not itself an ASCII word character. Call the
matched text, if any, the tail id. If a tail id was found, and the tail id's exact text is
not a member of a fixed registered set of domain words (described next), cut t back to
just before that tail id; otherwise leave t unchanged at this point. Then, from whatever
t is now, strip a trailing run of any of the following, in any combination: whitespace, a
forward slash, or one of six specific Chinese punctuation or connective characters (a
Chinese enumeration comma, an ASCII comma, a full-width comma, a middle dot, and two
single Chinese characters that are grammatical connectives roughly meaning "of" and
"and"). If, after this whole step, t did not change at all compared to before this step
ran, stop repeating; otherwise repeat step four again from the top.

Step five. Return t with trailing whitespace stripped.

The registered set of domain words used in step four is built like this: read every
markdown file found anywhere under a directory named .claude/kb (searched recursively),
and in each one, find every occurrence of an HTML comment of the exact shape
"less-than, exclamation mark, two hyphens, doc-lint:not-numbers, one or more of some
domain words separated by whitespace, two hyphens, greater-than"; the registered set is
the union of every domain word found this way, across every such comment in every such
file, read once and cached. This set is what step four checks a tail id against.

Exhibit 2 is a table of 24 rows. Every row was produced by literally calling the real,
unmodified clip function, loaded from the actual file, in a live Python process; the
registered set of domain words was reproduced by copying, verbatim, every
"doc-lint:not-numbers" marker line found anywhere under .claude/kb in this same project's
main working copy as of today, into a set of small stand-in files placed at the matching
relative paths under a scratch directory, and then running the real, unmodified clip
function with its current working directory pointed at that scratch directory (this
was necessary because the working tree being judged in this task does not itself carry
a full copy of .claude/kb). Nothing in this table was computed by hand or guessed by
reasoning about the regular expressions. In every row, "text" is the full original string
handed to clip, "n" is the target length handed to clip, "t at start" is exactly what
text[0:n] produces before clip does anything else (shown so you can see exactly how much
of the original string survived the initial cut), and "output" is the exact return value
of clip(text, n).

Rows 1 through 7 were built so that t at start stops exactly right after one particular
domain word, and that domain word is a member of the registered set described above.

Row 1 (registered word SHA256, stops right after it). text: "设备用 SHA256 做完整性校验，另附摘要文本用于核对"
n: 10. t at start: "设备用 SHA256". output: "设备用 SHA256".

Row 2 (registered word RAID5, stops right after it). text: "阵列走 RAID5 冗余方案，另附摘要文本用于核对"
n: 9. t at start: "阵列走 RAID5". output: "阵列走 RAID5".

Row 3 (registered word AES-256, stops right after it). text: "加密算法选 AES-256 分组密码，另附摘要文本用于核对"
n: 13. t at start: "加密算法选 AES-256". output: "加密算法选 AES-256".

Row 4 (registered word SHA-256, stops right after it). text: "摘要算法用 SHA-256 逐块计算，另附摘要文本用于核对"
n: 13. t at start: "摘要算法用 SHA-256". output: "摘要算法用 SHA-256".

Row 5 (registered word CRC32C, stops right after it). text: "校验和算法用 CRC32C 硬件加速，另附摘要文本用于核对"
n: 13. t at start: "校验和算法用 CRC32C". output: "校验和算法用 CRC32C".

Row 6 (registered word M1, stops right after it). text: "轮内记号 M1 指这一格判定，另附摘要文本用于核对"
n: 7. t at start: "轮内记号 M1". output: "轮内记号 M1".

Row 7 (registered word T10, stops right after it). text: "样本编号 T10 是这一格，另附摘要文本用于核对"
n: 8. t at start: "样本编号 T10". output: "样本编号 T10".

Rows 8 through 11 were built so that t at start stops exactly right after a genuine
id number (not a member of the registered set of domain words).

Row 8 (id D22, stops right after it). text: "承重面见 D22 的字段表，另附摘要文本用于核对"
n: 8. t at start: "承重面见 D22". output: "承重面见".

Row 9 (id E142, stops right after it). text: "干跑见 E142 的量五，另附摘要文本用于核对"
n: 8. t at start: "干跑见 E142". output: "干跑见".

Row 10 (id I-9.5, stops right after it). text: "不变量见 I-9.5 的判据，另附摘要文本用于核对"
n: 10. t at start: "不变量见 I-9.5". output: "不变量见".

Row 11 (id C510, stops right after it). text: "欠账见 C510 的登记，另附摘要文本用于核对"
n: 8. t at start: "欠账见 C510". output: "欠账见".

Rows 12 and 13 were built so that t at start stops in the middle of a registered domain
word, before that word is complete.

Row 12 (registered word SHA256, cut after just two of its letters). text: "设备用 SHA256 做完整性校验"
n: 6. t at start: "设备用 SH". output: "设备用".

Row 13 (registered word RAID5, cut after just three of its letters). text: "阵列走 RAID5 冗余方案设计"
n: 7. t at start: "阵列走 RAI". output: "阵列走".

Rows 14 and 15 were built so that t at start stops in the middle of a genuine id number,
before that id is complete.

Row 14 (id D22, cut after just two of its characters, "D2"). text: "承重面见 D22 的字段表说明"
n: 7. t at start: "承重面见 D2". output: "承重面见".

Row 15 (id E142, cut after just three of its characters, "E14"). text: "干跑见 E142 的量五说明"
n: 7. t at start: "干跑见 E14". output: "干跑见".

Rows 16 and 17 were built so that t at start stops in the middle of a Chinese-style
parenthetical remark that follows a genuine id number, before its closing full-width
parenthesis appears, and the id's own short name inside that parenthetical is what gets
cut off.

Row 16 (id D22 followed by an unclosed parenthetical remark naming it). text: "参考 D22（单元原子性怎么合成） 的字段表"
n: 8. t at start: "参考 D22（单". output: "参考".

Row 17 (id E142 followed by an unclosed parenthetical remark naming it). text: "参考 E142（第一个事务的干跑） 的量五"
n: 9. t at start: "参考 E142（第". output: "参考".

Row 18 was built the same way as rows 16 and 17, except the id in front of the unclosed
parenthetical remark is a registered domain word instead of a genuine id number.

Row 18 (registered word SHA256 followed by an unclosed parenthetical remark naming it).
text: "算法用 SHA256（安全散列算法二五六） 逐块计算完整性"
n: 12. t at start: "算法用 SHA256（安". output: "算法用 SHA256".

Row 19 was built so that t at start stops in the middle of a two-character registered
domain word, after only its first character.

Row 19 (registered word M1, cut after just its first character, "M"). text: "轮内记号 M1 指这一格"
n: 6. t at start: "轮内记号 M". output: "轮内记号".

Rows 20 through 22 were built so that t at start stops exactly right after an uppercase
word that is not a member of the registered set of domain words.

Row 20 (word NVMe, not a member of the registered set; its last letter is a lowercase
"e", so it does not end in a digit). text: "存储介质换成 NVMe 固态盘之后"
n: 11. t at start: "存储介质换成 NVMe". output: "存储介质换成 NVMe".

Row 21 (word SSD, not a member of the registered set; it has no digit in it at all).
text: "存储介质换成 SSD 固态盘之后"
n: 10. t at start: "存储介质换成 SSD". output: "存储介质换成 SSD".

Row 22 (word HDD4, not a member of the registered set; unlike rows 20 and 21, it does end
in a digit, so, letter by letter, it has the same shape as the ids in rows 8 through 11:
uppercase letters followed directly by a digit). text: "存储介质换成 HDD4 机械盘之后"
n: 11. t at start: "存储介质换成 HDD4". output: "存储介质换成".

Rows 23 and 24 were built so that n is not smaller than the length of text, meaning no
real truncation happens at all.

Row 23 (n is larger than the length of text; text contains a registered domain word).
text: "设备用 SHA256 校验"
n: 999 (the real length of text is 9). t at start: "设备用 SHA256 校验". output: "设备用 SHA256 校验".

Row 24 (n is larger than the length of text; text contains a genuine id number).
text: "承重面见 D22 说明"
n: 999 (the real length of text is 8). t at start: "承重面见 D22 说明". output: "承重面见 D22 说明".

T8-Q1. For each row from 1 to 24, classify its output into exactly one of these four
categories, and answer with the row number followed by the category name. Category
half-stripped-word means the output still ends with part of a word or id that was cut in
the middle, neither the whole thing nor none of it. Category
dropped-whole-word-unnecessarily means the output is missing an entire word or id that
would have fit within n characters of a genuine truncation, even though nothing about
that word or id itself needed to be removed. Category left-a-bare-reference means the
output still ends with a decision, experiment or similar id, or a domain word, glued
directly onto the surrounding text with nothing separating it, and that id or word had
its explanatory short name or the rest of its sentence cut away right after it. Category
correct means none of the first three apply. A single row may only get one category; pick
the one that fits best.

T8-Q2. Give the total count of rows in each of the four categories from T8-Q1. The four
counts must add up to 24.

T8-Q3. Compare row 12 and row 14. Both were cut so that t at start ends with exactly two
characters of the word or id that was being cut off, and both outputs came out the same
shape. Row 12 cuts into a registered domain word, SHA256; row 14 cuts into a genuine id,
D22. Are these two rows in the same category from T8-Q1, or different categories? Name
the category for each, and say whether step one alone (the step that runs before the
registered set is even consulted) is enough to explain why they end up the same, or
whether step four's registered-set check also has to be involved for this particular
pair.

T8-Q4. Rows 20, 21 and 22 all stop right after an uppercase word that is not a member of
the registered set. Rows 20 and 21 keep the whole word in the output; row 22 does not.
Using only the exact shape of the tail-id pattern described in step four (it requires one
or more digits, directly after the uppercase letters, with nothing else in between),
explain in one or two sentences why HDD4 is treated differently from NVMe and SSD here.
Then say what observation would refute your explanation.

T8-Q5. Is row 22 an instance of any of the three non-correct categories from T8-Q1? If
yes, name the category and say, in one sentence, what real-world harm that category
describes, applied to this specific row (HDD4 is a genuine storage-hardware term, not a
decision or experiment id). If no, say so plainly and say which category you gave it
instead.

T8-Q6. Row 18 shows an unclosed parenthetical remark whose bare id in front of it is a
registered domain word, SHA256. Rows 16 and 17 show the same kind of unclosed
parenthetical remark, but with a genuine id, D22 or E142, in front of it instead. All
three rows end up with different final outputs (row 18 keeps its id, rows 16 and 17 lose
theirs). Using only step two and step four as described in Exhibit 1, and the "t at
start" and "output" values given for rows 16, 17 and 18, explain which step is
responsible for this difference, and why the registered-set check in step four still
matters for row 18 even after step two has already run on it.

T8-Q7. Row 19 cuts a two-character registered domain word, M1, down to just its first
character, "M", before clip does anything else to it. Which step of Exhibit 1 is
responsible for what happens next in this row, step one or step four? Answer using only
what "t at start" and "output" look like for row 19.

This is the end of the task. Answer T4-Q1 through T4-Q8, then T8-Q1 through T8-Q7, in
that order, each on its own numbered line or short paragraph.
