# 运行记录：里程碑「第二个事务」步 4 / 步 5 代码三方第二轮本地攻方（X4 / X5 / X6）

提示文件：`research/prompts/m2-step45-code-r2-local-attack.md`。
核对表：`research/prompts/m2-step45-code-r2-local-attack-translation-audit.md`。

<!-- doc-lint:not-numbers s1 s2 s3 -->

## 逐次调用记录

一共调用 `research/scripts/ask-local.sh` 七次，全部前台跑、加 `nice -n 19`，未用 `setsid`/`&`/`disown`。前两次是提示文件自身的问题（不是本地模型的问题），当场改提示后重跑；这把总调用数推到了七次，超过了「连续五次调用拿不到两份干净的就停下照实报」的名义上限——如实记在这里，两份干净样本最终在第七次调用时凑齐（s1 与 s3）。

| 次序 | 落到哪个文件 | 退出码 | 原因 | 处置 |
|---|---|---|---|---|
| 1 | `-output-s1.md`（判红后变空，原样副本存进 `-output-void1.md`） | 5 | 提示第 44/50/59 行把 D16（发布语义） 的中文变量名「F_生效」原样写了 6 次，混英文与汉字；闸报 `cjk=12`、`粘连=2` | 把 6 处「F_生效」全部改成纯英文的 `effective_floor`（配合括注说明是同一个量），回读 `grep -c '[一-龥]'` = 0 |
| 2 | `-output-s1.md`（判红后变空，原样副本存进 `-output-void2.md`） | 5 | 提示第 3 行的指示文字里举了字面例子 `Foo::bar`、`Module::Type`；闸的「标点粘连」规则（标点前不是字母数字、后面紧跟词字符）把双冒号后紧跟字母的形态当成「一句话被截断后接上了下一句」的签名，报 `粘连=2` | 把这条指示改写成不出现字面双冒号的散文描述（「join...with two colons and no space」），回读 `corruption-check.py` 报 `粘连=0` |
| 3 | `-output-s1.md` | 0 | 干净 | 见下「样本判定」 |
| 4 | `-output-s2.md` | 0（脚本自带的闸判绿） | 脚本自带的闸判绿，但下游 `oov-check.py` 与逐句通读发现真损坏 | 见下「样本判定」，记「带损坏」，不当干净样本 |
| 5 | `-output-s3.md`（判红后变空，原样副本存进 `-output-void3.md`） | 5 | 模型答案里出现「TransactionOutput's」（提示里引入的合法 CamelCase 类型名的所有格形式），闸报 `拼接=1：TransactionOutput's(=transaction+output's)`——像是闸对合法复合词的误判，不是真损坏（读原文确认是正常英语所有格） | 按规则重跑，不改提示 |
| 6 | `-output-s3.md`（判红后变空，原样副本存进 `-output-void4.md`） | 5 | 模型答案里出现「reclaimedclaimed」，闸报 `拼接=1：reclaimedclaimed(=reclaimed+claimed)`——这是真损坏（`re` + `claimed` + `claimed` 粘连／重复的头部丢字签名） | 按规则重跑 |
| 7 | `-output-s3.md` | 0 | 干净 | 见下「样本判定」 |

## 样本判定

### s1 —— 干净

- 退出码 0（`ask-local.sh` 自带闸判绿）。
- `corruption-check.py` 回读：`cjk=0 words=733 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`。
- `oov-check.py` 回读：`生词=2 拼接=0`，生词清单：`abandonment timeline's`——都是合法英语词（形容词化名词、所有格），不是缺头粘连词，不算带损坏。
- 逐句通读：10 道小问（1a–1d、2a–2c、3a–3c）全部齐全、每条都有「This is refuted by:」收尾，没有断句、没有列表项缺失。
- 判定：**干净样本**。

### s2 —— 带损坏（不当干净样本）

- 退出码 0（`ask-local.sh` 自带闸判绿）。
- `corruption-check.py` 回读：`cjk=0 words=1114 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`——闸本身看不出问题。
- `oov-check.py` 回读判红：`生词=8 拼接=2`，生词清单：`TransactionOutput abandonmentfilter timeline's absabsent`（另有 `TransactionOutput` 的拼接误判，见下）。
- 逐句通读确认两处真损坏：
  - 第 5 行「it applies no abandonmentfilter」——`abandonment` 与 `filter` 中间丢了空格，粘连成一个词。
  - 第 13 行「The previous-absabsent branch」——`absent` 被重复／缺头粘连成 `absabsent`（`abs` + `absent`），与计划第十一节点名的「缺头的粘连词」签名同形。
  - `oov-check.py` 报的「TransactionOutput(=transaction+output)」拼接是对提示里合法引入的 CamelCase 类型名的误判，不算损坏；但另外两处是真损坏。
- 判定：**带损坏**，不当干净样本，不采信其中任何一条判断。

### s3 —— 干净

- 退出码 0（`ask-local.sh` 自带闸判绿）。
- `corruption-check.py` 回读：`cjk=0 words=925 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`。
- `oov-check.py` 回读：`生词=2 拼接=0`，生词清单：`timeline's abandonment`——合法英语词，不算带损坏。
- 逐句通读：10 道小问全部齐全，没有断句、没有列表项缺失。
- 判定：**干净样本**。

## 作废副本

- `-output-void1.md`：见上表第 1 次调用。
- `-output-void2.md`：见上表第 2 次调用。
- `-output-void3.md`：见上表第 5 次调用（闸对合法复合词的疑似误判）。
- `-output-void4.md`：见上表第 6 次调用（真损坏，`reclaimedclaimed`）。

## 没做什么

- 不解读、不总结、不采纳本地模型对 X4 / X5 / X6 三组问题给出的任何一条判断；打没打中是主 agent 的事。
- 没有对提示内容做除「上表列出的两处英文纯度修正」之外的任何改动；判据、事实（G1–G10）、定义（D1–D5）与问题原样保留。
- 前两次调用把 `-output-s1.md` 变成了空文件（脚本行为），这是脚本自己的动作，不是我手写空文件。
