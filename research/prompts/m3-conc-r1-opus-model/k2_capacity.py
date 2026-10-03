#!/usr/bin/env python3
"""m3-conc-r1 攻方：按事务切（问题单 Q7 的「按事务切」，E167 叫 K2）的状态数随在飞事务数 k 怎么长。

E167 第二段 L4：「K2 与积的闭式相等」——K2 的状态集是各事务各自那条链的枚举域之积。
每条链的状态数取三个点：
- E167 的 G1（一盘 u = 2）：K2 = 64，单事务链 8（64 = 8²）；
- E167 的 G4a（两盘）：K2 = 784，单事务链 28（第二段「两盘单事务链 28 个状态」）；
- 真实发布形：新池新建文件那一次发布的段 24+2+1+2（E167 第一段停机条款 S1 读到的 16777223），
  用与 crates/singlefs-harness/src/memory_pool.rs 的 closed_form_state_count 同一个闭式现算（1 + Σ(2^s − 1)）。
k 取 2..16（D25（目标负载优先级） 已定项 7：并发 fsync 下限 16）。
能力参照：u32 段内序号（E167 引 crash_verify_gpu.rs:305）、u64（closed_form_state_count 的返回类型）、
挂钟按两档实测吞吐：CPU 双机 4108 + 4320 态/秒（briefs/2026-09-28.md 第 19 行），
GPU 全量 2 616 xxx xxx 态 7 小时 37 分（提交 fcc4d340 的说明；本模型取 26.16 亿 ÷ 27420 秒）。
用法：k2_capacity.py
"""

U32 = 2 ** 32 - 1
U64 = 2 ** 64 - 1
CPU_RATE = 4108 + 4320
GPU_RATE = 2.616e9 / (7 * 3600 + 37 * 60)
SECONDS_PER_YEAR = 365.25 * 86400


def closed_form_state_count(segments):
    return 1 + sum(2 ** s - 1 for s in segments)


def main():
    chains = {
        "e167_g1_one_disk": 8,
        "e167_g4a_two_disks": 28,
        "new_pool_file_creation_publish_24_2_1_2": closed_form_state_count([24, 2, 1, 2]),
    }
    print(f"E name=chain check=closed_form_24_2_1_2 value={chains['new_pool_file_creation_publish_24_2_1_2']} "
          f"e167_s1_reads=16777223 equal={'true' if chains['new_pool_file_creation_publish_24_2_1_2'] == 16777223 else 'false'}")
    print(f"E name=chain check=e167_products g1_k2={8 ** 2} e167_g1_k2=64 g4a_k2={28 ** 2} e167_g4a_k2=784")
    for name, per_chain in chains.items():
        for k in (2, 3, 4, 8, 16):
            states = per_chain ** k
            print(f"E name=k2_states chain={name} per_transaction={per_chain} in_flight={k} states={states:.3e} "
                  f"exceeds_u32={'true' if states > U32 else 'false'} exceeds_u64={'true' if states > U64 else 'false'} "
                  f"cpu_years={states / CPU_RATE / SECONDS_PER_YEAR:.3e} gpu_years={states / GPU_RATE / SECONDS_PER_YEAR:.3e}")
    # 每条链多大时 k 个在飞的积越过 u64：最小的整数 c 使 c^k > 2^64 − 1
    for k in (2, 3, 4, 8, 16):
        c = int(round(U64 ** (1.0 / k)))
        while c ** k <= U64:
            c += 1
        while (c - 1) ** k > U64:
            c -= 1
        print(f"E name=u64_overflow_chain_size in_flight={k} smallest_per_transaction_states={c}")
    # 对照：串行（K1 = 真值）k 次发布首尾相接，状态数是各段闭式之和而不是积
    per = chains["new_pool_file_creation_publish_24_2_1_2"]
    for k in (2, 3, 16):
        serial = 1 + k * (per - 1)
        print(f"E name=serial_reference chain=new_pool_file_creation_publish_24_2_1_2 publishes={k} states={serial:.3e} "
              f"gpu_hours={serial / GPU_RATE / 3600:.2f}")


if __name__ == "__main__":
    main()
