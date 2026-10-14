# 工具与治理批的零轮形态：第一轮正文（2026-10-13）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 V7 V8 K1 K2 K3 K4 K5 K6 K7 K8 K9 K10 K11 W01 W06 W07 Q1 Q3 H4 -->

## 一、这一轮要判什么

上一题（`m3-defs-b3`）三轮三方跑完不开第四轮，第三轮判决 `research/prompts/m3-defs-b3-r3-main-verification.md`「交用户」表里被攻过零轮的形态，用户 2026-10-13 定「选错了 开三轮」：另立这一题，从头走三轮三方，攻完才合入。这一题攻的是这些零轮形态本身：

| 格 | 零轮形态 | 在哪 |
|---|---|---|
| V1 | K1、K2、K3：⑮ 括注里的中文数词与「分之」「合计」「减」「各」判红，只有以「填」起头的尖括号算骨架占位，类型表九种（加 clerk、three-way-forward、three-way-defense），⑭ 跟着；三种类型定义「输入」补线程上限与预计挂钟，骨架脚本跟上 | `.claude/hooks/runner-dispatch-guard.sh` ⑮ ⑭、`scripts/agents/dispatch-prompt.py`、`.claude/agents/clerk.md`、`three-way-forward.md`、`three-way-defense.md`、`investigator.md`、`tooling-writer.md`、`.claude/main-agent.md`、`.claude/agent-common.md` |
| V2 | K4：「让出 CPU」限定为用户或别的会话在消息、弹窗里明说；「在跑的不发消息」限定为不为改线程发消息 | `.claude/main-agent.md`、`.claude/agent-common.md` |
| V3 | K5、W01、W07 前两样：heavy-thread-defaults 读自己线程变量那一路——`#[cfg(test)]` 后接 `mod` 才跳过、回退看读它的那条语句、引号里的全大写名字只在读环境变量的写法里才算读、`["X"] = 值` 不算读、`;`/`&&`/`||` 之后同一行赋的算局部变量 | `.claude/gate.d/code-tooling.sh` heavy-thread-defaults 格 |
| V4 | K6（两轮的文字）与 `--test-threads 1`：单机分支带、双机分支与 admission 拼的 crash-case 命令、崩溃放量、双机驱动还没带；GPU 对拍 CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`、没设取核数 | `.claude/rules/verification.md`、`scripts/crash/crash-point-replay.sh`、`.claude/gate.d/checker-tier-layer0-replay.sh` |
| V5 | K7、K11（H4 乙）、W07 后两样：重型测试闸认 `env -C` / `--chdir`（切词在 `lib_shell_words.py`）、仓副本里 light 自证另比副本那份脚本与主仓那一份 sha256 | `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/lib_shell_words.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/agent-common.md` |
| V6 | K8、K9、W06：⑥ 出处那一行不许有别的轮名；⑥ 与门禁 abandoned-rounds 格共用 `lib_abandoned_rounds.py`（日期、同一轮重复、出处）；门禁格对 2026-10-12 之前的行不要求出处 | `.claude/hooks/lib_abandoned_rounds.py`、`.claude/hooks/runner-dispatch-guard.sh` ⑥、`.claude/gate.d/doc-process-records.sh`、`.claude/rules/three-way-inference.md` |
| V7 | K10：private-values 未跟踪文件整份读（超上限判红）、改动范围里的路径也比、红行里路径命中打成「〔文件名含私有值〕」 | `.claude/gate.d/doc-text.sh` |
| V8 | Q1、Q3：长活脚本文件头改登记值读法（`lib_long_run.py` 的 `judge_script_header`，`detached-run.sh` start / long-run-check、`host-dispatch.py` ⓪ 共用），豁免表 `.claude/long-run-live-hosts-exempt` 与门禁格 long-run-header-exempt；⑭ 与文件头共用现算词表 `available_parallelism|nproc|除以|在跑|核数|忙闲|空闲|负载` | `.claude/hooks/lib_long_run.py`、`scripts/hosts/detached-run.sh`、`scripts/hosts/host-dispatch.py`、`.claude/gate.d/code-tooling.sh` long-run-header-exempt 格 |

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/clerk.md`、`.claude/agents/investigator.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b6-r1/repo/`（集成副本标签 `after-b6-fixup` 那一版，没有 `.git`）；这些零轮形态动过的 27 份文件的 sha256 在 `research/prompts/m3-defs-b6-r1-snapshot/frozen-copy-sha256.txt`。零轮形态之前那一版（上一题第二轮冻结副本，`after-b4-fixup2`）在 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`，两份逐文件比就是这一题的对象。腿只读这两份冻结副本，不读主工作区。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一题不改；`crates/` 里自己定并行度默认值的写法登记在冻结副本 `.claude/heavy-thread-exempt` 的「代码批」行。
- 集成副本复核：批 5（`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify4.summary`，148 项）、批 5 收尾（`verify5-out/`）、批 6（`verify6.summary`，168 项判错 1 项：一个开关失去会红的样本）、批 6 收尾（`verify7-out/`：code-tooling 样本档绿、heavy-thread-defaults 15 个与 long-run-exempt 3 个开关全红、`dispatch-prompt.py --selftest` 绿、heavy-thread-defaults 与 long-run-header-exempt 两格在集成真树上绿）。
- 上一题三轮判决的另立账（第一、二、三轮判决各一节）不在这一题射程里，不再交，除非零轮形态让它变得更糟。

## 三、要答的

每格都问三件：这一形态有没有过度外推或越位（管了不该管的、判了判不了的）；它与已有条款、钩子、门禁之间有没有内部矛盾或循环依赖（照一处做违反另一处，一处收一处拒）；八格里哪一条最脆弱（换一个最自然的写法就绕过或误伤）。另各格的专问：

- V1：中文数词表与「说明性的词」边界（「两台」「各一」「一半」之外的日常写法会不会误伤）；九种类型里哪种派发其实不带重活却被逼写线程上限。
- V3：只认几种读环境变量写法之后，真树里有没有别的读法（自定义包装函数、配置文件、`argparse` 默认读环境）漏判。
- V5：sha256 比对的「主仓那一份」在主工作区合入之前与之后各指哪一份；副本里脚本 import 的别的文件改了会怎样。
- V6：2026-10-12 之前免出处的那几行，⑥ 与门禁格一收一拒还剩哪些形状。
- V7：整份读的内存与时长；路径比对的误报。
- V8：豁免表与 E171 下一次修订的衔接；词表与「台数取多机配置」那一句的交集（例：「核数」出现在「台数」说明里）。

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「定义与规则写的就是钩子、脚本、门禁做的」 | V1、V2、V4、V6、V8 | 每一句零轮新写或改写的定义文字，对着冻结副本里的钩子与脚本源码（文件:行）判「一致 / 不一致 / 文字没写到」，不一致的给出实际行为与能复现的喂法 |
| **云端攻方（Opus）** | 造工作流历史攻零轮形态 | V1、V3、V5、V6、V7、V8 | 每条历史写成可复现的步骤（派发提示原文、钩子喂法、命令序列），在冻结副本拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪；每条打中在上一题第二轮冻结副本上也喂一遍，写明零轮之前中不中 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | 两条腿在那一格都给不出照定义做却违反用户定案（默认 31、固定不看忙闲、只核自己的且豁免表不登记别人没审过的、私有信息不进仓、在跑的不动、仓副本自证收回放宽）或定义说拒而钩子不拒、钩子拒而定义没写的可达历史 |
| 一格判「站不住」⇒ 改法进下一批，改完开第二轮 | 给出那样一条可达历史 |
| 打中也打中零轮之前的那一版 ⇒ 另立一笔账 | 同一段历史在 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/` 上给出同样的错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b6-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b6-r1-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b6-r1-<腿名>-model/`（含 `tools/rerun.sh`）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」，主 agent 写判决时复跑。
