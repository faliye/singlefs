# gate-fix-forks-r3 本地攻方运行记录

提示文件：`research/prompts/gate-fix-forks-r3-local-attack.md`
核对表：`research/prompts/gate-fix-forks-r3-local-attack-translation-audit.md`
样本前缀：`research/prompts/gate-fix-forks-r3-local-attack`

跑前先对提示文件本身跑过 `corruption-check.py`、`oov-check.py`：两者都判绿
（`corruption-check.py`：cjk=531 words=4229 fffd=0 汉字复读=0 英文复读=0 反引号落单=0
星号落单=0 粘连=0 实词自复读=0 缩写自粘=0；`oov-check.py`：生词=17 拼接=0），
不是这一轮判红的成因。

## 调用记录（按发生顺序，前台跑，未用 setsid / & / disown）

第 1 次调用：`bash research/scripts/ask-local.sh research/prompts/gate-fix-forks-r3-local-attack.md > research/prompts/gate-fix-forks-r3-local-attack-output-s1.md`。
退出码：0。词数（`wc -w`）：389。`oov-check.py` 判定：绿，生词=0，拼接=0。
`corruption-check.py` 判定：绿（cjk=0 words=384 fffd=0 汉字复读=0 英文复读=0
反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）。逐行通读一遍：15 行答案
（T4-Q1 至 T4-Q8，T8-Q1 至 T8-Q7）各自成句，没有缺词导致的孤立标点，也没有列表
里整词消失的迹象。判定：干净。

第 2 次调用：同一条命令，重定向到 `research/prompts/gate-fix-forks-r3-local-attack-output-s2.md`。
退出码：0。词数：726。`oov-check.py` 判定：绿，生词=3（apparatus、blklogwrites、
contradicted），拼接=0。`corruption-check.py` 判定：绿（cjk=0 words=718 fffd=0
汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）。
逐行通读一遍：15 行答案各自成句，没有缺词导致的孤立标点，也没有列表里整词消失
的迹象。判定：干净。

## 汇总

干净样本 2 份：`gate-fix-forks-r3-local-attack-output-s1.md`、
`gate-fix-forks-r3-local-attack-output-s2.md`。
带损坏样本：0 份。
作废样本：0 份（`ask-local.sh` 目录下未生成任何 `-output-void*.md` 文件）。
连续调用次数：2 次，均判绿，未到「连续五次拿不到两份干净就停」的上限。
两份干净样本已够「至少两份」的门槛，停止继续抽样。
