# ledger-open-fixes-governance-r2：判决（2026-11-19）

正文 `research/prompts/_ledger-open-fixes-governance-r2-body.md`，开工快照 `research/prompts/ledger-open-fixes-governance-r2-snapshot/`。这是这一题的第二轮；第一轮判决 `research/prompts/ledger-open-fixes-governance-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（第 1 到 5 问） | `research/prompts/ledger-open-fixes-governance-r2-opus-output.md`（sha256 `9e0c7e70…`） | 第 1 问 `.taking` 留下的长挡与两种新起因打中；第 2 问第一轮第 5 条的改法碰不到打中的那一格；第 3 问认层不认写者打中、后果轻；第 4 问路径按 pathspec 解打中；第 5 问三问都打中；撤回第一轮攻方腿两条改法 |
| 辩方（第 6 到 9 问） | `research/prompts/ledger-open-fixes-governance-r2-sonnet-output.md`（sha256 `85c65961…`） | 18 条里 17 条站得住，第 18 条一句写宽；钟差判决够不着；六处偏离都偏得对；交用户那一条写得不够让用户定 |

核查命令（在主 agent 的草稿目录 `~/.local/share/singlefs/work/open-fixes-governance-r2-verify/` 下，日志在它的 `main-agent-logs/`）：

- 倒推快照：`python3 scripts/agents/snapshot-tree.py ledger-open-fixes-governance-r2 --out <草稿>/ledger-open-fixes-governance-r2-tree --repository <仓根>`，日志 `w0-倒推快照.log` 末行「清单点名 43 份，倒推出 42 份、sha256 对上 42 份；倒推不出 1 份」。倒推不出的是 `.claude/gate.d/break-switch-replay-files.tsv`（快照之后别的会话改了它）；腿引它的行主 agent 对着主工作区逐行核了。
- 引文：`python3 scripts/agents/cite-check.py <两份报告> --root <草稿>/ledger-open-fixes-governance-r2-tree --background research/prompts/_ledger-open-fixes-governance-r2-background.md`，末行「核了 61 处引文、28 处裸行号，对上 44 处，没判 45 处」，判错 0 处。
- 攻方腿五份探针拷到 `probe/` 下复跑，第 3、7 条在仓根起：
  - `w1-taking.log`：`arm=taking-left-by-kill-repository-lock … child_reached_taking=True child_exit=-9 … taking_left=True record_age_seconds=3600 next_swap_exit=10 seal_exit=10 refusal_names_taking=True refusal_promises_expiry=True after_record_removed_exit=0 … next_killed_holder_then_swap_exit=10 after_taking_removed_exit=0`；`arm=taking-left-by-term-with-scratch-held child_reached_taking=True child_exit=-15 … scratch_record_released=True taking_left=True`；`arm=taking-link-reply-lost first_held=False taking_left=True plain_retry_held=False plain_retry_promises_expiry=True`；对照 `arm=stale-record-without-taking-control held=True`。
  - `w2-evidence.log`：`sample=control-as-fixture exit=1`、`sample=exact-only-rerun exit=0`、`sample=unrelated-e915-file exit=0`。
  - `w3-evidence真仓.log`：真仓上这一格绿，输出里 `E162`、`E174`、`E184` 0 处。
  - `w4-own-layers.log`：`arm=successor-blocked-by-line-it-did-not-write action=block names_line=True`、`arm=nested-upper-blocked-by-lower-handover-copy action=block names_copy=True`，对照两臂照设计。
  - `w5-pathspec.log`：四份入口各两臂，文件名 `e07-vm-bench-r[2].out` 那一臂都是 `untracked_output_kept=false refused_by_delete_gate=false reached_build=true`，对照 `e07-vm-bench-r3.out` 都是 `untracked_output_kept=true refused_by_delete_gate=true`；`committed_file_untouched=true`。
  - `w6-pin.log`：六份样本 `inplace-rewrite`、`internal-whitespace`、`duplicated-line`、`repin-only`、`pin-with-trailing-comment`、`registry-named-second` 都 `exit=0`；`repin-only` 的成功句「装置没变不判 1 份」，`registry-named-second` 的「登记已归档、不在树里不判 1 份」。
  - `w7-archive.log`：`research/prompts/e185-r4-prereg.md` 在「留下 139 份还被代码当输入的产物」那一节，被 `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py` 引着，下次提交不归档。
- 辩方腿的钟差探针经任务服务交了四趟，都派在本机（`machine=52bfb723`），`skew_seconds=0`；第二台由主 agent 经 ssh 在第二台上照同一做法量了三次（往共享根写一份文件、比本地钟与它的修改时间）：`machine=bee27b3c skew_seconds=0` 三行，两台的钟当场比差几毫秒（含一次 ssh 往返）。
- 主 agent 现查：`.claude/gate.d/stage-owners.tsv:143` 仍写「新内容挪进现行账本」；`crates/singlefs-verification/src/crash_amplification.rs` 里 `GivenStatesEngine` 的文档写「几条崩溃注入的工作线程共用一个：库与账各一把锁」。

## 二、判决

坐实，这一轮改：

1. **换上工具的 `.taking` 留下之后没人清**（攻方第 1 问，探针 `w1`）：持有者被 KILL、被 TERM（`release_then_die` 放完草稿目录那一份就按默认处理重发，`finally` 跑不到）、摘旧记录那一次 `link` 回复丢了，三种都留下 `.taking`；之后再碰上一份被 KILL 的持有者留下的旧记录，这个仓的真换与 `--seal-outcome` 每次都退 10，拒绝说明里「300 秒没刷新就不再挡」那句在这里不成立。
2. **第一轮第 5 条的改法碰不到打中的那一格**（攻方第 2 问，探针 `w2`）：同一个实验有 timing 行时照旧比修改时刻，可比的是这个实验最新的那一份产物，只重跑一条 exact 驱动、甚至放一份不相干的 `e<号>` 起头的文件就绿；timing 那一段改了，照旧既不要新 timing 产物、也没有复跑。另外正文里「E184 以后改装置就红」不成立：E184 的装置在 `crates/` 下，这一格不认 `crates/*/src/bin/` 下的装置（攻方腿数得 19 个实验的装置在那里）。
3. **删旧闸的路径按 pathspec 解**（攻方第 4 问，探针 `w5`）：文件名带 `[` `]` 时，没进 git 的输出被当成「在 git 里、没改过」删掉，四份入口都这样。
4. **装置钉值的门禁格三处都漏**（攻方第 5 问，探针 `w6`）：只改钉值（第一轮第 13 条那一种）不触发；登记里原值一行原地改写、词中间多一个空格、抄一行已有的，都算「新加了修订」；钉值行带行尾注释的整份不进对象；注释里先点名别的登记的认错登记；登记已归档的不判，r2、r3 永远罩不到。
5. **完工清理闸嵌套派发时上层被下层的交接副本拦**（攻方第 3 问，探针 `w4` 第三臂）：下层已声明交接的副本，上层交回时被当成没声明的拦下。
6. **`.claude/gate.d/stage-owners.tsv:143` 漏改一句旧账本说法**（辩方第 8 问，现查）。

不采：

- **承接的子 agent 被前一个写坏的行拦**（攻方第 3 问第一臂）：承接者接手了那份草稿目录与写者锁，那一行此刻归它，拦下让它改是对的。
- **`_lib_changed_paths_run_git` 成功时照打 git 警告会让样本档判错**（攻方第 2 问末段）：攻方腿自己判不成立，样本档只核退出码与 `want=` 子串。
- **两份锁反序互等、`--dry-run` 不占按仓的锁**（攻方第 1 问）：攻方腿判不成立，取锁只有一种次序、取不到当场退；真换在锁里重读重核。

更正第一轮判决（冻结证据不回改，更正写在这里）：

- 第一轮第 18 条「`.claude/agent-common.md` 第 60、69 行「clippy / build」同改」写宽了：第 69 行说的是项目代码闸怎么判两种命令（辩方第 6 问），没改是对的。
- 第一轮「核不动」的钟差，这一轮量了（第一节末两条），两台都是 0 秒，第一轮「另一台看修改时间那一支」成立的前提今天满足。辩方腿判这一格第一轮就该量，判得对。
- 第一轮「交用户的」单例那一条写得不够（辩方第 9 问）：3 加 31 是 34，另 4 处（卡租约、卡池、开卡记录）没写；31 处里 4 处 `GivenStatesEngine` 带锁、建好之后还在变，本来就被例外的末句挡在外，不是「只读查表」。用户已定「照现在的写法」，结论不变：除了 3 处外部句柄，别的都要改；那 4 处要读类型判，派实现员时写明。

## 三、改法

- 换上工具（第 1 条）：`.taking` 里写持有者身份（与记录同一套），判它挡不挡照记录那一套（同一台看进程、另一台看修改时间）；TERM 与 HUP 处理里放掉这一趟占着的 `.taking`；摘旧记录那一次 `link` 报已有而读回来是自己的，当放成。拒绝说明照新判法改。零轮；每一种起因先在自证里做成会红的格。
- evidence-in-repo（第 2 条）：同一个实验有 timing 行时，按 timing 行点名的那一份产物比修改时刻，exact 行的产物与不相干的文件不算；`crates/*/src/bin/` 下的装置一并认。攻方腿撤回第一轮自己给的这条改法，照撤回后的这一条做。零轮。
- 删旧闸（第 3 条）：两条 git 的路径加 `:(literal)` 前缀，四份入口同改。零轮。
- 钉值格（第 4 条）：触发改成「钉值变了或装置变了」；修订认法改成登记第十二节新加的一行里出现新钉值（至少前 12 位）；钉值行允许行尾注释；登记从脚本里写明登记路径的那一处取，不取注释里头一处；登记不在树里时从 git 历史里取它被删前的那一版判。零轮。
- 完工清理闸（第 5 条）：下层已声明交接的副本，上层交回时算已声明。零轮。
- `stage-owners.tsv:143`（第 6 条）：改成封存的说法。

改完再攻第三轮（最后一轮）；第三轮改的修补在判决里标零轮。

## 交用户的

| 事 | 依据 | 被攻过几轮 |
|---|---|---|
| 告知：第一轮交你定的单例那一条写得不够——38 处里 3 处外部句柄、31 处不在例外里、另 4 处（卡租约、卡池、开卡记录）要读类型才知道；31 处里 4 处 `GivenStatesEngine` 带锁，不是只读查表。你定的「照现在的写法」照旧：除了 3 处外部句柄，其余都改，那 4 处派实现员时读类型判 | 辩方第 9 问；主 agent 现查 `crash_amplification.rs` 的 `GivenStatesEngine` 文档 | 一轮 |

## 这一轮被判过的文件

- research/prompts/_ledger-open-fixes-governance-r2-body.md：两条腿照它的九问作答；「E184 以后改装置就红」一句判不成立（第二节第 2 条）。
- research/prompts/ledger-open-fixes-governance-r1-main-verification.md：辩方腿复核，17 条站得住、第 18 条一句写宽、钟差该在第一轮量、交用户那一条写得不够（第二节更正）。
- scripts/gate/hook-swap.py：判了 `.taking` 留下之后的长挡与三种起因（第 1 条），两份锁的次序与演练不占锁判不成立。
- scripts/gate/changed-paths.sh：判了成功时照打 git 警告不让样本档判错。
- .claude/gate.d/doc-experiments.sh：判了 evidence-in-repo 照 timing 行比修改时刻的修法碰不到打中的那一格、不认 `crates/` 下的装置（第 2 条），钉值格三处都漏（第 4 条）。
- .claude/gate.d/doc-process-records.sh：判了多改的七处旧账本说法偏得对。
- .claude/hooks/completion-cleanup-guard.sh：判了认层不认写者，承接那一臂不采、嵌套那一臂坐实（第 5 条）。
- .claude/gate.d/split-proposal-ledgers.tsv：判了改成封存的说法与 `CLAUDE.md` 那一句对得上。
- .claude/gate.d/stage-owners.tsv：判了第 139 行改对、第 143 行漏改（第 6 条）。
- scripts/kb/stale-candidates-record-carriers.tsv：判了第 3 行改成旧账本封存的说法。
- .claude/gate.d/break-switch-replay-files.tsv：判了第 10、47、89–92 行的条数与开关复跑读数对得上。
- .claude/hooks/selftest-registry.tsv：判了第 103–106 行的格数与各份文件头对得上。
- scripts/experiment-apparatus/e07-vm-bench-run.sh：判了删旧闸按 pathspec 解（第 3 条），换名挪走之后登记对得上、git 不在时拒判不成立。
- scripts/experiment-apparatus/e53-vm-bench-run.sh：同 e07 那一份的删旧闸（第 3 条）。
- scripts/experiment-apparatus/e159-vm-bench-run.sh：同 e07 那一份的删旧闸（第 3 条）。
- scripts/experiment-apparatus/e56-vm-bench-run.sh：同 e07 那一份的删旧闸（第 3 条）。
- scripts/experiment-apparatus/e185-r2-segment-run.sh：判了钉值等于登记第十二节，登记已归档、钉值格罩不到（第 4 条）。
- scripts/experiment-apparatus/e185-r3-segment-run.sh：同 r2 那一份，登记已归档（第 4 条）。
- scripts/experiment-apparatus/e185-r4-segment-run.sh：判了钉值等于登记第十二节，登记靠重判脚本引用才保住。
- scripts/experiment-apparatus/README.md：判了三份入口的说明与脚本今天的行为对得上。
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-mixed-replay-red/setup.sh：判了它红在照 timing 行比修改时刻那一处，攻方腿的探针拿它做对照。
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-mixed-replay-red/expect：判了期望行与这一格红句对得上。
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-green/setup.sh：判了绿样本只罩「装置改了且登记新加一行」那一种，罩不到第 4 条那几种写法。
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-green/expect：判了期望与新格成功句对得上。
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-red/setup.sh：判了红样本只罩装置改了而登记没动那一种。
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-red/expect：判了期望与新格红句对得上。
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-unresolved-red/setup.sh：判了认不出装置的那一种红。
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-unresolved-red/expect：判了期望与红句对得上。
- .claude/gate.d/fixtures/doc-process-records.sh/batch-scope-git-warning-red/setup.sh：判了它造得出 git 往标准错误写警告，样本档判对。
- .claude/gate.d/fixtures/doc-process-records.sh/batch-scope-git-warning-red/expect：判了期望行写的是那一行警告。
- .claude/main-agent.md：判了第 1、15 条与派发表一行补的例外与第 4 条一个说法。
- .claude/rules/session-wrapup.md：判了补的协作工具当场修一句与 `CLAUDE.md` 对得上。
- .claude/rules/show-me-test-by-review.md：同 session-wrapup 那一句。
- .claude/agents/investigator.md：判了第 33 行改成照实现员第 1b 步、这一支不走合入后验证。
- .claude/agents/implementation-writer.md：判了第 4 步删 `cargo build` 之后与第 1b 步一个说法。
- .claude/agent-common.md：判了「子 agent 不交任务」那一条改成只写 clippy，另一处「clippy / build」说的是项目代码闸、没改是对的。

## 回看决策

不涉及决策：这一轮判的是治理文件与门禁格，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条；改的都是门禁、钩子、脚本与登记表。
