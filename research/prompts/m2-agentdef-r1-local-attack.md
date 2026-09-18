TASK

You are asked to mechanically apply four written detection criteria (labeled C1, C2, C3, C4 below) to ten short text samples (labeled P1 through P10). The criteria come from an automated checker that scans short instructional documents ("agent definitions") and flags any line or paragraph that records history, process narrative, dates, or explanatory prose instead of stating only what to do (steps, constraints, judging criteria, deliverables). Your job has two parts.

PART A: For every sample (P1 through P10) and every criterion (C1, C2, C3, C4), decide whether that specific criterion, applied strictly and literally to the given features of that sample, would flag the sample (call this RED) or would not flag it (call this GREEN). Ground every verdict in the specific fact(s) given below that make the criterion match or fail to match. Do not answer only RED or GREEN; name the fact. For every verdict also state one concrete observation that would overturn it (a fact that, if it were true instead of what is given, would flip your verdict).

PART B: For each of the four criteria separately, construct one hypothetical sample (a combination of features of the same kinds used in the table below: whether it is a heading line, whether it is a table row, its paragraph position, its opening text, whether it carries an out-of-quote date, whether the same line points to a records file or a kb file, and what immediately follows each relevant punctuation mark) for which applying that criterion's literal wording produces one verdict, while the checker's stated purpose (an instructional definition should contain only steps, constraints, judging criteria, and deliverables, never process narrative, dated records, or explanatory prose) calls for the opposite verdict. State the literal verdict, the intended verdict, why the two diverge, and one observation that would overturn your example.

FORMAT RULES

Do not use any markdown emphasis anywhere in your answer: no bold, no italics, no underline, no bold headings. Plain numbered lines and plain tables are fine. Number every answer, 1 through 44 (40 for part A, 4 for part B). In your answers, refer to samples only by their label (P1 through P10) and to criteria only by their label (C1 through C4). Do not cite any file path or line number in your answer, including a line number you might work out yourself while reading; if you need to point at something, point at it by sample label and criterion label only. Answer in English.

WHY THIS PROMPT USES TAGS INSTEAD OF THE ORIGINAL WORDS

The checker's rules are written against specific Chinese-language words and punctuation marks. Because this whole prompt is written in English, every one of those Chinese words or marks is replaced below by an all-capital tag. A tag always stands for exactly one specific original word or character; two different tags never stand for the same original word or character, and the same tag always stands for the same original wherever it appears. You do not need to know what the underlying Chinese glyph looks like; you only need to track, for each sample, which tag (if any) appears at which position, and apply the criteria below using the tags exactly as if they were the original words. Ordinary text that is not one of the checker's trigger words is given only as a short English paraphrase in square brackets, since its exact wording never matters to the criteria, only whether or not it equals one of the tagged trigger words.

TAG LEGEND (closed, complete list)

Eleven label words that can open an explanatory paragraph under criterion C3:
WEISHENME, YIJU, LIYOU, SHICE, JINGGUO, YUANYIN, LAILI, BEIJING, LISHI, YANGE, QIANQING
Each of these eleven tags is a distinct one-to-three-character Chinese word. Treat each as an exact-match token unrelated to the others unless this prompt says otherwise.

Four more label words used only inside criterion C4's first group, always alongside SHICE:
SHIPAO, CAIGUO, ZHUANGSHANG, ZHUANGGUO
These are four further distinct Chinese words, grouped with SHICE only because criterion C4 groups them, not because they share any other relationship.

Two conjunctions:
YINWEI, ZHISUOYI
Two distinct Chinese words, each meaning roughly "because" or "the reason why," used only as sentence-opening conjunctions.

One more word, used only in criterion C1's second option, distinct from LISHI:
BIANGENGSHI

The copula word:
SHI_COPULA
The single Chinese word meaning "is" or "to be."

Punctuation classes. Each class below covers exactly the character forms named for it; no other character belongs to that class, and no class overlaps another:
COLON covers a colon character, either the East Asian full width form or the plain ASCII form.
LPAREN covers an open parenthesis character, either the East Asian full width form or the plain ASCII form.
COMMA covers a comma character, either the East Asian full width form or the plain ASCII form.
PERIOD_FW covers only the East Asian full width period character; there is no ASCII period in this class.
DUN covers the Chinese enumeration-comma character (used to separate items in a list); it has no ASCII form.
SEMI covers a semicolon character, either the East Asian full width form or the plain ASCII form.
EMDASH2 covers a pair of East Asian em dash characters written together as one punctuation unit.

Two quote-mark tags, for the Chinese corner brackets used in the source text to quote a word or phrase:
CJKQ_OPEN is the opening corner bracket.
CJKQ_CLOSE is the closing corner bracket.

DATE means a date written as four digits, a hyphen, two digits, a hyphen, two digits (for example 2026-09-18). This pattern uses only ASCII digits and hyphens, so it needs no tag; treat the literal word DATE below as standing for one such date string.

WS means one or more whitespace characters (a space).

THE FOUR CRITERIA

C1, record-section heading. Applies only to a line that is itself a heading (title) line. A heading line is flagged if its own text contains the tag LISHI anywhere in it, or contains the tag BIANGENGSHI anywhere in it. C1 never looks at anything except heading lines; a line that is not a heading line cannot be flagged by C1, regardless of its content.

C2, a dated line with no pointer. A line is flagged if both of the following hold: (a) a DATE appears somewhere on that line outside of any CJKQ_OPEN ... CJKQ_CLOSE quoted span; and (b) that same line does not also contain a path pointing to a specific file under a directory named records or under a directory named .claude/kb. Not recognized, meaning this does NOT cause a flag: a DATE that appears only inside a CJKQ_OPEN ... CJKQ_CLOSE quoted span (for example, a line that quotes the title of a rule subsection, and that title happens to carry a date as part of its own name, is treated as a citation, not as a record, so that DATE does not count as "outside quotes"). Also not counted as "pointing to a file": naming only a directory (for instance, the directory .claude/kb immediately followed by a punctuation mark or by whitespace, with no specific file named after it) or naming some other, unrelated file's path elsewhere on the line (a path that is not under records or under .claude/kb).

C3, explanatory paragraph. Applies only to the first line of a paragraph (see the paragraph-segmentation rule below for what counts as a paragraph and what counts as its first line). Take that first line and strip from its very start: any leading list marker, any leading ">" character, any leading pair of asterisks used as bold markup, and any leading warning-sign glyph. Whatever token sits at the very start of what remains, the paragraph is flagged if that start is one of three kinds:
kind one: a DATE (a paragraph opening with a date is understood to be recording an event);
kind two: one of the eleven label words WEISHENME, YIJU, LIYOU, SHICE, JINGGUO, YUANYIN, LAILI, BEIJING, LISHI, YANGE, QIANQING, with the very next character immediately after that label word (no gap allowed) being one of: COLON, LPAREN, COMMA, PERIOD_FW, DUN, WS, or SHI_COPULA;
kind three: one of the two conjunctions YINWEI or ZHISUOYI.
Not recognized, meaning this does NOT cause a flag under C3: one of the eleven label words immediately followed by some character that is not in the connector list just given (for example, a label word directly attached to further ordinary text with no connector in between is understood to be part of a noun phrase or an input-item label, not the start of an explanation).

C4, explanatory half-sentence. Applies inside any line, not only at a paragraph's first line. Within a line, look at every point immediately after one of these five punctuation marks: LPAREN, COMMA, SEMI, PERIOD_FW, or EMDASH2. Between that punctuation mark and whatever is checked next, only WS and, optionally, one DATE may sit in between; nothing else may sit in that gap. The line is flagged at that point if, allowing only that WS-and-one-DATE gap, what comes next is:
group one: one of SHICE, SHIPAO, CAIGUO, ZHUANGSHANG, ZHUANGGUO, YINWEI, or ZHISUOYI, appearing with no further requirement on what follows it;
group two: one of WEISHENME, YIJU, LIYOU, YUANYIN, or JINGGUO, where in addition the character immediately after that label word (no gap allowed here) must be COLON or WS.
In addition to the punctuation-triggered check above: a line that is in the middle of its paragraph (that is, not the paragraph's first line) is also flagged under C4 if that middle line itself begins, after the same stripping described in C3, in the manner of C3's kind one, two, or three (for instance, a label word followed by COLON, appearing as the line right after some other line of the same paragraph, so that the two lines rendered together read as one paragraph whose second half opens with that label word).
Not recognized, meaning this does NOT cause a flag under C4: immediately after a DUN mark (for example, a list such as "write X, Y, and then YIJU" where YIJU is simply the last item of an enumeration is not flagged this way); one of the label words immediately followed by some character that is not a valid connector, with no gap (same noun-phrase exception as in C3); and a DATE that is separated from one of these words by some other, non-whitespace character in between (that configuration is governed by C2, not by C4, because the gap contains more than WS and one DATE).

EXEMPTIONS AND SEGMENTATION

Neither C3 nor C4 is ever applied to a line inside a fenced code block (a block delimited by a line of three backticks or three tildes opening it and another such line closing it), and neither C3 nor C4 is ever applied to a line that is a table row (a line starting with the pipe character). Table rows are exempt from C3 and C4 only; C2 still applies normally to a table row's own text. C1 and C2 apply inside fenced blocks the same way they apply anywhere else; only C3 and C4 are switched off for fenced-block lines.

Paragraph segmentation: each list item (a line that starts with a dash-and-space marker, a number-and-period marker such as "1. ", or a number-letter-and-period marker such as "3b. ") is its own separate paragraph. That paragraph runs from the list-item line until, but not including, whichever comes first: the next list-item line, a blank line, a heading line, a table, or a fenced block. A run of consecutive non-blank lines that is not part of a list is treated as one single paragraph, and its first line is its first physical line in that run.

KNOWN BLIND SPOT (background only, not a rule to apply): an explanatory sentence that uses none of the tagged words above and carries no DATE is not recognized by any of C1, C2, C3, or C4. This is given only as background. None of the ten samples below are being asked about specifically for this blind spot, though you may use the idea if it helps you build one of your part B examples.
THE TEN SAMPLES

Each sample below is one physical line taken from a real file (the file name is given only as context; do not cite it back, and do not cite any line number). For each sample: whether it is a heading line, whether it is a table row, its position within its paragraph, the token sequence at the very start of the line (after the C3 stripping), whether a DATE sits outside CJKQ quotes anywhere on the line, whether the line also contains a path to a specific file under records or under .claude/kb, and the full ordered list of every point on the line that sits immediately after one of LPAREN, COMMA, SEMI, PERIOD_FW, DUN, or EMDASH2, together with the token sequence that follows that point. Where a gap of ordinary text or a DATE sits between the punctuation and a tag, that gap is written out explicitly; do not assume adjacency that is not shown.

SAMPLE P1
Context only, do not cite: a shared-constraints file.
Heading line: no. Table row: no. Paragraph position: this line is the second (middle) line of a two-line paragraph; the C3 check does not apply to this line at all.
Line-start token sequence: [ordinary text, meaning roughly "this document and each definition"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: yes, inside a path fragment naming a records file (the path names a specific file, not just a directory).
Same line points to a records or .claude/kb file: yes.
Points after a trigger punctuation mark, in left-to-right order:
1. after PERIOD_FW: CJKQ_OPEN, WEISHENME, CJKQ_CLOSE, CJKQ_OPEN, SHICE, CJKQ_CLOSE, CJKQ_OPEN, JINGGUO, CJKQ_CLOSE, then [ordinary text].
2. after COMMA: [ordinary text, meaning roughly "is not written here"].
3. after SEMI: [ordinary text, meaning roughly "when that content is needed"].
4. after LPAREN: CJKQ_OPEN, then [ordinary text, meaning roughly "read this first before starting work"], then COLON, then more text, and the quoted span this citation shows is cut off before any CJKQ_CLOSE appears in what is shown here.
5. after PERIOD_FW: [ordinary text, meaning roughly "gate number 71"].
6. after LPAREN: [ordinary text, meaning "rule"], then COLON.

SAMPLE P2
Context only, do not cite: a main coordinating entry-point file.
Heading line: no. Table row: no. Paragraph position: first (head) line of its paragraph.
Line-start token sequence (after stripping a leading bold marker): [ordinary text, meaning roughly "all tasks enter from here"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: yes, inside a path fragment naming a records file.
Same line points to a records or .claude/kb file: yes.
Points after a trigger punctuation mark:
1. after SEMI: WEISHENME, then immediately (no gap at all) [ordinary text, meaning roughly "is decided this way"].
2. after COMMA: [ordinary text, meaning roughly "the shared context"].
3. after LPAREN: [ordinary text, meaning roughly "what the project is"], which contains SHI_COPULA later in it but SHI_COPULA is not the character immediately after this LPAREN.

SAMPLE P3
Context only, do not cite: the same main coordinating entry-point file, a later line.
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item (each list item is its own paragraph).
Line-start token sequence (after stripping the list marker and a leading bold marker): [ordinary text, meaning roughly "consolidation should happen on a regular schedule"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: no.
Same line points to a records or .claude/kb file: no.
Points after a trigger punctuation mark:
1. after COMMA: [ordinary text, meaning roughly "judge each item"].
2. after COMMA: [ordinary text, meaning roughly "not even one may silently vanish"].
3. after PERIOD_FW: [ordinary text, meaning roughly "divergence is exploration"], which contains SHI_COPULA but SHI_COPULA is not the first token after this PERIOD_FW and no label word appears at this point at all.
4. after COMMA: [ordinary text, meaning roughly "opening a loophole"].
5. after COMMA: [ordinary text, meaning roughly "once opened"].

SAMPLE P4
Context only, do not cite: a three-way-debate role file.
Heading line: no. Table row: no. Paragraph position: this line is the second (middle) line of its paragraph; the C3 check does not apply to this line at all.
Line-start token sequence: [ordinary text, meaning roughly "read this first before starting work"], then COLON.
DATE outside CJKQ quotes on this line: no; the only DATE on this line sits inside a CJKQ_OPEN ... CJKQ_CLOSE span (the span quotes the title of a rule subsection, and that title itself carries the DATE together with more text).
Same line points to a records or .claude/kb file: no.
Points after a trigger punctuation mark:
1. after LPAREN: DATE, then [ordinary text, meaning roughly "the user's explicit order"]; this whole point sits inside the CJKQ_OPEN ... CJKQ_CLOSE span described above.

SAMPLE P5
Context only, do not cite: an implementation-writer role file.
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item.
Line-start token sequence (after stripping the list marker): [ordinary text, meaning roughly "state in the report"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: no.
Same line points to a records or .claude/kb file: no.
Points after a trigger punctuation mark:
1. after LPAREN: [ordinary text, meaning roughly "which construction guarantees it"].
2. after COMMA: [ordinary text, meaning roughly "only then may it be written"].
Note: this line also contains, earlier, a CJKQ_OPEN, WEISHENME, [ordinary text meaning "cannot be reached"], CJKQ_CLOSE span, but that span sits right after the ordinary word "state," not right after any of LPAREN, COMMA, SEMI, PERIOD_FW, DUN, or EMDASH2, so it is not listed as a point above.
SAMPLE P6
Context only, do not cite: a three-way forward-argument role file.
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item.
Line-start token sequence (after stripping the list marker): [ordinary text, meaning roughly "facts that can be checked by a command"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: no.
Same line points to a records or .claude/kb file: no.
Points after a trigger punctuation mark:
1. after SEMI: [ordinary text, meaning roughly "only the main agent may"]; later in this same clause the word SHICE appears (as in, "the main agent has done SHICE"), but SHICE is not the token immediately after this SEMI, so it is not counted as this point's match.
2. after COMMA: [ordinary text, meaning "write"], CJKQ_OPEN, then [ordinary text, meaning roughly "cannot be reviewed"], and the quoted span this citation shows is cut off before any CJKQ_CLOSE appears in what is shown here.
Note: later on this same line, after the word "and," the label word YUANYIN appears, but it is not immediately preceded by any of LPAREN, COMMA, SEMI, PERIOD_FW, DUN, or EMDASH2, so it is not listed as a point above.

SAMPLE P7
Context only, do not cite: a bookkeeping-scribe role file.
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item.
Line-start token sequence (after stripping the list marker): [ordinary text, meaning roughly "an itemized change specification"], then COLON.
DATE outside CJKQ quotes on this line: no.
Same line points to a records or .claude/kb file: no.
Points after a trigger punctuation mark, in order (this line is an enumeration: file, old string, new string, basis):
1. after LPAREN (the one following the second item of the enumeration): [ordinary text, meaning roughly "the original text, the whole line"].
2. after DUN (the one following the third item of the enumeration): YIJU, with nothing at all in between (no gap of any kind).
3. after LPAREN (the one immediately following that same YIJU, with nothing in between): [ordinary text, meaning roughly "the verdict, or the source of the user's final decision"].

SAMPLE P8
Context only, do not cite: a pre-cleanup snapshot of a local-defense role file (this file is a historical snapshot kept for comparison, not one of today's live definitions).
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item.
Line-start token sequence (after stripping the list marker): SHICE, then immediately (no gap) LPAREN.
DATE outside CJKQ quotes on this line: yes; the date sits directly inside that same LPAREN ... RPAREN span (a plain parenthetical aside), not inside any CJKQ_OPEN ... CJKQ_CLOSE span.
Same line points to a records or .claude/kb file: no.
Points after a trigger punctuation mark:
1. after LPAREN (the one right after the opening SHICE): DATE, then [ordinary text, meaning roughly "first round"].
2. after SEMI: [ordinary text, meaning roughly "samples more easily fail to reach the needed count"].
3. after COMMA: [ordinary text, meaning roughly "report honestly if there are not enough"].

SAMPLE P9
Context only, do not cite: a pre-cleanup snapshot of the bookkeeping-scribe role file (a historical snapshot, not one of today's live definitions).
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item.
Line-start token sequence (after stripping the list marker): [ordinary text, meaning roughly "run the stage-ownership table"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: yes, two separate DATE occurrences on this one line.
Same line points to a records or .claude/kb file: no; the only paths on this line point to an agent-definitions file and to a lint-script file, neither of which is under records or under .claude/kb.
Points after a trigger punctuation mark, in left-to-right order (eighteen points on this one long line):
1. after LPAREN: [ordinary text, meaning roughly "the shared constraints file"].
2. after COMMA: [ordinary text, meaning roughly "not counted by hand"].
3. after LPAREN: DATE, then [ordinary text of two characters, meaning "twice"], then SHIPAO. Note the two-character ordinary-text gap sits between the DATE and SHIPAO; that gap is not WS and not a second DATE.
4. after PERIOD_FW: [ordinary text, meaning roughly "the red one, judge whether"].
5. after LPAREN: [ordinary text, meaning roughly "look at the ones that are named"].
6. after SEMI: [ordinary text, meaning roughly "not something from this round"].
7. after COMMA: [ordinary text, meaning roughly "follow it and write"].
8. after PERIOD_FW: [ordinary text, meaning roughly "gate number thirty reports"].
9. after COMMA: [ordinary text, meaning roughly "copy verbatim"].
10. after COMMA: [ordinary text, meaning roughly "not judged on this basis"].
11. after PERIOD_FW: [ordinary text, meaning roughly "run it once more"].
12. after COMMA: [ordinary text, meaning roughly "registered to you"].
13. after COMMA: [ordinary text, meaning roughly "but written as specified"].
14. after LPAREN: [ordinary text, meaning roughly "bare numbering"].
15. after SEMI: [ordinary text, meaning roughly "red within this round"].
16. after COMMA: [ordinary text, meaning roughly "stop and hand it to the main agent"].
17. after COMMA: [ordinary text, meaning roughly "do not fix it yourself"].
18. after LPAREN: DATE, then WS, then SHICE, then COLON.

SAMPLE P10
Context only, do not cite: a pre-cleanup snapshot of the implementation-writer role file (a historical snapshot, not one of today's live definitions).
Heading line: no. Table row: no. Paragraph position: first (head) line of its own paragraph, because it is a list item.
Line-start token sequence (after stripping the list marker): [ordinary text, meaning roughly "every new test that proves red"], not a DATE, not a label word, not YINWEI or ZHISUOYI.
DATE outside CJKQ quotes on this line: yes, one occurrence.
Same line points to a records or .claude/kb file: no; the only path on this line is a source-tree config file path, not under records or under .claude/kb.
Points after a trigger punctuation mark, in left-to-right order (eleven points on this one long line):
1. after LPAREN: [ordinary text, a literal shell-command example].
2. after COMMA: [ordinary text, meaning roughly "run that test"].
3. after COMMA: [ordinary text, meaning "record,"], CJKQ_OPEN, then [ordinary text, meaning roughly "broke it"], and the quoted span this citation shows is cut off before any CJKQ_CLOSE appears in what is shown here.
4. after LPAREN: [ordinary text, meaning roughly "do not"].
5. after COMMA: DATE, then WS, then SHICE, then [ordinary text, meaning roughly "the source code..."].
6. after COMMA: [ordinary text, meaning roughly "first run one unmodified copy"].
7. after LPAREN: [ordinary text, meaning roughly "mostly someone else's in-progress change"].
8. after COMMA: [ordinary text, meaning roughly "the mutation proof"].
9. after COMMA: [ordinary text, meaning "under debug"].
10. after COMMA: [ordinary text, meaning roughly "record it truthfully"].
11. after PERIOD_FW: [ordinary text, a literal file-path token].

FINAL REMINDER

Answer with exactly 44 numbered items: items 1 through 40 are part A (ten samples times four criteria each; you choose the order, but state which sample and which criterion each item is about), and items 41 through 44 are part B, one per criterion C1, C2, C3, C4. Every item must name the specific fact that decided it and must state one concrete observation that would overturn it. No markdown emphasis. No file paths or line numbers in your answer, only sample labels P1-P10 and criterion labels C1-C4. Answer in English.
