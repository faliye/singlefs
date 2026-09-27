# runner-compile-first-r1 本地攻方：运行记录

提示文件：`research/prompts/runner-compile-first-r1-local-attack.md`
核对表（转述审计）：`research/prompts/runner-compile-first-r1-local-attack-translation-audit.md`
样本前缀：`research/prompts/runner-compile-first-r1-local-attack-output`

## 调用序列（共 5 次 ask-local.sh 调用，2 份干净）

| 次序 | 目标号 | 退出码 | 结果文件 | 大小（字节） | 词数（corruption-check.py 自报 words=） | 生词 / 拼接（oov-check.py） | 判定 |
|---|---|---|---|---|---|---|---|
| 1 | s1 | 0 | `runner-compile-first-r1-local-attack-output-s1.md` | 1073 | 188 | 生词=0，拼接=0 | 干净 |
| 2 | s2（第一次） | 5 | `runner-compile-first-r1-local-attack-output-void1.md`（作废副本，脚本自留） | 2622 | 436 | 未跑（判红即作废，不再跑 oov-check） | 作废 |
| 3 | s2（第二次，`>|` 同号重跑） | 5 | `runner-compile-first-r1-local-attack-output-void2.md` | 3206 | 547 | 未跑 | 作废 |
| 4 | s2（第三次，`>|` 同号重跑） | 5 | `runner-compile-first-r1-local-attack-output-void3.md` | 1855 | 324 | 未跑 | 作废 |
| 5 | s2（第四次，`>|` 同号重跑，提示已改，见下） | 0 | `runner-compile-first-r1-local-attack-output-s2.md` | 945 | 160 | 生词=0，拼接=0 | 干净 |

达到「至少两份干净样本」的出口（s1、s2），停止抽样。

## s2 前三次作废的原因与处置

`corruption-check.py` 三次都判红在同一项：`星号落单=1`（成对标记落单类）。现跑该检查器核实（对 void1/void2/void3 各跑一次 `python3 research/scripts/corruption-check.py <文件>`，三次都只有 `星号落单=1` 这一项非零，其余七项均为 0）。
逐一 `grep -n '\*'` 三份作废副本，命中的都是模型逐字引用提示里事实表 F4/F5/F8 给的通配符样式（`crates/**`、`research/e7-index-bench/src/bin/**`、`/tmp/claude-1000/**`，与 `.claude/hooks/agent-write-scope.tsv:6,11,23` 原文写法一致）。这三行的双星号是 glob 通配符，不是 markdown 加粗标记；检测器的「成对标记落单」规则把孤立的一个 `**`（前后没有另一个 `**` 与它配对闭合）当成掉了半截的加粗标记，三次都误判。
处置：把提示文件里 F4、F5、F8 这三行改写成不含字面 `**` 的等价散文表述（语义不变，仍是「递归匹配这个目录下任意深度的路径」），用 `research/scripts/replace-once.py` 定点改（各命中一次），改动记录与理由写进核对表「五、跑样本时发现的一处措辞改动」一节。改完第四次调用（同一个 s2 号，`>|` 重跑）退出码 0、过闸。
s1 在这处改动之前就已经跑完并判绿：s1 的十条答复都很短，没有逐字引用这三个通配符样式，没有触发这一项，不受这处改动影响；已现跑 `python3 research/scripts/corruption-check.py` 与 `python3 research/scripts/oov-check.py` 独立复核 s1，两者都是绿（见上表）。s2 是改过提示之后的产物，与 s1 不是同一份提示逐字产出——这一点写在这里存档，供核查员核对提示改动的时间点与两份样本各自对应哪一版提示（旧版提示未保留副本；改动只涉及 F4/F5/F8 三行的字面写法，不涉及判据本身，具体新旧文字见转述审计文件与本仓 `research/scripts/replace-once.py` 调用的前后串）。

## 表里判「拒」的格

s1、s2 两份干净样本里，十道操作（op1-op10）的判词（verdict）都不是「拒」（reject）——两份样本没有任何一格答「reject」。定义里「表里判『拒』的格，另外拿钩子喂对应的工具调用 JSON 现跑一次核退出码」这一步因此没有可核的格：两份干净样本都没有产出这样的格。没有另跑钩子核验退出码。

## 5b：算好的答案表

派发提示里没有给一份算好的答案表的路径。运行记录照实写：没有答案表，算术没比。

## 没做什么

- 没有对 void1/void2/void3 三份作废副本跑 `oov-check.py`（判红即作废，脚本本身在判红时不再往下跑生词检查这一步；本记录只用 `corruption-check.py` 现跑核实作废原因）。
- 没有解读、总结或采纳 s1、s2 里模型给出的判词与依据；两份样本之间方向是否一致不在本记录里写，由主 agent 判。
- 没有为「表里判『拒』的格」跑额外的钩子核验：两份干净样本没有产出这样的格（见上）。
