# gate-shrink-r1 本地攻方运行记录

提示文件：`research/prompts/gate-shrink-r1-local-attack.md`
核对表：`research/prompts/gate-shrink-r1-local-attack-translation-audit.md`
样本前缀：`research/prompts/gate-shrink-r1-local-attack`

跑前先对提示文件本身跑过 `corruption-check.py`、`oov-check.py`：前者判绿
（`绿 ... cjk=0 words=9011 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0
星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0`）；后者判红（`生词=66 拼接=1`，
拼接命中 `faithfully(=faith+fully)`，是英文常见词被误切的假阳性，与这一轮判红的成因无关，
提示文件本身不是样本，不受这道闸约束）。

## 调用记录（按发生顺序，前台起 `run_in_background`，结束本轮等完成通知，未用 setsid / & / disown）

第 1 次调用：`bash research/scripts/ask-local.sh research/prompts/gate-shrink-r1-local-attack.md > research/prompts/gate-shrink-r1-local-attack-output-s1.md`。
退出码：5。`research/prompts/gate-shrink-r1-local-attack-output-s1.md` 确认为 0 字节。
脚本留下作废副本 `research/prompts/gate-shrink-r1-local-attack-output-void1.md`（28384 字节）。
对 void1 跑 `corruption-check.py`（带提示文件）：判红，原样行——
`红 research/prompts/gate-shrink-r1-local-attack-output-void1.md  cjk=0 words=4558 fffd=0
汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0
缩写自粘=0 单字母替换=1`，下一行`     单字母替换: alter×84（提示里是 after）`。
对 void1 跑 `oov-check.py`（带提示文件）：判绿，`绿 ... 生词=0 拼接=0`。
判定：作废，参考样本，损坏处是「alter」替换「after」，在样本里重复出现 84 次
（对应提示里 84 格 T{n}-Q2 每格各自的 refuted-if 分句）。

第 2 次调用：同一条命令，改用 `>|` 重定向到同一个号
`research/prompts/gate-shrink-r1-local-attack-output-s1.md`（跑前已确认该文件为 0 字节）。
退出码：5。`s1.md` 再次确认为 0 字节。脚本留下作废副本
`research/prompts/gate-shrink-r1-local-attack-output-void2.md`（32471 字节）。
对 void2 跑 `corruption-check.py`（带提示文件）：判红，原样行——
`红 research/prompts/gate-shrink-r1-local-attack-output-void2.md  cjk=0 words=5728 fffd=0
汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0
缩写自粘=0 单字母替换=1`，下一行`     单字母替换: alter×84（提示里是 after）`。
对 void2 跑 `oov-check.py`（带提示文件）：判绿，`绿 ... 生词=40 拼接=0`，
生词表（前 300 字符内，未打满）：
`records's discipline's experiments's tooling's separate's section's`。
判定：作废，参考样本，损坏处同上——「alter」替换「after」，重复 84 次。

第 3 次调用：同一条命令，仍用 `>|` 重定向到
`research/prompts/gate-shrink-r1-local-attack-output-s1.md`（跑前已确认为 0 字节）。
退出码：0。词数（`wc -w`）：3932。`corruption-check.py`（带提示文件）判绿：
`绿 ... cjk=0 words=4003 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0
星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0`。`oov-check.py`（带提示文件）
判绿：`绿 ... 生词=0 拼接=0`。逐行通读一遍全文（84 组 T1 至 T14 的 Q1/Q2、
Q-COUNT-1 至 3、TB-Q1 至 4）：每一行自成句，没有缺词导致的孤立标点，
也没有粘连词或整词消失的迹象。判定：干净。

第 4 次调用：`bash research/scripts/ask-local.sh research/prompts/gate-shrink-r1-local-attack.md > research/prompts/gate-shrink-r1-local-attack-output-s2.md`。
退出码：0。词数（`wc -w`）：2341。`corruption-check.py`（带提示文件）判绿：
`绿 ... cjk=0 words=2197 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0
星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0`。`oov-check.py`（带提示文件）
判绿：`绿 ... 生词=0 拼接=0`。逐行通读一遍全文：每一行自成句，没有缺词导致的
孤立标点，也没有粘连词或整词消失的迹象。判定：干净。

## 汇总

干净样本 2 份：
`research/prompts/gate-shrink-r1-local-attack-output-s1.md`（第 3 次调用，3932 词）、
`research/prompts/gate-shrink-r1-local-attack-output-s2.md`（第 4 次调用，2341 词）。

参考样本（带损坏）2 份，损坏处均为「alter」误替换「after」、在样本里各重复 84 次：
`research/prompts/gate-shrink-r1-local-attack-output-void1.md`（第 1 次调用，28384 字节，
corruption-check 报 words=4558）、
`research/prompts/gate-shrink-r1-local-attack-output-void2.md`（第 2 次调用，32471 字节，
corruption-check 报 words=5728）。

作废样本：0 份（作废内容已归入上面两份参考样本，`-output-void<n>.md` 是脚本对判红
那次调用留下的副本，不算额外一类）。

连续调用次数：4 次（前 2 次判红、后 2 次判绿），未到「连续五次拿不到两份干净就停」
的上限。两份干净样本已够「至少两份」的门槛，停止继续抽样。

核对表：`research/prompts/gate-shrink-r1-local-attack-translation-audit.md`。

## 算术核对（5b）

主 agent 这一轮没有另给算好的答案表路径。没有答案表，算术没比。
