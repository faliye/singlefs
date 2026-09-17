# 本地攻方运行记录：alloc-basis-r2

提示文件：`research/prompts/alloc-basis-r2-local-attack.md`（英文，自足；只攻 Z4——影子账的三种读法，判据 W2/W3/W4，不碰 Z1/Z2/Z3/Z5）。
核对表：`research/prompts/alloc-basis-r2-local-attack-translation-audit.md`（15 条转述逐句核对）。
草稿目录：`/tmp/claude-1000/alloc-basis-r2-local-attack/`（本轮未用到，产物直接落地）。

这一轮按主 agent 的换形态要求，提示里给了一张里程碑固定脚本（txg 3 到 17）的英文事实表（每条根的 txg、实例、区域、槽、F、哪些单元何时被谁释放），并要求模型按 Z4A/Z4B/Z4C/Z4D/Z4E 五组问题填具体槽号与具体数字的表，不许只答 yes/no。

两次前台调用（未用 setsid / 后台 / disown），均命中同一个网关 `http://127.0.0.1:8200/v1/chat/completions`。

## 样本 1：`alloc-basis-r2-local-attack-output-s1.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/alloc-basis-r2-local-attack.md > research/prompts/alloc-basis-r2-local-attack-output-s1.md`
- 退出码：0
- 词数：`wc -w` = 500；`corruption-check.py` 自报 words=439（两个计数器口径不同，均如实记录）
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=1（timepoint）拼接=0，退出码 0
  - 复核「timepoint」：不是缺头粘连词。提示文件本身第 331、333 行就用了「timepoints」「timepoint」这个复合词（"do all three readings produce the exact same lowest-timepoints in Z4A" / "the first timepoint (txg 10, txg 16, or txg 17)"），样本只是原样用了提示里给的词，不判带损坏。
- 通读复核：逐条答案（1–13 对应 Z4A 九格 + Z4B + Z4C + Z4D + Z4E）都是具体槽号、具体释放代、具体隔离数，没有列表里整词缺失、没有孤零零标点的断句。
- 判定：干净

## 样本 2：`alloc-basis-r2-local-attack-output-s2.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/alloc-basis-r2-local-attack.md > research/prompts/alloc-basis-r2-local-attack-output-s2.md`
- 退出码：0
- 词数：`wc -w` = 615；`corruption-check.py` 自报 words=593
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=3（timepoint、checker's、overlapped）拼接=0，退出码 0
  - 复核「timepoint」：同样本 1，提示自带的词。
  - 复核「checker's」：提示文件第 230、232、367、369 行原样有「the checker's candidate-set」「the checker's candidate set」，样本只是原样用了这个带撇号的所有格，不是粘连。
  - 复核「overlapped」：样本第 100 行「if either invariant detected the corruption when U1 was overlapped」——完整、拼写正确的英文过去分词，语义通顺（U1 被后来的写覆盖），不是缺头粘连词，也不是新造词。
- 通读复核：逐条答案（Z4A 九格 + Z4B + Z4C + Z4D + Z4E，用提示要求的编号原样标注）同样是具体槽号、具体数字，没有断句或整词缺失。
- 判定：干净

## 两份样本之间的差异（如实记录，不解读、不裁决）

逐格核对（样本 1 用数字编号 1–13，样本 2 用 Z4A/Z4B/... 编号，按同样的问题顺序一一对应）：

- Z4A 九格（narrow/conservative/G5 × txg10/txg16/txg17）：两份样本的 (a) 隔离状态、(b) 隔离数、(c) 与 34 之差、(d) 释放代与回收门槛、(e) 最低可发槽对，逐格数字完全一致。
- Z4B（三种读法是否等价）：两份都答「不等价，首次不同点在 txg16，narrow 给 50176-50177，conservative 给 50178-50179」；样本 2 额外点名 G5 在 txg16 与 conservative 同值（样本 1 没有单独点出 G5 是否与 narrow 在 txg16 也不同，只挑了 narrow/conservative 这一对）——两份都只答了「哪两种读法先不同」里的一对，没有矛盾，只是挑的对不完全一样详尽。
- Z4C（主句检查）：两份完全一致——narrow 的最低可发槽对（50176-50177）被被抛弃根 B 引用、构成违反主句；conservative 与 G5 的最低可发槽对（50178-50179）不被任何被抛弃根引用、不违反。
- Z4D（假性 ENOSPC）：两份都答「在这份脚本里不会发生」，但论证角度不同——样本 1 说「反正还有别的空闲槽可用」；样本 2 说「那 2 个多隔离的槽本来就该隔离（因为被 B 引用），conservative 报的 ENOSPC 是『准』的，narrow 不隔离反而可能造成损坏」。两份的结论方向一致（这份脚本里不会假性 ENOSPC），论证不同，如实并列，不裁决哪个更对。
- Z4E（可验证性）：两份完全一致——A 在 txg17 不在 checker 候选集里（3 < F_effective=11），I-2.1 与 I-3.1 都不会抓到 U1 被覆盖这件事。

## 作废副本

无。两次调用均未触发字词损坏闸，`research/prompts/` 目录下 `alloc-basis-r2-local-attack` 前缀只有 `-output-s1.md` 与 `-output-s2.md` 两个产物文件（`ls` 已现查，见上）。

## 样本数

达到「至少两份干净样本」的要求，两次调用（s1、s2）全部判绿，未消耗任何重试额度，未触及「连续五次拿不到两份干净的」这条停下条件。

## 与上一轮的差异（回应主 agent 对上一轮「没被有效攻过」的判词）

上一轮（alloc-basis-r1）本地攻方两份样本只给了 yes/no 加一句反驳条件，没有构造出带设备数、逐次发布分配/释放、F 值、槽号的具体历史。这一轮换了形态：提示里先给一张主 agent 从里程碑固定脚本推出的完整事实表（每条根的 txg/实例/区域/槽/F、四个单元 U1-U4 的分配与释放事件、abandoned-root 集合、conservative 读法已实测的两个锚点数 34/36 与 50178），再要求模型按 Z4A 九格逐格填具体槽号、具体隔离数、具体释放代与回收门槛的对比。两份样本都给出了这样一段可核的具体状态（不是抽象论断），其中 Z4C 的结论——narrow 读法在这份真实固定脚本上会把一个仍被被抛弃根 B 引用的槽（50176-50177）判定为可发，构成违反主句——与主 agent 自己独立推导的结果逐字一致，两份样本各自独立复现了这条结论。
