# 第二轮通读定义（read-R2）报告

日期：2026-10-17。批次：records/2026-10-17-全仓知识腐烂治理.md#批6。草稿目录：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-R2。
读的是工作区那一版（`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/mutation-triage.md` 在暂存区之外另有别的会话未暂存的改动，已按工作区读，没有追查那些改动）。
没建副本，没起编译，没改任何文件。

## 做法（可复核的命令）

- 16 份全文读完，不抽样。行数：`wc -l .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md`。
- 路径存在：从 16 份里抽出全部 `.claude/…`、`scripts/…`、`crates/…`、`research/…`、`records/…` 路径逐个 `[ -e ]`，缺的只有带 `<…>` 或 `*` 的形态写法与补丁目录里的 `crates.patch`、快照里的 `crates-sha256.txt`（都是形态）。
- 小节引用：把 16 份里「文件「小节」」的引用 70 余处逐条 `grep -nF`，全部命中（`所有旧数据都只是参考`、`核查与判决由主 agent 做`、`攻方腿的装置把用户动作写死时` 是正文里的加粗句、不是标题，引法是「那一条」，不算错）。
- 门禁格名：`grep -l 'gate-cell: <格名>' .claude/gate.d/*.sh`，archive-past-rounds、clock-times、prime-marks、private-values、format-constants、kb-shape、blocking-verdict、decision-links、results-cited、experiment-orphans、evidence-in-repo、knowledge-sync、implementation-premise、agent-def-adversarial-review、mutation-tables、agent-write-scope、research-script-selftests、experiment-replay、research-unit-tests、abandoned-rounds 全在。
- 钩子注册：`python3 -c` 读 `.claude/settings.json` 的 `hooks`，runner-dispatch-guard（Agent|Task）、continuation-guard（SendMessage）、handback-guard 与 handback-scratch-check（SubagentHandback）、ask-user-claim-guard（AskUserQuestion）、write-guard（Write|Edit|MultiEdit）、heavy-test-guard 与 run-scope-guard 与 bash-command-detector（Bash）都在。
- 编号对表：派发闸 ①–⑯ 对 `.claude/hooks/runner-dispatch-guard.sh` 文件头（⑤ 实现员、⑥ 快照、⑦ 并发 5、⑧ 规格、⑨ 准入 77、⑪ 岔路、⑫ 通用、⑬ 占位、⑭ 长活十种、⑮ 线程九种、⑯ 写者锁）；Bash 检出 ①–⑧ 对 `bash-command-detector.sh` 文件头；写闸第九道对 `write-guard.sh` 文件头；弹窗闸断言词九个对 `ask-user-claim-guard.sh` 文件头；交回正文闸 4000 字对 `handback-guard.sh`。全对得上。
- 退出码与数：`scripts/hosts/run-with-memory-cap.sh` 文件头「退出码」245、250–254、252；`scripts/agents/watch.conf` 与 `agent-watch.py` 的 8 分钟、10 分钟、20 分钟、30 分钟、8 GiB、120 分钟；`PRIME_MARKS` 五个码位；`.claude/heavy-thread-default` 的 31；变异 `⚓` 等标记与「计数：」六栏对 `scripts/mutation/mutate.sh`、`scripts/mutation/crates-mutation-rows.py:2443`。全对得上。
- 脚本选项：约 40 个脚本的选项字面（`--skip-apply`、`--wall-clock`、`--emit-hosts`、`--check-facts`、`--groups`、`--scratch`、`--clean`、`--memory-cap`、`--threads` 等）逐个 `grep -qF`，全在。
- 写范围表：`.claude/hooks/agent-write-scope.tsv` 的 clerk、experiment-runner、implementation-writer、investigator、kb-scribe、tooling-writer 各行与六份定义「写范围」一节逐项对过，一致。
- 实验与决策：E156（alloc-basis 四条岔路的代价数）、E158（择根与修复四岔路）的 `driver_e156`、`driver_e158` 在 `scripts/experiments/replay.sh`；E171 修订 8 仍是「待拍树」（`.claude/kb/experiments/171-三oracle合谋测试.md` 第 1 行标题）；D13 已定项 9 在 `.claude/kb/decisions/13-验证路线.md:240`。

## 逐条判定（只列要改、要补、要人看）

表格见下一段。判定顺序：先要改，再要补，最后要人看。

| 文件:行 | 原文（截 120 字） | 判定 | 改后句或理由（带现查命令） |
|---|---|---|---|
| .claude/agent-common.md:56 | 「…轻型三条是 `bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick`、`… bash .claude/gate.d/checker-tier-formal-verification.sh --quick <根>`、`… bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根>`…」 | 要改 | 这三条命令照原样写已被范围声明闸拒：`.claude/hooks/run-scope-guard.sh` 文件头「门禁阶段 …带 --quick 是 quick」「崩溃点重放 scripts/crash/crash-point-replay.sh：--quick 是 quick」，要求命令开头带 `SINGLEFS_RUN_SCOPE`；该钩子已注册（`python3 -c` 读 `.claude/settings.json`：PreToolUse / Bash 下有 run-scope-guard.sh）；`.claude/rules/verification.md:58`「起任何验证（轻型三条、重型……）的命令开头带范围声明」；`.claude/agents/implementation-writer.md:35`（第 1d 步）已带前缀。改后句：「轻型三条是 `SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=…; 范围=…; 为什么=…' bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick`、同样前缀加 `… bash .claude/gate.d/checker-tier-formal-verification.sh --quick <根>`、同样前缀加 `… bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根>`（范围声明四段的写法照 `.claude/hooks/run-scope-guard.sh` 文件头，整条示例见 `.claude/agents/implementation-writer.md` 第 1d 步；都可再带 `--item <项>`，…）」，后半句照旧。 |
| .claude/agent-common.md:46 | 「…副本只拷要编的（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`，要门禁表的再加 `.claude/gate.d`），不拷 `research/`、`target` 这类大目录。」 | 要改 | 只拷这几样的副本编不过：`grep -n 'include_str' crates/singlefs-harness/src/heavy_threads.rs` 命中 `:22`（`REGISTRATION_FILE_TEXT: &str = include_str!(concat!(…`，读 `.claude/heavy-thread-default`）；`scripts/hosts/compile-then-swap.py` 文件头做法第 1 步「与每台默认重活线程数的登记文件 .claude/heavy-thread-default（harness 包经 include_str! 在编译时读它，副本里没有就编不过）」也拷。改后句：「副本只拷要编的（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，要门禁表的再加 `.claude/gate.d`），不拷 `research/`、`target` 这类大目录。」 |
| .claude/main-agent.md:7 | 「…主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>`（打全部补丁、编一次、跑工作区除 checker 档包 `singlefs-checker-tier` 之外各包的全部测试、fmt 与 clippy；…」 | 要改 | 默认已不跑「各包的全部测试」：`scripts/gate/merge-and-verify.sh` 文件头「测试只跑改到的包：上面那几个包里只留这一批补丁改到的包与工作区里依赖它们的包」「--all-packages 测试步照旧跑工作区成员减掉 checker 档包的全部包」；`git show HEAD:scripts/gate/merge-and-verify.sh \| grep -c all-packages` 为 0、工作区版为 13（`grep -c`），是这一阶段暂存的改动。改后句：「…编一次、跑补丁改到的包与工作区里依赖它们的包（都除 checker 档包 `singlefs-checker-tier`；带 `--all-packages` 才跑工作区除它之外的全部包）的测试、fmt 与 clippy；…」 |
| .claude/main-agent.md:29 | 「…（一条 `cargo test -p <包>…`）」前文「编一次、跑工作区除 checker 档包之外各包的测试」 | 要改 | 同上，依据 `scripts/gate/merge-and-verify.sh` 文件头第 5–8 行与「test」一段。改后句：「编一次、跑补丁改到的包与依赖它们的包（除 checker 档包；`--all-packages` 跑全部）的测试（一条 `cargo test -p <包>…`），红了只重跑红的那几条」。 |
| .claude/main-agent.md:90 | 「…跑工作区除 checker 档包 `singlefs-checker-tier` 之外各包的全部测试（包从树根 `Cargo.toml` 的 `[workspace] members` 现读）、fmt、clippy、build…」 | 要改 | 同上。包仍从 `[workspace] members` 现读（`merge-and-verify.sh` 文件头「test … 包是工作区成员（树根 Cargo.toml 的 [workspace] members…）」），但默认只留补丁改到的包与依赖它们的包。改后句：「…跑补丁改到的包与依赖它们的包（工作区除 checker 档包 `singlefs-checker-tier`，成员从树根 `Cargo.toml` 的 `[workspace] members` 现读；`--all-packages` 跑全部）的测试、fmt、clippy、build…」 |
| .claude/agents/mutation-triage.md:33 | 「4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」…」 | 要改 | 表里每条都按判定库复用时收尾不是这句：`scripts/mutation/mutate.sh:22`「全部复用时不起 cargo：…收尾打一行「全部复用」代替「已还原，基线仍全绿」」，`mutate.sh:1961` 打「全部复用：表里 … 没起 cargo」。改后句：「`mutate.sh` 收尾要有「已还原，基线仍全绿」，或（表里每条都按判定库复用时）「全部复用：表里 … 没起 cargo」」。另：`mutate.sh` 的「计数：」行在六栏之后还有「现判 N 条、复用 N 条、拒算 N 条」三栏（`mutate.sh:593` 的正则），第 5 步（`.claude/agents/mutation-triage.md:34`）写的「六栏」仍对，只是不全，是否补三栏见「要人看」。 |
| .claude/agents/experiment-runner.md:36 | 「3. …`cd scripts/experiments && nice -n 19 bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<变异表>`…（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。」 | 要改 | 同上一行的依据（`mutate.sh:22`、`:1961`）。改后句：「（收尾要有「已还原，基线仍全绿」，或表里每条都按判定库复用时的「全部复用」）」。 |
| .claude/agents/three-way-attack.md:35 | 「…照崩溃点重放的枚举域取：调同一份文件里每一段都展开的 `enumerate_crash_states` / `enumerate_crash_states_versions`（录制写流，屏障与 FUA 切段…」 | 要改 | 「同一份文件」前文没有指代，下一条才点名 `crates/singlefs-checker-tier/src/crash.rs`，读者会当成 crash.rs；两个函数实际不在那里：`grep -rn 'fn enumerate_crash_states' crates --include=*.rs` 命中 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs:242` 与 `:283`，`crash.rs` 里没有这两个 `fn`。改后句：「调 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 里每一段都展开的 `enumerate_crash_states` / `enumerate_crash_states_versions`（…）」。 |
| .claude/agent-common.md:71-88 | 「- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法…」（列了 Bash 检出 ①–⑧、上游 pattern-process-guard、heavy-test-guard、write-guard、交回闸、弹窗闸、派发闸、续做闸、交回正文闸） | 要补 | 清单漏了三道已注册、在执行前拒的闸。① 范围声明闸：`.claude/hooks/run-scope-guard.sh`，挂 Bash，主 agent 与子 agent 都判（`.claude/settings.json` hooks.PreToolUse / Bash；`git status --short .claude/hooks/run-scope-guard.sh` 为 `A`，是这一阶段新加）。② 定时闸：`.claude/hooks/timer-only-remind-guard.sh`，挂 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor、Bash，除 `scripts/agents/remind.sh` 之外的定时办法一律拒，主 agent 与子 agent 一样（文件头「本工程设定时只许 scripts/agents/remind.sh 这一个办法」）。③ Bash 检出 ⑨（先编后换：`experiment-runner` 在 Bash 里写主工作区 `crates/` 下的 `.rs`，出路 `scripts/hosts/compile-then-swap.py`；`bash-command-detector.sh` 文件头有 ⑨、⑩，本清单只到 ⑧，⑩ 已在第 67 行说）。建议补三条：「范围声明闸（`.claude/hooks/run-scope-guard.sh`，挂 Bash，主 agent 与子 agent 一样判）：起验证的命令（门禁阶段、崩溃点重放入口、崩溃放量、`replay.sh`、`merge-and-verify.sh`、`gate-staged.sh`、`cargo test`）开头没带 `SINGLEFS_RUN_SCOPE='种类=…; 跑什么=…; 范围=…; 为什么=…'`、缺段、种类或范围数与命令形状对不上；子 agent 起 whole 一律拒。」「定时闸（`.claude/hooks/timer-only-remind-guard.sh`，主 agent 与子 agent 一样）：`scripts/agents/remind.sh` 之外的定时写法。」以及 Bash 检出清单里补 ⑨。 |
| .claude/agent-common.md:86 | 「- 派发闸（…）：派任何类型，提示里没有一行「重型测试：不跑」；派 general-purpose 这类内置通用类型…」 | 要补 | ① 的判法比这句宽：`.claude/hooks/runner-dispatch-guard.sh` 文件头 ①「有那一行也照判提示正文里许可子 agent 跑 checker 档 / 重型测试的句子（heavy_permission_sentences），有一句就拒」。建议在「没有一行「重型测试：不跑」」后补「、或提示正文里有许可子 agent 跑 checker 档 / 重型测试的句子（轻型三条带 `--quick` 的写法除外）」。 |
| .claude/main-agent.md:29（也可放第 50 行验证段第 1 步） | 「…一段实现的代码与测试基本写完之后…主 agent 跑 `bash scripts/gate/merge-and-verify.sh --skip-apply …`做「合入后验证」…」；第 94 行「…带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `scripts/gate/gate-staged.sh`…」 | 要补 | 主 agent 起合入后验证、轻型三条、`gate-staged.sh`、`cargo test` 点名包与目标的命令也要带范围声明，全文没提：`.claude/rules/verification.md:58`「起任何验证（轻型三条、重型、门禁阶段的一道或几格、`cargo test` 点名的包与目标、合入后验证、实验复跑）的命令开头带范围声明」；`.claude/hooks/run-scope-guard.sh` 文件头「合入后验证 scripts/gate/merge-and-verify.sh：whole」「整轮门禁 scripts/gate/gate-staged.sh…：whole」；`.claude/skills/crash-test/commit-stages.md` 第 1b 步已写 `SINGLEFS_RUN_SCOPE='种类=whole; …'`。建议补一句：「起验证的命令（合入后验证、轻型三条、`gate-staged.sh`、`cargo test`）开头带 `SINGLEFS_RUN_SCOPE='种类=…; 跑什么=…; 范围=…; 为什么=…'`，四段的写法与种类的判法以 `.claude/hooks/run-scope-guard.sh` 文件头为准（`.claude/rules/verification.md`「两档：轻型与重型」一节里「起任何验证」那一条，第 58 行）。」 |
| .claude/agents/implementation-writer.md:48（7a） | 「…报告里给出要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行原文（`test=singlefs-checker-tier:…`），由主 agent 或 tooling-writer 登记；…」 | 要补 | 只登记 stage-inputs 那一行，崩溃点重放入口仍不跑它：`.claude/rules/verification.md:74`「全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`，再让崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的一项在第二列点名它（没被点名的，入口 `scripts/crash/crash-point-replay.sh` 不跑，`scripts/crash/crash-case-check.py` 的 placement 判红）」；`crash-case-check.py` 文件头 ③④ 同。`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 为 14，与合表表头「14 条 crash-case:」一致。建议改成：「…给出要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行原文（`test=singlefs-checker-tier:…`），与 `.claude/gate.d/crash-point-replay-items.tsv` 里点名它的那一项（第二列写 `crash-case:<名>`，其余列照合表表头），由主 agent 或 tooling-writer 登记；…」 |
| .claude/agents/implementation-writer.md:34 与 :40（1b、第 4 步） | 「1b. 第 3 步的证红、第 4 步动到的测试二进制…经内存包装跑」；「…动到的 harness 档测试二进制（整个二进制，不按名字挑；…）…各贴末尾原样输出」 | 要补 | 这些 `cargo test -p <包> --test <目标>` 命令现在要带范围声明（`.claude/rules/verification.md:58`；`run-scope-guard.sh` 文件头「cargo test … 带 --test / --bin / --lib… 的是 target，N = 目标数」）。第 1d 步已写形式化那条的前缀，测试二进制这一步没写。建议在 1b 末补：「`cargo test` 点名包与目标的命令同样开头带 `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=…; 范围=1 个测试目标; 为什么=…'`（写法照第 1d 步引的 `.claude/hooks/run-scope-guard.sh` 文件头）」。拿不准处：被拒时钩子会给照抄的模板，所以不补也能自己改过来，属低优先。 |
| .claude/agents/experiment-runner.md:31（1b） | 「1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（例 `scripts/experiments/target/release/e<号>-<简称>`）照共用约束「不做」一节…经包装跑；…」 | 要补 | 同上一行：`cargo test` 点名目标的命令要带 `SINGLEFS_RUN_SCOPE`（`cargo run` 与直接跑装置二进制不在 `run-scope-guard.sh` 的表里，不要求）。低优先，理由同上。 |
| .claude/agents/three-way-attack.md:53 | 「…副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`，经内存包装、…）」 | 要补 | 同上（`cargo test -p <crate> --test <目标>` 是 target 种类，要声明）。低优先。 |
| .claude/agents/tooling-writer.md:51 | 「- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、`gate.sh`、全量 `cargo test`、`check.sh`）…」 | 要补 | 括号里的重型清单漏了 checker-tier-broad-mutation 与广谱变异驱动：`.claude/agent-common.md:56` 把「广谱变异（`scripts/mutation/broad-mutation.py` …）与它的提交门禁 checker-tier-broad-mutation」列为重型；`.claude/hooks/heavy-test-guard.sh` 文件头「广谱变异」一类。建议在括号里补「checker-tier-broad-mutation、`scripts/mutation/broad-mutation.py` 起 cargo 的用法」。低优先。 |
| .claude/agents/kb-scribe.md:22 | 「…`.claude/kb/decisions.md` 状态列由第 3 步的 `doc-decisions.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。」 | 要人看 | 句子不假，但不全：`.claude/gate.d/doc-decisions.sh:39` 的 `--write`「重写 decisions.md 的分项清单生成块与索引表「状态」「进度」两列」。写回员若只当「状态列」生成，会去手写进度列或分项清单块。拿不准要不要补，因为同一段后文已要求规格给「**进度：…**」标记。若补：「`.claude/kb/decisions.md` 的分项清单生成块与索引表状态、进度两列由第 3 步的 `doc-decisions.sh --write` 重生成…」 |
| .claude/agents/mutation-triage.md:34（第 5 步） | 「…`mutate.sh` 收尾那行「计数：」报内存撞顶、超时、排不上、被总上限挤掉、显存不够、锚点失效六栏；…」 | 要人看 | 六栏仍在，但 `scripts/mutation/mutate.sh:593` 的收尾正则是 `^计数：.*、锚点失效 1 条、现判 [0-9]+ 条、复用 [0-9]+ 条、拒算 [0-9]+ 条$`，行尾多了现判、复用、拒算三栏（`mutate.sh:22` 起的「按条入库」）。这三栏不是结局，不并进三个数，所以「六栏」不算错；是否要写一句「其后另有现判、复用、拒算三栏，不并进三个数」由主 agent 定。`experiment-runner.md:53`（产出）同样只列六栏。 |
| .claude/agent-common.md:92 | 「…子 agent 交回前不跑门禁阶段与各类 lint（gate-lint、shell-lint、preflight-lint、gate-overlap、doc-lint、rules-lint、hooks-registered）；…」 | 要人看 | 括号里七个都在（`ls .claude/singlefs-ai-sop/scripts/` 有 gate-lint.sh、shell-lint.sh、preflight-lint.py、gate-overlap.py、doc-lint.sh、rules-lint.sh、hooks-registered.sh），但 `.claude/hooks/heavy-test-guard.sh` 文件头「提交时才跑的检查」那张表还列了 `scripts/gate/gate-structure-check.py`（不带 --selftest 的）。括号是例举还是穷举，定义没说；若是穷举要补。 |
| .claude/agents/experiment-runner.md:34（第 2 步） | 「…在 `scripts/experiments/Cargo.toml` 末尾追加它的 `[[bin]]`…」 | 要人看 | 文件现状：`grep -n '^\[profile' scripts/experiments/Cargo.toml` 命中 `747:[profile.release]`，最后一个 `[[bin]]`（e178）在 `:734`，其后是注释与 `[profile.release]` 表（共 750 行）。前几次追加都落在 `[profile.release]` 之前，不是文件末尾；写在文件末尾（表之后）TOML 上也合法，所以不碍事。是改成「`[profile.release]` 之前、最后一个 `[[bin]]` 之后」，还是保留「末尾」，由主 agent 定。`.claude/hooks/agent-write-scope.tsv` 的 Cargo.toml 行同样写「在末尾追加」。 |

数行：`grep -c '^| \.claude/' <本报告>` 得 36 = 逐条判定 20 行（要改 8、要补 8、要人看 4）+ 汇总 16 行。

## 汇总

「现状句数」= 去掉 frontmatter 后按 `。` 与换行切句，含反引号片段、阿拉伯数字或「门禁 / 闸 / 钩子 / hook」字样的句数（`python3` 小脚本现算，机械数，不等于我逐句现查的件数；每句我都读了，含这些特征的句子都逐项对过仓）。「不改」的行不逐行列，数 = 现状句数 − 被判要改、要补、要人看的句子。

| 文件 | 行数 | 现状句数 | 要改 | 要补 | 要人看 |
|---|---|---|---|---|---|
| .claude/agents/clerk.md | 42 | 13 | 0 | 0 | 0 |
| .claude/agents/experiment-designer.md | 68 | 39 | 0 | 0 | 0 |
| .claude/agents/experiment-runner.md | 58 | 55 | 1 | 1 | 1 |
| .claude/agents/implementation-writer.md | 63 | 51 | 0 | 2 | 0 |
| .claude/agents/investigator.md | 48 | 20 | 0 | 0 | 0 |
| .claude/agents/kb-scribe.md | 52 | 32 | 0 | 0 | 1 |
| .claude/agents/mutation-triage.md | 47 | 19 | 1 | 0 | 1 |
| .claude/agents/prior-art.md | 40 | 7 | 0 | 0 | 0 |
| .claude/agents/sweep.md | 61 | 31 | 0 | 0 | 0 |
| .claude/agents/three-way-attack.md | 61 | 31 | 1 | 1 | 0 |
| .claude/agents/three-way-defense.md | 44 | 14 | 0 | 0 | 0 |
| .claude/agents/three-way-forward.md | 45 | 15 | 0 | 0 | 0 |
| .claude/agents/three-way-materials.md | 45 | 19 | 0 | 0 | 0 |
| .claude/agents/tooling-writer.md | 51 | 27 | 0 | 1 | 0 |
| .claude/agent-common.md | 104 | 105 | 2 | 2 | 1 |
| .claude/main-agent.md | 100 | 76 | 3 | 1 | 0 |

汇总表 16 行（`ls .claude/agents/*.md | wc -l` 得 14，加 `.claude/agent-common.md` 与 `.claude/main-agent.md` 两份）。现状句数的现算命令：`python3 <脚本> .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md`（脚本在 `/tmp/claude-1000/-home-fy5090-code-singlefs/03810af4-8a04-4407-bbc0-dfdd6bfe6a6c/scratchpad/count.py`，切句规则见上）。

## 要主 agent 现查

1. `.claude/hooks/run-scope-guard.sh` 文件头第 32 行写「合入后验证 scripts/gate/merge-and-verify.sh：whole（还不认只跑改到的包，等 4-3i 第 8 条）」，而 `scripts/gate/merge-and-verify.sh` 文件头（这一阶段暂存的改动）已写「测试只跑改到的包」并有 `--all-packages`。两处互相矛盾；钩子文件不在我分到的 16 份里，没判，请定哪边为准。它还决定子 agent 能不能带 target / items 声明起合入后验证（whole 子 agent 一律拒）。
2. 行号取的是工作区。`git status --short` 里 `.claude/main-agent.md` 是 `MM`、`.claude/agents/experiment-runner.md` 是 `MM`、`.claude/agents/mutation-triage.md` 是 ` M`：暂存区之外还有别的会话的未暂存改动（我读到的 `mutation-triage.md:34`、`experiment-runner.md:53` 已是六栏写法）。改定义前请对着最新工作区重取行号。
3. 机械查不了、我没判的数与做法（政策值，不是仓里能现查的现状）：`implementation-writer.md` / `experiment-runner.md` 的「cargo 调用合计不超过 10 次」，`three-way-attack.md` 的「40 分钟」「约 10⁶」「约 10⁷」，`kb-scribe.md` 的「一次不超过 8 条」，`main-agent.md` 的「30 份以上文件」「每段约 200 行」「每一种判定各抽不少于 5 条」「会话过 40 万上下文」，`agent-common.md` 的「1,000 行」「240000 毫秒」。
4. `agent-common.md:103` 的「子 agent 用 Write 建文件名以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头…会被工具层当场拒」是 harness 行为，仓里没有实现，我没验。
5. `.claude/hooks/handback-guard.sh` 文件头 ② 点名 `three-way-verifier`，`.claude/agents/` 里没有这份定义（`ls .claude/agents/` 14 份）：不在我分到的文件里，只记一笔。
6. 带编号的老引用我只核了「编号是否还在」，没核内容：`tooling-writer.md:20` 的「第四十节第 N 行」（`records/2026-09-16-subagent拆分提案.md`）、`agent-common.md:4` 的同一份记录。

## 什么现象会推翻上面的判定

- 要改 1、2（agent-common）：若 `run-scope-guard.sh` 从 `.claude/settings.json` 摘掉，或 `include_str!` 那一行（`crates/singlefs-harness/src/heavy_threads.rs:22`）撤了，则不成立。
- 要改 main-agent 三行：若 `merge-and-verify.sh` 暂存的 `--all-packages` 改动被撤回（`git show HEAD:… | grep -c all-packages` 为 0 即当前提交版），默认又回到全跑。
- 要改 mutation-triage、experiment-runner：若 `mutate.sh:1961` 的「全部复用」输出被去掉。
- 要改 three-way-attack：若 `enumerate_crash_states` 挪回 `crash.rs`。
- 要补各行：若相应的闸被摘掉（run-scope-guard、timer-only-remind-guard），或 `crash-case-check.py` 的 placement 不再要求合表点名。

## 没做什么

- 没建副本、没起编译、没改任何文件，只写了本报告与草稿目录里的小脚本；没跑 `--selftest`、没跑门禁；没读 `~/.claude/` 与会话记录。
- 小节引用只核了标题在不在，没核小节内容是不是还支撑定义里的转述。
- 脚本选项只核了字面在不在，没逐个跑。
- 不改的行不逐行列。
