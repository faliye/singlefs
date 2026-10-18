#!/usr/bin/env python3
"""丁（录入块跨主机断点）的「取回并进本机库」那一种：观察者录入块、看过的恢复镜像、桶里的读集见证三样怎么并。

照引擎今天的写法造一个小库（crash_amplification.rs record_one_observer_piece 与 verdict_store.rs）：
  - 一个状态落进的桶里有见证、这个状态在见证那几段上的摘要与见证相同，就取见证的镜像键、不再看，也不读那张镜像；
  - 没命中的看一遍，镜像连同这个桶的整份见证清单一批写进库（见证在、它引的镜像必在）；一块看完才写录入块；
  - 镜像键 = 桶键接见证摘要，见证的那几段是第一次看出它的那个状态读过的段：两台各自的见证不同，同一个状态两台给的镜像键可以不同；
  - 判段读录入块里每个镜像键对应的镜像，读不到就整趟报错（「先跑录入段」）。
三种并法：
  listed    候选正文的字面「观察者块（连同它桶里的见证）」：并录入块与见证清单（本机已有的清单照写盖掉），不并镜像；
  planned   再并取回的录入块引到的镜像（照今天按计划导入的做法：只读点名的键）；见证清单照写盖掉；
  union     见证清单两边取并集，取回的库里的镜像全并。
第二台在录一块的半中间被停（term）：那一块的几张镜像与见证已经落盘，那一块的录入块没写。
只用 python3 标准库；输出一行一格，键=值。
"""
import hashlib
import itertools
import random


def digest(*parts):
    h = hashlib.sha256()
    for part in parts:
        h.update(repr(part).encode())
    return h.hexdigest()[:16]


def make_states(blocks, states_per_block, seed, alphabet=3):
    rng = random.Random(seed)
    states = []
    for _ in range(blocks * states_per_block):
        # 四段字节；读集由第一段定：第一段相同的状态读同样的段（见证对它们才成立）
        states.append(tuple(rng.randrange(alphabet) for _ in range(4)))
    return states


def read_set(state):
    return (0, 1) if state[0] == 0 else (0, 2, 3)


def observe(state):
    return digest("observation", [state[i] for i in read_set(state)])


class Library:
    def __init__(self):
        self.images = {}       # 镜像键 -> 观察结果
        self.witnesses = {}    # 桶 -> [(读集, 摘要, 镜像键)]（存的次序）
        self.blocks = {}       # 块号 -> (镜像键…)

    def record_block(self, block, states, stop_after=None):
        keys = []
        for offset, state in enumerate(states):
            if stop_after is not None and offset >= stop_after:
                return False
            bucket = "bucket"
            match = None
            for ranges, witness_digest, image_key in self.witnesses.get(bucket, []):
                if digest([state[i] for i in ranges]) == witness_digest:
                    match = image_key
                    break
            if match is None:
                ranges = read_set(state)
                witness_digest = digest([state[i] for i in ranges])
                match = digest(bucket, witness_digest)
                # 一批：镜像与这个桶的整份见证清单
                self.images[match] = observe(state)
                self.witnesses.setdefault(bucket, []).append((ranges, witness_digest, match))
            keys.append(match)
        self.blocks[block] = tuple(keys)
        return True

    def judge(self, block):
        missing = [key for key in self.blocks[block] if key not in self.images]
        return missing


def merge(local, peer, way):
    for block, keys in peer.blocks.items():
        local.blocks.setdefault(block, keys)
    if way in ("listed", "planned"):
        for bucket, witness_list in peer.witnesses.items():
            local.witnesses[bucket] = list(witness_list)      # 不同的照写
    if way == "planned":
        for keys in peer.blocks.values():
            for key in keys:
                local.images.setdefault(key, peer.images[key])
    if way == "union":
        for bucket, witness_list in peer.witnesses.items():
            known = {(ranges, witness_digest) for ranges, witness_digest, _ in local.witnesses.get(bucket, [])}
            for entry in witness_list:
                if (entry[0], entry[1]) not in known:
                    local.witnesses.setdefault(bucket, []).append(entry)
        for key, value in peer.images.items():
            local.images.setdefault(key, value)


def run(way, seed, stop_after, alphabet, blocks=20, states_per_block=16, local_share=10, peer_stops_in=12):
    states = make_states(blocks, states_per_block, seed, alphabet)
    of_block = lambda block: states[block * states_per_block:(block + 1) * states_per_block]
    local, peer = Library(), Library()
    for block in range(local_share):
        local.record_block(block, of_block(block))
    for block in range(blocks - 1, peer_stops_in, -1):
        peer.record_block(block, of_block(block))
    peer.record_block(peer_stops_in, of_block(peer_stops_in), stop_after=stop_after)
    merge(local, peer, way)
    dangling_witnesses = sum(1 for witness_list in local.witnesses.values() for _, _, key in witness_list if key not in local.images)
    for block in range(blocks):
        if block not in local.blocks:
            local.record_block(block, of_block(block))
    erroring_blocks = sum(1 for block in range(blocks) if local.judge(block))
    return dangling_witnesses, erroring_blocks


def double_record(seed, blocks=20, states_per_block=16, local_share=10):
    """同一块两台都录了（两端对扫的交叉、重分时的重复）：录入块的字节比一比，观察结果比一比。"""
    states = make_states(blocks, states_per_block, seed)
    of_block = lambda block: states[block * states_per_block:(block + 1) * states_per_block]
    local, peer = Library(), Library()
    for block in range(local_share + 1):
        local.record_block(block, of_block(block))
    for block in range(blocks - 1, local_share - 1, -1):
        peer.record_block(block, of_block(block))
    shared = local_share
    bytes_differ = local.blocks[shared] != peer.blocks[shared]
    observations_differ = [local.images[key] for key in local.blocks[shared]] != [peer.images[key] for key in peer.blocks[shared]]
    return bytes_differ, observations_differ


def main():
    seeds = range(40)
    for way, alphabet in itertools.product(("listed", "planned", "union"), (3, 8)):
        runs = 0
        with_dangling = 0
        with_errors = 0
        worst = 0
        for seed, stop_after in itertools.product(seeds, range(1, 16)):
            dangling, erroring = run(way, seed, stop_after, alphabet)
            runs += 1
            with_dangling += dangling > 0
            with_errors += erroring > 0
            worst = max(worst, erroring)
        print(f"name=ding_merge way={way} alphabet={alphabet} runs={runs} runs_with_witnesses_whose_image_is_missing={with_dangling} runs_where_the_judge_stage_errors={with_errors} most_erroring_blocks_of_twenty={worst}")
    differ = [double_record(seed) for seed in seeds]
    print(f"name=ding_double_record samples={len(differ)} block_bytes_differ={sum(1 for bytes_differ, _ in differ if bytes_differ)} observations_differ={sum(1 for _, observations in differ if observations)}")


if __name__ == "__main__":
    main()
