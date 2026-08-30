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

YOUR TASK, ROUND 2 OF 5. Your assigned stance is: attack the shape itself, not the execution.

The claim under attack is: "splitting sub-items into a settled section and an undecided
section, while keeping one shared numbering namespace that is never re-sequenced, is a better
shape for a retrieval-read knowledge base than what it replaced."

Assume the execution was flawless. Attack the design. Consider at least these angles:
- A reader retrieves exactly one entry, with no surrounding context. What can they now
  misread that they could not misread before?
- Numbers within one section are non-contiguous. What does a rendering pipeline or a diffing
  tool do with that, and does it matter here?
- When an item moves from undecided to settled, it changes sections. What breaks at that
  moment, and what has to be updated in lockstep?
- The status now lives in the section heading, that is, in the item's location rather than in
  its text. Is location a safe carrier for a fact in a system read by retrieval?
- Is there a third shape that dominates both the old one and the new one?

State plainly which angle produces the strongest objection, and say what observation in the
repository would confirm or refute it. Do not restate the background.

Answer in English. Do not use any markdown emphasis such as asterisks.
