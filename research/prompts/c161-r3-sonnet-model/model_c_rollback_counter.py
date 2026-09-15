#!/usr/bin/env python3
"""
C161 round 3, defense leg. Model C: yi_G's T3 requires "计数器随发布落盘"
(seq counter persisted with each publish) but does not say WHERE it lives.
This checks the one placement question that is directly testable without
building a full crash/replay engine: does an admin rollback (D23 已定项14)
to an older root R_old leave the counter internally consistent with what is
actually reachable, comparing "counter lives in the root record" (the same
placement already used for the tree-ID watermark, D8 已定项8 second bullet:
"住根记录，取根环里全部根记录该字段的 max") against "counter lives in a
single superblock field, not reset on rollback".

Scenario: normal operation runs checkpoints g0..g1..g2, each publishing a
root; the seq counter climbs monotonically as write-buffer messages are
issued. An admin rollback selects R_old = the root of g0 (discarding g1, g2).
After rollback a NEW instance resumes issuing seq numbers. We check whether
any freshly issued seq number can collide with a seq number already visible
inside the chosen root's own reachable tree (the only thing a normal reader,
or the C52 detector operating on the chosen root, will ever compare against).
"""
import sys


class Root:
    def __init__(self, txg, counter_field, max_seq_in_tree):
        self.txg = txg
        self.counter_field = counter_field   # what this root's own record says
        self.max_seq_in_tree = max_seq_in_tree  # largest seq actually reachable
                                                  # from this root's accounting tree


def simulate(placement):
    """placement: 'root' (counter stored in the root record, rolls back with
    root selection) or 'superblock' (single global field, does not roll back
    with root selection -- it is simply whatever it was last set to)."""
    roots = []
    global_superblock_counter = 0
    seq = 0

    def checkpoint(n_msgs):
        nonlocal seq, global_superblock_counter
        first = seq + 1
        for _ in range(n_msgs):
            seq += 1
        last = seq
        global_superblock_counter = seq
        roots.append(Root(txg=len(roots), counter_field=seq, max_seq_in_tree=last))
        return first, last

    g0 = checkpoint(3)   # seq 1..3, published as root 0
    g1 = checkpoint(4)   # seq 4..7, published as root 1
    g2 = checkpoint(2)   # seq 8..9, published as root 2 (about to be rolled back past)

    # admin rollback: choose R_old = root 0 (g0). Per D23 已定项14, roots 1
    # and 2 and everything they published are not applied; their accounting
    # values are reloaded from R_old.
    r_old = roots[0]

    if placement == 'root':
        restored_counter = r_old.counter_field   # = 3, comes back with the root
    elif placement == 'superblock':
        restored_counter = global_superblock_counter  # = 9, was never rolled back
    else:
        raise ValueError(placement)

    # new instance resumes issuing seq from restored_counter + 1
    new_seq_start = restored_counter + 1
    new_msgs = [new_seq_start + i for i in range(3)]  # 3 fresh messages

    # what is actually reachable after the rollback (only r_old's own tree --
    # nothing from the abandoned g1/g2 is reachable via the chosen root; this
    # matches C314's "被抛弃时间线的根引用的单元不许重新分配也不许抹头" --
    # they stay on disk but are not part of *this* root's reachable state).
    reachable_max_seq_before_new_writes = r_old.max_seq_in_tree  # = 3

    collision = any(s <= reachable_max_seq_before_new_writes for s in new_msgs)
    # a "collision" here means: a freshly issued seq number duplicates one
    # that is still visible in the tree the new instance is about to write
    # into -- i.e. the counter did not actually advance past what's on disk.
    return {
        'placement': placement,
        'r_old_txg': r_old.txg,
        'restored_counter': restored_counter,
        'reachable_max_seq_before_new_writes': reachable_max_seq_before_new_writes,
        'new_msgs': new_msgs,
        'collision': collision,
    }


def main():
    for placement in ('root', 'superblock'):
        r = simulate(placement)
        print(r)
    r_root = simulate('root')
    r_sb = simulate('superblock')
    print(f"root placement collision: {r_root['collision']} "
          f"(counter correctly reverts to 3 together with the chosen root; "
          f"new seq 4,5,6 do not collide with anything reachable from R_old)")
    print(f"superblock placement collision: {r_sb['collision']} "
          f"(counter stayed at 9 from the abandoned g2, new seq 10,11,12 are "
          f"*larger* than anything reachable -- no collision either, but the "
          f"counter now silently skips 4..9 forever, a live seq range that "
          f"was never actually consumed by anything reachable from R_old)")
    print(f"emitted=2")


if __name__ == '__main__':
    main()
