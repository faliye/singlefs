BACKGROUND (all facts below were verified by running commands in the repository today).

Project: singlefs, a from-scratch copy-on-write filesystem in the design phase.
Its knowledge base lives under .claude/kb/ and is written in Chinese.
It is consumed by retrieval: a model pulls out ONE entry at a time, not by reading front to back.

Vocabulary you need:
- A "decision" is D1 through D25. Each has one body file under .claude/kb/kb/decisions/.
- A "sub-item" is a numbered sub-question inside one decision. Numbers are per decision.
- Chinese term 未定项 means "undecided item". Chinese term 已定项 means "settled item".

THE DEFECT THAT WAS FIXED

Before the change, every sub-item of a decision lived in one section whose heading was
literally "undecided items", including the ones that had already been settled. Counts measured:
- 3 decisions had that section fully settled, so the heading was simply false.
- 17 headings read "undecided item N -- settled", a self contradiction.
- 86 in-text references read "undecided item N is settled".
- 34 settled sub-items sat inside undecided-item tables across 15 decisions.

Separately, two decisions (D8 and D9) already used a SECOND independent numbering namespace
also called "settled items", for their main conclusions. In D8, "settled item 2" was the
criterion for node size while "undecided item 2" was the chosen node size value. Two different
things, same number, same file.

THE CHANGE THAT WAS MADE

1. Each decision now has two sections: settled items, and undecided items. There is ONE
   numbering namespace per decision. Numbers are preserved and never re-sequenced, so a
   settled section can hold 2 and 5 while the undecided section holds 1 and 3.
2. The two namespaces in D8 and D9 were merged. The side with more references (86 sub-item
   references) kept its numbers. The side with fewer (18 main-conclusion references) was
   renumbered: D8 main conclusions 1, 2, 3 became 3, 4, 5. In D9 the first main conclusion
   was the same question as sub-item 1, so it was merged into it; the second became item 10.
3. All 353 occurrences of the deictic phrase "this decision" were replaced with an explicit
   decision number plus its short name.
4. A new gate stage compares every reference of the form "D<n>(short name) settled item k"
   or "undecided item k" against the two index tables in that decision body, and fails on a
   mismatch. It scans the knowledge base, the session records, and the research directory
   including Rust source comments. Red and green self-test fixtures exist and both pass.
5. The kb-shape gate stage gained a check that the settled section contains no item marked
   undecided and vice versa. Proven to go red in both directions and green after reverting.
6. Gate stages 60 and 61 previously filtered the undecided-items section with a text filter
   that dropped any line containing the word "settled". That silently skipped genuinely
   undecided items whose text happened to mention another decision being settled. The filter
   was removed because the section now contains only undecided items by construction.

CURRENT STATE: the full gate passes except two stages. One is an unrelated upstream version
bump. The other uses git log -L and therefore cannot see uncommitted working tree edits.

ADDITIONAL FACTS since the background was written:
- A new rule: every sub-item index row must carry a status word in its own text, so status is
  not carried by section location alone. 18 rows were missing it and were repaired.
- A new link checker: every relative markdown link must resolve, and every "see document X
  section N" pointer must name a section that exists. Its first version enumerated files with
  a pattern that skips dot-directories, so it scanned almost nothing and still reported
  success while a broken link existed in the tree.
- The status checker compares only the status word of a reference against the body. It does
  not verify that a number still names the same question it named before.

YOUR TASK, ROUND 3 OF 5. Your stance: make the checks pass while the content is wrong.

Give at most four scenarios in which every check above reports success and the content is
still wrong. For each, write three short lines and nothing else:
  What someone writes:
  Which check should have caught it:
  Why it does not:

Then give one line ranking them by how likely each is during ordinary editing.

Constraints on your answer: be terse, no preamble, no summary paragraph, no repetition of the
background, no markdown emphasis, plain English sentences only.
