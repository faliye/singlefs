# d137-gov-r3：一百三十七节那一轮的治理文件改动——第三轮（最终轮）判决（2026-10-31）

## 一、这一轮的材料与证据

- 正文：`research/prompts/_d137-gov-r3-body.md`；开工快照：`research/prompts/d137-gov-r3-snapshot/files-sha256.txt`（17 份）。
- 辩方腿（Opus，格 R1）：`research/prompts/d137-gov-r3-opus-defense-output.md`（187 行，sha256 b1d06537…），模型 `research/prompts/d137-gov-r3-opus-defense-model/`。
- 攻方腿（Opus，格 R2）：`research/prompts/d137-gov-r3-opus-output.md`（294 行，sha256 a650893a…），模型 `research/prompts/d137-gov-r3-opus-model/`。
- 倒推开工树与核引文（原样）：

```
python3 scripts/agents/snapshot-tree.py d137-gov-r3 --out <草稿目录>/d137-gov-r3-tree
python3 scripts/agents/cite-check.py research/prompts/d137-gov-r3-opus-output.md research/prompts/d137-gov-r3-opus-defense-output.md --root <草稿目录>/d137-gov-r3-tree --background research/prompts/_d137-gov-r3-body.md
```

- 倒推结果：清单 17 份，倒推出 15 份、sha256 对上 15 份；倒推不出 2 份：`scripts/mutation/crates-mutation-rows.py`、`.claude/gate.d/break-switch-replay-files.tsv`。这两份在开工快照之后被别的会话改过（HEAD 没动，`git log` 最近一次改它们是 2026-10-26 的 f4605214d），diff 里没有它们的基底。腿引到它们的行逐处人工核：攻方腿引复跑表的条数与「逐个跑过」那几句，主 agent 照此刻文件与件 16 报告核过，这一轮主 agent 自己改的三行（lib_heavy_tests 74 条、heavy-test-guard 79 条、bash-command-detector 118 条）此刻还在；`crates-mutation-rows.py` 两条腿都没引行。
- 引文核对：核了 35 处引文、51 处裸行号，对上 20 处，0 处对不上；没判 66 处里裸行号没有引文可比 4 处。没判的那一处引 `research/prompts/d137-gov-r2-main-verification.md:34`（不在开工树里），主 agent 现核原文：「| `printf "echo hi\n" > x.sh; cd "$PWD" && bash ./x.sh` | 2 | 放（过度修补 F） |」，与辩方腿引的相同。
- 腿的原样输出主 agent 逐份读过：攻方腿 `out/lsw-coverage.txt`（`not_listed=14`）、`out/w2-parallel-close.txt`、`out/c1-boundary.txt`、`out/feed-dispatch-forms.txt`（样本 2 范围声明闸 rc=0、样本 3 对照 rc=2、样本 7 `--plan` rc=2）、`out/tool-timeout-probe-file.txt`（「late: survived the tool timeout」）与 `out/tool-timeout-probe-tool-reply.txt`（「moved to the background」）；辩方腿 `out/feed-1.txt` 第 23 行、`out/feed-2.txt` 第 38 行的拒绝原文。
- 主 agent 现读代码坐实：`scripts/hosts/core_usage.py` 收段那一句不带写者时取 `open_segments[-1:]`（最近开的那一段，不认写者）；`scripts/hosts/host-dispatch.py` 第 51 行「④ 挑一台：本机在前、第二台在后，挑第一台「预测量 + 余量放得下」的」；`.claude/hooks/run-scope-guard.sh` 里 `host-dispatch` 出现 0 次；`.claude/hooks/heavy-test-guard.sh` 第 1443–1445 行文件头 ③ 把 echo / printf 写出再起列为认不出（出处⑱ Z4）。
- 核不动的：两条腿模型的复跑（`tools/rerun.sh`）没由主 agent 在副本里重跑——用户通知新的脚本任务先不起、等统一任务启动器；两条腿各自报整份复跑一次 rc=0、逐项相同。最终钩子上的四份复跑与语料再喂（第二轮 M5）同理没跑，判决里那一格记「够不着」。

## 二、逐条判决

### R1　论证前两轮的结论与判定过程（辩方腿）

| 条 | 腿的判 | 主 agent 核 | 判 |
|---|---|---|---|
| 前两轮 44 行 | 站住 32（9 行只照修补件读数）、带保留 10、够不着 1、不复核 1 | 抽核带保留里的 F、H3、A2 三条与够不着的 M5 | 采 |
| 第二轮过度修补 F | 带保留：判决第 34 行那条命令此刻照拒，理由换成「printf 写出的写法认不出」 | 那条命令把两件事放在一起：`cd` 之后起相对脚本（F 本身，已改）与 printf 写出再起（⑱ Z4，用户 2026-10-29 定「认不出的一律拒（偏严）」）；拒的理由已从 F 换成 Z4 | F 修补站住；第二轮判决第 34 行的样本把 F 与 Z4 混在一条命令里，记成样本不准 |
| H3、A2 | 带保留：第一轮喂法真 bash 不起 cargo / 先被另一道规矩拒 | 与第二轮判决 R1 H3 那一行同一件事 | 采保留 |
| M5 | 够不着：最终钩子上没有复跑读数 | 照实：复跑跑在件 13、15、16 之前的钩子上 | 采；最终钩子上的复跑等统一任务启动器 |
| 判定过程 | 第一轮「复现」只量钩子退出码、没在真 bash 里核；撑在没量过范围上的结论 7 条（「没有回退」只在旧钩子上量过；A 类 207 条里 97 条目录是同一条命令里才建的，不全是回放假象） | 97 条那一条与下一行 H5 过度同源 | 采 |
| H5 修补过度 | cd 到盘上不在的目录之后（含同一条命令里 `mkdir -p D; cd D`），同一条命令里 heredoc 写出、在同一目录起的脚本照拒；起的东西其实定得下来 | `out/feed-1.txt` 第 23 行拒绝原文核过；语料里同形约 26 条（正则粗分，推的）加 A 类 97 条同源 | 打中，采（零轮修补） |
| 文件头 ③ echo / printf | 与第二轮判决第 34、57 行冲突 | 见 F 那一行：冲突来自第二轮样本混了两件；Z4 是用户定的偏严 | 不采；「echo 不带选项、参数全是单引号字面串」那一窄条放不放写进交用户表（零轮） |
| 什么都不起的三种写错命令 | 只列不主张改 | — | 记录 |

### R2　已定撑不撑得住、未定清单、报出的数、零轮五件（攻方腿）

| 条 | 腿的判 | 主 agent 核 | 判 |
|---|---|---|---|
| W2 并行行错收 | 打中：`declare-parallel` 与 verus 写的「此刻并行」不带写者，收的是最近开的那一段；对照组带写者第 2400 秒叫醒，不带的顶到 7200 秒 | 代码现读坐实（`open_segments[-1:]`）；模型输出核过 | 打中，采（并行行带写者，零轮修补） |
| C1 跨 7200 那一窗口 | 边角：同一行既写「声明解释得了」又写「拖过了它算数的 7200 秒」，照定义归「只记」，晚一窗口叫醒 | `out/c1-boundary.txt` 第 2 行 `both_phrases=True` 核过；用户定「超过 7200叫醒 检查」 | 打中，采：两串都在时归「拖过 7200 秒」那一种（`.claude/main-agent.md` 次序与看门狗判法一起改，零轮） |
| M6 verus 预计秒数 | 边角：不含排队时间，排队不限时之后不是上界，后果是多叫醒、不漏（推的） | — | 采为线索，不改 |
| M3 lsw 十四条开关 | 打中：删掉 `lib_shell_words.py` 那一行之后，它文件头 76 条开关里 14 条不在任何登记脚本的文件头表里，提交前门禁不复跑（含 H1、H2、H3、H4、H6、H8 的开关） | `out/lsw-coverage.txt` 核过 | 打中，采：把 14 条点名进导入它的那几份自证的文件头开关表（零轮修补） |
| 包装不在复跑表 | 打中（漏登） | 复跑表现查没有 `run-with-memory-cap.sh` 一行；它登记 light | 打中，采：登进复跑表，四条旧开关条数一并复跑 |
| 数 | 79 / 74 / 118 / 22 条数对得上；118 条「逐个跑过」撑不住（111 条跑在件 15、16 改共用切词之前）；2502→2577、161→163 是版本之差；第二轮腿乙 30 条实为 38 条；30,595 / 326 对得上；592 秒数对、成因一半（最后 231 秒是它自己当排头放不下） | 复跑表那一行与正文、调度记录照实改 | 采 |
| 前台超时转后台（零轮：共用约束第 62 行与第 75 行） | 打中：前台超时不杀命令、转成后台，子 agent 照第 62 行在队里等，一超时就成了第 75 行禁止的后台任务 | 探针原样输出核过 | 打中；怎么并交用户定（与统一任务启动器那件同一层） |
| 均衡句先本机（零轮：主 agent 定义） | 打中：`host-dispatch.py` 本机在前、第一台放得下的；与用户「均分」不一样；照抄命令进 `--plan` 时键对不上峰值表、预测落到默认 8G（推的） | 文件头第 51 行现读坐实 | 打中；挑机器的规则交用户定（与统一任务启动器那件同一层） |
| 范围声明闸不剥派发器（零轮） | 打中：经 `host-dispatch.py` 起、不带范围声明照放，本机经包装就拒 | `run-scope-guard.sh` 里 0 处；样本 2、3 核过 | 打中，采（范围声明闸剥开两种起法，零轮修补）；不是构造绕过写法，是件 13 只改了重型闸、漏了范围声明闸 |
| `--plan` 不带前缀被拒（零轮） | 边角：只查不跑的 `--plan` 被重型闸拒 | 样本 7 rc=2 核过 | 打中，采（重型闸认 `--plan` 只查不跑，零轮修补） |
| 派发提示与第 63 行（零轮） | 轻：正文与派发提示写「自证经 run-with-memory-cap.sh」与第 63 行不一致 | — | 采；之后的派发照第 63 行写 |
| 闸剥开两种起法的选项表、E4 同一行别名、排队判法 | 没打中 | 一次没打中不算「没打中」：照 `.claude/rules/three-way-inference.md` 记一次 | 记录，不据此下结论 |
| H12、D1、D2、U1、A1 | 没打中（只读代码） | 同上 | 记录 |

## 三、怎么办

这是第三轮，也是最终轮，不开第四轮。三轮的结论：

- 前两轮站住的形态（钩子判法、派发闸、看门狗声明判法、串行声明、开关复跑登记）这一轮撑住，除 W2 与 C1 边角两处；修补过度的只剩 H5 那一处。
- 下面的修补都是这一轮里新冒出来的零轮形态，照规则不再为它们开一轮，修完写进交用户表、标「零轮」：

| 修补 | 改哪里 | 状态 |
|---|---|---|
| W2：并行行带写者 | `scripts/hosts/core_usage.py`（`declare-parallel` 写写者、收段只收自己的）与写「此刻并行」的调用方（`scripts/gate/verus-proof-check.py` 等） | 零轮 |
| C1：两串都在时归「拖过 7200 秒」 | `.claude/main-agent.md` 三种处置的次序与 `scripts/agents/agent-watch.py` 判法 | 零轮 |
| M3：lsw 十四条开关点名进导入它的自证文件头 | `.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/bash-command-detector.sh` 的文件头开关表 | 零轮 |
| 包装登进复跑表 | `.claude/gate.d/break-switch-replay-files.tsv` 加 `run-with-memory-cap.sh` 一行；agent-watch 那一行条数改成门禁读法现数；bash-command-detector 那一行写明 111 条跑在共用切词改动之前 | 零轮 |
| H5 过度：cd 之后同一目录写出再起 | `.claude/hooks/lib_shell_words.py` / `heavy-test-guard.sh` 的 cd 失败判法 | 零轮 |
| 范围声明闸剥开两种起法 | `.claude/hooks/run-scope-guard.sh` | 零轮 |
| 重型闸认 `--plan` 只查不跑 | `.claude/hooks/lib_heavy_tests.py` | 零轮 |
| 最终钩子上的四份复跑与语料再喂（第二轮 M5） | 主 agent 经统一任务启动器起 | 等启动器 |

交用户的：

| 问 | 依据 | 零轮 |
|---|---|---|
| 共用约束第 62 行（排不上在队里等）与第 75 行（前台不超过 240 秒、不起后台）怎么并：前台超时会转后台 | 攻方腿 `out/tool-timeout-probe-*` | 是 |
| 两台按资源「均分」的挑法：`host-dispatch.py` 此刻是先本机 | `scripts/hosts/host-dispatch.py:51` | 是 |
| echo 不带选项、参数全是单引号字面串写出再起的那一窄条放不放（⑱ Z4 用户定的偏严） | 辩方腿第三节第 2 处 | 是 |

## 回看决策

不涉及决策：被判的是项目的协作钩子、定义、规则与研究脚本，不碰 `.claude/kb/decisions/` 下任何一条。

## 这一轮被判过的文件

- `.claude/hooks/heavy-test-guard.sh`：R1 撑住（第二轮 F 修补站住）；R2 `--plan` 被拒（零轮修补）
- `.claude/hooks/lib_heavy_tests.py`：R2 `--plan` 被拒（零轮修补）；剥开两种起法没打中
- `.claude/hooks/bash-command-detector.sh`：R2 复跑范围那一句撑不住（复跑表改写）
- `.claude/hooks/lib_shell_words.py`：R1 H5 修补过度（零轮修补）；R2 M3 十四条开关无人复跑（零轮修补）；E4 没打中
- `.claude/hooks/runner-dispatch-guard.sh`：R2 H12、D1、D2 没打中
- `.claude/agent-common.md`：R2 第 62 行与第 75 行矛盾（交用户）；第 63 行与范围声明闸（零轮修补）
- `.claude/main-agent.md`：R2 C1 边角（零轮修补）；均衡句先本机（交用户）
- `scripts/agents/agent-watch.py`：R2 C1 边角（零轮修补）；U1、A1 没打中
- `scripts/hosts/core_usage.py`：R2 W2 打中（零轮修补）
- `scripts/hosts/detached-run.sh`：两条腿都没打中
- `scripts/hosts/run-with-memory-cap.sh`：R2 不在复跑表（零轮修补）；排队判法没打中
- `scripts/mutation/crates-mutation-rows.py`：两条腿都没打中（开工快照之后被别的会话改过，倒推不出）
- `scripts/agents/dispatch-prompt.py`：两条腿都没打中
- `scripts/gate/verus-proof-check.py`：R2 M6 边角（采为线索）；W2 写「此刻并行」的调用方之一
- `.claude/gate.d/checker-tier-formal-verification.sh`：两条腿都没打中
- `.claude/gate.d/break-switch-replay-files.tsv`：R2 M3、包装漏登、条数（零轮修补）；开工快照之后被别的会话改过，倒推不出
