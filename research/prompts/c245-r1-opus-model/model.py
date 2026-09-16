#!/usr/bin/env python3
"""C245（超级块槽的写频率，两条条款说反话） 第一轮攻方腿（Opus）的计数模型。

问：超级块里的 tail 承不承重。
实现（每种再乘两条「下一次写接的计数器」规则：prefix = 前缀末 + 1（D23 已定项 14 注 3 字面）；
ringmax = max(前缀末, 环里自证通过的最大计数器) + 1（攻方腿提的收严，被攻过零轮））：
  ORACLE              条款真值：链从所选根覆盖的最后一条记录之后接，那一条的计数器取模型真值
                      （盘上今天没有这个量；它等价于「根记录带覆盖到的计数器」那条臂，E78 出路 B 的原形）
  JIA_LITERAL         甲字面：全环扫描，tail 只决定扫描次序；链首从环里读得出的同实例、txg ≤ 根的最大计数器推
  JIA_TAIL_FALLBACK   甲读 tail：链首 = max(环里推出的, 超级块 tail)；回退的起始计数器也这么取
  JIA_TAIL_CHAIN_ONLY 甲读 tail：只在恢复接链时取 max(环里推出的, tail)；回退的起始计数器照环里推
  YI_INLINE           乙：记录头带 inline_tail；所选根 T 的实例里 checkpoint_txg = T + 1 的记录的 inline_tail 就是链首前一条
  BING_STRICT         丙：不存 tail，链首从环里推（与 JIA_LITERAL 同算法，不读超级块）
  BING_FIRST          丙的另一种写法：链首 = 水位之上同实例的最小计数器
对照：TRUST_TAIL（从 tail 起往前走、失配即中止，违反 D23 已定项 3「恢复不许先信 tail」，证明模型分得出 tail 的取值）、
CUR_IMPL（照抄 crates/singlefs-core/src/recovery.rs 的 replay_journal：水位之上按 (实例, 计数器) 排序、从第一条起连续）。
族：first（一次挂载崩在任一状态）、second（崩后各实现自己恢复、再挂一次、再崩）、rollback（管理员回退那次发布崩在任一状态）、
rollback-isolated（同上，另把挂载时水位之上记录点名的单元隔离到回退发布持久为止）、long（一次挂载发布很多次，出现块重用）、
faults（层 1：一条记录读不出 / 一个根槽读不出 / 各一）。

只用 std。确定性模型：同一输入跑 N 遍结果必然相同，轮数不是证据。
层 0 = 屏障与 FUA 切段、段内任意整写子集；层 1 = 在层 0 状态上再加介质故障（记录两份都读不出、根槽读不出）。
"""
import sys
from dataclasses import dataclass, field

RING = 12                       # journal 环槽数（模型取小，让回绕进得了枚举）；记录 n 落 (n - 1) mod RING
F_SAFETY = 3
N_LIMIT = RING // F_SAFETY      # 在飞记录数上限 = 一次恢复重放的前缀最多 N 条（D23 已定项 18）
REGIONS = 3                     # 根环区域数 R
SLOTS_PER_REGION = 2            # 每区槽数 S（取小，让根槽被轮转覆写）
UNIT_SLOTS = 16                 # 单元落点数（取小，让块重用发生）；可再分配的界 = 环里最旧有效根（D16 已定项 1，F 不动）
DISKS = 2


@dataclass(frozen=True)
class Record:
    instance: int
    counter: int
    checkpoint_txg: int
    transaction: int            # 0 = 空发布
    commit: bool
    named: tuple                # ((unit_slot, content), ...)：施加前逐项验证
    new_root_version: tuple     # (file_version, unit_slots)：新根段，施加 = 换成它
    inline_tail: int            # 乙臂才读：写这条记录时本实例最新持久根覆盖到的计数器


@dataclass(frozen=True)
class Root:
    instance: int
    txg: int
    version: tuple              # (file_version, unit_slots)；file_version 0 = 没有文件
    instance_table: tuple       # ((instance, T, W), ...)
    covered_truth: int          # 模型真值：这个根覆盖的最后一条记录的计数器；只有 ORACLE 读，盘上没有


@dataclass(frozen=True)
class Superblock:
    generation: int
    instance: int
    tail: int


def record_slot(counter):
    return (counter - 1) % RING


def root_slot(txg):
    return (txg % REGIONS, (txg // REGIONS) % SLOTS_PER_REGION)


class Image:
    """崩溃后的盘面。两块盘的 journal 与单元按「任一份在即在」折成一份；超级块逐盘两槽。"""

    def __init__(self):
        self.ring = {}
        self.roots = {}
        self.units = {}
        self.superblocks = {}
        self.unreadable_ring_slots = frozenset()
        self.unreadable_root_slots = frozenset()

    def copy(self):
        duplicate = Image()
        duplicate.ring = dict(self.ring)
        duplicate.roots = dict(self.roots)
        duplicate.units = dict(self.units)
        duplicate.superblocks = dict(self.superblocks)
        duplicate.unreadable_ring_slots = self.unreadable_ring_slots
        duplicate.unreadable_root_slots = self.unreadable_root_slots
        return duplicate

    def apply(self, write):
        kind = write[0]
        if kind == 'unit':
            self.units[write[1]] = write[2]
        elif kind == 'record':
            self.ring[record_slot(write[1].counter)] = write[1]
        elif kind == 'root':
            self.roots[root_slot(write[1].txg)] = write[1]
        elif kind == 'superblock':
            self.superblocks[(write[1], write[2])] = write[3]
        elif kind == 'barrier':
            pass
        else:
            raise AssertionError(f'未知写：{kind}')

    def readable_records(self):
        return {(record.instance, record.counter): record
                for slot, record in self.ring.items() if slot not in self.unreadable_ring_slots}

    def readable_roots(self):
        return [root for key, root in self.roots.items() if key not in self.unreadable_root_slots]


def segments(writes):
    """D13 已定项 4 的切段：屏障与 FUA 写切段。根槽写是 FUA，自成一段。"""
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
        else:
            current.append(write)
    if current:
        all_segments.append(current)
    return all_segments


def crash_states(base_image, writes):
    """层 0：空状态 + 每段的每个非空整写子集（前面各段全落）。闭式 1 + Σ(2^n − 1)。"""
    image = base_image.copy()
    yield ('nothing', 0), image.copy()
    for segment_index, segment in enumerate(segments(writes)):
        for mask in range(1, 1 << len(segment)):
            state = image.copy()
            for write_index, write in enumerate(segment):
                if mask >> write_index & 1:
                    state.apply(write)
            yield (segment_index, mask), state
        for write in segment:
            image.apply(write)


def crash_state_closed_form(writes):
    return 1 + sum((1 << len(segment)) - 1 for segment in segments(writes))


class Writer:
    """写者视图：每条写立刻施加到一份完整镜像上（算世代号、分配落点都读它），同时记进录制流。"""

    def __init__(self, image):
        self.image = image
        self.writes = []
        # 版本号在整个盘面上不许撞：单元里的、记录点名的、根里的都算（崩掉没落盘的版本号可以复用，它不在盘上）。
        contents = list(image.units.values())
        contents += [content for record in image.ring.values() for _, content in record.named]
        file_versions = [content[0] for content in contents if isinstance(content[0], int)]
        file_versions += [root.version[0] for root in image.roots.values()]
        file_versions += [record.new_root_version[0] for record in image.ring.values()]
        self.next_file_version = max(file_versions + [0]) + 1
        metadata_versions = [content[1] for content in contents if content[0] == 'metadata']
        self.next_metadata_version = max(metadata_versions + [0]) + 1

    def emit(self, *write):
        self.writes.append(write)
        self.image.apply(write)


@dataclass
class MountedState:
    instance: int
    next_counter: int
    next_txg: int
    version: tuple
    instance_table: tuple
    covered_last: int           # 本实例最新持久根覆盖到的计数器（写进下一条记录的 inline_tail）
    ring_root_versions: dict    # 根环各槽里那个根的版本：环里有效根引用的单元不复用（D16 已定项 1 的可再分配界）
    isolated_slots: frozenset   # 回退影子账隔离的落点
    transaction: int = 0
    superblock_tail_writes: bool = True   # 甲写超级块 tail；乙、丙发布末尾不写超级块


def allocate(state, count):
    used = set(state.version[1]) | set(state.isolated_slots)
    for version in state.ring_root_versions.values():
        used |= set(version[1])
    free = [slot for slot in range(UNIT_SLOTS) if slot not in used]
    assert len(free) >= count, f'落点不够：used={sorted(used)}'
    return free[:count]


def rotate_superblocks(writer, instance, tail):
    """D22 已定项 16：逐盘取这块盘上自证过的最大世代号 + 1，落槽 = 世代号 mod 2。"""
    for disk in range(DISKS):
        generations = [block.generation for (owner, _), block in writer.image.superblocks.items() if owner == disk]
        generation = max(generations, default=0) + 1
        writer.emit('superblock', disk, generation % 2, Superblock(generation, instance, tail))


def publish(writer, state, record_count, payload):
    """一次发布。payload: 'empty'（暖机空发布）/ 'data'（新文件版本，每条记录点名一个单元）/
    'metadata'（回退那次：写单元、点名，但文件版本不变）。持久顺序 D16 已定项 7。"""
    base_version = state.version
    if payload == 'empty':
        writer.emit('barrier')
        writer.emit('record', Record(state.instance, state.next_counter, state.next_txg, 0, True, (),
                                     base_version, state.covered_last))
        state.next_counter += 1
        new_version = base_version
    else:
        slots = allocate(state, record_count)
        if payload == 'data':
            file_version = writer.next_file_version
            writer.next_file_version += 1
            contents = [(file_version, index) for index in range(record_count)]
            new_version = (file_version, tuple(slots))
        else:
            contents = [('metadata', writer.next_metadata_version, index) for index in range(record_count)]
            writer.next_metadata_version += 1
            new_version = base_version
        for slot, content in zip(slots, contents):
            writer.emit('unit', slot, content)
        writer.emit('barrier')
        state.transaction += 1
        for index, (slot, content) in enumerate(zip(slots, contents)):
            writer.emit('record', Record(state.instance, state.next_counter, state.next_txg, state.transaction,
                                         index == record_count - 1, ((slot, content),), new_version,
                                         state.covered_last))
            state.next_counter += 1
        if payload == 'metadata':
            state.isolated_slots = frozenset(set(state.isolated_slots) | set(slots))
    writer.emit('barrier')
    last_counter = state.next_counter - 1
    writer.emit('root', Root(state.instance, state.next_txg, new_version, state.instance_table, last_counter))
    if state.superblock_tail_writes:
        rotate_superblocks(writer, state.instance, last_counter)
    state.ring_root_versions = dict(state.ring_root_versions)
    state.ring_root_versions[root_slot(state.next_txg)] = new_version
    state.version = new_version
    state.next_txg += 1
    state.covered_last = last_counter


def make_filesystem():
    """mkfs 不进枚举（同 E142 的 G19）：第 0 代根种进全部区域，两盘两槽世代号 1、实例代号 0、tail 0。"""
    image = Image()
    genesis = Root(0, 0, (0, ()), (), 0)
    for region in range(REGIONS):
        image.roots[(region, 0)] = genesis
    for disk in range(DISKS):
        for slot in range(2):
            image.superblocks[(disk, slot)] = Superblock(1, 0, 0)
    return image


def chosen_superblock(image):
    best = None
    for key in sorted(image.superblocks):
        block = image.superblocks[key]
        if best is None or block.generation > best.generation:
            best = block
    return best


def acquire_instance(writer, carried_tail):
    """取号：max(超级块, 根环) + 1，先写进每一份超级块（D23 已定项 16）。tail 照抄择到的超级块（字节表 a1 的写法）。"""
    image = writer.image
    highest = max([block.instance for block in image.superblocks.values()]
                  + [root.instance for root in image.readable_roots()])
    rotate_superblocks(writer, highest + 1, carried_tail)
    return highest + 1


def first_txg_of_new_instance(image):
    """C143 定案 CJ2：max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1。"""
    return max([root.txg for root in image.readable_roots()]
               + [record.checkpoint_txg for record in image.readable_records().values()]) + 1


ARMS = ('ORACLE', 'JIA_LITERAL', 'JIA_TAIL_FALLBACK', 'JIA_TAIL_CHAIN_ONLY', 'YI_INLINE', 'BING_STRICT', 'BING_FIRST')
CONTROLS = ('CUR_IMPL', 'TRUST_TAIL')
NEXT_RULES = ('prefix', 'ringmax')


@dataclass(frozen=True)
class Recovery:
    arm: str
    next_rule: str
    root: tuple                 # 所选根 (实例代号, txg)
    applied: tuple              # 施加的记录 ((实例代号, 计数器), ...)
    file: object                # 读回：文件版本号 / 'no-file' / 'read-error' / 'recovery-failed'
    next_counter: int           # 下一次写接的计数器
    version: tuple
    instance_table: tuple
    ring_root_versions: dict
    prefix_end: object
    carried_tail: int

    def outcome(self):
        return (self.root, self.applied, self.file)

    def full(self):
        return (self.root, self.applied, self.file, self.next_counter)


def ring_covered(records, root):
    """丙能拿到的「所选根覆盖的最后一条」：环里读得出的、同实例、checkpoint_txg ≤ 根的最大计数器。"""
    covered = [record.counter for record in records.values()
               if record.instance == root.instance and record.checkpoint_txg <= root.txg]
    if covered:
        return max(covered)
    return 0 if root.instance == 0 else None


def inline_hint(records, root):
    """乙：同实例、checkpoint_txg = 根 + 1 的记录写下时，本实例最新持久根就是这个根 ⇒ 它的 inline_tail 绑定这个根。"""
    hints = {record.inline_tail for record in records.values()
             if record.instance == root.instance and record.checkpoint_txg == root.txg + 1}
    assert len(hints) <= 1, f'同一发布的记录 inline_tail 不一：{hints}'
    return next(iter(hints)) if hints else None


def last_covered(arm, records, root, tail):
    if arm == 'ORACLE':
        return root.covered_truth
    ring_value = ring_covered(records, root)
    if arm == 'JIA_LITERAL' and MUTATION == 'literal_trusts_tail_as_covered':
        return tail
    if arm in ('JIA_LITERAL', 'BING_STRICT'):
        return ring_value
    if arm in ('JIA_TAIL_FALLBACK', 'JIA_TAIL_CHAIN_ONLY'):
        return tail if ring_value is None else max(ring_value, tail)
    if arm == 'YI_INLINE':
        hint = inline_hint(records, root)
        return ring_value if hint is None else hint
    raise AssertionError(arm)


def verified(image, record):
    return all(image.units.get(slot) == content for slot, content in record.named)


def walk_chain(image, records, root, start_counter):
    """六条口径里模型碰得到的五条：同实例严格连续（跨实例即停）、水位之上、提交标记齐全才施加、
    施加前验证点名单元、回退行 W 截断；前缀最多 N_LIMIT 条。一次发布 = 一个事务，整体施加由提交标记给出。"""
    applied, pending, version = [], [], root.version
    rollback_caps = [row[2] for row in root.instance_table if row[0] == root.instance]
    counter, steps = start_counter, 0
    while steps < N_LIMIT:
        record = records.get((root.instance, counter))
        if record is None or record.checkpoint_txg <= root.txg:
            break
        if rollback_caps and record.transaction != 0 and record.transaction > min(rollback_caps):
            break
        if not verified(image, record):
            break
        pending.append(record)
        steps += 1
        if record.commit:
            applied.extend(pending)
            version = record.new_root_version
            pending = []
        counter += 1
    return applied, version


def read_file(image, version):
    file_version, slots = version
    if file_version == 0:
        return 'no-file'
    intact = all(image.units.get(slot) == (file_version, index) for index, slot in enumerate(slots))
    return file_version if intact else 'read-error'


def scan_records(image, start_slot):
    """全环扫描；甲的「扫描起点」只决定扫的次序。重复的 (实例代号, 计数器) 先到先得。"""
    records = {}
    for offset in range(RING):
        slot = (start_slot + offset) % RING
        record = image.ring.get(slot)
        if record is not None and slot not in image.unreadable_ring_slots:
            records.setdefault((record.instance, record.counter), record)
    return records


def ring_versions_after_recovery(image, highest_txg=None):
    """恢复后的分配账：根环里读得出的根引用的单元都不复用；highest_txg 给出时只算不晚于它的根（回退从 R_old 那棵账载入）。"""
    return {key: root.version for key, root in image.roots.items()
            if key not in image.unreadable_root_slots and (highest_txg is None or root.txg <= highest_txg)}


def recover(image, arm, next_rule, tail_override=None):
    superblock = chosen_superblock(image)
    tail = superblock.tail if tail_override is None else tail_override
    reads_tail_for_scan = arm in TAIL_READING_ARMS
    records = scan_records(image, record_slot(tail + 1) if reads_tail_for_scan else 0)
    roots = image.readable_roots()
    root = max(roots, key=lambda candidate: (candidate.txg, candidate.instance))
    ring_max = max([record.counter for record in records.values()], default=0)
    if arm == 'CUR_IMPL':
        above = sorted((record for record in records.values()
                        if (record.instance, record.checkpoint_txg) > (root.instance, root.txg)),
                       key=lambda record: (record.instance, record.counter))
        applied, version, expected = [], root.version, None
        for record in above[:N_LIMIT]:
            if expected is not None and (record.instance, record.counter) != expected:
                break
            expected = (record.instance, record.counter + 1)
            if not record.commit or not verified(image, record):
                break
            applied.append(record)
            version = record.new_root_version
        prefix_end = applied[-1].counter if applied else ring_covered(records, root)
    elif arm == 'TRUST_TAIL':
        applied, version, counter, instance = [], root.version, tail + 1, None
        failed = False
        while len(applied) < RING:
            record = image.ring.get(record_slot(counter))
            if record is None or record.counter != counter or (instance is not None and record.instance != instance):
                break
            if not verified(image, record):
                failed = True        # E78 中止臂：失配即中止
                break
            instance = record.instance
            applied.append(record)
            version = record.new_root_version
            counter += 1
        prefix_end = applied[-1].counter if applied else tail
        if failed:
            return Recovery(arm, next_rule, (root.instance, root.txg), tuple((r.instance, r.counter) for r in applied),
                            'recovery-failed', 0, version, root.instance_table, {}, prefix_end, superblock.tail)
    else:
        covered = None if arm == 'BING_FIRST' else last_covered(arm, records, root, tail)
        if arm == 'BING_FIRST':
            above = [record.counter for record in records.values()
                     if record.instance == root.instance and record.checkpoint_txg > root.txg]
            start = min(above) if above else None
            applied, version = walk_chain(image, records, root, start) if start is not None else ([], root.version)
            covered = ring_covered(records, root)
        elif covered is None:
            applied, version = [], root.version
        else:
            off_by_one = MUTATION == 'strict_start_at_covered' and arm == 'BING_STRICT'
            applied, version = walk_chain(image, records, root, covered + (0 if off_by_one else 1))
        prefix_end = applied[-1].counter if applied else covered
    if next_rule == 'prefix':
        next_counter = (prefix_end if prefix_end is not None else ring_max) + 1
    else:
        next_counter = max(prefix_end or 0, ring_max) + 1
    return Recovery(arm, next_rule, (root.instance, root.txg), tuple((r.instance, r.counter) for r in applied),
                    read_file(image, version), next_counter, version, root.instance_table,
                    ring_versions_after_recovery(image), prefix_end, superblock.tail)


def mount_normal(image, recovery, data_record_counts, superblock_tail_writes=True):
    """普通可写挂载：取号 → 暖机两次空发布（D16 已定项 8）→ 若干次数据发布。"""
    writer = Writer(image.copy())
    instance = acquire_instance(writer, recovery.carried_tail)
    state = MountedState(instance, recovery.next_counter, first_txg_of_new_instance(image), recovery.version,
                         recovery.instance_table, recovery.prefix_end if recovery.prefix_end is not None else 0,
                         recovery.ring_root_versions, frozenset(), 0, superblock_tail_writes)
    publish(writer, state, 0, 'empty')
    publish(writer, state, 0, 'empty')
    for count in data_record_counts:
        publish(writer, state, count, 'data')
    return writer.writes, state


def rollback_next_counter(arm, next_rule, image, rollback_root, tail):
    records = image.readable_records()
    ring_max = max([record.counter for record in records.values()], default=0)
    if arm in ('CUR_IMPL', 'BING_FIRST', 'JIA_TAIL_CHAIN_ONLY'):
        covered = ring_covered(records, rollback_root)
    elif arm == 'TRUST_TAIL':
        covered = tail
    else:
        covered = last_covered(arm, records, rollback_root, tail)
    if next_rule == 'prefix' or MUTATION == 'ringmax_ignores_ring':
        return (covered if covered is not None else ring_max) + 1, covered
    return max(covered or 0, ring_max) + 1, covered


def mount_rollback(image, recovery, rollback_root, rollback_record_count, superblock_tail_writes=True):
    """管理员回退（D23 已定项 14 的显式例外）：取号 → 回退那次发布（写实例表单元、带回退行、文件版本 = R_old）。
    影子账：环里可读、比 R_old 新的根（被抛弃时间线）引用的落点隔离，R_old 自己的账引用的不算。"""
    writer = Writer(image.copy())
    instance = acquire_instance(writer, recovery.carried_tail)
    readable = image.readable_roots()
    abandoned = [root for root in readable if (root.txg, root.instance) > (rollback_root.txg, rollback_root.instance)]
    ring_versions = ring_versions_after_recovery(image, rollback_root.txg)
    live = set(rollback_root.version[1])
    for version in ring_versions.values():
        live |= set(version[1])
    isolated = {slot for root in abandoned for slot in root.version[1]} - live
    if ROLLBACK_ISOLATES_ABOVE_WATER_UNITS:
        # 攻方腿自己的收严（只在本模型上量过、被攻过零轮）：挂载时所选根之上那些记录点名的单元也隔离到回退那次发布持久为止。
        latest = max(readable, key=lambda root: (root.txg, root.instance))
        isolated |= {slot for record in image.readable_records().values()
                     if (record.instance, record.checkpoint_txg) > (latest.instance, latest.txg)
                     for slot, _ in record.named}
    isolated = frozenset(isolated)
    next_counter, covered = rollback_next_counter(recovery.arm, recovery.next_rule, image, rollback_root,
                                                  recovery.carried_tail)
    table = rollback_root.instance_table + ((rollback_root.instance, rollback_root.txg, 0),)
    state = MountedState(instance, next_counter, first_txg_of_new_instance(image), rollback_root.version, table,
                         covered if covered is not None else 0, ring_versions, isolated, 0, superblock_tail_writes)
    publish(writer, state, rollback_record_count, 'metadata')
    return writer.writes, state


# ---------------------------------------------------------------- 驱动：录制流与判定

def first_mount_stream(record_counts, superblock_tail_writes=True):
    """mkfs 之后：第一次可写挂载取号 → 暖机两次 → 按 record_counts 连做数据发布（每次发布一个事务，1 或 2 条记录）。"""
    base = make_filesystem()
    writer = Writer(base.copy())
    rotate_superblocks(writer, 1, 0)
    state = MountedState(1, 1, 1, (0, ()), (), 0, {key: (0, ()) for key in base.roots}, frozenset(), 0,
                         superblock_tail_writes)
    publish(writer, state, 0, 'empty')
    publish(writer, state, 0, 'empty')
    for count in record_counts:
        publish(writer, state, count, 'data')
    return base, writer.writes


def patterns(max_length):
    result = [()]
    frontier = [()]
    for _ in range(max_length):
        frontier = [pattern + (count,) for pattern in frontier for count in (1, 2)]
        result.extend(frontier)
    return result


def crash_states_with_persisted_roots(base_image, writes):
    """与 crash_states 同一个枚举，另给出这个状态里落过盘的根（轮转覆写之前 fsync 就已返回的也算）。"""
    persisted_before = []
    image = base_image.copy()
    yield ('nothing', 0), image.copy(), ()
    for segment_index, segment in enumerate(segments(writes)):
        for mask in range(1, 1 << len(segment)):
            state = image.copy()
            chosen = [write for write_index, write in enumerate(segment) if mask >> write_index & 1]
            for write in chosen:
                state.apply(write)
            roots = tuple(persisted_before + [write[1] for write in chosen if write[0] == 'root'])
            yield (segment_index, mask), state, roots
        for write in segment:
            image.apply(write)
            if write[0] == 'root':
                persisted_before.append(write[1])


def acknowledged_file_version(persisted_roots, instance):
    """fsync 返回过的最新文件版本：本实例的根覆盖两块盘（暖机两次都持久，D16 已定项 8）之后持久的根里 txg 最大的那个。"""
    own = sorted((root for root in persisted_roots if root.instance == instance), key=lambda root: root.txg)
    if len(own) < 2:
        return None
    return own[-1].version[0]


class Tally:
    def __init__(self):
        self.counts = {}
        self.examples = {}

    def add(self, key, example=None, amount=1):
        self.counts[key] = self.counts.get(key, 0) + amount
        if example is not None and amount and key not in self.examples:
            self.examples[key] = example


def describe_image(image):
    ring = ' '.join(f'[{slot}]({record.instance},{record.counter},t{record.checkpoint_txg},x{record.transaction}'
                    f'{"c" if record.commit else ""},it{record.inline_tail})'
                    for slot, record in sorted(image.ring.items()))
    roots = ' '.join(f'{key}:({root.instance},{root.txg},v{root.version[0]})' for key, root in sorted(image.roots.items()))
    superblocks = ' '.join(f'd{disk}s{slot}:g{block.generation}/i{block.instance}/t{block.tail}'
                           for (disk, slot), block in sorted(image.superblocks.items()))
    units = ' '.join(f'{slot}:{content}' for slot, content in sorted(image.units.items()))
    bad = f' unreadable_ring={sorted(image.unreadable_ring_slots)} unreadable_roots={sorted(image.unreadable_root_slots)}' \
        if image.unreadable_ring_slots or image.unreadable_root_slots else ''
    return f'ring {{{ring}}} roots {{{roots}}} sb {{{superblocks}}} units {{{units}}}{bad}'


def describe_recovery(recovery):
    return (f'{recovery.arm}/{recovery.next_rule}: root={recovery.root} applied={list(recovery.applied)} '
            f'file={recovery.file} next={recovery.next_counter}')


def all_recoveries(image, tail_override=None):
    return {(arm, rule): recover(image, arm, rule, tail_override)
            for arm in ARMS + CONTROLS for rule in NEXT_RULES}


def compare_to_oracle(tally, family, recoveries, example_prefix):
    for (arm, rule), recovery in recoveries.items():
        oracle = recoveries[('ORACLE', rule)]
        if arm == 'ORACLE':
            continue
        example = f'{example_prefix} || {describe_recovery(recovery)} || {describe_recovery(oracle)}'
        tally.add((family, arm, rule, 'states'))
        tally.add((family, arm, rule, 'outcome_differs'), example, int(recovery.outcome() != oracle.outcome()))
        tally.add((family, arm, rule, 'next_differs'), example, int(recovery.next_counter != oracle.next_counter))
        tally.add((family, arm, rule, 'file_differs'), example, int(recovery.file != oracle.file))


TAIL_READING_ARMS = ('JIA_LITERAL', 'JIA_TAIL_FALLBACK', 'JIA_TAIL_CHAIN_ONLY', 'TRUST_TAIL')
TAIL_DISTANCES = (1, 2, 3, RING // 2, RING - 1)   # 「任意 k」取小、半环、差一圈


def tail_perturbations(true_tail):
    if true_tail != 0:
        yield 'zero', 0
    for distance in TAIL_DISTANCES:
        if true_tail - distance >= 0:
            yield 'stale', true_tail - distance
        yield 'ahead', true_tail + distance


def reuse_visible(image):
    """块重用叠上来了：环里某条读得出的记录点名的单元已被别的内容盖掉。"""
    return any(not verified(image, record) for record in image.readable_records().values())


def judge_tail(tally, family, image, recoveries, prefix):
    """T2：甲下把超级块 tail 改成 0 / 小 k / 大 k，恢复结果变不变；叠块重用的子集另记一份。"""
    true_tail = chosen_superblock(image).tail
    reuse = reuse_visible(image)
    for arm in TAIL_READING_ARMS:
        for rule in NEXT_RULES:
            base = recoveries[(arm, rule)]
            for kind, value in tail_perturbations(true_tail):
                perturbed = recover(image, arm, rule, value)
                example = f'{prefix} || tail {true_tail}->{value} || {describe_recovery(base)} || {describe_recovery(perturbed)}'
                for scope in (('T2',) + (('T2-reuse',) if reuse else ())):
                    tally.add((family, scope, arm, rule, kind, 'perturbations'))
                    tally.add((family, scope, arm, rule, kind, 'outcome_changed'), example,
                              int(perturbed.outcome() != base.outcome()))
                    tally.add((family, scope, arm, rule, kind, 'next_changed'), example,
                              int(perturbed.next_counter != base.next_counter))
                    tally.add((family, scope, arm, rule, kind, 'file_changed'), example,
                              int(perturbed.file != base.file))


def judge_durability(tally, family, recoveries, persisted_roots, instance, prefix):
    acknowledged = acknowledged_file_version(persisted_roots, instance)
    for (arm, rule), recovery in recoveries.items():
        tally.add((family, 'durability', arm, rule, 'states'))
        if acknowledged is None:
            continue
        if acknowledged == 0:
            holds = recovery.file not in ('read-error', 'recovery-failed')
        else:
            holds = isinstance(recovery.file, int) and recovery.file >= acknowledged
        tally.add((family, 'durability', arm, rule, 'violations'),
                  f'{prefix} || acknowledged={acknowledged} || {describe_recovery(recovery)}', int(not holds))


def run_first_mount(tally, max_length, with_tail_judgement=True, pattern_list=None, family='L0-first-mount'):
    for pattern in (patterns(max_length) if pattern_list is None else pattern_list):
        base, writes = first_mount_stream(pattern)
        tally.add((family, 'closed_form_ok'), None, int(
            sum(1 for _ in crash_states(base, writes)) == crash_state_closed_form(writes)))
        for key, image, roots in crash_states_with_persisted_roots(base, writes):
            recoveries = all_recoveries(image)
            prefix = f'pattern={list(pattern)} crash={key} image: {describe_image(image)}'
            compare_to_oracle(tally, family, recoveries, prefix)
            judge_durability(tally, family, recoveries, roots, 1, prefix)
            if with_tail_judgement:
                judge_tail(tally, family, image, recoveries, prefix)


def run_second_mount(tally, max_length, data_after=((1,), (2,))):
    """两次挂载：第一次崩在任一层 0 状态 → 各臂自己恢复 → 第二次挂载（取号、暖机、数据发布）→ 再崩 → 各臂自己恢复。"""
    family = 'L0-second-mount'
    for pattern in patterns(max_length):
        base, writes = first_mount_stream(pattern)
        for key1, image1, _ in crash_states_with_persisted_roots(base, writes):
            for counts in data_after:
                per_arm = {}
                for arm in ARMS + CONTROLS:
                    for rule in NEXT_RULES:
                        first = recover(image1, arm, rule)
                        if first.file == 'recovery-failed':
                            continue
                        writes2, state2 = mount_normal(image1, first, counts)
                        for key2, image2, roots2 in crash_states_with_persisted_roots(image1, writes2):
                            second = recover(image2, arm, rule)
                            per_arm.setdefault(key2, {})[(arm, rule)] = second
                            prefix = (f'pattern={list(pattern)} crash1={key1} first={describe_recovery(first)} '
                                      f'second-mount data={list(counts)} crash2={key2} image: {describe_image(image2)}')
                            judge_durability(tally, family, {(arm, rule): second}, roots2, state2.instance, prefix)
                for key2, recoveries in per_arm.items():
                    if any(('ORACLE', rule) not in recoveries for rule in NEXT_RULES):
                        continue
                    compare_to_oracle(tally, family, recoveries,
                                      f'pattern={list(pattern)} crash1={key1} data={list(counts)} crash2={key2}')


def run_rollback(tally, max_length, rollback_record_counts=(1, 2)):
    """管理员回退：第一次挂载崩在任一层 0 状态 → 管理员从比所选根旧的可读根里选 R_old → 取号 + 回退那次发布 → 崩在任一层 0 状态。
    判两件事：回退根没落盘时结果与「没发起回退」相同（D23 已定项 14 显式例外那一句）；落盘了则挂上回退根、读回 R_old 的文件。"""
    family = 'L0-rollback'
    for pattern in patterns(max_length):
        base, writes = first_mount_stream(pattern)
        for key1, image1, _ in crash_states_with_persisted_roots(base, writes):
            chosen = max(image1.readable_roots(), key=lambda root: (root.txg, root.instance))
            candidates = sorted((root for root in image1.readable_roots()
                                 if (root.txg, root.instance) < (chosen.txg, chosen.instance)),
                                key=lambda root: -root.txg)
            for rollback_root in candidates:
                for record_count in rollback_record_counts:
                    per_arm = {}
                    for arm in ARMS + CONTROLS:
                        for rule in NEXT_RULES:
                            baseline = recover(image1, arm, rule)
                            if baseline.file == 'recovery-failed':
                                continue
                            try:
                                writes2, state2 = mount_rollback(image1, baseline, rollback_root, record_count)
                            except AssertionError:
                                tally.add((family, arm, rule, 'allocation_exhausted'))
                                continue
                            for key2, image2, roots2 in crash_states_with_persisted_roots(image1, writes2):
                                after = recover(image2, arm, rule)
                                per_arm.setdefault(key2, {})[(arm, rule)] = after
                                prefix = (f'pattern={list(pattern)} crash1={key1} R_old=({rollback_root.instance},{rollback_root.txg}) '
                                          f'rollback_records={record_count} crash2={key2} image: {describe_image(image2)}')
                                committed = any(root.instance == state2.instance for root in roots2)
                                if arm in TAIL_READING_ARMS:
                                    judge_tail_one_arm(tally, family, image2, after, prefix)
                                if not committed:
                                    differs = after.outcome() != baseline.outcome()
                                    tally.add((family, arm, rule, 'uncommitted_states'))
                                    tally.add((family, arm, rule, 'uncommitted_differs_from_no_rollback'),
                                              f'{prefix} || baseline {describe_recovery(baseline)} || after {describe_recovery(after)}',
                                              int(differs))
                                    if differs:
                                        cause = rollback_difference_cause(image2, baseline)
                                        tally.add((family, arm, rule, 'uncommitted_differs_cause', cause),
                                                  f'{prefix} || baseline {describe_recovery(baseline)} || after {describe_recovery(after)}')
                                else:
                                    holds = after.root == (state2.instance, state2.next_txg - 1) and \
                                        after.file == (rollback_root.version[0] if rollback_root.version[0] else 'no-file')
                                    tally.add((family, arm, rule, 'committed_states'))
                                    tally.add((family, arm, rule, 'committed_wrong'),
                                              f'{prefix} || {describe_recovery(after)}', int(not holds))
                    for key2, recoveries in per_arm.items():
                        if all(('ORACLE', rule) in recoveries for rule in NEXT_RULES):
                            compare_to_oracle(tally, family, recoveries,
                                              f'pattern={list(pattern)} crash1={key1} R_old=({rollback_root.instance},'
                                              f'{rollback_root.txg}) rollback_records={record_count} crash2={key2}')


def rollback_difference_cause(image_after, baseline):
    """回退没提交却改了结果，归到哪一类：基线施加的记录被回退的记录盖掉 / 记录还在而点名单元被回退的单元写盖掉 / 都不是。"""
    records = image_after.readable_records()
    for key in baseline.applied:
        if key not in records:
            return 'baseline_chain_record_overwritten'
    for key in baseline.applied:
        if not verified(image_after, records[key]):
            return 'baseline_chain_unit_reused'
    return 'chain_start_anchor_lost'


def judge_tail_one_arm(tally, family, image, base, prefix):
    true_tail = chosen_superblock(image).tail
    for kind, value in tail_perturbations(true_tail):
        perturbed = recover(image, base.arm, base.next_rule, value)
        example = f'{prefix} || tail {true_tail}->{value} || {describe_recovery(base)} || {describe_recovery(perturbed)}'
        key = (family, 'T2', base.arm, base.next_rule, kind)
        tally.add(key + ('perturbations',))
        tally.add(key + ('outcome_changed',), example, int(perturbed.outcome() != base.outcome()))
        tally.add(key + ('file_changed',), example, int(perturbed.file != base.file))
        tally.add(key + ('next_changed',), example, int(perturbed.next_counter != base.next_counter))


def faulted_images(image):
    """层 1：一条记录两份都读不出、一个根槽读不出、以及两者各一。"""
    ring_slots = sorted(image.ring)
    root_slots = sorted(key for key, root in image.roots.items() if root.instance != 0)
    for slot in ring_slots:
        faulted = image.copy()
        faulted.unreadable_ring_slots = frozenset({slot})
        yield f'ring{slot}', faulted
    for key in root_slots:
        faulted = image.copy()
        faulted.unreadable_root_slots = frozenset({key})
        yield f'root{key}', faulted
        for slot in ring_slots:
            double = faulted.copy()
            double.unreadable_ring_slots = frozenset({slot})
            yield f'root{key}+ring{slot}', double


def run_faults(tally, max_length):
    family = 'L1-faults'
    for pattern in patterns(max_length):
        base, writes = first_mount_stream(pattern)
        for key, image, roots in crash_states_with_persisted_roots(base, writes):
            for fault, faulted in faulted_images(image):
                if not faulted.readable_roots():
                    continue
                recoveries = all_recoveries(faulted)
                prefix = f'pattern={list(pattern)} crash={key} fault={fault} image: {describe_image(faulted)}'
                compare_to_oracle(tally, family, recoveries, prefix)
                judge_durability(tally, family, recoveries, roots, 1, prefix)
                judge_tail(tally, family, faulted, recoveries, prefix)


ROLLBACK_ISOLATES_ABOVE_WATER_UNITS = False
# 一次挂载里连做很多次发布：根环 6 槽轮转一圈以上，环里的旧记录点名的单元会被合法复用（T2 叠块重用那一格要的状态）。
LONG_PATTERNS = [(1,) * length for length in range(7, 15)] + [(2,) * length for length in range(5, 9)] + \
    [(1, 2) * length for length in range(3, 6)]
MUTATION = None   # 自证用：None / 'strict_start_at_covered' / 'literal_trusts_tail_as_covered' / 'trust_tail_starts_at_ring_max'


def run_rollback_with_isolation(tally):
    global ROLLBACK_ISOLATES_ABOVE_WATER_UNITS
    ROLLBACK_ISOLATES_ABOVE_WATER_UNITS = True
    try:
        run_rollback(tally, 4)
    finally:
        ROLLBACK_ISOLATES_ABOVE_WATER_UNITS = False


def selftest():
    """判别力自证：每条检查在注入对应的毛病后必须由绿转红；不注入时必须是绿的。"""
    global MUTATION
    checks = [
        (None, 3, ('L0-first-mount', 'BING_STRICT', 'prefix', 'outcome_differs'), 'zero'),
        ('strict_start_at_covered', 3, ('L0-first-mount', 'BING_STRICT', 'prefix', 'outcome_differs'), 'positive'),
        (None, 3, ('L0-first-mount', 'T2', 'JIA_LITERAL', 'prefix', 'stale', 'outcome_changed'), 'zero'),
        ('literal_trusts_tail_as_covered', 3, ('L0-first-mount', 'T2', 'JIA_LITERAL', 'prefix', 'stale', 'outcome_changed'), 'positive'),
        (None, 6, ('L0-first-mount', 'T2', 'TRUST_TAIL', 'prefix', 'stale', 'file_changed'), 'positive'),
    ]
    failures = 0
    for mutation, length, key, expectation in checks:
        MUTATION = mutation
        tally = Tally()
        run_first_mount(tally, length)
        value = tally.counts.get(key, 0)
        holds = value == 0 if expectation == 'zero' else value > 0
        failures += int(not holds)
        print(f'C245 name=selftest mutation={mutation} key={"/".join(key)} expect={expectation} value={value} ok={holds}')
    MUTATION = 'ringmax_ignores_ring'
    tally = Tally()
    run_rollback(tally, 2)
    value = tally.counts.get(('L0-rollback', 'ORACLE', 'ringmax', 'uncommitted_differs_cause', 'baseline_chain_record_overwritten'), 0)
    failures += int(value == 0)
    print(f'C245 name=selftest mutation=ringmax_ignores_ring key=L0-rollback/ORACLE/ringmax/uncommitted_differs_cause/'
          f'baseline_chain_record_overwritten expect=positive value={value} ok={value > 0}')
    MUTATION = None
    print(f'C245 name=selftest failures={failures}')
    if failures:
        sys.exit(1)


def emit_tally(tally, output_lines, name):
    for key in sorted(tally.counts, key=str):
        output_lines.append(f'C245 name={name} key={"/".join(map(str, key))} value={tally.counts[key]}')
    for key in sorted(tally.examples, key=str):
        output_lines.append(f'C245 name={name}-example key={"/".join(map(str, key))} text={tally.examples[key]}')


def main(arguments):
    selected = arguments[0] if arguments else 'all'
    lines = [f'C245 name=config ring={RING} f={F_SAFETY} n_limit={N_LIMIT} regions={REGIONS} '
             f'slots_per_region={SLOTS_PER_REGION} unit_slots={UNIT_SLOTS} disks={DISKS} mutation={MUTATION}']
    families = {
        'first': lambda tally: run_first_mount(tally, 6),
        'second': lambda tally: run_second_mount(tally, 4),
        'rollback': lambda tally: run_rollback(tally, 4),
        'rollback-isolated': run_rollback_with_isolation,
        'long': lambda tally: run_first_mount(tally, 0, True, LONG_PATTERNS, 'L0-long-single-mount'),
        'faults': lambda tally: run_faults(tally, 4),
    }
    if selected == '--selftest':
        selftest()
        return
    for name, runner in families.items():
        if selected not in ('all', name):
            continue
        tally = Tally()
        runner(tally)
        emit_tally(tally, lines, name)
    lines.append(f'C245 name=end emitted={len(lines) + 1}')
    print('\n'.join(lines))


if __name__ == '__main__':
    main(sys.argv[1:])
