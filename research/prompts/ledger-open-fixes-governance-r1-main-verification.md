# ledger-open-fixes-governance-r1：判决（2026-11-19）

正文 `research/prompts/_ledger-open-fixes-governance-r1-body.md`，开工快照 `research/prompts/ledger-open-fixes-governance-r1-snapshot/`。这是这一题的第一轮，之前没有判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（第 1 到 5 问） | `research/prompts/ledger-open-fixes-governance-r1-opus-output.md`（sha256 `f1af8ed3…`） | 第 1 问进口、缝一、新加一格打中；第 2 问 exact 行一路、标准错误打中；第 3 问多拒、漏拒打中；第 4 问退 78 打中；第 5 问钉值不等于登记第十二节打中 |
| 正推（第 6 到 10 问） | `research/prompts/ledger-open-fixes-governance-r1-sonnet-output.md`（sha256 `5fc4223a…`） | 第 6 问三句超出原话、同文件三处旧句、全仓九处旧说法；第 7 问成立；第 8 问射程不成立；第 9 问登记行「七十二条」不成立；第 10 问调查员定义不成立 |

核查命令（在主 agent 的草稿目录 `~/.local/share/singlefs/work/open-fixes-governance-r1-verify/` 下，日志在它的 `main-agent-logs/`）：

- 倒推快照：`python3 scripts/agents/snapshot-tree.py ledger-open-fixes-governance-r1 --out <草稿>/ledger-open-fixes-governance-r1-tree --repository <仓根>`，经任务服务交，日志 `v0-倒推快照.log` 末行「清单点名 30 份，倒推出 29 份、sha256 对上 29 份；倒推不出 1 份、对不上 0 份」。倒推不出的是 `.claude/gate.d/break-switch-replay-files.tsv`：快照之后别的会话改了它，diff 里没有它的基底。腿引它的只有第 10、90–92 行，主 agent 对着主工作区逐行核了（第 10 行仍写「七十二条开关」，第 90–92 行三份入口各「十二条开关」），这几行别的会话没碰。
- 引文：`python3 scripts/agents/cite-check.py <两份报告> --root <草稿>/ledger-open-fixes-governance-r1-tree --background research/prompts/_ledger-open-fixes-governance-r1-background.md`，末行「核了 118 处引文、111 处裸行号，对上 71 处，没判 158 处」，判错 0 处。判决要用的引文另外逐处核过：`.claude/main-agent.md` 第 14、24、27、38、100 行，`.claude/agents/investigator.md:33`，`.claude/agents/implementation-writer.md` 第 32、51 行，`research/prompts/e185-r4-prereg.md` 第 538、579 行。
- 攻方腿四条探针拷到主 agent 草稿目录的 `probe/` 下复跑（`evidence-mixed-replay-feed.sh` 的运行目录守卫改指主 agent 的草稿目录，起门禁那一行补了范围声明，判法没动）：
  - `v1-换上工具六臂.log`：`outcome-overlap-control … next_swap_pending_refusal=9`；`outcome-overlap-different-scratch a_killed=True b_exit=0 x1_landed=True x2_landed=False y_landed=True next_swap_pending_refusal=none read_outcome_exit=0`；`seal-outcome-while-swap-running pending_while_running=9 seal_exit=0 next_swap_pending_refusal=none read_outcome_exit=0`；`gap-three-contenders-on-stale-record holding=A,C a_record_still_on_disk=False`；`gap-link-reply-lost held=False refused=True refusal_names_own_pid=True record_left=True`；`same-machine-id-other-boot live_holder_counts_as_occupying=False`；`probe-done arms=6`。
  - `v2-交付物四臂.log`：`parent-directory-bad-line-blocks-child-handback action=block named=True`、`quoted-path-with-space-blocks action=block named=True`、`symlinked-declaration-with-bad-line action=pass named=False`、`nested-declaration-with-bad-line action=pass named=False`。
  - `v3-复跑表exact行.log`：E913 两行（`|timing`、`|exact`），红行里没有 `scripts/experiment-apparatus/e913_replay_timing.rs（装置源码，改于 2026-09-05）`；样本原本期望这一行（`.claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-red/expect:16`）。
  - `v4-增行报错.log`：`gate_changed_paths exit=1 header_line=1`、`gate_added_lines exit=1 header_line=0`。
- 正推腿两条开关复跑：`v5-完工清理闸开关.log`「复跑了 73 条开关（1 份），自证红且点名的格与实际红的格逐格相同 73 条」；`v6-三份入口开关.log`「复跑了 36 条开关（3 份）…逐格相同 36 条」。
- 主 agent 现查：`grep -rn 'static [A-Z_0-9]*: *OnceLock' crates/ --include=*.rs | wc -l` 得 38；E185 三份脚本钉的都是 `d312b9aa71d0…`，登记 `research/prompts/e185-r4-prereg.md:579` 写的是 `9e813cbd…`。

## 二、判决

坐实，这一轮改：

1. **换上工具 `--seal-outcome` 不占、不看开始行那一趟还在不在**（第 1 问，探针 `seal-outcome-while-swap-running`）：一趟真换还在跑时补写完成成功，下一趟不再拦。
2. **结局文件按仓共用，占用锁按草稿目录**（第 1 问新格，探针 `outcome-overlap-different-scratch` 对对照臂）：拒绝说明叫并行的各给一个草稿目录，两趟真换就在读结局文件与写开始行之间重叠；先写的那一趟换到一半被杀，下一趟不拦，`x2` 停在半截。文件头第 87 行「主工作区那一头仍有基线核对与结局文件拦着」不成立。
3. **缝一三趟可达**（探针 `gap-three-contenders-on-stale-record`）：甲、丙同时占着，甲的记录被乙删掉。
4. **缝二自己的进程号被当成别人**（探针 `gap-link-reply-lost`）：后果只是一次点名自己的假退 10，互斥不破；照「认出是自己的记录就当放成」一并改。
5. **evidence-in-repo 的 exact 行一路按实验号放**（第 2 问，探针 v3）：一个实验有一条 exact 行，它的 timing 部分改了也放过，与 `.claude/gate.d/doc-experiments.sh:821` 相反。
6. **`doc-process-records.sh:841` 说 `gate_added_lines` 失败时也写「哪一条 git、退几」，实现不写**（探针 v4）：改实现，让 `gate_added_lines` 同样写，不改那一句。
7. **`load_change_range` 在取到时吞掉命令的标准错误**（第 2 问表）：原来照打，改回照打。
8. **完工清理闸多拒两种**（第 3 问，探针 v2 前两臂）：上层目录里别人写坏的一行拦下层子 agent 的交回；带引号、带空格的路径照文件头写法也判写法不合。
9. **完工清理闸漏拒两种**（探针 v2 后两臂）：符号链接的 `.deliverables` 整份跳过；写在副本那一层的不读。前一种改成判写法不合并点名；后一种在文件头写明只读哪几层，不加读。
10. **三份复跑入口漏了内核镜像的准入**（第 4 问）：缺可读内核镜像时退 1 当作废，该退 78。
11. **三份复跑入口的删旧闸会删掉没进 git 的同名产物**（第 4 问，读代码，没复跑）：改成输出文件已存在而没进 git 时拒绝、给出路，进了 git 的照删。
12. **E53、E159 两处说明与实现不一**（第 4 问）：E53 自证喂了参数、`VM_DISKS` 不核；E159 用法行写要子命令而实现不核。
13. **E185 三份钉值重钉成现算值，把停机条款 K-S6、J-S7 变成了照跑**（第 5 问，现查）：装置在登记之后改过，登记第十二节没有修订行；照起跑脚本自己的出路，改钉值要连登记一起改，不该单改脚本。撤回这一次重钉，实验页的说法一起改回。
14. **`.claude/main-agent.md` 第 24、38、100 行与新写的第 2、4 条矛盾**（第 6.3 问）：第 1 条「不当场派」、第 15 条「并进这一轮下一批」、派发表「测试或门禁结果与预期不符」那一行，都没给子 agent 与协作工具的问题留出「当场修」。
15. **门禁与登记表里七处仍写往现行账本追加**（第 6.4 问）：`.claude/gate.d/split-proposal-ledgers.tsv:6`、`.claude/gate.d/doc-process-records.sh` 第 454、458、772、2917 行、`.claude/gate.d/stage-owners.tsv:139`、`scripts/kb/stale-candidates-record-carriers.tsv:3`。账本不删，这几处改成「封存、不再往里登」；第 454 行那一格的退役判据已满足，那一格不删（它判封存之后编号不被改），只改判据句。
16. **两份项目规则「先记欠账、按批立」没给协作工具留出例外**（第 6.4 问）：`.claude/rules/session-wrapup.md:15`、`.claude/rules/show-me-test-by-review.md:12`。上游 `.claude/singlefs-ai-sop/rules/sop-first.md:45` 不在本仓改。
17. **`.claude/gate.d/break-switch-replay-files.tsv:10` 写「七十二条」，开关复跑数得 73 条**（第 9 问，v5）。
18. **`.claude/agents/investigator.md:33` 写「链接留给合入后验证」，而「查到就修」那一支在主工作区改、不走合入后验证；同一行里证红命令的树与前面 `--cwd` 写仓根对不上**（第 10 问）；**`.claude/agents/implementation-writer.md:51`（第 4 步）仍写 `cargo build`，与第 1b 步（第 32 行）矛盾**；`.claude/agent-common.md` 第 60、69 行「clippy / build」同改。

不采：

- **「不弹窗问」「同一时刻撞上的几条并进同一个工具员」超出用户原话**（第 6.2 问）：前一句从「撞上就修」直接推得出，用户在别处定过不为授权弹窗；后一句与 `.claude/agents/tooling-writer.md:20` 一个意思。不改。
- **两台 machine-id 相同时把活着的持有者判成不在**（第 1 问）：探针读数坐实了这一条件下会判错，但今天两台 machine-id 不同（攻方腿读结局文件开始行的机器段量的），部署上够不着；在文件头写明这一条件，不改判法。

核不动：

- 另一台看修改时间那一支要两台钟差小于约 285 秒：第二台的钟没核（攻方腿与主 agent 都没量）。
- 第二台跑不跑得起三份复跑入口（KVM、musl、QEMU、内核镜像）：没派到第二台跑过。

## 三、改法

- 换上工具（第 1 至 4 条）：真换那一段（读结局文件到写完成或退回）另占一份按仓的锁，放在结局文件同一目录、用同一套 `link` 记录；`--seal-outcome` 也占它，并像 `pending_refusal` 那样看开始行那一趟还在不在、在就退 9；摘旧记录改成摘走之后核内容、不是自己读到的那一份就原样放回且不删；`link` 报 EEXIST 而读回来是自己这一趟的记录时当放成。来源是攻方腿的推导，零轮；每一条先在自证里做成会红的格再改。
- 文档门禁（第 5 至 7 条）：exact 行一路按行放，同一个实验有 timing 行而装置改了，照旧比修改时刻；`gate_added_lines` 失败时写「哪一条 git、退几」与原文；`load_change_range` 取到时照打命令的标准错误。零轮；各配样本与开关。
- 完工清理闸（第 8、9 条）：只判交回的这一个 agent 自己写的那几层；带引号的路径先按引号切再判三段；符号链接的 `.deliverables` 判写法不合。零轮。
- 三份复跑入口（第 10 至 12 条）：run-condition 加可读内核镜像；删旧闸前核输出文件进没进 git；E53 自证不喂参数并核 `VM_DISKS`；E159 核子命令。零轮。
- E185（第 13 条）：三份起跑脚本的钉值改回各自登记第十二节写的那一个（r4 照 `research/prompts/e185-r4-prereg.md` 第十二节，r3 照 `git show a24379563^:research/prompts/e185-r3-prereg.md` 第十二节，r2 照它自己的登记），三语实验页里「改成现值」的那几句一起改回；装置改了而登记没补修订行时提交就红的门禁格，交用户定立不立。
- 规则与定义（第 14、16、18 条）：主 agent 定点改，`main-agent.md` 三处与两份项目规则都补一句「子 agent 与协作工具的问题照第 4 条当场修」，调查员定义「查到就修」那一支写明不走合入后验证、clippy 照第 1b 步，实现员定义第 4 步删 `cargo build`。
- 门禁文字与登记表（第 15、17 条）：工具员照第 15 条改那七处；登记表第 10 行改成七十三条。

改完再攻一轮（治理文件轮第二轮），攻这一轮的修补，加这一轮只读代码、没复跑的第 11 条与核不动的两条。

## 交用户的

| 事 | 依据 | 被攻过几轮 |
|---|---|---|
| 单例例外的射程：`crates/` 里 38 处 `static OnceLock`，只有 3 处明确是外部句柄；另 31 处是一次建好、只读的查表（含 `singlefs-core`、`singlefs-checker`、`singlefs-unit-graph`）。照现在的写法它们都违例。放宽成「建好之后只读的进程级值」，还是照现在的写法、把这 31 处改掉 | 正推腿第 8.2 问；主 agent 现数 38 | 零轮 |
| 装置改了而登记第十二节没补修订行时，提交就红的门禁格立不立 | 攻方腿第 5 问「以后怎么拦」 | 零轮 |

## 这一轮被判过的文件

- research/prompts/_ledger-open-fixes-governance-r1-body.md：两条腿照它的十问作答，问题与范围够判，「实现今天的样子」里「真换的日志」一句被核过。
- scripts/gate/hook-swap.py：判了进口与出口、同一台四样、另一台心跳、两条缝、结局文件与占用锁的键不同；坐实四条要改。
- scripts/gate/changed-paths.sh：判了报错写到标准错误之后格的结局不变；`gate_added_lines` 失败不写哪一条 git 坐实要改。
- .claude/gate.d/doc-text.sh：判了预取报错接住之后格的结局与成功、拒绝句不变，开关点名照复跑读数对。
- .claude/gate.d/doc-process-records.sh：判了 `load_change_range` 与原五个函数逐个对，值与状态相同、标准错误不同；第 841 行说法与实现不一；第 454、458、772、2917 行旧账本说法要改。
- .claude/gate.d/doc-experiments.sh：判了 evidence-in-repo 的 exact 行一路按实验号放，与第 821 行相反，坐实要改。
- .claude/hooks/completion-cleanup-guard.sh：判了交回时拒写法不合的多拒两种、漏拒两种，挪场景之后 `s` 的赋值成立，开关复跑 73 条逐格相同。
- .claude/hooks/project-code-guard.sh：判了只改注释的那几处与开关点名，正推腿第 9 问逐条对上。
- scripts/hosts/memory_peaks_lib.py：判了第 42 行注释改成现状这一处，两条腿都没打中。
- scripts/experiment-apparatus/e07-vm-bench-run.sh：判了入口脚本那一节不该守、退 78 准入漏内核镜像、删旧闸会删未进 git 的产物。
- scripts/experiment-apparatus/e53-vm-bench-run.sh：判了与 e07 只差设计内的几处，自证喂参数与 `VM_DISKS` 不核两处说明与实现不一。
- scripts/experiment-apparatus/e159-vm-bench-run.sh：判了与 e07 只差设计内的几处，用法行要子命令而实现不核。
- .claude/hooks/selftest-registry.tsv：判了第 104–106 行三份入口的登记与脚本的 selftest 类、格数对得上。
- .claude/gate.d/break-switch-replay-files.tsv：判了第 90–92 行三份入口各十二条与开关复跑 36 条对上，第 10 行「七十二条」与复跑 73 条对不上。
- .claude/gate.d/code-tooling.sh：判了第 2344 行起登记三份入口自证的那几项与脚本对得上。
- scripts/experiment-apparatus/e185-r2-segment-run.sh：判了钉值与现算 sha256 是同一个量，重钉绕过了停机，要改回登记写的那一个。
- scripts/experiment-apparatus/e185-r3-segment-run.sh：判了钉值不等于 r3 登记第十二节，重钉把停机 J-S7 变成照跑。
- scripts/experiment-apparatus/e185-r4-segment-run.sh：判了钉值不等于 r4 登记第十二节，重钉把停机 K-S6 变成照跑。
- CLAUDE.md：判了「经验与约定写在哪」那一段与用户原话、两句弹窗定案逐句对，「不弹窗问」推得出、不改。
- .claude/main-agent.md：判了第 2、4、14 条新写法对得上用户原话，第 24、38、100 行三处与新写法矛盾要改。
- .claude/rules/implementation-workflow.md：判了「治理文件同规矩」改成当场走三方，对得上弹窗定案。
- .claude/agent-common.md：判了第 4 行与 ⑭、第十一道两句与两份钩子逐项对上，第 60、69 行「clippy / build」要改。
- .claude/agents/tooling-writer.md：判了第 20 行与实现员定义第 1b 步、「子 agent 不交任务」一个说法。
- .claude/agents/investigator.md：判了第 33 行「链接留给合入后验证」与「查到就修」那一支不走合入后验证矛盾，证红的树写法前后不一。
- .claude/rules/code-discipline-by-review.md：判了第 54 行单例例外与上游编码纪律、机器优先不冲突，射程罩不住 `crates/` 里 31 处只读查表，交用户。

## 回看决策

不涉及决策：这一轮判的是治理文件与实验起跑脚本的钉值，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条；E185 钉值改回登记值，实验结论与产物不动。
