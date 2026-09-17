# 本地攻方运行记录：alloc-basis-r3

提示文件：`research/prompts/alloc-basis-r3-local-attack.md`（英文，自足；只攻 Z3′——串-重判，判据 V3 假性 ENOSPC 与卡死，不碰 Z1′/Z2′/Z4′/Z5′）。
核对表：`research/prompts/alloc-basis-r3-local-attack-translation-audit.md`（12 条转述/抽取逐条核对，另有机械抽取自证一节）。
草稿目录：`/tmp/claude-1000/alloc-basis-r3-local-attack/`（本轮未用到，全部草稿留在会话自己的 scratchpad，产物直接落地到 `research/prompts/`）。

提示里给了一段从第二轮 Opus 攻方腿模型真实产物机械抽取的历史（`research/prompts/alloc-basis-r2-opus-model/outputs/z3-results.txt` 第 255–323 行，乙′ 读法=串-重判，H1 那一段，69 行全部机械解析，数字未改动），在这段历史的 txg=12 状态上搭了两个第二轮模型没建的新场景：Section SA（同窗两个 16 块请求的在飞合成）、Section SB（推空发布之中崩溃、第一条带新 F 的根只落一块盘，之后重开、写行那次发布不推空、用户数据重做照走准入）。问题分四组：SA（问题 1–4）、SB（问题 5–8）、跨两场景的假性 ENOSPC（问题 9）、跨两场景的界 3 可用性（问题 10–12）。要求每条答案带「This would be refuted by」一句，不许只答 yes/no，表格要给具体数字。

两次前台调用（未用 setsid / 后台 / disown），均命中同一个网关 `http://127.0.0.1:8200/v1/chat/completions`。

## 样本 1：`alloc-basis-r3-local-attack-output-s1.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/alloc-basis-r3-local-attack.md > research/prompts/alloc-basis-r3-local-attack-output-s1.md`
- 退出码：0
- 词数：`wc -w` = 293；`wc -l` = 35
- `ask-local.sh` 内部串联的 `corruption-check.py`/`oov-check.py` 均已判绿（否则会退出码 5、产物落到 `-output-void*.md`，这里没有触发）
- 独立复跑 `oov-check.py`：绿，生词=0 拼接=0，退出码 0
- 通读复核（缺词、断句这类闸抓不到的损坏）：12 条编号答案逐条读过，每条都是完整短语加一句「This would be refuted by」，没有列表里整词缺失、没有孤零零标点的断句、没有缺头粘连词
- 内容形态：极简——多数答案只给数字加两三个词的判断语，没有把「the arithmetic」摊开写（提示第 4、6、9 问都明确要求「show the arithmetic」），第 6、8 问尤其简短，没有说明 gate1=5、「absence of rule」这两句判断是怎么从给定规则推出来的；这是内容完整性观察，不改变字词损坏闸的判定
- 判定：干净

## 样本 2：`alloc-basis-r3-local-attack-output-s2.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/alloc-basis-r3-local-attack.md > research/prompts/alloc-basis-r3-local-attack-output-s2.md`
- 退出码：0
- 词数：`wc -w` = 396；`wc -l` = 35
- `ask-local.sh` 内部串联的字词损坏闸均已判绿
- 独立复跑 `oov-check.py`：绿，生词=1（subtracts）拼接=0，退出码 0
  - 复核「subtracts」：完整、拼写正确的英文动词第三人称单数变位（subtract + s），样本第 4 行「Definition (ii) does not match because it subtracts in_flight_approved」，语义通顺、前后没有粘连迹象（不是「subtractsin」这类无空格粘连，也不是缺头词），只是不在生词表词典里，不判带损坏
- 通读复核：12 条编号答案逐条读过，比样本 1 更完整——第 2 问额外给了「definition (i) 为什么对」的一句解释，第 4/9 问把算术摊开写了（`df=37 >= 6, but admission reading=37-32=5 < 6`），没有整词缺失或断句
- 判定：干净

## 两份样本之间的一致与差异（如实记录，不解读、不裁决）

逐题核对（按提示的编号 1–12）：

- 问题 1（Q 的准入读数与结果）：两份完全一致——21、admitted、in_flight_approved 之后为 32。
- 问题 2（df 的两种候选定义）：两份完全一致——definition (i) 给 37、definition (ii) 给 5，两份都选 definition (i) 为「实际被措辞支持的那一种」。样本 2 多给了一句理由（in_flight_approved 不是「已发布统计量」的一部分），样本 1 没给理由、只给了结论。
- 问题 3（第三个请求 R 的准入读数上界）：两份完全一致——5。
- 问题 4（df≥s 而写 s 失败的具体例子）：两份完全一致——都给 s=6、df=37、准入读数=5、结论「yes，存在这样的 s」。这是本轮在 SA 场景下两份样本独立复现的同一个假性-ENOSPC 形状的具体反例（按 definition (i) 读法）。
- 问题 5（F_effective 崩溃重开后的值）：两份完全一致——0，与主 agent 提示里给出的待核「tentative reading」一致（两份样本都认可这条 tentative reading，没有给出不同的 F_effective）。
- 问题 6（gate1 在写行发布评估 R 的重做写入时的值）：两份完全一致——5，与主 agent 的 tentative 推断不同（主 agent 在提示第 6 问的括注里暗示「若没有别的计数器被崩溃影响，会与 txg=12 的 gate1=9 相同」；两份样本都没有采用 9，都给了 5，暗示模型认为崩溃里已经持久到一块盘上的那次推空发布的资源开销（如 defer_pending 的增量）没有被那次崩溃撤销，仍然按单盘计入了这一盘自己的账）。两份样本都没有展开这一步的推导过程（样本 1 给了「if defer_pending was not increased on one device」这句反证条件，暗示了同样的机制；样本 2 只给了「if gate1 was not 5, such as 9」，没有解释为什么是 5 不是 9）。
- 问题 7（R 的重做写入结果）：两份完全一致——ENOSPC，gate1=5<16。
- 问题 8（重复崩溃模式下 R 会不会永远失败）：两份结论一致——R 的写永远不会成功。样本 1 额外给了归因「absence of rule to handle inconsistent state」（没有规则处理这种不一致状态），样本 2 没有给出对应的归因句，只重申了结论。
- 问题 9（跨 SA/SB 的假性 ENOSPC 问题）：两份完全一致——SA 是 yes（与问题 4 同一个反例），SB 是 no（df 与实际准入读数都是 5，不存在「报出的比实际能用的多」这件事，SB 的失败是「真的不够」不是「假报」）。
- 问题 10（SA 里有没有隐含的 delete）：两份完全一致——SA 没有 delete，界 3 那条要求不直接适用。
- 问题 11（写行发布算不算改变用户可见状态的发布）：两份完全一致——算，因为它捎带了用户数据。
- 问题 12（界 3 要求管不管 R 这种没有前置 delete 的情形）：两份完全一致——不管，界 3 只管「删除之后的重写」，不管没有前置删除的普通新写。

12 道题里没有一题两份样本给出相反的数字或相反的 yes/no 结论；差异只在「有没有把推导过程摊开写」这一层，不影响数字本身。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，这两份是独立的两次抽样，结论一致，可以合起来记「两份样本一致」。

## 作废副本

无。两次调用均未触发字词损坏闸（退出码均为 0），`research/prompts/` 目录下 `alloc-basis-r3-local-attack` 前缀只有 `-output-s1.md` 与 `-output-s2.md` 两个产物文件（现查 `ls research/prompts/alloc-basis-r3-local-attack-output-*` 只有这两个）。

## 样本数

达到「至少两份干净样本」的要求，两次调用（s1、s2）全部判绿，未消耗任何重试额度，未触及「连续五次调用（判红作废的也算）拿不到两份干净的」这条停下条件。

## 本轮没做什么

- 不解读、不判定「打没打中」：问题 4/9 里两份样本独立给出的 df=37 而写 6 块失败这个具体反例，是否构成对 V3（假性 ENOSPC 与卡死）的有效攻击，是否推翻串-重判这条臂，留给主 agent 与 verifier 判断。
- 不评价问题 6/8 两份样本给出的 gate1=5、「absence of rule」这两句判断本身对不对——它们依据的是提示里明确留给模型自己判断的「tentative reading」分支，本地攻方腿只如实转达，不核实它是不是全仓已有条款支持的读法。
- 没有攻 Z1′/Z2′/Z4′/Z5′，按分工只攻 Z3′。
- 没有用 `ASK_LOCAL_ALLOW_CORRUPT=1`，两次都是干净判定，不需要这个开关。
