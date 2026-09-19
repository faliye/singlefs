#!/usr/bin/env python3
"""m2-s1-r1 攻方腿（Opus）的小模型：wal_full / 乙 在 K9 与 K9' 下，一个故障后已确认的 fsync 还追不追得回来。

只读导入 crates/ 的事实（启动时逐条核锚点串在文件里，缺一条就退出 2）：
  F1 用户数据取最低空槽          allocator.rs  `fn lowest_user_data_slot`
  F2 释放进 defer、释放代 <= floor 才回收   allocator.rs  `pub fn reclaim_released_up_to`
  F3 重放逐项验点名单元、验不过即停   recovery.rs   `report.verification_failed += 1;`
  F4 重放只施加水位之上、同实例的记录   recovery.rs   `record.instance == root.instance && (record.instance, record.checkpoint_txg) > water`
条款事实（背景材料附录抄的原文，不从 crates 读）：根环跨区轮转、区域 = txg mod 3、区域归属盘 0/1/0；根槽不镜像；
单元与 journal 环两盘镜像；wal_full / 乙 的 fsync 与 checkpoint 照 E155 跑前登记 5.4；K9 / K9' 照 5.1。

臂：
  jia        每次 W 是一次全量发布（单元 + 固定点 FP + 记录 + 根槽）；C 是一次空发布（重写 FP，D16 已定项 9）
  wal_K9     W = 数据 + 脊柱 + 记录（不发根）；C = FP + 记录 + 根槽；间隔内被换掉的中间版在根落盘后直接回空闲（K9）
  wal_K9p    同上，中间版在根落盘时记成「已释放、释放代 = 本 checkpoint」，走 defer 谓词（K9'）
  yi_K9      W = 数据 + 记录（只点名叶）；C = 脊柱 + FP + 记录 + 根槽；中间版照 K9
  yi_K9p     同上照 K9'
  wal_K9p_ckptunit  wal_K9p，但重放按「一次发布 = 一次 checkpoint」整组施加（D23 已定项 14 第六条的字面读法之一）
  jia_mut_noflor    阳性对照：甲，但释放不进 defer、当场回空闲（同一次发布就能复用）——必须红
"""
import hashlib, itertools, os, subprocess, sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..'))
ANCHORS = [
    ('crates/singlefs-core/src/allocator.rs', 'pub fn lowest_user_data_slot('),
    ('crates/singlefs-core/src/allocator.rs', 'pub fn reclaim_released_up_to('),
    ('crates/singlefs-core/src/allocator.rs', '.filter(|record| record.is_released && record.generation <= floor)'),
    ('crates/singlefs-core/src/recovery.rs', 'report.verification_failed += 1;'),
    ('crates/singlefs-core/src/recovery.rs', 'record.instance == root.instance && (record.instance, record.checkpoint_txg) > water'),
]

def import_facts():
    out = []
    for rel, needle in ANCHORS:
        path = os.path.join(REPO, rel)
        with open(path, encoding='utf-8') as fh:
            lines = fh.read().split('\n')
        hits = [i + 1 for i, line in enumerate(lines) if needle in line]
        if len(hits) != 1:
            print(f'FACT-MISSING {rel} {needle!r} hits={hits}')
            sys.exit(2)
        blob = subprocess.run(['git', 'hash-object', path], capture_output=True, text=True).stdout.strip()
        out.append(f'fact {rel}:{hits[0]} hash-object={blob} {needle}')
    return out

REGION_DISK = [0, 1, 0]   # D16 已定项 8：根环三个区域的设备归属第一版写死 0 / 1 / 0
FILES = 2

class Pool:
    """一个池：单元区（两盘镜像，所以一份字典就够）、journal 环（镜像）、根环（不镜像，按 txg 落区域）、分配器。"""
    def __init__(self, arm, per_region):
        self.arm = arm
        self.per_region = per_region
        self.units = {}          # slot -> token（两盘同内容）
        self.occupied = set()    # 分配器认为占着的槽
        self.defer = []          # (slot, 释放代)
        self.parked = []         # K9 / K9' 下间隔内被换掉的中间版：根落盘之前谁都不许复用
        self.journal = []        # 记录：dict(jsn, txg, kind, names=[(slot, token)], state=...)
        self.roots = {}          # (region, index) -> (txg, state)
        self.token = 0
        self.jsn = 0
        self.txg = 0             # 最近一个落盘的根的 txg
        self.live = {}           # 最近一个根引用的：('d', f) / 'S' / 'FP' -> (slot, token)
        self.cur = {}            # 内存里的现行版（fsync 已确认的）
        self.born_in_interval = set()  # 这个间隔里写出的槽
        self.acked = {f: None for f in range(FILES)}
        # mkfs：第 0 代根，文件还没有版本
        for f in range(FILES):
            self.cur[('d', f)] = None
        self.cur['S'] = self._write('S0')
        self.cur['FP'] = self._write('FP0')
        self.born_in_interval = set()
        self.live = dict(self.cur)
        self._root(0, self._state())

    def _alloc(self):
        if os.environ.get('ALLOC') == 'bump':      # 对照：分配器永不回头复用（不是 crates 的政策）
            self.bump = getattr(self, 'bump', -1) + 1
            self.occupied.add(self.bump)
            return self.bump
        s = 0                                       # crates：用户数据取最低空槽（allocator.rs lowest_user_data_slot）
        while s in self.occupied:
            s += 1
        self.occupied.add(s)
        return s

    def _write(self, tag):
        self.token += 1
        tok = f'{tag}#{self.token}'
        slot = self._alloc()
        self.units[slot] = tok
        self.born_in_interval.add(slot)
        return (slot, tok)

    def _state(self):
        return {f: (self.cur[('d', f)][1] if self.cur[('d', f)] else None) for f in range(FILES)}

    def _root(self, txg, state):
        region = txg % 3
        self.roots[(region, (txg // 3) % self.per_region)] = (txg, state)

    def _floor(self):
        return min(t for t, _ in self.roots.values())   # 环里最旧有效根（F 不动，取 0 与它的较大者 = 它）

    def _release(self, ref, gen):
        if ref is None:
            return
        slot, _ = ref
        if self.arm == 'jia_mut_noflor':
            self.occupied.discard(slot)        # 变异：释放即回空闲（不进 defer），同一次发布就能复用
            return
        if self.arm.startswith('wal') or self.arm.startswith('yi'):
            if slot in self.born_in_interval:
                self.parked.append(slot)   # 中间版：K9 / K9' 都要等这个间隔的根落盘
                return
        self.defer.append((slot, gen))

    def _reclaim(self):
        floor = self._floor()
        keep = []
        for slot, gen in self.defer:
            if gen <= floor:
                self.occupied.discard(slot)
            else:
                keep.append((slot, gen))
        self.defer = keep

    def _record(self, txg, kind, names, state):
        self.jsn += 1
        self.journal.append(dict(jsn=self.jsn, txg=txg, kind=kind, names=names, state=state))

    # ---- 操作 ----
    def op_write(self, f):
        if self.arm.startswith('jia'):
            t = self.txg + 1
            for key in (('d', f), 'S', 'FP'):
                self._release(self.cur[key], t)          # 释放先于分配（transaction.rs 的 publish_admitted 开头）
            self.cur[('d', f)] = self._write(f'd{f}')
            self.cur['S'] = self._write('S')
            self.cur['FP'] = self._write('FP')
            self._record(t, 'pub', [self.cur[('d', f)], self.cur['S'], self.cur['FP']], self._state())
            self._land(t)
        else:
            c = self.txg + 1                                  # 开放 checkpoint 的 txg
            self._release(self.cur[('d', f)], c)
            self.cur[('d', f)] = self._write(f'd{f}')
            names = [self.cur[('d', f)]]
            if self.arm.startswith('wal'):
                self._release(self.cur['S'], c)
                self.cur['S'] = self._write('S')              # wal_full：脏叶 + 全部祖先
                names.append(self.cur['S'])
            else:
                self.s_dirty = True                           # 乙：祖先延到 checkpoint
            self._record(c, 'fsync', names, self._state())  # 单元 -> 屏障 -> 记录 -> 屏障，不写根槽
        self.acked[f] = self.cur[('d', f)][1]

    def op_checkpoint(self, torn=False):
        t = self.txg + 1
        names = []
        if self.arm.startswith('yi') and getattr(self, 's_dirty', False):
            self._release(self.cur['S'], t)
            self.cur['S'] = self._write('S')
            names.append(self.cur['S'])
            self.s_dirty = False
        self._release(self.cur['FP'], t)
        self.cur['FP'] = self._write('FP')
        names.append(self.cur['FP'])
        kind = 'pub' if self.arm.startswith('jia') else 'ckpt'
        self._record(t, kind, names, self._state())
        if torn:
            return                                            # 崩在 checkpoint 中途：单元与记录已落盘，根槽没写
        self._land(t)

    def _land(self, t):
        self._root(t, self._state())                          # 根槽 FUA
        self.txg = t
        self.live = dict(self.cur)
        for slot in self.parked:
            if self.arm.endswith('K9') :
                self.occupied.discard(slot)                   # K9：根落盘之后中间版的槽直接回空闲
            else:
                self.defer.append((slot, t))                  # K9'：记成已释放、释放代 = 本 checkpoint
        self.parked = []
        self.born_in_interval = set()
        self._reclaim()

    # ---- 恢复 ----
    def recover(self, fault):
        def readable(txg):
            disk = REGION_DISK[txg % 3]
            if fault[0] == 'slot' and fault[1] == txg:
                return False
            if fault[0] == 'disk' and fault[1] == disk:
                return False
            return True
        cands = [(t, s) for t, s in self.roots.values() if readable(t)]
        if not cands:
            return None, 'no-root'
        root_txg, state = max(cands, key=lambda x: x[0])
        state = dict(state)
        self.last_applied_kind = None
        anchor = [r for r in self.journal if r['txg'] == root_txg and r['kind'] in ('pub', 'ckpt')]
        above = sorted((r for r in self.journal if r['txg'] > root_txg), key=lambda r: r['jsn'])
        expected = anchor[0]['jsn'] + 1 if anchor else None
        def verified(r):
            return all(self.units.get(slot) == tok for slot, tok in r['names'])
        if self.arm.endswith('ckptunit'):
            groups = []
            for r in above:
                if groups and groups[-1][0]['txg'] == r['txg']:
                    groups[-1].append(r)
                else:
                    groups.append([r])
            for g in groups:
                if expected is not None and g[0]['jsn'] != expected:
                    break
                if expected is None and g[0]['txg'] != root_txg + 1:
                    break
                if not all(verified(r) for r in g) or g[-1]['kind'] != 'ckpt':
                    break                                     # 整组施加或整组不施加
                state = dict(g[-1]['state'])
                self.last_applied_kind = g[-1]['kind']
                expected = g[-1]['jsn'] + 1
            return state, root_txg
        for r in above:
            if expected is not None and r['jsn'] != expected:
                break
            if expected is None and r['txg'] != root_txg + 1:
                break
            expected = r['jsn'] + 1
            if not verified(r):
                break                                         # 断链即止（recovery.rs）
            state = dict(r['state'])
            self.last_applied_kind = r['kind']
        return state, root_txg


ARMS = ['jia', 'wal_K9', 'wal_K9p', 'yi_K9', 'yi_K9p', 'wal_K9p_ckptunit', 'jia_mut_noflor']
OPS = ['W0', 'W1', 'C']   # 'X'（崩在中途的 checkpoint）只出现在末尾，见 histories()

def run(arm, per_region, history):
    pool = Pool(arm, per_region)
    for op in history:
        if op == 'X':
            pool.op_checkpoint(torn=True)                     # 只出现在末尾：这一次 checkpoint 写到记录、没写根槽就崩
        elif op == 'C':
            pool.op_checkpoint()
        else:
            pool.op_write(int(op[1]))
    return pool

def histories(n):
    """长度 n 的全部操作串，外加「前 n-1 步任意、最后一步是崩在中途的 checkpoint」。"""
    yield from itertools.product(OPS, repeat=n)
    for head in itertools.product(OPS, repeat=n - 1):
        yield head + ('X',)


def faults_for(pool):
    fs = [('none',)]
    fs += [('slot', t) for t, _ in sorted(pool.roots.values())]
    fs += [('disk', 0), ('disk', 1)]
    return fs

def scan(max_len, per_regions):
    """把用户动作（写哪个文件、什么时候 checkpoint）全部放开：长度 1..max_len 的全部操作串 × 全部单故障。"""
    table = {}
    first = {}
    for per_region in per_regions:
        for arm in ARMS:
            cells = hits = hits_no_fault = 0
            by_kind = {"none": 0, "slot": 0, "disk": 0}
            for n in range(1, max_len + 1):
                for history in histories(n):
                    pool = run(arm, per_region, history)
                    for fault in faults_for(pool):
                        state, root = pool.recover(fault)
                        cells += 1
                        if state is None:
                            continue   # 一个根都读不出：挂不上，不算「丢已确认的写」这一格
                        acked = {f: pool.acked[f] for f in range(FILES)}
                        if state != acked:
                            hits += 1
                            by_kind[fault[0]] += 1
                            if fault[0] == 'none':
                                hits_no_fault += 1
                            key = (per_region, arm)
                            if key not in first:
                                first[key] = (history, fault, root, state, acked)
            table[(per_region, arm)] = (cells, hits, hits_no_fault, by_kind)
    return table, first

def jia_blind_cells(max_len, per_regions, arm):
    """arm 中的格，按「同一段操作串上甲在任何一个单故障下都不中」与「甲在某个单故障下也中」分开数。
    两臂的 txg 编号不同（甲每次 W 一个 txg），所以不按同名故障配对，按「甲在这段历史上有没有任何一个单故障会丢」配对。"""
    distinguish = shared = 0
    for per_region in per_regions:
        for n in range(1, max_len + 1):
            for history in histories(n):
                a = run(arm, per_region, history)
                j = run('jia', per_region, history)
                jia_bad = False
                for fault in faults_for(j):
                    sj, _ = j.recover(fault)
                    if sj is not None and sj != j.acked:
                        jia_bad = True
                for fault in faults_for(a):
                    sa, _ = a.recover(fault)
                    if sa is not None and sa != a.acked:
                        if jia_bad:
                            shared += 1
                        else:
                            distinguish += 1
    return distinguish, shared

def main():
    max_len = int(os.environ.get('MAX_LEN', '7'))
    per_regions = [1, 2, 8]
    for line in import_facts():
        print(line)
    print(f'config max_len={max_len} per_region={per_regions} files={FILES} ops={OPS} region_disk={REGION_DISK}')
    table, first = scan(max_len, per_regions)
    for (per_region, arm), (cells, hits, hnf, by_kind) in sorted(table.items()):
        print(f'cells per_region={per_region} arm={arm} cells={cells} lost_acked={hits} '
              f'by_fault_none={by_kind["none"]} by_fault_root_slot={by_kind["slot"]} by_fault_disk={by_kind["disk"]}')
    for (per_region, arm), (history, fault, root, state, acked) in sorted(first.items()):
        print(f'first per_region={per_region} arm={arm} history={"".join(history)} '
              f'fault={fault} selected_root_txg={root} recovered={state} acked={acked}')
    for arm in ['wal_K9', 'yi_K9', 'wal_K9p_ckptunit']:
        d, s = jia_blind_cells(max_len, per_regions, arm)
        print(f'versus_jia arm={arm} hit_and_jia_clean={d} hit_and_jia_hit={s}')
    for arm, (calls, dh, kinds) in replay_needs(max_len).items():
        print(f'replay arm={arm} allocator_calls_during_replay={calls} cells_ending_on_fsync_record={dh} derived_in_memory_not_on_disk={kinds}')
    print('done')

# replay_needs 在文件末尾定义，入口也挪到末尾。


def replay_needs(max_len, per_region=8):
    """H6 / H3：重放本身调不调分配器（把 _alloc 换成抛错）；重放之后留在内存、要由挂载后第一次发布分配落盘的单元是哪几样。
    第一次发布（暖机，D16 已定项 8）按 D16 已定项 9 至少重写固定点 FP，三臂都一样；这里只数「重放派生、盘上没有」的那几样。"""
    out = {}
    for arm in ['jia', 'wal_K9p', 'yi_K9p']:
        alloc_calls = derived_cells = 0
        derived_kinds = set()
        for n in range(1, max_len + 1):
            for history in histories(n):
                pool = run(arm, per_region, history)
                def boom():
                    raise RuntimeError('replay called the allocator')
                pool._alloc = boom
                for fault in faults_for(pool):
                    try:
                        pool.recover(fault)
                    except RuntimeError:
                        alloc_calls += 1
                        continue
                    if getattr(pool, 'last_applied_kind', None) == 'fsync':
                        derived_cells += 1
                        derived_kinds |= {'FP'} if arm.startswith('wal') else {'S', 'FP'}
        out[arm] = (alloc_calls, derived_cells, sorted(derived_kinds))
    return out


if __name__ == '__main__':
    main()
