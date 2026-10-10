# m3-batch17-code-r1 快照对不上的两份（2026-10-05）

`bash research/scripts/snapshot-check.sh m3-batch17-code-r1`：7 个文件里 2 个对不上——`crates/singlefs-core/src/mount.rs`（快照 e9b32b9051c5）与 `crates/mutations.tsv`。快照之后改它们的是别的会话合进主工作区的补丁：根环线第 10b 批（`mount.rs` 把「挂载时读不出的槽」换成「挂载时读坏的槽」、RGall 去掉一圈窗口那一滤、C664 拒因拆开）与形式化线 26-3；变异表是两条线的锚点改写与追加。

倒推不用逐处反做：冻结副本 `/tmp/claude-1000/m3-batch17-code-r1/tree/` 里这两份的 sha256 与快照逐行相同（`sha256sum`：`mount.rs` e9b32b9051c55e50…、`mutations.tsv` 4140c7847b56fa71…），它就是快照时的原样。两条腿引的代码行都写冻结副本路径，`cite-check.py --root <冻结副本> --root .` 42 处全对上。

被改的处数（`diff <冻结副本> <主工作区>` 的 hunk 数）：`mount.rs` 128 处；`mutations.tsv` 10 处改写，另追加 80 行（2485 → 2565 行）。腿引 `mount.rs` 的第 1683、1838、1849、1866、1884、1942、3432、4756 行都按冻结副本核；腿引的变异表第 2472–2477 行是第 17 批自己的行，冻结副本里原样在。

主工作区今天的 `mount.rs`（sha256 77678733681f5d86…）上，主 agent 另用攻方装置把甲一、甲二与不注错对照那五段层 0 重跑了一遍，逐数与冻结副本上相同（判决第一节）。
