# 运行记录：m2-supp3-item4-code-r1-local-attack（2026-09-21 UTC）

提示文件：
- `research/prompts/m2-supp3-item4-code-r1-local-attack-k3.md`（英文，K3：设备说谎那一类该不该进；3 道 judgment，各 3 项子答复，9 格）
- `research/prompts/m2-supp3-item4-code-r1-local-attack-k6.md`（英文，K6：收口表第 40 行那一格「只记不判」；3 道 judgment，各 3 项子答复，9 格）

两份均不用任何 markdown 强调，答案按 judgment 编号，指路只用 fact 编号，明令不写文件行号或代码行号
（需要指代码时按函数名）。均在「一份不超过 16 格」之内（各 9 格），未拆分、未撞输出预算。
转述核对表：`research/prompts/m2-supp3-item4-code-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
前台跑，未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：全程零命中，不是性能测量在跑。

## K3（设备说谎那一类该不该进，3 道 judgment × 3 项子答复 = 9 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-supp3-item4-code-r1-local-attack-k3-output-s1.md` | 0 | 234 | 绿，生词=8 拼接=0（BlockDeviceError、WriteIsSwallowed、BarrierIsSwallowed、exercises、histories 等，均为提示里给出的标识符或规则表外的正常派生词） | 绿，全部计数为 0 | 干净 |
| 2 | `m2-supp3-item4-code-r1-local-attack-k3-output-s2.md` | 0 | 238 | 绿，生词=6 拼接=0（BlockDeviceError、swallowed、requirement's、histories、exercises） | 绿，全部计数为 0 | 干净 |

通读两份全文：3 道 judgment 各给出 verdict / justification / this would be refuted by 三项，无缺词、无断句、
无孤立标点、无列表项整词缺失。两份干净样本达标，K3 到此为止，不再抽样。

## K6（收口表第 40 行那一格「只记不判」，3 道 judgment × 3 项子答复 = 9 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-supp3-item4-code-r1-local-attack-k6-output-s1.md` | 0 | 231 | 绿，生词=7 拼接=0（BlockDeviceError、sanctioned、OutsideEverythingTheModelAllows、TheVersionTheFaultedStepWasWriting 等，均为提示里给出的标签标识符或正常英文词） | 绿，全部计数为 0 | 干净 |
| 2 | `m2-supp3-item4-code-r1-local-attack-k6-output-s2.md` | 0 | 253 | 绿，生词=4 拼接=0（OutsideEverythingTheModelAllows、TheVersionTheFaultedStepWasWriting） | 绿，全部计数为 0 | 干净 |

通读两份全文：3 道 judgment 各给出 verdict / justification / this would be refuted by 三项，无缺词、无断句、
无孤立标点、无列表项整词缺失。两份干净样本达标，K6 到此为止，不再抽样。

## 干净样本清点（合计）

- K3：2 份干净（s1、s2）。
- K6：2 份干净（s1、s2）。
- 两个提示均未撞输出预算（各 9 格，均在「一份不超过 16 格」之内），未拆分。
- 没有判红作废（退出码 5）的调用，没有 void 副本（`ls research/prompts/ | grep local-attack` 核过，
  目录下只有两份提示、四份 output-sN、一份核对表，没有 output-void 文件）。
- 没有退出码非 0 非 5 或网关不通的情况，本地腿全程未缺席；四次调用全部退出码 0、一次重跑都没用上。

## 没做什么

- 不解读、不总结、不采纳四份样本里对 K3（设备说谎该不该进随机抽样）、K6（放行读法的代价、C381
  改法定案后这一格要改成什么）的具体判词，不判两次抽样之间方向是否一致——那是主 agent 的事。
- 未跑云端攻方腿（Opus，K1/K2）、云端正推腿（Sonnet，K4/K5）或核实员，不在这条腿的任务范围内。
- 未碰 K1、K2、K4、K5 四格，按分工表不许碰。
- 未读这一轮别的腿的提示与产出（禁读清单：`research/prompts/m2-supp3-item4-code-r1-opus-output.md`、
  `-sonnet-output.md`）、主 agent 这一轮的核实、同时在飞的别的轮次。
