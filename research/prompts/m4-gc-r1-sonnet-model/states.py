# 一批搬迁的单元段写数 = 盘数 x (搬的单元数 K + 这次新写的提交内生单元数 P)；状态数上界 2^写数（每次写取两态；段内任意子集）
# P 取 11 / 12：现有删除流的各次发布 11-13 个单元（意图 13、两批 12 与 11、最后一批 12，见 O9）。盘数 2。
DEPTH = 30   # scripts/crash/crash-amplification.sh 的 FULL_EXPAND_UP_TO
for p in (11, 12):
    print("P =", p)
    for k in (1, 2, 3, 4, 5, 8):
        writes = 2 * (k + p)
        print("  K =", k, "单元段写数", writes, "2^写数", 2 ** writes, "全量展开" if writes <= DEPTH else "越过展开深度 30，exhaustive=false")
