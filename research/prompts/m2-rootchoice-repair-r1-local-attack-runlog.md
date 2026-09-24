# 运行记录：m2-rootchoice-repair-r1-local-attack（2026-09-22 UTC）

提示文件：`research/prompts/m2-rootchoice-repair-r1-local-attack.md`（英文，K4：C335 链条怎么不再是
吸收态，6 个 judgment 各 3 项子答复 = 18 格；K6：K2/K3/K4/K5 四格改法会不会互相打架，18 个 judgment
各 3 项子答复 = 54 格；合计 72 格）。不用任何 markdown 强调，答案按 judgment 编号，指路只用 fact 编号
或 label（2a/2b/2c、3a/3b/3c、5、4），明令不写文件名或行号（需要指代码时按函数名，本轮未涉及代码）。
转述核对表：`research/prompts/m2-rootchoice-repair-r1-local-attack-translation-audit.md`（含两处真实翻译
错误的发现与改正：Fact 2 把「回退下界 F」误译成"the recovery watermark"，Fact 5 的切换预留公式漏抄
`max(1, ...)` 外层，均已在写提示的同一遍核对里查出并改正，提示文件里没有残留错误版本）。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，前台跑，
未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 过滤
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：零命中，不是性能测量在跑。

## 调用记录

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py（独立跑一遍，读绿之后也跑） | corruption-check.py（独立跑一遍） | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-rootchoice-repair-r1-local-attack-output-s1.md` | 0 | 569 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（`cjk=0 words=509 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`） | 干净 |
| 2 | `m2-rootchoice-repair-r1-local-attack-output-void1.md` | 5 | 678 | 未跑（`ask-local.sh` 判红即退出，未落到能独立复核的位置；脚本自带的 `corruption-check.py` 已判红） | 红（脚本内置输出：`英文复读=2(3.09/千)`，复读样本 `same invariant \| same invariant`），独立复核：该样本正文里确有两处「same invariant same invariant」连续出现、中间没有内容（`judgment 2b-5` 与 `judgment 4-3b` 两行），是真损坏、不是检测器误报 | 作废（这一号按规矩空出重跑，不算一次观测） |
| 2（重跑，沿用同一个号） | `m2-rootchoice-repair-r1-local-attack-output-s2.md` | 0 | 923 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（`cjk=0 words=893 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`） | 干净 |

`oov-check.py` 与 `corruption-check.py` 均在 `ask-local.sh` 判绿之后又对 `output-s1.md`、`output-s2.md`
各自独立跑了一遍（`ask-local.sh` 自己判绿时不打印生词表，独立跑一遍才看得到），两份的生词表都是空
（`生词=0`），不存在「判绿但生词表里有缺头粘连词」的情况。

通读两份干净样本全文（`output-s1.md` 120 行、`output-s2.md` 120 行）：K4 六个 judgment、K6 十八个
judgment 均给出 part one/two/three 三项，没有缺词、断句、孤立标点或列表项整词缺失；`output-s1.md`
的三项答复普遍比 `output-s2.md` 简短（前者多为「fact 编号 + 短语」，后者多为完整句子），但两份都是
完整作答，不构成「带损坏」的判据（判断答复详略、方向是否一致不归这份运行记录，见下方「没做什么」）。

## 干净样本清点

- 2 份干净（`output-s1.md`、`output-s2.md`），达到「至少两份干净样本」的门槛，未继续抽样。
- 1 次判红作废（退出码 5，`output-void1.md`），沿用同一个序号（2）重跑后得到 `output-s2.md`；
  全程调用次数（含作废）= 3 次，未触达「连续五次调用拿不到两份干净的」停下条款。
- 没有退出码非 0 非 5、也没有网关不通的情况；本地腿全程未缺席。
- 提示文件本身也过了一遍 `corruption-check.py`（脚本内置，跑在 `output-void1.md` 那次调用的诊断输出
  里）：`绿 research/prompts/m2-rootchoice-repair-r1-local-attack.md  cjk=0 ... 全部计数为 0`，提示文件
  本身不带中文、不含损坏形态。

## 没做什么

- 不解读、不总结、不采纳两份样本对 K4（C335 链条哪一环可断、断开后哪条已定条款会红）、K6（K2/K3/K4/K5
  改法两两冲不冲突）给出的具体判词，不比较两次抽样之间方向是否一致——那是主 agent 的事。
- 未跑云端攻方腿（Opus，K2/K5）或云端正推腿（Sonnet，K1/K3），不在这条腿的任务范围内；未读
  `research/prompts/m2-rootchoice-repair-r1-opus-output.md`、`-sonnet-output.md`（这一轮另两条腿的产出，
  按「各条腿必须互不重复」不读）。
- K6 的 72 格里，K4 候选（label 4）与 K2/K3/K5 候选的交叉只挑了「模型自己判断最可能碰系统配置或根环
  字节的那一个」与 2b、3b、5 各配一次（3 对），未把 K4 六个 judgment 的全部候选与 2a/2b/2c/3a/3b/3c/5
  逐一交叉（6×7 = 42 对全交叉未做）；这一取舍已在提示正文与翻译核对表的「多出来的限定词」一节写明，
  不是穷举，是这一次调用规模内的取舍。
- 未验证两份样本里对 K4/K6 给出的判词本身站不站得住——判它有没有打中、够不够格采信是主 agent 的事。
