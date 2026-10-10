# K5 计数模型：纯算术。输入全部来自：层 0 一侧探针/代码闭式（今天的树）与 records / 日志里记的放量数（旧代码，标「记录」）
second = 1_678_377_204   # 探针 L0 second-stream states_full
rollback = 67_502_198    # 探针 L0 through-rollback
multi = 1_358_954_634    # 探针 L0 multi-record
unlink = 1_463_812_278   # 探针 L0 unlink
first = 16_777_260       # 探针 L0 first-stream
tree = 78_905_428        # 代码闭式（ignore 理由）与记录相同
pat = 553_664_602        # 记录 2026-09-30（按位置寻址 5 条，第 3 条 10 态）
floor, sigma, iomin = 8_223, 65_548, 33   # 记录
shared_prefix = 16_777_259                 # 记录：多记录 reused=16777259
naive = first + second + rollback + multi + unlink + tree + pat + floor + sigma + iomin
distinct = second + (multi - shared_prefix) + (unlink - multi) + tree + pat + floor + sigma + iomin
print("naive_sum_of_domains", naive)
print("distinct_if_every_common_prefix_is_shared", distinct, "unlink_extra_over_multi", unlink - multi)
rates = {
    "GPU 2026-09-30 whole run (2563727926 states / 27288 s)": 2563727926 / 27288,
    "GPU 2026-10-08 unlink item (1448083638 / 4680 s)": 1448083638 / 4680,
    "GPU 2026-10-08 multi item (1342177375 / 3950 s)": 1342177375 / 3950,
    "CPU two hosts (2 x 24.8k/s, records 2026-09-28)": 2 * 24_800,
    "CPU one host 24.8k/s": 24_800,
    "CPU one host 23.7k/s": 23_700,
}
for name, rate in rates.items():
    print(f"{name:58s} rate={rate:10.0f}/s  naive={naive/rate/3600:6.1f} h  distinct={distinct/rate/3600:6.1f} h")
print("unlink per-state-mount CPU lower bound: two hosts %.1f h, one host %.1f h" % (unlink / 48_500 / 3600, unlink / 24_800 / 3600))
print("12 tree/position flows x ~500 s recording = %.2f h" % (12 * 500 / 3600))
