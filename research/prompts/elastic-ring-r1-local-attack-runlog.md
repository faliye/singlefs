# 本地攻方运行记录：elastic-ring-r1

提示文件：`research/prompts/elastic-ring-r1-local-attack.md`（英文，自足，无 markdown 强调；15 格判据，覆盖 K4（上下限取什么、谁来定）与 K3 里「今天盘上存了什么字段」那几格；不碰 K1、K2、K5、K3 的 incompat 位judgment）。
核对表：`research/prompts/elastic-ring-r1-local-attack-translation-audit.md`（13 条转述逐句核对）。
两次前台调用（未用 setsid / 后台 / disown），命令一致：
`nice -n 19 bash research/scripts/ask-local.sh research/prompts/elastic-ring-r1-local-attack.md > research/prompts/elastic-ring-r1-local-attack-output-sN.md`

## 样本 1：`elastic-ring-r1-local-attack-output-s1.md`

- 退出码：0
- 词数：`wc -w` = 739（含提示回显？不含，`ask-local.sh` 只落模型正文）；`corruption-check.py` 自报 words=720
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=3（DefaultWins、derivable）拼接=0，退出码 0——`DefaultWins` 是提示里自定义的规则标签（非缺头粘连词），`derivable` 是标准英文词，均非损坏形态
- 通读结果：全文 15 行答案逐行读过，句子完整、列表齐全，未见缺词、断句或孤零标点
- 判定：干净

## 样本 2：`elastic-ring-r1-local-attack-output-s2.md`

- 退出码：0
- 词数：`wc -w` = 657；`corruption-check.py` 自报 words=638
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=5（contradicting、DefaultWins、computable）拼接=0，退出码 0——均为标准英文词或提示自带标签，非损坏形态
- 通读结果：第 19 行（G3.DefaultWins 那条）出现词序颠倒「widen would the default range」（应为「would widen」），是普通语序错漏，不属于闸定义的四类损坏（复读、成对标记落单、拼接/粘连、实词自复读）中任何一类，也不是「整段掉了只剩孤零标点」那种断句损坏；其余 14 行未见异常
- 判定：干净（词序颠倒一处已如实记录，未据此改判「带损坏」，因其不落在已登记的损坏签名内）

## 作废副本

无。两次调用均一次性判绿，`research/prompts/` 目录下 `elastic-ring-r1-local-attack` 前缀未生成任何 `-output-void*.md`（已现查目录列表）。

## 样本数与停止条件

两次前台调用即拿到两份干净样本，达到「至少两份干净样本」的下限，未消耗更多重试额度，未触及「连续五次拿不到两份干净的」这条停下条件。
