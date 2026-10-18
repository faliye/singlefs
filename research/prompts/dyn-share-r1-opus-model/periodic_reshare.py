#!/usr/bin/env python3
"""丙（静态分工加定期重分）的时序模型。

分工照引擎今天的写法：WeightedShares::owner_of_block（crash_amplification.rs）平滑加权轮转，第 k 块归谁只由权重与 k 定；
引擎开跑时按环境里的权重算一次这一份（同一文件 run_from_environment_with_observers 里 my_blocks / selected），跑着不换。
驱动每隔 interval 秒量两台最近一段的块速，预计剩余用时差超过 threshold 秒就重分：
  peer-only  只叫第二台停（停下文件，判完手里那一块）、按新权重重起第二台；本机那一趟不停，仍按开跑时的权重领块
             （候选正文「代价是每次重分第二台要重起一趟」的字面）；
  both       两台都停：本机先导入第二台取回的库（库开着不能拷：two-hosts 驱动「库开着的时候不能拷」），照新权重两台都重起。
新权重照量到的块速（naive），或把重起要花的秒数折进去（aware）。重起一趟要 restart 秒（拷树、编、起卡），这期间那一台不干活。
块的用时沿块序号分两段：前 hard_from 之前是 1 倍，之后是 hard_factor 倍（同一条流不同点上录入速度差几十倍，见记录监控第 27、33 轮）。
只用 python3 标准库；输出一行一格，键=值。
"""
import itertools
import sys


def owner_pattern(weights):
    """与 owner_of_block 同一算法：返回长度为权重和的归属表，第 k 块的归属是 pattern[k % total]。"""
    total = sum(weights)
    current = [0] * len(weights)
    pattern = []
    for _ in range(total):
        for index, weight in enumerate(weights):
            current[index] += weight
        owner = max(range(len(current)), key=lambda index: (current[index], -index))
        current[owner] -= total
        pattern.append(owner)
    return pattern


def simulate(blocks, base_seconds, weights, policy, interval, threshold, restart, hard_from, hard_factor, rate_policy, max_events=10**7):
    cost = lambda host, block: base_seconds[host] * (hard_factor if block >= hard_from else 1.0)
    done = [set(), set()]          # 各台库里判完的块（本机库 / 第二台这一份）
    visible = [set(), set()]       # 各台开跑时看得到的已判块（本机：自己库；第二台：送过去的快照加它自己取回的）
    run_weights = [list(weights), list(weights)]
    queue = [[], []]
    position = [0, 0]
    busy_until = [0.0, 0.0]
    in_hand = [None, None]
    finished_in_window = [0, 0]
    working_in_window = [0.0, 0.0]
    restarts = 0
    duplicates = 0

    def start(host, now):
        pattern = owner_pattern(run_weights[host])
        total = len(pattern)
        queue[host] = [k for k in range(blocks) if pattern[k % total] == host and k not in visible[host]]
        position[host] = 0
        busy_until[host] = now

    start(0, 0.0)
    start(1, 0.0)
    now = 0.0
    next_check = interval
    events = 0
    while events < max_events:
        events += 1
        candidates = []
        for host in (0, 1):
            if in_hand[host] is not None or position[host] < len(queue[host]):
                candidates.append(busy_until[host])
        if not candidates:
            break
        now = min(min(candidates), next_check)
        for host in (0, 1):
            if in_hand[host] is not None and busy_until[host] <= now:
                block = in_hand[host]
                if block in done[0] or block in done[1]:
                    duplicates += 1
                done[host].add(block)
                in_hand[host] = None
                finished_in_window[host] += 1
                working_in_window[host] += cost(host, block)
            if in_hand[host] is None and busy_until[host] <= now and position[host] < len(queue[host]):
                block = queue[host][position[host]]
                position[host] += 1
                if block in done[host]:
                    continue
                in_hand[host] = block
                busy_until[host] = now + cost(host, block)
        if now >= next_check:
            # 块速只按这一段里真在干活的秒数算（重起那几分钟不算进去）；一块都没判完的那一台按它手头那一块的用时估
            rates = [finished_in_window[host] / working_in_window[host] if finished_in_window[host] else 1.0 / base_seconds[host] for host in (0, 1)]
            left = [len(queue[host]) - position[host] for host in (0, 1)]
            finish = [max(busy_until[host] - now, 0.0) + left[host] / rates[host] for host in (0, 1)]
            if max(finish) - min(finish) > threshold and min(left) > 0:
                restarts += 1
                new_weights = [max(1, round(rates[host] * 1000)) for host in (0, 1)]
                if rate_policy == "aware":
                    remaining_seconds = max(finish)
                    new_weights[1] = max(1, round(rates[1] * 1000 * max(remaining_seconds - restart, 1) / remaining_seconds))
                    if policy == "both":
                        new_weights[0] = max(1, round(rates[0] * 1000 * max(remaining_seconds - restart, 1) / remaining_seconds))
                # 第二台：判完手里那一块再停，取回；新的一趟看得到旧快照加它自己取回的
                if in_hand[1] is not None:
                    done[1].add(in_hand[1])
                    in_hand[1] = None
                visible[1] = visible[1] | done[1]
                run_weights[1] = new_weights
                if policy == "both":
                    if in_hand[0] is not None:
                        done[0].add(in_hand[0])
                        in_hand[0] = None
                    done[0] |= done[1]          # 导入第二台取回的库（按旧权重点名，旧权重下它领的都在）
                    visible[0] = set(done[0])
                    visible[1] = visible[1] | done[0]   # 新快照
                    run_weights[0] = new_weights
                    start(0, now + restart)
                start(1, now + restart)
            finished_in_window = [0, 0]
            working_in_window = [0.0, 0.0]
            next_check = now + interval
    union = done[0] | done[1]
    holes = blocks - len(union)
    return {"makespan": now, "restarts": restarts, "duplicates": duplicates, "holes_at_the_end_of_the_run": holes,
            "local_blocks": len(done[0]), "peer_blocks": len(done[1])}


def main():
    blocks = 22365   # 第 6 条 unlink：本机 8946 块、第二台 13419 块（正文 K1 那一格的数）
    weights = [2, 3]
    pattern = owner_pattern(weights)
    print(f"name=bing_split_check weights=2,3 local_blocks={sum(1 for k in range(blocks) if pattern[k % 5] == 0)} peer_blocks={sum(1 for k in range(blocks) if pattern[k % 5] == 1)}")
    # 块的基础用时：第二台 mount_continuation 每秒 1.0 块（记录监控第 39 轮，量的）；本机的录入速度没量，按倍数扫
    for local_factor, policy, rate_policy, interval, threshold, restart, hard_factor in itertools.product(
            (1.0, 2.0), ("peer-only", "both"), ("naive", "aware"), (600, 1800), (120, 600), (300, 1200), (1.0, 10.0)):
        base = [1.0 / local_factor, 1.0]
        row = simulate(blocks, base, weights, policy, interval, threshold, restart, int(blocks * 0.7), hard_factor, rate_policy)
        print(f"name=bing_reshare local_speed_factor={local_factor} policy={policy} rate_policy={rate_policy} interval={interval} threshold={threshold} restart={restart} hard_factor={hard_factor} "
              + " ".join(f"{key}={value:.0f}" if isinstance(value, float) else f"{key}={value}" for key, value in row.items()))
    for local_factor, hard_factor in itertools.product((1.0, 2.0), (1.0, 10.0)):
        base = [1.0 / local_factor, 1.0]
        static = simulate(blocks, base, weights, "both", 10**12, 10**12, 0, int(blocks * 0.7), hard_factor, "naive")
        print(f"name=bing_static local_speed_factor={local_factor} hard_factor={hard_factor} makespan={static['makespan']:.0f} local_blocks={static['local_blocks']} peer_blocks={static['peer_blocks']}")


if __name__ == "__main__":
    main()
