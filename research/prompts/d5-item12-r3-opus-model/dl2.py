#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12（deadlist 条目的形态） 第二轮攻方腿模型，只用 Python 标准库。
# 攻击面：级联按意图分批（D8 已定项 5）与崩溃 / 回退；分批的级联与另一次销毁、建快照、克隆交错；
# 克隆与给克隆建快照落在 origin 那个 checkpoint；同一个头第二次杀同一块（计数 second_kill）。
# 真值 = D5 前提二：块被引用 <=> 在某个头的活集合或某个活快照的捕获集合里；被销毁的快照从写意图那一刻起不再引用。
# 多放任何时刻都判；漏放只在没有未完成意图的时刻判（意图未完成时「该放未放」是合法的中间态，I-4.6）。
import sys, random, pickle, json
from collections import deque

INF = 1 << 62
# 强配置：每个旋钮取本腿认为最强的读法；变体一次只改一个旋钮（VARIANTS）。
STRONG = {"inherit": "intent",    # prev 传递在写意图的那个事务里做（end：最后一批时做）
          "order": "fifo",        # 意图按提交序一个接一个做（any：任意交错）
          "hi": "fixed",          # 丙-区间：意图记过滤区间上界，下一侧是头时取写意图的 checkpoint 号（inf：[S.txg, ∞)）
          "scan": "rescan",       # 原地过滤：每批重找区间里 birth > P 的最小键（cursor：带游标顺扫）
          "relabel": "intent",    # 乙：换号在写意图时做（end：最后一批时做）
          "merge": "min",         # 乙：改键条数少的那一份（moveS / moveN：固定方向）
          "first_seg": "origin",  # 丙′：克隆头的第一段装 origin 的树 ID（zero：装 0）
          "mut": ()}              # 变异：noprop（不传递 prev）、ge（删除判据写成 birth ≥ prev）


def clone_state(value):
    return pickle.loads(pickle.dumps(value, protocol=pickle.HIGHEST_PROTOCOL))


class Block:
    __slots__ = ("bid", "cls", "btree", "birth", "wseq")

    def __init__(self, bid, cls, btree, birth, wseq):
        self.bid, self.cls, self.btree, self.birth, self.wseq = bid, cls, btree, birth, wseq

    def __getstate__(self):
        return (self.bid, self.cls, self.btree, self.birth, self.wseq)

    def __setstate__(self, state):
        self.bid, self.cls, self.btree, self.birth, self.wseq = state

    def mapkey(self):   # 中央映射 key 的段序（D19 已定项 6）：类标签, 出生树, 出生 txg, 写序
        return (self.cls, self.btree, self.birth, self.wseq)


class World:
    """池的结构状态：头、快照、块。与臂无关；臂只决定放哪些块、deadlist 里写什么键。"""

    def __init__(self):
        self.txg, self.next_tid, self.next_bid = 1, 11, 0
        self.blocks, self.heads, self.snaps, self.mk2bid = {}, {}, {}, {}
        self.root = self.alloc_tid()
        self.heads[self.root] = {"live": set(), "origin": None, "wseq": 0}

    def alloc_tid(self):
        tid = self.next_tid
        self.next_tid += 1
        return tid

    def alive_snaps(self):
        return sorted(self.snaps, key=lambda sid: (self.snaps[sid]["txg"], sid))   # (诞生 txg, 树 ID) 全序

    def head_snaps(self, head):
        return [sid for sid in self.alive_snaps() if self.snaps[sid]["head"] == head]

    def next_owner(self, sid):
        head = self.snaps[sid]["head"]
        lineage = self.head_snaps(head)
        position = lineage.index(sid)
        return ("s", lineage[position + 1]) if position + 1 < len(lineage) else ("h", head)

    def clones_of(self, sid):
        return [head for head in self.heads if self.heads[head]["origin"] == sid]

    def referenced(self, bid):
        return any(bid in state["live"] for state in self.heads.values()) or \
            any(bid in state["cap"] for state in self.snaps.values())

    def lineage_members(self, head):
        origin = self.heads[head]["origin"]
        members = [("o", origin)] if origin is not None else []
        return members + [("s", sid) for sid in self.head_snaps(head)] + [("h", head)]

    def world_prev_txg(self, owner):
        kind, ident = owner
        head = ident if kind == "h" else self.snaps[ident]["head"]
        members = self.lineage_members(head)
        position = members.index(owner)
        return 0 if position == 0 else self.snaps[members[position - 1][1]]["txg"]

    def owner_txg(self, owner):
        return self.snaps[owner[1]]["txg"] if owner[0] == "s" else INF


class KeySpace:
    """一棵定宽 key 的树：集合语义（同 key 第二次插入折叠成一条），value 空。"""

    def __init__(self):
        self.entries, self.collapse = {}, 0

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
    """三条臂共用：删除判定、交接、prev 传递、意图队列与每批的调度。"""
    name = "BASE"

    def __init__(self, world, cfg):
        self.cfg = dict(STRONG, **cfg)
        self.freed, self.double, self.intents, self.seq, self.killed = set(), 0, [], 0, set()
        self.ph, self.ps = {world.root: 0}, {}
        self.ks = KeySpace()
        self.m = dict(moves=0, frees=0, units=0, count_reads=0, second_kill=0, max_intents=0)

    def free(self, bid):
        if bid in self.freed:
            self.double += 1
        self.freed.add(bid)
        self.m["frees"] += 1

    def release_key(self, world, key):
        self.ks.delete(key)
        self.free(world.mk2bid[key[-4:]])

    def rekey(self, key, new_key):
        self.ks.delete(key)
        self.ks.insert(new_key)
        self.m["moves"] += 1

    def kill(self, world, head, block):
        if (head, block.bid) in self.killed:
            self.m["second_kill"] += 1
        self.killed.add((head, block.bid))
        threshold = self.ph[head]
        if block.birth > threshold or ("ge" in self.cfg["mut"] and block.birth == threshold):
            self.free(block.bid)
        else:
            self.dl_append(world, head, block)

    def dl_append(self, world, head, block):
        self.ks.insert(self.kill_key(head, block))

    def snapshot(self, world, head, sid):
        self.ps[sid] = self.ph[head]
        self.on_snapshot(world, head, sid)
        self.ph[head] = world.snaps[sid]["txg"]

    def clone(self, world, origin_sid, new_head):
        self.ph[new_head] = world.snaps[origin_sid]["txg"]
        self.on_clone(world, origin_sid, new_head)

    def stored_prev(self, owner):
        return self.ph[owner[1]] if owner[0] == "h" else self.ps[owner[1]]

    def set_prev(self, owner, value):
        table = self.ph if owner[0] == "h" else self.ps
        if owner[1] in table and "noprop" not in self.cfg["mut"]:
            table[owner[1]] = value

    def destroy_snap(self, world, sid, info):
        prev_txg = self.ps.pop(sid)
        intent = self.make_cascade(world, sid, info, prev_txg)
        intent.update(kind="cascade", P=prev_txg, next=info["next"], seq=self.seq)
        self.seq += 1
        if self.cfg["inherit"] == "intent":
            self.inherit(intent)
        self.intents.append(intent)
        self.m["max_intents"] = max(self.m["max_intents"], len(self.intents))

    def inherit(self, intent):
        self.set_prev(intent["next"], intent["P"])
        self.inherit_extra(intent)

    def inherit_extra(self, intent):
        pass

    def destroy_head(self, world, head, info):
        intent = dict(kind="drophead", head=head, frees=sorted(info["livelist"]), seq=self.seq)
        intent.update(self.drop_handle(head))
        self.seq += 1
        del self.ph[head]
        self.intents.append(intent)

    def drop_handle(self, head):
        return {}

    def drop_keys(self, intent):   # 丙-区间 / 丙′：按头前缀范围删（乙改写）
        return self.ks.range((intent["head"],), (intent["head"] + 1,))

    def run_unit(self, world, index):
        """一批 = 处理一条条目（或一块 livelist 块）；做完就在同一批里删掉意图（D8 已定项 5）。"""
        intent = self.intents[index]
        self.m["units"] += 1
        if intent["kind"] == "drophead":
            if intent["frees"]:
                self.free(intent["frees"].pop(0))
            else:
                keys = self.drop_keys(intent)
                if keys:
                    self.ks.delete(keys[0])
        else:
            self.cascade_unit(world, intent)
        if self.intent_done(intent):
            self.intents.pop(index)
            if intent["kind"] == "cascade":
                self.finish_cascade(intent)

    def finish_cascade(self, intent):
        if self.cfg["inherit"] == "end":
            self.inherit(intent)

    def intent_done(self, intent):
        if intent["kind"] == "drophead":
            return not intent["frees"] and not self.drop_keys(intent)
        return self.cascade_done(intent)


class Ref(ArmBase):
    """阳性对照：deadlist 是挂在头 / 快照对象上的集合，交接换指针，级联与销毁头一次做完（不分批）。"""
    name = "REF"

    def __init__(self, world, cfg):
        super().__init__(world, cfg)
        self.dl = {("h", world.root): set()}

    def dl_append(self, world, head, block):
        self.dl[("h", head)].add(block.bid)

    def on_snapshot(self, world, head, sid):
        self.dl[("s", sid)] = self.dl[("h", head)]
        self.dl[("h", head)] = set()

    def on_clone(self, world, origin_sid, new_head):
        self.dl[("h", new_head)] = set()

    def destroy_snap(self, world, sid, info):
        prev_txg, next_owner = self.ps.pop(sid), info["next"]
        own = self.dl.pop(("s", sid))
        released = {bid for bid in self.dl[next_owner] if world.blocks[bid].birth > prev_txg}
        self.dl[next_owner] = (self.dl[next_owner] - released) | own
        for bid in sorted(released):
            self.free(bid)
        self.set_prev(next_owner, prev_txg)

    def destroy_head(self, world, head, info):
        for bid in info["livelist"]:
            self.free(bid)
        self.dl.pop(("h", head))
        del self.ph[head]

    def resolve_all(self, world):
        return [(("own", owner), bid) for owner, bids in self.dl.items() for bid in bids]


class BingRange(ArmBase):
    """丙-区间：key = 头的树 ID 8 + 纪元 8 + 映射 key 27；纪元 = 杀块那一刻头的 previous_snapshot_txg。
    X 的 deadlist = 纪元落在 [prev(X), X.txg) 的条目，头的 = [prev(头), ∞)。级联只放不改键。"""
    name = "BING_R"

    def kill_key(self, head, block):
        return (head, self.ph[head]) + block.mapkey()

    def on_snapshot(self, world, head, sid):
        pass

    def on_clone(self, world, origin_sid, new_head):
        pass

    def make_cascade(self, world, sid, info, prev_txg):
        if info["next"][0] == "s":
            upper = info["next_txg"]
        else:
            upper = info["commit_txg"] if self.cfg["hi"] == "fixed" else INF
        return dict(head=info["head"], lo=info["s_txg"], hi=upper, cursor=None)

    def candidates(self, intent):
        keys = self.ks.range((intent["head"], intent["lo"]), (intent["head"], intent["hi"]))
        if self.cfg["scan"] == "cursor":
            return [key for key in keys if intent["cursor"] is None or key > intent["cursor"]]
        return [key for key in keys if birth_of(key) > intent["P"]]

    def cascade_unit(self, world, intent):
        rest = self.candidates(intent)
        if rest:
            intent["cursor"] = rest[0]
            if birth_of(rest[0]) > intent["P"]:
                self.release_key(world, rest[0])

    def cascade_done(self, intent):
        return not self.candidates(intent)

    def owner_of(self, world, head, epoch):
        if head not in world.heads:
            return None
        owners = [("s", sid) for sid in world.head_snaps(head)] + [("h", head)]
        hit = [owner for owner in owners if self.stored_prev(owner) <= epoch < world.owner_txg(owner)]
        return hit[0] if len(hit) == 1 else None

    def resolve_all(self, world):
        """checker 的读法（不走臂的执行代码）：未完成意图会放掉的判 free，被销毁头的判 drop，其余按区间认主。"""
        result = []
        for key in self.ks.entries:
            verdict = None
            for intent in self.intents:
                if intent["kind"] == "drophead" and intent["head"] == key[0]:
                    verdict = ("drop", None)
                elif (intent["kind"] == "cascade" and intent["head"] == key[0] and intent["lo"] <= key[1] < intent["hi"]
                      and birth_of(key) > intent["P"]
                      and (self.cfg["scan"] == "rescan" or intent["cursor"] is None or key > intent["cursor"])):
                    verdict = ("free", None)
                if verdict:
                    break
            if verdict is None:
                owner = self.owner_of(world, key[0], key[1])
                verdict = ("own", owner) if owner else ("orphan", None)
            result.append((verdict, world.mk2bid[key[-4:]]))
        return result


class BingTid(ArmBase):
    """丙′：key = 头的树 ID 8 + 杀块那一刻头的上一个快照的树 ID 8 + 映射 key 27；克隆头第一段装 origin 的树 ID。
    销毁 S：下一侧那一段 birth > prev(S) 的放掉，其余改键到 S 的上一个快照的树 ID；下一侧继承 prev(S) 与那个树 ID。"""
    name = "BING_T"

    def __init__(self, world, cfg):
        super().__init__(world, cfg)
        self.pth, self.pts = {world.root: 0}, {}

    def kill_key(self, head, block):
        return (head, self.pth[head]) + block.mapkey()

    def seg_of(self, owner):
        return self.pth[owner[1]] if owner[0] == "h" else self.pts[owner[1]]

    def on_snapshot(self, world, head, sid):
        self.pts[sid] = self.pth[head]
        self.pth[head] = sid

    def on_clone(self, world, origin_sid, new_head):
        self.pth[new_head] = origin_sid if self.cfg["first_seg"] == "origin" else 0

    def make_cascade(self, world, sid, info, prev_txg):
        return dict(head=info["head"], src=self.seg_of(info["next"]), dst=self.pts.pop(sid))

    def inherit_extra(self, intent):
        owner = intent["next"]
        table = self.pth if owner[0] == "h" else self.pts
        if owner[1] in table and "noprop" not in self.cfg["mut"]:
            table[owner[1]] = intent["dst"]

    def segment(self, intent):
        return self.ks.range((intent["head"], intent["src"]), (intent["head"], intent["src"] + 1))

    def cascade_unit(self, world, intent):
        keys = self.segment(intent)
        if keys:
            if birth_of(keys[0]) > intent["P"]:
                self.release_key(world, keys[0])
            else:
                self.rekey(keys[0], (intent["head"], intent["dst"]) + keys[0][2:])

    def cascade_done(self, intent):
        return not self.segment(intent)

    def destroy_head(self, world, head, info):
        super().destroy_head(world, head, info)
        del self.pth[head]

    def resolve_all(self, world):
        """checker 的读法：按意图的提交序把条目一段段送下去（先进先出的合成），再按段号认主。"""
        result = []
        for key in self.ks.entries:
            head, seg, verdict = key[0], key[1], None
            for intent in self.intents:
                if intent["kind"] == "drophead" and intent["head"] == head:
                    verdict = ("drop", None)
                    break
                if intent["kind"] == "cascade" and intent["head"] == head and intent["src"] == seg:
                    if birth_of(key) > intent["P"]:
                        verdict = ("free", None)
                        break
                    seg = intent["dst"]
            if verdict is None:
                hit = []
                if head in world.heads:
                    owners = [("s", sid) for sid in world.head_snaps(head)] + [("h", head)]
                    hit = [owner for owner in owners if self.seg_of(owner) == seg]
                verdict = ("own", hit[0]) if len(hit) == 1 else ("orphan", None)
            result.append((verdict, world.mk2bid[key[-4:]]))
        return result


class Yi(ArmBase):
    """乙：头 / 快照的树表条目预留里 8 字节放 deadlist 号；key = 号 8 + 映射 key 27。建快照时新快照拿走头的号、
    头换成新快照的树 ID。销毁 S：下一侧那一份 birth > prev(S) 的放掉，合并时改键条数少的那一份，号跟着换。"""
    name = "YI"

    def __init__(self, world, cfg):
        super().__init__(world, cfg)
        self.number = {("h", world.root): world.root}

    def kill_key(self, head, block):
        return (self.number[("h", head)],) + block.mapkey()

    def on_snapshot(self, world, head, sid):
        self.number[("s", sid)] = self.number[("h", head)]
        self.number[("h", head)] = sid

    def on_clone(self, world, origin_sid, new_head):
        self.number[("h", new_head)] = new_head

    def keys_of(self, number):
        return self.ks.range((number,), (number + 1,))

    def make_cascade(self, world, sid, info, prev_txg):
        next_owner = info["next"]
        own_number, next_number = self.number.pop(("s", sid)), self.number[next_owner]
        own = len(self.keys_of(own_number))
        kept = sum(1 for key in self.keys_of(next_number) if birth_of(key) <= prev_txg)
        self.m["count_reads"] += own + len(self.keys_of(next_number))   # 挑少的那一边要先数两份（盘上没有计数器）
        if self.cfg["merge"] == "moveS" or (self.cfg["merge"] == "min" and own <= kept):
            return dict(src=own_number, dst=next_number, filt=next_number, cursor=None, phase="filter", relabel=None)
        if self.cfg["relabel"] == "intent":
            self.number[next_owner] = own_number
        return dict(src=next_number, dst=own_number, filt=None, cursor=None, phase="move",
                    relabel=(next_number, own_number))

    def filter_candidates(self, intent):
        keys = self.keys_of(intent["filt"])
        if self.cfg["scan"] == "cursor":
            return [key for key in keys if intent["cursor"] is None or key > intent["cursor"]]
        return [key for key in keys if birth_of(key) > intent["P"]]

    def cascade_unit(self, world, intent):
        if intent["phase"] == "filter":
            rest = self.filter_candidates(intent)
            if rest:
                intent["cursor"] = rest[0]
                if birth_of(rest[0]) > intent["P"]:
                    self.release_key(world, rest[0])
                return
            intent["phase"] = "move"
        keys = self.keys_of(intent["src"])
        if keys:
            if intent["filt"] is None and birth_of(keys[0]) > intent["P"]:
                self.release_key(world, keys[0])
            else:
                self.rekey(keys[0], (intent["dst"],) + keys[0][1:])

    def cascade_done(self, intent):
        filter_left = intent["phase"] == "filter" and self.filter_candidates(intent)
        return not filter_left and not self.keys_of(intent["src"])

    def finish_cascade(self, intent):
        super().finish_cascade(intent)
        if intent["relabel"] and self.cfg["relabel"] == "end" and self.number.get(intent["next"]) == intent["relabel"][0]:
            self.number[intent["next"]] = intent["relabel"][1]

    def drop_handle(self, head):
        return {"num": self.number.pop(("h", head))}

    def drop_keys(self, intent):
        return self.keys_of(intent["num"])

    def resolve_all(self, world):
        """checker 的读法：按意图的提交序合成——过滤号里 birth > P 的判 free，被搬的跟着搬到目的号，最后按号认主。"""
        owner_of = {number: owner for owner, number in self.number.items()}
        for intent in self.intents:   # 换号放在最后一批时，目的号在意图做完之前归下一侧
            if intent["kind"] == "cascade" and intent["relabel"] and self.cfg["relabel"] == "end":
                if self.number.get(intent["next"]) == intent["relabel"][0]:
                    owner_of[intent["relabel"][1]] = intent["next"]
        result = []
        for key in self.ks.entries:
            number, verdict = key[0], None
            for intent in self.intents:
                if intent["kind"] == "drophead":
                    if intent["num"] == number:
                        verdict = ("drop", None)
                        break
                    continue
                if (intent["filt"] == number and intent["phase"] == "filter" and birth_of(key) > intent["P"]
                        and (self.cfg["scan"] == "rescan" or intent["cursor"] is None or key > intent["cursor"])):
                    verdict = ("free", None)
                    break
                if intent["src"] == number:
                    if intent["filt"] is None and birth_of(key) > intent["P"]:
                        verdict = ("free", None)
                        break
                    number = intent["dst"]
            if verdict is None:
                owner = owner_of.get(number)
                verdict = ("own", owner) if owner else ("orphan", None)
            result.append((verdict, world.mk2bid[key[-4:]]))
        return result


ARMS = {"REF": Ref, "BING_R": BingRange, "BING_T": BingTid, "YI": Yi}


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
    """U4 的检查（读意图，不走臂的执行代码），红的返回名字：pend（未完成意图将放掉一个仍被引用的块）、orphan（条目认不出主）、
    vector（每份 deadlist 与现算的期望逐份相等）、i36（birth ≤ 从世界现算的 prev(X)）、c195（臂存的 prev 等于现算的 prev）。"""
    red, decoded = set(), {}
    for (kind, owner), bid in arm.resolve_all(world):
        if kind == "free" and world.referenced(bid):
            red.add("pend")
        elif kind == "orphan":
            red.add("orphan")
        elif kind == "own":
            decoded.setdefault(owner, set()).add(bid)
    expected = expected_deadlists(world)
    if any(decoded.get(owner, set()) != expected.get(owner, set()) for owner in set(decoded) | set(expected)):
        red.add("vector")
    for owner, bids in decoded.items():
        if any(world.blocks[bid].birth > world.world_prev_txg(owner) for bid in bids):
            red.add("i36")
    for head in world.heads:
        for owner in world.lineage_members(head):
            if owner[0] != "o" and arm.stored_prev(owner) != world.world_prev_txg(owner):
                red.add("c195")
    return sorted(red)


def oracle(world, arm):
    """真值：返回（多放的块，重复放的次数，漏放的块）；漏放只在臂没有未完成意图时判。"""
    over = sorted(bid for bid in arm.freed if world.referenced(bid))
    leak = [] if arm.intents else sorted(bid for bid in world.blocks if not world.referenced(bid) and bid not in arm.freed)
    return over, arm.double, leak


def new_state(arm_specs, keep_ring=True):
    world = World()
    arms = [ARMS[name](world, dict(cfg)) for name, cfg in arm_specs]
    return {"world": world, "arms": arms, "ring": [], "keep_ring": keep_ring}


def publish(state):
    """发布：本 checkpoint 的全部改动连同意图原子落盘；根环留最近 3 个根（崩溃 / 回退从这里取）。"""
    world = state["world"]
    published_txg = world.txg
    world.txg += 1
    if state["keep_ring"]:
        state["ring"].append({"copy": clone_state((world, state["arms"])), "txg": published_txg,
                              "tid": world.next_tid, "abandoned": False})
        state["ring"] = state["ring"][-3:]


def restore(state, entry):
    """崩溃挂回 entry 那个根：开放 checkpoint 里的改动全丢；新 txg = 根环全部根 txg 的 max + 1，树 ID 水位取 max（D8 已定项 8 ②）。"""
    world, arms = clone_state(entry["copy"])
    world.txg = max(item["txg"] for item in state["ring"]) + 1
    world.next_tid = max(item["tid"] for item in state["ring"])
    state["world"], state["arms"] = world, arms


def take_snapshot(world, arms, head):
    sid = world.alloc_tid()
    world.snaps[sid] = {"head": head, "txg": world.txg, "cap": frozenset(world.heads[head]["live"])}
    for arm in arms:
        arm.snapshot(world, head, sid)
    return sid


def make_clone(world, arms, origin_sid):
    new_head = world.alloc_tid()
    world.heads[new_head] = {"live": set(world.snaps[origin_sid]["cap"]), "origin": origin_sid, "wseq": 0}
    for arm in arms:
        arm.clone(world, origin_sid, new_head)
    return new_head


def step(state, op):
    """操作：W 写块、K 杀块、P / PS / PSS / PSC 发布（PS 建一个快照，PSS 同一 checkpoint 建两个，PSC 建快照 O、从 O 克隆、
    给克隆建快照，全在同一个 checkpoint 末尾）、C 从快照克隆、D 销毁快照（写意图）、X 销毁头（写意图）、B 跑一批、
    CR 崩溃挂回最新的根、RB 回退到上一个根（最新的根作废）。"""
    world, arms, kind = state["world"], state["arms"], op[0]
    if kind == "W":
        head_state = world.heads[op[1]]
        head_state["wseq"] += 1
        block = Block(world.next_bid, op[2], op[1], world.txg, head_state["wseq"])
        world.next_bid += 1
        world.blocks[block.bid] = block
        world.mk2bid[block.mapkey()] = block.bid
        head_state["live"].add(block.bid)
    elif kind == "K":
        world.heads[op[1]]["live"].discard(op[2])
        for arm in arms:
            arm.kill(world, op[1], world.blocks[op[2]])
    elif kind in ("P", "PS", "PSS", "PSC"):
        if kind != "P":
            origin = take_snapshot(world, arms, op[1])
        if kind == "PSS":
            take_snapshot(world, arms, op[1])
        if kind == "PSC":
            take_snapshot(world, arms, make_clone(world, arms, origin))
        publish(state)
    elif kind == "C":
        make_clone(world, arms, op[1])
    elif kind == "D":
        sid = op[1]
        next_owner = world.next_owner(sid)
        info = {"head": world.snaps[sid]["head"], "next": next_owner, "s_txg": world.snaps[sid]["txg"],
                "next_txg": world.owner_txg(next_owner), "commit_txg": world.txg}
        del world.snaps[sid]
        for arm in arms:
            arm.destroy_snap(world, sid, info)
    elif kind == "X":
        head_state = world.heads.pop(op[1])
        origin_txg = 0 if head_state["origin"] is None else world.snaps[head_state["origin"]]["txg"]
        info = {"livelist": [bid for bid in head_state["live"] if world.blocks[bid].birth > origin_txg]}
        for arm in arms:
            arm.destroy_head(world, op[1], info)
    elif kind == "B":
        for arm in arms:
            if arm.intents:
                arm.run_unit(world, 0 if arm.cfg["order"] == "fifo" else op[1] % len(arm.intents))
    elif kind == "CR":
        restore(state, [entry for entry in state["ring"] if not entry["abandoned"]][-1])
    elif kind == "RB":
        alive = [entry for entry in state["ring"] if not entry["abandoned"]]
        alive[-1]["abandoned"] = True
        restore(state, alive[-2])
    else:
        raise ValueError(op)


def legal_ops(state, limits, classes=(1,)):
    world, arms, ops = state["world"], state["arms"], []
    for head in sorted(world.heads):
        if len(world.blocks) < limits["blocks"]:
            ops += [("W", head, cls) for cls in classes]
        ops += [("K", head, bid) for bid in sorted(world.heads[head]["live"])]
    if world.txg < limits["txg"]:
        ops.append(("P",))
        for head in sorted(world.heads):
            if len(world.snaps) + 1 <= limits["snaps"]:
                ops.append(("PS", head))
            if limits.get("pss") and len(world.snaps) + 2 <= limits["snaps"]:
                ops.append(("PSS", head))
            if limits.get("psc") and len(world.snaps) + 2 <= limits["snaps"] and len(world.heads) < limits["heads"]:
                ops.append(("PSC", head))
    for sid in world.alive_snaps():
        if not world.clones_of(sid):                       # 分叉点闸：children 数克隆（D5 第 2 条）
            ops.append(("D", sid))
        if limits.get("clone") and len(world.heads) < limits["heads"]:
            ops.append(("C", sid))
    for head in sorted(world.heads):
        if limits.get("x", True) and head != world.root and not world.head_snaps(head):   # 第二道闸：有自己活快照的头不许销毁
            ops.append(("X", head))
    pending = max(len(arm.intents) for arm in arms)
    if pending:
        wide = any(arm.cfg["order"] == "any" for arm in arms)
        ops += [("B", index) for index in (range(pending) if wide else [0])]
    if limits.get("crash") and state["ring"]:
        ops.append(("CR",))
        if sum(1 for entry in state["ring"] if not entry["abandoned"]) >= 2:
            ops.append(("RB",))
    return ops


def world_sig(world):
    heads = tuple(sorted((head, tuple(sorted(s["live"])), s["origin"], s["wseq"]) for head, s in world.heads.items()))
    snaps = tuple(sorted((sid, s["head"], s["txg"], tuple(sorted(s["cap"]))) for sid, s in world.snaps.items()))
    blocks = tuple(sorted((b.bid, b.cls, b.btree, b.birth, b.wseq) for b in world.blocks.values()))
    return (world.txg, world.next_tid, world.next_bid, heads, snaps, blocks)


def arm_sig(arm):
    parts = [tuple(sorted(arm.freed)), arm.double, tuple(sorted(arm.ph.items())), tuple(sorted(arm.ps.items())),
             tuple(sorted(arm.ks.entries)), repr(arm.intents)]
    for attribute in ("dl", "number", "pth", "pts"):
        if hasattr(arm, attribute):
            parts.append(repr(sorted((key, sorted(value) if isinstance(value, set) else value)
                                     for key, value in getattr(arm, attribute).items())))
    return tuple(parts)


def state_sig(state):
    sig = (world_sig(state["world"]), tuple(arm_sig(arm) for arm in state["arms"]))
    if state["keep_ring"]:
        sig += (tuple((entry["txg"], entry["tid"], entry["abandoned"], world_sig(entry["copy"][0]),
                       tuple(arm_sig(arm) for arm in entry["copy"][1])) for entry in state["ring"]),)
    return sig


def bfs_shortest(arm_name, cfg, limits, max_depth, budget=300000):
    """按层 BFS（状态去重），返回第一段让真值判红的历史——即这个取样范围里最短的一段。"""
    state = new_state([(arm_name, cfg)], keep_ring=bool(limits.get("crash")))
    queue, seen, explored = deque([(state, ())]), {state_sig(state)}, 0
    while queue:
        state, path = queue.popleft()
        if len(path) >= max_depth:
            continue
        for op in legal_ops(state, limits):
            child = clone_state(state)
            step(child, op)
            explored += 1
            over, double, leak = oracle(child["world"], child["arms"][0])
            if over or double or leak:
                return {"arm": arm_name, "cfg": cfg, "found": True, "path": path + (op,), "over": over,
                        "double": double, "leak": leak, "explored": explored}
            signature = state_sig(child)
            if signature not in seen:
                seen.add(signature)
                queue.append((child, path + (op,)))
            if explored >= budget:
                return {"arm": arm_name, "cfg": cfg, "found": False, "explored": explored,
                        "depth_reached": len(path), "truncated": True}
    return {"arm": arm_name, "cfg": cfg, "found": False, "explored": explored, "depth_reached": max_depth,
            "truncated": False, "states": len(seen)}


def label(name, cfg):
    knobs = [f"{key}={value}" for key, value in sorted(cfg.items()) if key != "mut"] + (["mut=" + "+".join(cfg["mut"])] if cfg.get("mut") else [])
    return name + ("[" + ",".join(knobs) + "]" if knobs else "")


def replay_summary(arm_specs, path):
    """同一段历史喂给几条臂：每条臂真值第一次判红、检查第一次判红各在第几步。"""
    state, result = new_state(arm_specs), {}
    for number, op in enumerate(path, 1):
        step(state, tuple(op))
        for (name, cfg), arm in zip(arm_specs, state["arms"]):
            entry = result.setdefault(label(name, cfg), {"bad": None, "red": None})
            over, double, leak = oracle(state["world"], arm)
            if entry["bad"] is None and (over or double or leak):
                entry["bad"] = {"step": number, "over": over, "double": double, "leak": leak}
            red = checker(state["world"], arm)
            if entry["red"] is None and red:
                entry["red"] = {"step": number, "red": red}
    for (name, cfg), arm in zip(arm_specs, state["arms"]):   # 历史走完后按提交序把剩下的意图做完，再判一次
        while arm.intents:
            arm.run_unit(state["world"], 0)
        entry = result[label(name, cfg)]
        over, double, leak = oracle(state["world"], arm)
        if entry["bad"] is None and (over or double or leak):
            entry["bad"] = {"step": "drain", "over": over, "double": double, "leak": leak}
    return result


def trace(arm_specs, path):
    """逐步打印每条臂的 deadlist 键、prev、意图数、放掉的块、真值与检查——报告里的历史表由它核。"""
    state, lines = new_state(arm_specs), []
    for number, op in enumerate(path, 1):
        step(state, tuple(op))
        for (name, cfg), arm in zip(arm_specs, state["arms"]):
            over, double, leak = oracle(state["world"], arm)
            keys = sorted(arm.ks.entries) if name != "REF" else sorted((o, sorted(v)) for o, v in arm.dl.items())
            lines.append(f"{number} {tuple(op)} | {label(name, cfg)} | dl={keys} | ph={sorted(arm.ph.items())} "
                         f"ps={sorted(arm.ps.items())} | intents={len(arm.intents)} | freed={sorted(arm.freed)} "
                         f"| over={over} leak={leak} | checker={checker(state['world'], arm)}")
    return lines


def drain(state, rng, limit=100000):
    for _ in range(limit):
        if not any(arm.intents for arm in state["arms"]):
            return True
        step(state, ("B", rng.randrange(1 << 16)))
    return False


WEIGHTS = {"W": 4.0, "K": 3.0, "P": 1.0, "PS": 1.5, "PSS": 0.4, "PSC": 0.3, "C": 0.4, "D": 1.5, "X": 0.3,
           "B": 2.5, "CR": 0.3, "RB": 0.15}


def fuzz(arm_specs, seeds, length, limits, classes=(1, 2, 3)):
    """随机历史（含崩溃 / 回退）：每步判真值与检查；末尾把意图做完再判漏放。red_not_bad = 检查误报的历史数。"""
    labels = [label(name, cfg) for name, cfg in arm_specs]
    summary = {lab: dict(oracle_bad=0, checker_red=0, red_not_bad=0, bad_not_red=0, red_after_bad=0,
                         first=None, second_kill=0, collapse=0) for lab in labels}
    for seed in range(seeds):
        rng = random.Random(seed)
        state = new_state(arm_specs)
        bad, red = [None] * len(labels), [None] * len(labels)
        for index in range(length + 1):
            if index < length:
                ops = legal_ops(state, limits, classes)
                kinds = sorted({op[0] for op in ops})
                kind = rng.choices(kinds, [WEIGHTS[k] for k in kinds])[0]
                op = rng.choice([candidate for candidate in ops if candidate[0] == kind])
                step(state, ("B", rng.randrange(1 << 16)) if kind == "B" else op)
            else:
                drain(state, rng)
            for position, arm in enumerate(state["arms"]):
                over, double, leak = oracle(state["world"], arm)
                if bad[position] is None and (over or double or leak):
                    bad[position] = index
                    if summary[labels[position]]["first"] is None:
                        summary[labels[position]]["first"] = {"seed": seed, "step": index, "over": len(over),
                                                              "double": double, "leak": len(leak)}
                if red[position] is None and checker(state["world"], arm):
                    red[position] = index
        for position, arm in enumerate(state["arms"]):
            entry = summary[labels[position]]
            entry["oracle_bad"] += bad[position] is not None
            entry["checker_red"] += red[position] is not None
            entry["red_not_bad"] += red[position] is not None and bad[position] is None
            entry["bad_not_red"] += bad[position] is not None and red[position] is None
            entry["red_after_bad"] += bad[position] is not None and red[position] is not None and red[position] > bad[position]
            entry["second_kill"] += arm.m["second_kill"]
            entry["collapse"] += arm.ks.collapse
    return summary


def geometry_ops(shape, n, classes=(1, 2, 3)):
    """U2 的几何（根头 11；快照依次 12 / 13 / 14…；销毁中间那个 13）。G6：级联进头还没做完时，头又建 3 个快照、每个之后杀一段。"""
    root, writes = 11, [("W", 11, classes[i % len(classes)]) for i in range(n)]

    def kills(low, high):
        return [("K", root, bid) for bid in range(low, high)]
    if shape == "G1":
        return writes + [("PS", root)] + kills(0, n) + [("PS", root)]
    if shape == "G2":
        return writes + [("PS", root), ("PS", root)] + kills(0, n) + [("PS", root), ("D", 13)]
    if shape == "G3":
        return writes + [("PS", root)] + kills(0, n) + [("PS", root), ("PS", root), ("D", 13)]
    if shape == "G4":
        return writes + [("PS", root)] + kills(0, n // 2) + [("PS", root)] + kills(n // 2, n) + [("PS", root), ("D", 13)]
    if shape == "G5":
        return writes + [("PS", root)] + writes + [("PS", root)] + kills(0, 2 * n) + [("PS", root), ("D", 13)]
    if shape == "G6":
        ops = writes + [("PS", root)] + kills(0, n // 2) + [("PS", root)] + kills(n // 2, n) + [("D", 13)]
        for round_index in range(3):
            base = n + round_index * (n // 4)
            ops += [("W", root, 1)] * (n // 4) + [("PS", root)] + kills(base, base + n // 4)
        return ops
    raise ValueError(shape)


def geometry(shape, n, arm_specs):
    state = new_state(arm_specs, keep_ring=False)
    for op in geometry_ops(shape, n):
        step(state, op)
    drain(state, random.Random(0))
    rows = []
    for (name, cfg), arm in zip(arm_specs, state["arms"]):
        over, double, leak = oracle(state["world"], arm)
        rows.append(dict(shape=shape, n=n, arm=label(name, cfg), over=len(over), double=double, leak=len(leak), **arm.m))
    return rows


VARIANTS = [("REF", {}),
            ("BING_R", {}), ("BING_R", {"hi": "inf"}), ("BING_R", {"scan": "cursor"}), ("BING_R", {"inherit": "end"}),
            ("BING_R", {"order": "any"}),
            ("YI", {}), ("YI", {"order": "any"}), ("YI", {"scan": "cursor"}), ("YI", {"inherit": "end"}),
            ("YI", {"relabel": "end"}), ("YI", {"merge": "moveS"}), ("YI", {"merge": "moveN"}),
            ("BING_T", {}), ("BING_T", {"order": "any"}), ("BING_T", {"inherit": "end"}), ("BING_T", {"first_seg": "zero"})]
STRONG_ARMS = [("REF", {}), ("BING_R", {}), ("YI", {}), ("BING_T", {})]
LIM_BATCH = {"blocks": 2, "heads": 1, "snaps": 2, "txg": 3}
LIM_CLONE = {"blocks": 2, "heads": 2, "snaps": 3, "txg": 3, "clone": True, "psc": True, "pss": True}
LIM_CRASH = {"blocks": 2, "heads": 1, "snaps": 2, "txg": 3, "crash": True}
LIM_BIG = {"blocks": 3, "heads": 1, "snaps": 3, "txg": 4, "pss": True}
LIM_CLONE_BIG = {"blocks": 2, "heads": 2, "snaps": 3, "txg": 4, "clone": True, "psc": True, "pss": True}
BIG_SPECS = [("BING_R", {}), ("BING_R", {"order": "any"}), ("YI", {}), ("BING_T", {})]
FUZZ = {"blocks": 30, "heads": 4, "snaps": 6, "txg": 150, "clone": True, "psc": True, "pss": True, "crash": True}
HISTORIES = {
    "H1 丙-区间过滤上界取 ∞：级联没做完时头建快照再杀块": [("W", 11, 1), ("PS", 11), ("D", 12), ("PS", 11), ("K", 11, 0), ("B", 0)],
    "H2 两次级联交错（后一次先做完）": [("W", 11, 1), ("W", 11, 1), ("PS", 11), ("K", 11, 0), ("PS", 11), ("K", 11, 1),
                                 ("D", 13), ("D", 12), ("B", 1), ("B", 0)],
    "H3 克隆与给克隆建快照落在 origin 那个 checkpoint": [("W", 11, 1), ("W", 11, 1), ("PSC", 11), ("K", 13, 0), ("PS", 13),
                                                ("K", 13, 1), ("D", 14), ("B", 0), ("B", 0)],
}
TRACE_SPECS = [("REF", {}), ("BING_R", {}), ("BING_R", {"hi": "inf"}), ("YI", {}), ("YI", {"order": "any"}),
               ("BING_T", {}), ("BING_T", {"order": "any"})]


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    rows = []

    def emit(section, payload):
        rows.append(1)
        print(json.dumps({"section": section, **payload}, ensure_ascii=False, default=str), flush=True)
    if mode in ("bfs", "all"):
        for name, cfg in VARIANTS:
            result = bfs_shortest(name, cfg, LIM_BATCH, 10)
            emit("bfs-batch", result)
            if result["found"]:
                emit("bfs-batch-replay", {"variant": label(name, cfg), "path": result["path"],
                                          "arms": replay_summary([(name, cfg)] + STRONG_ARMS, result["path"])})
        for name, cfg in STRONG_ARMS:
            emit("bfs-clone", bfs_shortest(name, cfg, LIM_CLONE, 8))
            emit("bfs-crash", bfs_shortest(name, cfg, LIM_CRASH, 8))
    if mode in ("bfs-big", "all"):
        for name, cfg in BIG_SPECS:
            emit("bfs-big", bfs_shortest(name, cfg, LIM_BIG, 11, budget=2500000))
        for name, cfg in STRONG_ARMS[1:]:
            emit("bfs-clone-big", bfs_shortest(name, cfg, LIM_CLONE_BIG, 8, budget=2500000))
    if mode in ("replay", "all"):
        for title, path in HISTORIES.items():
            emit("history", {"title": title, "path": path, "arms": replay_summary(VARIANTS, path)})
            for line in trace(TRACE_SPECS, path):
                emit("trace", {"title": title, "line": line})
    if mode in ("fuzz", "all"):
        emit("fuzz", {"seeds": 300, "length": 150, "result": fuzz(VARIANTS, 300, 150, FUZZ)})
        any_specs = [spec for spec in VARIANTS if spec[1].get("order") == "any"]
        emit("fuzz-noX", {"seeds": 300, "length": 150, "result": fuzz(any_specs, 300, 150, dict(FUZZ, x=False))})
        for mut in (("noprop",), ("ge",)):
            specs = [(name, dict(cfg, mut=mut)) for name, cfg in STRONG_ARMS[1:]]
            emit("fuzz-mutation", {"mut": mut, "seeds": 150, "result": fuzz(specs, 150, 150, FUZZ)})
    if mode in ("geometry", "all"):
        specs = [("REF", {}), ("BING_R", {}), ("YI", {}), ("YI", {"merge": "moveS"}), ("YI", {"merge": "moveN"}), ("BING_T", {})]
        for shape in ("G1", "G2", "G3", "G4", "G5", "G6"):
            for row in geometry(shape, 1000, specs):
                emit("geometry", row)
    print(json.dumps({"section": "END", "rows": len(rows)}))


if __name__ == "__main__":
    main()
