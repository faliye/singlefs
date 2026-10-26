"""第三轮正推腿自己的小模型（不读也不复用第二轮攻方原型的代码）：
按 crates/singlefs-harness/src/segments.rs 的切段规则、crates/singlefs-checker-tier/src/crash.rs 的原地覆写第三态判据与
crates/singlefs-core/src/transaction.rs 的系统配置槽写协议，把「系统配置槽」符号化后枚举崩溃状态。
槽内容：('Z',) 全零；('V', g) 自证过、世代号 g；('T',) 撕裂（非零、自证不过）。纯 Python，不起 cargo。"""
import itertools

Z = ('Z',)
T = ('T',)
ZERO_TORN = ('ZT',)  # 记号：撕裂的整段清零，落到盘面上是 (Z, T)


def V(g):
    return ('V', g)


def valid(c):
    return c[0] == 'V'


class Write:
    def __init__(self, dev, kind, slot=None, content=None, fua=False, covers_slots=True):
        self.dev, self.kind, self.slot, self.content, self.fua = dev, kind, slot, content, fua
        # 整段清零只有罩住 [0, 根环基址) 的那一次碰系统配置槽；清根环与 journal 环的不碰
        self.covers_slots = covers_slots
        # 只有 write_at 发的写算「有字节」；整段清零（zerofill）不算（crash.rs is_tearable_in_place_overwrite 第二条）
        self.has_bytes = kind != 'zerofill'


class Recorder:
    """录制 + 切段。切段规则逐条照 segments.rs 的 SegmentClosingRule。"""

    def __init__(self, base, devices=(0, 1)):
        self.base = {d: list(base[d]) for d in devices}
        self.disk = {d: list(base[d]) for d in devices}
        self.devices = list(devices)
        self.writes, self.segments, self.cur = [], [], []
        self.unreleased, self.cur_writes = set(), 0

    def _close_if_released(self):
        if self.cur_writes > 0 and not self.unreleased:
            self.segments.append(self.cur)
            self.cur, self.cur_writes = [], 0

    def write(self, dev, kind, slot=None, content=None, fua=False, covers_slots=True):
        w = Write(dev, kind, slot, content, fua, covers_slots)
        self.writes.append(w)
        idx = len(self.writes) - 1
        self.cur.append(idx)
        self.cur_writes += 1
        if kind == 'sys':
            self.disk[dev][slot] = content
        elif kind == 'zerofill' and covers_slots:
            self.disk[dev] = [Z, Z]
        if fua:
            self.unreleased.discard(dev)
            self._close_if_released()
        else:
            self.unreleased.add(dev)

    def barrier(self, dev):
        self.unreleased.discard(dev)
        self._close_if_released()

    def barrier_all(self):
        for d in self.devices:
            self.barrier(d)

    def finish(self):
        if self.cur:
            self.segments.append(self.cur)
            self.cur, self.cur_writes = [], 0
        return self


def tearable(rec, i):
    """crash.rs is_tearable_in_place_overwrite 的三条：不是单元写、长于一个扇区、罩住的范围里原来有东西（基镜像非零，或同盘更早有带字节的写与它重叠）。
    系统配置槽写罩一个槽；整段清零（zerofill，mkfs 清 [0, 根环基址)）罩这块盘的两个槽。"""
    w = rec.writes[i]
    if w.kind == 'sys':
        if rec.base[w.dev][w.slot] != Z:
            return True
        return any(e.kind == 'sys' and e.has_bytes and e.dev == w.dev and e.slot == w.slot
                   for e in rec.writes[:i])
    if w.kind == 'zerofill' and w.covers_slots:
        if any(c != Z for c in rec.base[w.dev]):
            return True
        return any(e.kind == 'sys' and e.has_bytes and e.dev == w.dev for e in rec.writes[:i])
    return False


def torn_contents(rec):
    """每个可撕写的撕裂内容：旧字节取「这次写之前的写全落了」；新旧相同则撕不出第三种。"""
    out, disk = {}, {d: list(rec.base[d]) for d in rec.devices}
    for i, w in enumerate(rec.writes):
        if w.kind == 'sys':
            if tearable(rec, i):
                old = disk[w.dev][w.slot]
                out[i] = w.content if old == w.content else T
            disk[w.dev][w.slot] = w.content
        elif w.kind == 'zerofill' and w.covers_slots:
            if tearable(rec, i):
                out[i] = ZERO_TORN  # 撕裂的整段清零：前一半是零（盖住两个槽的头），后一半还是旧的 ⇒ 槽 0 为零、槽 1 撕坏
            disk[w.dev] = [Z, Z]
    return out
