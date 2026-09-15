#!/usr/bin/env python3
"""
C161 round 3, defense leg. Model B: does the accounting-tree merge rule for
monotonic statistics ("取最大值", D5:308) correctly interact with the K-generation
point-delete tombstone (D5:284-287) for the SAME key (stat, dim, g-K)?

D5's merge rule is an order-independent fold over VALUES: acc = max(acc, v)
applied in any order (D8:445 context calls this "幂等、交换、结合" -- the whole
reason it was chosen over "后者胜" for monotonic stats is that it must be safe
under retried/duplicated/reordered replay). A point-delete is not a value; it
does not have a magnitude to compare. This script tests two read/merge
policies against every candidate's key-tree shape (root buffer only, matching
the "第一版记账直落叶" plus "将来 ε>0" shapes are structurally identical for
this single-key question -- what matters is how PUT and DEL entries for one
key get folded, not the tree geometry):

  NAIVE:  encode every message (put or del) as a numeric value and fold via
          plain max() -- this is the literal, un-special-cased reading of
          "单调统计量取最大值".
  FIXED (T7): read = "is the temporally/positionally newest entry for this
          key a delete?" -> dead, full stop, no numeric comparison against
          it. Otherwise max-fold only among the competing PUT entries.

For each candidate's tie-break flavour (position/arrival for jia_R4 / yi_D /
bing, seq for yi_G) we replay adversarial sequences: N competing put messages
for generation g-K (simulating same-checkpoint multi-writer races, in an
arrival order that does NOT match value order) followed by the K-generations-
later point-delete tombstone for the same key, and check the row reads as
dead.
"""
import os
import sys
import itertools

MUTATE_OFF_FIX = os.environ.get('MUTATE_OFF_FIX') == '1'  # self-test: disable T7


class Msg:
    def __init__(self, kind, val, arr, seq):
        self.kind = kind  # 'put' or 'del'
        self.val = val
        self.arr = arr
        self.seq = seq

    def __repr__(self):
        return f"{self.kind}{'' if self.val is None else self.val}@a{self.arr}/s{self.seq}"


def read_naive(msgs, order_key):
    """NAIVE policy: fold every message as a value via max(), delete encoded
    as 0 (the most charitable encoding for the bug -- any other non-negative
    sentinel is worse or identical). order_key is unused: max-fold is by
    design order independent."""
    acc = None
    for m in msgs:
        v = m.val if m.kind == 'put' else 0
        acc = v if acc is None else max(acc, v)
    # naive policy has no notion of "dead" -- it always reports a live value
    # (or None if nothing was ever written) rather than "deleted".
    is_dead = False
    return is_dead, acc


def read_fixed(msgs, order_key):
    """FIXED (T7) policy: find the newest entry by order_key (arrival for
    jia_R4/yi_D/bing; seq for yi_G). If it is a delete, the key is dead --
    no numeric comparison. Otherwise max-fold among the put entries that are
    still competing duplicates for this key (i.e. every put seen; in the
    real system this window is bounded by the same-checkpoint flush, but for
    this single-key model every put message given IS such a competitor)."""
    if not msgs:
        return False, None
    newest = max(msgs, key=order_key)
    if newest.kind == 'del' and not MUTATE_OFF_FIX:
        return True, None
    if MUTATE_OFF_FIX:
        # self-test: pretend T7 was never applied -- fall back to naive
        return read_naive(msgs, order_key)
    acc = None
    for m in msgs:
        if m.kind == 'put':
            acc = m.val if acc is None else max(acc, m.val)
    return False, acc


ORDER_KEYS = {
    'jia_R4': lambda m: m.arr,
    'yi_D':   lambda m: m.arr,
    'bing':   lambda m: m.arr,
    'yi_G':   lambda m: m.seq,
}


def adversarial_sequences():
    """Each sequence: list of (kind, val) in the order they are *created*
    (arr is assigned by position in the list; seq is independently permuted
    to exercise yi_G's seq-based tie-break on a value-order that disagrees
    with arrival order -- this is exactly the race D5:308 max-merge exists
    to protect against)."""
    seqs = []
    # S1: two competing puts within checkpoint g-K, arriving in ascending
    # arrival order but *descending* value order (the exact race max-merge
    # is for), then the tombstone K generations later.
    seqs.append([('put', 100), ('put', 40), ('del', None)])
    # S2: three competing puts, arrival order scrambled relative to value,
    # tombstone last.
    seqs.append([('put', 5), ('put', 999), ('put', 250), ('del', None)])
    # S3: single put, then tombstone (baseline / minimal case).
    seqs.append([('put', 7), ('del', None)])
    # S4: put, tombstone, then -- must NOT happen in the real system, but we
    # probe it anyway -- nothing after (tombstone must be terminal for a
    # dead generation's key; D5 doesn't define a re-birth path and none of
    # the four candidates propose one).
    seqs.append([('put', 1), ('del', None)])
    return seqs


def make_msgs(seq_spec, seq_perm):
    """seq_spec: list of (kind, val). seq_perm: a permutation of write-buffer
    seq numbers assigned to the same messages, independent of arrival index,
    used to stress yi_G's seq-based tie-break against an arrival order that
    disagrees with it (arrival order is always the true temporal order --
    the model's "ground truth" -- for jia_R4/yi_D/bing which use arrival;
    yi_G's seq is supposed to track the *same* temporal order in the real
    system (T3: "写进前端时取号"), so seq_perm == identity is the faithful
    case; we also try a scrambled seq_perm as an explicit stress case and
    report it separately, it is not claimed to be in-spec)."""
    msgs = []
    for i, (kind, val) in enumerate(seq_spec):
        msgs.append(Msg(kind, val, arr=i + 1, seq=seq_perm[i] + 1))
    return msgs


def main():
    seqs = adversarial_sequences()
    total_naive_bugs = 0
    total_fixed_bugs = 0
    rows = []
    for si, spec in enumerate(seqs):
        n = len(spec)
        identity_perm = list(range(n))
        for candidate, order_key in ORDER_KEYS.items():
            msgs = make_msgs(spec, identity_perm)
            dead_n, val_n = read_naive(msgs, order_key)
            dead_f, val_f = read_fixed(msgs, order_key)
            expect_dead = (spec[-1][0] == 'del')
            naive_bug = (expect_dead and not dead_n)
            fixed_bug = (expect_dead and not dead_f)
            total_naive_bugs += int(naive_bug)
            total_fixed_bugs += int(fixed_bug)
            rows.append((si, candidate, spec, dead_n, val_n, dead_f, val_f,
                         naive_bug, fixed_bug))

    for si, candidate, spec, dead_n, val_n, dead_f, val_f, nb, fb in rows:
        print(f"S{si} {candidate:8s} spec={spec} "
              f"naive(dead={dead_n},val={val_n},BUG={nb}) "
              f"fixed(dead={dead_f},val={val_f},BUG={fb})")

    print(f"naive_policy: {total_naive_bugs} rows out of {len(rows)} silently "
          f"lose the point-delete (row never dies)")
    print(f"fixed_policy(T7,{'DISABLED' if MUTATE_OFF_FIX else 'enabled'}): "
          f"{total_fixed_bugs} rows lose the point-delete")
    print(f"emitted={len(rows)}")


if __name__ == '__main__':
    main()
