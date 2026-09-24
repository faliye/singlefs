# 本地攻方运行记录：c355-c363-r3（W3）

提示文件：`research/prompts/c355-c363-r3-local-attack.md`（英文，自足，无 markdown 强调；14 格判据，只覆盖 W3——两种单位写法（一份副本 / 副本之和）分别在 D5ROW（D5 已定项 4 对照表两行）、GATE51（门禁 51 号读的式子）、CRATES（`crates/` 维护处）、INVARIANT（I 类不变量）四处的射程；提示正文显式排除 W1（释放链）与 W2（(b) 改法自己的死角），不让本地腿碰）。
核对表：`research/prompts/c355-c363-r3-local-attack-translation-audit.md`（27 条转述逐句核对，含事实表四行各自的来源文件:行）。
四次前台调用（未用 setsid / 后台 / disown），命令一致：
`nice -n 19 bash research/scripts/ask-local.sh research/prompts/c355-c363-r3-local-attack.md > research/prompts/c355-c363-r3-local-attack-output-<目标文件>`

## 样本 1：`c355-c363-r3-local-attack-output-s1.md`

- 退出码：0
- 词数：`wc -w` = 650；`corruption-check.py` 自报 words=666
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=7（列出 SUMCOPIES、overestimation、subtracts），拼接=0，退出码 0——均为提示自带标签或标准英文词，非损坏形态
- 通读结果：14 项答案齐全；标点计数 `grep -o '[.,;:]'`：逗号 25、句号 43、冒号 14，冒号数与「This would be refuted by:」出现次数相符；未见缺词、断句或孤零标点
- 判定：干净

## 样本 2：`c355-c363-r3-local-attack-output-s2.md`

- 退出码：0
- 词数：`wc -w` = 753；`corruption-check.py` 自报 words=774
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=10（列出 dimensionality、SUMCOPIES、formula's、deducting），拼接=0，退出码 0
- 通读结果：14 项答案齐全，但全文标点异常——`grep -o '[.,;:]'` 计数：句号 16、逗号 0、冒号 0、分号 0；对照样本 1（逗号 25、冒号 14）与样本 3（逗号 15、冒号 14），样本 2 连「This would be refuted by:」这一要求逐字复现的短语自带的冒号都被吞掉，14 项答案各自连成无逗号无冒号的长句，只在每项末尾留一个句号。四类登记签名（复读、成对标记落单、拼接、实词自复读）均未触发，但落在「缺词、断句这类损坏」里「断句」一类——拿不准算不算，按规则从严记为带损坏
- 判定：带损坏（不计入「干净样本」，文件保留、不作废）

## 作废：`c355-c363-r3-local-attack-output-void1.md`

- 产生于向 `-output-s3.md` 重定向的那次调用：该次退出码 5，脚本判红并将原文原样另存为本文件；原定向目标 `s3.md` 当次是空文件，沿用同一个号重跑（未推进到 s4）
- 脚本诊断：`实词自复读=1`，词 `sentinel`；词数 `wc -w` = 921，`corruption-check.py` 自报 words=924
- 这一次判红作废，不算一次观测

## 样本 3：`c355-c363-r3-local-attack-output-s3.md`（沿用被判红那次的号重跑）

- 退出码：0
- 词数：`wc -w` = 565；`corruption-check.py` 自报 words=576
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=12（列出 SUMCOPIES、statistic's、invariant's、deducting、convention's），拼接=0，退出码 0
- 通读结果：14 项答案齐全；标点计数 `grep -o '[.,;:]'`：逗号 15、句号 42、冒号 14、分号 1，与样本 1 同型；未见样本 2 那种断句损坏，也未见缺词或孤零标点
- 判定：干净

## 样本数与停止条件

四次调用（含一次判红作废）取得两份干净样本（样本 1、样本 3），达到「至少两份干净样本」的下限。样本 2 带损坏、不计入干净数，但未耗尽「连续五次调用拿不到两份干净」这条停止条款（实耗 4 次，含 1 次作废）。已停止，不再调用 `ask-local.sh`。
