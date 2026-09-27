Round: sync-local-legs-r1, local attack leg. Attack surface: only the wording of the rules that tell this leg whether to run its own call to ask-local.sh in the foreground or in the background, and the arithmetic of the time limits those rules name. This prompt does not ask about hooks or tooling.

Instructions for your answer. Write every answer in English. Number your answers to match the item numbers below. Do not write any file:line citation and do not write any code line number anywhere in your answer; when you need to point at something, name it by its fact letter (Fact A, Fact B, Fact C, Fact D, Fact E1, Fact E2, Fact E3, Fact E4) or by its table row number instead. For every numbered item, end it with one separate sentence starting with the words "This would be wrong if" that states what observation would show your answer for that item is wrong. Do not answer yes or no by itself anywhere; every item asks for a number, a table, or a stated reason, not a bare yes or no.

===================================================================
FACT SET. Six facts, each with its source.

FACT A. Source: research/scripts/ask-local.sh, line 21. The line itself, verbatim (this is a line of shell code, not natural language):
TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"
Meaning: this sets the shell variable TIMEOUT to the value of the environment variable ASK_LOCAL_TIMEOUT if that variable is set, otherwise to 900. Later in the same script this variable is passed to curl's -m flag (curl's own per-request timeout, in seconds). So: absent any override, a single call to ask-local.sh lets its network request run for up to 900 seconds before curl itself cuts it off. Default value: 900 seconds (900000 milliseconds).

FACT B. Source: the specification of the Bash tool that this leg uses to run shell commands. This text is not stored anywhere in the repository; it is part of the tool's own description given to the agent.
Quoted: "You may specify an optional timeout in milliseconds (up to 600000ms / 10 minutes)."
Meaning: a foreground Bash call's timeout parameter cannot be set above 600000 milliseconds (600 seconds). This is the tool's own absolute ceiling; nothing in this fact set says what happens if a command has no timeout parameter at all, and no other fact given here says either, so do not assume a default value beyond this ceiling.

FACT C. Source: .claude/agents/three-way-local-attack.md, line 29. This is the current text of the definition that is supposed to govern how this very leg calls ask-local.sh. English translation of the relevant sentence:
"Run it in the foreground: bash research/scripts/ask-local.sh <prompt file> > <prefix>-output-s<n>.md; do not use setsid, &, or disown. When the single request's maximum, ASK_LOCAL_TIMEOUT (default 900 seconds), exceeds the Bash tool's foreground cap, switch to starting the same command with Bash's run_in_background instead; once started, end this turn and wait for the completion notification, and take the exit code from that notification."
Note on this sentence's own wording: the source sentence does not say explicitly whether "exceeds the Bash tool's foreground cap" means comparing the fixed number 900 seconds against the fixed cap (a comparison with one answer, decided before any particular call is ever made), or comparing how long one particular call actually turns out to take against the cap (a comparison whose answer is not known until that call has already finished). Both readings are grammatically available in the source sentence; this prompt does not pick one for you.

FACT D. Source: .claude/rules/three-way-inference.md, line 206. English translation:
"The local leg is started with Bash's run_in_background; end this turn and wait for the completion notification (a single request takes at most 900 seconds, which exceeds the foreground cap); the command must not add setsid, &, or disown."
Note on this sentence's own wording: the instruction to use run_in_background is not written with any "when such-and-such happens" clause; the parenthetical "(a single request takes at most 900 seconds, which exceeds the foreground cap)" is offered as the stated reason for the instruction, and it compares two fixed numbers (900 seconds, and "the foreground cap"), not the actual length of any particular call.

FACT E1. Source: .claude/agent-common.md, line 57, opening clause. English translation:
"Long-lived tasks can be waited on; do not give them a timeout, and do not kill them yourself partway through: long-lived tasks such as compiling, mutation tables, and replays are started with Bash's run_in_background; once started, end this turn, and you will be notified when it finishes."
Note: the three named examples (compiling, mutation tables, replays) are introduced with a word meaning "such as" (an open, non-exhaustive list), not a word meaning "namely" or "only". Calling ask-local.sh is not one of the three named examples.

FACT E2. Source: .claude/agent-common.md, line 57, a middle clause of the same line. English translation:
"A command handed to it must keep running all the way until the real work is actually finished before it exits: do not, inside that command, put the work into the background a second time -- do not use disown, coproc, setsid -f, nohup ... &, the detached mode of tmux or screen, or a systemd-run that does not wait until it ends; do not write a bare & inside a subshell or inside $( ... ) either; & is only used when the same command is immediately followed by a plain wait with no arguments to join on it, with each job's exit code collected the way a separately numbered item elsewhere in the same file specifies."

FACT E3. Source: .claude/agent-common.md, line 57, a later clause of the same line (this is the clause naming a foreground timeout ceiling). English translation:
"Ending this turn means this reply writes only a single sentence saying what it is waiting for, and calls no more tools; a foreground command's timeout must not exceed 240000 milliseconds."

FACT E4. Source: .claude/agent-common.md, line 57, closing clause. English translation:
"Waiting for a background task you started yourself must always be done by ending this turn and waiting for the completion notification; do not write a loop that polls its output file."

===================================================================
TABLE 1. Numeric conversion table. Fill in every blank with a number. Show milliseconds as seconds times 1000.

Row 1. ASK_LOCAL_TIMEOUT default (Fact A). Seconds: ___. Milliseconds: ___.
Row 2. Bash tool foreground timeout parameter ceiling (Fact B). Seconds: ___. Milliseconds: ___.
Row 3. agent-common.md foreground command timeout ceiling (Fact E3). Seconds: ___. Milliseconds: ___.
Row 4. Row 1 minus Row 2, in seconds. State the sign (positive, negative, or zero).
Row 5. Row 1 minus Row 3, in seconds. State the sign.
Row 6. Row 2 minus Row 3, in seconds. State the sign.

TASK for Table 1 (item 1 of your answer): fill in every blank in Table 1 with the actual number, then write one paragraph stating, in plain language, what each of Row 4's, Row 5's, and Row 6's sign tells you about whether a 900-second call (Fact A's default) can finish inside a foreground command whose timeout parameter is capped the way Row 2 describes, or the way Row 3 describes.

===================================================================
TASK 2 (item 2 of your answer): routing decision before the call starts.
Using Facts A, B, C, D, and E1: before this leg issues one particular call to ask-local.sh, does it have any fact available to it (from this prompt, from the repository, or from the Bash tool's own specification) that would let it compute, in advance of making that call, whether this particular call will finish inside whatever foreground timeout cap applies? Name every fact your answer relies on by its letter. If your answer is no, state exactly what would have to be true for it to become knowable in advance (for example: a fixed value of ASK_LOCAL_TIMEOUT set below 240 seconds, or something else -- name what you think would be sufficient).

===================================================================
TASK 3 (item 3 of your answer): do Facts C, D, and E1 require the same action.
Build a table with one row for Fact C, one row for Fact D, and one row for Fact E1. For each row, state in your own words: what that fact requires this leg to do about running ask-local.sh -- foreground unconditionally, background unconditionally, foreground by default with a background fallback, or something else -- and whether that fact's own wording names a specific, checkable condition under which the foreground-or-background choice changes (and if so, what that condition is). Then add three more rows, one for each pair (C versus D, C versus E1, D versus E1). For each pair, state: for a call that is expected to take exactly 300 seconds, do the two facts in that pair require the same action from this leg; for a call that is expected to take exactly 700 seconds, do the two facts in that pair require the same action; are your two answers (300 seconds, 700 seconds) the same as each other for that pair, or different.

===================================================================
TASK 4 (item 4 of your answer): worst case if this leg follows Fact C literally and runs the call in the foreground.
Consider two sub-cases separately.
Sub-case (i): this leg runs ask-local.sh in the foreground with an explicit timeout parameter set to Fact E3's own required ceiling of 240000 milliseconds.
Sub-case (ii): this leg runs ask-local.sh in the foreground with an explicit timeout parameter set to Fact B's own maximum of 600000 milliseconds (that is, staying within the tool's absolute limit but not within Fact E3's tighter ceiling).
For each sub-case, using Fact A (ask-local.sh's own curl call can legitimately run up to 900 seconds before ask-local.sh's own internal timeout fires), state: is there an achievable single-request length (state it as a number of seconds, or a range of seconds) for which the Bash tool's own foreground timeout would end the command before ask-local.sh's own curl call would have ended it on its own? Give the numeric range for each sub-case.
Then, for each sub-case, answer: when the Bash tool ends a foreground command early because the command hit the tool's own timeout, and the command line was exactly Fact C's redirection form (bash research/scripts/ask-local.sh <prompt file> > <prefix>-output-s<n>.md), what do you expect to be true of the destination file named after the > sign -- does it get created at all, and if it does, is it empty, partially written, or something else -- and what exit code do you expect this leg to see reported for that call.
State plainly, as a separate sentence for each sub-case, which parts of your answer you derived strictly from the facts given above, and which parts are your own inference about shell or tool behavior that goes beyond those facts.
