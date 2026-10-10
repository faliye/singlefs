#!/usr/bin/env python3
"""m3-leaf-death-r3 攻方腿第二次抽样（opus-s2）：克隆头共享叶上三个候选并排的模型（不依赖 crates/，crates/ 没有克隆头）。

两个头：origin O 与克隆 K（在 txg F 从 O 分叉，共享 O 那一刻的叶）。txg 全池一条，每次发布只改一个头。
inode 叶容器的身份 (出生树, 容器号, 出生代)；单元头带身份与诞生代号，不带「哪个头写的」（共享叶头里的出生树是 origin，D8 已定项 6 射程那一句）。
跨头首次 COW 重生（D8 已定项 6 / C113 P6）：K 第一次改一片出生树是 O 的叶，新身份 = (K, 原容器号, 这次 txg)；写不写旧身份的退役记录是臂。
写路径两种：摘叶（删空即摘，`crates/` 今天）与零条目一直留（候选二）；合并：左吸收右（两片之和 ≤ 容量）。unlink 两次发布：意图（nlink 0、本头墓碑树追加整对象墓碑）、最后一批（删记录、删空 / 合并）。
扫盘重建（每次发布之后、两个头各一遍），与那个头的真值比 inode 记录 (号, 对象出生代, nlink)：
  归属：带祖先表的读法——本头出生树的身份取全部版本；克隆头读 origin 出生树的身份只取诞生代号 ≤ F 的版本；
        退役 / 墓碑：本头的全部 + origin 的死亡 txg ≤ F 的。naive 读法不分头（txg 全池一条，谱系父链对两个头是同一条）。
  读法：C1（候选一：退役记录点名的身份整组不收）、TSZ（候选三：本头墓碑点名的 (号, 出生代) 在所有版本里不收 + 供出规则）、
        C2（候选二：零条目版本一直留，每组取最新一版，同号取诞生代号最大的供出；不读退役也不读墓碑）、
        C2r（C2 加「重生时写旧身份的退役记录」）、C2T（C2 加 TSZ 那一套墓碑读法）；各带 _naive（不分头）。
记录 = (号, 对象出生代, nlink, 改动计数)，改动计数让「旧版供出」看得见。
用法：clone_heads.py [深度，默认 4] 或 clone_heads.py trace <写路径臂> <后缀，几步用 + 连>；输出每个（写路径, 读法）的格：后缀数、r / m 为正的后缀数、峰值，与第一条反例。
"""
import itertools
import sys

CAP = 3


class World:
    def __init__(self, zero_path, rebirth_retirement):
        self.zero_path = zero_path
        self.rebirth_retirement = rebirth_retirement
        self.txg = 0
        self.units = []  # (身份, 诞生代号, 记录元组)
        self.heads = {}  # 头 -> dict(leaves=[[身份, [记录]]], watermark, queue, origin, fork)
        self.ret = []    # (写者头, 身份, 死亡 txg)
        self.tomb = []   # (写者头, 号, 对象出生代, 死亡 txg)
        self.log = []

    def publish(self, head, dirty_ids):
        """把这一头叶序里身份在 dirty_ids 的叶各写一版（诞生代号 = 这次 txg）。"""
        h = self.heads[head]
        for leaf in h['leaves']:
            if leaf[0] in dirty_ids:
                self.units.append((leaf[0], self.txg, tuple(leaf[1])))

    def begin(self):
        self.txg += 1
        return self.txg

    def own(self, head, idx):
        """改叶之前：出生树不是本头的叶先重生（新身份），交回新身份。"""
        h = self.heads[head]
        leaf = h['leaves'][idx]
        ident = leaf[0]
        if ident[0] != head:
            new = (head, ident[1], self.txg)
            if self.rebirth_retirement:
                self.ret.append((head, ident, self.txg))
            leaf[0] = new
            self.log.append(f'{head} rebirth {ident}->{new}')
        return leaf[0]

    def create(self, head):
        t = self.begin()
        h = self.heads[head]
        ino = h['watermark']
        h['watermark'] += 1
        rec = (ino, t, 1, t)
        if h['leaves'] and len(h['leaves'][-1][1]) < CAP:
            ident = self.own(head, len(h['leaves']) - 1)
            h['leaves'][-1][1].append(rec)
            dirty = {ident}
        else:
            ident = (head, ino, t)
            h['leaves'].append([ident, [rec]])
            dirty = {ident}
        self.publish(head, dirty)
        self.log.append(f'txg {t} {head} create {ino}')

    def find(self, head, ino):
        for i, leaf in enumerate(self.heads[head]['leaves']):
            for k, r in enumerate(leaf[1]):
                if r[0] == ino:
                    return i, k
        return None

    def modify(self, head, ino):
        """改一条记录（覆盖写那一类）：记录的改动计数换成这次 txg，叶重写一版。"""
        t = self.begin()
        i, k = self.find(head, ino)
        ident = self.own(head, i)
        r = self.heads[head]['leaves'][i][1][k]
        self.heads[head]['leaves'][i][1][k] = (r[0], r[1], r[2], t)
        self.publish(head, {ident})
        self.log.append(f'txg {t} {head} modify {ino}')

    def unlink(self, head, ino):
        h = self.heads[head]
        t = self.begin()
        i, k = self.find(head, ino)
        ident = self.own(head, i)
        r = h['leaves'][i][1][k]
        h['leaves'][i][1][k] = (r[0], r[1], 0, t)
        self.tomb.append((head, r[0], r[1], t))
        h['queue'] = ino
        self.publish(head, {ident})
        self.log.append(f'txg {t} {head} unlink intent {ino}')
        self.judge_all()
        t = self.begin()
        i, k = self.find(head, ino)
        ident = self.own(head, i)
        del h['leaves'][i][1][k]
        h['queue'] = None
        dirty = {ident}
        leaves = h['leaves']
        if not leaves[i][1] and not self.zero_path:
            gone = leaves.pop(i)
            self.ret.append((head, gone[0], t))
            self.log.append(f'txg {t} {head} drop {gone[0]}')
        elif leaves[i][1] or not self.zero_path:
            # 左吸收右：这一片与右边第一片有记录的之和 ≤ 容量
            j = next((q for q in range(i + 1, len(leaves)) if leaves[q][1]), None)
            if j is not None and len(leaves[i][1]) + len(leaves[j][1]) <= CAP:
                rid = self.own(head, j)
                leaves[i][1].extend(leaves[j][1])
                leaves[j][1] = []
                if self.zero_path:
                    dirty.add(rid)
                else:
                    gone = leaves.pop(j)
                    self.ret.append((head, gone[0], t))
                self.log.append(f'txg {t} {head} absorb {rid} into {ident}')
        self.publish(head, dirty)
        self.log.append(f'txg {t} {head} unlink last {ino}')

    def fork(self, origin, clone):
        t = self.begin()
        o = self.heads[origin]
        self.heads[clone] = dict(leaves=[[leaf[0], list(leaf[1])] for leaf in o['leaves']], watermark=o['watermark'],
                                 queue=None, origin=origin, fork=t)
        self.log.append(f'txg {t} fork {clone} from {origin}')

    # ── 扫盘重建
    def visible(self, head, naive):
        h = self.heads[head]
        out = {}
        for ident, birth, recs in self.units:
            if not naive:
                if ident[0] == head:
                    pass
                elif h['origin'] is not None and ident[0] == h['origin'] and birth <= h['fork']:
                    pass
                else:
                    continue
            if ident not in out or birth > out[ident][0]:
                out[ident] = (birth, recs)
        return out

    def scoped(self, head, naive, rows, who):
        h = self.heads[head]
        keep = []
        for row in rows:
            writer, death = row[0], row[-1]
            if naive or writer == head or (h['origin'] is not None and writer == h['origin'] and death <= h['fork']):
                keep.append(row)
        return keep

    def rebuild(self, head, reader):
        naive = reader.endswith('_naive')
        base = reader.replace('_naive', '')
        vis = self.visible(head, naive)
        retired = set()
        tombed = set()
        if base in ('C1', 'C2r'):
            retired = {r[1] for r in self.scoped(head, naive, self.ret, 'ret')}
        if base in ('TSZ', 'C2T'):
            tombed = {(t[1], t[2]) for t in self.scoped(head, naive, self.tomb, 'tomb')}
        best = {}
        for ident, (birth, recs) in vis.items():
            if ident in retired:
                continue
            for rec in recs:
                if (rec[0], rec[1]) in tombed:
                    continue
                if rec[0] not in best or birth > best[rec[0]][0]:
                    best[rec[0]] = (birth, rec)
        return {ino: v[1] for ino, v in best.items()}

    def truth(self, head):
        return {r[0]: r for leaf in self.heads[head]['leaves'] for r in leaf[1]}

    def judge_all(self):
        readers = ['C2', 'C2r', 'C2T', 'C2_naive', 'C2r_naive'] if self.zero_path else ['C1', 'TSZ', 'C1_naive', 'TSZ_naive']
        for head in self.heads:
            truth = self.truth(head)
            q = self.heads[head]['queue']
            for reader in readers:
                got = self.rebuild(head, reader)
                r = sum(1 for ino, rec in got.items() if truth.get(ino) != rec)
                missing = [ino for ino in truth if ino not in got]
                if reader.startswith('TSZ') or reader.startswith('C2T'):
                    missing = [ino for ino in missing if ino != q]  # unlink 途中那一个是 TSZ 自报的代价
                self.results.setdefault((head, reader), []).append((r, len(missing), self.txg))


MENU = ['K.create', 'K.unlink_min', 'K.unlink_max', 'K.modify_min', 'O.create', 'O.unlink_min', 'O.unlink_max', 'O.modify_min']


def act(w, a):
    head, op = a.split('.')
    if head not in w.heads:
        return
    live = sorted(r[0] for leaf in w.heads[head]['leaves'] for r in leaf[1] if r[2] > 0)
    if op == 'create':
        w.create(head)
    elif not live:
        return
    elif op == 'unlink_min':
        w.unlink(head, live[0])
    elif op == 'unlink_max':
        w.unlink(head, live[-1])
    elif op == 'modify_min':
        w.modify(head, live[0])
    w.judge_all()


def prefix(zero_path, rebirth_retirement):
    w = World(zero_path, rebirth_retirement)
    w.results = {}
    w.heads['O'] = dict(leaves=[], watermark=1, queue=None, origin=None, fork=None)
    for _ in range(6):
        w.create('O')
    w.fork('O', 'K')
    w.judge_all()
    return w


def trace(arm, suffix):
    zero, rr = {'drop_path_rebirth_retirement': (False, True), 'drop_path_no_rebirth_retirement': (False, False),
                'zero_path_no_rebirth_retirement': (True, False), 'zero_path_rebirth_retirement': (True, True)}[arm]
    w = prefix(zero, rr)
    for a in [x for x in suffix.split('+') if x and x != 'none']:
        act(w, a)
    for line in w.log:
        print(f'R3C trace {line}')
    for ident, birth, recs in w.units:
        print(f'R3C trace unit identity={ident} birth={birth} records={list(recs)}')
    readers = ['C2', 'C2r', 'C2T'] if zero else ['C1', 'TSZ']
    for head in w.heads:
        print(f'R3C trace head={head} truth={sorted(w.truth(head).values())}')
        for reader in readers:
            print(f'R3C trace head={head} reader={reader} rebuilt={sorted(w.rebuild(head, reader).values())}')


def main():
    if len(sys.argv) > 1 and sys.argv[1] == 'trace':
        trace(sys.argv[2], sys.argv[3])
        return
    depth = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    arms = [('drop_path_rebirth_retirement', False, True), ('drop_path_no_rebirth_retirement', False, False),
            ('zero_path_no_rebirth_retirement', True, False), ('zero_path_rebirth_retirement', True, True)]
    for arm, zero, rr in arms:
        cells = {}
        n = 0
        for d in range(depth + 1):
            for suffix in itertools.product(MENU, repeat=d):
                w = prefix(zero, rr)
                for a in suffix:
                    act(w, a)
                n += 1
                for (head, reader), rows in w.results.items():
                    rp = max(x[0] for x in rows)
                    mp = max(x[1] for x in rows)
                    c = cells.setdefault((head, reader), [0, 0, 0, 0, 0, None])
                    c[0] += 1
                    c[1] += rp > 0
                    c[2] += mp > 0
                    c[3] = max(c[3], rp)
                    c[4] = max(c[4], mp)
                    if (rp > 0 or mp > 0) and c[5] is None:
                        c[5] = '+'.join(suffix) or 'none'
        for (head, reader), c in sorted(cells.items()):
            print(f'R3C arm={arm} head={head} reader={reader} suffixes={c[0]} suffixes_r_pos={c[1]} suffixes_m_pos={c[2]} '
                  f'r_max={c[3]} m_max={c[4]} first={c[5] or "none"}')
        print(f'R3C arm={arm} total_suffixes={n} depth={depth}')


if __name__ == '__main__':
    main()
