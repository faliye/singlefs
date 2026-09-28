# k3-9-barrier-r1 本地攻方腿：运行记录

提示文件：`research/prompts/k3-9-barrier-r1-local-attack.md`（438 行，英文，无 markdown 强调，事实号 Fact 1–23，行标签 R1–R5，列标签 C1–C4）。
核对表：`research/prompts/k3-9-barrier-r1-local-attack-translation-audit.md`。
分工：正文「四、格」W3——结局网格，本地攻方按写死的事实表逐格推，不跑代码。

## 调用记录

第 1 次调用：`bash research/scripts/ask-local.sh research/prompts/k3-9-barrier-r1-local-attack.md > research/prompts/k3-9-barrier-r1-local-attack-output-s1.md`，退出码 5（字词损坏闸判红）。
- `research/prompts/k3-9-barrier-r1-local-attack-output-s1.md`：判红那次重定向建出的空文件，已确认 0 字节（`ls -la` 原样：`-rw-rw-r-- 1 fy5090 fy5090 0 Sep 28 07:08`）。
- `research/prompts/k3-9-barrier-r1-local-attack-output-void1.md`：脚本留的作废副本，9104 字节，词数 1498（`corruption-check.py` 报的 `words=1498`），参考样本。
  - `corruption-check.py` 原样：`红 research/prompts/k3-9-barrier-r1-local-attack-output-void1.md  cjk=0 words=1498 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=1` 接一行 `     单字母替换: affect×10（提示里是 effect）`。
  - `oov-check.py` 原样：`绿 research/prompts/k3-9-barrier-r1-local-attack-output-void1.md  生词=0 拼接=0`。
  - 损坏处：全文十处「does not affect」，落在第 3、4、7、8、11、12、15、16、19、20 行（`grep -n affect` 原样定位）；判红理由是 `corruption-check.py` 的单字母替换签名（提示里出现的是「effect」，样本里全部写成「affect」），这十处不用作干净样本，但每一处都是「does not affect」这个完整短语、不是缺头粘连词，其余部分（各格结论、判据、事实号引用）按「一条腿只抽一次样不算一次观测」一节当参考样本处理，主 agent 判决时现查。

第 2 次调用（判红那次的补跑，同号 `s1`）：`bash research/scripts/ask-local.sh research/prompts/k3-9-barrier-r1-local-attack.md >| research/prompts/k3-9-barrier-r1-local-attack-output-s1.md`，退出码 0。
- `research/prompts/k3-9-barrier-r1-local-attack-output-s1.md`：8107 字节，词数 1417（`wc -w` 原样）。
- `oov-check.py` 原样：`绿 research/prompts/k3-9-barrier-r1-local-attack-output-s1.md  生词=0 拼接=0`（生词表 0 项，未打满 300 字符，不必整份通读判满）。
- 通读结果：整份逐行读过，没有缺头粘连词、没有缺词或断句类损坏。判定：干净样本。

第 3 次调用：`bash research/scripts/ask-local.sh research/prompts/k3-9-barrier-r1-local-attack.md > research/prompts/k3-9-barrier-r1-local-attack-output-s2.md`，退出码 0。
- `research/prompts/k3-9-barrier-r1-local-attack-output-s2.md`：7434 字节，词数 1198（`wc -w` 原样）。
- `oov-check.py` 原样：`绿 research/prompts/k3-9-barrier-r1-local-attack-output-s2.md  生词=0 拼接=0`。
- 通读结果：整份逐行读过，没有缺头粘连词、没有缺词或断句类损坏。判定：干净样本。

## 5b 算术核对

本轮派发提示没有给算好的答案表路径（W3 是逐格分类 + 红/不红判据的推理网格，不是纯算术题）；没有答案表，算术没比。

## 汇总

干净样本 2 份：
- `research/prompts/k3-9-barrier-r1-local-attack-output-s1.md`
- `research/prompts/k3-9-barrier-r1-local-attack-output-s2.md`

参考样本 1 份（作废副本，字词损坏闸判红）：
- `research/prompts/k3-9-barrier-r1-local-attack-output-void1.md`

三次调用均计入号池（`s1` 占两次调用中的号，`s2` 占一次调用），连续调用次数 3 次即拿到两份干净样本，未触发「连续五次调用拿不到两份干净的」停止条款。
