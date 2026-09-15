#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12（deadlist 条目的形态） 第三轮攻方腿模型：三条臂的第三轮收严形态与阳性对照，只用 Python 标准库。
# 底座 dl2.py 是第二轮攻方腿模型的逐字节拷贝（sha256 f5020d0ecf49…），这一轮不改它，只在这里加：
#   可动意图 eligible()：order=r3 是第三轮材料写的前提——丙-区间的级联之间不排次序、销毁头等同一个头上更早的级联；
#     乙、丙′ 同一个头上的意图按写意图的次序（头与头之间不排）；lineage 让丙-区间也按这条次序；any / fifo 同第二轮；
#   一批处理多条（dl3_batch）；prev 分到下一次发布才传（inherit=nextpub：恢复时照意图补传 / nextpub_lost：崩溃后不补）；
#   丙-区间上界的误读 hi=pub（取写意图时最后一个已发布的号）。
# 第三轮三条臂（材料第三节）：丙-区间 hi=fixed（下一侧是快照取它的 txg，是头取写意图的 checkpoint 号）；
#   乙 merge=moveS（挑边固定为搬 S 那一份，不数条数）；丙′ first_seg=zero（克隆头的第一段装 0），那个树 ID 存在头条目里。
import dl2 as base
from dl3_batch import BatchMixin, clone_state

STRONG3 = dict(base.STRONG, order="r3", merge="moveS", first_seg="zero")
ORDERS = ("fifo", "any", "r3", "lineage")
INHERITS = ("intent", "end", "nextpub", "nextpub_lost")


class Mixin3(BatchMixin):
    def __init__(self, world, cfg):
        super().__init__(world, cfg)
        assert self.cfg["order"] in ORDERS and self.cfg["inherit"] in INHERITS, self.cfg
        self.log, self.in_release, self.pending = None, False, []

    # ---- 动作记录：BS / BG 在影子上跑一段，把动作记下来再施加到本体 ----
    def release_key(self, world, key):
        if self.log is not None:
            self.log.append(("rel", key, world.mk2bid[key[-4:]]))
        self.in_release = True
        super().release_key(world, key)
        self.in_release = False

    def rekey(self, key, new_key):
        if self.log is not None:
            self.log.append(("rek", key, new_key))
        super().rekey(key, new_key)

    def free(self, bid):
        if self.log is not None and not self.in_release:
            self.log.append(("free", bid))
        super().free(bid)

    # ---- 销毁快照：写意图 ----
    def destroy_snap(self, world, sid, info):
        """意图带上头、P、下一侧与序号；prev 按 inherit 在写意图时传 / 下一次发布之后传 / 最后一批传（end，dl2 的变体）。"""
        prev_txg = self.ps.pop(sid)
        intent = self.make_cascade(world, sid, info, prev_txg)
        intent.update(kind="cascade", head=info["head"], P=prev_txg, next=info["next"], seq=self.seq)
        self.seq += 1
        if self.cfg["inherit"] == "intent":
            self.inherit(intent)
        elif self.cfg["inherit"] in ("nextpub", "nextpub_lost"):
            self.pending.append(intent)
        self.intents.append(intent)
        self.m["max_intents"] = max(self.m["max_intents"], len(self.intents))

    def apply_pending_passes(self):
        for intent in self.pending:
            self.inherit(intent)
        self.pending = []

    def drop_pending_passes(self):
        self.pending = []

    # ---- 哪些意图此刻可以动条目 ----
    def eligible(self):
        order, result = self.cfg["order"], []
        if not self.intents:
            return result
        if order == "fifo":
            return [0]
        for index, intent in enumerate(self.intents):
            earlier = self.intents[:index]
            if order == "any":
                blocked = False
            elif order == "lineage" or self.name in ("YI", "BING_T"):
                blocked = any(other["head"] == intent["head"] for other in earlier)
            else:   # r3 下的丙-区间：级联之间不排次序；销毁头等同一个头上更早的级联
                blocked = intent["kind"] == "drophead" and any(
                    other["kind"] == "cascade" and other["head"] == intent["head"] for other in earlier)
            if not blocked:
                result.append(index)
        return result

    def unit_work(self, world, intent):
        """一条条目（或一块 livelist 块）的工作，不判做完、不删意图；与 dl2 ArmBase.run_unit 的工作部分同一段逻辑。"""
        self.m["units"] += 1
        if intent["kind"] == "drophead":
            if intent["frees"]:
                self.free(intent["frees"].pop(0))
            else:
                keys = self.drop_keys(intent)
                if keys:
                    if self.log is not None:
                        self.log.append(("del", keys[0]))
                    self.ks.delete(keys[0])
        else:
            self.cascade_unit(world, intent)

    def run_unit(self, world, index):
        intent = self.intents[index]
        self.unit_work(world, intent)
        if self.intent_done(intent):
            self.intents.pop(index)
            if intent["kind"] == "cascade":
                self.finish_cascade(intent)


class Ref3(Mixin3, base.Ref):
    """阳性对照：交接换指针、级联与销毁头一次做完，不写意图（dl2 Ref 原样）。"""
    destroy_snap = base.Ref.destroy_snap
    destroy_head = base.Ref.destroy_head


class BingRange3(Mixin3, base.BingRange):
    def make_cascade(self, world, sid, info, prev_txg):
        intent = super().make_cascade(world, sid, info, prev_txg)
        if self.cfg["hi"] == "pub" and info["next"][0] == "h":
            intent["hi"] = info["commit_txg"] - 1
        return intent


class Yi3(Mixin3, base.Yi):
    pass


class BingTid3(Mixin3, base.BingTid):
    pass


ARMS3 = {"REF": Ref3, "BING_R": BingRange3, "YI": Yi3, "BING_T": BingTid3}
