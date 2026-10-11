# m3-broad-gate-gov-r3 辩方腿（Sonnet）复跑材料（2026-10-03）

脚本里的路径写的是这一轮的草稿目录 `/tmp/claude-1000/m3-gov-r3-defense/` 与冻结副本 `/tmp/claude-1000/m3-gov-r3-freeze/tree/`；换地方就改脚本头几行。
都不起驱动真跑、不起整道门禁（门禁只经样本档起）、不编译。

- `runsamples.sh <弄坏开关或空> [样本名过滤]`：36 份门禁样本逐份拷进临时目录、起 setup.sh、cd 进样本根、根参数写同一个目录、带 --force（`stage-selftest.sh` 的起法；那一支被闸拒，因为 10 份新样本未暂存），打 ok / FLIP。
- `mut2.py <名>`：把冻结副本的 `scripts` 与 `.claude/scripts` 拷到草稿目录下、按名字在副本里写错驱动（16 种，export-no-cargo 在 mut3.py）；再用 `sed -n 202,389p <冻结副本>/.claude/gate.d/checker-tier-broad-mutation.sh > cell.py` 抽出门禁第一格的内嵌 python，
  `python3 cell.py <样本根> <样本根>/.broad-sample/home <写错版驱动> <样本根>/.broad-sample/rustc-vv.txt "" <样本根>/.broad-sample/home/tree.source <样本根>/.broad-sample/tool-versions.txt <样本根>/.broad-sample/exclusions.tsv <样本根>/.broad-sample/environment` 判。
- `combos2.py <驱动> <gate_verdict 内嵌 python> <目录> <趟数> <种子>`：第二格（`sed -n 399,624p`）在随机结果表上的判定与「每条有结论且没红」对照；这一版另掺升档标签的 caught 行（没有登记行）。
- `envgap.py`：全局键对 7 种构建环境的变化。
- `outputs/`：本轮原样输出。
