# test-script-align-r2 攻方腿（Opus）：U1、U2

日期：2026-10-01。被攻的格：U1（钩子收紧之后的放行面）、U2（入口与脚本的改后判法）。前几轮判决：`research/prompts/test-script-align-r1-main-verification.md`。不重复第一轮的 T2、T3 探针：这一轮找收紧之后新开的口子。

## 复跑

```
bash research/prompts/test-script-align-r2-opus-model/rerun.sh /tmp/claude-1000/<新草稿目录>
```

`SHA256SUMS` 原样：

```
17030c90740e53910247f341845f03acd56399ba18014f72340e0e58cb64b7c7  hook-probe.py
bfb8bcf6594cdedc695c5678e481e58d1fe516af416b167e32db45091052a466  probes.sh
becbfff157803fb053baece2660fe49f9c047327d11c88e9d23967cc4620a378  wild-probes.sh
259a3e09890c8d5c1840aedfe587a1a1321194eef28e4e3e78f34fd65f5145f0  rerun.sh
dc84544f1ee4f0e27fda51ee4c998a30402402d36f585485a118d3ff91607f4a  outputs/probes.out
300b786532bcae99b6da0041398bf9d775d25b53059811f054e1b3cf00d00152  outputs/wild-probes.out
```

钩子只喂 JSON（`tool_name=Bash`、带 `agent_type`、清掉 `SINGLEFS_HEAVY_TESTS`）、看退出码（`hook-probe.py` 喂的是本仓那一份 `.claude/hooks/heavy-test-guard.sh`，钩子只认本仓路径上的登记；检出记录写进草稿目录）。被判的命令一条都没起，没跑任何重型测试。坏树（改过的 `stage-selftest.sh` 拷贝、带假 rustc 的 54 号样本根）都建在草稿目录下，主工作区只读。

## 各格判定一览

| 编号 | 格 | 被攻的 | 判定 | 证据形态 |
|---|---|---|---|---|
| U1-1 | U1 | 命令词带通配的放行面（T2-6 的改法周边） | **打中（真重型放行）**：`bash .claude/gate.d/checker-tier-layer0-replay.s? <根>`、QEMU 那一道、`mutate.s? crates/mutations.tsv`、三份重型 `--selftest`，钩子一律退 0；执行时 bash 展开成真的那一份跑起来 | 钩子 JSON 量过；bash 展开那一步是推的 |
| U1-2 | U1 | 不带 `--item` 的 stage-selftest.sh 只按名字认 | **打中（非本仓拷贝放行）**：改过正文的 `stage-selftest.sh` 拷贝（路径尾部相同），喂一个装着本仓字节相同轻样本的阶段目录，钩子退 0、正文不读；带 `--item` 的同一条被拒 | 钩子 JSON 量过；拷贝正文里起 checker 档那一步是推的 |
| U2-1 | U2 | 崩溃放量入口改后的判法 | **没打中**：有红先于 78、退 78 而行里报红记红、缺 red= 判红、零态判红、点名取并集，试过的形状都照文件头走（退出码对得上） | 假驱动量过 |
| U2-2 | U2 | `mutate.sh` 252/254 的分类 | **没打中**：252「排不上」、254「被总上限挤掉」各单列、整轮退 1；平常跑带只供自证的变量退 2；并行理由只设在自己起的包装里、不 export | 读代码、看自证登记 |
| U2-3 | U2 | 54 号样本档核假工具 | **没打中重活**：假 rustc 不核内容是文件头写明的「罩不住的」，但样本流程里没有一处起真 rustc，放进去的恶意 rustc 不被调用 | 钩子 JSON、读 54 号正文 |

## U1-1 命令词带通配时，重型测试闸整条看不见——真层 0、真 QEMU、真变异整表、重型自证都放行

这是这一轮最重的一处。派发提示点名要找的「尤其命令词带通配的不当自证」「通配写法 `bash research/scripts/heavy*.sh --selftest` 一类」，打中的面比自证宽得多：**凡命令词里带 `*`、`?`、`[`，钩子对它一无所知、一律放行，执行时 bash 把它展开成真的重型目标跑起来。**

两处机理，缺一这一格就不中：

1. 判这一条是不是重型，`classify` 按 basename 取名字（`.claude/hooks/lib_heavy_tests.py:1693` `name, arguments = os.path.basename(words[0]), words[1:]`）。`checker-tier-layer0-replay.s?` 的 basename 不等于 `checker-tier-layer0-replay.sh`，`gate_stage_name` 的正则 `GATE_STAGE_PATH`（`\.sh$`）不命中；`vm-bench*.sh`、`mutate.s?` 同理对不上 `if name == "vm-bench.sh"`、`if name == "mutate.sh"` 那几支。自证这一侧 `classify_selftest` 更是直接把带通配的命令词挡在门外（`.claude/hooks/lib_heavy_tests.py:939` `if SELFTEST_PATTERN_CHARACTERS & set(word) and not break_is_set("selftest-pattern-words-judged"): return False, None`），于是整条 `classify` 走到末尾返回 None（放行）。
2. 把执行的脚本读进去逐层判这一道也看不见它：`script_path_of` 命令词里有 `NOT_A_LITERAL_PATH`（`.claude/hooks/heavy-test-guard.sh:260`，集合里含 `*?[`）就交 (None, None)、不读（`.claude/hooks/heavy-test-guard.sh:428` `if command.launcher == "python" or word.startswith("-") or NOT_A_LITERAL_PATH & set(word): return None, None`）。

两头都失手，命令退 0。`outputs/wild-probes.out` 原样（退 0 = 放行；括号里是 bash 展开后真跑的那一份）：

```
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=bash .claude/gate.d/checker-tier-layer0-replay.s? /home/fy5090/code/singlefs	stderr1=
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=bash .claude/gate.d/checker-tier-qemu-device-streams.s? /home/fy5090/code/singlefs	stderr1=
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=bash research/scripts/mutate.s? crates/mutations.tsv	stderr1=
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=bash research/scripts/vm-bench*.sh --selftest	stderr1=
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=bash research/scripts/crash-amplification-two-host?.sh --selftest	stderr1=
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=python3 research/scripts/compile-then-swap.p? --selftest	stderr1=
```

非通配对照（同一个 agent、同一份钩子，照拒）：

```
exit=2	…	command=bash .claude/gate.d/checker-tier-layer0-replay.sh /home/fy5090/code/singlefs	stderr1=✗ 重型测试被拒：门禁 checker-tier-layer0-replay.sh…
exit=2	…	command=bash research/scripts/vm-bench.sh --selftest	stderr1=✗ 重型测试被拒：research/scripts/vm-bench.sh（--selftest 也起虚机）（QEMU）…
exit=2	…	command=cargo test -p singlefs-checker-tier	stderr1=✗ 重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试…
```

可达的历史：任何子 agent 想跑真层 0，写 `bash .claude/gate.d/checker-tier-layer0-replay.s? <根>`（目录里只有 `checker-tier-layer0-replay.sh` 一份，`.s?` 只展开到它）；想起 QEMU 写 `checker-tier-qemu-device-streams.s?`；想跑 crates 变异整表写 `mutate.s? crates/mutations.tsv`。一个 `?` 就够，不需要造样本、不需要字节相同。

这是「打中不分辨臂」那一型（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：它让每一类重型一起出局，病根在各条判法共用的前提——通配命令词一律看不见。所以不拿它判某一类，把共用前提另立一笔账先修。

四问：
- 分不分辨臂：不分辨。层 0、QEMU、crates 变异、几份重型自证一起放行，病根在 `script_path_of` 与 `classify` 对通配词的共同处理。
- 被判的系统当时看不看得到判别它的东西：看得到。判的那一刻这些文件在盘上，钩子可以把带通配的命令词按当前目录 glob 展开、逐个 `classify`，有一个展开成重型就拒（像 `classify_selftest` 对通配词那样保守地不放）。它现在选择不展开（文件头 `.claude/hooks/heavy-test-guard.sh:123` 把通配写的命令路径列进「看不见的（照常放行）」）。
- 满足的是判据字面的哪一个分句：U1「收紧之后还有哪条子 agent 会写的命令能跑起真重活（尤其通配写法 `bash research/scripts/heavy*.sh --selftest` 一类）」。
- 跑前条款给的每个改法在打中的格上还中不中：这一格第一轮没给过改法，是零轮形态。我提的改法（推的、只在我的模型上量过、被攻过零轮、没实现）：`script_path_of` 与 `classify` 遇到命令词带 `*?[` 时，按 `command.directory` 把它 glob 展开，逐个展开结果 `classify`／当脚本读；有一个展开成重型目标、按名字判的重型脚本或门禁阶段就拒（展开不出唯一的一份时也拒：判不出跑的是哪一份）。保守方向与自证那一侧已有的做法一致。

## U1-2 不带 --item 的 stage-selftest.sh 只按名字认：改过正文的非本仓拷贝放行、正文不读

第一轮给 `stage-selftest.sh` 加了 `--item` 放行，并要求带 `--item` 时命令词解到的真实路径正是本仓那一份（`stage_selftest_is_own_copy`，`.claude/hooks/lib_heavy_tests.py:1206`；在 `stage_selftest_item_problems` 第 1218 行拦）。可**不带 `--item` 的那条路没加这个检查**：`stage_selftest_problems`（`.claude/hooks/lib_heavy_tests.py:1242`）走到第 1250 行 `positional = [...]` 之后，只核喂进去的阶段目录里配样本的每一道（阶段文件与样本字节相同），从不核 `stage-selftest.sh` 本身是不是本仓那一份。配上 `judged_by_name`（`.claude/hooks/lib_heavy_tests.py:1790`）把 `stage-selftest.sh` 当按名字判的脚本、不读正文，一份改过正文的拷贝就能带着任意重活放行。

文件头把这一条写成「老口子」：`.claude/hooks/lib_heavy_tests.py:128`「不带 --item / --list-items 的 ① 仍按名字与位置认 stage-selftest.sh：别处树里路径尾部相同、改过的拷贝喂一个轻的阶段目录照放行（老口子）」。第一轮给 `--item` 补了 own-copy，没给这条不带 `--item` 的补，收紧之后口子还在。

`outputs/probes.out` 原样（U1-a 节）：

```
拷贝末尾那一行（本该起真 checker 档）：
# EVIL: SINGLEFS_HEAVY_TESTS=commit cargo test --release -p singlefs-checker-tier --lib --tests
exit=0	agent=implementation-writer	cwd=/home/fy5090/code/singlefs	command=bash /tmp/claude-1000/test-script-align-r2-opus/run/evil-stage-selftest/.claude/singlefs-ai-sop/scripts/stage-selftest.sh /tmp/claude-1000/test-script-align-r2-opus/run/evil-stage-selftest/gate.d	stderr1=
```

同一份改过的拷贝、同一个阶段目录，带 `--item` 就被拦（own-copy 检查），带坏样本也被拦（样本字节不符）：

```
对照 1：…stage-selftest.sh …/gate.d --item doc-text.sh	stderr1=✗ …解到的不是本仓那一份 /home/fy5090/code/singlefs/.claude/singlefs-a…
对照 2：…evil-stage-badsample…stage-selftest.sh …/gate.d	stderr1=✗ …doc-text.sh 的样本与 /home/fy5090/code/singlefs 的 HEAD 加暂存区不同…
```

对照 2 说明不带 `--item` 这条路**唯一**拦得住的是「喂的样本与本仓暂存区不符」；脚本本身可以是别处树里改过正文的拷贝。造法：子 agent 拷一份 `stage-selftest.sh` 到 `<草稿>/.claude/singlefs-ai-sop/scripts/`（路径尾部相同）、往正文里塞重活，再建一个 `gate.d`、把本仓某一道轻阶段连同它的 `fixtures/` 原样拷进去（字节相同），`bash <草稿>/.../stage-selftest.sh <草稿>/gate.d` 就放行，拷贝正文里的重活随之跑起来。

四问：
- 分不分辨臂：分辨。只打中不带 `--item` 的 ① 这一条放行路；`--item` 与直接起（②）都要 own-copy 或标记逐字节相同。
- 看不看得到：看得到。`--item` 那条已经用 `stage_selftest_is_own_copy` 核过命令词解到的真实路径，这条路同样拿得到。
- 字面分句：U1「不带 `--item` 的 stage-selftest.sh 只按名字认」。
- 改法（推的、只在我的模型上量过、被攻过零轮、没实现）：`stage_selftest_problems` 不带 `--item` 的那条路开头也调 `stage_selftest_is_own_copy`，命令词解不到本仓那一份就照旧算提交时才跑的检查、拒。合法用法（`gate.sh`、提交前 `gate-staged.sh` 起本仓那一份喂 `.claude/gate.d`）都是 own-copy，不受影响。

## U2 入口与脚本的改后判法：试过的形状都没打中假绿／假未跑／假红

崩溃放量入口与 `mutate.sh` 的改后判法，用假驱动、假包装喂了一圈，第一轮判决采的那几条改法都站得住，没找到新的假绿、假「本次未跑」、假红。这里是观测（跑命令、看退出码），不是模型主观的「找不到」。

### U2-1 崩溃放量入口

第一轮采的改法在 `research/scripts/crash-amplification.sh` 里都落成了会红的判法与自证格：
- 有红先于 78：`research/scripts/crash-amplification.sh:855` `if (( ! run_verdict )) || crash_amplification_break_is condition-before-red; then exit 78; fi`——有红（`run_verdict`）就走到 858 退 1，退 78 那一项照样在 ✗ 里点名（自证第 22 格）。
- 退 78 而行里报红记红：`research/scripts/crash-amplification.sh:780` 一带，`item_stopped_by_condition` 且 `red_reasons` 非空时 `verdict=red`（文件头「一项怎么判」，自证第 9、22 格）。
- 缺 `red=` 或不是非负整数判红：`research/scripts/crash-amplification.sh:190` `if (!red_seen) bad_red++`，主循环第 775 行把 `item_bad_red>0` 收成红（自证第 26 格）。
- 不带参数的全量里一项现判 0 态、复用 0 态判红：`research/scripts/crash-amplification.sh:791`（自证第 27 格）。
- `ignored_points`、`states_skipped_by_ignore` 读进来、成功句报跳过了多少：`research/scripts/crash-amplification.sh:191`、第 887 行的 `ignore_part`（自证第 24、25 格）。
- 只供自证的五个变量平常跑退 2：`research/scripts/crash-amplification.sh:658`（文件头「只供自证」）。

试过的形状：前一项红＋后一项退 78（退 1）、退 78 而 red=2（记红）、零态全量（退 1）、缺 red=／red=?／red=-1（各退 1）、`ignored_points` 带数而成功句照报。退出码都对得上文件头「退出码」表，没打中。

### U2-2 mutate.sh 的 252/254 与并行理由

`research/scripts/mutate.sh:980` 把包装退 252 收成 `queuefull`、判失败位 1；第 984 行把退 254 收成 `squeezed`、判失败位 1；两支都排在「编译失败、无效」与「没跑完」两支之前（文件头「包装的另两种结局」）。自证里有专门一格核这两种各归各类、整轮退 1（`research/scripts/mutate.sh:469` 那个 `for outcome_label in queue-full squeezed`）。平常跑带只供自证的变量退 2（`research/scripts/mutate.sh:627` 一带）。并行理由 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 只设在自己起的每次包装的环境里、不 export（`research/scripts/mutate.sh:880` 的 `parallel_environment`），`--check` 与别的命令看不见它，没让「日常一条」对别的命令失去意义。

调用方读法：门禁 `checker-tier-crates-mutation-replay.sh` 走的是 `research/scripts/crates-mutation-rows.py`（`run_stage`），不经 `mutate.sh`；它自己认 252/254（`research/scripts/crates-mutation-rows.py:640`、第 641 行），读 `run.returncode`。两条路各自分类，没找到退出码对不上的地方。`mutate.sh` 文件头退出码表把 252/254 都归进「退 1」那一行，与收尾单列计数一致。没打中。

### U2-3 54 号样本档核假工具：假 rustc 不核内容，但不被调用

钩子 `stage_sample_marker_matches` 只核 `STAGE_SAMPLE_FAKE_TOOLS` 登记的假工具（`.claude/hooks/lib_heavy_tests.py:267` 里只有 `cargo`），假 rustc 不核内容——文件头 `.claude/hooks/lib_heavy_tests.py:127` 自己写明「② 只核 STAGE_SAMPLE_FAKE_TOOLS 登记的假工具（54 号的假 rustc 不核）」。我造了一个样本根：假 cargo 与本仓 green 样本逐字节相同、可执行，假 rustc 换成一行 `echo EVIL-RUSTC` 的可执行脚本。钩子放行（`outputs/probes.out` U2-a 节 `exit=0`）。但 54 号样本流程里起的是那份字节相同的假 cargo（只打日志、不调 rustc，`.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/green/.layer0-sample-tools/cargo` 原样），阶段正文里也没有一处直接执行 rustc（`.claude/gate.d/checker-tier-layer0-replay.sh:242` 只核它在、可执行，不调用）。放进去的恶意 rustc 没有被调用，跑不起真重活。没打中重活；只是一处与文件头相符的未核项。

## 没打中的形状

- **崩溃放量入口**（U2-1）：有红先于 78、退 78 记红、缺 red= 判红、零态判红、点名取并集、`ignored_points` 报数——试过的形状都照文件头走。只用假驱动跑了入口这一层，没跑真的分项驱动。
- **mutate.sh**（U2-2）：252/254 各单列判失败、平常跑拒只供自证的变量、并行理由不 export。读代码与自证登记，没用假包装把 252/254 再演一遍（第一轮已演过，这一轮不重复）。
- **54 号样本档假 rustc**（U2-3）：钩子放行，但样本流程不调 rustc，放进去的恶意 rustc 不被执行；是未核项，不是起真重活的口子。
- **样本刚改没暂存时被拒**（U1 的「有没有该放的被误拒」）：`sample_directory_changes` 拿 HEAD 加暂存区比，改完没暂存的样本照拒——这是第一轮 T2-4 采的改法，字面如此，不算误拒。
- **改钩子的人自己的自证**：钩子只认本仓路径上的登记，仓副本里跑不了 `--selftest`；照本仓路径只读地跑是设计如此，没找到该放被误拒的新例。

## 这条腿自己的限度

- 跑过的全是喂钩子 JSON、看退出码；U1-1、U1-2 的最后一步（bash 展开通配后真跑起重型、改过的拷贝正文里起 checker 档）是按 bash 展开与 `judged_by_name` 不读正文推的，我没有在带 git 的真树上真起那些重型目标（那就是跑重型测试）。
- 自己提的两个改法全是推的、只在我的模型上想过、被攻过零轮、没实现；写回时要标「被攻过零轮」，并各记一笔自己的检查账。
- U1-1 的改法（判定时 glob 展开命令词）会不会误拒某些合法的通配写法、glob 展开与 bash 在不同当前目录下是否一致，我没有扫全，交主 agent 判。
- U2 几格是「没打中」：按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，这里的「没打中」是确定性观测（跑命令看退出码），不是模型主观判断，但仍只覆盖我试过的那几种形状，没穷举。

## 没做什么

- 没判 U3、U4（辩方那两格）；没替主 agent 采纳；坏树上量出的放行只是副本观测，主 agent 要坐实请在本仓路径上只读地复喂同一批 JSON。
- 没跑任何重型测试：没真起 54 号、QEMU、crates 变异整表、`mutate.sh` 的真 cargo、重型自证。
- 没改主工作区任何文件；写进去的只有本报告与模型目录。
- 草稿目录 `/tmp/claude-1000/test-script-align-r2-opus/` 里留着 `run/`、`wild/` 两棵坏树（改过的 `stage-selftest.sh` 拷贝、假 rustc 的 54 样本根）与检出记录，没有编译目录、没有仓副本。交回前删掉，见交回正文。
