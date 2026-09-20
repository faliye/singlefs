You are reviewing a diff to two AI agent definition files in a software project. You are not being asked whether the project design is good. You are being asked three narrow, mechanical questions about the TEXT of the diff. Answer every numbered item. Fill every table cell. Do not answer any item with only yes or no; always also fill the reason cell. Do not write source-code line numbers or file line numbers anywhere in your answer; refer to rows by their Row ID, or to steps by their step name (for example "step 7b"), never by a line number.

BACKGROUND

Two agent definition files, experiment-runner.md and kb-scribe.md, were both edited in the same change. Both files open with the line: "Read `.claude/agent-common.md` before starting; this definition sets `omitClaudeMd`, so it does not inherit the project CLAUDE.md or the rules it `@`-includes."

RULE 1 (the standard for question set A). This is the literal text of `.claude/agent-common.md`, the shared constraints file both definitions point to:

"This file and each agent definition write only HOW: steps, constraints, judgment criteria, deliverables. WHY, lived experience, and dated records go into a separate records file or the knowledge base; they are not written here, and no explanatory tail is left behind. When that content is still needed here, it is written as a read-pointer instruction, in the form: 'Read before starting: `file` section \"...\"'. A project gate (numbered 71) checks specifically for this rule."

Under RULE 1, classify each sentence below as one of:
HOW = an executable step, a constraint on an action, a pass/fail judgment criterion, or a deliverable to produce.
WHY = an explanation of why something is required, motivation, or reasoning.
WHAT = a statement of a fact, a definition of a concept, or a description of how something in the world already works (independent of what this particular agent should do about it) — the kind of statement that would still be true even if a completely different agent, or a human, carried out the task.

RULE 2 (the standard for question set B, a second, independent test). This is the literal text of a knowledge-base discipline rule, item 4, called "Contradiction is worse than a blank":

"The same fact is allowed exactly one authoritative record; every other place must link to it instead of restating it. A human reading start to finish can spot a contradiction. Search cannot: it will surface only one of the two conflicting records, and it will not tell you which one it picked."

Under RULE 2, two passages below (Table C and Table D) each pair a NEW sentence added to an agent definition with an EXISTING sentence from a project rule file that appears to describe the same fact. For each row, judge: do the two sentences state the exact same fact in the same words (aside from harmless paraphrase), or does one of them add, drop, or change a condition, an exception, a trigger, or a remedy that the other does not have? If they differ, that is evidence of a second, drifting authoritative record rather than one record with a pointer to it.

QUESTION SET A — TABLE 1: per-sentence classification under RULE 1

Each row is one sentence newly added by this diff to one of the two agent definition files. Fill three cells: Verdict (write exactly one of HOW, WHY, WHAT), Reason (one or two sentences explaining the verdict; if you judge the sentence to be WHAT, name the fact it states and say whether that fact would still need to be written down somewhere even if this sentence were deleted), and Falsifier (one sentence: what would you need to see in the sentence's wording for you to change your verdict).

Table 1 header: | Row ID | Sentence | Verdict | Reason | Falsifier |

Row A1. Sentence (from experiment-runner.md, new step 7b, first clause): "The experiment page must have a section named `### Decisions Affected`, placed before the `## History Version` section, containing one table with columns `| Decision item | Relation | Review |`."

Row A2. Sentence (from experiment-runner.md, new step 7b): "Every decision mentioned in the page's body text (outside the History Version section and this section itself) gets one row; the relation must be one of: Supports / Overturns / Material / No-effect."

Row A3. Sentence (from experiment-runner.md, new step 7b): "Supports and Overturns rows must be written down to the specific item level (`D<n>(short name) Established item k`); before writing, check whether that item's Basis section cites this experiment back — if it does not cite back, rewrite the relation as Material or No-effect and list that cell in the report."

Row A4. Sentence (from experiment-runner.md, new step 7b): "The table must have at least one row whose relation is Supports, Overturns, or Material."

Row A5. Sentence (from experiment-runner.md, new step 7b): "The Review cell is written as `<today> Unaffected: reason` or `<today> Changed`; when the reason refers to this page's own experiment, write `E<number>(short name)`, not the phrase 'this experiment'."

Row A6. Sentence (from experiment-runner.md, new step 7b): "After rerunning an existing experiment, or after this page's artifact changes, or after a history entry is added, every row in the table must be reviewed again, with a date no earlier than this change."

Row A7. Sentence (from experiment-runner.md, new step 7b): "When that decision has not yet been converted to the slimmed-down format (it is still listed in `.claude/decision-links-pending`), the relation is still judged the same way as above, and the item number is still written the same way."

Row A8. Sentence (from experiment-runner.md, the "read before starting" line, new clause appended to it): "When you are going to write an experiment page, additionally read the paragraph 'The experiment page's Decisions Affected' inside the rule section 'Decision body only states the current state, basis written as pointers; decisions and experiments are cross-registered' of the format-evolution rule file."

Row A9. Sentence (from kb-scribe.md, new input bullet, first clause): "For writing decision body text (establishing a new item, or changing an item's Ruling / Scope / Basis): the spec must be given in four blocks — Ruling (the current rule, no date), Scope (what it covers, what it does not, known edge cases), Basis (one line per entry: what `E<number>(short name)` proves, without copying numbers already on the experiment page; a three-way verdict is given as a file path; a user's ruling is written as 'the exact words are in the change history'), and Owed (`C<number>(short name)`, or 'none' if there is none)."

Row A10. Sentence (from kb-scribe.md, new input bullet): "The Basis block must cite at least one experiment; an item with nothing quantifiable to measure instead writes `No experiment: reason`, and that sentence must not contain any experiment number — gate 75 treats any `E<number>(` appearing anywhere in the Basis block as a citation."

Row A11. Sentence (from kb-scribe.md, new input bullet): "The index-table row is one sentence, no date, no more than 100 characters, ending with 'Status: Established.'; an open (undecided) item's row additionally carries 'Changes bytes of the first transaction: No (yyyy-mm-dd, basis: ...)' — gate 31 requires this to be written in the table row itself, because it cannot be located if written in the paragraph below the table."

Row A12. Sentence (from kb-scribe.md, new input bullet): "If the spec is missing any of these blocks, stop and report it; do not fill it in yourself."

Row A13. Sentence (from kb-scribe.md, new step 3b, first sentence): "When the spec writes or changes an item's Basis section: for every experiment that section cites, go to that experiment's page and check whether its `### Decisions Affected` table has a row pointing back to this item with relation Supports or Overturns."

Row A14. Sentence (from kb-scribe.md, new step 3b): "If the row is missing, the item number does not match, or the relation is Material / No-effect, list each case in the report to the main agent; do not touch the experiment page's table yourself — judging the relation is not part of your job."

Row A15. Sentence (from kb-scribe.md, new step 3b): "When an item's status is flipped or its ruling is changed, likewise list in the report that 'the experiment pages cited by this item's Basis need to be reviewed again.'"

Row A16. Sentence (from kb-scribe.md, the "read before starting" line, changed from one section to two): "Read before starting: ... the format-evolution rule file's sections 'Hard constraints' and 'Decision body only states the current state, basis written as pointers; decisions and experiments are cross-registered' (now two sections instead of one); ..."

QUESTION SET B — TABLE C: new step 7b (experiment-runner.md) versus the rule file's own "Decisions Affected" paragraph

The rule file (format-evolution.md) already has a paragraph, titled "The experiment page's Decisions Affected", that describes the same table that step 7b describes. For each row, fill: Same fact? (write exactly one of SAME, DIFFERENT), and Discrepancy (if DIFFERENT: name exactly what condition, exception, trigger, or remedy appears in one sentence but not the other; if SAME: say so in one sentence), and Verdict (write exactly one of: SECOND-RECORD-DRIFTING if the two sentences restate the same rule with different content, NEW-RULE-NO-COUNTERPART if the new sentence has no corresponding sentence in the rule file at all, or NO-ISSUE if they match).

Table C header: | Row ID | New text (agent definition) | Existing text (rule file) | Same fact? | Discrepancy | Verdict |

Row C1.
New text: "The experiment page must have a section named `### Decisions Affected`, placed before the `## History Version` section, containing one table with columns `| Decision item | Relation | Review |`."
Existing text: "Every experiment page has a section `### Decisions Affected`, containing one table `| Decision item | Relation | Review |`:"

Row C2.
New text: "Every decision mentioned in the page's body text (outside the History Version section and this section itself) gets one row; the relation must be one of: Supports / Overturns / Material / No-effect."
Existing text: "The decision-item cell is written as `D<n>(short name) Established item k` or `Open item k`; a decision with no item, or one only mentioned as background, is written as `D<n>(short name)`; every decision mentioned in the experiment's body text (outside the History Version section and this section itself) must have a row."

Row C3.
New text: "Supports and Overturns rows must be written down to the specific item level (`D<n>(short name) Established item k`); before writing, check whether that item's Basis section cites this experiment back — if it does not cite back, rewrite the relation as Material or No-effect and list that cell in the report."
Existing text: "The relation is one of Supports / Overturns / Material / No-effect. Supports and Overturns must be written down to the item level, and that item's Basis must cite the experiment back; conversely, every experiment cited in a decision item's Basis must have this row in that experiment's page, with relation Supports or Overturns."

Row C4.
New text: "The table must have at least one row whose relation is Supports, Overturns, or Material."
Existing text: "An experiment must correspond to a decision: the table must have at least one row whose relation is Supports, Overturns, or Material; an experiment whose conclusion is retired or voided states so in its title's status, and this rule does not apply to it."

Row C5.
New text: "The Review cell is written as `<today> Unaffected: reason` or `<today> Changed`; when the reason refers to this page's own experiment, write `E<number>(short name)`, not the phrase 'this experiment'."
Existing text: "The Review cell is written as `yyyy-mm-dd Changed` or `yyyy-mm-dd Unaffected: reason`."

Row C6.
New text: "After rerunning an existing experiment, or after this page's artifact changes, or after a history entry is added, every row in the table must be reviewed again, with a date no earlier than this change."
Existing text: "After the experiment reaches a new conclusion (a new dated entry in the page's own History section or in the experiments-history file, a change to the body text, or a changed artifact), every row must be reviewed again, with a date no earlier than that change; a row written as 'Changed' requires that decision's file to be part of the same change."

Row C7.
New text: "When that decision has not yet been converted to the slimmed-down format (it is still listed in `.claude/decision-links-pending`), the relation is still judged the same way as above, and the item number is still written the same way."
Existing text: (no corresponding sentence exists anywhere in the rule file's "Decisions Affected" paragraph)

QUESTION SET B — TABLE D: new kb-scribe.md input bullet versus the rule file's own four-block definition

The rule file (format-evolution.md) already defines the four required blocks of a decision item (Ruling, Scope, Basis, Owed) and the index-table row format. The new kb-scribe.md input bullet restates the same four blocks and the same index-table row format for the person filling out the spec. Fill the same four columns as Table C.

Table D header: | Row ID | New text (agent definition) | Existing text (rule file) | Same fact? | Discrepancy | Verdict |

Row D1.
New text: "Scope (what it covers, what it does not, known edge cases)"
Existing text: "Scope: what it covers, what it does not, known edge cases, each in one or two sentences; argumentation, derivation, or notes like 'that experiment's measurement is imprecise' do not belong in Scope — they go in the experiment page's 'what it cannot answer' section or in the three-way verdict."

Row D2.
New text: "The Basis block must cite at least one experiment; an item with nothing quantifiable to measure instead writes `No experiment: reason`, and that sentence must not contain any experiment number — gate 75 treats any `E<number>(` appearing anywhere in the Basis block as a citation."
Existing text (rule file): "Only with an experiment is there a decision: the Basis must cite at least one experiment; a purely policy matter with nothing quantifiable writes 'No experiment: reason', and the gate reports the count of such items."
Existing text (gate 75's own header comment, a second existing source): "An established item's Basis section cites at least one experiment, or writes 'No experiment: reason' (the reason must be at least eight characters); the success line reports how many established items write 'No experiment'."

Row D3.
New text: "The index-table row is one sentence, no date, no more than 100 characters, ending with 'Status: Established.'; an open (undecided) item's row additionally carries 'Changes bytes of the first transaction: No (yyyy-mm-dd, basis: ...)' (gate 31 requires this to be written in the row itself, because it cannot be located if written in the paragraph below the table)."
Existing text: "The Established-items / Open-items index table: each row is one sentence stating the ruling, not copied from the item's body text, no date, no more than 100 characters; the row still ends with 'Status: Established.' as before (this standard marker does not count toward the character limit). An open item's row must additionally carry 'Changes bytes of the first transaction: No (yyyy-mm-dd, basis: ...)' (gate 31 requires it written in the row itself, because it cannot be located if written in the paragraph below the table); this too is a standard marker that does not count toward the limit, and that date does not count as 'having a date' either."

Row D4. This row has no existing-text counterpart to compare against; instead answer a direct question. Gate 75's own header comment (quoted in Row D2 above) requires the "No experiment: reason" sentence to have a reason of at least eight characters. Does the new kb-scribe.md input bullet (quoted across Rows A9–A12 above) tell the person filling out the spec about this eight-character minimum anywhere? Answer in the Discrepancy column: YES it is stated, or NO it is not stated anywhere in the new bullet. Leave Same fact? and Verdict as NOT-APPLICABLE for this row.

ANSWER FORMAT

Write your answer in plain English prose and tables, no markdown bold or italic markers. For each of Table 1, Table C, and Table D, reproduce every row and fill every cell described above; do not skip a row and do not merge rows. Do not write any source-code line number or file line number anywhere in your answer. If you need to point at a specific piece of text, refer to it by its Row ID (A1-A16, C1-C7, D1-D4) or by its step name (for example "step 7b", "step 3b", "the input bullet"). Do not answer any cell with only "yes" or "no" by itself; a bare yes/no answer without the accompanying reason or discrepancy sentence is not acceptable. If you are genuinely unsure about a verdict, say so explicitly and explain what additional information would resolve it, rather than guessing silently.
