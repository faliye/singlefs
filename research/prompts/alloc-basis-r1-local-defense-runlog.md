# alloc-basis-r1 本地辩方腿：运行记录（2026-09-17）

提示：`research/prompts/alloc-basis-r1-local-defense.md`。核对表：`research/prompts/alloc-basis-r1-local-defense-translation-audit.md`。前台跑 `research/scripts/ask-local.sh`，不 `setsid`、不 `&`、不 `disown`。

| 样本 | 退出码 | 词数 | oov-check.py 生词 | 拼接 | 判定 |
|---|---|---|---|---|---|
| s1 | 0 | 518 | formula's、subtracts（生词=4，均为合法英文词，非损坏） | 0 | 干净 |
| （第一次 s2 尝试，作废） | 5 | — | 生词=2（rechecked、reclaimedclaimed）；拼接=1（reclaimedclaimed = reclaimed+claimed） | 1 | 带损坏，`ask-local.sh` 内部字词损坏闸判红，原样输出留存为 `research/prompts/alloc-basis-r1-local-defense-output-void1.md`；`-output-s2.md` 本次为空（0 字节），换下一次重跑覆盖同一个 s2 槽位 |
| s2（重跑） | 0 | 472 | formula's（生词=1，合法英文词，非损坏） | 0 | 干净 |

两份干净样本已够（`three-way-local-defense.md` 要求至少两份）。五次尝试里跑了两次，一次判红、一次判绿，坏率与定义里「实测（2026-09-16 第一轮）：辩方提示在本地模型上五次坏三次」同方向（本轮 2 次里 1 次坏），未到「连续五次拿不到两份干净」的停止线。

本地网关（`~/code/ai-center` :8200）全程可用，没有缺席。
