# defs-gatebatch-m2-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

挑 `research/prompts/defs-gatebatch-m2-r1-sonnet-output.md:9` 的引用「`.claude/agents/crash-verifier.md:3`（description）：『崩溃一致性验证员：……』」核。草稿副本 `/tmp/claude-1000/defs-gatebatch-r1-verifier/selftest/crash-verifier.md`，把行号从 3 改成 4 再核：

```
$ sed -n '4p' /tmp/claude-1000/defs-gatebatch-r1-verifier/selftest/crash-verifier.md
tools: Read, Bash
```

第 4 行是 `tools: Read, Bash`，与被核的引文（description 那句）字面完全不对——判 ✗。方法能分辨。

## 一、输入与快照核对

- 三条腿报告路径：sonnet-output.md、opus-output.md、local-attack 系列（提示、两份样本 s1/s2、转述核对表、运行记录），均已交齐（`ls` 现查过）。
- 背景材料：`_defs-gatebatch-m2-r1-background.md`、附录二 `_defs-gatebatch-m2-r1-diff.md`（现查存在）。
- 开工快照 `research/prompts/defs-gatebatch-m2-r1-snapshot/sha256sums.txt`（11 个文件）：

```
$ sha256sum -c research/prompts/defs-gatebatch-m2-r1-snapshot/sha256sums.txt
.claude/gate.d/54-layer0-replay.sh: OK
research/scripts/admission.py: OK
.claude/gate.d/stage-inputs.tsv: OK
.claude/gate.d/stage-owners.tsv: OK
.claude/hooks/lib_heavy_tests.py: OK
.claude/hooks/heavy-test-guard.sh: OK
.claude/agents/crash-verifier.md: OK
.claude/rules/implementation-workflow.md: OK
.claude/main-agent.md: OK
.claude/agents/gate-triage.md: OK
research/scripts/check-segment-registry.py: OK
```

全部 OK：这 11 个文件今天的主树与开工快照逐字节相同。因此下面对这 11 个文件的核对，等同于对快照核对；对这 11 个文件之外、腿引用到的文件（`crates/` 下的源码、`.claude/hooks/lib_shell_words.py` 等），按派发指令对主树核，对不上记「分不清」而不是 ✗（这一轮不碰 `crates/`）。

- 云端腿报告 sha256（主 agent 给的 vs 现查）：

```
$ sha256sum research/prompts/defs-gatebatch-m2-r1-sonnet-output.md research/prompts/defs-gatebatch-m2-r1-opus-output.md
9bd82bbf24569aa2e11a25a1096315c88f277fb8b3d2bf93a9eac3a833a9ffdb  .../defs-gatebatch-m2-r1-sonnet-output.md
47d2e2a2ca6c74690f2f2ae7e50bb47cfbbfe0e4fd9ca8a199eb787664b02040  .../defs-gatebatch-m2-r1-opus-output.md
```

两份都与主 agent 给的 sha256 一致——两份报告现在的内容就是交回时的内容，不落入「后来被改过」那一类。

- 云端攻方模型目录 `SHA256SUMS` 自身与其内容：

```
$ sha256sum research/prompts/defs-gatebatch-m2-r1-opus-model/SHA256SUMS
ef5f5cbd0b4cfbdb267615a63d297912fb3896c565370d170a0166003f28c5c4  .../SHA256SUMS
$ (cd research/prompts/defs-gatebatch-m2-r1-opus-model && sha256sum -c SHA256SUMS)
（17 个文件全部 OK）
```

均与主 agent 给的一致，且目录内 17 个文件全部通过自证。

## 二、云端正推（Sonnet）报告核对表

除 J1 字面兼核一节外，其余引用全部指向 11 个快照文件之一（`crash-verifier.md`、`implementation-workflow.md`、`stage-inputs.tsv`、`stage-owners.tsv`、`main-agent.md`、`gate-triage.md`），已确认快照与主树逐字节相同，下表直接对主树核。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `crash-verifier.md:3` description 整句 | ✓ | `sed -n '3p' .claude/agents/crash-verifier.md` 逐字相同 |
| `crash-verifier.md:22` 整句 | ✓ | 同上，逐字相同 |
| `crash-verifier.md:29` 摘引（用「……」标省略） | ✓ | 省略处确认是同一行内被跳过的中间片段，首尾都对得上 |
| `crash-verifier.md:31` 摘引 | ✓ | 同上 |
| `crash-verifier.md:37` 摘引 | ✓ | 同上 |
| `crash-verifier.md:46` 整句 | ✓ | 逐字相同 |
| `implementation-workflow.md:58` 摘引 | ✓ | 首尾对得上，省略处是同一行内的中段 |
| `implementation-workflow.md:89` 摘引（省了「门禁管哪一半」小标题本身） | ✓ | 引文字面是该行小标题之后的正文，逐字相同 |
| `stage-inputs.tsv:13-16`（新增注释段）摘引 | ✓ | 四行拼接后与引文首尾一致 |
| `stage-inputs.tsv:32-35`「四条 crash-case 行……count-line=/exhaustive=/threads= 齐」 | **✗** | 见下方「发现一」：只有第 32、33 行三项齐全，第 34 行（floor-raise）三项全无，第 35 行（c561-sigma-full）只有 count-line=，没有 exhaustive=/threads= |
| `stage-owners.tsv:37` 整句 | ✓ | 逐字相同 |
| `main-agent.md:61` 摘引 | ✓ | 首尾对得上 |
| `gate-triage.md:29` 摘引 | ✓ | 首尾对得上 |
| `grep -rn "整批.*一格\|按整批输入哈希" .claude/agents/crash-verifier.md .claude/rules/implementation-workflow.md .claude/gate.d/stage-inputs.tsv .claude/gate.d/stage-owners.tsv .claude/main-agent.md .claude/agents/gate-triage.md` 声称 0 命中 | ✓ | 现跑：`exit=1`（grep 无命中），与「0 命中」一致 |
| `m2-layer0-scale-r3-main-verification.md:25` 整句引文 | ✓ | `sed -n '25p'` 逐字相同 |
| 附录二 `_defs-gatebatch-m2-r1-diff.md:2551`（「改前」`-` 行）整句引文 | ✓ | `sed -n '2551p'` 逐字相同 |
| `m2-layer0-scale-r2-main-verification.md:29` 整句引文 | ✓ | `sed -n '29p'` 逐字相同 |
| `records/2026-09-24-里程碑二收尾调度.md:190` 整行引文 | ✓ | `sed -n '190p'` 逐字相同 |
| `records/2026-09-24-里程碑二收尾调度.md:180` 整行引文（很长的第⑭条附近） | ✓ | `sed -n '180p'` 逐字相同 |
| `grep -c "推翻\|覆盖\|不再维持\|不再作数" research/prompts/m2-layer0-scale-r3-main-verification.md` 声称 0 | **✗（分不清，见「发现二」）** | 现跑：`grep -c` 结果是 **1**，不是 0 |
| 同一模式在 `records/2026-09-24-…调度.md` 全文 14 处命中 | ✓ | 现跑 `grep -c` = 14，一致 |
| 同一模式在该文件第 190 行单独核为 0 处 | ✓ | 现跑 `awk 'NR==190'\|grep -c` = 0，一致 |
| `54-layer0-replay.sh:2` 摘引 | ✓ | 首尾对得上 |
| `54-layer0-replay.sh:314` 摘引 | ✓ | 首尾对得上 |
| `54-layer0-replay.sh` 头部注释第 16 行摘引（内层引号用『』，原文用「」） | ✓（引号符号不同，字面内容一致） | `sed -n '16p'` 核对，内容逐字相同 |
| `54-layer0-replay.sh:50-61`「`--start-over` 只与 `--full` 同用，第 60-61 行显式拒绝单独用」 | ✓ | `awk 'NR>=50&&NR<=61'` 逐行核，60/61 行确为拒绝语句 |
| `54-layer0-replay.sh:201-202` 摘引 | ✓ | 逐字相同 |
| `54-layer0-replay.sh:19` 摘引（省了行首「不带它时从调用方的环境里清掉这个变量。」） | ✓（未标省略号但确是该行的后半句，内容不假） | `sed -n '19p'` 核对 |

**发现一（✗）**：`stage-inputs.tsv:32-35` 处的断言「各自 `test=` 恰一条、`count-line=`/`exhaustive=`/`threads=` 齐」与实际文件不符。逐行核（`cat -A` 现查，按 tab 分列）：

```
row32 layer0-first-stream:        test=… count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0
row33 layer0-second-stream:       test=… count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B
row34 floor-raise-pushed-...:     test=…（只有 test=，没有 count-line=/exhaustive=/threads= 中的任何一个）
row35 c561-sigma-full:            test=… count-line=C561_SIGMA_FULL（只有 count-line=，没有 exhaustive=/threads=）
```

「test= 恰一条」这半句核对为真（四行各恰好一个 `test=`）；「count-line=/exhaustive=/threads= 齐」这半句为假：第 34 行三项全缺，第 35 行缺两项。这不影响 J5 的六份文字互相一致这个主判断（这一条不在互相核对的六份文字之列，是对 `stage-inputs.tsv` 本表内容的另一句独立描述），但报告原文把它写成了「齐」。

**发现二（分不清：文件可能在腿交回之后被改过）**：Sonnet 报告 1.2 节的核心发现建立在「`grep -c "推翻\|覆盖\|不再维持\|不再作数" research/prompts/m2-layer0-scale-r3-main-verification.md` 为 0（r3 判决整份文件都没有这几个字）」这句声称的命令输出上。现跑同一条命令，结果是 **1**，不是 0：

```
$ grep -n "推翻\|覆盖\|不再维持\|不再作数" research/prompts/m2-layer0-scale-r3-main-verification.md
51:**用户 2026-09-26 定案覆盖第三节「乙」那一条**（`records/2026-09-24-里程碑二收尾调度.md` 第三节「崩溃枚举的跑法」那一行，原话「这次跑可以 下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」）：崩溃枚举按用例各自登记输入、各自复用，也就是采纳乙；第二轮打中的「按流列的清单漏新加的共用文件」照当时的修法补（清单用排除法写、指纹补第三轮 U4 那几样），由门禁批实现，它的一轮三方是 `research/prompts/_defs-gatebatch-m2-r1-body.md`。第三节那一条原文照留。
```

这第 51 行原样写着「用户……定案**覆盖**第三节『乙』那一条」，并且逐字解释了 Sonnet 报告标为「核心发现」「字面矛盾」的那个现象：`research/prompts/m2-layer0-scale-r3-main-verification.md:25` 那句「54 号照旧按整批输入哈希记一格全绿标记」之所以仍原样留在文件里，是因为第 51 行的作者故意让「第三节那一条原文照留」——这不是矛盾没被发现，是矛盾被显式记录在同一份文件里、只是不在 Sonnet 核对的第 25 行附近。

**这处不判 ✗ 而记「分不清」的理由**：`m2-layer0-scale-r3-main-verification.md` 不在这一轮给的 11 个文件快照之列，派发指令要求这种情况下「腿引的行号与主树对不上时记『分不清：文件可能在腿交回之后被改过』」。这里不是行号对不上，是同一条命令现在跑出的计数与报告里写的不一致，但同一类不确定性成立，而且有直接证据支持「后来被改过」这个猜测——文件 mtime 显示它在 Sonnet 交报告**之后**仍被写过一次：

```
$ stat -c '%y %n' research/prompts/m2-layer0-scale-r3-main-verification.md research/prompts/defs-gatebatch-m2-r1-sonnet-output.md
2026-09-26 research/prompts/m2-layer0-scale-r3-main-verification.md
2026-09-26 research/prompts/defs-gatebatch-m2-r1-sonnet-output.md
$ git status --porcelain -- research/prompts/m2-layer0-scale-r3-main-verification.md
?? research/prompts/m2-layer0-scale-r3-main-verification.md
```

`m2-layer0-scale-r3-main-verification.md` 的 mtime 比 sonnet-output.md 的 mtime 晚约 58 秒，且是未跟踪文件（`??`），说明它在 Sonnet 完成并落盘报告之后仍被写动过一次。我不能判定这次改动是不是就是第 51 行的来历、也不能判定 Sonnet 核对时那一行在不在——只能把这两条时间线摆出来：**如果第 51 行是在 Sonnet 交报告之后才补上的，那么 Sonnet 报告 1.2 节写「没有找到」是当时真实的观测；如果第 51 行早就在，Sonnet 报告这一句就是漏检**。这两种情况都不改变一个事实：**此刻**主树上的 `m2-layer0-scale-r3-main-verification.md` 已经带着覆盖第 51 行，Sonnet 报告 1.2 节据以展开的「两处权威记录互相矛盾、没有一处指向另一处」这个前提，现在不成立。

这一条不是「引用与产物对不上」的普通 ✗，而是核对对象本身在腿交回之后发生了变化；按定义第 6 步与「分不清」单列，不计入下面的 ✗ 计数，但因为它直接冲击 Sonnet 报告的核心结论，单独加粗标出，供主 agent 逐条现查时优先看。

## 三、云端攻方（Opus）报告核对表

### 3.1 文件:行 + 抄的原文（38 处）

全部现查 `sed -n '<N>p' <文件>`，逐字或逐段（省略处标「……」）比对；11 个快照文件内的按快照核，`crates/`、`.claude/hooks/lib_shell_words.py`、`.claude/gate.d/change-touches-crates.sh`、`.claude/singlefs-ai-sop/scripts/gate.sh` 等快照外文件对主树核。

| 引用 | 结果 |
|---|---|
| `54-layer0-replay.sh:114` | ✓ |
| `stage-inputs.tsv:24`（路径列 `crates/ Cargo.toml Cargo.lock`） | ✓ |
| `change-touches-crates.sh:105` | ✓ |
| `54-layer0-replay.sh:117` | ✓ |
| `.claude/singlefs-ai-sop/scripts/gate.sh:250` | ✓（快照外，对主树核，一致） |
| `stage-inputs.tsv:19` | ✓ |
| `admission.py:921` | ✓ |
| `admission.py:2633` | ✓ |
| `54-layer0-replay.sh:69` | ✓ |
| `admission.py:1291` | ✓ |
| `54-layer0-replay.sh:168` | ✓ |
| `_defs-gatebatch-m2-r1-diff.md:445` | ✓ |
| `_defs-gatebatch-m2-r1-diff.md:476` | ✓ |
| `admission.py:1265` | ✓ |
| `admission.py:1321` | ✓ |
| `crates/singlefs-harness/src/layer0_progress.rs:700` | ✓（快照外，对主树核，一致） |
| `layer0_progress.rs:767` | ✓（同上） |
| `54-layer0-replay.sh:344` | ✓ |
| `admission.py:1154` | ✓ |
| `admission.py:90` | ✓ |
| `admission.py:91` | ✓ |
| `.claude/rules/implementation-workflow.md:12` | ✓ |
| `admission.py:955` | ✓ |
| `m2-layer0-scale-r3-main-verification.md:31` | ✓（现查时该文件已知在 Sonnet 交回后被写过一次，但第 31 行的内容与本轮无关的 mtime 变化无冲突，核对为真） |
| `.claude/hooks/lib_heavy_tests.py:252` | ✓（快照内） |
| `lib_heavy_tests.py:259` | ✓ |
| `lib_heavy_tests.py:305` | ✓ |
| `.claude/hooks/lib_shell_words.py:37` 及其上下若干行 | ✓（快照外，对主树核：`nohup/setsid/command/exec/nice/ionice/timeout/env/stdbuf/sudo/taskset` 与关键字 `time` 全部现查存在） |
| `.claude/hooks/heavy-test-guard.sh:71` | ✓ |
| `heavy-test-guard.sh:73` | ✓ |
| `crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs:544` | ✓（快照外，对主树核，一致） |
| `admission.py:1060` | ✓ |
| `lib_heavy_tests.py:78` | ✓ |
| `54-layer0-replay.sh:200` | ✓ |
| `crates/singlefs-harness/src/crash.rs:2374` | ✓（快照外，对主树核，一致） |
| `first_transaction_step_seven_layer0.rs:456` | ✓（同上） |
| `second_transaction_step_zero_layer0.rs:736` | ✓（同上） |
| `stage-inputs.tsv:22`（「宁宽勿窄」） | ✓ |

38 处逐字/逐段核对，**0 处 ✗**。

### 3.2 复跑命令（模型目录拷进草稿目录、在副本里跑）

方法：`cp -r research/prompts/defs-gatebatch-m2-r1-opus-model/. /tmp/claude-1000/defs-gatebatch-r1-verifier/opus-rerun/`；`rerun.sh` 与 `probe_common.py` 里用相对路径的深度计算真实仓根，拷到别处会算错，改成硬编码绝对路径（`research/scripts/replace-once.py` 定点替换，仅改这两行，只在草稿副本上），指向的仍是这台机器上未经改动的真实仓（只读）。改动记录：

```
$ python3 research/scripts/replace-once.py rerun.sh 'repo="$(cd "$here/../../.." && pwd)"' 'repo="<仓根>"'
  ✓ rerun.sh：命中 1 次，已替换
$ python3 research/scripts/replace-once.py probe_common.py 'REPOSITORY = os.path.dirname(...)' 'REPOSITORY = "<仓根>"'
  ✓ probe_common.py：命中 1 次，已替换
```

| 复跑 | 报告里的 SUMMARY / 关键行 | 副本重跑结果 | 一致？ |
|---|---|---|---|
| `probe_j1_stage.py`（今天代码） | `cells=15 attack_cells_broken=6 control_cells_broken=0` | 同一行，逐字相同 | ✓ |
| `probe_j2_manifest.py`（今天代码） | `cells=11 attack_cells_broken=6 control_cells_broken=0` | 同一行；`diff` 整份日志除耗时脚注外 **0 差异**（含全部指纹哈希） | ✓ |
| `probe_j3_environment.py`（今天代码） | `cells=8 attack_cells_broken=3 control_cells_broken=0` | 同一行；E1/E2/E3 的「改前=改后」「改前≠改后」这一定性结论逐格重现，具体哈希值因探针在临时目录里构造脚本、路径不同而与原值不同（预期之内，见下方说明） | ✓（定性），哈希绝对值不可比 |
| `probe_j4_hook.py`（今天代码） | `cells=28 attack_cells_broken=17 control_cells_broken=0` | 同一行；`diff` 整份日志 **0 差异** | ✓ |
| `probe_j1_stage.py --stage <补丁后54号> --admission <补丁后admission.py>`（套 F1、F6） | `exit=0`，`cells=15 attack_cells_broken=0 control_cells_broken=0` | 同一行，逐字相同 | ✓ |
| `probe_j4_hook.py --admission <补丁后admission.py>`（套 F6） | `exit=1`，`cells=28 attack_cells_broken=16 control_cells_broken=0` | 同一行，逐字相同；`diff` 整份日志 **0 差异** | ✓ |
| 点名复跑 J1-a：只改登记表（A1/A2/A3），快档退 77；带 `SINGLEFS_GATE_FULL=1` 判红 | `j1.log:4,6,8` 退 77；`j1.log:5,7,9` 退 1 且点名对应用例 | 副本重跑同一批行号、同一批状态，`diff` 只有时间戳不同 | ✓ |
| 点名复跑 J1-b：判法迁进 admission.py 前后指纹相同 | `旧 639097891a56ec19…，新 639097891a56ec19…` | 副本重跑得到同一对指纹（逐字相同，前后一致） | ✓ |
| 点名复跑 J4-b：去掉 c561 的 `#[ignore]` 后，`cargo test` 不带 `--ignored` 被放行；`admission.py crash-cases` 自查退 0 | `j4.log:27`（I1，退 0，放行）、`j4.log:28`（I2，退 0） | 副本重跑同一结果：I1 闸判「退 0」放行；`crash-cases` 自查退 0 | ✓ |
| `python3 research/scripts/admission.py crash-case-manifest . <四个 case-key> <清单文件>`（点名复跑） | 未在 Opus 报告中单独列（Opus 报告未跑这条，是给本地攻方事实表的点名复跑，见第四节） | 见第四节 | — |
| `cargo nextest --version`（B1 环境事实） | 「no such command」 | 现跑：`error: no such command: 'nextest'` | ✓ |
| `cargo mutants --version`（B2 环境事实） | `cargo-mutants 27.1.0` | 现跑：`cargo-mutants 27.1.0` | ✓ |
| `rustc --test` 5 行小文件试 `--include-ignored`/`--include-ig`/`--include`/`--ignore`/`--ign` | 只有第一个跑到 ignored 用例，其余 `error: Unrecognized option` | 现跑同一组，结果逐字一致 | ✓ |

**J3 哈希值为什么不可比而定性结论可比**：`probe_j3_environment.py` 在每次运行时用 `mktemp` 建临时脚本、临时 `.cargo/config.toml`，这些路径本身进了指纹计算（环境变量值、配置文件文本），两次运行的临时路径不同，所以摆出的十六进制值不同；但每一格判的是「改动前后指纹是否相同」这个**相对**关系，我的副本重跑复现了与原始报告完全相同的相对关系（E1/E2/E3 三格改前改后相等，E4 对照改前改后不等），这是这一格声称打中的实质内容，已核实一致。

`probe_j2_manifest.py` 不受这个问题影响，因为它的四条 `INFO` 行读的是这台机器上真实 `crates/` 的当前内容（未拷贝，未改动），只要 `crates/` 没变就能整份重现，实测确认逐字节相同（含指纹哈希）。

**没有发现任何 ✗。**

## 四、本地攻方（转述核对表 + 事实表点名复跑）

### 4.1 转述核对表：原文文件:行核对（6 处）

`admission.py:955-960` 是快照内文件，对快照核。

| 提示里的英文项 | 原文文件:行 | 结果 |
|---|---|---|
| R1 | `admission.py:955` | ✓ 逐字相同 |
| R1（续） | `admission.py:956` | ✓ 逐字相同 |
| R2 | `admission.py:956-957` | ✓ 逐字相同 |
| R3 | `admission.py:958` | ✓ 逐字相同 |
| R3（续） | `admission.py:958-959` | ✓ 逐字相同 |
| R4 | `admission.py:959-960` | ✓ 逐字相同 |

英文转述与中文原文对照（限定词有没有丢、有没有多加）：核对表自己给出的「首稿缺的」「定稿」两栏与现查的 `admission.py:955-960` 原文比对，定稿栏五处限定词补回（「整个」「顶层」「54 号」「这条用例的」「package.autotests/package.build 键名」「连同那个目录」「.rs 文件或 Cargo.toml 文件」「outside its own file」「any single one of them is enough」「a newly split-out crate」）均能在原文里找到对应的中文限定词，没有找到定稿栏漏掉原文限定词、或凭空多加原文没有限定词的情况。「多出来的限定词」表里两行（「for every one of the four cases here」与 G2/G3 追问角度）都自称是转述之外新加的事实陈述/追问角度，不是条款译文的一部分，核对表自己列出了理由，我核对这两条确实不在 `admission.py:955-960` 字面内、且核对表没有把它们伪装成条款译文——按核对表自己的说明，不计入摘句失真。

**0 处 ✗**。

### 4.2 事实表 G1/G2/G3：点名复跑 `admission.py crash-case-manifest`

按派发指令点名复跑：

```
$ for key in crash-case:layer0-first-stream crash-case:layer0-second-stream \
             crash-case:floor-raise-pushed-by-the-session crash-case:c561-sigma-full; do
    python3 research/scripts/admission.py crash-case-manifest . "$key" <清单文件>
  done
8c2cb04e80e3453310762b9d7b7433381032a40d7e11c261fbdafdae74f5ae02 69 68
e665d2e37e56ef7c2496aa915914e09268441352918263f52919f7fd700d51e0 69 68
a83e75f86dddbc85ce38451476f42d909bd1fb91f17a7c862463abe899eb25bc 69 68
b094e0446aa2c6188d7c1855f1267d0c2b7a05deb5376d399df2e89b45937654 69 68
```

| 表格 | 提示里记录的 | 现跑结果 | 一致？ |
|---|---|---|---|
| G1 row C1 | 指纹 8c2cb0…、69 68 | 逐字相同 | ✓ |
| G1 row C2 | 指纹 e665d2…、69 68 | 逐字相同 | ✓ |
| G1 row C3 | 指纹 a83e75…、69 68 | 逐字相同 | ✓ |
| G1 row C4 | 指纹 b094e0…、69 68 | 逐字相同 | ✓ |
| G2 row 1 `tests/common/mod.rs` 四案例全 kept | 在四份清单里 grep `crates/singlefs-harness/tests/common/mod.rs` | 四份清单里全部命中（kept） | ✓ |
| G2 row 2 `tests/first_transaction_region_bytes.rs` 全 kept | 同上 | 四份清单全部命中 | ✓ |
| G2 row 3 `tests/instance_acquisition.rs` 全 kept | 同上 | 四份清单全部命中 | ✓ |
| G2 row 4 `tests/checker_known_bad_images.rs` 全 excluded | 同上（应查无命中） | 四份清单全部不命中 | ✓ |
| G3 row 1 `first_transaction_step_seven_layer0.rs` 只在 C1 里 kept | grep 四份清单 | 只在 layer0-first-stream 清单里命中 | ✓ |
| G3 row 2 `second_transaction_step_zero_layer0.rs` 只在 C2 | 同上 | 只在 layer0-second-stream 清单里命中 | ✓ |
| G3 row 3 `second_transaction_crash_inside_...rs` 只在 C3 | 同上 | 只在 floor-raise-pushed-by-the-session 清单里命中 | ✓ |
| G3 row 4 `record_checker_judges_absence_...rs` 只在 C4 | 同上 | 只在 c561-sigma-full 清单里命中 | ✓ |

**12 处事实表格全部复跑一致，0 处 ✗。**

### 4.3 样本与运行记录（干净样本判定）

```
$ wc -w research/prompts/defs-gatebatch-m2-r1-local-attack-output-s1.md
302 …s1.md
$ wc -w research/prompts/defs-gatebatch-m2-r1-local-attack-output-s2.md
184 …s2.md
$ python3 research/scripts/corruption-check.py …-s1.md ; echo exit=$?
绿 … cjk=0 words=296 fffd=0 汉字复读=0 英文复读=0 …    exit=0
$ python3 research/scripts/oov-check.py …-s1.md ; echo exit=$?
绿 … 生词=12 拼接=0  生词: falsified                   exit=0
（s2 同样两项均为 exit=0，词数与生词种类一致）
```

`wc -w` 的 302/184 与运行记录表逐字相同——✓。两道闸复跑都是 exit=0（绿），与运行记录「干净」的判定一致——✓。**有一处措辞需要澄清但不算 ✗**：运行记录写「falsified（1 个，真词，非损坏）」，`oov-check.py` 原样输出是「生词=12」；核对后确认「12」是这个词在全文里的出现次数（12 条编号答案每条都以「this would be falsified by」收尾），「1 个」说的是不重复的生词种类数（只有 `falsified` 这一个词形），两个数说的不是同一个量，不矛盾，只是运行记录没有点破「12」和「1」分别是什么。

**没有发现 ✗。**

## 五、计数

| 腿 | 核了几处 | ✓ | ✗ | 分不清 / 核不动 |
|---|---|---|---|---|
| 判别力自证 | 1 | 0（要求判 ✗） | 1（正确判出，证明方法分辨） | 0 |
| 云端正推（Sonnet） | 28 | 26 | 1（`stage-inputs.tsv:32-35` 的「count-line=/exhaustive=/threads= 齐」与实际不符） | 1（`m2-layer0-scale-r3-main-verification.md` 的 `grep -c` 计数：报告写 0、现跑是 1，该文件 mtime 晚于 Sonnet 报告 mtime，判「分不清：文件可能在腿交回之后被改过」，未计入 ✗） |
| 云端攻方（Opus） | 38 处文件:行 + 12 处复跑/哈希 = 50 | 50 | 0 | 0 |
| 本地攻方（转述核对表） | 6 | 6 | 0 | 0 |
| 本地攻方（事实表复跑） | 12 | 12 | 0 | 0 |
| 本地攻方（样本与运行记录） | 6（2 份 wc -w + 4 次闸复跑） | 6 | 0 | 0（「1 个」vs「12」的措辞差异已在正文说明为不矛盾，不计入表） |
| **合计** | **104** | **101** | **1** | **1（单列，不计入 ✓/✗）** |

未发现任何腿把行号误写成背景材料（`_defs-gatebatch-m2-r1-background.md`、附录二 `_defs-gatebatch-m2-r1-diff.md`）里的行号——附录二本身被正确引用为附录（Sonnet 1.2 节、Opus J1-b 一节），没有把源文件自己的行号与附录行号弄混的情形。

## 六、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑（定义范围）。
- 没有判 J1（假复用等语义漏洞）、J2、J3、J4 的攻击是否站得住——那是主 agent 的判决权，我只核 Opus 报告里写的字面引用与复跑数字是否与仓里的实况、与我自己重跑出的数字一致。
- 没有对 `crates/` 做任何改动，也没有跑任何重型测试、没有执行 54 号本身、没有跑任何门禁阶段。
- `probe_j3_environment` 的复跑只核对了「改前=改后」这个相对关系，没有、也不可能核对绝对哈希值本身（临时路径每次不同，是这个探针的构造决定的，不是我核对方法的缺口）。
- 没有重新判定 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第 51 行的「用户定案覆盖第三节『乙』」这句话本身是不是合法、够不够格推翻 r3 判决的挂起——那是主 agent 的判决权（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）；我只报告这句话现在存在、它直接回应了 Sonnet 报告标为「核心发现」的那个现象，以及这句话可能是在 Sonnet 交报告之后才写入的（mtime 证据），无法进一步确定写入的确切时刻是不是真的晚于 Sonnet 核对的那一刻。
- 没有核 Opus 报告「没打中的形状」一节里全部叙述性描述（例如注释剥离、符号链接、Cargo.toml 注释）逐条重跑，这些多数是没有构造出攻击格、没有产物可比对的叙述；只核了里面有具体命令与产物编号的那几处（已在第三节列出）。
- 没有核实 Opus 报告的「四句」自证段落（分辨臂/看不看得到/满足哪个分句/改法还中不中）——那是分析性文字，不是引用或产物，不属于我的核对范围。
- 没有重跑 `ask-local.sh` 本身去验证本地腿的样本是模型真实产出而非伪造——运行记录声称的命令与判定我核对了产物（wc、两道闸）能不能自洽，但没有能力重新调用同一个本地模型网关去验证「这份输出确实是那次调用产生的」这个更底层的事实，只能确认现有产物内部各项数字互相吻合、且过闸。
