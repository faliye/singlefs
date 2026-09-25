SINGLEFS DESIGN QUESTION - LOCAL ATTACK LEG - ROUND c363b-r1 - SLOT V3

You are one of several independent reviewers looking at a design gap in singlefs, a copy-on-write filesystem being designed from scratch. You are given a self-contained set of facts below. Do not assume anything not stated here, and do not use outside knowledge of any other filesystem's tree structure to fill a gap that these facts leave open. Read carefully: every qualifier in this text is load-bearing. Do not use any markdown emphasis anywhere in your answer (no bold, no italics, no asterisk bullets, no backtick code spans, no headings); plain tables using the vertical bar character are fine if you want them, but are not required. Number your answers to match the labels given at the end. For every numbered answer, add one sentence starting exactly with "This would be refuted by:" describing the specific observation that would prove that answer wrong. Answer entirely in English. Do not cite, invent, or guess at any source-code line number or design-document line number anywhere in your answer, including any you might work out for yourself while reasoning; refer only to the fact labels given below (for example FACT B4, FACT G, ROW Q1.3), or to plain descriptive names for functions and constants (for example "the height-reading function" or "the accounting tree's internal entry width" are fine; a line number is not).

BACKGROUND

singlefs stores its metadata in fixed-size 16384-byte blocks called code-two nodes. Each code-two node has a self-describing header, followed by a whole number of fixed-width entries. Two of this project's trees, called the accounting tree and the central mapping tree, are built as ordinary multi-level trees out of these nodes: a level-0 node is a leaf and holds data entries directly; a node above level 0 is an internal node and holds entries that are each a separator key plus a pointer to one child node. Every code-two node's header carries a level number. This project defines a tree's current height as the level number found in its root node's header, plus 1 (so a tree that is just one leaf, at level 0, has height 1). A publish is one commit of a new, consistent, on-disk version of the filesystem (also called a checkpoint). This project also defines a quantity called ckpt_cost, computed fresh at every publish, that some other decisions use when deciding how much space to hold in reserve. Your only task in this material is arithmetic: given the facts below about node capacities and about how these trees grow and shrink, compute tree heights at specific entry counts, and combine them into ckpt_cost's value at several points. You are not asked to decide, and should not try to decide, whether any of this is good design.

SCOPE

Two other reviewers, in this same round, are covering different, non-overlapping questions about a related design gap, using separate material. One of them is constructing candidate crash-and-recovery histories to check whether a specific allocation failure is actually reachable in this project's code today. The other is reasoning from already-settled decisions about whether a write being let through the admission check guarantees that a specific space-allocation step for that write will succeed, and about which trees a particular sum should include. Do not answer either of those two questions, and do not try to decide which trees the sum should include; if you find yourself reasoning about either, stop and return to the arithmetic asked below. The facts below give you three candidate versions of that sum to compute values for; computing a value for a candidate is not the same as endorsing it, and you are not asked to pick one.

FACT TABLE A. A shared node-capacity formula, used for every tree named below.

A1. The fixed block size for a code-two node is 16384 bytes.
A2. The part of a code-two node's header that does not depend on the key width is 86 bytes.
A3. After that fixed part, and after two copies of the key width described in A4, there are 29 more reserved bytes before the entry region starts.
A4. header_bytes(key_width) = A2 + 2 * key_width + A3, that is, 86 + 2 * key_width + 29.
A5. node_entry_capacity(key_width, entry_width) = the result of dividing (A1 minus header_bytes(key_width)) by entry_width and dropping any remainder (integer division; for example 7 divided by 2 this way gives 3, not 3.5).
A6. The width of one child pointer, used inside an internal-node entry, is 86 bytes.
A7. internal_entry_width(key_width) = key_width + A6, that is, key_width + 86.

FACT TABLE B. The accounting tree.

B1. Key width = 22 bytes, made of four fields of width 2, 8, 4, 8.
B2. Leaf entry width = 34 bytes.
B3. Internal entry width = 108 bytes, and this equals B1 + A6 (22 + 86 = 108).
B4. Leaf capacity, computed by formula A5 from B1 and B2, equals 477.
B5. Internal capacity, computed by formula A5 from B1 and B3, equals 150.
B6. This tree is one of exactly two trees, in this project's current code, handled by the general multi-level splitting and height-reading machinery described in FACT G below; the other one is the central mapping tree, FACT TABLE C. No other tree named in this material is.
B7. The function that reads this tree's current height reads the level field out of the root node's on-disk header and returns level + 1; it does not keep any separate count anywhere else.

FACT TABLE C. The central mapping tree.

C1. Key width = 27 bytes, made of five fields of width 1, 8, 8, 4, 6.
C2. Leaf entry width = 55 bytes.
C3. Internal entry width = 113 bytes, and this equals C1 + A6 (27 + 86 = 113).
C4. Leaf capacity, computed by formula A5 from C1 and C2, equals 294.
C5. Internal capacity, computed by formula A5 from C1 and C3, equals 143.
C6. This is the other of the exactly two trees named in FACT B6.
C7. The same kind of height-reading function as FACT B7 exists for this tree, reading this tree's own root node's level field and returning level + 1.

FACT TABLE D. The allocation record tree, as it is actually built in this project's code today.

D1. Key width = 10 bytes.
D2. One record (this tree's only entry width) = 20 bytes.
D3. In today's code, this tree is built as exactly one node, always at level 0; no splitting is implemented for it anywhere. If the number of records that would exist after a publish is more than this single node's capacity (computed by formula A5 from D1 and D2), the publish is refused outright with a distinct error, instead of the tree growing an extra level. The capacity computed this way is 812.
D4. This tree is not one of the two trees named in FACT B6 and FACT C6; today's code has no function analogous to FACT B7 or FACT C7 that reads a height for it.

FACT TABLE E. The allocation record tree, under a different, position-addressed design that a separate decision item describes but that has not been implemented anywhere in this project's code yet.

E1. Quoted and translated from that decision item: "Allocation record tree: addressed by absolute slot number, by position. Each leaf covers one fixed range of slots [k * W, (k + 1) * W), W is even (so a two-slot data unit never crosses a leaf boundary), and W is less than or equal to the leaf entry capacity 812 (one record covers at least one slot, so the leaf can always hold it); a record is placed in the leaf that covers its starting slot. The levels above are likewise addressed by position: which range a child node covers is computed from that child's position within its parent. A range with no records is entirely free: that leaf is not written, and the corresponding slot in the parent is left empty. The checker checks, node by node, that the range a node claims to cover really is the range its position dictates, and that a record's last slot does not run past the last slot of the leaf it is in."
E2. The number 812 in E1 is exactly formula A5 applied to D1 and D2; it is the same single-node capacity named in D3, not a separately chosen number.
E3. Quoted and translated, one sentence taken out of a longer paragraph in the same decision item that also covers other, unrelated topics not reproduced here: "The specific value of the leaf width W, the fan-out of the upper level, and the complete field layout of the upper-level leaf entries are things this decision item does not write down; whatever it does not write down, the implementer is to choose following the points above and state that choice in the hand-back; the lead agent will decide and add it into this section afterward."
E4. As of today, no implementer has yet done the work named in E3; today's code still has only the shape described in FACT TABLE D, not the shape described in E1. The leaf width W and the upper-level fan-out for this design are therefore not fixed to any number anywhere in this project as of today.
E5. Note carefully what E1 says about how the levels above the leaf are addressed: a child node's slot range is computed from its position within its parent, not from how many records currently exist. This is a different growth rule from FACT G's rule for the accounting and mapping trees, where a node's existence and a tree's height are driven by how many entries have been inserted. Under E1's rule, whether a tree of this design needs one level or several is driven by how large a slot range the whole tree must be able to address, which is set by the storage device's total slot count, not directly by how many of those slots currently hold a live record.

FACT TABLE F. The tree table.

F1. In today's code, the tree table is built as exactly one node, always at level 0, the same way FACT D3 describes for the allocation record tree; no split code path exists for it anywhere, and there is no function like FACT B7 or FACT C7 that reads a height for it. Separately, quoted and translated, one sentence taken out of the same longer paragraph excerpted in FACT E3: "The accounting tree (the authoritative state), the central mapping tree, the tree table, the instance table, and the inode tree are all built with the one shared code-two btree (the splitting and shrinking rule)." So while that decision groups the tree table together with the accounting and mapping trees under the same growth-and-shrink rule given in FACT G below, today's actual code has not wired the tree table into the general multi-level splitting and height-reading machinery that FACT B6, B7, C6, C7 describe; it only ever writes one node for it, at level 0.
F2. Key width = 8 bytes.
F3. One tree-table entry width = 200 bytes.
F4. No function or written rule was found anywhere in today's code, or in the decision file quoted above, that refuses a publish or otherwise reports an error if the number of tree-table entries after a publish would be more than one node can hold, unlike FACT D3's explicit refusal for the allocation record tree. If today's code ever tried to write more tree-table entries than fit in one node, the general node-building code has its own internal check, which reports the situation as a broken invariant (a program assertion failure), not as a returned, handleable error.

FACT G. The growth, shrink, and height rule. Quoted and translated from the decision item that states it: "How a multi-level code-two tree grows and shrinks (the accounting tree, the central mapping tree, and any tree that still uses the code-two btree): 1) within one publish, a node newly created by a split lands in the same segment as the other units, and the whole root-to-leaf path undergoes copy-on-write as usual, with no extra write-ordering step and no barrier; 2) when a leaf does not fit, it is split down the middle; 3) shrinking only removes empty nodes, and when the root is left with exactly one child, the height drops by one level; there is no below-half merging and no borrowing of entries; 4) separator keys are kept up to date: inserting below the leftmost separator key lowers it, and after entries move between siblings, the separator key for the child on the right is changed to that child's new minimum key; this can be checked level by level: a separator key must be less than or equal to its child node header's minimum key, and greater than the left neighbor child node header's maximum key; 5) tree height equals the level field in the root node's code-two header, plus 1."

FACT G2. A precision found only in the implementing source code's own comment, not in the decision item quoted in FACT G: when a leaf's entry count exceeds capacity after an insertion, the split divides it so the left half gets the result of dividing the entry count by 2 and rounding up, and the right half gets the rest; the same division rule is used for internal nodes when they overflow.

FACT G3. FACT G's rule 3 removes a node from its parent only once that node has become completely empty, and only collapses the root by one level once the root has exactly one child left; it does not say that a tree currently holding a given number of entries always has one single well-defined height. Depending on the sequence of past insertions and deletions, a tree that currently holds a given number of entries could be taller than the shortest tree that could hold that many entries, if it once held more entries and was then only partly emptied out; a shrink from a taller shape stops at "no more entries in this node" or "root has one child", not at "as short as this many entries would allow".

FACT H. The ckpt_cost formula. Quoted and translated from the decision item that states it: "Form: ckpt_cost = the sum, over each record tree, of that tree's current height, plus the number of accounting-tree nodes rewritten in this publish; recomputed at every publish from the current tree heights. Each code-two tree's current height equals the level field read right now from its root node's code-two header, plus 1; it is not cached anywhere in memory. How to read the height of the allocation-record tree and the extent tree, whose shapes are defined by key-space position, is left to a separate round of review. The unit is one 16 KiB metadata block; when substituted into a separate admission formula, which is written in bytes, it is multiplied by 16384. It is a quantity computed at runtime; there is no field for it on disk."

FACT H2. Nowhere in FACT H, or anywhere else given to you in this material, is a number, or a formula in terms of entry counts, given for "the number of accounting-tree nodes rewritten in this publish". Do not invent one. If a question below needs this quantity and none of the given facts pin it down, say so explicitly and leave it as a named unknown in your answer instead of substituting a guessed number.

QUESTIONS

QUESTION SET Q1. The accounting tree. Use FACT TABLE B (leaf capacity 477 from B4, internal capacity 150 from B5) and FACT G (with G2 and G3). Fill in every column of every row of this table. The DERIVATION column must show your arithmetic, not just a number. The TALLER HEIGHT ALSO POSSIBLE column must answer yes or no and, if yes, sketch one example sequence of inserts and deletes that would produce a taller tree still holding exactly this many entries; if no, say why not.

ROW | ENTRY COUNT n | SHORTEST POSSIBLE HEIGHT | DERIVATION | TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n | THIS WOULD BE REFUTED BY
Q1.1 | 1 | | | |
Q1.2 | 477 | | | |
Q1.3 | 478 | | | |
Q1.4 | 71550 (this is 150 * 477) | | | |
Q1.5 | 71551 (this is 150 * 477, plus 1) | | | |

QUESTION SET Q2. The central mapping tree. Use FACT TABLE C (leaf capacity 294 from C4, internal capacity 143 from C5) and FACT G (with G2 and G3). Same columns and same instructions as Q1.

ROW | ENTRY COUNT n | SHORTEST POSSIBLE HEIGHT | DERIVATION | TALLER HEIGHT ALSO POSSIBLE FOR THIS SAME n | THIS WOULD BE REFUTED BY
Q2.1 | 1 | | | |
Q2.2 | 294 | | | |
Q2.3 | 295 | | | |
Q2.4 | 42042 (this is 143 * 294) | | | |
Q2.5 | 42043 (this is 143 * 294, plus 1) | | | |

QUESTION SET Q3. The allocation record tree.

Q3a. Under today's actual code (FACT TABLE D), fill in this table for entry counts 1, 812, and 813. If no valid on-disk state exists at that entry count under today's code, say so explicitly in the HEIGHT column instead of guessing a number, and say why in the DERIVATION column.

ROW | ENTRY COUNT n | VALID STATE UNDER TODAY'S CODE (YES OR NO) | HEIGHT IF VALID | DERIVATION | THIS WOULD BE REFUTED BY
Q3a.1 | 1 | | | |
Q3a.2 | 812 | | | |
Q3a.3 | 813 | | | |

Q3b. Under the position-addressed design of FACT TABLE E, which has not been implemented anywhere in today's code, can the height be computed as a function of entry count alone, the same way Q1 and Q2 do it for the accounting and mapping trees? Using FACT E1 through FACT E5, say plainly what additional information, if any, would be needed. If you can express a height in terms of a named unknown (for example an unspecified fan-out, or an unspecified total device slot count), do so and show the derivation. This question is explicitly asking you to identify what is undetermined, not to guess a number for it. End your answer to Q3b with one sentence starting exactly with "This would be refuted by:".

QUESTION SET Q4. ckpt_cost at five size regimes, for three different candidate choices of which trees the sum in FACT H ranges over. This is a purely arithmetic exercise: assume that in regime i (i from 1 to 5), the accounting tree has the entry count used in row Q1.i above, and the mapping tree has the entry count used in row Q2.i above, at the same time. Nothing in the facts above says these two trees' entry counts actually move together like this; this is only a way to define five points at which to evaluate the sum. For the allocation record tree, in regimes 1 and 2 use the entry counts and heights from Q3a.1 and Q3a.2; regimes 3, 4, and 5 have no valid state for this tree under today's code (FACT D3), so mark any cell that needs this tree's height at those three regimes as undefined, rather than substituting a number. For the tree table, use the height given by FACT F1 at every regime. For every row's shortest-possible height from Q1 and Q2, use the SHORTEST POSSIBLE HEIGHT you gave, not a taller alternative.

First, fill in this table of the heights you are about to add together.

REGIME | ACCOUNTING HEIGHT (FROM Q1) | MAPPING HEIGHT (FROM Q2) | ALLOCATION HEIGHT (FROM Q3a, OR "UNDEFINED") | TREE TABLE HEIGHT (FROM FACT F1)
1 | | | |
2 | | | |
3 | | | |
4 | | | |
5 | | | |

Second, fill in this table of three candidate sums, one column for each candidate set of trees named in the column header, plus the full ckpt_cost expression using FACT H and FACT H2's named unknown. If any cell of the first table needed for a given regime and column is undefined, mark that whole cell "undefined" rather than a number, and say briefly why in the CKPT_COST EXPRESSION column.

REGIME | TWO-TREE SUM (accounting height plus mapping height) | THREE-TREE SUM (TWO-TREE SUM plus allocation height) | FOUR-TREE SUM (THREE-TREE SUM plus tree table height) | CKPT_COST EXPRESSION FOR EACH OF THE THREE SUMS (use FACT H2's named unknown for the accounting-tree-nodes-rewritten term) | THIS WOULD BE REFUTED BY
1 | | | | |
2 | | | | |
3 | | | | |
4 | | | | |
5 | | | | |

FORMAT RULES

Answer every row of Q1 and Q2 (10 rows total), every row of Q3a (3 rows), the whole of Q3b as one answer, and every row of both Q4 tables (10 rows total). Use the exact row labels given (Q1.1 through Q1.5, Q2.1 through Q2.5, Q3a.1 through Q3a.3, Q3b, Q4-HEIGHTS regime 1 through regime 5, Q4-SUMS regime 1 through regime 5). Do not answer any cell with only a single word or short phrase such as only "yes", only "no", or only a bare number with no derivation; always also give the specific reasoning that supports it, even inside a table cell. For every row, end that row's own answer with one sentence starting exactly with "This would be refuted by:" describing the specific observation that would prove that row's answer wrong; for Q4, one such sentence per regime is enough, covering all three sums in that regime. Do not use any markdown emphasis anywhere in your answer. Do not cite, invent, or guess at any source-code line number or design-document line number anywhere in your answer; refer only to the fact labels given above, or to plain descriptive names for functions and constants. Answer entirely in English.

END OF FACTS. Answer Q1.1 through the last row of Q4-SUMS now, in the format given above.
