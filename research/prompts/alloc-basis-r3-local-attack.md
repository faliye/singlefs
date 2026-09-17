SINGLEFS DESIGN QUESTION - LOCAL ATTACK LEG - ROUND alloc-basis-r3

You are one of four independent reviewers checking a design decision for
singlefs, a copy-on-write filesystem being designed from scratch. You are
given a self-contained set of facts below about a two-gate space-admission
mechanism and one specific, concrete recorded history of publishes (from a
fixed calibration script that has actually been run against a model of the
real implementation). Do not assume anything not stated here. Read
carefully: every qualifier in this text is load-bearing (a definition with
one word removed can flip the answer). Do not use any markdown emphasis (no
bold, no italics, no asterisk bullets) anywhere in your answer. Number your
answers to match the question numbers given at the end. For every numbered
answer you give, add one sentence starting with "This would be refuted by:"
describing the specific observation that would prove that answer wrong.
Where a question asks you to fill in a table, give the table with explicit
numbers for every column, not just "yes" or "no".

Background, the two admission gates.

Every allocation request must pass two gates in series before it is
granted. Gate one: available >= demand, where available = capacity minus
still-allocated minus defer-pending-release minus isolated (shadow-ledger
isolated blocks, from an unrelated prior rollback, not otherwise discussed
in this round) minus a fixed per-device switch reserve minus a fixed
checkpoint reserve pool (all in units of 16 KiB blocks). This value is
called gate1 below. Gate two, once gate one passes, is a second, separate
check from a different rule; it is not the subject of this round, do not
analyze gate two's own internal logic, only that it runs after gate one and
only once gate one has already passed.

If gate one fails, the filesystem does not immediately report ENOSPC.
Instead it pushes one empty publish (a publish that changes no
user-visible state) whose purpose is to raise the rollback floor F,
reclaims whatever becomes reclaimable once the new F takes effect, and
re-evaluates gate one. This push-reclaim-rejudge cycle repeats up to B = 8
times total (B = 4 + 2 * k_tol, k_tol = 2) within a single admission
attempt; if gate one still fails after B pushes, the filesystem reports
ENOSPC to the caller.

Exception: the very first publish made by a newly started instance (called
the write-row publish, because it rewrites the instance table with a new
row) never triggers this push-empty-raise-F cycle beforehand. Its own
metadata cost is paid out of a separate, pre-committed switch reserve, not
out of gate one. If that same write-row publish also happens to carry
ordinary user data (for example, a write that was retried after a crash
and remount), that user data still goes through the normal gate-one check
like any other request; if gate one is insufficient for it, the filesystem
reports ENOSPC for that user data write, it does not fall back to pushing
an empty publish first.

The quantity the user sees when querying free space (the value returned by
a df-style call) equals the left-hand side of gate one at the moment of
the query (gate1, as defined above, using the currently published
statistics).

Raising F takes two steps because of a durability rule. F is raised only
once every surviving device carries a persisted root record with the new,
higher F value; before that point, the previous, lower F value remains the
one in effect (call it F_effective, F_eff below). A push-empty publish that
lands on only one device therefore does not, by itself, raise F_effective;
it takes a second push-empty publish landing on the other device before
F_effective actually rises. In the recorded history below, each
push-empty-raise-F event is shown as two consecutive txg rows for exactly
this reason: the first row still shows the old F_eff, the second row shows
the new F_eff once both devices are covered.
Separately, decision D3 item 9 requires two things, both hard requirements.
First: as long as df reports free space >= s blocks, a write of s blocks
must succeed (no false ENOSPC is allowed). Second: after deleting an
s-block object, a write of the same size must succeed within a bounded
number of steps; the bound is 3 publishes that change user-visible state;
publishes the filesystem pushes on its own (the push-empty-raise-F
publishes described above) do not count toward this bound of 3, because
they do not change user-visible state.

Separately, admission also has an in-flight component, which applies
independently of the push-empty mechanism above and is not shown anywhere
in the recorded history below (every request in the recorded history below
happens strictly one at a time, so this component is always zero there).
The rule (decision D28 item 2) is: when a request is evaluated, the
admission reading used for it is published statistics minus the total
amount already approved in-flight for other requests that have been
admitted in the current window but have not yet actually published.
Releases produced within the current window do not generate usable credit
until they are actually published.

Constants used throughout the recorded history below. Total pool capacity:
300 blocks. Ordinary object size: 16 blocks (a few requests instead use a
smaller 1-block object, called tiny). c_max (current worst-case cost of
one push-empty publish) = 4 blocks. Fixed per-device switch reserve = 56
blocks. Fixed checkpoint reserve pool = 4 blocks. So gate1 = 300 minus
still_allocated minus defer_pending minus isolated minus 56 minus 4 = 240
minus still_allocated minus defer_pending minus isolated. B (maximum
pushes per single admission attempt) = 8. Before any publish in this
script, 6 blocks are already still_allocated (4 pre-existing fixed-point
blocks plus a 2-block instance-table unit), and isolated is 0 throughout
this whole recorded history (no rollback happens in it).
Section H1: the recorded history (a real calibration script, one line per
publish; each state line gives every relevant counter as it stands right
after that publish; a line starting with "request" gives, for the write
that triggered that publish, the demand, the df value it was checked
against (the df value at the end of the previous line), the outcome, and
how many push-empty cycles were needed before it was admitted; a line
starting with "after delete" gives, for a write retried after a delete,
which numbered change-user-visible-state publish this is since that
delete, and the outcome). still_allocated, defer_pending, isolated,
gate1, df, and allocatable are all per-device counts, in blocks. oldest is
oldest_valid_root_txg.

txg=1 event=write tiny(1) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=0 still_allocated=7 defer_pending=4 isolated=0 restorable=0 lag=4 gate1=229 df=229 allocatable=289
 request name=tiny demand_blocks=1 df_at_evaluation=234 outcome=[result=success pushes=0]
txg=2 event=write o0(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=0 still_allocated=23 defer_pending=8 isolated=0 restorable=0 lag=8 gate1=209 df=209 allocatable=269
 request name=o0 demand_blocks=16 df_at_evaluation=229 outcome=[result=success pushes=0]
txg=3 event=write o1(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=0 still_allocated=39 defer_pending=12 isolated=0 restorable=0 lag=12 gate1=189 df=189 allocatable=249
 request name=o1 demand_blocks=16 df_at_evaluation=209 outcome=[result=success pushes=0]
txg=4 event=write o2(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=1 still_allocated=55 defer_pending=16 isolated=0 restorable=4 lag=12 gate1=169 df=169 allocatable=229
 request name=o2 demand_blocks=16 df_at_evaluation=189 outcome=[result=success pushes=0]
txg=5 event=write o3(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=2 still_allocated=71 defer_pending=20 isolated=0 restorable=8 lag=12 gate1=149 df=149 allocatable=209
 request name=o3 demand_blocks=16 df_at_evaluation=169 outcome=[result=success pushes=0]
txg=6 event=write o4(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=3 still_allocated=87 defer_pending=24 isolated=0 restorable=12 lag=12 gate1=129 df=129 allocatable=189
 request name=o4 demand_blocks=16 df_at_evaluation=149 outcome=[result=success pushes=0]
txg=7 event=write o5(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=4 still_allocated=103 defer_pending=28 isolated=0 restorable=16 lag=12 gate1=109 df=109 allocatable=169
 request name=o5 demand_blocks=16 df_at_evaluation=129 outcome=[result=success pushes=0]
txg=8 event=write o6(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=5 still_allocated=119 defer_pending=32 isolated=0 restorable=20 lag=12 gate1=89 df=89 allocatable=149
 request name=o6 demand_blocks=16 df_at_evaluation=109 outcome=[result=success pushes=0]
txg=9 event=write o7(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=6 still_allocated=135 defer_pending=36 isolated=0 restorable=24 lag=12 gate1=69 df=69 allocatable=129
 request name=o7 demand_blocks=16 df_at_evaluation=89 outcome=[result=success pushes=0]
txg=10 event=write o8(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=7 still_allocated=151 defer_pending=40 isolated=0 restorable=28 lag=12 gate1=49 df=49 allocatable=109
 request name=o8 demand_blocks=16 df_at_evaluation=69 outcome=[result=success pushes=0]
txg=11 event=write o9(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=8 still_allocated=167 defer_pending=44 isolated=0 restorable=32 lag=12 gate1=29 df=29 allocatable=89
 request name=o9 demand_blocks=16 df_at_evaluation=49 outcome=[result=success pushes=0]
txg=12 event=write o10(16) F=0 F_eff=0 oldest_valid_root_txg=0 ceiling=9 still_allocated=183 defer_pending=48 isolated=0 restorable=36 lag=12 gate1=9 df=9 allocatable=69
 request name=o10 demand_blocks=16 df_at_evaluation=29 outcome=[result=success pushes=0]
txg=13 event=push-empty raise F to 9 F=9 F_eff=0 oldest_valid_root_txg=0 ceiling=9 still_allocated=183 defer_pending=52 isolated=0 restorable=36 lag=12 gate1=5 df=5 allocatable=65
txg=14 event=push-empty raise F to 9 F=9 F_eff=9 oldest_valid_root_txg=0 ceiling=9 still_allocated=183 defer_pending=20 isolated=0 restorable=0 lag=12 gate1=37 df=37 allocatable=97
txg=15 event=write o11(16) F=9 F_eff=9 oldest_valid_root_txg=0 ceiling=10 still_allocated=199 defer_pending=24 isolated=0 restorable=4 lag=12 gate1=17 df=17 allocatable=77
 request name=o11 demand_blocks=16 df_at_evaluation=9 outcome=[result=success pushes=2]
txg=16 event=write o12(16) F=9 F_eff=9 oldest_valid_root_txg=0 ceiling=11 still_allocated=215 defer_pending=28 isolated=0 restorable=8 lag=12 gate1=-3 df=-3 allocatable=57
 request name=o12 demand_blocks=16 df_at_evaluation=17 outcome=[result=success pushes=0]
txg=17 event=push-empty raise F to 11 F=11 F_eff=9 oldest_valid_root_txg=0 ceiling=11 still_allocated=215 defer_pending=32 isolated=0 restorable=8 lag=12 gate1=-7 df=-7 allocatable=53
txg=18 event=push-empty raise F to 11 F=11 F_eff=9 oldest_valid_root_txg=0 ceiling=11 still_allocated=215 defer_pending=36 isolated=0 restorable=8 lag=12 gate1=-11 df=-11 allocatable=49
txg=19 event=push-empty raise F to 11 F=11 F_eff=11 oldest_valid_root_txg=0 ceiling=11 still_allocated=215 defer_pending=32 isolated=0 restorable=0 lag=12 gate1=-7 df=-7 allocatable=53
 request name=o13 demand_blocks=16 df_at_evaluation=-3 outcome=[gate1=-7 < demand=16 result=ENOSPC]
txg=20 event=delete o0(16) F=11 F_eff=11 oldest_valid_root_txg=0 ceiling=12 still_allocated=199 defer_pending=52 isolated=0 restorable=4 lag=28 gate1=-11 df=-11 allocatable=49
 after delete: change_visible_state_publish_count=0 retry_write_blocks=16 outcome=[ENOSPC (df=-11)]
txg=21 event=push-empty raise F to 12 F=12 F_eff=11 oldest_valid_root_txg=0 ceiling=12 still_allocated=199 defer_pending=56 isolated=0 restorable=4 lag=28 gate1=-15 df=-15 allocatable=45
txg=22 event=push-empty raise F to 12 F=12 F_eff=12 oldest_valid_root_txg=0 ceiling=12 still_allocated=199 defer_pending=56 isolated=0 restorable=0 lag=28 gate1=-15 df=-15 allocatable=45
 request name=tiny demand_blocks=1 df_at_evaluation=-11 outcome=[gate1=-15 < demand=1 result=ENOSPC]
 note: the 1-block overwrite was rejected; the workload instead deletes o1 as this window's change-user-visible-state publish
txg=23 event=delete o1(16) F=12 F_eff=12 oldest_valid_root_txg=0 ceiling=15 still_allocated=183 defer_pending=76 isolated=0 restorable=12 lag=44 gate1=-19 df=-19 allocatable=41
 after delete: change_visible_state_publish_count=1 retry_write_blocks=16 outcome=[ENOSPC (df=-19)]
txg=24 event=push-empty raise F to 15 F=15 F_eff=12 oldest_valid_root_txg=1 ceiling=15 still_allocated=183 defer_pending=80 isolated=0 restorable=12 lag=44 gate1=-23 df=-23 allocatable=37
txg=25 event=push-empty raise F to 15 F=15 F_eff=15 oldest_valid_root_txg=2 ceiling=15 still_allocated=183 defer_pending=72 isolated=0 restorable=0 lag=44 gate1=-15 df=-15 allocatable=45
 request name=tiny demand_blocks=1 df_at_evaluation=-19 outcome=[gate1=-15 < demand=1 result=ENOSPC]
 note: the 1-block overwrite was rejected; the workload instead deletes o2 as this window's change-user-visible-state publish
txg=26 event=delete o2(16) F=15 F_eff=15 oldest_valid_root_txg=3 ceiling=16 still_allocated=167 defer_pending=92 isolated=0 restorable=4 lag=60 gate1=-19 df=-19 allocatable=41
 after delete: change_visible_state_publish_count=2 retry_write_blocks=16 outcome=[ENOSPC (df=-19)]
txg=27 event=push-empty raise F to 16 F=16 F_eff=15 oldest_valid_root_txg=4 ceiling=16 still_allocated=167 defer_pending=96 isolated=0 restorable=4 lag=60 gate1=-23 df=-23 allocatable=37
txg=28 event=push-empty raise F to 16 F=16 F_eff=16 oldest_valid_root_txg=5 ceiling=16 still_allocated=167 defer_pending=96 isolated=0 restorable=0 lag=60 gate1=-23 df=-23 allocatable=37
 request name=tiny demand_blocks=1 df_at_evaluation=-19 outcome=[gate1=-23 < demand=1 result=ENOSPC]
 note: the 1-block overwrite was rejected; the workload instead deletes o3 as this window's change-user-visible-state publish
txg=29 event=delete o3(16) F=16 F_eff=16 oldest_valid_root_txg=6 ceiling=20 still_allocated=151 defer_pending=116 isolated=0 restorable=32 lag=60 gate1=-27 df=-27 allocatable=33
 after delete: change_visible_state_publish_count=3 retry_write_blocks=16 outcome=[ENOSPC (df=-27)]
txg=30 event=push-empty raise F to 20 F=20 F_eff=16 oldest_valid_root_txg=7 ceiling=20 still_allocated=151 defer_pending=120 isolated=0 restorable=32 lag=60 gate1=-31 df=-31 allocatable=29
txg=31 event=push-empty raise F to 20 F=20 F_eff=20 oldest_valid_root_txg=8 ceiling=20 still_allocated=151 defer_pending=92 isolated=0 restorable=0 lag=60 gate1=-3 df=-3 allocatable=57
 request name=tiny demand_blocks=1 df_at_evaluation=-27 outcome=[gate1=-3 < demand=1 result=ENOSPC]
 note: the 1-block overwrite was rejected; the workload instead deletes o4 as this window's change-user-visible-state publish
txg=32 event=delete o4(16) F=20 F_eff=20 oldest_valid_root_txg=9 ceiling=23 still_allocated=135 defer_pending=112 isolated=0 restorable=28 lag=60 gate1=-7 df=-7 allocatable=53
 after delete: change_visible_state_publish_count=4 retry_write_blocks=16 outcome=[ENOSPC (df=-7)]
txg=33 event=push-empty raise F to 23 F=23 F_eff=20 oldest_valid_root_txg=10 ceiling=23 still_allocated=135 defer_pending=116 isolated=0 restorable=28 lag=60 gate1=-11 df=-11 allocatable=49
txg=34 event=push-empty raise F to 23 F=23 F_eff=23 oldest_valid_root_txg=11 ceiling=23 still_allocated=135 defer_pending=92 isolated=0 restorable=0 lag=60 gate1=13 df=13 allocatable=73
txg=35 event=write tiny(1) F=23 F_eff=23 oldest_valid_root_txg=12 ceiling=26 still_allocated=135 defer_pending=97 isolated=0 restorable=28 lag=45 gate1=8 df=8 allocatable=68
 request name=tiny demand_blocks=1 df_at_evaluation=-7 outcome=[result=success pushes=2]
 after delete: change_visible_state_publish_count=5 retry_write_blocks=16 outcome=[success (df=8)]
txg=36 event=push-empty raise F to 26 F=26 F_eff=23 oldest_valid_root_txg=13 ceiling=26 still_allocated=135 defer_pending=101 isolated=0 restorable=28 lag=45 gate1=4 df=4 allocatable=64
txg=37 event=push-empty raise F to 26 F=26 F_eff=26 oldest_valid_root_txg=14 ceiling=26 still_allocated=135 defer_pending=77 isolated=0 restorable=0 lag=45 gate1=28 df=28 allocatable=88
txg=38 event=write again(16) F=26 F_eff=26 oldest_valid_root_txg=15 ceiling=29 still_allocated=151 defer_pending=81 isolated=0 restorable=28 lag=29 gate1=8 df=8 allocatable=68
 request name=again demand_blocks=16 df_at_evaluation=8 outcome=[result=success pushes=2]

Source of Section H1: a fixed calibration script's recorded output, lines
255 through 323 of that output file (one particular run labelled with an
arm name and a reading name, both internal identifiers not needed here),
machine-extracted field by field from that file, numbers unchanged;
English event labels are a direct word-for-word substitution of the
original short operation labels (write, delete, push-empty raise F to N,
request, blocks). The constants above come from that same script's
parameter header (total capacity, ordinary object size, c_max, the two
reserve-formula inputs, the push-count limit, the switch reserve, the
checkpoint reserve pool, and the fixed-point block count).
Section SA: a scenario not present in the recorded history above, built
on top of it. This scenario branches right after the txg=12 line above (so
its starting state is exactly that line's state: F=0, F_eff=0,
still_allocated=183, defer_pending=48, isolated=0, gate1=9, df=9,
in_flight_approved=0). Two new write requests, P and Q, each demanding 16
blocks, arrive in the same admission window, before either one has
actually published. They are evaluated one after another, P first, then Q,
using the in-flight rule from decision D28 item 2 quoted above.

Step SA1: P is evaluated. The published-statistics-only gate1 value at
this instant is 9 (unchanged from the txg=12 line, since nothing has
published yet). In_flight_approved so far is 0, so P's admission reading
is 9 - 0 = 9. 9 < 16, gate one fails for P.

Step SA2: because gate one failed, the filesystem pushes empty publishes
to raise F, exactly as shown for the single-request case at txg=13 and
txg=14 above (the starting state is identical, so those two push-empty
publishes proceed exactly as recorded: F reaches 9, F_eff reaches 9 only
after the second push, and the published-statistics-only gate1 value
becomes 37, matching the txg=14 line above). P's admission reading is then
re-evaluated: 37 - 0 = 37 (in_flight_approved for P is still being
evaluated, so it is not yet subtracted from its own check). 37 >= 16, so P
is admitted. In_flight_approved is now increased by P's own demand, to 16.
P has not published yet (its own publish, which would actually write its
16 blocks and update still_allocated, has not happened; only the two
push-empty publishes above have happened so far).

Step SA3: Q is now evaluated, in the same window, still before P has
published. The published-statistics-only gate1 value is still 37
(unchanged, since P has not published). Q's admission reading, per the
in-flight rule, is 37 - 16 (P's in_flight_approved amount) = 21.

Question set SA (fill in every blank with an explicit number, do not
answer only yes or no):

1. Is Q's admission reading (from step SA3) >= 16? State the number and
whether Q is admitted. If Q is admitted, what does in_flight_approved
become after Q (name the number)?

2. Consider two candidate definitions of what a df query returns at the
instant right after Q has been evaluated (both P and Q admitted, neither
has published yet): definition (i), df equals the raw
published-statistics-only gate1 value (37, unaffected by any request's own
in-flight-approved reservation); definition (ii), df equals the admission
reading that a new, third request would see right now, meaning
published-statistics gate1 minus the total in_flight_approved from step
SA1 through step SA3. Give the numeric value of df under each of these two
definitions. State which one of these two definitions is actually
supported by the exact wording given above for what a df query returns
("the left-hand side of gate one... using the currently published
statistics"), and explain in one sentence why the other definition either
does or does not match that wording.

3. Suppose a third request R, demanding s blocks, is evaluated
immediately after step SA3 (P and Q both admitted and in-flight, neither
published), and its own admission reading is computed the same way Q's
was (published-statistics gate1 minus total in_flight_approved from P and
Q). What is the largest integer s for which R would still be admitted
under this rule? Give that number.

4. Using your answer to question 2 (whichever definition of df you
concluded is correct), and your answer to question 3, does there exist a
value of s such that a df query, made at the instant right after step
SA3, reports a free-space number >= s, while a hard would-be write of s
blocks (evaluated the same way R was in question 3, i.e. after P and Q are
both already in-flight-approved) would fail gate one? If such an s exists,
name one concrete value of s and show the arithmetic. If no such s exists
given your answer to question 2, say so explicitly and show why not.
Section SB: a second scenario not present in the recorded history above,
also branching right after the txg=12 line (same starting state: F=0,
F_eff=0, still_allocated=183, defer_pending=48, isolated=0, gate1=9,
df=9). A single write request R, demanding 16 blocks, arrives. Gate one
fails for it (9 < 16), so the filesystem begins the push-empty-raise-F
cycle, as at txg=13 above: the first push-empty publish, carrying the new
value F=9, is durably written to one device only, then the system crashes
(power loss) before the second push-empty publish (the one that would
reach the other device) ever happens. So after the crash, one device has a
newest persisted root carrying F=9; the other device's newest persisted
root is still the one from the txg=12 state, carrying F=0.

The system then reboots and remounts as a new instance. Its first publish
is the write-row publish (per the exception given in the Background
section above): it rewrites the instance table, its own metadata cost
comes from the switch reserve, and it does not push an empty publish
before itself. R's original 16-block write is retried and is carried
inside this same write-row publish as ordinary user data, going through
gate one exactly as described in that exception.

A tentative reading, to be checked against the rules given above, not
assumed correct: because F_effective is defined (in the Background
section) as taking effect "only once every surviving device carries a
persisted root record with the new, higher F value," and the crashed
push-empty publish reached only one device, F_effective after the reboot
is the minimum, over the two devices, of the maximum F carried by each
device's own newest persisted root, which would be min(9, 0) = 0, i.e. the
crash leaves F_effective exactly where it was before the crash began (0),
as if the one-device push-empty had never happened for the purpose of
F_effective. Check this tentative reading against the exact wording given
above and state whether you agree with it or would compute a different
F_effective, and why.

Question set SB (fill in every blank with an explicit number, do not
answer only yes or no):

5. State the value of F_effective right after the reboot, per your own
check of the tentative reading above (agree with 0, or give your own
different number and justify it using only the rules given in this
prompt).

6. Using your answer to question 5, what is gate1's published-statistics
value at the moment the write-row publish evaluates R's retried 16-block
user-data write? (If you agree F_effective reverts to 0, and no other
counter given in the txg=12 state changed during the crashed attempt,
this would be the same gate1=9 as the txg=12 line; if you believe some
counter did change and was not rolled back by the crash, e.g. because the
one-device push-empty's own 4-block cost stayed durably charged against
that one device, say so explicitly and give the resulting gate1 value
instead.)

7. Per the exception rule, the write-row publish does not push another
empty publish before itself even if gate one is insufficient for R's
retried write. Using your answer to question 6, does R's retried
16-block write succeed or get ENOSPC at this write-row publish? State the
outcome and the arithmetic (gate1 value versus demand 16).

8. Suppose this exact crash (first push-empty reaches one device, then
power loss before the second one) happens again on every subsequent
attempt, every time the system tries to retry R's write and its own first
push-empty publish gets underway, forever. Using your answers to
questions 5 through 7, does R's 16-block write ever succeed under this
repeating pattern, or does it fail with ENOSPC every single time, forever,
even though the 21 blocks of headroom that a single successful two-device
push-empty would have unlocked (gate1 rising from 9 to 37, per the
txg=13/txg=14 lines in Section H1) never gets more than one device's
worth of credit under this pattern? State your conclusion and name which
specific rule given in this prompt is the one that keeps this either from
happening or from being a problem, or, if you conclude it is not kept from
happening, name which specific rule's absence allows it.
Question set F (false ENOSPC, across both scenarios). Using everything you
worked out in Section SA and Section SB above:

9. Across Section SA (questions 1 through 4) and Section SB (questions 5
through 8), is there at least one concrete step at which df reports free
space >= s while an actual write of s blocks fails gate one (a false
ENOSPC, forbidden by decision D3 item 9's first requirement)? Answer
separately for SA and for SB. For each of the two, if yes, point to the
specific question number above whose arithmetic shows it, and restate that
arithmetic in one line; if no, say so explicitly for that scenario and
explain in one sentence what stops it, referring to a specific rule given
in this prompt rather than a general argument.
Question set B3 (recoverability within the bound of 3). Decision D3 item
9's second requirement is: after deleting an s-block object, a write of
the same size must succeed within a bounded number of steps, the bound
being 3 publishes that change user-visible state; the filesystem's own
push-empty publishes do not count toward this bound of 3.

10. In Section SA, no delete happens (P and Q are both new writes, not
retries after a delete), so this requirement does not directly apply
there; confirm this in one sentence, or, if you find a delete implied
somewhere in Section SA that was not stated explicitly, name it.

11. In Section SB, the write-row publish (the one carrying R's retried
16-block write) is itself a publish. Does it count as a "publish that
changes user-visible state" for the purposes of the 3-publish bound, given
that its own primary purpose is a metadata-only instance-table rewrite,
but it also happens to carry R's ordinary user-data write in the same
publish? State your answer and justify it from the wording given in this
prompt (a publish carrying user data would ordinarily count; a
metadata-only rewrite by itself would not, per the given wording "an empty
publish does not change user-visible state," and a write-row publish is
explicitly not an empty publish since it rewrites the instance table with
a real new row, but that is a different kind of non-empty publish than a
user-data write; work out which category the wording given here actually
places it in, do not guess).

12. Still in Section SB, if R's write keeps failing with ENOSPC across
repeated crash-and-retry cycles as in question 8, and no user-visible
delete ever happens anywhere in this scenario (R is a plain new write, not
a retry after a delete), does the 3-publish bound from decision D3 item 9
even apply to R's situation at all? State your answer and, if it does not
apply, name what protection (if any) decision D3 item 9 or any other rule
given in this prompt does give to a plain new write (not preceded by a
delete) that keeps failing gate one across repeated crashes in the pattern
from question 8.
Do not answer any question outside questions 1 through 12 as defined
above. In particular do not discuss gate two's own internal logic, the
shadow-ledger isolation mechanism from a prior rollback, or any candidate
definition of df beyond definitions (i) and (ii) given in question 2 --
those are separate problems, not assigned to you.
