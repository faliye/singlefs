#!/usr/bin/env python3
"""
C161 round 3, defense leg. Model D: D11 已定项5 requires "任何一次索引重建
之前，各层缓冲必须先排空，且这一步由重建入口强制执行". "各层" textually
covers >=1 layer of node-internal message buffer, which is what D11 itself
is about. But C161's own finding is that there is a SECOND, structurally
separate layer -- the in-memory write-buffer front-end (D8's structure) --
that D11 已定项5 was written without knowing existed as a distinct thing
(C161 was only discovered later, 2026-09-06, when the front-end/node-buffer
coexistence surfaced). This checks whether "重建入口强制排空各层" -- if
naively implemented as "drain the node buffer" only, because that is the
literal object D11 talks about -- silently misses the front-end, causing a
rebuild to read stale/incomplete state for a key whose only copy is still
sitting in the front-end.
"""


def rebuild_read(node_buffer, leaf, key, drain_scope):
    """drain_scope: which layers get force-drained before rebuild reads.
    'node_only' models a naive reading of D11 已定项5 that only knows about
    the node buffer (the layer D11 itself defines). 'both' models the fix
    this round's candidates require: front-end AND node buffer both drain
    before rebuild."""
    fe = node_buffer.get('__front_end__', {})
    nb = {k: v for k, v in node_buffer.items() if k != '__front_end__'}

    if drain_scope in ('node_only', 'both'):
        # node buffer contents get pushed to leaf unconditionally
        for k, v in nb.items():
            leaf[k] = v
        nb = {}
    if drain_scope == 'both':
        for k, v in fe.items():
            leaf[k] = v
        fe = {}
    # rebuild only ever reads the leaf (the durable, materialized layer);
    # any front-end / node-buffer content left un-drained is invisible to it
    return leaf.get(key)


def main():
    true_value = ('put', 42)
    state = {
        '__front_end__': {'k': true_value},  # the only copy of the write
        'other_key': ('put', 1),             # already in node buffer
    }
    leaf_node_only = {}
    r_node_only = rebuild_read(dict(state), leaf_node_only, 'k', 'node_only')
    leaf_both = {}
    r_both = rebuild_read(dict(state), leaf_both, 'k', 'both')

    print(f"drain_scope=node_only: rebuild reads k={r_node_only} (truth={true_value})"
          f" -> {'STALE/MISSING' if r_node_only != true_value else 'correct'}")
    print(f"drain_scope=both:      rebuild reads k={r_both} (truth={true_value})"
          f" -> {'STALE/MISSING' if r_both != true_value else 'correct'}")

    bug = (r_node_only != true_value)
    fixed = (r_both == true_value)
    print(f"bug_demonstrated={bug} fix_confirmed={fixed}")
    print("emitted=2")


if __name__ == '__main__':
    main()
