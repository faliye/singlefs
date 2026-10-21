#!/usr/bin/env python3
"""m4-snapshot-r1 攻方腿的抽象模型：只读快照在 D5（快照 / 空间记账机制） 判定下的生死账。

模型的边界（推的，不是 crates/ 的实现）：
- 一次发布原子（D16 已定项 4），崩溃只落在发布边界；崩溃后挂载把未完成的销毁意图续做到底（承诺类）。
- 文件两个数据单元 + 一个用户可见树节点（extent / inode 根的代表，被快照捕获）+ 一个池级元数据单元
  （中央映射 / 记账 / 分配记录树的代表，不被快照捕获），每次发布都换下元数据单元。
- 单元一旦被释放就算可再分配（defer 窗口照 D16 已定项 15 只挡同一窗口，这里不建模复用的时刻）：
  被释放时仍被活头或某个活快照（不在销毁中的）引用 = 丢数据。
每个旋钮的两个值里，BASE 取的是按条款字面不出错的那一个，别的值是攻方要量的候选读法或实现读法。
"""
import argparse
import json
import random
import sys

BASE = dict(
    mode='indep',      # indep：建快照是一次独立发布；same：快照点钉在一次带数据变更的发布末尾（D16 已定项 14）
    prev_val='last',   # indep 下建快照写 prev(头) 的值：last = 建快照之前最后一个已发布的 txg；creation = 建快照那次发布的 txg
    stxg='explicit',   # 快照的 S.txg 从哪读：explicit = 条目里写着、与 prev(头) 写的同一个值（乙）；
                       # birth = 读快照条目的诞生 txg（甲没有 S.txg 字段时最近的字段）；chain = 从下一侧的 prev 派生（甲）
    killprev='old',    # same 下，同一次发布里快照点之前的杀块拿哪个 prev 判：old = 发布前的；new = 这次发布写进树表的新值
    meta='release',    # 池级元数据单元被换下：release = 一律释放；judged = 同样走 D5 判定（birth ≤ prev 进 deadlist）
    node='judged',     # 用户可见树节点被换下：judged = 走 D5 判定；release = 一律释放（阳性对照）
    carrier='full',    # 销毁意图：full = (lo, hi, P) 在写意图那次写死；flag = 快照条目只标「销毁中」，lo / hi 每批现取
    tie='sid',         # 同一 txg 的几个快照在「头 → 活快照链」里的次序：sid = 建的次序；reverse = 反过来
    pass_at='intent',  # prev 传递：intent = 写意图那次（D5 已定项 13）；final = 最后一批（阳性对照）
    interleave=True,   # 销毁分批之间许不许插别的用户操作
    rb_snap='target',  # 管理员回退：target = 树表整张取目标版（快照、prev、deadlist 随之）；refuse_diff = 快照集合不同就拒
    rb_rel='all',      # 回退释放集：all = 现行版已分配减目标版全部引用（活头 ∪ 快照 ∪ deadlist）；live = 今天的写法（只看两版活头）
    rb_res='all',      # 回退复活集：all = 目标版全部引用里已释放的；live = 今天的写法（只看目标版活头）
    rb_destroying='excluded',  # 回退时目标版里销毁中的快照：excluded = 不算引用（C337 带进回退）；included = 算引用
    batch=1,           # 一批放几条
)


class Cfg:
    def __init__(self, **kw):
        unknown = set(kw) - set(BASE)
        if unknown:
            raise SystemExit(f'不认识的旋钮 {sorted(unknown)}')
        self.__dict__.update(BASE)
        self.__dict__.update(kw)


# 状态：dict，值都是不可变的（tuple / frozenset / int），一步操作拷一份浅的 dict 再改。
# 块 = (诞生 txg, 序号)。快照 = (sid, 存的 txg 或 None, 诞生 txg, prev, 捕获的块, 意图)。
# 意图 = None（活快照）/ ('full', lo, hi, P, 下一侧 sid 或 'H') / ('flag', P, 下一侧 sid 或 'H')。

def initial_state():
    st = dict(t=1, nid=0, head=None, node=None, meta=None, prev=0, snaps=(), dl=frozenset(),
              alloc=frozenset(), rel={}, vers=(), nsid=0, viol=frozenset())
    st['head'] = (new_block(st, 1), new_block(st, 1))
    st['node'] = new_block(st, 1)
    st['meta'] = new_block(st, 1)
    st['meta2'] = new_block(st, 1)
    st['vers'] = (version_of(st),)
    return st


def new_block(st, c):
    b = (c, st['nid'])
    st['nid'] += 1
    st['alloc'] = st['alloc'] | {b}
    return b


def flag(st, what):
    st['viol'] = st['viol'] | {what}


def release(st, b):
    if b not in st['alloc']:
        flag(st, 'double_free')
        return
    st['alloc'] = st['alloc'] - {b}
    rel = dict(st['rel'])
    rel[b] = st['t'] + 1
    st['rel'] = rel


def kill(st, cfg, b, kind, p):
    if (kind == 'meta' and cfg.meta == 'release') or (kind == 'node' and cfg.node == 'release'):
        release(st, b)
    elif b[0] > p:
        release(st, b)
    else:
        st['dl'] = st['dl'] | {(p, b)}


def head_refs(st):
    out = set()
    if st['head'] is not None:
        out.update(st['head'])
        out.add(st['node'])
    return out


def live_snaps(st):
    return [s for s in st['snaps'] if s[5] is None]


def live_refs(st):
    out = head_refs(st)
    for s in live_snaps(st):
        out.update(s[4])
    return out


def version_of(st):
    walk_in = set(head_refs(st))
    walk_in.add(st['meta'])
    walk_in.add(st['meta2'])
    walk_ex = set(walk_in)
    for s in st['snaps']:
        walk_in.update(s[4])
        if s[5] is None:
            walk_ex.update(s[4])
    return dict(t=st['t'], head=st['head'], node=st['node'], prev=st['prev'], snaps=st['snaps'],
                dl=st['dl'], walk_in=frozenset(walk_in), walk_ex=frozenset(walk_ex))


# ---- 头 → 活快照链（D5 已定项 12「下一侧怎么找」：挂载时扫树表建的派生态） ----

def raw_txg(cfg, s):
    return s[1] if cfg.stxg == 'explicit' else s[2]


def chain_key(cfg, s):
    if cfg.stxg == 'chain':
        return (s[0],)
    return (raw_txg(cfg, s), s[0] if cfg.tie == 'sid' else -s[0])


def chain_order(st, cfg):
    return sorted(live_snaps(st), key=lambda s: chain_key(cfg, s))


def next_side(st, cfg, s):
    """s 之后的第一个活快照（s 自己在销毁中时按键找），没有就是头（None）。"""
    k = chain_key(cfg, s)
    later = [x for x in chain_order(st, cfg) if chain_key(cfg, x) > k and x[0] != s[0]]
    return later[0] if later else None


def txg_of(st, cfg, s):
    if cfg.stxg in ('explicit', 'birth'):
        return raw_txg(cfg, s)
    if s[5] is not None:
        return s[2]          # 甲：销毁中的快照从链上派生不出来，只剩诞生 txg
    nxt = next_side(st, cfg, s)
    return st['prev'] if nxt is None else nxt[3]


def replace_snap(st, sid, idx, value):
    out = []
    for s in st['snaps']:
        if s[0] == sid:
            s = s[:idx] + (value,) + s[idx + 1:]
        out.append(s)
    st['snaps'] = tuple(out)


def set_prev(st, side, value):
    if side is None:
        st['prev'] = value
    else:
        replace_snap(st, side[0], 3, value)


def snap_by_sid(st, sid):
    for s in st['snaps']:
        if s[0] == sid:
            return s
    return None


# ---- 一次发布 ----

def begin(st, cfg, p_kill):
    st = dict(st)
    c = st['t'] + 1
    old = st['meta']
    st['meta'] = new_block(st, c)
    kill(st, cfg, old, 'meta', p_kill)
    return st, c


def end(st, c):
    st['t'] = c
    st['vers'] = st['vers'] + (version_of(st),)
    if set(st['rel']).intersection(live_refs(st)):
        flag(st, 'loss')
    return st


def rewrite_meta2(st, cfg, c, p_kill):
    """第二个池级元数据单元：只在改到第 1 个数据单元或删文件时换下（映射 / 记账树里不是每次都 COW 的那几片的代表）。"""
    old = st['meta2']
    st['meta2'] = new_block(st, c)
    kill(st, cfg, old, 'meta', p_kill)


def create_snap(st, cfg, value, c):
    sid = st['nsid']
    st['nsid'] = sid + 1
    stored = value if cfg.stxg == 'explicit' else None
    st['snaps'] = st['snaps'] + ((sid, stored, c, st['prev'], frozenset(head_refs(st)), None),)
    st['prev'] = value


def op_w(st, cfg, i, nsnap):
    if st['head'] is None:
        return None
    c = st['t'] + 1
    p_kill = c if (cfg.mode == 'same' and nsnap and cfg.killprev == 'new') else st['prev']
    st, c = begin(st, cfg, p_kill)
    old = st['head'][i]
    head = list(st['head'])
    head[i] = new_block(st, c)
    st['head'] = tuple(head)
    kill(st, cfg, old, 'data', p_kill)
    oldn = st['node']
    st['node'] = new_block(st, c)
    kill(st, cfg, oldn, 'node', p_kill)
    if i == 1:
        rewrite_meta2(st, cfg, c, p_kill)
    for _ in range(nsnap):
        create_snap(st, cfg, c, c)
    return end(st, c)


def op_d(st, cfg, nsnap):
    if st['head'] is None:
        return None
    c = st['t'] + 1
    p_kill = c if (cfg.mode == 'same' and nsnap and cfg.killprev == 'new') else st['prev']
    st, c = begin(st, cfg, p_kill)
    for b in st['head']:
        kill(st, cfg, b, 'data', p_kill)
    kill(st, cfg, st['node'], 'node', p_kill)
    rewrite_meta2(st, cfg, c, p_kill)
    st['head'] = None
    st['node'] = None
    for _ in range(nsnap):
        create_snap(st, cfg, c, c)
    return end(st, c)


def op_s(st, cfg, n):
    t_before = st['t']
    st, c = begin(st, cfg, st['prev'])
    value = t_before if (cfg.mode == 'indep' and cfg.prev_val == 'last') else c
    for _ in range(n):
        create_snap(st, cfg, value, c)
    return end(st, c)


# ---- 销毁快照：写意图 → 分批 → 最后一批删条目（D5 已定项 12） ----

def op_x(st, cfg, sid):
    s = snap_by_sid(st, sid)
    if s is None or s[5] is not None:
        return None
    st, c = begin(st, cfg, st['prev'])
    s = snap_by_sid(st, sid)
    nxt = next_side(st, cfg, s)
    lo = txg_of(st, cfg, s)
    hi = c if nxt is None else txg_of(st, cfg, nxt)
    P = s[3]
    if cfg.pass_at == 'intent':
        set_prev(st, nxt, P)
    nxt_id = 'H' if nxt is None else nxt[0]
    intent = ('full', lo, hi, P, nxt_id) if cfg.carrier == 'full' else ('flag', P, nxt_id)
    replace_snap(st, sid, 5, intent)
    return end(st, c)


def destroy_range(st, cfg, s, c):
    it = s[5]
    if it[0] == 'full':
        return it[1], it[2], it[3]
    P = it[1]
    lo = s[1] if cfg.stxg == 'explicit' else s[2]
    nxt = next_side(st, cfg, s)
    hi = c if nxt is None else txg_of(st, cfg, nxt)
    return lo, hi, P


def pending(st):
    return [s for s in st['snaps'] if s[5] is not None]


def op_b(st, cfg):
    pend = pending(st)
    if not pend:
        return None
    st, c = begin(st, cfg, st['prev'])
    s = pending(st)[0]
    lo, hi, P = destroy_range(st, cfg, s, c)
    cands = sorted(e for e in st['dl'] if lo <= e[0] < hi and e[1][0] > P)
    for e in cands[:cfg.batch]:
        st['dl'] = st['dl'] - {e}
        release(st, e[1])
    if len(cands) <= cfg.batch:
        if cfg.pass_at == 'final':
            set_prev(st, next_side(st, cfg, s), P)
        st['snaps'] = tuple(x for x in st['snaps'] if x[0] != s[0])
    return end(st, c)


def op_c(st, cfg):
    """崩溃并重挂：挂载续做把意图走到底（D8 已定项 5 承诺类）。"""
    if not pending(st):
        return None
    for _ in range(256):
        if not pending(st):
            return st
        st = op_b(st, cfg)
    flag(st, 'continuation_does_not_end')
    return st


# ---- 管理员回退：向前发布一版，内容取目标版（mount.rs 的 roll_back_by_a_forward_publish） ----

def sids(snaps):
    return sorted(s[0] for s in snaps)


def op_r(st, cfg, k):
    vers = st['vers']
    if len(vers) <= k:
        return None
    tgt = vers[len(vers) - 1 - k]
    if cfg.rb_snap in ('refuse_diff', 'today_refuse_diff') and sids(tgt['snaps']) != sids(st['snaps']):
        return None
    # today：照今天 version_to_publish_the_rollback_after 的分法——用户可见树（连同 inode 树条目里的 prev）取目标版，
    # 树表别的条目（快照）与 deadlist 树取现行版
    keep_current = cfg.rb_snap in ('today', 'today_refuse_diff')
    after_snaps = st['snaps'] if keep_current else tgt['snaps']
    after_dl = st['dl'] if keep_current else tgt['dl']
    st, c = begin(st, cfg, st['prev'])
    tgt_live = set()
    if tgt['head'] is not None:
        tgt_live.update(tgt['head'])
        tgt_live.add(tgt['node'])
    tgt_all = set(tgt_live)
    for s in after_snaps:
        if s[5] is None or cfg.rb_destroying == 'included':
            tgt_all.update(s[4])
    tgt_all.update(e[1] for e in after_dl)
    if cfg.rb_rel == 'live':
        relset = head_refs(st) - tgt_live
    else:
        relset = set(st['alloc']) - tgt_all
        relset.discard(st['meta'])
        relset.discard(st['meta2'])
    basis = tgt_live if cfg.rb_res == 'live' else tgt_all
    resset = basis.intersection(st['rel'])
    rel = dict(st['rel'])
    for b in resset:
        del rel[b]
    st['rel'] = rel
    st['alloc'] = st['alloc'].union(resset)
    for b in sorted(relset):
        release(st, b)
    st['head'] = tgt['head']
    st['node'] = tgt['node']
    st['prev'] = tgt['prev']
    st['snaps'] = after_snaps
    st['dl'] = after_dl
    return end(st, c)


def ops(st, cfg):
    """这一步用户（或挂载续做）能做的每一个动作：用户动作全部放开扫，不写死。"""
    out = []
    if pending(st) and not cfg.interleave:
        out.append(('B', lambda x: op_b(x, cfg)))
        out.append(('C', lambda x: op_c(x, cfg)))
        return out
    if st['head'] is not None:
        for i in (0, 1):
            out.append(('W%d' % i, lambda x, i=i: op_w(x, cfg, i, 0)))
            if cfg.mode == 'same':
                out.append(('W%d+S' % i, lambda x, i=i: op_w(x, cfg, i, 1)))
        if cfg.mode == 'same':
            out.append(('W0+SS', lambda x: op_w(x, cfg, 0, 2)))
        out.append(('D', lambda x: op_d(x, cfg, 0)))
        if cfg.mode == 'same':
            out.append(('D+S', lambda x: op_d(x, cfg, 1)))
    out.append(('S', lambda x: op_s(x, cfg, 1)))
    out.append(('SS', lambda x: op_s(x, cfg, 2)))
    for s in live_snaps(st):
        out.append(('X%d' % s[0], lambda x, sid=s[0]: op_x(x, cfg, sid)))
    if pending(st):
        out.append(('B', lambda x: op_b(x, cfg)))
        out.append(('C', lambda x: op_c(x, cfg)))
    for k in (1, 2, 3):
        if len(st['vers']) > k:
            out.append(('R-%d' % k, lambda x, k=k: op_r(x, cfg, k)))
    return out


# ---- 判据：checker 按条款字面判的几条，与地面真相 ----
INF = float('inf')


def sides(st, cfg):
    """各侧的 deadlist 份（D5 已定项 12「哪一份」）：快照 X = [prev(X), X.txg)，头 = [prev(头), ∞)。"""
    out = [(s[0], s[3], txg_of(st, cfg, s), s[4]) for s in chain_order(st, cfg)]
    out.append(('H', st['prev'], INF, frozenset(head_refs(st))))
    return out


def side_index(sd, e):
    for j, side in enumerate(sd):
        if side[1] <= e < side[2]:
            return j
    return None


def i39_violations(st, which):
    bad = 0
    vers = st['vers']
    for b, r in st['rel'].items():
        refs_t = [v['t'] for v in vers if b in v[which]]
        if not refs_t:
            continue
        last = max(refs_t)
        later = [v['t'] for v in vers if v['t'] > last]
        if not later or not (last < r <= min(later)):
            bad += 1
    return bad


def metrics(st, cfg):
    hr = head_refs(st)
    live_blocks = set()
    for s in live_snaps(st):
        live_blocks.update(s[4])
    all_blocks = set()
    for s in st['snaps']:
        all_blocks.update(s[4])
    sd = sides(st, cfg)
    i36 = unowned = 0
    for e, b in st['dl']:
        j = side_index(sd, e)
        if j is None:
            unowned += 1
        elif b[0] > sd[j][1]:
            i36 += 1
    i353 = 0
    for b in live_blocks - hr:
        ents = [e for e in st['dl'] if e[1] == b]
        if len(ents) != 1:
            i353 += 1
            continue
        holders = [j for j, side in enumerate(sd[:-1]) if b in side[3]]
        owner = holders[-1] + 1
        if side_index(sd, ents[0][0]) != owner:
            i353 += 1
    i354 = sum(1 for e in st['dl'] if e[1] in hr)
    conv = sum(1 for e in st['dl'] if e[1] not in all_blocks)
    loss_now = len(set(st['rel']).intersection(live_refs(st)))
    return dict(i36=i36, unowned=unowned, i353=i353, i354=i354, conv=conv, loss_now=loss_now,
                i39_in=i39_violations(st, 'walk_in'), i39_ex=i39_violations(st, 'walk_ex'))


def cleanup(st, cfg, newest_first):
    """把剩下的活快照全删掉、续做到底，看还剩什么没放（终局漏空间）与这一路上丢没丢数据。"""
    x = st
    if pending(x):
        x = op_c(x, cfg)
    for _ in range(32):
        order = chain_order(x, cfg)
        if not order:
            break
        pick = order[-1] if newest_first else order[0]
        x = op_x(x, cfg, pick[0])
        x = op_c(x, cfg) or x
    leak = set(x['alloc']) - head_refs(x)
    leak.discard(x['meta'])
    leak.discard(x['meta2'])
    return x['viol'] - st['viol'], len(leak), len(x['dl'])


CONFIGS = [
    ('base-indep', dict()),
    ('base-same', dict(mode='same')),
    ('a1-jia-stxg-from-birth', dict(stxg='birth')),
    ('a1c-jia-stxg-from-birth-prev-creation', dict(stxg='birth', prev_val='creation')),
    ('a2-jia-stxg-from-chain', dict(stxg='chain')),
    ('a2s-jia-stxg-from-chain-same', dict(stxg='chain', mode='same')),
    ('a3-same-killprev-new', dict(mode='same', killprev='new')),
    ('a4-meta-judged', dict(meta='judged')),
    ('a5-node-release-control', dict(node='release')),
    ('a6-flag-explicit', dict(carrier='flag')),
    ('a7-flag-jia-birth', dict(carrier='flag', stxg='birth')),
    ('a7c-flag-jia-chain', dict(carrier='flag', stxg='chain')),
    ('a8-same-tie-reverse', dict(mode='same', tie='reverse')),
    ('a8i-indep-tie-reverse', dict(tie='reverse')),
    ('a9-rb-release-live', dict(rb_rel='live')),
    ('a10-rb-resurrect-live', dict(rb_res='live')),
    ('a11-rb-today', dict(rb_rel='live', rb_res='live')),
    ('a11r-rb-today-refuse-diff', dict(rb_rel='live', rb_res='live', rb_snap='refuse_diff')),
    ('a11d-rb-destroying-included', dict(rb_destroying='included')),
    ('a14-rb-today-split-yi', dict(rb_snap='today', rb_rel='live', rb_res='live')),
    ('a14r-rb-today-split-jia', dict(rb_snap='today_refuse_diff', rb_rel='live', rb_res='live')),
    ('a14a-rb-split-only', dict(rb_snap='today')),
    ('a12-pass-final-control', dict(pass_at='final')),
    ('a13-flag-no-interleave', dict(carrier='flag', interleave=False)),
]

CATEGORIES = ['loss', 'double_free', 'misdl', 'leak_final', 'loss_in_cleanup', 'blind_misdl',
              'blind_loss', 'i36_red_legal', 'i35_red_legal', 'i39in_red_legal', 'i39ex_red_legal',
              'unowned_legal']


def judge_state(st, cfg):
    m = metrics(st, cfg)
    new_old, leak_old, _ = cleanup(st, cfg, False)
    new_new, leak_new, _ = cleanup(st, cfg, True)
    hit = set()
    if 'loss' in st['viol']:
        hit.add('loss')
    if 'double_free' in st['viol']:
        hit.add('double_free')
    if m['conv']:
        hit.add('misdl')
    if leak_old or leak_new:
        hit.add('leak_final')
    if 'loss' in new_old or 'loss' in new_new or 'double_free' in new_old or 'double_free' in new_new:
        hit.add('loss_in_cleanup')
    quiet = m['i36'] == 0 and m['i353'] == 0 and m['i354'] == 0
    if m['conv'] and quiet:
        hit.add('blind_misdl')
    if m['loss_now'] and quiet:
        hit.add('blind_loss')
    if not hit:
        if m['i36']:
            hit.add('i36_red_legal')
        if m['i353'] or m['i354']:
            hit.add('i35_red_legal')
        if m['i39_in']:
            hit.add('i39in_red_legal')
        if m['i39_ex']:
            hit.add('i39ex_red_legal')
        if m['unowned']:
            hit.add('unowned_legal')
    return hit


def explore(cfg, depth):
    counts = dict.fromkeys(CATEGORIES, 0)
    witness = {}
    nodes = 0
    stack = [(initial_state(), ())]
    while stack:
        st, hist = stack.pop()
        nodes += 1
        for h in judge_state(st, cfg):
            counts[h] += 1
            if h not in witness or len(hist) < len(witness[h]):
                witness[h] = hist
        if len(hist) == depth:
            continue
        for label, fn in ops(st, cfg):
            nxt = fn(st)
            if nxt is not None:
                stack.append((nxt, hist + (label,)))
    return nodes, counts, witness


def sample(cfg, n, length, seed):
    rng = random.Random(seed)
    counts = dict.fromkeys(CATEGORIES, 0)
    witness = {}
    nodes = 0
    for _ in range(n):
        st, hist = initial_state(), ()
        for _ in range(length):
            choices = ops(st, cfg)
            rng.shuffle(choices)
            for label, fn in choices:
                nxt = fn(st)
                if nxt is not None:
                    st, hist = nxt, hist + (label,)
                    break
            else:
                break
        nodes += 1
        for h in judge_state(st, cfg):
            counts[h] += 1
            if h not in witness or len(hist) < len(witness[h]):
                witness[h] = hist
    return nodes, counts, witness


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--depth', type=int, default=0, help='穷举的历史长度上限（每个前缀都判）')
    ap.add_argument('--random', type=int, default=0, help='随机历史条数')
    ap.add_argument('--length', type=int, default=10)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--only', default='', help='逗号分隔的配置名，空 = 全部')
    a = ap.parse_args()
    wanted = set(x for x in a.only.split(',') if x)
    for name, knobs in CONFIGS:
        if wanted and name not in wanted:
            continue
        cfg = Cfg(**knobs)
        if a.depth:
            nodes, counts, witness = explore(cfg, a.depth)
            how = 'exhaustive depth=%d' % a.depth
        else:
            nodes, counts, witness = sample(cfg, a.random, a.length, a.seed)
            how = 'random n=%d length=%d seed=%d' % (a.random, a.length, a.seed)
        row = dict(name=name, knobs=knobs, how=how, states=nodes, counts=counts,
                   witness=dict((k, ' '.join(v)) for k, v in sorted(witness.items())))
        print(json.dumps(row, ensure_ascii=False, sort_keys=True))
        sys.stdout.flush()


if __name__ == '__main__':
    main()
