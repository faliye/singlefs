# 本地攻方运行记录（gate-fix-forks-r2-local-attack）

提示文件：`research/prompts/gate-fix-forks-r2-local-attack.md`
核对表：`research/prompts/gate-fix-forks-r2-local-attack-translation-audit.md`

## 调用记录（按发生顺序，前台跑，未用 setsid/后台）

第 1 次调用：`bash research/scripts/ask-local.sh research/prompts/gate-fix-forks-r2-local-attack.md > research/prompts/gate-fix-forks-r2-local-attack-output-s1.md`
退出码：5（闸判红，按规则整轮作废）。判红原因：不是模型答复本身，是提示文件自己被
`corruption-check.py` 判红——提示文件里 T8 部分四行（当时的 Row 17-20）重复使用了
「很长很长」这一中文双字叠词，命中它的汉字二元组复读检测（汉字复读=12(10.09/千)）；
同时提示文件里多处用了 Python 切片写法 `text[:n]`、双冒号 `scenario::run_first_transaction`，
命中它的「标点粘连」检测（粘连=31，后来陆续修到 0）。作废副本由脚本自动留存：
`research/prompts/gate-fix-forks-r2-local-attack-output-void1.md`；这一次重定向建出的
`gate-fix-forks-r2-local-attack-output-s1.md` 是 0 字节（脚本判红时在 `cat "$TXT"` 之前就
`exit 5`，stdout 没收到任何内容）。
处置：改写提示文件本身消掉这三类假阳性根源（改用不含叠词的填充句、把 `[:n]` 全部
改写成 `[0:n]`、`scenario::` 后面加一个空格、`SyntaxError` 改成两个词 `Syntax Error`），
不改提示要考的事实或数据，逐条改动都用 `research/scripts/replace-batch.py` 定点替换、
回读确认；改完再跑 `corruption-check.py`、`oov-check.py` 确认提示文件本身判绿。
沿用同一个号 s1 重跑（未跳号）。

第 2 次调用：同一条命令，重定向到同一个文件名 `...-output-s1.md`（覆盖第 1 次调用留下的
0 字节文件）。退出码：0。词数（`wc -w`）：699。`oov-check.py` 判定：绿，生词 2 个
（`parenthetical`）。`corruption-check.py` 判定：绿（cjk=0 words=695 fffd=0 汉字复读=0
英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）。逐行通读一遍，没有
缺词、断句、孤立标点这类闸抓不到的损坏。判定：干净。

第 3 次调用：同一条命令，重定向到 `...-output-s2.md`。退出码：0。词数：695。
`oov-check.py` 判定：绿，生词 2 个（`parenthetical`）。`corruption-check.py` 判定：绿
（cjk=0 words=693 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0
缩写自粘=0）。逐行通读一遍，没有缺词、断句、孤立标点这类损坏。判定：干净。

## 汇总

干净样本 2 份：`gate-fix-forks-r2-local-attack-output-s1.md`、
`gate-fix-forks-r2-local-attack-output-s2.md`。
带损坏样本：0 份。
作废样本：1 份（`gate-fix-forks-r2-local-attack-output-void1.md`，退出码 5，
原因是提示文件本身触发字词损坏闸，不是模型答复的问题；详见上文第 1 次调用）。
连续调用次数：3 次（1 次作废 + 2 次判绿），未到「连续五次拿不到两份干净就停」的上限。
两份干净样本已够「至少两份」的门槛，停止继续抽样。
