#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12（deadlist 条目的形态） 第一轮攻方腿模型。只用 Python 标准库。
# 真值 = D5 前提二的引用区间：块 b 被根 R 引用 <=> b 在 R 的活集合（头）或捕获集合（快照）里。
# 每一步之后比对：臂放掉的块集合 == 已无任何根引用的块集合（多放 = 数据丢失方向，漏放 = 泄漏方向）。
import sys, random, copy, json
from collections import deque

INF_TXG = 1 << 62


class Block:
    __slots__ = ("bid", "cls", "btree", "birth", "wseq")

    def __init__(self, bid, cls, btree, birth, wseq):
        self.bid, self.cls, self.btree, self.birth, self.wseq = bid, cls, btree, birth, wseq

    def mapkey(self):  # 中央映射 key 的段序（D19 已定项 6）：类标签, 出生树, 出生 txg, 写序
        return (self.cls, self.btree, self.birth, self.wseq)


class World:
    """池的结构状态：头、快照、块。与臂无关；臂只决定放哪些块、deadlist 里写什么键。"""

    def __init__(self):
        self.txg = 1          # 开放 checkpoint 的号；发布时 +1
        self.next_tid = 11    # 树 ID 从 11 起（D8 已定项 11）
        self.next_bid = 0
        self.blocks, self.heads, self.snaps, self.mk2bid = {}, {}, {}, {}
        self.root = self.alloc_tid()
        self.heads[self.root] = {"live": set(), "origin": None, "wseq": 0}

    def alloc_tid(self):
        tid = self.next_tid
        self.next_tid += 1
        return tid

    def alive_snaps(self):
        return sorted(self.snaps, key=lambda sid: (self.snaps[sid]["txg"], sid))  # (诞生 txg, 树 ID) 全序

    def head_snaps(self, head):
        return [sid for sid in self.alive_snaps() if self.snaps[sid]["head"] == head]

    def next_owner(self, sid):
        head = self.snaps[sid]["head"]
        lineage = self.head_snaps(head)
        position = lineage.index(sid)
        return ("s", lineage[position + 1]) if position + 1 < len(lineage) else ("h", head)

    def clones_of(self, sid):
        return [head for head in self.heads if self.heads[head]["origin"] == sid]

    def refs(self, bid):
        count = sum(1 for head_state in self.heads.values() if bid in head_state["live"])
        return count + sum(1 for snap_state in self.snaps.values() if bid in snap_state["cap"])

    def lineage_members(self, head):
        """头 head 的谱系成员，按序：origin 快照（克隆头才有）、它自己的活快照、头本身。"""
        members = []
        origin = self.heads[head]["origin"]
        if origin is not None:
            members.append(("o", origin))
        members += [("s", sid) for sid in self.head_snaps(head)]
        members.append(("h", head))
        return members

    def world_prev_txg(self, owner):
        """从世界现算的 prev：同头前一个活快照的 txg；克隆头的第一个取 origin 的 txg；否则 0。"""
        kind, ident = owner
        head = ident if kind == "h" else self.snaps[ident]["head"]
        members = self.lineage_members(head)
        position = members.index(owner)
        if position == 0:
            return 0
        previous_kind, previous_ident = members[position - 1]
        return self.snaps[previous_ident]["txg"]


class KeySpace:
    """一棵定宽 key 的树的模型：集合语义（同 key 第二次插入折叠成一条），value 空。"""

    def __init__(self):
        self.entries = {}
        self.collapse = 0

    def insert(self, key):
        if key in self.entries:
            self.collapse += 1
        self.entries[key] = None

    def delete(self, key):
        del self.entries[key]

    def range(self, low, high):
        return sorted(key for key in self.entries if low <= key < high)


def birth_of(key):   # 映射 key 住在每条臂键的末尾四段；出生 txg 是倒数第二段
    return key[-2]


class ArmBase:
    """三条臂共用的删除判定、交接与 prev 传递；变异开关 mut 用来证明真值检查会红。"""

    name = "BASE"

    def __init__(self, world, mut=()):
        self.mut = set(mut)
        self.freed, self.double = set(), 0
        self.ph = {world.root: 0}   # previous_snapshot_txg，每头一份
        self.ps = {}                # 每个快照的 prev（逻辑值）
        self.m = dict(appends=0, rewrites=0, free_deletes=0, snap_rewrite_max=0,
                      destroy_rewrite_max=0, scan_linear=0, scan_i=0, seeks_i=0,
                      scan_ii=0, seeks_ii=0)

    def do_free(self, bids):
        for bid in bids:
            if bid in self.freed:
                self.double += 1
            self.freed.add(bid)

    def kill(self, world, head, block):
        # brt_naive：「别的头还活引用它（旁表计数 > 1）就只减计数、不走 D5 那条路」这种协议读法，与臂无关
        if "brt_naive" in self.mut and any(block.bid in state["live"] for state in world.heads.values()):
            return
        prev_txg = self.ph[head]
        immediate = block.birth > prev_txg or ("ge" in self.mut and block.birth == prev_txg)
        if immediate:
            self.do_free([block.bid])
        else:
            self.m["appends"] += 1
            self.dl_append(world, head, block)

    def snapshot(self, world, head, sid):
        self.ps[sid] = self.ph[head]
        self.dl_handover(world, head, sid)
        self.ph[head] = world.snaps[sid]["txg"]

    def clone(self, world, origin_sid, new_head):
        self.ph[new_head] = 0 if "clone0" in self.mut else world.snaps[origin_sid]["txg"]
        self.dl_newhead(world, new_head)

    def destroy_snap(self, world, sid, info):
        prev_txg = self.ps[sid]
        self.dl_cascade(world, sid, info, prev_txg)
        next_kind, next_ident = info["next"]
        if "noprop" not in self.mut:
            if next_kind == "s":
                self.ps[next_ident] = prev_txg
            else:
                self.ph[next_ident] = prev_txg
        del self.ps[sid]

    def destroy_head(self, world, head, info):
        self.do_free([bid for bid in info["live"] if world.blocks[bid].birth > info["origin_txg"]])
        self.dl_drophead(world, head)
        del self.ph[head]

    def publish(self, world):
        pass

    def stored_prev(self, owner):
        kind, ident = owner
        return self.ph[ident] if kind == "h" else self.ps[ident]


def account_filter(arm, keys, prev_txg, group_len):
    """记过滤代价：线性扫描看 len(keys) 条；(i) 按（前缀, 类, 出生树）跳扫 = 被放条数 + 组数、
    每组一次定位；(ii) 键里前置一份出生 txg = 被放条数、每个纪元段一次定位。返回要放的键。"""
    released = [key for key in keys if birth_of(key) > prev_txg]
    groups = len({key[:group_len] for key in keys})
    arm.m["scan_linear"] += len(keys)
    arm.m["scan_i"] += len(released) + groups
    arm.m["seeks_i"] += groups
    arm.m["scan_ii"] += len(released)
    arm.m["seeks_ii"] += len({key[:group_len - 2] for key in keys})
    return released


class Ref(ArmBase):
    """阳性对照：deadlist 是挂在头 / 快照对象上的集合，交接是换指针（ZFS 形态）。"""
    name = "REF"

    def __init__(self, world, mut=()):
        super().__init__(world, mut)
        self.dl = {("h", world.root): set()}

    def dl_append(self, world, head, block):
        self.dl[("h", head)].add(block.bid)

    def dl_handover(self, world, head, sid):
        self.dl[("s", sid)] = self.dl[("h", head)]
        self.dl[("h", head)] = set()

    def dl_newhead(self, world, new_head):
        self.dl[("h", new_head)] = set()

    def dl_cascade(self, world, sid, info, prev_txg):
        next_owner = info["next"]
        next_set, own_set = self.dl[next_owner], self.dl.pop(("s", sid))
        if "filter_own" in self.mut:   # 变异：过滤被销毁那一份自己的，而不是下一侧的
            released = {bid for bid in own_set if world.blocks[bid].birth > prev_txg}
            self.dl[next_owner] = next_set | (own_set - released)
        else:
            released = {bid for bid in next_set if world.blocks[bid].birth > prev_txg}
            self.dl[next_owner] = (next_set - released) | own_set
        self.do_free(released)

    def dl_drophead(self, world, head):
        self.dl.pop(("h", head))

    def decode(self, world):
        return {owner: set(bids) for owner, bids in self.dl.items()}, []


class Jia(ArmBase):
    """甲：key = 归属树 ID 8 + 映射 key；建快照时把活头的条目逐条改键到新快照的树 ID。"""
    name = "JIA"

    def __init__(self, world, mut=()):
        super().__init__(world, mut)
        self.ks = KeySpace()

    def rekey(self, keys, new_prefix):
        for key in keys:
            self.ks.delete(key)
            self.ks.insert((new_prefix,) + key[1:])
        self.m["rewrites"] += len(keys)
        return len(keys)

    def dl_append(self, world, head, block):
        self.ks.insert((head,) + block.mapkey())

    def dl_handover(self, world, head, sid):
        moved = self.rekey(self.ks.range((head,), (head + 1,)), sid)
        self.m["snap_rewrite_max"] = max(self.m["snap_rewrite_max"], moved)

    def dl_newhead(self, world, new_head):
        pass

    def release_keys(self, world, keys):
        for key in keys:
            self.ks.delete(key)
            self.m["free_deletes"] += 1
            self.do_free([world.mk2bid[key[-4:]]])

    def dl_cascade(self, world, sid, info, prev_txg):
        next_tid = info["next"][1]
        self.release_keys(world, account_filter(self, self.ks.range((next_tid,), (next_tid + 1,)), prev_txg, 3))
        moved = self.rekey(self.ks.range((sid,), (sid + 1,)), next_tid)
        self.m["destroy_rewrite_max"] = max(self.m["destroy_rewrite_max"], moved)

    def dl_drophead(self, world, head):
        for key in self.ks.range((head,), (head + 1,)):
            self.ks.delete(key)

    def decode(self, world):
        result, anomalies = {}, []
        for key in self.ks.entries:
            tid = key[0]
            owner = ("h", tid) if tid in world.heads else ("s", tid)
            if owner[0] == "s" and tid not in world.snaps:
                anomalies.append(("orphan", key))
                continue
            result.setdefault(owner, set()).add(world.mk2bid[key[-4:]])
        return result, anomalies


class JiaBatched(Jia):
    """甲拆批：交接每次发布只改键 batch 条（D8 已定项 5 意图机制那种走法）。键里没有纪元，
    只能按键序挑『交接那一刻已在活头名下的那几条』。这是问甲的支持者：不带纪元怎么拆批。"""
    name = "JIA_B"

    def __init__(self, world, mut=(), batch=1):
        super().__init__(world, mut)
        self.batch, self.pending = batch, {}

    def dl_handover(self, world, head, sid):
        owed = sum(remaining for _, remaining in self.pending.get(head, []))
        count = len(self.ks.range((head,), (head + 1,))) - owed
        self.pending.setdefault(head, []).append([sid, count])

    def advance(self, head, budget):
        queue = self.pending.get(head, [])
        while queue and budget > 0:
            sid, remaining = queue[0]
            keys = self.ks.range((head,), (head + 1,))[:min(remaining, budget)]
            self.rekey(keys, sid)
            queue[0][1] -= len(keys)
            budget -= len(keys)
            if queue[0][1] <= 0 or not keys:
                queue.pop(0)

    def publish(self, world):
        for head in list(self.pending):
            self.advance(head, self.batch)

    def destroy_snap(self, world, sid, info):
        self.advance(info["head"], INF_TXG)   # 销毁前先把欠着的交接做完
        super().destroy_snap(world, sid, info)

    def destroy_head(self, world, head, info):
        self.advance(head, INF_TXG)
        self.pending.pop(head, None)
        super().destroy_head(world, head, info)


class Yi(ArmBase):
    """乙：树表条目带 deadlist 号；key = deadlist 号 8 + 映射 key。建快照换号 O(1)，新号取新快照的树 ID。
    merge=move：级联一律把被销毁那一份改键到下一侧的号；merge=min：两个方向挑少的那一边（ZFS 旧实现的换对象）。"""
    name = "YI"

    def __init__(self, world, mut=(), merge="min"):
        super().__init__(world, mut)
        self.ks, self.merge = KeySpace(), merge
        self.name = "YI_" + merge
        self.number = {("h", world.root): world.root}

    def move(self, keys, new_number):
        for key in keys:
            self.ks.delete(key)
            self.ks.insert((new_number,) + key[1:])
        self.m["rewrites"] += len(keys)

    def dl_append(self, world, head, block):
        self.ks.insert((self.number[("h", head)],) + block.mapkey())

    def dl_handover(self, world, head, sid):
        self.number[("s", sid)] = self.number[("h", head)]
        self.number[("h", head)] = sid

    def dl_newhead(self, world, new_head):
        self.number[("h", new_head)] = new_head

    def dl_cascade(self, world, sid, info, prev_txg):
        next_owner = info["next"]
        next_number, own_number = self.number[next_owner], self.number.pop(("s", sid))
        released = account_filter(self, self.ks.range((next_number,), (next_number + 1,)), prev_txg, 3)
        for key in released:
            self.ks.delete(key)
            self.m["free_deletes"] += 1
            self.do_free([world.mk2bid[key[-4:]]])
        kept = self.ks.range((next_number,), (next_number + 1,))
        own = self.ks.range((own_number,), (own_number + 1,))
        if self.merge == "move" or len(own) <= len(kept):
            self.move(own, next_number)
            moved = len(own)
        else:
            self.move(kept, own_number)
            self.number[next_owner] = own_number
            moved = len(kept)
        self.m["destroy_rewrite_max"] = max(self.m["destroy_rewrite_max"], moved)

    def dl_drophead(self, world, head):
        number = self.number.pop(("h", head))
        for key in self.ks.range((number,), (number + 1,)):
            self.ks.delete(key)

    def decode(self, world):
        owner_of = {number: owner for owner, number in self.number.items()}
        result, anomalies = {}, []
        for key in self.ks.entries:
            owner = owner_of.get(key[0])
            if owner is None:
                anomalies.append(("orphan", key))
                continue
            result.setdefault(owner, set()).add(world.mk2bid[key[-4:]])
        return result, anomalies


class Bing(ArmBase):
    """丙：key = 头的树 ID 8 + 纪元 8 + 映射 key；纪元 = 块被杀那一刻头的 previous_snapshot_txg。
    mode A：字面读法——把 S 那一段 (头, prev(S)) 并进下一侧原来的纪元，再按 birth > prev(S) 过滤，prev 传递。
    mode B：一致读法——下一侧那一段按 birth > prev(S) 放掉，留下的改键到 (头, prev(S))，prev 传递。
    mode R：区间读法（本腿提的收严）——X 的 deadlist = 纪元落在 [prev(X), X.txg) 的全部段，级联只放不改键。"""
    name = "BING"

    def __init__(self, world, mut=(), mode="B"):
        super().__init__(world, mut)
        self.ks, self.mode = KeySpace(), mode
        self.name = "BING_" + mode

    def epoch_of_head(self, head):
        return self.ph[head]

    def owner_epoch(self, owner):
        return self.stored_prev(owner)

    def segment(self, head, epoch):
        return self.ks.range((head, epoch), (head, epoch + 1))

    def rekey(self, keys, head, new_epoch):
        for key in keys:
            self.ks.delete(key)
            self.ks.insert((head, new_epoch) + key[2:])
        self.m["rewrites"] += len(keys)
        return len(keys)

    def release(self, world, keys):
        for key in keys:
            self.ks.delete(key)
            self.m["free_deletes"] += 1
            self.do_free([world.mk2bid[key[-4:]]])

    def dl_append(self, world, head, block):
        self.ks.insert((head, self.epoch_of_head(head)) + block.mapkey())

    def dl_handover(self, world, head, sid):
        pass   # 交接零改写：之后被杀的块自然带上新纪元

    def dl_newhead(self, world, new_head):
        pass

    def dl_cascade(self, world, sid, info, prev_txg):
        head, next_owner = info["head"], info["next"]
        next_epoch, own_epoch = self.owner_epoch(next_owner), self.owner_epoch(("s", sid))
        moved = 0
        if self.mode == "A":
            if own_epoch != next_epoch:
                moved = self.rekey(self.segment(head, own_epoch), head, next_epoch)
            self.release(world, account_filter(self, self.segment(head, next_epoch), prev_txg, 4))
        elif self.mode == "B":
            self.release(world, account_filter(self, self.segment(head, next_epoch), prev_txg, 4))
            if next_epoch != own_epoch:
                moved = self.rekey(self.segment(head, next_epoch), head, own_epoch)
        else:
            keys = self.ks.range((head, next_epoch), (head, info["next_txg"]))
            self.release(world, account_filter(self, keys, prev_txg, 4))
        self.m["destroy_rewrite_max"] = max(self.m["destroy_rewrite_max"], moved)

    def dl_drophead(self, world, head):
        for key in self.ks.range((head,), (head + 1,)):
            self.ks.delete(key)

    def decode(self, world):
        result, anomalies = {}, []
        for key in self.ks.entries:
            head, epoch = key[0], key[1]
            if head not in world.heads:
                anomalies.append(("orphan-head", key))
                continue
            owners = [("s", sid) for sid in world.head_snaps(head)] + [("h", head)]
            if self.mode == "R":
                hit = [owner for owner in owners if self.stored_prev(owner) <= epoch <
                       (world.snaps[owner[1]]["txg"] if owner[0] == "s" else INF_TXG)]
            else:
                hit = [owner for owner in owners if self.owner_epoch(owner) == epoch]
            if len(hit) != 1:
                anomalies.append(("orphan" if not hit else "ambiguous", key))
            for owner in hit:
                result.setdefault(owner, set()).add(world.mk2bid[key[-4:]])
        return result, anomalies


class BingTid(Bing):
    """丙′（本腿提的另一种收严，只在本模型上量过）：纪元段装『头上一个快照的树 ID』，克隆头的第一段装
    origin 快照的树 ID，没有快照装 0；点读法 + 一致级联。树 ID 随创建序严格递增，同一 checkpoint 的两个快照也不同号。"""

    def __init__(self, world, mut=()):
        super().__init__(world, mut, mode="B")
        self.name = "BING_T"
        self.pth, self.pts = {world.root: 0}, {}

    def epoch_of_head(self, head):
        return self.pth[head]

    def owner_epoch(self, owner):
        kind, ident = owner
        return self.pth[ident] if kind == "h" else self.pts[ident]

    def snapshot(self, world, head, sid):
        self.pts[sid] = self.pth[head]
        super().snapshot(world, head, sid)
        self.pth[head] = sid

    def clone(self, world, origin_sid, new_head):
        super().clone(world, origin_sid, new_head)
        self.pth[new_head] = origin_sid

    def destroy_snap(self, world, sid, info):
        own_epoch = self.pts[sid]
        super().destroy_snap(world, sid, info)
        kind, ident = info["next"]
        if "noprop" not in self.mut:
            if kind == "s":
                self.pts[ident] = own_epoch
            else:
                self.pth[ident] = own_epoch
        del self.pts[sid]

    def destroy_head(self, world, head, info):
        super().destroy_head(world, head, info)
        del self.pth[head]


def arm_signature(arm):
    extra = []
    for attribute in ("dl", "ks", "number", "pth", "pts", "pending"):
        if hasattr(arm, attribute):
            value = getattr(arm, attribute)
            if attribute == "ks":
                value = sorted(value.entries)
            elif attribute == "dl":
                value = sorted((owner, sorted(bids)) for owner, bids in value.items())
            elif isinstance(value, dict):
                value = sorted(value.items())
            extra.append(repr(value))
    return (tuple(sorted(arm.freed)), arm.double, tuple(sorted(arm.ph.items())),
            tuple(sorted(arm.ps.items())), tuple(extra))


def signature(world, arm):
    heads = tuple(sorted((head, tuple(sorted(state["live"])), state["origin"]) for head, state in world.heads.items()))
    snaps = tuple(sorted((sid, state["head"], state["txg"], tuple(sorted(state["cap"])))
                         for sid, state in world.snaps.items()))
    blocks = tuple(sorted((block.bid, block.cls, block.btree, block.birth) for block in world.blocks.values()))
    return (world.txg, world.next_tid, heads, snaps, blocks, arm_signature(arm))


def oracle(world, arm):
    """真值：返回（多放的块，重复放的次数，漏放的块）。"""
    over = sorted(bid for bid in arm.freed if world.refs(bid) > 0)
    leak = sorted(bid for bid in world.blocks if world.refs(bid) == 0 and bid not in arm.freed)
    return over, arm.double, leak


def expected_deadlists(world):
    """从世界现算每份 deadlist 该装什么（按快照分箱的向量等式，与臂的记账不共用代码）：头 h 丢掉了块 b、
    b 仍被 h 谱系里某个成员捕获 ⇒ b 在 next(L) 那一份里，L = 谱系里最后一个捕获 b 的成员。"""
    expected = {}
    for head in world.heads:
        members = world.lineage_members(head)
        for owner in members:
            if owner[0] != "o":
                expected.setdefault(owner, set())
        last_capture = {}
        for position, (kind, ident) in enumerate(members[:-1]):
            for bid in world.snaps[ident]["cap"]:
                last_capture[bid] = position
        for bid, position in last_capture.items():
            if bid not in world.heads[head]["live"]:
                expected[members[position + 1]].add(bid)
    return expected


def checker(world, arm):
    """U4 的四道检查，红的返回名字：vector（每份 deadlist 与现算的期望逐份相等）、i36（birth ≤ 从世界现算的
    prev(X)）、c195（臂存的 prev 等于从世界现算的 prev）、decode（没有孤儿键、没有一个键落进两份）。"""
    red = []
    decoded, anomalies = arm.decode(world)
    expected = expected_deadlists(world)
    if any(decoded.get(owner, set()) != expected.get(owner, set()) for owner in set(decoded) | set(expected)):
        red.append("vector")
    for owner, bids in decoded.items():
        if owner in expected and any(world.blocks[bid].birth > world.world_prev_txg(owner) for bid in bids):
            red.append("i36")
            break
    stale = [owner for head in world.heads for owner in world.lineage_members(head)
             if owner[0] != "o" and arm.stored_prev(owner) != world.world_prev_txg(owner)]
    if stale:
        red.append("c195")
    if anomalies:
        red.append("decode")
    return red


def step(world, arms, op):
    kind = op[0]
    if kind == "W":
        head = op[1]
        head_state = world.heads[head]
        head_state["wseq"] += 1
        block = Block(world.next_bid, op[2] if len(op) > 2 else 1, head, world.txg, head_state["wseq"])
        world.next_bid += 1
        world.blocks[block.bid] = block
        world.mk2bid[block.mapkey()] = block.bid
        head_state["live"].add(block.bid)
    elif kind == "K":
        head, bid = op[1], op[2]
        world.heads[head]["live"].discard(bid)
        for arm in arms:
            arm.kill(world, head, world.blocks[bid])
    elif kind in ("P", "PS", "PSS"):
        for _ in range({"P": 0, "PS": 1, "PSS": 2}[kind]):
            sid = world.alloc_tid()
            world.snaps[sid] = {"head": op[1], "txg": world.txg, "cap": frozenset(world.heads[op[1]]["live"])}
            for arm in arms:
                arm.snapshot(world, op[1], sid)
        for arm in arms:
            arm.publish(world)
        world.txg += 1
    elif kind == "D":
        sid = op[1]
        next_owner = world.next_owner(sid)
        info = {"head": world.snaps[sid]["head"], "next": next_owner,
                "next_txg": world.snaps[next_owner[1]]["txg"] if next_owner[0] == "s" else INF_TXG}
        del world.snaps[sid]
        for arm in arms:
            arm.destroy_snap(world, sid, info)
    elif kind == "C":
        sid, new_head = op[1], world.alloc_tid()
        world.heads[new_head] = {"live": set(world.snaps[sid]["cap"]), "origin": sid, "wseq": 0}
        for arm in arms:
            arm.clone(world, sid, new_head)
    elif kind == "X":
        head = op[1]
        head_state = world.heads.pop(head)
        origin = head_state["origin"]
        info = {"live": set(head_state["live"]), "origin_txg": 0 if origin is None else world.snaps[origin]["txg"]}
        for arm in arms:
            arm.destroy_head(world, head, info)


ARMS = {
    "REF": lambda world, mut=(): Ref(world, mut),
    "JIA": lambda world, mut=(): Jia(world, mut),
    "JIA_B": lambda world, mut=(): JiaBatched(world, mut, batch=1),
    "YI_min": lambda world, mut=(): Yi(world, mut, merge="min"),
    "YI_move": lambda world, mut=(): Yi(world, mut, merge="move"),
    "BING_A": lambda world, mut=(): Bing(world, mut, mode="A"),
    "BING_B": lambda world, mut=(): Bing(world, mut, mode="B"),
    "BING_R": lambda world, mut=(): Bing(world, mut, mode="R"),
    "BING_T": lambda world, mut=(): BingTid(world, mut),
}
WEIGHTS = {"W": 4.0, "K": 3.0, "P": 1.0, "PS": 1.5, "PSS": 0.4, "D": 1.2, "C": 0.5, "X": 0.3}


def legal_ops(world, limits, classes=(1,)):
    ops = []
    for head in sorted(world.heads):
        if len(world.blocks) < limits["blocks"]:
            ops += [("W", head, cls) for cls in classes]
        ops += [("K", head, bid) for bid in sorted(world.heads[head]["live"])]
    if world.txg < limits["txg"]:
        ops.append(("P",))
        for head in sorted(world.heads):
            if len(world.snaps) + 1 <= limits["snaps"]:
                ops.append(("PS", head))
            if len(world.snaps) + 2 <= limits["snaps"] and limits.get("pss", True):
                ops.append(("PSS", head))
    for sid in world.alive_snaps():
        if not world.clones_of(sid):            # 分叉点闸：children 数克隆（D5 第 2 条）
            ops.append(("D", sid))
        if len(world.heads) < limits["heads"]:
            ops.append(("C", sid))
    for head in sorted(world.heads):
        if head != world.root and not world.head_snaps(head):   # 第二道闸：有自己活快照的头不许销毁
            ops.append(("X", head))
    return ops


def bfs_shortest(arm_name, limits, max_depth, mut=(), budget=300000):
    """按层 BFS（状态去重），返回第一段让真值判红的历史——即最短的那一段；同时记最短的检查判红历史。"""
    world = World()
    arm = ARMS[arm_name](world, mut)
    queue, seen, explored = deque([(world, arm, ())]), {signature(world, arm)}, 0
    first_checker = None
    while queue:
        world, arm, path = queue.popleft()
        if len(path) >= max_depth:
            continue
        for op in legal_ops(world, limits):
            world_copy, arm_copy = copy.deepcopy(world), copy.deepcopy(arm)
            step(world_copy, [arm_copy], op)
            explored += 1
            over, double, leak = oracle(world_copy, arm_copy)
            if first_checker is None:
                red = checker(world_copy, arm_copy)
                if red:
                    first_checker = {"path": path + (op,), "red": red}
            if over or double or leak:
                return {"arm": arm_name, "mut": list(mut), "found": True, "path": path + (op,), "over": over,
                        "double": double, "leak": leak, "explored": explored, "first_checker": first_checker}
            state = signature(world_copy, arm_copy)
            if state not in seen:
                seen.add(state)
                queue.append((world_copy, arm_copy, path + (op,)))
            if explored >= budget:
                return {"arm": arm_name, "mut": list(mut), "found": False, "explored": explored,
                        "depth_reached": len(path), "truncated": True, "first_checker": first_checker}
    return {"arm": arm_name, "mut": list(mut), "found": False, "explored": explored, "depth_reached": max_depth,
            "truncated": False, "first_checker": first_checker}


def fuzz(arm_names, seeds, length, limits, mut=(), classes=(1, 2, 3), check_every=1):
    """随机历史：每步对每条臂跑真值；每 check_every 步跑一次 U4 检查。返回每条臂判红的历史数与第一例。"""
    summary = {name: {"oracle_bad": 0, "checker_red": 0, "first": None, "collapse": 0} for name in arm_names}
    for seed in range(seeds):
        rng = random.Random(seed)
        world = World()
        arms = [ARMS[name](world, mut) for name in arm_names]
        bad, red = [False] * len(arms), [False] * len(arms)
        for index in range(length):
            ops = legal_ops(world, limits, classes)
            if not ops:
                break
            kinds = sorted({op[0] for op in ops})
            kind = rng.choices(kinds, [WEIGHTS[k] for k in kinds])[0]
            op = rng.choice([candidate for candidate in ops if candidate[0] == kind])
            step(world, arms, op)
            for position, arm in enumerate(arms):
                entry = summary[arm_names[position]]
                if not bad[position]:
                    over, double, leak = oracle(world, arm)
                    if over or double or leak:
                        bad[position] = True
                        entry["oracle_bad"] += 1
                        if entry["first"] is None:
                            entry["first"] = {"seed": seed, "step": index, "over": len(over),
                                              "double": double, "leak": len(leak)}
                if not red[position] and index % check_every == 0 and checker(world, arm):
                    red[position] = True
                    entry["checker_red"] += 1
        for position, arm in enumerate(arms):
            summary[arm_names[position]]["collapse"] += getattr(getattr(arm, "ks", None), "collapse", 0)
    return summary


def geometry_ops(shape, n, classes=(1,)):
    """U2 的最坏几何（根头树 ID 11；三个快照依次是 12 / 13 / 14；销毁的是中间那个 13）。"""
    root = 11
    writes = [("W", root, classes[i % len(classes)]) for i in range(n)]
    later = [("W", root, classes[i % len(classes)]) for i in range(n)]

    def kills(low, high):
        return [("K", root, bid) for bid in range(low, high)]
    if shape == "G1":   # 删光 N 个旧块后建快照：甲的交接逐条改键
        return writes + [("PS", root)] + kills(0, n) + [("PS", root)]
    if shape == "G2":   # 下一侧留下的条目多：丙点读法的级联改键
        return writes + [("PS", root), ("PS", root)] + kills(0, n) + [("PS", root), ("D", 13)]
    if shape == "G3":   # 被销毁那一份多：甲、乙 move 的级联改键
        return writes + [("PS", root)] + kills(0, n) + [("PS", root), ("PS", root), ("D", 13)]
    if shape == "G4":   # 两边各一半：乙 min 也躲不开
        return (writes + [("PS", root)] + kills(0, n // 2) + [("PS", root)] + kills(n // 2, n)
                + [("PS", root), ("D", 13)])
    if shape == "G5":   # 一半该放、一半留下：过滤代价（分桶问）
        return writes + [("PS", root)] + later + [("PS", root)] + kills(0, 2 * n) + [("PS", root), ("D", 13)]
    raise ValueError(shape)


def geometry(shape, n, arm_names, classes=(1,)):
    world = World()
    arms = [ARMS[name](world) for name in arm_names]
    for op in geometry_ops(shape, n, classes):
        step(world, arms, op)
    rows = []
    for name, arm in zip(arm_names, arms):
        over, double, leak = oracle(world, arm)
        rows.append(dict(shape=shape, n=n, arm=name, over=len(over), double=double, leak=len(leak), **arm.m))
    return rows


def width_table():
    rows = []
    for label, key_width, entry_width in (("甲/乙 分桶(i)", 35, 35), ("丙 分桶(i)", 43, 43), ("甲/乙 分桶(ii)", 43, 43),
                                          ("丙 分桶(ii)", 51, 51), ("livelist（参照）", 36, 36), ("稀疏旁表（参照）", 27, 31)):
        rows.append(dict(label=label, key_width=key_width, entry_width=entry_width,
                         leaf_fanout_header_115_plus_2k=(16384 - 115 - 2 * key_width) // entry_width,
                         leaf_fanout_header_86_plus_2k=(16384 - 86 - 2 * key_width) // entry_width))
    for extra in (0, 8, 16):
        rows.append(dict(label="树表条目", entry_width=148 + extra, per_level=(16384 - 131) // (148 + extra)))
    return rows


def replay(arm_names, path, mut=()):
    world = World()
    arms = [ARMS[name](world, mut) for name in arm_names]
    lines = []
    for op in path:
        step(world, arms, tuple(op))
        for name, arm in zip(arm_names, arms):
            over, double, leak = oracle(world, arm)
            state = sorted(arm.ks.entries) if hasattr(arm, "ks") else sorted((o, sorted(v)) for o, v in arm.dl.items())
            lines.append(f"{tuple(op)} | {name} | dl={state} | ph={sorted(arm.ph.items())} ps={sorted(arm.ps.items())}"
                         f" | freed={sorted(arm.freed)} | over={over} leak={leak} | checker={checker(world, arm)}")
    return lines


SMALL = {"blocks": 2, "heads": 2, "snaps": 3, "txg": 4}
FUZZ = {"blocks": 40, "heads": 4, "snaps": 8, "txg": 400}


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    rows = []

    def emit(section, payload):
        rows.append(1)
        print(json.dumps({"section": section, **payload}, ensure_ascii=False, default=str))
    if mode in ("bfs", "all"):
        for name in ARMS:
            emit("bfs", bfs_shortest(name, SMALL, 7))
        for mut in (("noprop",), ("filter_own",), ("ge",), ("clone0",), ("brt_naive",)):
            emit("bfs-mutation", bfs_shortest("REF", SMALL, 7, mut))
        emit("bfs-no-pss", bfs_shortest("BING_B", dict(SMALL, pss=False), 7))
        emit("bfs-no-pss", bfs_shortest("BING_A", dict(SMALL, pss=False), 8))
    if mode in ("fuzz", "all"):
        emit("fuzz", {"result": fuzz(list(ARMS), 400, 120, FUZZ)})
        emit("fuzz-no-pss", {"result": fuzz(["BING_B"], 400, 120, dict(FUZZ, pss=False))})
        for mut in (("noprop",), ("ge",), ("clone0",), ("brt_naive",)):
            emit("fuzz-mutation", {"mut": mut, "result": fuzz(["REF", "JIA", "YI_min", "BING_R", "BING_T"], 150, 120, FUZZ, mut)})
        emit("fuzz-mutation", {"mut": ("filter_own",), "result": fuzz(["REF"], 150, 120, FUZZ, ("filter_own",))})
    if mode in ("geometry", "all"):
        for shape in ("G1", "G2", "G3", "G4", "G5"):
            for n in (1000, 4000):
                for row in geometry(shape, n, ["JIA", "YI_min", "YI_move", "BING_A", "BING_B", "BING_R", "BING_T"], (1, 2, 3)):
                    emit("geometry", row)
    if mode in ("table", "all"):
        for row in width_table():
            emit("width", row)
    print(json.dumps({"section": "END", "rows": len(rows)}))


if __name__ == "__main__":
    main()
