#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12（deadlist 条目的形态） 第三轮攻方腿模型：「一批处理多条条目」的两种施加法。
#   顺序施加（BA）：可动意图轮流做一条，每一步都看得见前一步的结果——等价于一条一批连着做。
#   计划后施加（BS / BG）：批开始时对每个可动意图在它自己的影子上各算一份工作（至多 rounds 条），把动作记下来再一起施加；
#   BS 施加时不核条目还在（最弱读法），BG 核（条目已被同一批里前面的动作拿走就跳过）。
# 被 dl3.Mixin3 继承；这里只用臂的 intents / ks / free / intent_done / finish_cascade / unit_work / eligible / m。
import pickle


def clone_state(value):
    return pickle.loads(pickle.dumps(value, protocol=pickle.HIGHEST_PROTOCOL))


class BatchMixin:
    def index_of(self, seq):
        for index, intent in enumerate(self.intents):
            if intent["seq"] == seq:
                return index
        return None

    def run_batch_seq(self, world, rounds):
        """一批顺序施加：每一轮把此刻可动的意图各做一条；做满 rounds 轮或没有可动意图为止。"""
        for _ in range(rounds):
            seqs = [self.intents[index]["seq"] for index in self.eligible()]
            if not seqs:
                return
            for seq in seqs:
                index = self.index_of(seq)
                if index is not None and index in self.eligible():
                    self.run_unit(world, index)

    def run_batch_snap(self, world, rounds, recheck):
        """一批按批开始时的状态计划：可动意图集合在批开始时定；每个意图在自己的影子上做至多 rounds 条，
        动作按意图的序号依次施加到本体；施加完再按本体的状态判哪些意图做完、在同一批里删掉。"""
        seqs = [self.intents[index]["seq"] for index in self.eligible()]
        plans = []
        for seq in seqs:
            shadow = clone_state(self)
            shadow.log = []
            intent = shadow.intents[shadow.index_of(seq)]
            for _ in range(rounds):
                if shadow.intent_done(intent):
                    break
                shadow.unit_work(world, intent)
            plans.append((seq, shadow.log, {field: intent[field] for field in ("cursor", "phase", "frees") if field in intent}))
        for seq, actions, fields in plans:
            for action in actions:
                self.apply_action(action, recheck)
            index = self.index_of(seq)
            if index is not None:
                self.intents[index].update(fields)
        for seq in seqs:
            index = self.index_of(seq)
            if index is not None and self.intent_done(self.intents[index]):
                intent = self.intents.pop(index)
                if intent["kind"] == "cascade":
                    self.finish_cascade(intent)

    def apply_action(self, action, recheck):
        """rel = 删条目并经映射放块；rek = 改键；free = livelist 放块；del = 按前缀 / 号删条目（不放块）。"""
        kind = action[0]
        self.m["units"] += 1
        if kind == "rel":
            present = action[1] in self.ks.entries
            if present:
                self.ks.delete(action[1])
            if present or not recheck:
                self.free(action[2])
        elif kind == "rek":
            present = action[1] in self.ks.entries
            if present:
                self.ks.delete(action[1])
            if present or not recheck:
                self.ks.insert(action[2])
                self.m["moves"] += 1
        elif kind == "free":
            self.free(action[1])
        elif kind == "del":
            if action[1] in self.ks.entries:
                self.ks.delete(action[1])
        else:
            raise ValueError(action)
