#!/usr/bin/env python3
"""C245（超级块槽的写频率，两条条款说反话） 第二轮攻方腿（Opus）的计数模型。只用 std，确定性，零文件操作。

问：丙的最强形态在 H1（回退崩溃链首失据）/ H3（层 1）/ 一次发布多条记录上与条款真值同不同；
乙（inline tail）、丁（根记录带计数器）、ringmax、隔离四个改法在切换、多记录发布、回绕、反向链下中不中。

真值 TRUTH6 = D23 已定项 14 六条口径逐条：链从所选根覆盖的最后一条记录（模型真值 root.covered）之后接、
同实例严格连续、水位之上、提交标记齐全、施加前验证点名单元、回退行 W 截断、第六条按发布边界截短（模型真值 pcount）。
臂（链首怎么找 × 链尾第六条怎么判）：
  DING        丁：根记录带覆盖到的计数器（链首 = 真值），链尾不知道发布边界（施加到最后一个提交的事务）
  YI          乙：记录头带 inline_tail（写这条记录时本实例最新持久根覆盖到的计数器），链首 = (i, T+1) 记录的 inline_tail + 1
  BING_R1     第一轮的丙：环里同实例、txg ≤ T 的最大计数器 + 1
  T1_STRICT   丙：链首 = (i, T+1) 记录里计数器最小的 h；前一条读得出且 txg ≤ T 且 CRC(头) = h.反向链 才接；前一条读不出就不接
  T1_OPT      同上，前一条读不出就接
  T1_TX_*     记录层最强：前一条读不出时再用前前一条的 txg / 提交标记 / 事务号与 h 的事务号判「缺的那条在不在根里」，判不出时 STRICT 不接、OPT 接
  IDX         记录头带「本次发布第几条」：前一条读不出时 h 是第 1 条才接
  IDXN        记录头带「第几条 / 共几条」：链首同 IDX，链尾按「共几条」截短（第六条有输入）
  WALK        丙 + 树：前一条读不出时接 h，但链首与链尾那两次发布都要过「新根段里出生 txg = 这次发布的单元全被这次发布读得出的记录点名」
  CUR_IMPL    fbae43e 的 replay_journal：水位之上按 (实例, 计数器) 排序从第一条起连续、非提交即停、不看反向链
"""
import hashlib
import sys
from dataclasses import dataclass, replace

RING = 12
F_SAFETY = 3
N_LIMIT = RING // F_SAFETY
REGIONS = 3
SLOTS_PER_REGION = 2
UNIT_SLOTS = 24


@dataclass(frozen=True)
class Version:
    b0: object          # (单元落点, 内容) 或 None
    b1: object
    attr: int           # 元数据事务（不写数据单元）改的量，效果只在固定点单元里
    meta: object        # 固定点单元（树表的替身）：(落点, 内容) 或 None


GENESIS = Version(None, None, 0, None)


@dataclass(frozen=True)
class Rec:
    inst: int
    ctr: int
    txg: int
    tx: int             # 0 = 空发布
    commit: bool
    back: int           # 本实例内逻辑前一条记录头的摘要；本实例第一条恒 0（D23 已定项 19 ②）
    seg: Version        # 新根段 = 这次发布的根（D23 已定项 15），一次发布的记录都一样
    named: tuple        # ((落点, 内容), ...)
    inline_tail: int    # 乙才读
    pidx: int           # IDX / IDXN 才读：本次发布第几条（从 1 起）
    pcount: int         # IDXN 与真值读：本次发布共几条


@dataclass(frozen=True)
class Root:
    inst: int
    txg: int
    version: Version
    table: tuple        # ((实例, T, W), ...)
    covered: int        # 模型真值：这个根覆盖的最后一条记录的计数器；丁 = 盘上带着它


def header_digest(record):
    """反向链的摘要：整条头（含它自己的反向链）；64 位截断，模型里不撞（宽度不是这一轮的问题）。"""
    return int(hashlib.sha256(repr(record).encode()).hexdigest()[:16], 16)


def record_slot(counter):
    return (counter - 1) % RING


def root_slot(txg):
    return (txg % REGIONS, (txg // REGIONS) % SLOTS_PER_REGION)


def version_units(version):
    return [unit for unit in (version.b0, version.b1, version.meta) if unit is not None]


class Image:
    def __init__(self):
        self.ring = {}
        self.roots = {}
        self.units = {}
        self.superblock_instance = 0
        self.bad_ring = frozenset()
        self.bad_roots = frozenset()

    def copy(self):
        other = Image()
        other.ring = dict(self.ring)
        other.roots = dict(self.roots)
        other.units = dict(self.units)
        other.superblock_instance = self.superblock_instance
        other.bad_ring = self.bad_ring
        other.bad_roots = self.bad_roots
        return other

    def apply(self, write):
        kind = write[0]
        if kind == 'unit':
            self.units[write[1]] = write[2]
        elif kind == 'rec':
            self.ring[record_slot(write[1].ctr)] = write[1]
        elif kind == 'root':
            self.roots[root_slot(write[1].txg)] = write[1]
        elif kind == 'sb':
            self.superblock_instance = max(self.superblock_instance, write[1])
        elif kind in ('barrier', 'failed'):
            pass
        else:
            raise AssertionError(kind)

    def records(self):
        return {(record.inst, record.ctr): record for slot, record in self.ring.items() if slot not in self.bad_ring}

    def readable_roots(self):
        return [root for key, root in self.roots.items() if key not in self.bad_roots]

    def chosen_root(self):
        return max(self.readable_roots(), key=lambda root: (root.txg, root.inst))

    def max_uid(self):
        contents = list(self.units.values())
        for record in self.ring.values():
            contents += [content for _, content in record.named]
        return max([content[-1] for content in contents], default=0)


def segments(writes):
    """D13 已定项 4 的切段：屏障与 FUA（根槽）切段；失败的写不落盘、不切段。"""
    all_segments, current = [], []
    for write in writes:
        if write[0] == 'barrier':
            if current:
                all_segments.append(current)
                current = []
        elif write[0] == 'root':
            if current:
                all_segments.append(current)
                current = []
            all_segments.append([write])
        elif write[0] == 'failed':
            continue
        else:
            current.append(write)
    if current:
        all_segments.append(current)
    return all_segments


def crash_states(base, writes):
    """层 0：空状态 + 每段的每个非空整写子集（前面各段全落）；另给出这个状态里落过盘的根。"""
    image = base.copy()
    persisted = []
    yield ('nothing', 0), image.copy(), ()
    for index, segment in enumerate(segments(writes)):
        for mask in range(1, 1 << len(segment)):
            state = image.copy()
            chosen = [write for bit, write in enumerate(segment) if mask >> bit & 1]
            for write in chosen:
                state.apply(write)
            yield (index, mask), state, tuple(persisted + [write[1] for write in chosen if write[0] == 'root'])
        for write in segment:
            image.apply(write)
            if write[0] == 'root':
                persisted.append(write[1])


def closed_form(writes):
    return 1 + sum((1 << len(segment)) - 1 for segment in segments(writes))


class Writer:
    def __init__(self, image):
        self.image = image.copy()
        self.writes = []
        self.next_uid = image.max_uid() + 1
        self.versions = []          # (txg, 实例, Version)：这条流里发布完成的版本
        self.all_versions = []      # 同上，含没发布完的（链尾撕裂时实现会装上它），判持久性用

    def emit(self, write):
        self.writes.append(write)
        self.image.apply(write)

    def uid(self):
        value = self.next_uid
        self.next_uid += 1
        return value


@dataclass
class Mount:
    inst: int
    next_ctr: int
    next_txg: int
    next_tx: int
    version: Version
    table: tuple
    covered_durable: int        # 本实例最新持久根覆盖到的计数器（写进 inline_tail）；本实例还没有根时 -1
    last_record: object         # 本实例上一条记录（反向链）
    ring_versions: dict         # 根环各槽的根引用的版本：这些单元不复用（D16 已定项 1 的可再分配界，F 不动）
    isolated: frozenset


def allocate(mount, count):
    used = {slot for slot, _ in version_units(mount.version)} | set(mount.isolated)
    for version in mount.ring_versions.values():
        used |= {slot for slot, _ in version_units(version)}
    free = [slot for slot in range(UNIT_SLOTS) if slot not in used]
    assert len(free) >= count, f'落点不够 used={sorted(used)}'
    return free[:count]


def publish(writer, mount, shape, meta_units, fail_at=None, reuse_data=None, reuse_attr=None):
    """一次发布（D16 已定项 7 的持久顺序）。shape 里每项一个事务：'d0' / 'd1' 写一个数据单元一条记录；
    'm' 只改元数据（不写数据单元，一条记录不点名数据）；'D' 一个事务写两个数据单元、跨两条记录（提交标记在第二条）。
    固定点单元 meta_units ∈ {0, 1} 个，点名在这次发布的最后一条记录。空发布 shape = ()，一条记录事务号 0。
    fail_at：'root' = 根槽写报错（不落盘），整数 m = 第 m 条记录写报错；返回时发布没有完成。
    reuse_data：切换重发时照旧的数据单元（号 ≤ W 的事务不重写数据单元）。"""
    txg = mount.next_txg
    data_order = []             # 这次发布写出（或照旧）的数据单元，按记录序；切换重发时拿它照旧
    data_needed = 0 if reuse_data is not None else sum(1 for kind in shape if kind in ('d0', 'd1')) + \
        2 * sum(1 for kind in shape if kind == 'D')
    slots = iter(allocate(mount, data_needed + meta_units))
    blocks = [mount.version.b0, mount.version.b1]
    attr = mount.version.attr
    unit_writes, plan = [], []
    tx = mount.next_tx
    reuse = iter(reuse_data or [])
    if not shape:
        plan.append([0, True, []])
    for kind in shape:
        if kind in ('d0', 'd1', 'D'):
            named = []
            for block in ((0, 1) if kind == 'D' else (int(kind[1]),)):
                if reuse_data is not None:
                    unit = next(reuse)
                else:
                    unit = (next(slots), ('d', txg, mount.inst, tx, block, writer.uid()))
                    unit_writes.append(unit)
                blocks[block] = unit
                named.append(unit)
                data_order.append(unit)
            if kind == 'D':
                plan.append([tx, False, [named[0]]])
                plan.append([tx, True, [named[1]]])
            else:
                plan.append([tx, True, named])
        else:
            attr = writer.uid() if reuse_attr is None else reuse_attr
            plan.append([tx, True, []])
        tx += 1
    meta = mount.version.meta
    if meta_units:
        meta = (next(slots), ('m', txg, mount.inst, writer.uid()))
        unit_writes.append(meta)
        plan[-1][2] = plan[-1][2] + [meta]
    new_version = Version(blocks[0], blocks[1], attr, meta)
    writer.all_versions.append((txg, mount.inst, new_version))
    for slot, content in unit_writes:
        writer.emit(('unit', slot, content))
    writer.emit(('barrier',))
    for index, (txn, commit, named) in enumerate(plan):
        record = Rec(mount.inst, mount.next_ctr, txg, txn, commit,
                     header_digest(mount.last_record) if mount.last_record is not None else 0,
                     new_version, tuple(named), mount.covered_durable, index + 1, len(plan))
        mount.last_record = record
        mount.next_ctr += 1
        if fail_at == index + 1:
            writer.writes.append(('failed', record))
            return 'failed', new_version, txg, data_order
        writer.emit(('rec', record))
    writer.emit(('barrier',))
    root = Root(mount.inst, txg, new_version, mount.table, mount.next_ctr - 1)
    if fail_at == 'root':
        writer.writes.append(('failed', root))
        return 'failed', new_version, txg, data_order
    writer.emit(('root', root))
    writer.versions.append((txg, mount.inst, new_version))
    mount.ring_versions = dict(mount.ring_versions)
    mount.ring_versions[root_slot(txg)] = new_version
    mount.version = new_version
    mount.next_txg += 1
    mount.next_tx = tx
    mount.covered_durable = mount.next_ctr - 1
    return 'ok', new_version, txg, data_order


SHAPES = {
    '1': ('d0',),               # 单记录发布
    '2': ('d0', 'd1'),          # 两个事务两条记录
    'S': ('d0', 'd0'),          # 第二个事务盖掉第一个的数据单元（同一次发布内被取代）
    'M': ('m', 'd0'),           # 头一个事务只改元数据
    'E': ('d0', 'm'),           # 末一个事务只改元数据
    'D': ('D',),                # 一个事务跨两条记录
    '3': ('d0', 'd1', 'd0'),    # 三个事务
}


def make_filesystem():
    image = Image()
    for region in range(REGIONS):
        image.roots[(region, 0)] = Root(0, 0, GENESIS, (), 0)
    return image


def first_mount_stream(pattern):
    """mkfs → 取号（实例 1）→ 暖机两次空发布（第一次挂载零单元，D16 已定项 9）→ 按 pattern 连做数据发布。"""
    base = make_filesystem()
    writer = Writer(base)
    writer.emit(('sb', 1))
    mount = Mount(1, 1, 1, 1, GENESIS, (), -1, None, {key: GENESIS for key in base.roots}, frozenset())
    publish(writer, mount, (), 0)
    publish(writer, mount, (), 0)
    for key in pattern:
        publish(writer, mount, SHAPES[key], 1)
    return base, writer, mount


ARMS = ('TRUTH6', 'DING', 'DING_DROPLAST', 'YI', 'BING_R1', 'T1_STRICT', 'T1_OPT', 'T1_TX_STRICT', 'T1_TX_OPT', 'FP_LAST',
        'IDX', 'IDXN', 'WALK', 'CUR_IMPL')
RULES = ('prefix', 'ringmax')
MUTATION = None


@dataclass(frozen=True)
class Recovery:
    arm: str
    rule: str
    root: tuple
    applied: tuple
    file: object
    next_ctr: int
    version: Version
    table: tuple
    prefix_end: object
    head: object
    unverified_install: int     # 装上的版本里出生 txg 大于所选根、却没被任何施加了的记录点名的单元数

    def outcome(self):
        return (self.root, self.applied, self.file)


def read_file(image, version):
    for slot, content in version_units(version):
        if image.units.get(slot) != content:
            return 'read-error'
    return (version.b0[1][-1] if version.b0 else 0, version.b1[1][-1] if version.b1 else 0, version.attr)


def verified(image, record):
    return all(image.units.get(slot) == content for slot, content in record.named)


def ring_covered(records, root):
    values = [record.ctr for record in records.values() if record.inst == root.inst and record.txg <= root.txg]
    if values:
        return max(values)
    return 0 if root.inst == 0 else None


def tx_rule(records, root, head_record):
    """记录层能拿到的全部判别量：前一条读不出时，缺的那条 p 在不在根 T 里。"""
    if head_record.tx == 0:
        return 'covered'                    # 空发布只有一条记录 ⇒ p 不在 T+1
    before = records.get((root.inst, head_record.ctr - 2))
    if before is None:
        return 'ambiguous'
    if before.txg < root.txg:
        return 'covered'                    # T 至少一条记录，只能是 p
    if before.txg > root.txg:
        return 'broken'
    if not before.commit:
        return 'covered'                    # p 接着 before 的事务 ⇒ 同一次发布 T
    if before.tx == 0:
        return 'same-publish'               # before 是空发布 T 的唯一一条 ⇒ p 属于 T+1
    if head_record.tx == before.tx + 1:
        return 'same-publish'               # p 与 h 同一个事务
    if head_record.tx == before.tx + 2:
        return 'ambiguous'                  # p 是一个单记录事务，在 T 还是 T+1 记录层看不出
    return 'broken'


def head_for(arm, records, root):
    if root.inst == 0:
        return None                         # 注 1：mkfs 第 0 代根一条都不施加
    if arm in ('TRUTH6', 'DING', 'DING_DROPLAST'):
        return root.covered + 1
    if arm == 'YI':
        hints = {record.inline_tail for record in records.values()
                 if record.inst == root.inst and record.txg == root.txg + 1}
        assert len(hints) <= 1, hints
        if not hints:
            return None
        return next(iter(hints)) + (2 if MUTATION == 'yi_hint_off_by_one' else 1)
    if arm == 'BING_R1':
        covered = ring_covered(records, root)
        return None if covered is None else covered + 1
    candidates = [record for record in records.values() if record.inst == root.inst and record.txg == root.txg + 1]
    if not candidates:
        return None
    head_record = min(candidates, key=lambda record: record.ctr)
    previous = records.get((root.inst, head_record.ctr - 1))
    if previous is not None:
        linked = previous.txg <= root.txg and header_digest(previous) == head_record.back
        if MUTATION == 'no_back_chain':
            linked = previous.txg <= root.txg
        return head_record.ctr if linked else None
    if head_record.back == 0:
        return None
    if arm == 'T1_STRICT':
        return None
    if arm in ('T1_OPT', 'WALK'):
        return head_record.ctr
    if arm in ('IDX', 'IDXN'):
        return head_record.ctr if head_record.pidx == 1 else None
    if arm == 'FP_LAST':
        if head_record.tx == 0:
            return head_record.ctr
        before = records.get((root.inst, head_record.ctr - 2))
        if before is None or before.txg > root.txg:
            return None
        if before.txg < root.txg or before.tx != 0 and not names_fixed_point(before):
            return head_record.ctr
        return None
    verdict = tx_rule(records, root, head_record)
    if verdict == 'covered':
        return head_record.ctr
    if verdict == 'ambiguous' and arm == 'T1_TX_OPT':
        return head_record.ctr
    return None


def names_fixed_point(record):
    return any(content[0] == 'm' for _, content in record.named)


def publish_complete_by_walk(publication):
    """新根段里出生 txg = 这次发布的单元（剪掉更早出生的指针就只剩它们）全被这次发布读得出的记录点名。"""
    if MUTATION == 'walk_skips_check':
        return True
    txg = publication[0].txg
    reachable = {unit for unit in version_units(publication[-1].seg) if unit[1][1] == txg}
    named = {unit for record in publication for unit in record.named}
    return reachable <= named


def walk_chain(arm, image, records, root, head):
    """六条口径里除链首之外的五条 + 第六条（按臂）。返回施加的发布（每次发布一个记录列表）。"""
    caps = [row[2] for row in root.table if row[0] == root.inst]
    cap = min(caps) if caps else None
    committed, pending, last, counter = [], [], None, head
    steps = 0
    while head is not None and steps < N_LIMIT:
        record = records.get((root.inst, counter))
        if record is None or record.txg <= root.txg:
            break
        link_to = last if last is not None else records.get((root.inst, counter - 1))
        if link_to is not None and record.back != header_digest(link_to) and MUTATION != 'no_back_chain':
            break
        if cap is not None and record.tx != 0 and record.tx > cap:
            break
        if not verified(image, record):
            break
        steps += 1
        pending.append(record)
        if record.commit:
            committed.extend(pending)
            pending = []
        last = record
        counter += 1
    publications = []
    for record in committed:
        if publications and publications[-1][0].txg == record.txg:
            publications[-1].append(record)
        else:
            publications.append([record])
    if not publications:
        return publications
    if arm == 'WALK' and not publish_complete_by_walk(publications[0]):
        return []
    tail = publications[-1]
    stopped_at = records.get((root.inst, counter)) if head is not None else None
    evidence_of_more = (pending and pending[-1].txg == tail[0].txg) or \
        (stopped_at is not None and stopped_at.txg == tail[0].txg)
    if evidence_of_more and MUTATION != 'tail_ignores_evidence':
        # 记录层能拿到的那一半第六条：链停在一条读得出、同一 txg 的记录上（验证不过、反向链不接、提交标记没出现），这次发布就没写完
        publications = publications[:-1]
        if not publications:
            return publications
        tail = publications[-1]
    if arm == 'DING_DROPLAST':
        publications = publications[:-1]
    elif arm in ('TRUTH6', 'IDXN'):
        complete = tail[0].pidx == 1 and len(tail) == tail[0].pcount
        if MUTATION == 'truth_ignores_sixth_rule' and arm == 'TRUTH6':
            complete = True
        if not complete:
            publications = publications[:-1]
    elif arm == 'WALK':
        if not publish_complete_by_walk(tail):
            publications = publications[:-1]
    elif arm == 'FP_LAST':
        if tail[-1].tx != 0 and not names_fixed_point(tail[-1]):
            publications = publications[:-1]
    return publications


def recover(image, arm, rule):
    records = image.records()
    root = image.chosen_root()
    ring_max = max((counter for _, counter in records), default=0)
    head = None
    if arm == 'CUR_IMPL':
        above = sorted((record for record in records.values() if (record.inst, record.txg) > (root.inst, root.txg)),
                       key=lambda record: (record.inst, record.ctr))
        applied, version, expected = [], root.version, None
        for record in above[:N_LIMIT]:
            if expected is not None and (record.inst, record.ctr) != expected:
                break
            expected = (record.inst, record.ctr + 1)
            if not record.commit or not verified(image, record):
                break
            applied.append(record)
            version = record.seg
        prefix_end = applied[-1].ctr if applied else None
    else:
        head = head_for(arm, records, root)
        publications = walk_chain(arm, image, records, root, head)
        applied = [record for publication in publications for record in publication]
        version = applied[-1].seg if applied else root.version
        prefix_end = applied[-1].ctr if applied else (head - 1 if head is not None else None)
    end = prefix_end
    if end is None:
        covered = ring_covered(records, root)
        end = covered if covered is not None else ring_max
    next_ctr = end + 1 if rule == 'prefix' or MUTATION == 'ringmax_is_prefix' else max(end, ring_max) + 1
    named = {unit for record in applied for unit in record.named}
    unverified = sum(1 for unit in version_units(version) if unit[1][1] > root.txg and unit not in named)
    return Recovery(arm, rule, (root.inst, root.txg), tuple((record.inst, record.ctr) for record in applied),
                    read_file(image, version), next_ctr, version, root.table, prefix_end, head, unverified)


class Tally:
    def __init__(self):
        self.counts = {}
        self.examples = {}

    def add(self, key, example=None, amount=1):
        self.counts[key] = self.counts.get(key, 0) + amount
        if example is not None and amount and key not in self.examples:
            self.examples[key] = example() if callable(example) else example


def describe_image(image):
    ring = ' '.join(f'[{slot}]({r.inst},{r.ctr},t{r.txg},x{r.tx}{"c" if r.commit else "o"},p{r.pidx}/{r.pcount},'
                    f'it{r.inline_tail},n{len(r.named)})' for slot, r in sorted(image.ring.items()))
    roots = ' '.join(f'({root.inst},{root.txg},cov{root.covered})' for _, root in sorted(image.roots.items())
                     if root.inst != 0)
    bad = ''
    if image.bad_ring or image.bad_roots:
        bad = f' bad_ring={sorted(image.bad_ring)} bad_roots={sorted(image.bad_roots)}'
    return f'ring{{{ring}}} roots{{{roots}}}{bad}'


def describe_recovery(recovery):
    return (f'{recovery.arm}/{recovery.rule}: root={recovery.root} head={recovery.head} applied={list(recovery.applied)} '
            f'file={recovery.file} next={recovery.next_ctr}')


def acceptable_files(writer, persisted_roots, instance):
    """fsync 返回过的版本（本实例两个暖机根都落过盘之后落盘的根里 txg 最大的）及其之后写出的任何版本。"""
    own = sorted(root.txg for root in persisted_roots if root.inst == instance)
    if len(own) < 2:
        return None
    acknowledged = own[-1]
    return {read_file_view(version) for txg, inst, version in writer.all_versions if txg >= acknowledged}


def read_file_view(version):
    return (version.b0[1][-1] if version.b0 else 0, version.b1[1][-1] if version.b1 else 0, version.attr)


def patterns(max_length, keys=tuple(SHAPES)):
    result, frontier = [()], [()]
    for _ in range(max_length):
        frontier = [pattern + (key,) for pattern in frontier for key in keys]
        result.extend(frontier)
    return result


def all_recoveries(image, arms=ARMS):
    return {(arm, rule): recover(image, arm, rule) for arm in arms for rule in RULES}


def judge_against_truth(tally, family, recoveries, context, acceptable=None):
    for (arm, rule), recovery in recoveries.items():
        truth = recoveries[('TRUTH6', rule)]
        ding = recoveries[('DING', rule)]
        base = (family, arm, rule)
        example = lambda: f'{context()} || {describe_recovery(recovery)} || {describe_recovery(truth)}'
        tally.add(base + ('states',))
        tally.add(base + ('outcome_differs',), example, int(recovery.outcome() != truth.outcome()))
        tally.add(base + ('file_differs',), example, int(recovery.file != truth.file))
        tally.add(base + ('head_differs_vs_DING',), example, int(recovery.outcome() != ding.outcome()))
        tally.add(base + ('next_differs',), example, int(recovery.next_ctr != truth.next_ctr))
        tally.add(base + ('unverified_install',), example, int(recovery.unverified_install > 0))
        tally.add(base + ('read_error_truth_reads',), example,
                  int(recovery.file == 'read-error' and truth.file != 'read-error'))
        if acceptable is not None:
            tally.add(base + ('durability_states',))
            tally.add(base + ('durability_violations',), lambda: f'{context()} || acceptable={sorted(acceptable)} || '
                      f'{describe_recovery(recovery)}', int(recovery.file not in acceptable))
            tally.add(base + ('durability_violations_truth_holds',), lambda: f'{context()} || acceptable='
                      f'{sorted(acceptable)} || {describe_recovery(recovery)} || {describe_recovery(truth)}',
                      int(recovery.file not in acceptable and truth.file in acceptable))
            tally.add(base + ('durability_holds_truth_violates',), lambda: f'{context()} || acceptable='
                      f'{sorted(acceptable)} || {describe_recovery(recovery)} || {describe_recovery(truth)}',
                      int(recovery.file in acceptable and truth.file not in acceptable))


def run_layer0(tally, max_length, family='L0'):
    run_layer0_patterns(tally, patterns(max_length), family)


def run_layer0_patterns(tally, pattern_list, family):
    for pattern in pattern_list:
        base, writer, _ = first_mount_stream(pattern)
        tally.add((family, 'closed_form_ok'), None, int(sum(1 for _ in crash_states(base, writer.writes))
                                                        == closed_form(writer.writes)))
        for key, image, persisted in crash_states(base, writer.writes):
            recoveries = all_recoveries(image)
            acceptable = acceptable_files(writer, persisted, 1)
            judge_against_truth(tally, family, recoveries,
                                lambda: f'pattern={list(pattern)} crash={key} {describe_image(image)}', acceptable)


def faulted_images(image):
    occupied = sorted(image.ring)
    root_keys = sorted(key for key, root in image.roots.items() if root.inst != 0)
    for slot in occupied:
        faulted = image.copy()
        faulted.bad_ring = frozenset({slot})
        yield f'ring{slot}', faulted
    for key in root_keys:
        faulted = image.copy()
        faulted.bad_roots = frozenset({key})
        yield f'root{key}', faulted
        for slot in occupied:
            double = faulted.copy()
            double.bad_ring = frozenset({slot})
            yield f'root{key}+ring{slot}', double


def run_layer1(tally, max_length, family='L1'):
    for pattern in patterns(max_length):
        base, writer, _ = first_mount_stream(pattern)
        for key, image, persisted in crash_states(base, writer.writes):
            for fault, faulted in faulted_images(image):
                recoveries = all_recoveries(faulted)
                acceptable = acceptable_files(writer, persisted, 1)
                kind = 'root+ring' if '+' in fault else ('root' if fault.startswith('root') else 'ring')
                judge_against_truth(tally, f'{family}-{kind}', recoveries,
                                    lambda: f'pattern={list(pattern)} crash={key} fault={fault} {describe_image(faulted)}',
                                    acceptable)


def new_instance_number(image):
    return max([image.superblock_instance] + [root.inst for root in image.readable_roots()]) + 1


def cj2_txg(image):
    return max([root.txg for root in image.readable_roots()] +
               [record.txg for record in image.records().values()]) + 1


def run_second_mount(tally, max_length, family='M2'):
    """第一次挂载崩在任一层 0 状态 → 各臂自己恢复 → 新实例取号、暖机两次空发布（每次一个固定点单元）→ 崩在任一状态 → 各臂自己恢复。"""
    for pattern in patterns(max_length):
        base, writer1, _ = first_mount_stream(pattern)
        for key1, image1, persisted1 in crash_states(base, writer1.writes):
            acceptable1 = acceptable_files(writer1, persisted1, 1)
            per_state = {}
            for arm in ARMS:
                for rule in RULES:
                    first = recover(image1, arm, rule)
                    writer2 = Writer(image1)
                    instance = new_instance_number(image1)
                    writer2.emit(('sb', instance))
                    writer2.emit(('barrier',))
                    ring_versions = {key: root.version for key, root in image1.roots.items()
                                     if key not in image1.bad_roots}
                    mount = Mount(instance, first.next_ctr, cj2_txg(image1), 1, first.version, first.table, -1, None,
                                  ring_versions, frozenset())
                    publish(writer2, mount, (), 1)
                    publish(writer2, mount, (), 1)
                    for key2, image2, persisted2 in crash_states(image1, writer2.writes):
                        second = recover(image2, arm, rule)
                        per_state.setdefault(key2, {})[(arm, rule)] = (second, image2, first, writer2)
            for key2, entries in per_state.items():
                for (arm, rule), (second, image2, first, writer2) in entries.items():
                    truth = recover(image2, 'TRUTH6', rule)
                    base_key = (family, arm, rule)
                    context = lambda: (f'pattern={list(pattern)} crash1={key1} first={describe_recovery(first)} '
                                       f'crash2={key2} {describe_image(image2)}')
                    tally.add(base_key + ('states',))
                    tally.add(base_key + ('outcome_differs',), lambda: f'{context()} || {describe_recovery(second)} || '
                              f'{describe_recovery(truth)}', int(second.outcome() != truth.outcome()))
                    tally.add(base_key + ('read_error_truth_reads',), lambda: f'{context()} || {describe_recovery(second)}'
                              f' || {describe_recovery(truth)}',
                              int(second.file == 'read-error' and truth.file != 'read-error'))
                    if acceptable1 is not None:
                        acceptable = acceptable1 | {read_file_view(v) for _, _, v in writer2.all_versions}
                        tally.add(base_key + ('durability_violations',), lambda: f'{context()} || acceptable='
                                  f'{sorted(acceptable)} || {describe_recovery(second)}', int(second.file not in acceptable))


def run_layer1_units(tally, max_length, family='L1U'):
    """层 1 的另一种：最后一次发布写出的某个单元读不出（盘上内容坏了）。只坏一个单元，记录与根都好。"""
    for pattern in patterns(max_length):
        base, writer, _ = first_mount_stream(pattern)
        last_units = [write for write in writer.writes if write[0] == 'unit']
        if not pattern:
            continue
        last_txg = writer.all_versions[-1][0]
        targets = [(slot, content) for _, slot, content in last_units if content[1] == last_txg]
        for key, image, persisted in crash_states(base, writer.writes):
            for slot, content in targets:
                if image.units.get(slot) != content:
                    continue
                faulted = image.copy()
                faulted.units[slot] = ('bad', -1, slot)
                recoveries = all_recoveries(faulted)
                acceptable = acceptable_files(writer, persisted, 1)
                judge_against_truth(tally, family, recoveries,
                                    lambda: f'pattern={list(pattern)} crash={key} bad_unit={slot} {describe_image(faulted)}',
                                    acceptable)


ROLLBACK_ARMS = ('TRUTH6', 'DING', 'YI', 'BING_R1', 'T1_TX_STRICT', 'T1_TX_OPT', 'FP_LAST', 'IDXN', 'WALK', 'CUR_IMPL')
COUNTER_RULES = ('old', 'mount', 'ringmax')
ISOLATIONS = ('shadow', 'named', 'reach')


def covered_as_seen_by(arm, records, root):
    """回退起始计数器「R_old 覆盖到的计数器」在这条臂上怎么求（与它恢复时求链首同一个办法）。"""
    if arm == 'CUR_IMPL':
        return ring_covered(records, root)
    head = head_for(arm, records, root)
    if head is not None:
        return head - 1
    return ring_covered(records, root)


def rollback_stream(image, arm, counter_rule, isolation, r_old, record_count, baseline):
    """管理员回退（D23 已定项 14 显式例外）：取号 → 屏障 → 回退那次发布（record_count 条记录，写一个固定点单元，
    文件视图 = R_old）。影子账按窄读法：比 R_old 新的可读根引用、而 R_old 那棵账不引用的落点隔离。
    named：另隔离挂载时所选根水位之上读得出的记录点名的单元（第一轮攻方腿的收严）；reach：再隔离那些记录新根段里出生 txg 大于所选根的单元。"""
    records = image.records()
    chosen = image.chosen_root()
    ring_max = max((counter for _, counter in records), default=0)
    writer = Writer(image)
    instance = new_instance_number(image)
    writer.emit(('sb', instance))
    writer.emit(('barrier',))
    covered = covered_as_seen_by(arm, records, r_old)
    old_start = (covered if covered is not None else ring_max) + 1
    if arm in ('TRUTH6', 'DING'):
        old_start = r_old.covered + 1
    if counter_rule == 'old':
        start = old_start
    elif counter_rule == 'mount':
        end = baseline.prefix_end
        start = (end if end is not None else baseline.next_ctr - 1) + 1
    else:
        start = max(old_start - 1, ring_max) + 1
    readable = {key: root for key, root in image.roots.items() if key not in image.bad_roots}
    ring_versions = {key: root.version for key, root in readable.items()
                     if (root.txg, root.inst) <= (r_old.txg, r_old.inst)}
    live = {unit for version in ring_versions.values() for unit in version_units(version)}
    live |= set(version_units(r_old.version))
    abandoned = {unit for root in readable.values() if (root.txg, root.inst) > (r_old.txg, r_old.inst)
                 for unit in version_units(root.version)}
    isolated = {slot for slot, _ in abandoned - live}
    above = [record for record in records.values() if (record.inst, record.txg) > (chosen.inst, chosen.txg)]
    if isolation in ('named', 'reach'):
        isolated |= {slot for record in above for slot, _ in record.named}
    if isolation == 'reach':
        isolated |= {slot for record in above for slot, content in version_units(record.seg) if content[1] > chosen.txg}
    table = r_old.table + ((r_old.inst, r_old.txg, 0),)
    mount = Mount(instance, start, cj2_txg(image), 1, r_old.version, table, -1, None, ring_versions, frozenset(isolated))
    publish(writer, mount, ('m',) * record_count, 1, reuse_attr=r_old.version.attr)
    return writer, instance


def rollback_cause(image_after, baseline):
    records = image_after.records()
    for key in baseline.applied:
        if key not in records:
            return 'baseline_record_overwritten'
    for key in baseline.applied:
        if not verified(image_after, records[key]):
            return 'baseline_unit_reused'
    return 'head_or_other'


def run_rollback(tally, pattern_list, family='RB'):
    for pattern in pattern_list:
        base, writer1, _ = first_mount_stream(pattern)
        for key1, image1, _ in crash_states(base, writer1.writes):
            chosen = image1.chosen_root()
            candidates = sorted({root for root in image1.readable_roots()
                                 if (root.txg, root.inst) <= (chosen.txg, chosen.inst)}, key=lambda root: -root.txg)
            for r_old in candidates:
                for record_count in (1, 2):
                    for counter_rule in COUNTER_RULES:
                        for isolation in ISOLATIONS:
                            run_one_rollback(tally, family, pattern, key1, image1, r_old, record_count,
                                             counter_rule, isolation)


def run_one_rollback(tally, family, pattern, key1, image1, r_old, record_count, counter_rule, isolation):
    for arm in ROLLBACK_ARMS:
        rule = 'prefix'
        baseline = recover(image1, arm, rule)
        try:
            writer2, instance = rollback_stream(image1, arm, counter_rule, isolation, r_old, record_count, baseline)
        except AssertionError:
            tally.add((family, arm, counter_rule, isolation, 'allocation_exhausted'))
            continue
        for key2, image2, persisted2 in crash_states(image1, writer2.writes):
            after = recover(image2, arm, rule)
            committed = any(root.inst == instance for root in persisted2)
            base_key = (family, arm, counter_rule, isolation)
            context = lambda: (f'pattern={list(pattern)} crash1={key1} R_old=({r_old.inst},{r_old.txg}) '
                               f'records={record_count} crash2={key2} {describe_image(image2)} || baseline '
                               f'{describe_recovery(baseline)} || after {describe_recovery(after)}')
            if committed:
                holds = after.root[0] == instance and after.file == read_file_view(r_old.version)
                tally.add(base_key + ('committed_states',))
                tally.add(base_key + ('committed_wrong',), context, int(not holds))
                continue
            differs = after.outcome() != baseline.outcome()
            tally.add(base_key + ('uncommitted_states',))
            tally.add(base_key + ('uncommitted_differs_from_no_rollback',), context, int(differs))
            if differs:
                tally.add(base_key + ('cause', rollback_cause(image2, baseline)), context)
                tally.add(base_key + ('cause_read_error', str(after.file == 'read-error')), context)
            if arm != 'TRUTH6':
                truth_after = recover(image2, 'TRUTH6', rule)
                tally.add(base_key + ('outcome_differs_vs_truth_after',), lambda: f'{context()} || truth '
                          f'{describe_recovery(truth_after)}', int(after.outcome() != truth_after.outcome()))


SWITCH_READINGS = ('mount', 'root', 'mem', 'ringmax')


def switch_stream(pattern, fail_at, reading):
    """第一次挂载的最后一次数据发布写失败（fail_at = 'root' 或第 m 条记录）→ 挂载内实例切换（D23 已定项 14 索引行：
    取新实例代号、写行、重发在飞 checkpoint；号 ≤ W 的事务照旧、数据单元不重写，固定点单元按新实例重写）。
    新实例从哪个计数器接（注 3「前缀末 + 1」对切换没有定义）四种读法：mount = 这次挂载开头那次恢复的前缀末（mkfs 之后是 0）；
    root = 切换的所选根覆盖到的计数器；mem = 内存里下一个计数器；ringmax = max(挂载时环里最大, 内存里最大) + 1。"""
    base = make_filesystem()
    writer = Writer(base)
    writer.emit(('sb', 1))
    mount = Mount(1, 1, 1, 1, GENESIS, (), -1, None, {key: GENESIS for key in base.roots}, frozenset())
    publish(writer, mount, (), 0)
    publish(writer, mount, (), 0)
    for key in pattern[:-1]:
        publish(writer, mount, SHAPES[key], 1)
    base_version, base_covered, base_txg = mount.version, mount.covered_durable, mount.next_txg - 1
    first_tx = mount.next_tx
    shape = SHAPES[pattern[-1]]
    status, failed_version, failed_txg, data_order = publish(writer, mount, shape, 1, fail_at=fail_at)
    assert status == 'failed'
    instance = 2
    writer.emit(('sb', instance))
    switch_segment_write = len(writer.writes) - 1
    writer.emit(('barrier',))
    start = {'mount': 1, 'root': base_covered + 1, 'mem': mount.next_ctr, 'ringmax': mount.next_ctr}[reading]
    max_tx = first_tx + len(shape) - 1
    table = mount.table + ((1, base_txg, max_tx),)
    switched = Mount(instance, start, failed_txg + 1, first_tx, base_version, table, -1, None, mount.ring_versions,
                     frozenset(slot for slot, _ in data_order))
    publish(writer, switched, shape, 1, reuse_data=data_order, reuse_attr=failed_version.attr)
    return base, writer, switch_segment_write


def run_switch(tally, pattern_list, family='SW'):
    for pattern in pattern_list:
        record_total = sum(2 if kind == 'D' else 1 for kind in SHAPES[pattern[-1]])
        for fail_at in ['root'] + list(range(1, record_total + 1)):
            for reading in SWITCH_READINGS:
                base, writer, switch_write_index = switch_stream(pattern, fail_at, reading)
                sb_write = writer.writes[switch_write_index]
                segment_of_switch = next(index for index, segment in enumerate(segments(writer.writes))
                                         if any(write is sb_write for write in segment))
                for key, image, persisted in crash_states(base, writer.writes):
                    if key == ('nothing', 0) or key[0] < segment_of_switch:
                        continue
                    recoveries = all_recoveries(image)
                    acceptable = acceptable_files(writer, persisted, 1)
                    committed = any(root.inst == 2 for root in persisted)
                    family_key = f'{family}-{reading}'
                    judge_against_truth(tally, family_key, recoveries,
                                        lambda: f'pattern={list(pattern)} fail_at={fail_at} crash={key} '
                                                f'switch_committed={committed} {describe_image(image)}', acceptable)


WRAP_ARMS = ('TRUTH6', 'DING', 'YI', 'T1_TX_OPT', 'IDXN', 'WALK', 'CUR_IMPL')


def crash_loop_until_loss(arm, rule, acknowledged, max_loops):
    """一次发布（两条记录，计数器 4、5，txg 4）之后：acknowledged = 根 (1, 4) 落了盘、fsync 已返回，而它的根槽持续读不出（层 1，
    一个故障）；否则根没落盘。之后每次挂载：恢复 → 取号 → 暖机第一次空发布写完记录、根槽之前崩。
    返回读回的文件第一次不是 txg 4 那一版之前，完整做完了几轮；到 max_loops 都没丢返回 -1。"""
    base, writer, mount = first_mount_stream(('1',))
    publish(writer, mount, SHAPES['2'], 1)
    target = read_file_view(mount.version)
    image = base.copy()
    last_root_index = max(index for index, write in enumerate(writer.writes) if write[0] == 'root')
    for index, write in enumerate(writer.writes):
        if acknowledged or index != last_root_index:
            image.apply(write)
    if acknowledged:
        image.bad_roots = frozenset({root_slot(4)})
    for loop in range(max_loops + 1):
        recovery = recover(image, arm, rule)
        if recovery.file != target:
            return loop
        if loop == max_loops:
            return -1
        loop_writer = Writer(image)
        instance = new_instance_number(image)
        loop_writer.emit(('sb', instance))
        loop_writer.emit(('barrier',))
        ring_versions = {key: root.version for key, root in image.roots.items() if key not in image.bad_roots}
        loop_mount = Mount(instance, recovery.next_ctr, cj2_txg(image), 1, recovery.version, recovery.table, -1, None,
                           ring_versions, frozenset())
        publish(loop_writer, loop_mount, (), 1)
        image = image.copy()
        for write in loop_writer.writes:
            if write[0] != 'root':
                image.apply(write)
    return -1


def run_wrap(tally, ring_sizes=(12, 24, 48, 96)):
    global RING, N_LIMIT
    saved = (RING, N_LIMIT)
    try:
        for ring in ring_sizes:
            RING, N_LIMIT = ring, ring // F_SAFETY
            for acknowledged in (True, False):
                for arm in WRAP_ARMS:
                    for rule in RULES:
                        value = crash_loop_until_loss(arm, rule, acknowledged, 3 * ring)
                        tally.add(('WRAP', f'ring{ring}', 'acknowledged' if acknowledged else 'unacknowledged', arm, rule,
                                   'loops_before_file_lost'), None, value)
    finally:
        RING, N_LIMIT = saved


SWITCH_PATTERNS = [('1',), ('2',), ('S',), ('M',), ('D',), ('3',), ('1', '1'), ('2', '2'), ('1', '2'), ('2', '1')]
ROLLBACK_PATTERNS = patterns(1) + [('1', '1'), ('1', '2'), ('2', '1'), ('2', '2'), ('1', 'S'), ('1', 'M')]


def emit_tally(tally, lines, name):
    for key in sorted(tally.counts, key=str):
        lines.append(f'C245R2 name={name} key={"/".join(map(str, key))} value={tally.counts[key]}')
    for key in sorted(tally.examples, key=str):
        lines.append(f'C245R2 name={name}-example key={"/".join(map(str, key))} text={tally.examples[key]}')


def record_level_view(image):
    """丙在记录层读得到、而且知道怎么用的东西：所选根的 (实例, txg)，每条读得出的记录的 (实例, 计数器, txg, 事务号, 提交标记)。"""
    root = image.chosen_root()
    return ((root.inst, root.txg),
            tuple(sorted((r.inst, r.ctr, r.txg, r.tx, r.commit) for r in image.records().values())))


def run_pair(tally):
    """U4 那一对：X = 发布 txg 3 两条记录 (1,3)(1,4)、发布 txg 4 一条 (1,5)，根 (1,4) 落盘（fsync 已返回）后根槽 (1,4) 读不出、
    记录 (1,4) 两份读不出（第一轮 H3 那一格的多记录版）；Y = 发布 txg 3 一条 (1,3)、发布 txg 4 两条 (1,4)(1,5)，崩在只落了 (1,5)。"""
    base_x, writer_x, _ = first_mount_stream(('2', '1'))
    image_x = base_x.copy()
    for write in writer_x.writes:
        image_x.apply(write)
    image_x.bad_roots = frozenset({root_slot(4)})
    image_x.bad_ring = frozenset({record_slot(4)})
    base_y, writer_y, _ = first_mount_stream(('1', '2'))
    image_y = None
    for key, state, _ in crash_states(base_y, writer_y.writes):
        records = state.records()
        if (1, 5) in records and (1, 4) not in records and not any(root.txg == 4 for root in state.readable_roots()):
            image_y = state
            break
    assert image_y is not None
    same_view = record_level_view(image_x) == record_level_view(image_y)
    tally.add(('PAIR', 'record_level_view_identical'), None, int(same_view))
    tally.examples[('PAIR', 'X')] = describe_image(image_x)
    tally.examples[('PAIR', 'Y')] = describe_image(image_y)
    tally.examples[('PAIR', 'record_level_view')] = str(record_level_view(image_x))
    records_x, records_y = image_x.records(), image_y.records()
    differing = []
    for key in sorted(set(records_x) | set(records_y)):
        left, right = records_x.get(key), records_y.get(key)
        if left is None or right is None:
            differing.append(f'{key}:presence')
            continue
        for field in ('back', 'seg', 'pidx', 'pcount', 'inline_tail', 'named'):
            if getattr(left, field) != getattr(right, field):
                differing.append(f'{key}:{field}')
    root_x, root_y = image_x.chosen_root(), image_y.chosen_root()
    if root_x.version != root_y.version:
        differing.append('chosen_root:version')
    if root_x.covered != root_y.covered:
        differing.append('chosen_root:covered(丁才有)')
    tally.examples[('PAIR', 'fields_that_differ')] = ' '.join(differing)
    for arm in ARMS:
        for name, image in (('X', image_x), ('Y', image_y)):
            recovery = recover(image, arm, 'prefix')
            tally.examples[('PAIR', name, arm)] = describe_recovery(recovery)
    for name, writer, persisted_image in (('X', writer_x, image_x), ('Y', writer_y, image_y)):
        tally.examples[('PAIR', name, 'truth_file')] = str(recover(persisted_image, 'TRUTH6', 'prefix').file)


LONG_PATTERNS = [('1',) * length for length in range(5, 12)] + [('2',) * length for length in range(3, 7)] + \
    [('1', '2') * length for length in range(2, 5)] + [('2', 'S', 'M') * 2, ('D', '1', 'E') * 3]


FAMILIES = {
    'PAIR': run_pair,
    'LONG': lambda tally: run_layer0_patterns(tally, LONG_PATTERNS, 'LONG'),
    'L0': lambda tally: run_layer0(tally, 3),
    'L1': lambda tally: run_layer1(tally, 2),
    'L1U': lambda tally: run_layer1_units(tally, 2),
    'M2': lambda tally: run_second_mount(tally, 2),
    'RB': lambda tally: run_rollback(tally, ROLLBACK_PATTERNS),
    'SW': lambda tally: run_switch(tally, SWITCH_PATTERNS),
    'WRAP': run_wrap,
}


def selftest():
    """判别力自证：每条检查注入对应的毛病必须由绿转红（或由红转绿），不注入时是基线值。"""
    global MUTATION
    checks = [
        (None, 'L0', ('L0', 'TRUTH6', 'prefix', 'outcome_differs'), 'zero'),
        (None, 'L0', ('L0', 'DING', 'prefix', 'outcome_differs'), 'positive'),
        ('truth_ignores_sixth_rule', 'L0', ('L0', 'DING', 'prefix', 'outcome_differs'), 'zero'),
        (None, 'L0', ('L0', 'WALK', 'prefix', 'unverified_install'), 'zero'),
        (None, 'L0', ('L0', 'TRUTH6', 'prefix', 'unverified_install'), 'zero'),
        (None, 'L0', ('L0', 'DING', 'prefix', 'unverified_install'), 'positive'),
        (None, 'L0', ('L0', 'T1_OPT', 'prefix', 'head_differs_vs_DING'), 'positive'),
        ('walk_skips_check', 'L0', ('L0', 'WALK', 'prefix', 'unverified_install'), 'positive'),
        (None, 'L1', ('L1-root+ring', 'YI', 'prefix', 'head_differs_vs_DING'), 'zero'),
        ('yi_hint_off_by_one', 'L1', ('L1-root+ring', 'YI', 'prefix', 'head_differs_vs_DING'), 'positive'),
        (None, 'L1', ('L1-root+ring', 'BING_R1', 'prefix', 'durability_violations_truth_holds'), 'positive'),
        (None, 'L1', ('L1-root+ring', 'WALK', 'prefix', 'durability_violations_truth_holds'), 'zero'),
        (None, 'L1', ('L1-root+ring', 'T1_TX_STRICT', 'prefix', 'durability_violations_truth_holds'), 'positive'),
        (None, 'L1', ('L1-root', 'DING_DROPLAST', 'prefix', 'durability_violations_truth_holds'), 'positive'),
        (None, 'L1U', ('L1U', 'DING', 'prefix', 'read_error_truth_reads'), 'positive'),
        (None, 'L1U', ('L1U', 'WALK', 'prefix', 'read_error_truth_reads'), 'zero'),
        ('walk_skips_check', 'L1U', ('L1U', 'WALK', 'prefix', 'read_error_truth_reads'), 'positive'),
        (None, 'L1U', ('L1U', 'IDXN', 'prefix', 'read_error_truth_reads'), 'zero'),
        ('truth_ignores_sixth_rule', 'L1U', ('L1U', 'TRUTH6', 'prefix', 'unverified_install'), 'positive'),
        (None, 'RB', ('RB', 'TRUTH6', 'mount', 'named', 'uncommitted_differs_from_no_rollback'), 'zero'),
        (None, 'RB', ('RB', 'TRUTH6', 'old', 'named', 'cause', 'baseline_record_overwritten'), 'positive'),
        (None, 'RB', ('RB', 'TRUTH6', 'mount', 'shadow', 'cause', 'baseline_unit_reused'), 'positive'),
        (None, 'RB', ('RB', 'DING', 'mount', 'named', 'cause', 'head_or_other'), 'positive'),
        (None, 'RB', ('RB', 'DING', 'mount', 'reach', 'uncommitted_differs_from_no_rollback'), 'zero'),
    ]
    failures = 0
    for mutation, family, key, expectation in checks:
        MUTATION = mutation
        tally = Tally()
        if family == 'L0':
            run_layer0(tally, 1)
        elif family == 'L1':
            run_layer1(tally, 2)
        elif family == 'RB':
            run_rollback(tally, [('2',), ('1', '2')])
        else:
            run_layer1_units(tally, 1)
        value = tally.counts.get(key, 0)
        holds = value == 0 if expectation == 'zero' else value > 0
        failures += int(not holds)
        print(f'C245R2 name=selftest mutation={mutation} key={"/".join(key)} expect={expectation} value={value} ok={holds}')
    MUTATION = 'ringmax_is_prefix'
    tally = Tally()
    run_wrap(tally, (12,))
    value = tally.counts.get(('WRAP', 'ring12', 'acknowledged', 'DING', 'ringmax', 'loops_before_file_lost'))
    holds = value == -1
    failures += int(not holds)
    print(f'C245R2 name=selftest mutation=ringmax_is_prefix key=WRAP/ring12/acknowledged/DING/ringmax expect=-1 '
          f'value={value} ok={holds}')
    MUTATION = None
    print(f'C245R2 name=selftest failures={failures}')
    return failures


def main(arguments):
    global MUTATION
    selected = arguments[0] if arguments else 'all'
    if len(arguments) == 3 and arguments[1] == '--mutation':
        MUTATION = arguments[2]
    if selected == '--selftest':
        sys.exit(1 if selftest() else 0)
    lines = [f'C245R2 name=config ring={RING} f={F_SAFETY} n_limit={N_LIMIT} regions={REGIONS} '
             f'slots_per_region={SLOTS_PER_REGION} unit_slots={UNIT_SLOTS} mutation={MUTATION}']
    for name, runner in FAMILIES.items():
        if selected not in ('all', name):
            continue
        tally = Tally()
        runner(tally)
        emit_tally(tally, lines, name)
    lines.append(f'C245R2 name=end emitted={len(lines) + 1}')
    print('\n'.join(lines))


if __name__ == '__main__':
    main(sys.argv[1:])
