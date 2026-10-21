# W3 正推腿的模型（m3-w3-r1-sonnet-model）

在 E175 装置（`scripts/experiments/e175_product_path_inode_watermark_regression/device-source.rs`）上加了一段另算（`w3-device.diff`），
对每个候选的「盘上发号历史」会不会让某一步判红，逐步算出来，并拿 E175 自己的发号事件（`name=event` 行，Q1）当对照。不改装置原有的任何一行判定。

## 它算什么

候选的历史行 (g, H) 与两条判法 A（地板）、B（出生代截断）的定义在报告 `research/prompts/m3-w3-r1-sonnet-output.md` 一·1.1。

| 记号 | 历史行从哪来（模型里怎么算） |
|---|---|
| c1a | 实例表行预留里记所选根的水位：行 (实例 i, T_i) 第一次出现的那一步，取「那一步之前」读到的根 (i, T_i) 的水位 M_a，(g, H) = (T_i, M_a) |
| c1b | 同上，但取那一步之前根环里读得出的、实例 i 且 txg ≤ T_i 的根的水位最大值 M_b |
| c2i | 记账里每个实例留最高那一行：实例 i（表里有 T_i 的取 txg ≤ T_i，现行实例取全部）的根的水位最大值，g = 第一次到它的 txg |
| c2all | 记账里每个根的 (txg, 水位) 都留（时间线上的根，按现行表判不被抛弃） |
| c3 | 按出生代账判：最新根某条 (n, b)，账里同号有别的出生代、持有它的根里有一条不被抛弃 |
| c4 | 根记录里带一对 (全时间线最大水位, 第一次到它的 txg) |

c2all、c3 是「历史完整保留」的上界：它们用的是装置一路攒下的全部根与出生代账，不是产品里某个容器的读法。
c1a、c1b、c2i、c4 的信息量是真有限的：只用各自那份历史行。
B 判法只比 inode 记录，不比谱系上的墓碑（装置的 `RootFacts` 里没有墓碑）。

输出行：`name=w3step`（有事件或有候选判红的步，一步一行，`own=` 后是该步每个事件自己那条记录各候选判不判红，顺序 c1a c1b c2i c2all c3 c4，R 红 `-` 不红）与 `name=w3summary`（一段历史一行）。
`aggregate.py` 汇总；`expected/` 是这一份代码上跑出的预期。

## 重跑

```
SINGLEFS_REPO=<冻结树根> bash rerun.sh prepare     # 拷两份冻结树（V1 臂施 patch-V1.diff，P 臂原样）、放进加了另算的装置
bash rerun.sh build                                # 各编一次，编译 6G，自己的编译目录
bash rerun.sh run d | g-long-v1 | f2-v1 | f1-broad-v1 | f1-reuse-v1 | f1-deleting-v1 | f1-newest-v1 | f3a-v1 | g-long-p | f3a-p
bash rerun.sh compare                              # 与 expected 比；不读 SINGLEFS_REPO
```

`SINGLEFS_REPO` = `~/.local/share/singlefs/work/m3-w3-r1/tree`（冻结树，`research/prompts/m3-w3-r1-snapshot/tree-sha256.txt` 的清单）。
每个 `run` 步可重复调，打出 `DONE` 为止；跑 4G。2026-10-08 在空工作目录里从 `prepare` 起逐步跑通，`compare` 末行「对不上 0 处」。

## 没入库的东西

- F2 的 P 臂 256 段（15 616 步）早一版装置（还没有 c4 与定向段）上跑过：c1a c1b c2i c2all c3 在这 15 616 步上都 0 次判红、0 个事件；
  结果文件拷进了 `out/f2-P-earlier-device.out`（728 304 字节，没有 c4 字段，所以 `aggregate.py` 读不了它、`expected/` 里也没有它）；这一版装置的 `rerun.sh` 不跑它（14 分钟）。
- `out/` 里放了 7 份小结果文件（D 两臂、F1 四组 V1、G-long V1）；F2、F3a 与 G-long P 的原文件按 `expected/normalized-sha256.txt` 比。
