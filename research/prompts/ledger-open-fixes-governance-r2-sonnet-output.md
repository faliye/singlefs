# 旧账本开着项当场修的治理文件轮第二轮：辩方腿报告（ledger-open-fixes-governance-r2，2026-11-19）

复核的是第一轮判决 `research/prompts/ledger-open-fixes-governance-r1-main-verification.md`，做正文「要腿判的」第 6 到 9 问。没写模型，没有 `rerun.sh`；全部是读文件与只读命令。没往任务服务交任务，判决要引的新读数（钟差）在「要主 agent 跑的命令」一节，现在都是「推的，待主 agent 跑」。

## 读到的范围

- 整份读：被复核的判决、第一轮两条腿的报告、第一轮正文、第二轮正文、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/evidence-discipline.md`。第二轮背景材料 346 KB（多半是 kb 附录），只读了正文与「实现今天的样子」。
- 第一轮开工那一刻的树：主 agent 倒推出来的 `~/.local/share/singlefs/work/open-fixes-governance-r1-verify/ledger-open-fixes-governance-r1-tree/`（29 份），第一轮四份探针在旧版上的读数在同目录 `main-agent-logs/v1`、`v2`、`v3`、`v4`、`v5` 五份日志里。判「修补有没有偏」时，旧版取这棵树、新版取主工作区此刻那一份。
- 只有这条腿读到的：`scripts/gate/hook-swap.py` 摘旧记录那一段与文件头、`scripts/experiment-apparatus/replay.sh` 里 E162、E174、E184 的登记行与驱动函数、`scripts/gate/changed-paths.sh` 的 `_lib_changed_paths_run_git`、`.claude/gate.d/doc-process-records.sh` 对第一轮树的差、E185 三份起跑脚本与 `research/results/` 里 E185 的三份产物头、`crates/` 里 38 处 `static OnceLock`（命令数的）。
- 没读：`.claude/hooks/completion-cleanup-guard.sh` 与 `project-code-guard.sh` 的实现（只读了后者文件头里写 build 的两处）、`doc-text.sh`、两道文档门禁样本档。

## 判定一览

| 问 | 判 | 一句话 |
|---|---|---|
| 6 | 18 条里 17 条判决站得住；第 15 条站得住而够不着一处残留；第 18 条里「第 69 行同改」那半句写宽了 | 第 1 到 6、8、9 条的理由有主 agent 在旧版上量的探针读数；第 13 条另有产物头里的第二条独立路；第 11 条不该要求复跑 |
| 7 | 「不弹窗问」不采：站得住，且有判决没引的更直接出处；machine-id 不采：站得住；钟差核不动：够不着（便宜、该核）；第二台跑不跑得起：站得住 | 见第 7 问表 |
| 8 | 六处偏离：五处偏得对（其中两处是判决字面本身够不着），一处不是偏离 | 摘旧记录用硬链接偏得对但留下一条新的长挡，归攻方腿第 1 问判可达 |
| 9 | 判决交用户那一条写得不够让用户定：38 不等于 3 加 31、31 里 4 处不是只读查表、没写「改掉」的代价与违例的后果 | 用户已据此定「31 处只读查表改掉」，边界比数出来的含糊 |

## 第 6 问：第一轮坐实的 18 条

序号对应判决「坐实，这一轮改」那 18 条（判决文件第 28 到 45 行，第 n 条在第 27 加 n 行）。

| 条 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|
| 1 `--seal-outcome` 不占不看 | 站得住 | 主 agent 旧版读数 `arm=seal-outcome-while-swap-running pending_while_running=9 seal_exit=0 next_swap_pending_refusal=none read_outcome_exit=0`（`main-agent-logs/v1-换上工具六臂.log` 第 4 行，量的） | 旧版上 `seal_exit` 退 9 |
| 2 结局文件按仓、锁按草稿目录 | 站得住 | `arm=outcome-overlap-different-scratch a_killed=True b_exit=0 x1_landed=True x2_landed=False y_landed=True next_swap_pending_refusal=none`，对照臂同位置是 9（同日志第 2、3 行，量的）；文件头原写「主工作区那一头仍有基线核对与结局文件拦着」，被这一臂推翻 | 对照臂与差异臂的 `next_swap_pending_refusal` 相同 |
| 3 缝一三趟可达 | 站得住（存在性）；频率没量 | `arm=gap-three-contenders-on-stale-record holding=A,C a_record_still_on_disk=False`（同日志第 5 行，量的）。次序是探针用线程挂在 `os.rename` 上造的，说明交错在代码里合法，不说明多常见；自证里同一格的注释写的就是这个次序（`scripts/gate/hook-swap.py:4611-4612`） | 旧版上 `holding` 只有一个 |
| 4 缝二自己的进程号被当别人 | 站得住（改法代价小、判决只说「一并改」） | `arm=gap-link-reply-lost held=False refused=True refusal_names_own_pid=True record_left=True`（同日志第 6 行，量的）。这是让 `link` 先做成再抛已有造出来的，真共享盘上回复丢失够不够得着取决于挂载协议，第一轮两条腿都没核。现行文件头写的放成规则：「放成：link 报已有、而读回来的就是这一趟自己写的那份记录」（`scripts/gate/hook-swap.py:79`） | 共享盘协议保证 `link` 不会回复丢失（那样这条改法多余但无害） |
| 5 exact 行一路按实验号放 | 站得住 | 复跑表 E913 两行 `E913\|e913-replay-timing…\|timing`、`E913\|e913-anchors…\|exact`，红行里没有 `e913_replay_timing.rs（装置源码…）`（`main-agent-logs/v3-复跑表exact行.log`，量的；该日志 `grep -c` 得 0） | 旧版上 E913 的 timing 装置照红 |
| 6 `gate_added_lines` 失败不写哪一条 git | 站得住 | `gate_changed_paths exit=1 header_line=1` 对 `gate_added_lines exit=1 header_line=0`（`main-agent-logs/v4-增行报错.log`，量的） | 旧版上第二行 `header_line=1` |
| 7 `load_change_range` 吞标准错误 | 站得住，但判决点名的层不够：实际两层都吞（见第 8 问 f） | HEAD 版五个 `load_*` 直接 `$(gate_changed_paths …)`，git 的标准错误照打（`git show HEAD:.claude/gate.d/doc-process-records.sh` 第 836 到 866 行）；第一轮树里 `_lib_changed_paths_run_git` 成了就丢 `error_file` | HEAD 版在取到时也不打 |
| 8 完工清理闸多拒两种 | 站得住 | `parent-directory-bad-line-blocks-child-handback action=block named=True`、`quoted-path-with-space-blocks action=block named=True`（`main-agent-logs/v2-交付物四臂.log` 第 2、3 行，量的） | 旧版上两臂 `action=pass` |
| 9 完工清理闸漏拒两种 | 站得住；判决给的改法（一种判写法不合、一种只在文件头写明）与「轻」的定性相称 | `symlinked-declaration-with-bad-line action=pass named=False`、`nested-declaration-with-bad-line action=pass named=False`（同日志第 4、5 行，量的） | 旧版上两臂 `action=block` |
| 10 三份入口缺内核镜像的准入 | 站得住 | 虚机驱动缺内核镜像走 `KERNEL="$(find_kernel)" \|\| die`（`scripts/devices/vm-bench.sh:301`），`die` 退 1，入口把它当作废；规则要求「整个脚本都离不开的工具、设备、硬件写成 `run-condition`」（`.claude/rules/test-script-discipline.md:30`）。准入规则自己把运行条件定义成环境撑不撑得住（`.claude/singlefs-ai-sop/rules/preflight-discipline.md` 的「两类条件」表），内核镜像与同文件已有的 musl 目标、`/dev/kvm` 同类 | 有人指出缺内核镜像时 vm-bench 退的是 78 |
| 11 删旧闸会删没进 git 的产物（只读代码、没复跑） | 站得住；判决当时**不该**要求复跑 | 这一句是读源码得的观测，不是推论（`.claude/rules/three-way-inference.md@适用范围` 把「查一个事实（跑命令、读文件、看源码）」排在推论之外）：第一轮树里的入口在建目录检查之后无条件 `rm -f -- "$output_file"`，旧版自证有一格把「上一趟留下的输出」放进输出路径、核它被删（第一轮树 `e07-vm-bench-run.sh` 第 222 行），那一格本来就在每趟自证里跑，等于已经有人复跑着。复跑拿不到新信息；要测的是改法，那是零轮，主 agent 在第二轮背景里已拿开关复跑（e07 14 条、e53 15 条、e159 15 条逐格相同）。没复跑的是「真有调用方把没提交的同名产物当输出路径」这一可达性，判决没把它写成已测 | 有人指出旧版入口在 `rm` 之前就核过输出路径 |
| 12 E53、E159 说明与实现不一 | 站得住 | 第一轮树 `e53-vm-bench-run.sh` 第 12 行写「复跑不给参数」而自证第 171 行喂 `selfcheck 4`；`e159-vm-bench-run.sh` 第 73 行用法行要 `<子命令>` 而第 327 行只核输出文件（读的是第一轮树，行号对旧版）。装置只要两块盘以上就照跑（`scripts/experiment-apparatus/e53_ring_failure.rs:168` 起 `device_paths.len() < 2`），所以 `VM_DISKS` 不核会跑出 `devices=2` 的产物 | 旧版里已有核 `VM_DISKS` 的一行 |
| 13 E185 钉值重钉成现算值 | 站得住；另有判决没引的第二条独立路 | 钉值与登记不等是量的：起跑脚本文件头写「不等就写完输入清单、退 3（停机 K-S6），不编、不跑。」（`scripts/experiment-apparatus/e185-r4-segment-run.sh:18`），登记第十二节写 `9e813cbd36900f1b0c7001e6de0f98de01080bbf7ac6aab39d8e7c9854bd97f8`（`research/prompts/e185-r4-prereg.md:579`），重钉前的钉值是 `d312b9aa71d0…`。第二条路是两份已出的产物头：「E7RESULT name=apparatus source_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d registered_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d」（`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out:2`）与「E7RESULT name=apparatus source_sha256=4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea registered_sha256=4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea」（`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out:2`）：产物自己记的装置哈希就是改回去的那两个值，重钉成 `d312b9aa…` 会让起跑脚本去跑一份与产物头不符的装置。第四次重跑的登记值 `9e813cbd…` 查不到对应的提交：我对 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` 与 `crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs` 两个路径在 `git log --all` 里列出的每个提交各算了一次 sha256：18 个（提交，路径）组合有内容，只有 5 种不同的哈希（`b3a50d50…`、`fdfd80d6…`、`c2509795…`、`d312b9aa…`、`043d8cac…`），25850 行的那一版（提交 `ff7e45d01`）是 `fdfd80d6…`，没有一份等于 `9e813cbd…`（量的，范围只是这两个路径）。这支持判决「不单改脚本、连登记一起改」：当前没有任何一份装置源码能让第四次重跑过 K-S6 | 某个提交里的装置哈希等于 `9e813cbd…` |
| 14 `main-agent.md` 三处旧句 | 站得住 | 第一轮树里第 14 行已有「子 agent 与协作工具的问题例外」而第 24 行写「批内冒出的发现只进下一轮的清单，不当场派。」没有例外、第 38 行写「并进这一轮下一批」。现行第 24 行已补「子 agent 与协作工具的问题例外，照第 4 条当场修。」（`.claude/main-agent.md:24`）。细则文件 `.claude/skills/dispatch/SKILL.md` 里没有写「不当场派」（`grep -nF '不当场派'` 零命中，量的） | 第 4 条的例外射程明说不含批内新发现 |
| 15 七处旧账本说法 | 站得住；第一轮的全仓搜没搜全，留一行同类 | 七处里我逐句 `grep -nF` 核到的是六处：`doc-process-records.sh` 的四处在第一轮树里命中，`split-proposal-ledgers.tsv` 第 6 行与 `stage-owners.tsv` 第 139 行在 HEAD 里命中；`scripts/kb/stale-candidates-record-carriers.tsv` 第 3 行第一轮腿引的「现行账本，装着开着的欠账」我在 HEAD 里没复现（HEAD 那一行写「装着开着的欠账」，别的会话之后改过它，该文件不在第一轮树里），这一处只能按腿的引文算，当「推的」。残留：`.claude/gate.d/stage-owners.tsv:143` 仍写「新内容挪进现行账本」，是 `sealed-ledgers-unchanged` 那一格的归属行，与已改的第 142 行相邻；第一轮 6.4 的搜词表（`往账本`、`登进账本`、`记进现行账本` 等）不含「新内容挪进」。另有守卫的出路文字仍指向加节脚本（`.claude/hooks/write-guard.sh:831`、`.claude/hooks/bash-command-detector.sh:377`），那是描述拦截的出路、不是叫人往里登，不算矛盾，列给主 agent 判。这与 `.claude/rules/evidence-discipline.md:156` 的「回扫按旧说法搜，不只按新名字搜」是同一个缺口 | 搜到 143 行说的是别的现行账本 |
| 16 两份规则「先记欠账、按批立」 | 站得住（两种读法下都成立） | 正推腿自己留了一个读法：这两句管的是 `.claude/kb/checks-owed.md` 那张欠账表、不是子 agent 账本（`.claude/rules/session-wrapup.md:15`、`.claude/rules/show-me-test-by-review.md:12`，上游 `.claude/singlefs-ai-sop/rules/sop-first.md:45` 写「一周合一批评估」）。若是后者，原句与「不记账」矛盾，补例外正确；若是前者，补的那半句只是把协作工具的坑划出去，对其余的坑原句照旧，也没有改坏。`.claude/rules/evidence-discipline.md:153` 那句「新立的判据、撤回的数各记一行进项目的欠账表」说的是回扫，不是立判据本身，不与当场修冲突 | 用户说协作工具的坑也要先记欠账 |
| 17 登记表「七十二条」 | 站得住 | 开关复跑「复跑了 73 条开关（1 份）…逐格相同 73 条」（`main-agent-logs/v5-完工清理闸开关.log:99`，量的）对旧值七十二 | 复跑数不是 73 |
| 18 调查员与实现员定义、共用约束「clippy / build」 | 前两处站得住；「共用约束第 69 行同改」那半句写宽了 | 实现员第 4 步 HEAD 版第 51 行列着 `cargo build --offline --all-targets` 而第一轮时第 32 行（1b）已写「不写 cargo build」，现行第 4 步已无（`grep -nF 'cargo build' .claude/agents/implementation-writer.md` 只剩第 32 行那句「不写」）。调查员第 33 行的「链接留给合入后验证」搬了一个这一支没有的下游，现行改成「这一支在主工作区改、不走合入后验证，链接留给提交时的门禁」。共用约束第 69 行（现第 70 行）「clippy / build 逐条的判法在 `.claude/hooks/project-code-guard.sh` 文件头」是在说这份文件头怎么判两种命令，文件头写着「会编的 cargo（build、check、clippy、test、run、bench、doc」（`.claude/hooks/project-code-guard.sh:187`），不是在叫子 agent 写 build；正推腿原话也只判它「推不出」。判决把它与第 60 行并列写成「同改」，比腿的依据宽一格 | 门禁文件头不判 `cargo build` |

## 第 7 问：不采的两条与核不动的两条，先写「该在第一轮改或核」那一方最强的理由，再判

| 条 | 该改或该核那一方最强的理由 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|---|
| 「不弹窗问」超出用户原话（不采） | 用户的两句弹窗定案是「不删账本，只改规则」「撞上就修，三方也当场」，没有「不弹窗」；`CLAUDE.md` 是每个新会话必读的规范句，往里写「也不先弹窗问要不要修」把问的口子关死，用户想被问的场合（放宽一道门禁、删一个钩子）也被关；`.claude/main-agent.md:14` 同一句里并排写着「扩大任务必须弹窗」，两句并读容易读成协作工具的问题永不弹窗 | 判决站得住，且有判决没引的直接出处 | 判决写的是「用户在别处定过不为授权弹窗」，没给出处。出处在同一份调度记录里：用户说「本次要把欠账基本都还清  如果有需要问我的放到最后」（`records/2026-11-17-subagent账本清欠调度记录.md:114`），随后「之后就等我问你 再弹窗」，「此后不弹窗，要问的攒着等用户来问」（同文件第 117 行）；更早有「授权你所有操作」不弹窗（`records/2026-10-12-里程碑三工具与治理批.md:61` 的写法）。同一份记录第 371 行又写设计岔路仍弹窗问了用户，所以「不弹窗」的射程是授权与处置、岔路照问；`CLAUDE.md` 那句只到「要不要修」，是授权类，落在射程里。弱点只剩一句：`CLAUDE.md` 的句子比弹窗定案那两句多了一截，写进规范句的人该写它的依据，当前没写 | 用户说「协作工具的问题撞上了要先问我」 |
| 「同一时刻撞上的几条并进同一个工具员」超出原话（不采） | 用户没说过「并进」；并进会让一个工具员的出口变多 | 判决站得住 | 工具员定义写「可以不止一条，主 agent 按要改的文件聚簇，逐条列、各有出口」（`.claude/agents/tooling-writer.md:20`）；`.claude/main-agent.md:14` 同句写「一件活可以关多条问题」。是已有的聚簇做法，不是新规 | 工具员定义里没有「聚簇」那一句 |
| 两台 machine-id 相同时判错（不采，只在文件头写条件） | 多机可用是没有例外的核心策略（`.claude/agent-common.md` 「所有任务都要多机可用」一条），判决却靠「今天两台 machine-id 不同」这一部署事实挡住一个静默双持；这个事实只在结局文件开始行里量过一次，没有任何检查盯它（我在 `scripts`、`.claude/gate.d`、`.claude/hooks`、`task-server` 里 `grep -rln machine-id` 只命中 `hook-swap.py`，量的）；记录里已经有主机名一列，把机器键并上主机名几乎零成本；探针已经量到条件成立时的错判：`same-machine-id-other-boot live_holder_counts_as_occupying=False process_state=不在`（`main-agent-logs/v1-换上工具六臂.log` 第 7 行，量的） | 判决站得住；但「条件无人盯」是真缺口 | 条件已写进文件头：「判法不改，两台的 machine-id 要不同」（`scripts/gate/hook-swap.py:57`）。不改判法的理由成立：machine-id 相同只发生在克隆镜像，换键（并上主机名）也有自己的失败（主机名可重名、可改），后果限于这一个换上工具。「没有检查盯」不是第一轮判决能补的，是一条可以另开的小检查（两台 machine-id 不同），我只记下，不替主 agent 定 | 两台的 machine-id 被量到相同（`/etc/machine-id` 的 sha256 前 16 位），或换上工具被派到第三台 |
| 另一台的钟差（核不动） | 这一支的存亡就是这一个数：文件头把 300 秒写成「容得下共享盘属性缓存与两台时钟的差」（`scripts/gate/hook-swap.py:2642`），没有一处量过；它不需要挑台——往共享根写一个文件、比本地钟与它的修改时间就是这一支用的那个差（共享盘上文件的修改时间由服务端那一台定），经任务服务多交几趟两台都会轮到，输出里带 machine 前 8 位区分；探针在「要主 agent 跑的命令」第 1 条；钟差超过约 285 秒的方向会让活着的另一台持有者被算过期、两趟同时进，这一条也没写进文件头的条件 | 判决够不着：该在第一轮就核，成本是两趟极小的任务 | 判决写的是「第二台的钟没核（攻方腿与主 agent 都没量）」，没写为什么不量；机制上服务选台、子 agent 与主 agent 都不能点名（`.claude/agent-common.md` 「命令怎么写」一段禁了 `--peer`、`--local-only`），所以「点名第二台量」做不到，但「不点名、多交几趟」做得到。先验上两台都对时则差在毫秒量级，所以这一条是便宜该核、不是高风险 | 探针两台各出一行、`skew_seconds` 都在 ±5 以内 |
| 第二台跑不跑得起三份入口（核不动） | 第一轮判决把「派到某一台就红的是缺陷」当核心策略却没验第二台；入口依赖的 KVM、musl、QEMU、内核镜像第二台有没有一个都没看 | 判决站得住：核不动是对的 | 缺任何一样，入口按设计退 78 不是红：现行第 6 行 run-condition 已含内核镜像（`scripts/experiment-apparatus/e07-vm-bench-run.sh:6`），musl 与 `/dev/kvm` 在第 4、5 行；而 78 是「运行条件没满足」，门禁不把它记成红（`.claude/rules/test-script-discipline.md` 退出码一条：「78 准入或运行条件不满足」）。真跑得起要在第二台有 KVM 才能见，服务选台不由调用方定；没有记录说明第二台跑过虚机入口（我对 `records/*.md` 搜同一行里有 `vm-bench` 与「第二台」的两处命中都不是这件事，范围只是这一次搜）。自证用假驱动、与机器无关，所以「自证两台都跑得起」是另一件事、已由主 agent 在第二轮背景里的读数覆盖 | 某趟入口在缺 KVM 的机器上退 1 而不是 78 |

## 第 8 问：修补偏离判决字面的六处

| 处 | 辩方判定 | 依据 | 什么现象会推翻 |
|---|---|---|---|
| a 摘旧记录用硬链接、不改名 | 偏得对：判决字面（改名摘走、核内容、放回且不删）修不到判决要修的缝；代价是新添一条长挡，归攻方腿第 1 问判可达 | 判决写「摘旧记录改成摘走之后核内容、不是自己读到的那一份就原样放回且不删」（判决第 59 行）。改名会让记录的位置空出来：三趟里的丙在空位上放成自己的，乙再放回就撞已有，不论乙删不删摘到的那份，甲（记录在乙手里或已被删）与丙都在占着。这正是弄坏开关 `stale-record-renamed-away` 要造的交错：「A 的记录被挪开、C 放成、放回去撞上 C 的、A 的记录被删，A 与 C 同时占着」（`scripts/gate/hook-swap.py:4612`），开关说明见 `scripts/gate/hook-swap.py:320`。「不删」只改变摘到的那份留在盘上还是被删，不改变甲在路径上没有记录这件事（这一段是读代码推的，没另造一个「不删」版本去复跑）。现行做法：「记录从头到尾不挪开，别的趟不会在中间看到一个空位放进去」（`scripts/gate/hook-swap.py:84`）。代价：摘的那一趟在两次文件操作之间被 KILL 会留下 `.taking`，之后所有趟都当占着退 10，没有按时间过期的出路：「摘的那一趟被 KILL 会留下 .taking，拒绝说明点名它，确认没有在跑的 hook-swap.py 就删它」（`scripts/gate/hook-swap.py:86`）；这是把安全性放在存活性前面的一次交换，窗口只有两次系统调用与一次读，是否可达归攻方腿 | 用改名版加判决的「不删」在三趟交错下探针仍只有一趟占着 |
| b evidence-in-repo 取「做不到按行」那一支 | 不是偏离：判决字面第二分句就是这一支 | 判决写「exact 行一路按行放，同一个实验有 timing 行而装置改了，照旧比修改时刻」（判决第 60 行）；实现与文件头一致：「所以同一个实验里有一条 timing 行，整个实验都不走复跑表这一路」（`.claude/gate.d/doc-experiments.sh:846`）。「做不到按行」有证据：复跑表的行点名的是驱动函数名（`scripts/experiment-apparatus/replay.sh:2815` 的 `E162\|@driver_e162_anchors\|\|…\|exact`），驱动函数体里是「`"$REPLAY_RELEASE_DIRECTORY"/e162-crash-verdict-block-store anchors`」（`scripts/experiment-apparatus/replay.sh:4281`）即二进制名加子命令，同一份源码里的子命令分段没有机读的对应。今天受影响的实验有三个：我数 `replay.sh` 的登记行，E162 为 exact 2 条、timing 3 条，E174 为 exact 4 条、timing 2 条，E184 为 exact 1 条、timing 1 条（量的）；它们 exact 行点名的产物今天都不在 `research/results/`（`ls` 没有这几个文件名，量的），所以这一支今天本来也走比修改时刻，副作用要等产物回库才出现 | 某个同时有 exact 与 timing 行的实验，驱动函数可机读地映射到装置源码文件 |
| c 删旧闸比判决严一格（在 git 里而有没提交的改动也拒） | 偏得对 | 判决只拒「没进 git」，理由是删了找不回（判决第 38 行）；在 git 里而与 HEAD 不同的文件，删了同样找不回（git 里只有 HEAD 那一份），同一个理由。现行判法：「已存在的输出文件只有在 git 里、内容与 HEAD 相同（删了在 git 历史里找得回）时才删」（`scripts/experiment-apparatus/e07-vm-bench-run.sh:399`）。与 Bash 检出钩子 ⑤ 同向（整份覆盖没进 git 的 `research/results/` 产物要拒）。代价：同名产物第二次复跑（第一次留下的是改过没提交的）要换名或挪走；门禁记的是产物名，所以这对复跑登记表的影响归攻方腿第 4 问 | 有一条正常的复跑流程要求第二次覆盖同名未提交产物 |
| d `agent-common.md` 另一处「clippy / build」没改 | 偏得对；是判决写宽了 | 第 70 行那句说的是项目代码闸文件头里怎么判两种命令：`.claude/agent-common.md:70`「clippy / build 逐条的判法在 `.claude/hooks/project-code-guard.sh` 文件头」，文件头确实写了会编的 cargo「build、check、clippy、test、run、bench、doc」（`.claude/hooks/project-code-guard.sh:187`）。改成只写 clippy 会让这句话变成假话。第一轮正推腿对它的判定是「推不出」而不是「不成立」，判决写「同改」比腿的依据宽一格（见第 6 问第 18 条）。第 61 行那一句（要写进报告的命令种类）已改成「clippy（不写 `cargo build`，照 `.claude/agents/implementation-writer.md` 第 1b 步）」，这一处才是该改的 | 项目代码闸文件头不再判 `cargo build` |
| e `doc-process-records.sh` 多改了七处旧账本说法 | 偏得对；同一个缺陷的同族，按「回扫按旧说法搜」就该一起改；仍漏一行 | 我把第一轮树与现行文件对差、只看含「账本」「封存」「追加」的变动行（`diff <(cat <第一轮树>/.claude/gate.d/doc-process-records.sh) .claude/gate.d/doc-process-records.sh \| grep -E '^[<>]' \| grep -E '账本\|封存\|追加'`，变动成对出现），除判决点名的四处外，另有三类：红行的出路写「往现行账本追加」改成「新内容不往账本里登」（现 `.claude/gate.d/doc-process-records.sh:806`、`:3070`）、红行说明里的同句（`:643`）、一句 problem 文字「现行账本还在追加，不封存」改成「是还没封存的那一行」。它们是门禁红了之后叫人怎么办的文字，留着旧句会在红的那一刻把人导回已废的做法；`.claude/rules/evidence-discipline.md:156`「回扫按旧说法搜，不只按新名字搜」支持这样做。现行仓里按这四个旧句 `grep -rnE '往现行账本追加\|现行账本还在追加\|新欠账登这份\|新内容往现行账本'` 在 `.claude/gate.d/*.sh`、`.claude/hooks/*.sh`、`scripts/kb` 里只剩守卫的名字与描述，不是叫人往里登（量的）。仍漏的是第 6 问第 15 条记的 `.claude/gate.d/stage-owners.tsv:143`。没有样本的 `expect` 写着被改掉的旧句（上面那个 grep 在 `.claude/gate.d/fixtures` 里零命中，量的） | 某份样本的 `expect` 要求旧句 |
| f `_lib_changed_paths_run_git` 成功时照打标准错误 | 偏得对；判决点名的层不够 | 判决第 7 条写的是 `load_change_range` 吞标准错误、「原来照打，改回照打」。HEAD 版 `gate_changed_paths` 直接起 git，标准错误当场照打；第一轮树里 `_lib_changed_paths_run_git` 把标准错误存进文件、成功时就丢，`load_change_range` 再把取到时存的那份当成空的。只改 `load_change_range` 打的是空文件，所以得在下层成功时也打：「成了照交它的标准输出，它往标准错误写的警告（例：重命名探测超上限）原样打到标准错误」（`scripts/gate/changed-paths.sh:159`）。这是把行为改回 HEAD 的样子，不是新行为。受影响的是六份调用方的输出里多出 git 的警告，归攻方腿第 2 问判样本档比对 | HEAD 版 `gate_changed_paths` 取到时不打 git 的警告 |

## 第 9 问：交用户那一条写得够不够让用户定

判：够不着。依据与两个选项都写了，但依据里的分类不实、数不闭合，代价与违例的后果没写；用户据此定的「31 处只读查表改掉」（`records/2026-11-17-subagent账本清欠调度记录.md:395`）边界比实际含糊。逐项：

| 项 | 判决那一行写了什么 | 我现查到的 | 判定 |
|---|---|---|---|
| 数 | 判决那行写「38 处 `static OnceLock`，只有 3 处明确是外部句柄；另 31 处是一次建好、只读的查表」（判决第 73 行） | 38 是对的：`grep -rnE 'static [A-Z_0-9]+ *:[^=;]*OnceLock' crates --include=*.rs` 得 38 行（量的，清单在 `~/.local/share/singlefs/work/open-fixes-governance-r2-defense/statics.txt`）。但 3 加 31 是 34：正推腿报告把另 4 处标「推不出」（卡租约、卡池、开卡记录），分类表第一轮报告第 127 行与 125 行；判决那行没提这 4 处，读的人看不出 38 去了哪 | 不闭合：缺 4 处 |
| 31 的定性 | 「一次建好、只读的查表」 | 正推腿的 31 是三类拼的：`GivenStatesEngine` 4 处、算出来的查表与摘要 11 处、缓存的一次性数据与测试夹具 16 处（`research/prompts/ledger-open-fixes-governance-r1-sonnet-output.md:127`，腿自己标「靠读声明与文档注释分的类，是推的」）。其中 `GivenStatesEngine` 那 4 处不是只读查表：类型文档写「几条崩溃注入的工作线程共用一个：库与账各一把锁」（`crates/singlefs-verification/src/crash_amplification.rs:27707`），字段里有「library: LibraryOpenInThisProcess,」（同文件第 27723 行）；声明在 `e156_allocation_basis_counts.rs:721`、`e168_delete_ring_window_borrow.rs:1368`、`e171_three_oracle_collusion.rs:2008`、`e171_three_oracle_collusion_after_the_oracle_fixes.rs:5556`（量的，类型在声明下一行）。例外句末写「建好之后还会被改的状态不在例外里」（`.claude/rules/code-discipline-by-review.md:54`），所以「放宽成建好之后只读的进程级值」也罩不到这 4 处，对它们两个选项不是放宽对改掉，只有改掉（或把「会被改」的那条也放宽，那是另一个选项）。另 16 处缓存与夹具是不是「只读」我没读到类型，腿也标了「推的」 | 不实：至少 4 处不是只读查表 |
| 代价 | 只写了「把这 31 处改掉」 | 没写改成什么。其中生产代码是 `crates/singlefs-core/src/checksum.rs:25`（`static TABLES: std::sync::OnceLock<[[u32; 256]; 8]>`）、同文件另一处、`crates/singlefs-checker/src/lib.rs:122`、`crates/singlefs-unit-graph/src/rust_items.rs:114` 与 `wgsl.rs:251`（量的，声明行见 statics.txt）；改它们要走三方与实现员（`.claude/agent-common.md` 「`crates/` 下全部代码（含测试代码）与治理文件的改动一律走三方」那一条），判决没给动几个包、几处生产代码、改成编译期常量表还是显式传参。没有这些，用户选「改掉」是在不知道量的情况下选 | 没写 |
| 违例的后果 | 「照现在的写法它们都违例」 | 这条规则只在 review 判，没有门禁（`.claude/rules/code-discipline-by-review.md` 文件名与首行「靠 review 的那一半」）；38 处今天都在库里、没有红。判决没说违例当前没有机器后果，用户读到「都违例」会以为有拦 | 没写 |
| 选项的写法 | 两个选项都有名字：放宽成「建好之后只读的进程级值」，还是照现在的写法把这 31 处改掉 | 选项本身清楚、互斥；缺的是第三种写法：先分类（查表、夹具、引擎），按类各定。这一点不能怪判决——用户当时手上就是这两个 | 清楚，但不全 |
| 依据的出处 | 「正推腿第 8.2 问；主 agent 现数 38」 | 出处写了；腿的分类没给复核（判决「这一轮被判过的文件」那一条只写了「射程罩不住 `crates/` 里 31 处只读查表，交用户」） | 够 |

后果：调度记录第 399 行写「派实现员改 `crates/` 里 31 处只读查表的 `OnceLock`」，实现员会在这 31 处里碰到 4 处带锁的引擎，边界要回头问；这是我从「判决那行把分类写成一类」推出的，实现员还没派，推的。我不替主 agent定怎么补，只给能落在判决那行上的三样：补上缺的 4 处、把 31 按腿的三类拆开写、写明违例今天只在 review 判。

推翻这一条的现象：用户在被问时拿到的不只是判决那一行（例如弹窗里另给了分类与代价）。我没看到弹窗原文，只看到判决那一行与调度记录里的定案。

## 要主 agent 跑的命令

没有别的复跑要求；判决要引的读数里，只有钟差没量过。草稿目录里有探针 `/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-defense/clock-skew-probe.sh`（sha256 `47b3c891dac248af069fb34136d17ebba3adaf7c6bd6f4836b0b99ec59d9b2ae`，`bash -n` 过）。它往共享根下一个已有目录写一个小文件，打一行 `machine=<本机 /etc/machine-id 的 sha256 前 8 位> skew_seconds=<本地钟减文件修改时间>`，不写主机名。服务选台，多交几趟直到两行 `machine` 不同；每趟都是独立任务，不串、不夹别的命令。

1. 钟差，一趟
   - 命令：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-defense/clock-skew-probe.sh /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-defense`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪一行：`TASK_SERVER_RESULT` 的 `exit=0`，`task-server/task log <号>` 里的 `machine=… skew_seconds=…` 一行。两趟的 `machine` 不同时，两行 `skew_seconds` 的绝对值都小于 285 就是第 7 问「钟差」那一格判决照旧站得住；任何一行的绝对值不小于 285，或两趟 `machine` 总是相同（服务没轮到另一台），那一格按「还没核」写、不当核过。读数没量过，现在是「推的」。
2. 钟差，第二趟：同一条命令再交一次，看另一台的行。

## 没做什么

- 不替主 agent 撤销或维持任何一条判决，只交复核结果。
- 没往统一任务服务交任务；钟差读数没量，上面所有「站得住」里依据主 agent 旧版探针读数的格，读数都是第一轮主 agent 量的、我没重跑；第一轮树上我只对文字与代码做了 `grep -nF` 与 `diff`。
- 第 8 问 a 的「不删版本在三趟交错下仍双占」是读代码推的，没造那一版去复跑。
- 没读 `completion-cleanup-guard.sh` 与 `project-code-guard.sh` 的实现，第 6 问第 8、9 条只认第一轮的探针读数。
- 第 9 问没看到弹窗原文。
- 清理：没建仓副本、没建编译目录。没删 `/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-defense/statics.txt`（38 行文本读数，第 9 问的数要用它核，几 KB）与 `clock-skew-probe.sh`（上面命令要用它，不到 1 KB）；认领的目录就是这个草稿目录。
