#!/usr/bin/env python3
"""按一串操作标签重放一段历史，逐步打印状态与判据（给报告里的具体历史作底）。
用法：trace.py <配置名> <标签…>       例：trace.py a1-jia-stxg-from-birth S SS X0 D C
另：trace.py --split-i36 <配置名> <深度>   穷举到深度，把 I-3.6 在合法状态上红的那几格按「有没有在做的销毁」分开数。"""
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import snapmodel as m  # noqa: E402


def cfg_named(name):
    for n, knobs in m.CONFIGS:
        if n == name:
            return m.Cfg(**knobs)
    raise SystemExit('没有配置 %s' % name)


def show(st, cfg, label):
    def blk(b):
        return 'b%d@%d' % (b[1], b[0])
    head = '-' if st['head'] is None else ','.join(blk(b) for b in st['head']) + ' node=' + blk(st['node'])
    snaps = []
    for s in st['snaps']:
        state = 'live' if s[5] is None else 'destroying' + str(s[5])
        snaps.append('S%d(txg=%s,birth=%d,prev=%d,%s,{%s})' % (
            s[0], m.txg_of(st, cfg, s), s[2], s[3], state, ','.join(blk(b) for b in sorted(s[4]))))
    dl = ' '.join('(%d,%s)' % (e, blk(b)) for e, b in sorted(st['dl']))
    rel = ' '.join('%s:%d' % (blk(b), r) for b, r in sorted(st['rel'].items()))
    mm = m.metrics(st, cfg)
    print('%-6s t=%d head=%s prev(head)=%d' % (label, st['t'], head, st['prev']))
    print('       snaps=[%s]' % ' '.join(snaps))
    print('       deadlist=[%s]' % dl)
    print('       released=[%s]' % rel)
    print('       viol=%s metrics=%s' % (sorted(st['viol']), mm))


def replay(name, labels):
    cfg = cfg_named(name)
    st = m.initial_state()
    show(st, cfg, 'init')
    for label in labels:
        table = dict(m.ops(st, cfg))
        if label not in table:
            raise SystemExit('这一步做不了：%s（可做的：%s）' % (label, sorted(table)))
        nxt = table[label](st)
        if nxt is None:
            raise SystemExit('这一步被拒：%s' % label)
        st = nxt
        show(st, cfg, label)
    print('judge_state=%s' % sorted(m.judge_state(st, cfg)))


def split_i36(name, depth):
    cfg = cfg_named(name)
    tally = dict(mid_destroy=0, quiescent=0)
    stack = [(m.initial_state(), 0)]
    while stack:
        st, d = stack.pop()
        if 'i36_red_legal' in m.judge_state(st, cfg):
            tally['mid_destroy' if m.pending(st) else 'quiescent'] += 1
        if d < depth:
            for _, fn in m.ops(st, cfg):
                nxt = fn(st)
                if nxt is not None:
                    stack.append((nxt, d + 1))
    print('name=%s depth=%d i36_red_legal_mid_destroy=%d i36_red_legal_quiescent=%d' % (
        name, depth, tally['mid_destroy'], tally['quiescent']))


if __name__ == '__main__':
    if sys.argv[1] == '--split-i36':
        split_i36(sys.argv[2], int(sys.argv[3]))
    else:
        replay(sys.argv[1], sys.argv[2:])
