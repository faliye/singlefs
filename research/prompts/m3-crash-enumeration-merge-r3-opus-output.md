# m3-crash-enumeration-merge-r3 攻方腿（Opus）报告（2026-10-10）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 H1 H2 H3 H4 H5 H6 H7 H8 H9 H10 H11 H12 H13 H14 L1 L2 L3 L4 L5 L6 M1 M2 M3 M4 M5 M6 K1 K2 K3 K4 K5 K6 K7 W -->

攻击面：正文第四节 M1、M2、M3、M4；只攻第三节 H1–H14。K1-1 … K7-2、L1-1 … L6-6 不重攻，只攻它们的修补与它们没碰过的地方。

## 复跑与现场

- 模型目录 `research/prompts/m3-crash-enumeration-merge-r3-opus-model/`。复跑：`bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-crash-enumeration-merge-r3-opus-model/rerun.sh`，末行要是 `MODEL_SELFCHECK compared=20 mismatched=0 []`。
- 它做什么：`extract.sh` 从 `crates/singlefs-checker-tier/src/crash_identity.rs`（钉 sha256 488f75ff…d2f0，变了退 3）按行原样抽出词法切分与「按模块引用收闭包」那几段（第 278–875、1009–1235 行），接上我写的驱动 `closure_main.rs`，编成不带依赖的单文件程序；不链 checker 档的库、不跑 checker 档的任何测试。摘要换成 FNV（只比文件集合）。
- 自证：抽出来的算法在今天的树上复现 `crash_identity.rs` 自己那条测试钉的判器闭包成员（10 份）与非成员（9 份）；会红的一格：不设停止表时 `layer0_progress.rs` 进闭包（`unstopped_has_layer0_progress=true`），停止表那一刀模型分得出。
- `SHA256SUMS` 原样：
  - `0f4e23c4bb956f67ae5196d7ed758ddfd95eee016f26370aa1c28115912f252d  closure_head.rs`
  - `13078f2e1421679abab07c455b5450ab7072f3442e4ce9dea013239c5081a8ee  closure_main.rs`
  - `0cb7bd8f37114c697ac59336be6d43c6eded42327eccb144f911055828c489c8  extract.sh`
  - `69f6440c878c991d8668b99e1c206db6be9cc971c40a7376d87c3d14ca243c2b  rerun.sh`
- 开工快照：`sha256sum -c --quiet research/prompts/m3-crash-enumeration-merge-r3-snapshot/crates-sha256.txt` 退 0，`kb-sha256.txt` 退 0（收工结果在末节）。
- 中途收到主 agent 的定义改动通知（实现员定义第 17 行加一句）：在写闭包模型驱动那一步收到，与这条腿无关，没改任何一步；只是 `.claude/agents/implementation-writer.md` 下面引的那一行因此在第 36 行，按收到之后现查的行号写。
- 负载：开工 `ps` 看到主 agent 的一条 checker 档 `cargo test … --features verdict-store,gpu` 与一条 `cargo fmt` 在跑，没有性能测量；我只编了一个零依赖的小程序（0.7 秒），没等锁。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| M1-1 | 打中 | H9「轻型由实现员跑」与今天的重型测试闸、实现员定义、共用约束「子 agent 一律不跑」字面相反，H2、H12、H14 改的文件里没有这三处；轻型含 GPU 对拍，子 agent 起 GPU 还要清场 |
| M1-2 | 打中 | H5 的豁免表是仓内一张「不走 GPU」的负表，今天选路认的是私有显卡配置里 `GPU_COMPUTATION_CLASSES` 那张「走 GPU」的正表：私有配置漏写 `crash_judge_gpu` 时镜像照判在 CPU 上、不在豁免表里、仓内门禁看不见；H5 没写正表留不留 |
| M1-3 | 打中 | H11「提交那一趟整域在 HEAD + 暂存区的 worktree 里跑」：放量驱动在 worktree 里读的是那棵树根上的 `multi-host.env`（不在），层 0 那套读主工作树的；合成一套时取哪一种没写，取前一种就是配了双机的机器提交时单机跑、违「配了分片就分片跑」 |
| M1-4 | 打中（小） | 「全量」一词撞车：SOP 入口表「不带 = 全量」指全部项，H9「全量（清空复用）」指不复用；H2「不带参数 = 快档全部项加整域全部项」时整域取剪枝还是清空复用没写 |
| M1-5 | 打中（小） | H11「没写 ENABLE_GPU 算开」改的是同一份多机配置，E171 编排脚本照旧按「没写算 0」把 `ENABLE_GPU=0` 写进它造的配置；H1、H11 的改动清单里没有它 |
| M1-6 | 没打中 | G12、G13 放到 H5、H6、H9 底下，没找到字面打架的一句 |
| M2-1 | 打中（模型上量过） | H8「CPU 判器版本 = 设点文件出发的闭包 ∪ CPU 判器闭包」：设点文件都 `use …crash_amplification::CrashPointRecorder`，闭包把整份引擎、GPU 派活 `crash_judge_dispatch.rs`、`gpu_unit_checks.rs`、`verdict_store.rs` 都收进来，与同一条 H8「引擎不整份进版本」相反 |
| M2-2 | 打中（推的，读码 + 现成用例） | H3「B、C 挪进公共模块、共享到 C」做不到：计划哈希里有「被判的根槽写在这个点的哪里」，多记录流的被判根槽写就在 C 里、删除流的在最后一批，C 这个点两项的块键不同；挪完共享约 2.852 亿态，不是 13.59 亿，去重约 48.65 亿，不是 37.9 亿 |
| M2-3 | 打中（定义缺口） | H4 只登记了节点上挂哪几个观察者，没定观察者读什么：删除流的挂载观察者读整条删除流的 `Prepared`（删除那几版各还差几步），单跑多记录流照 H4 跑它就得先把删除流整条录完；观察者结果只按块键存，读块键之外的流程输入时两项共用一份结果没有保证 |
| M2-4 | 打中（定义缺口） | H6「失败点」两边没有同一套编码：CPU 是带载荷的 `RecoveryFailure` 枚举，GPU 第 5、7 字是内核里另编的 u32（1、2、3、4、100+拒绝码…），今天没有一处把前者换成后者；比到什么粒度没定 |
| M2-5 | 打中（定义缺口，推的） | H8「GPU 判器版本带驱动版本」与「编译时算进二进制、跑时再算、不等退 78」：驱动版本编译时拿不到；两台驱动不同时一趟整域的块分在两把版本下，H9「判完才算判定」按哪把版本算没写 |
| M2-6 | 打中（小，推的无害） | 闭包按 `mod` 与路径收：公共模块里的 `use super::common::…` 按测试目标的规矩解到 `tests/common_crash_points/common.rs`（不在），B、C 挪进公共模块后这两点的版本不含 harness 的 `tests/common/mod.rs`；它改的东西多半也改写表、进块键，所以推的是无害 |
| M2-7 | 打中（小） | H5「甲二第三批之前登记临时豁免」、H7「第三批之前……报本次未核」写的是新条款下的提交，H12 又说新条款第四批之后才生效：这两句在提交上没有可用的时段 |
| M3 | 给量法 | 报给用户的数逐个分成「一条命令能重算」「只能量」；给最小量法与四种跑法的挂钟算式（见 M3 节与末节），不给推的挂钟 |
| M4 | 列出 15 条 | 会让实现员两边都能做的岔路逐条列，写明挡哪一批 |

## M1　前两轮站住的形态放到 H5、H6、H9 底下

### M1-1　「轻型由实现员跑」：今天的闸与定义一律拒，H 里没有一批去改它们

**许可它的那一句**：H9「轻型由实现员与合入后验证跑，每次都跑它的全部」，同一条「连同小输入上的 GPU 与 CPU 对拍」；H5「配了 GPU 时能走 GPU 的计算必须走」。

**今天挡着它的三处**：
- 重型测试闸：「子 agent 一律不跑；主 agent 带 =commit 或 =user-request 都能跑。」（`.claude/hooks/lib_heavy_tests.py:304`，说的是崩溃放量那几份脚本）；checker 档包的任何测试同样按包判。
- 实现员定义：「不由实现员跑」（`.claude/agents/implementation-writer.md:36`）。
- 共用约束：「子 agent 一律不跑，只跑 harness 档」（`.claude/agent-common.md:52`）。D13 已定项 15 也写着 checker 档「默认只在提交时跑，能单独跑」（`.claude/kb/decisions/13-验证路线.md:395`）。

H2 只给闸加「新入口起整域，子 agent 被拒」一格，没说 `--quick` 放行子 agent；H12 要改的是 D13 已定项 9、15、`verification.md`、`implementation-workflow.md`、`main-agent.md`、`commit-stages.md`，没有 `agent-common.md`、`implementation-writer.md` 与闸的 `classify`；H1 碰 `lib_heavy_tests.py` 只为改名。

**造出来的历史**：第四批合入、新条款生效之后，主 agent 派实现员改 `recovery.rs`。实现员照 H9 起 `<新入口> --quick`：闸按包与入口名认成 checker 档，子 agent 一律拒；照实现员定义它本来也不该跑。轻型「每次都跑它的全部」在这一步做不到，第一次跑到轻型的是提交时的主 agent。轻型里还有 GPU 对拍：照 H5 要走 GPU，照 `verification.md`「凡用 GPU 先清场」要经 `scripts/crash/gpu-quiesce-run.sh`，它「跑完、跑红或被打断都不复原」（同文件第 4 行），卡上有别的计算进程时「照拒（退 78），出路是等那个进程跑完再起。」（`scripts/crash/gpu-quiesce-run.sh:22`）——主 agent 在跑重型那几个小时里，实现员的轻型要么退 78、要么与重型同卡（读回命令是私有的，同卡的另一趟算不算「白名单外」我没法核，推的）。

**四句**：① 不分臂（只有一种形态）。② 不是观测问题，是形态里没有改闸的那一步。③ 满足 M1「与新定案字面打架」：H9 一句与 G2 / H2 的闸、与三份现行定义同时成立不了。④ 形态没给改法。我提的：甲，H14 第一批写明改 `lib_heavy_tests.py` 的 `classify`（子 agent 起新入口 `--quick` 放行、不带参数与 `--item` 照拒）、`agent-common.md` 那一句与实现员定义那一行，并写明子 agent 起轻型的 GPU 那一截怎么与在跑的重型分卡；乙，轻型里要 GPU 的那几样（小输入对拍）不交实现员、由合入后验证跑，实现员只跑轻型的 CPU 部分——乙与 H5「能走 GPU 的必须走」有张力，要用户点头。只在读码上成立，被攻过零轮。

### M1-2　H5 的豁免负表与今天选路认的私有正表

**许可它的那一句**：H5「配了 GPU 时能走 GPU 的计算必须走，不走的一截一行登记豁免表……门禁核两边逐项对上（C626（走 GPU 的计算类没有登记））」。

**今天的选路**：「照配置定每一类计算在哪算：没开 GPU（`ENABLE_GPU=0`）时登记不读，三类都在 CPU；开了 GPU，登记了的类只走 GPU、没登记的只走 CPU。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1030`），登记在私有显卡配置的 `GPU_COMPUTATION_CLASSES`，模板写「登记了的那几类只走 GPU，没登记的只走 CPU；」（`gpu-cards.env.example:20`）；SOP 原句「登记了的那几类只走加速路径」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:79`）。C626 那一行自己的改法也是「一格门禁：代码里走 GPU 的计算类与登记逐个对上」（`.claude/kb/checks-owed.md:486`）。

**造出来的历史**：用户机器上的私有显卡配置写 `GPU_COMPUTATION_CLASSES='gpu_unit_checks crash_verify_gpu'`（漏了 `crash_judge_gpu`），多机配置没写 `ENABLE_GPU`（H11：算开）。第 ① 段的判器照 `computation_routes` 走 CPU，每个状态的镜像由 CPU 判；H5 的豁免表里没有「判镜像」这一截，仓内门禁「两边逐项对上」读的是仓里的代码与仓里的豁免表，私有配置不在它眼里，判绿。H6 的对照这时判的是 CPU 对 CPU（GPU 那一遍不存在），H5「必须走 GPU」被一个仓外的键静默绕开。

**四句**：① 不分臂。② 门禁那一刻看不到私有配置（它在仓外、各机不同），属「判别子观测不到」。③ G5「能走 GPU 的必须走 GPU」那一分句，在 H5 底下被私有正表架空；C626 的改法与 H5 的改法是两张方向相反的表。④ 改法：甲，H5 写明删掉 `GPU_COMPUTATION_CLASSES`（选路只由 `ENABLE_GPU` 与仓内豁免表定），同批改 `crash_amplification.rs` 选路、`crash-amplification-lib.sh` 读法、模板与 SOP 那一句的本地口径；乙，正表留着，但开了 GPU 时正表必须等于「全部可上 GPU 的类」，少一类退 78。甲改动大、口径单一；乙留两张表。只在读码上成立，被攻过零轮。

### M1-3　提交那一趟在 worktree 里读哪一份多机配置

**许可它的那几句**：H11「多机配置合成一份……层 0 分片与放量双机两套驱动合成一套……提交那一趟整域照门禁 `--full` 在 HEAD + 暂存区的 worktree 里跑」；H5「配了双机就两台分片」。

**今天两套读法不同**：
- 放量：`repository_root` 是脚本所在那棵树的根（「repository_root="$(cd "$item_script_directory/../.." && pwd)"」，`scripts/crash/crash-amplification-item.sh:100`），配置取「configuration_file="${SINGLEFS_MULTI_HOST_CONFIG:-$repository_root/multi-host.env}"」（`scripts/crash/crash-amplification-lib.sh:127`；分项驱动第 549 行同形）。
- 层 0：按 git common-dir 取主工作树那一份，「configuration_file="$(dirname "$common_directory")/multi-host.env"」（`scripts/crash/layer0-shard-configuration-check.sh:88`），门禁文件头写明「<主工作树的根>/multi-host.env」（`.claude/gate.d/checker-tier-layer0-replay.sh:41`）。
- `multi-host.env` 不进 git，HEAD + 暂存区的 worktree 里没有它。

**造出来的历史**：用户配了双机（主工作树 `multi-host.env` 写 `ENABLE_ACROSS_MACHINES=1`），提交时照 H11 在 worktree 里起合成后的整域。合成的驱动若沿用放量那一套读法：worktree 根上没有配置文件，双机开关读成没写（今天的判法：没写算关），这一趟单机跑；`ENABLE_GPU` 照 H11 没写算开，还要卡。用户第二句「配置了分片就分片跑」在提交那一趟不成立，而且没有一行报错。沿用层 0 那一套则照常双机。H11 只说「合成一套」，没说取哪种找法。

同一组句子的另一面（这一格的附带，不攻用户定案本身）：一台没装卡、也没建多机配置的机器，照 H11「没写 ENABLE_GPU 算开」是「配了 GPU」，照 H5「卡用不上退 78」整道退 78；「没配 GPU 走 CPU」只在显式写 `ENABLE_GPU=0` 时才走得到。这是 H5、H11 对用户两句原话的合读，归辩方 M6 判读得对不对；这里只记它在实现上是同一个找配置的函数决定的。

**四句**：① 不分臂。② 驱动在 worktree 里看得到主工作树（git common-dir 取得到），观测得到；是形态没写取哪一种。③ G11「双机照今天放量与层 0 分片合成一套」那一分句：两套「今天」的找法不同，合成时必须二选一。④ 改法：H11 写明合成后一律按 git common-dir 找主工作树那一份（层 0 那种），`SINGLEFS_MULTI_HOST_CONFIG` 照旧优先；第二台的树副本照今天读环境变量。被攻过零轮。

### M1-4　「全量」撞车：SOP 的「不带参数」与 H9 的「清空复用」

**许可它的那几句**：H2「不带参数 = 快档全部项加整域全部项」；H9 重型「有三种跑法可选：剪枝（只按键与版本相同复用）、逐条……、全量（清空复用）」。SOP 入口表「| 不带 | 全量：快档与慢档的全部项 |」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:21`）。

**造出来的历史**：主 agent 提交时照 SOP 不带参数起入口（「全量」）。入口照 H2 跑快档全部加整域全部；整域那一截是剪枝还是清空复用？照 SOP 字面它是「全量」，照 H9 字面「全量」就是清空复用——整域每一块重判一遍；照 H9 的「剪枝」才复用。两种读法挂钟差的就是整域一趟（今天的树没量，M3）。

**四句**：① 不分臂。② 入口看得到自己的参数，是名字没定。③ M1「与新定案字面打架」：「全量」在 SOP 里是范围、在 H9 里是复用政策。④ 改法：H9 第三种改叫「清空复用」或「不复用」，「全量」只留 SOP 的范围义；H2 写明不带参数时整域按剪枝跑、清空复用要显式参数。被攻过零轮。

### M1-5　E171 编排脚本照旧按「没写算 0」转写配置（小）

H11「没写 `ENABLE_GPU` 算开」改的是同一份多机配置；今天的模板与判法都写「没写或写 0 一律算关」（`multi-host.env.example:26`）。E171 编排脚本给第二台造配置时：「keys.update({"ENABLE_ACROSS_MACHINES": "1", "ENABLE_GPU": values.get("ENABLE_GPU", "0")})」（`scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py:839`）。主工作树配置没写 `ENABLE_GPU` 时，本机照 H11 开 GPU，E171 那一趟显式写 0 关掉。H1 的改名清单与 H11 都没点到它。四句：① 不分臂；② 看得到；③ G11 / H11「多机配置合成一份」——同一份配置两种默认；④ 改法：H11 的改动清单加上所有读 `ENABLE_GPU` 的地方（`grep -rln ENABLE_GPU` 今天 12 份，含两份 E171 编排脚本与三份 checker 档测试）。被攻过零轮。

### M1-6　没打中：G12、G13

G12（条款随改）放到 H6、H9 底下：H12 已经把「全量指两样」改成一样、层表时机归 H9，没找到与 H5、H6、H9 字面相反的一句（试过：H6「对照默认跑」对「提交前 = 快速档全绿」（`.claude/kb/decisions/13-验证路线.md:251`）——H12 第四批之后才生效，在那之前照今天的条款，不打架）。G13（旧数作废）与 H5、H6、H9 无交集。这两条是读码与读文的「没打中」，只抽了一次。

## M2　第二轮的修补本身

### M2-1　H8 的 CPU 判器版本把整份引擎带回来了（模型上量过）

**许可它的那两句**：H8「CPU 结论（块键，CPU 判器版本 = 设这个点的文件出发的闭包 ∪ CPU 判器闭包）」；同一条「引擎不整份进版本：改结论的几处……搬进判器一侧的文件，其余留在流水线」。

**为什么相反**：每份设点的测试文件都要录点，都引录制器：公共模块是「use singlefs_checker_tier::crash_amplification::CrashPointRecorder;」（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:11`），别的设点文件同形。闭包「顺着路径与 `use` 引用收文件」，一引就收进整份 `crash_amplification.rs`，再顺着它收进它引的每一份。GPU 判器文件头写派活那一份「改它不换判法版本；这一份不许引它。」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:20`），不许引的表在「pub const FILES_THE_JUDGES_MUST_NOT_REACH: &[(&str, &str)] = &[」（`crates/singlefs-checker-tier/src/crash_identity.rs:861`）——它们经设点文件全回到了 CPU 判器版本里。

**模型原样**（`rerun.sh`，抽出的就是 H8 说的「同一套闭包」；带不带判器的停止表各算一遍）：

```
MODEL judges files=53 stopped=3 unstopped_files=56 unstopped_has_layer0_progress=true
MODEL setting=common_crash_points closure=no_stop files=65 union_with_judges=65 not_in_judges=12 crash_amplification.rs=1 crash_judge_dispatch.rs=1 gpu_unit_checks.rs=1 verdict_store.rs=1 crash_identity.rs=1 layer0_progress.rs=1 crash_injection.rs=0 bad_disk_input.rs=0 harness_tests_common_mod.rs=0 history.rs=0
MODEL setting=common_crash_points closure=judge_stops files=62 union_with_judges=62 not_in_judges=9 crash_amplification.rs=1 crash_judge_dispatch.rs=1 gpu_unit_checks.rs=1 verdict_store.rs=1 crash_identity.rs=1 layer0_progress.rs=0 crash_injection.rs=0 bad_disk_input.rs=0 harness_tests_common_mod.rs=0 history.rs=0
MODEL setting=crash_enumeration_unlink_after_the_multi_record_publish_stream closure=no_stop files=69 union_with_judges=69 not_in_judges=16 crash_amplification.rs=1 crash_judge_dispatch.rs=1 gpu_unit_checks.rs=1 verdict_store.rs=1 crash_identity.rs=1 layer0_progress.rs=1 crash_injection.rs=0 bad_disk_input.rs=0 harness_tests_common_mod.rs=1 history.rs=0
MODEL_SELFCHECK compared=20 mismatched=0 []
```

七份设点文件（`grep -l 'recorder.step(\|record_span(\|one_crash_point_over_every_write('` 在 `tests/` 下的七份）十四行全是 `crash_amplification.rs=1 crash_judge_dispatch.rs=1`，带不带停止表都一样；公共模块那一份闭包比判器闭包多 12 份，多的是 `crash_amplification.rs`、`crash_identity.rs`、`crash_judge_dispatch.rs`、`crash_verify_gpu.rs`、`gpu_card_configuration.rs`、`gpu_unit_checks.rs`、`layer0_progress.rs`、两份 `lib.rs`、`verdict_store.rs`、公共模块自己、`singlefs-harness/src/gpu_memory_ledger.rs`。

**造出来的历史（推的那一半是挂钟）**：第四批之后某次提交只改 `crash_judge_dispatch.rs` 里一次派活带几个状态（判法闭包外、GPU 结论版本不变）。照 H8 字面，每个设点文件的闭包都变，每个节点的 CPU 判器版本都变，「哪一种没有就只排那一种」——整域每一块的 CPU 结论重排、重判，H6 对照默认跑、H9「对照……都判完才算判定」，这次提交要等整域 CPU 重判完。CPU 结论逐位与改前相同（判器一个字没变）。历史上这种「只动引擎」的提交多不多：`git log --since=2026-09-28` 里碰这 9 份引擎文件的 4 次提交都同时碰了判器闭包（数法：对两份清单各取提交号、`comm -23`，得 0），提交是成批的；工作区里这 9 份对 HEAD 有「6 files changed, 15937 insertions(+), 704 deletions(-)」。所以代价是形态上的（每次只改派活就整域 CPU 重判），不是历史上已经发生过的次数。

**四句**：① 不分臂（H8 一种形态，打的是它两句之间）。② 系统在算版本那一刻看得到闭包里有什么——打中的是定义，不是观测。③ M2「H8 CPU 判器版本按设点文件取……」那一问，与 H8 自己「引擎不整份进版本」那一分句。④ 改法：甲，CPU 判器版本里设点文件那一项改成「设点文件自身的词法摘要 + 它 `mod`、`#[path]` 进来的测试模块」，不顺着 `use` 进 `src/`（库里的部分只经判器闭包进）；乙，设点文件的闭包加停止表，停在 `crash_amplification.rs`、`crash_identity.rs`、`verdict_store.rs`、`crash_judge_dispatch.rs`、`gpu_unit_checks.rs`、`layer0_progress.rs`、`gpu_card_configuration.rs`、`crash_verify_gpu.rs` 门外。两者都要再回答「设点文件进版本到底护的是什么」：判器另读的流程输入今天已按数据进计划哈希（`crash_amplification.rs` 第 417 行起的 `judge_inputs`），设点文件的代码改了而数据没变时 CPU 结论不变，护的那一截可能是空的——空的话最干净的是删掉这一项。只在我的模型上量过、被攻过零轮。

### M2-2　B、C 挪进公共模块，C 照样不共享（推的：读码 + 现成用例 + records 一行）

**许可它的那一句**：H3「多记录流的 B、C 两步挪进 `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` 设点，多记录流与删除流共享到 C」；用户定的是「挪进公共模块共享」，这一格不攻要不要挪，攻挪完共享不共享得到 C。

**块键里有什么**：块键带计划哈希（「enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),」，`crates/singlefs-checker-tier/src/crash_amplification.rs:253`），计划哈希接判器另读的输入（「"judge_inputs={}\n",」，同文件第 417 行），判器另读的输入里写着被判的根槽写落在这个点之前、之内还是之后：「"inside_the_point_at_write_{}",」（同文件第 714 行）、「"never_persisted".to_string()」（同文件第 718 行），整行进摘要（「"judged_root_slot_write={judged_root_slot_write}\n"」，同文件第 721 行）。被判的根槽写是整条流最后一条根槽写，两条流各取各的：多记录流「.rposition(|write| write.kind == StepKind::RootRecordFua)」（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:451`），删除流同一句（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1126`）。现成用例钉住了「被判的根槽写换位置，判器另读的输入跟着变」：「fn moving_the_judged_root_slot_write_changes_the_judge_inputs_but_not_the_tree() {」（`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:841`）。

**造出来的历史**：照 H3 把两份文件里 B、C 的 `recorder.step(file!(), "sequential_write", …)` 挪进公共模块，两边的内容、参数逐字相同（两份文件第 434–441 行与第 1100–1106 行同形）。
- 多记录流的流到 C 为止，被判的根槽写是 C 的那一条，落在 C 这个点里：`inside_the_point_at_write_k`。
- 删除流的被判根槽写是最后一批的，在 C 之后：`never_persisted`。
- 于是 C 这个点两项的计划哈希不同、块键不同，库里各判一遍；A、B 两点两边都是 `never_persisted`，版本表按上界截到 B（意图那一版 txg 更大、截掉），照常共享。
- 核对今天的数：records 里 2026-10-08 那一趟删除流先跑（「CRASH_AMPLIFICATION_ITEM item=unlink-after-the-multi-record-publish exit=0 states=1448083638 reused=0」），多记录流后跑（「CRASH_AMPLIFICATION_ITEM item=multi-record-publish exit=0 states=1342177375 reused=16777259」，两句都在 `records/2026-10-01-删除与并发事务.md:1292`）：今天只共享取号暖机与 A，与这一读法一致（B、C 今天各在自己的文件里设点，挪不挪都分不出这一格；分得出的是挪完之后的那一趟）。

**数（推的，算术）**：探针段形 `PROBE_SEGMENT_SIZES`（`scripts/crash/crash-amplification.sh:42`）前 19 段两项相同，两态闭式 1 358 954 604（`python3` 一行算出，与第二轮 `records_assumption` 的共享量同数）。C 那个点管的单元段是 30 写那一段，2^30 − 1 = 1 073 741 823 态。挪完的共享量 ∈ [16 777 259 + (2^28 − 1), 1 358 954 604 − (2^30 − 1)] = [285 212 714, 285 212 781]（下界只加 B 的 28 写段，上界只扣 C 的 30 写段，C 那个点另管的几段小段两端各差几十态）。照第二轮模型的今天去重 5 133 787 951 换算，挪完去重 4 865 352 429 – 4 865 352 496，不是第二轮判决交用户表里的 3 791 610 606。

**四句**：① 不分臂（H3 一种形态）。② 系统排计划那一刻看得到两条流各自的被判根槽写——判别子观测得到；打中的是块键把「这一项最后一条根」写进了共享节点。③ M2「H3……删除流与多记录流在 C 之前的节点是不是同一把版本」：C 之前（A、B）是；C 本身不是，H3「共享到 C」那一分句不成立。④ 改法：甲，被判根槽写不进判器另读的输入：版本表形态下它只喂计数 `root_persisted_states`（`crash.rs` 判一张镜像那一段，红项不读它；引擎那一段注释自己写着「不进逐项判定；照样进键」（`crates/singlefs-checker-tier/src/crash_amplification.rs:497`）），计数改成各项按自己的被判根从块的持久集合事后数，块里不存；乙，接受 C 不共享，交用户表把共享量与去重数改成上面那组；丙，删除流的被判根取「与多记录流共用的最后一条」——改了删除流判的是什么，不推荐。甲动引擎与 H7 计数向量（第二批），乙只改数。只在读码与算术上成立、没跑，被攻过零轮；判法见末节第 2 条。

### M2-3　H4 登记了「挂哪几个」，没登记「拿什么跑」

**许可它的那几句**：H4「哪个节点挂哪几个观察者由登记表静态列出……单跑一项照表跑它经过的每个节点上登记的全部观察者……观察者结果按（块键，观察者版本）存」；H9「点名单跑一项与它在整域里跑判定相同」。

**今天的观察者读什么**：删除流那三个逐状态观察者都吃整条流的 `Prepared`。挂载观察者「fn observe_the_mount_continuation(」（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:545`）按「fn landed_version<'prepared>(」（同文件第 521 行）定续做还差几步，后者查「.deletion_publishes」（同文件第 527 行）——意图、两批、最后一批各自还差几步，是录完整条删除流才有的表。

**造出来的历史**：H3 落地之后，C（以及照 M2-2 实际只有 A、B）是两项共享的标准前缀，用户定挂载观察者挂在标准前缀上、不得例外，H4 照表把它登记在这些节点上。
- 单跑 `--item multi-record-publish`：照 H4 要在 A、B 的每个状态上跑删除流的挂载观察者。观察者要 `Prepared`，`Prepared` 只能由录删除流整条（意图、两批、最后一批）得来——单跑多记录流就得把删除流也录一遍（第二轮 L1-3 报过的那一形，H4 只修了「挂哪几个」）。
- 存法：观察者结果只按（块键，观察者版本）存。块键管的是这一步的写表、起步镜像与判器另读的输入，不含观察者另读的流程输入。今天这一对流在 A、B 上读到的 `deletion_publishes` 只影响 txg 比 B 大的版本，答案碰巧相同；下一条接在 B 后面、带另一张续做表的流挂同一个观察者，就读回前一条流的结果。
**四句**：① 不分臂。② 观察者跑的那一刻看得到自己读了什么，但存库的键里没有它——对复用那一侧是「判别子观测不到」。③ H4「观察者结果按（块键，观察者版本）存」与 H9「单跑与整域判定相同」两分句。④ 改法：甲，H4 写明观察者只许读块键定得了的东西（起步镜像、写表、计划哈希里的判器另读输入），续做步数改成从恢复落到的根与镜像现算；乙，观察者另读的流程输入取摘要并进观察者结果的键（与判器另读输入同一个做法），单跑一项时由登记表点名「这个观察者要录哪条流」、照录。被攻过零轮。

### M2-4　H6 比「失败点」，两边没有一套共同的编码

**许可它的那一句**：H6「逐状态比红项与两遍各自的落到的根（实例代号、checkpoint_txg 低 32 位）、结局、失败点共八个字」。

**今天**：CPU 那一边换成比较字的函数只给四个字、不含失败点（「pub fn recovery_words_of(judgement: &crate::crash::Layer0StateJudgement) -> [u32; 4] {」，`crates/singlefs-checker-tier/src/crash_amplification.rs:1788`）；CPU 的失败是带载荷的枚举（「pub enum RecoveryFailure {」，`crates/singlefs-core/src/recovery.rs:185`，成员带盘号、槽号、`&'static str` 说明）；GPU 写的是内核里另编的数（「verdicts[out + 5u] = consulted.failure;」，`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:644`；赋值有 1、2、3、4 与 `100u + … .refusal` 几类）。`grep -rn 'words\[5\]\|words\[7\]'` 在 checker 档 `src/` 与 GPU 对拍测试里 0 处：今天没有一处把 CPU 的枚举换成 GPU 那一套数。

**造出来的历史**：一个状态两边都走读失败，CPU 是 `UnitMalformed { what: "某字段" }`，GPU 写 3。第三批照 H6 比八个字：「失败点」要比，就得有一张 CPU 枚举 → u32 的对照表；表写成多细（`what` 的每个串一个码，还是一类一个码）决定了哪些 GPU 判错比得出来。H6 没写这张表在哪、谁定、粒度多细；按最粗的读法（只比结局类别）它就退回了第二轮的四个字。

**四句**：① 不分臂。② 两边各自看得到自己的失败点，缺的是共同编码。③ H6「失败点」一分句。④ 改法：第三批之前定一张失败点码表（放 `crash_judge_gpu.rs` 判器一侧，与红项编码同处，H8 已把改结论的几处搬进判器一侧），CPU 侧加 `failure_code_of(&RecoveryFailure) -> u32`、GPU 照它写，码表改动换两边的判器版本；或者 H6 写明失败点只比到类别。用户已定把「WGSL 与 Rust 怎么保持一致」挪到 E171 再谈——这张码表算不算那件事，要先定（M4 第 8 条）。被攻过零轮。

### M2-5　GPU 判器版本带驱动：两台不同驱动时「判完」按哪把版本

**许可它的那几句**：H8「GPU 结论（块键，GPU 判器版本 = GPU 判器闭包 + 驱动与 wgpu 版本）」「版本在编译时算进二进制（`build.rs` 照同一套闭包算），跑时从盘上再算一遍，不等就整趟不写库、退 78」；H5「配了双机就两台分片」；H9「提交前判镜像、观察者、对照……都判完才算判定」。

**造出来的历史（推的，第二台的驱动版本我没去查）**：本机与第二台各一套卡（开工时资源钩子报本机 RTX 5090、5060 Ti，第二台 RTX 5080 与两张 5060 Ti），驱动版本各装各的。① 驱动版本 `build.rs` 拿不到（编译机不一定有卡、编出的二进制可以在另一台跑），「编译时算进二进制 / 跑时再算 / 不等退 78」只能比闭包那一半；驱动那一半跑时才知道，H8 没写它怎么进比对。② 一趟整域两台按速度分块：本机判的块存在（块键，V本机），第二台的导回来存在（块键，V第二台）。下一趟剪枝，同一块这次分给另一台：（块键，V另一台）查不到，重判；整域两台分法每趟按量到的速度变，剪枝能复用多少取决于分法。③ 提交那一刻「GPU 结论都判完」——要每一块在「这一趟用到的哪一把」版本下有结论？两把都算？H8、H9 没写。另：第二轮判决 L3-4 写的是「带驱动版本与卡型」，H8 写「驱动与 wgpu 版本」，卡型没了；带卡型时一台机器上两张不同型号的卡一趟里就是两把版本，同一个问题更早出现。

**四句**：① 不分臂。② 每台在判的那一刻看得到自己的驱动；判「齐没齐」的那一刻要看所有台的——属定义缺口。③ H8 那两句与 H9「都判完」。④ 改法：甲，驱动不进版本键，改成每台开跑时核一次「驱动版本在登记的可用集合里」，不在就退 78；判器版本只认闭包（GPU 判器换了驱动就算的结果不同时，H6 对照抓得到）；乙，驱动进键，「判完」定义成「每块在至少一把登记过的驱动版本下有结论」，登记集合进多机配置。被攻过零轮。

### M2-6　公共模块里的 `use super::common::…` 闭包收不到（小，推的无害）

公共模块头一句「use super::common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};」（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:10`）。闭包对测试目标的规矩是「tests/ 下的测试目标自成一个 crate：模块路径从空起，子模块与 #[path] 按它所在的目录解。」（`crates/singlefs-checker-tier/src/crash_identity.rs:537`）：`super::common` 按公共模块自己的目录解成 `tests/common_crash_points/common.rs`，那里没有文件，落空。模型里公共模块那两行都是 `harness_tests_common_mod.rs=0`，别的设点文件是 1。B、C 挪进公共模块之后，这两点的 CPU 判器版本（H8）与挂在上面的观察者版本（H4，若观察者也写进公共模块）不含 harness 的 `tests/common/mod.rs`。推的无害：那份文件里 `parameters()`、`build_pool` 改了多半改写表与起步镜像，进块键；改了而写表不变的那种（例：只改一个断言消息）本来也不该换版本。记在这里是因为 M2-1 的改法甲要「设点文件 + 它 `mod` 进来的测试模块」，公共模块那一份按今天的解法收不到它的兄弟模块，改法甲要连这一处一起改。被攻过零轮。

### M2-7　「第三批之前」那两句在提交上没有可用的时段（小）

H5「甲二在第三批之前登记一行临时豁免、第三批删」与 H7「第三批之前、配了 GPU 而对照被授权跳过的那一趟，逐条判到 / 不适用两列报「本次未核」」写的是新入口、新条款下的跑法；H12「新条款在第四批合入之后的第一次提交起生效，第一至第四批的提交照今天的条款」。第三批之前的提交照今天的条款，不跑新入口的轻型与对照；这两句只剩「用户要求时手跑一趟」可用，形态没写那一趟算什么。四句：① 不分臂；② 看得到；③ H5、H7 那两分句与 H12 的生效时点；④ 改法：两句改成「第三批之前用户要求时跑的那一趟」，或删掉、在第三批写死。被攻过零轮。

## M3　数字能不能核：一条命令重算的、只能量的、怎么量

### 报给用户的数，逐个归类

| 数（出处：两轮判决第三节与第一轮弹窗） | 今天怎么得 | 归类 |
|---|---|---|
| 各项整域态数（例：删除流层 0 两态闭式 1 463 812 228、三态 1 463 812 278） | 测试里钉死的字面量，`grep -n 'assert_eq!(closed_form' <测试文件>` 读出；放量各项按展开 30 的态数要录流（每项录一次） | 一条命令（层 0 那 13 条）；放量 8 项要录流、不判（第 1 条命令） |
| 共享 16 777 259（取号暖机 + A） | records 一行，今天的树可重测 | 只能量（第 2 条命令，抽样） |
| 去重 37.7 亿 / 37.9 亿（第一轮）、51.3 亿与挪后 37.9 亿（第二轮） | 第二轮模型的算术，输入是 records 的段形与共享假设；挪后那一个照 M2-2 已不成立 | 只能量（第 2 条命令） |
| 1 358 954 604（前 19 段两态闭式） | `python3` 一行从 `PROBE_SEGMENT_SIZES` 算出，与第二轮同数 | 一条命令，但它是探针段形，今天的删除流第一段钉 24 写、探针是 20 写（第一轮 K2-3），换今天的段形要录流 |
| GPU 单判 1.5–15 小时、整域 GPU 串跑 9.8–10.8 小时、下界 4.9–7.2 小时 | 2026-09-30、10-08 的树上的速率乘态数 | 只能量（第 2 条命令） |
| CPU 每态判 21.6–124.4 小时、CPU 跟上 73–220 小时 | 同上，速率随流与位置差 5.8 倍（第二轮 L2-2） | 只能量（第 2 条命令的 CPU 那一趟） |
| 删除流逐态挂载约 50.8 小时 | 第二轮模型，挂载速率出处不明 | 只能量（第 3 条命令） |
| 改观察者 GPU 白重判 1.30–2.40 小时、按项版本多判 1 426 456 802 态 | 第二轮模型；H8 改了版本形态之后这两格的前提已换（M2-1、M2-2） | 作废，不必重量 |
| 92.8% | 第一轮已撤 | 作废 |

### 最小量法（只能量的那几格合成一套）

要点：放量的块按「平滑加权轮转」分给各方（「pub fn owner_of_block(&self, block_sequence: u64) -> usize {」，`crates/singlefs-checker-tier/src/crash_amplification.rs:906`），`SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/<N>` 领的是每 N 块里的一块，一项从头到尾均匀抽到，不是只抽开头那一截（第二轮 L2-2 打中的就是「开头一分钟的短跑」）。所以一趟 `0/N` 就是这一项全部位置上的 1/N 等距样本；同一份库里各项的共享前缀块号相同（前缀点在每项计划的最前面），后跑的项在样本里照样复用到前面那一项样本里的共享块。

由样本推全量（只是换算式，数等主 agent 量）：
- 第 i 项整域态数 S_i ≈ N ×（这一项样本行的 states + reused），误差不超过 N × 65 536（`STATES_PER_BLOCK`）；
- 去重 D ≈ N × Σ states（各项按登记次序进同一份新库）；
- 共享量 = N × 后跑那一项的 reused；
- 判的速率 R = Σ states / Σ 第 ① 段秒数（`CRASH_AMPLIFICATION_PHASES` 行的 `seconds_judging`，GPU 那一趟另有 `CRASH_AMPLIFICATION_GPU_TOTAL … states_per_second`）；
- 每项固定开销 F_i = `seconds_planning_and_recording`（录流，不随 N 变）。

四种跑法的挂钟算式（单位秒；G、C 下标是 GPU、CPU；「改了哪把版本」照 H8）：
- 轻型：直接量，不推。新入口还没有，今天最近的代理是 checker 档快档整趟（主 agent 正在跑的 `cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --lib --tests`），读它的总挂钟；轻型落地后在第二批末量一次新入口 `--quick` 的总挂钟替换它。
- 重型·全量（清空复用）：Σ F_i + D / R_G，对照不跳过时再加 D / R_C（H6 没写 GPU 与 CPU 那一遍并行还是先后：并行取两者大的那个，先后取和——这一句要先定，M4 第 9 条），再加删除流逐态挂载 S_删除 / R_挂载。
- 重型·剪枝：Σ F_i + D_换G / R_G + D_换C / R_C。改了 GPU 判器：D_换G = D；只改 CPU 判器闭包里的文件：D_换C = D；照 H8 字面只改引擎（M2-1）：D_换C 也 = D。
- 重型·逐条（点名 k 项）：Σ_{i∈k} F_i + S_k 去重 / R，库里有共享前缀时减去复用的那一截。

### 每条命令读哪一行、什么读数推翻哪一句

| 命令（原样在末节） | 读哪一行 | 读数 ⇒ 推翻什么 |
|---|---|---|
| 1 只录不判 | 每项 `CRASH_AMPLIFICATION_PLACEMENT … pending_states=` | 与第 2 条换算出的 S_i 差过 N × 65 536 ⇒ 抽样换算式不能用，第 2 条那一趟作废 |
| 2 抽样整域（GPU 一趟、CPU 一趟） | 每项 `CRASH_AMPLIFICATION_ITEM … states= reused= seconds=` 与 `CRASH_AMPLIFICATION_PHASES` | 多记录流那一项 reused × N 不在 16 777 259 ± N × 65 536 之内 ⇒ 第二轮「今天只共享取号暖机与 A」不成立；各项 R_C 最大与最小之比远小于 5.8 ⇒ 第二轮 L2-2「随位置差 5.8 倍」不成立 |
| 2b 同上，在 B、C 挪进公共模块的仓副本里 | 多记录流那一项 `reused=` | reused × N 落在 [285 212 714, 285 212 781] ± N × 65 536 ⇒ M2-2 成立；接近 1 358 954 604 ⇒ M2-2 错，H3「共享到 C」成立 |
| 3 删除流层 0 全量限时 | 进度文件里判过的态数与挂钟 | 给出 R_挂载；第二轮 50.8 小时只是被替换，不是被推翻（它没有出处） |
| 4 闭包模型复跑 | `MODEL setting=… crash_judge_dispatch.rs=` | 任何一行是 0 ⇒ M2-1 错 |

## M4　未定项清单：实现员两边都能做的岔路

| # | 岔路（两种都能照 H 字面做） | 挡哪一批 | 来自 |
|---|---|---|---|
| 1 | 轻型由谁跑：闸放行子 agent 起新入口 `--quick`（改 `classify`、`agent-common.md`、实现员定义），还是轻型只由合入后验证（主 agent）跑 | 第一批（钩子、条款） | M1-1 |
| 2 | 轻型要卡时卡上正跑着重型：等、退 78、还是分卡（清场「不复原」对每天多次的轻型也照做吗） | 第一批 | M1-1 |
| 3 | 私有显卡配置的 `GPU_COMPUTATION_CLASSES` 正表：删掉（选路只看开关与仓内豁免表），还是留着并要求等于全集 | 第一批（脚本）、第二批（选路代码） | M1-2 |
| 4 | 合成后的驱动在 worktree 里按 git common-dir 找主工作树的多机配置，还是按树根找（今天两套各一种） | 第一批 | M1-3 |
| 5 | 「全量」只留范围义、第三种跑法另起名；不带参数时整域按剪枝还是清空复用 | 第一批（入口、门禁） | M1-4 |
| 6 | CPU 判器版本里「设点文件出发的闭包」取到哪：只取设点文件与它 `mod` 进来的测试模块、加停止表、还是整项删掉 | 第二批 | M2-1 |
| 7 | 被判根槽写出不出计划哈希：出（C 共享，计数事后数）、不出（C 不共享，交用户的代价数改成 M2-2 那组） | 第二批；同时改交用户表 | M2-2 |
| 8 | 失败点码表：放在 H6（第三批定、两边同改），还是算「WGSL 与 Rust 怎么保持一致」挪到 E171；比到类别还是比到成员 | 第三批 | M2-4 |
| 9 | 对照那一遍与 GPU 那一遍：同一趟并行、还是 GPU 先判完再补 CPU；CPU 那一遍落在后面时，提交等它判完才算判定（H9 字面）还是允许「GPU 判完、CPU 在补」 | 第一批（入口的成功句）、第二批 | H6 × H9 |
| 10 | 观察者只许读块键定得了的东西，还是另读流程输入并进观察者结果的键；单跑一项要不要录别的项的流 | 第二批 | M2-3 |
| 11 | 驱动版本进不进 GPU 判器版本；两台驱动不同时「判完」按哪一把 | 第二批（版本）、第三批（GPU） | M2-5 |
| 12 | 提交时选逐条跑：点几项、谁点、点漏了改动碰到的项算不算「判定」（H9「没点到的逐项报本次未跑」与「都判完才算判定」两句之间） | 第一批（门禁的判定句）；要用户定 | H9 |
| 13 | 第一批改了门禁与入口、引擎（第二批）还没合：第一批合入后的提交里门禁快档跑今天的 `cargo test --release -p singlefs-checker-tier --lib --tests`，还是新入口 `--quick`（它要的甲二与给定状态表第二、三批才有） | 第一批 | H12 × H14 |
| 14 | 读 `ENABLE_GPU` 的别处（E171 两份编排脚本、三份 checker 档测试）随 H11 改默认，还是各留各的 | 第一批；E171 冻结前 | M1-5 |
| 15 | H10 按「段号 + 写数」登记：写表变了、段号挪位之后，登记的那一行对不上时引擎拒跑、按写数重找、还是照段号跳过 | 第二批 | H10 |

第 6、7、9 三条改的是交用户的代价数本身：第 7 条决定去重是约 37.9 亿还是约 48.65 亿，第 6 条决定「只改引擎的提交」要不要整域 CPU 重判，第 9 条决定对照那一截加在挂钟上还是与 GPU 那一截取大。

## 没打中的形状

| 试过的形状 | 取样范围 | 结论 |
|---|---|---|
| H6「GPU 判不了的个别状态记一条不一致、判红」与 H10「装不下的段记 GPU 判不了」：两处同一个去向，与 H5「不降级」一致 | 读 H5、H6、H10 三句与 `first_state_the_gpu_could_not_judge` | 没打中 |
| H8 `build.rs` 在 `cargo test` 增量编译下漏重算：闭包里新加一份文件，一定是某份已在闭包里的文件多了一句 `use` / `mod`，那份文件变了就会重跑 | 读闭包算法（模型那一段）；没实现 `build.rs` 去试 | 没打中（推的） |
| H8 worktree、另一个目录下闭包不同：身份与闭包只用仓内相对路径，现成用例 `judging_code_is_the_same_in_a_copy_of_the_repository_at_another_path` 钉着 | 读 `crash_identity.rs` 文件头与那条用例名 | 没打中 |
| H7 计数向量按块存、共享节点被几项复用时重复计：各项的断言按自己的计划把块的计数加起来，共享块对每项各计一次正是要的 | 读 H7 | 没打中 |
| H4 静态登记表与「单跑一项判定相同」在「挂哪几个」上打架 | 第二轮 L1-3 的造法重走 | H4 修住了这一半；没修的那一半是 M2-3 |
| G12、G13 放到 H5、H6、H9 底下 | 见 M1-6 | 没打中，只抽了一次 |

这些「没打中」都只抽了一次，照 `.claude/rules/three-way-inference.md` 不支撑「没问题」。

## 这条腿自己的限度

- 闭包模型是同一段代码的原样拷贝，不是独立实现：它证明的是「H8 说的那套闭包算法在今天的树上收进了什么」，不证明算法本身对；自证只核到它与 `crash_identity.rs` 自己那条测试的成员表一致。
- M2-2 没跑：结论来自读计划哈希的拼法、两份测试文件的被判根槽写取法、一条现成用例的名字与 records 一行；共享量区间是算术，C 那个点另管的小段没逐个数。末节第 2b 条是它的判法。
- M2-5 第二台的驱动版本没查（不登第二台）；资源钩子报的卡型是开工时那一刻的。
- M3 的抽样量法依赖两件我没核的事：直接起用例时 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/N` 一份单跑能走完（不在对账那一步因为别的份没判而停）；样本里共享块的块号在两项之间对得上。末节命令里写了先核哪一步。
- 私有配置（`multi-host.env`、显卡配置）没读，M1-2、M1-3 用的是模板与读法代码。
- 速率、挂钟一个都没量（checker 档与 GPU 都不归这条腿）。

## 没做什么

- 没跑任何 checker 档测试、门禁阶段、崩溃放量、GPU；没建仓副本；只编跑了一个零依赖的闭包模型（经内存包装，2G）。
- 没判 M5、M6（辩方腿）；M1-3 里提到的「没卡也没配置的机器退 78」只记事实、不判 H5 / H11 读用户原话读得对不对。
- 没替主 agent 采纳；模型上的数不算入库装置上的数。
- 草稿目录 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-opus/` 里有模型的编译目录 `closure-build/`、`closure-target/` 与输出 `closure-out/`、日志 `closure-run.log`、`rerun.log`、两份提交号清单；交回前删掉编译目录（见交回）。

## 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`；第 1–3 条是 checker 档或要卡，子 agent 不跑。`<库根>` 取一个新的空目录（例 `~/.local/share/singlefs/work/m3-r3-measure/`，不放开机会清的目录）；`<卡>` 取私有显卡配置里 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 那一串；`<bindgen>` 取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。不经入口 `crash-amplification.sh`，因为分项驱动起本机那一趟时传的是 `0/1`（`scripts/crash/crash-amplification-item.sh` 第 609 行），共用库第 174 行把它写进环境、盖掉调用方给的值，抽样要直接起用例。各项的测试目标与用例函数取 `bash scripts/crash/crash-amplification.sh --list-items` 的第 2、3 列（今天 8 项，次序就是登记次序）。

**第 0 步（先核两件事，几分钟）**：在第 2 条的循环里只跑第一项、`N=2000`，看用例退 0、`CRASH_AMPLIFICATION flow=` 那一行带 `unjudged_blocks>0`、`red=0`；若用例在对账那一步因别的份没判而 panic，这套抽样不能用，改成整项跑第 1 项（删除流）一次当速率样本。再跑第二项，看它的 `states_reused` 是否大于 0（共享块号在两项之间对得上）。用这两项的挂钟按项数与 N 估第 2 条全量的挂钟，再定 N。

**第 1 条　只录不判，拿每项整域态数**（每项约一次录流的时间；判法：每项一行 `CRASH_AMPLIFICATION_PLACEMENT … pending_states=`，与第 2 条换算出的 S_i 比）：

```
printf 'ENABLE_GPU=1\nENABLE_ACROSS_MACHINES=0\n' > <库根>/gpu-on.env
SINGLEFS_HEAVY_TESTS=user-request bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/run-with-memory-cap.sh 24G env SINGLEFS_MULTI_HOST_CONFIG=<库根>/gpu-on.env GPU_COMPUTATION_CLASSES='gpu_unit_checks crash_verify_gpu crash_judge_gpu' GPU_CARDS_BY_PRIORITY='<卡>' SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=<库根>/placement SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=30 SINGLEFS_CRASH_AMPLIFICATION_THREADS=31 BINDGEN_EXTRA_CLANG_ARGS='<bindgen>' cargo test --offline --release -p singlefs-checker-tier --features verdict-store,gpu --test <测试目标> -- --exact <用例函数> --nocapture
```

**第 2 条　整域等距抽样，GPU 一趟、CPU 一趟**（8 项按登记次序进同一份新库；判法见 M3 节「每条命令读哪一行」表第 2 行；读每项 `CRASH_AMPLIFICATION flow=… states_checked_here=… states_reused=…`、`CRASH_AMPLIFICATION_PHASES … seconds_planning_and_recording=… seconds_judging=…`，GPU 那一趟另读 `CRASH_AMPLIFICATION_GPU_TOTAL … states_per_second=`）：

```
N=<第 0 步定的数>
bash scripts/crash/crash-amplification.sh --list-items | cut -f2,3 > <库根>/items.tsv
# GPU 一趟（库 <库根>/gpu-$N）
while IFS=$'\t' read -r target case_name; do SINGLEFS_HEAVY_TESTS=user-request bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/run-with-memory-cap.sh 24G env SINGLEFS_MULTI_HOST_CONFIG=<库根>/gpu-on.env GPU_COMPUTATION_CLASSES='gpu_unit_checks crash_verify_gpu crash_judge_gpu' GPU_CARDS_BY_PRIORITY='<卡>' SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/$N SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=<库根>/gpu-$N SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=30 SINGLEFS_CRASH_AMPLIFICATION_THREADS=31 BINDGEN_EXTRA_CLANG_ARGS='<bindgen>' cargo test --offline --release -p singlefs-checker-tier --features verdict-store,gpu --test "$target" -- --exact "$case_name" --nocapture >> <库根>/gpu-$N.log 2>&1; echo "ITEM_RC $target $?" >> <库根>/gpu-$N.log; done < <库根>/items.tsv
# CPU 一趟（库 <库根>/cpu-$N；配置写 ENABLE_GPU=0，不经清场）
printf 'ENABLE_GPU=0\nENABLE_ACROSS_MACHINES=0\n' > <库根>/gpu-off.env
while IFS=$'\t' read -r target case_name; do SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 24G env SINGLEFS_MULTI_HOST_CONFIG=<库根>/gpu-off.env SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/$N SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=<库根>/cpu-$N SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=30 SINGLEFS_CRASH_AMPLIFICATION_THREADS=31 BINDGEN_EXTRA_CLANG_ARGS='<bindgen>' cargo test --offline --release -p singlefs-checker-tier --features verdict-store --test "$target" -- --exact "$case_name" --nocapture >> <库根>/cpu-$N.log 2>&1; echo "ITEM_RC $target $?" >> <库根>/cpu-$N.log; done < <库根>/items.tsv
grep -c '^ITEM_RC' <库根>/gpu-$N.log <库根>/cpu-$N.log   # 各要是 8；有一行 rc 不是 0 的那一项作废
grep -h 'CRASH_AMPLIFICATION flow=\|CRASH_AMPLIFICATION_PHASES\|CRASH_AMPLIFICATION_GPU_TOTAL' <库根>/gpu-$N.log <库根>/cpu-$N.log
```

换算照 M3 节「由样本推全量」那几式。另取 `SHARE=1/$N`、`2/$N` 各再跑一趟 CPU，三份 R_C 差得多就说明一份样本还不够、N 要放小。

**第 2b 条　判 M2-2（C 共享不共享）**：拷一份仓副本，在副本里把两份测试文件里 B、C 那两次 `recorder.step(file!(), "sequential_write", …)` 挪进 `tests/common_crash_points/mod.rs` 的一个函数（内容与参数照原样传进去）。在副本里起第 2 条的 CPU 循环，只跑 `unlink-after-the-multi-record-publish` 再跑 `multi-record-publish` 两项、展开 6（`SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6`、`SHARE=0/1`，小，几分钟到几十分钟，没量过）。判法：第一项那一趟的 `CRASH_AMPLIFICATION_BLOCK point=… states=…` 按点求和；第二项 `states_reused` 等于取号暖机、A、B 三个点之和 ⇒ M2-2 成立；等于再加 C 那个点 ⇒ M2-2 错、H3「共享到 C」成立。

**第 3 条　删除流逐态可写挂载的速率**：在 HEAD + 暂存区的 worktree 里照门禁的出路句起 `checker-tier-layer0-replay.sh --full`，只点删除流那一条（`--item crash-case:<删除流的登记名>`，名字取 `--list-items`）；跑 T 分钟后从最底层的进程停起（`proc.py stop`，逐个）。读续跑进度目录 `<git common-dir>/singlefs-layer0-progress/<输入指纹>/` 里判完的切片数与切片长（读法照 `crates/singlefs-checker-tier/src/layer0_progress.rs` 文件头），R_挂载 = 判完的态数 / 秒数。这条判的是每态「两遍恢复 + checker + 记录核对 + 一次可写挂载」的合计速率，挂载单独那一截要再减第 2 条 CPU 那一趟删除流的 R_C。

**第 4 条　复跑闭包模型（不是 checker 档，谁都能跑）**：`bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-crash-enumeration-merge-r3-opus-model/rerun.sh`。判法：末行 `MODEL_SELFCHECK compared=20 mismatched=0 []`；十四行 `MODEL setting=` 里任何一行 `crash_judge_dispatch.rs=0` ⇒ M2-1 错。`crash_identity.rs` 变了它退 3，要先按新行号改 `extract.sh` 的两个区间与钉的 sha256。

## 收工

- 开工快照收工再核：`sha256sum -c --quiet …/crates-sha256.txt` 退 0，`…/kb-sha256.txt` 退 0。
- 模型的编译目录（草稿目录下 `closure-build/`、`closure-target/`）已删；草稿目录里留下模型输出 `closure-out/`、两份日志与两份提交号清单，没有仓副本。
