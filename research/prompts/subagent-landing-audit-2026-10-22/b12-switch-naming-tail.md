# 第十二批任务 51：弄坏开关点名逐格相同的收尾

日期：2026-10-24（开工时 2026-10-23）。复跑工具是兄弟任务 42 草稿目录里的 `tools/replay.py` 原样拷进本草稿目录 `tools/`（`cmp` 无差异；判法与门禁 code-tooling 的 break-switch-replay 格同一份，开关表库现读 code-tooling.sh），续跑表在本目录 `runs/`（`<sha256 前 16 位>-<开关>.json`；第二趟记成 `…#second.json`，由本目录 `tools/second.py` 起；`tools/judge2.py` 两趟各判一次）。开工时的「改前」判定从兄弟任务的续跑表按此刻 sha256 拷来（同一 sha256 沿用，复用规则照派发提示），存在 `judge-before.jsonl`。

## 结论

六条都做到了，复跑表在文末（`rerun-table.tsv` 同内容）。改前 20 份 204 条里对 62 条；改后同样这 20 份 206 条（别的会话这期间给 changed-paths.sh 加了两条开关）按最后一版 sha256 每条跑两趟：412 趟里 411 趟逐格相同，1 趟不同（detached-run.sh in-caller-tree 下一个计时格少红，另一趟相同，见第 5 条）。新登记进 `.claude/gate.d/break-switch-replay-files.tsv` 18 行。

推翻条件：门禁 break-switch-replay 格在提交前那一趟对这 20 份里任一份报「点名的项没在失败行里」「失败行没被点名」「设上之后自证照样退 0」，且单独重跑一次照样报（格自己会重跑一次）；或 `bash scripts/hosts/detached-run.sh --selftest` 不设开关在负载正常时退非 0。

还剩的、不归这一条的（详见「没做什么」）：detached-run.sh 的 Ⓡ、Ⓠ 报警、Ⓕ、Ⓔ 被 TERM 停、in-caller-tree 下的 Ⓥ 命令退出后登记写不进这几格是计时格，负载过百时偶发多红或少红，单独重跑一次就对；replay.sh 的「编译目录 build-directory-removed-when-stopped」「5 秒内退 143」一格同样在负载过百时偶发多红。

## 第 1 条：completion-cleanup-guard.sh 11 条、lib_prompt_scratch_line.py 4 条多红

只改文件头开关表：每条开关那一块末尾逐行补上实际多红的格（completion-cleanup-guard 照它自己的写法一行一个「✗ 自证 <格名>：」，lib_prompt_scratch_line 一行一个「✗ 自检：<格名>」），点名的格取兄弟任务两趟（`<开关>` 与 `<开关>#recheck`）都多红的那几行。completion-cleanup-guard 的自证开关轮读的是代码里的 `BREAK_TABLE`，不读文件头，文件头只是注释。

换上：一份钩子规格（两份都在 `.claude/hooks/` 这一层），`python3 scripts/gate/hook-swap.py specs/item1.json --hook .claude/hooks/completion-cleanup-guard.sh --feeds specs/cleanup-feeds.json --scratch <草稿目录>`（喂法两条：空对象放行、会话号带路径符号放行）：

```
✓ 草稿里验过：2 份文件语法检查、内嵌 python 1 段编译、喂了 2 条
✓ 换上了 2 份（改名换上新 inode、回读一致），主工作区里再验过：2 份文件语法检查、内嵌 python 1 段编译、喂了 2 条
```

复跑：34 条，两趟都对（`judge2-final.jsonl`）。红样本是每条开关设上之后的自证（点名的格红、退 1），绿样本是不设开关的基准趟（completion-cleanup-guard 17.49 秒退 0，lib_prompt_scratch_line 0.05 秒退 0）。

## 第 2 条：失败细节不以 ✗ 起头的六份

| 份 | 改了什么（打印） | 开关表 |
|---|---|---|
| `scripts/crash/check-segment-registry.py` | 逐条 `✗ <problem>`（`# gate-lint:detail`），末尾汇总 `✗ --selftest 没通过：N 项（上面逐条列着）` 接原来的 → | 三条改成「」括住 ✗ 起头的失败行；path-rows-to-product 补上它连带红的四格（含转述的「✗ 找不到 …replay.sh」） |
| `scripts/gate/gate-structure-check.py` | 逐条前缀 `     ` 换成 `  ✗ `（原来就标了 detail） | 没动（点名的「bad：…」照得出） |
| `scripts/gate/stage-mine.py` | 新函数 `selftest_red()`：`✗ selftest: <格>` 加一行 →，五处 `print('selftest: …'); return 1` 改调它 | 三条改成「✗ selftest: …」，by-row-takes-block 补连带的「✗ table.md：要写进索引的内容里…」 |
| `scripts/gate/changed-paths.sh` | `sed 's/^FAIL /      /'` 改成 `sed 's/^FAIL /  ✗ /'` | 全表改成「✗ <标签>：」，实测改用『』写；别的会话这期间加了三格与 LAG_BASELINE_BREAK 两条，quotepath、untracked、moved-kept 跟着补了点名 |
| `scripts/kb/kb-sections.py` | 新函数 `selftest_red()`，六处判错改调它；命令行那一格的两行改成 `✗ selftest: …` 加 → | 没动 |
| `scripts/kb/replace-once.py` | 同 check-segment-registry | 三条改成「」括住 ✗ 起头的失败行；no-clock-guard 列全九格，no-session-guard 写成逐格以「✗ 写会话号：」起头 |

六份都经 `scripts/gate/hook-swap.py` 的脚本规格换上（先打印、再开关表各一次），`--verify` 是各自的 `--selftest`，每次都是「验证命令退 0」；replace-once.py 两次换上之间与之后别的会话照常在用它，没有回滚。

复跑：六份 25 条（3、4、3、11、1、3）两趟都对。红样本：每条开关设上之后点名的 ✗ 行出现（例：`STAGE_MINE_BREAK=first-occurrence-anchor` 下「✗ selftest: 重复行：| MINE | 应当落在第二张表的 |---| 之后…」）；绿样本：基准趟退 0（check-segment-registry 0.25 秒、gate-structure-check 0.91 秒、stage-mine 0.18 秒、changed-paths 13.21 秒、kb-sections 0.86 秒、replace-once 0.23 秒）。

## 第 3 条：migrate-changelog-format.py 接不进复跑

查下来退 78 的不是 `inputs-changed`：这份的准入是 `admission: always`，拦它的是 `run-condition: single-instance`。兄弟任务的续跑表里开关趟的原样输出是「运行 single-instance（第 3 行）：同一个脚本已有 6 个实例在跑」——复跑并行起同一份的自证，彼此算别的实例。门禁 break-switch-replay 格也按至多 8 条并行起（`min(heavy_thread_default, 8)`），同样会撞上，所以改在脚本这一侧：

- 第 3 行 `# run-condition: single-instance` 换成 `# run-condition: check out="$(python3 .claude/singlefs-ai-sop/scripts/proc.py find python3 --argument=--write)" && ! grep -q migrate-changelog-format.py <<<"$out" :: …`。只挡另一个带 `--write` 的实例（`proc.py find` 不列发出命令的那一支，自己不算）；`--selftest` 只写临时目录，几份同时跑不再互相挡。`proc.py` 失败时 `&&` 短路，条件判不成立（拒，不放）。
- 第一次换上时写成 `--argument --write`，argparse 把 `--write` 当选项、退 2，`--verify` 退 78，hook-swap 按反向规格回滚（退出码 5）；改成 `--argument=--write` 再换上，验证命令退 0。
- 红样本：先起一个假的带 `--write` 的实例（`python3 -c 'import time; time.sleep(30)' scripts/kb/migrate-changelog-format.py --write`），再跑 `--selftest`：退 78，原样输出首段：

```
  ✗ scripts/kb/migrate-changelog-format.py：1 条条件没满足，拒绝执行（退出码 78）：
       - 运行 check（第 3 行）：判据不成立（退出码 1）：out="$(python3 .claude/singlefs-ai-sop/scripts/proc.py find python3 --argument=--write)" && ! grep -q migrate-changelog-format.py <<<"$out"
```

  （那个假实例用 `kill "$!"` 没停掉，`$!` 在这一次的 Bash 里不是它；随后照 `ps` 看清楚、`proc.py stop 729757` 停了它，`proc.py find` 复查 0 个。）绿样本：没有别的实例时 `--selftest` 退 0「✓ 自证：37 格都过」。
- 开关表：六条开关原来写的「另有 N 条跟着红」逐格补成「」点名的失败行。
- 复跑：10 条两趟都对，没有一趟退 78。

## 第 4 条：点名不带「」、格比不了的

| 份 | 条数 | 怎么改的 |
|---|---|---|
| `scripts/hosts/detached-run.sh` | 53 | 见第 5 条：第 5 条的代码改完、两趟复跑之后，用 `tools/drnames.py` 按两趟都红的失败行出规格：旧点名行能对上一行失败行的换成「✗ 自证格不对：<格名开头>」（进程号换成 …，旧行括号里的注照抄成『』）；块里已有「」项照得出的旧行留着当说明；对不上的旧行留着；没被照出来的失败行补在块末；照不出的旧「」项换成『』 |
| `scripts/experiment-apparatus/replay.sh` | 4 | quiesce-failure-ignored、cards-not-from-quiesced-list、run-lines-without-quiesce、not-run-not-itemized 四条在原格名后面加（「✗ 自检：…」），只动这四行（这份别的会话在改） |
| `scripts/gate/changed-paths.sh` | 1（fallback） | 随第 2 条整表改成「✗ <标签>：」 |
| `.claude/hooks/write-guard.sh` | 1 | 被认成开关表的是自证代码里一行注释 `# LIB_WRITER_LOCK_BREAK=takeover-sweeps-new-claims（…）下这两格红`，后面紧跟的代码行被当成点名的项。注释下面加两行「✗ 自检：写者锁:…」，再空一行隔开代码（只动这一处注释，钩子规格经 hook-swap 换上，喂法两条：写钩子目录新文件放行、实现员写 /etc/hosts 拒） |
| `scripts/crash/cpu-judging-sites-check.py`、`gpu-exemptions-check.py`、`crates-mutation-light-rows.py`、`scripts/kb/milestone-close-owed-check.py`、e197 的 `drive.py`、`report.py`、`scripts/crash/crash-flow-coverage.py` | 8、4、6、7、5、6、11 | 原来写的格名（`unmarked-site` 这类）就地换成「✗ 自证 <格名>：」（e197 与 crash-flow-coverage 是「✗ 自检 <格名>：」）；多红的补在块末（milestone-close-owed-check 的自证转述被测输出，里面以 ✗ 起头的行也补了；crash-flow-coverage 的汇总句带格名、每条不同，写成「✗ 自检 …格里有…格判错」） |
| `scripts/crash/crash-amplification-lib.sh` | 4 | 它的 `--selftest` 末尾开关轮自己读这张表（`crash_amplification_read_switch_table`：括号外的每一段算一格名）。第一次照上一行的写法换成「」，开关轮红（「文件头点名的是「「✗ 自证 rocksdb-entry-refused：」」」），hook-swap 退 5 回滚；改成格名后面括号里写失败行原文 `term-reaches-cargo（「✗ 自证 term-reaches-cargo：」）`，先用库里同一段读表 python 在改后文本上读一遍、读出的格名与原来相同，再换上，验证退 0 |

每份都是先拿 `tools/autoname.py` 从续跑记录现出项、逐份看过再落规格；`scripts/` 下经 hook-swap 脚本规格（`--verify` 各自的 `--selftest`），钩子经钩子规格。复跑：第 4 条这几份（不含 detached-run.sh）84 条一趟对，第二趟同样对；replay.sh 32 条在别的会话改过它之后重跑两趟都对。

## 第 5 条：detached-run.sh 另三样

**核数② 时红时不红。** 查到三层原因，都在自证这一侧：①单元跑的是一个进程里 31 条线程忙等，解释器锁在机器忙时把它们排成一条，量出来「同时在算的线程约 1.0 条」；②改成 31 个 `python3 busy-load 1` 进程之后，机器忙时一个个起解释器要好几秒，第一窗口里先起的算了好一阵、后起的刚开始，量到 7.0（阈值 7.75）；③忙等只跑 8 秒，watch 一轮要好几秒时单元在判之前就跑完，只打了「不满一个窗口」那一行。根子是这一格拿此刻量出来的核数判「不报」，机器被别的活占满时两个量都会掉到阈值以下（并行复跑里量到过 6.5 与 7.7）。改法（`scripts/hosts/detached-run.sh`）：
- 核数② 的单元改由一个 bash 一下子 fork 出登记数那么多个忙等的子 shell；三条单元都忙等到 watch 判过它（「<名> 在用 X 核（登记 N）：最近」那一行）再由自证 `stop` 停掉，三条都停了 watch 自己退 0（新函数 `detached_run_selftest_wait_for_judged`，按轮数等：列满 `CORE_JUDGE_ROUNDS=4` 轮还没判的不再等，弄坏开关 core-usage-unwatched 下就是这样；忙等 30 秒是上限）。
- 核数② 的判据改成「watch 照判器的结论打、照结论记」：判过一次、✗ 行数等于告警记录条数（格名「核数② 登记数那么多条线程忙等的单元：watch 打判过的那一行，判成过少的才打 ✗、写告警记录（✗ 几行告警就几条）」）。判器在机器忙时报不报由 `scripts/hosts/core_usage.py` 自己的自证拿造的样本序列核（它文件头 parallel-threads-ignored 那一格），这一格不再靠此刻的负载。核数①③（1 条线程）在用核数不过 1、同时在算不过 1，判成过少与负载无关，判据没动。
- 证据：53 条开关下核数② 多红（不算点名它的 in-caller-tree、core-usage-unwatched）的条数，兄弟任务那一版 9 条，本批中间几版 11、4、2 条，最后一版两趟 0 条（`dr-extra-*.txt`）。core-usage-unwatched 下核数①②③ 照样都红（两趟）。

**exit-dropped 过 200 秒。** 拿 `PS4='+${SECONDS}:${LINENO}: ' bash -x` 跟踪一趟（`dr-exit-dropped-trace.txt`）：等文件的循环那一行（`sleep 0.1`）累计 116 秒——九处 `detached_run_selftest_wait_for_file "<登记>/exit" 100|200` 在退出码永远不来时各空等满上限。新函数 `detached_run_selftest_wait_for_exit <登记目录> <上限>`：单元已不在跑、命令的进程也没了时再等 1 秒收尾、不等满上限；九处改调它。实测（同一份最后版本）：负载约 40 时 148 秒（同时跑的不设开关 67 秒），复跑里 134、148 秒；改前跟踪那一趟 231 秒到 `timeout 230` 还没收尾。剩下比基准趟多的几十秒在 `detached-run.sh stop` 自己等退出码（这个开关下永远等不到）与几格 watch 等结束，是被测代码在这个开关下的行为，没再动。

**watch-skips-unknown 开关下自证照样退 0。** 开关接上了，是格没判到：Ⓙ 的登记起跑秒数写的是 `$(date +%s)`，与 watch 开跑往往在同一秒，`detached_run_observe_started_since`（watch-misses-short-runs 那一处后加的）把不早于 watch 开跑的登记一律记成观察到的，绕过了这个开关弄坏的那一判。Ⓙ 与 Ⓠ 的登记起跑秒数都改成 `$(($(date +%s) - 5))`（注释写明为什么）。改后单跑：watch-skips-unknown 下红在 Ⓠ 报警与 Ⓙ 两格、退 1；unknown-alarm-missing 下只红 Ⓠ 报警；query-failure-as-gone 下红 Ⓠ 四格与 Ⓙ（`p7-*.txt`）。Ⓠ 报警这一格在改 Ⓠ 的起跑秒数之前也时红时不红（同一个同秒问题），一并修了，开关表那一条照实点名这两格。

开关表：第 5 条改完之后照第 4 条的办法整表换成「」点名（`specs/dr-names-final.json`，43 处，含 `CORE_JUDGE_ROUNDS` 从 8 降到 4），watch-skips-unknown 那一条最后照实测写成 Ⓠ 报警加 Ⓙ。复跑（最后一版 sha256 `7707070345a39080…`）：一趟 52 对、1 不同，二趟 53 对。不同的那一条是 in-caller-tree 下「Ⓥ 命令退出后登记写不进」那一格这一趟没红（前面四趟都红），门禁格判不对时会单独重跑一次。

## 第 6 条：登记进 break-switch-replay-files.tsv

经 `scripts/kb/replace-once.py` 一次接在表末（别的会话这期间在表里加了 resource_release_broadcast.py 一行，照留），每行 3 栏、秒数是正整数、指到的文件都在（核过 43 行、0 行不合）。限时照各份续跑记录里跑完的最慢一条乘二再往上取到 60、120、300、600：

```
scripts/crash/check-segment-registry.py	60	# 三条开关逐个跑过两趟：点名的格与实际红的格逐格相同（失败细节逐条以 ✗ 起头，末尾一句汇总）；单跑一条最慢 2 秒
scripts/gate/gate-structure-check.py	60	# 四条开关逐个跑过两趟：点名的格与实际红的格逐格相同（失败细节逐条以 ✗ 起头）；单跑一条最慢 4 秒
scripts/gate/stage-mine.py	60	# 三条开关逐个跑过两趟：点名的格与实际红的格逐格相同（判错的那一格以 ✗ 起头）；单跑一条最慢 3 秒
scripts/gate/changed-paths.sh	60	# 十一条开关逐个跑过两趟：点名的格与实际红的格逐格相同（失败细节逐条以 ✗ 起头）；单跑一条最慢 20 秒
scripts/kb/kb-sections.py	60	# 一条开关跑过两趟：点名的格与实际红的格逐格相同（判错的那一格以 ✗ 起头）；单跑一次不到 1 秒
scripts/kb/replace-once.py	60	# 三条开关逐个跑过两趟：点名的格与实际红的格逐格相同（失败细节逐条以 ✗ 起头，末尾一句汇总）；单跑一条最慢 6 秒
scripts/kb/migrate-changelog-format.py	60	# 十条开关逐个跑过两趟、并行起也不被准入拒（运行条件只挡另一个带 --write 的实例）：点名的格与实际红的格逐格相同；单跑一条最慢 11 秒
scripts/crash/cpu-judging-sites-check.py	60	# 八条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条最慢 3 秒
scripts/crash/crash-flow-coverage.py	60	# 十一条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条最慢 3 秒
scripts/crash/gpu-exemptions-check.py	60	# 四条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条不到 1 秒
scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py	60	# 五条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条不到 1 秒
scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py	60	# 六条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条不到 1 秒
scripts/kb/milestone-close-owed-check.py	60	# 七条开关逐个跑过两趟：点名的格与实际红的格逐格相同（自证转述的被测输出里以 ✗ 起头的行也点了名）；单跑一条不到 1 秒
scripts/mutation/crates-mutation-light-rows.py	60	# 六条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条不到 1 秒
scripts/crash/crash-amplification-lib.sh	120	# 四条开关逐个跑过两趟：点名的格与实际红的格逐格相同（它自己的开关轮读同一张表，括号里的失败行原文不读）；负载高时单跑一条最慢 49 秒
.claude/hooks/write-guard.sh	120	# 一条开关跑过两趟：点名的格与实际红的格逐格相同；负载高时单跑一次最慢 33 秒
scripts/experiment-apparatus/replay.sh	300	# 三十二条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条最慢 46 秒，负载过百时有一条过了 120 秒
scripts/hosts/detached-run.sh	600	# 五十三条开关逐个跑过：点名的格与实际红的格逐格相同（负载过百时 Ⓡ、Ⓠ 报警、Ⓕ、Ⓔ 被 TERM 停几格偶发多红，单独重跑一次不红）；负载 50 到 180 时单跑一条最慢 290 秒
```

新写的 `# gate-similar:` / `# hook-events:` 行：没有（没新建门禁阶段与钩子）。

## 改过的文件

仓内（都经 hook-swap 换上或 replace-once 定点改，改前原件在 `originals/originals/`，每次换上的反向规格在 `hook-swap/reverse-*.json`）：

`.claude/hooks/completion-cleanup-guard.sh`、`.claude/hooks/lib_prompt_scratch_line.py`、`.claude/hooks/write-guard.sh`（只动文件头或那一行注释）；`scripts/crash/check-segment-registry.py`、`scripts/gate/gate-structure-check.py`、`scripts/gate/stage-mine.py`、`scripts/gate/changed-paths.sh`、`scripts/kb/kb-sections.py`、`scripts/kb/replace-once.py`、`scripts/kb/migrate-changelog-format.py`、`scripts/hosts/detached-run.sh`（打印、自证与开关表）；`scripts/experiment-apparatus/replay.sh`、`scripts/crash/cpu-judging-sites-check.py`、`scripts/crash/crash-amplification-lib.sh`、`scripts/crash/crash-flow-coverage.py`、`scripts/crash/gpu-exemptions-check.py`、e197 的 `drive.py` 与 `report.py`、`scripts/kb/milestone-close-owed-check.py`、`scripts/mutation/crates-mutation-light-rows.py`（只动开关表）；`.claude/gate.d/break-switch-replay-files.tsv`（加 18 行）。

这批文件主 agent 这期间已经 `git add` 过（暂存区里含本批的改动），所以 `git diff --stat`（工作区对暂存区）只剩三份，原样：

```
 .claude/gate.d/break-switch-replay-files.tsv |  18 ++++
 scripts/crash/crash-amplification-lib.sh     |  19 +++-
 scripts/hosts/detached-run.sh                | 129 ++++++++++++++-------------
 3 files changed, 103 insertions(+), 63 deletions(-)
```

对开工时留的原件比（`git diff --no-index --numstat originals/originals/<文件> <文件>`，增 删 文件；changed-paths.sh、crash-amplification-lib.sh、replay.sh、break-switch-replay-files.tsv 的数含别的会话这期间的改动）：

```
19	0	.claude/gate.d/break-switch-replay-files.tsv
36	0	.claude/hooks/completion-cleanup-guard.sh
14	0	.claude/hooks/lib_prompt_scratch_line.py
4	1	.claude/hooks/write-guard.sh
8	6	scripts/crash/check-segment-registry.py
8	8	scripts/crash/cpu-judging-sites-check.py
20	7	scripts/crash/crash-amplification-lib.sh
28	11	scripts/crash/crash-flow-coverage.py
4	4	scripts/crash/gpu-exemptions-check.py
5	5	scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py
9	6	scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py
16	16	scripts/experiment-apparatus/replay.sh
67	31	scripts/gate/changed-paths.sh
1	1	scripts/gate/gate-structure-check.py
17	9	scripts/gate/stage-mine.py
146	82	scripts/hosts/detached-run.sh
16	8	scripts/kb/kb-sections.py
40	1	scripts/kb/migrate-changelog-format.py
26	7	scripts/kb/milestone-close-owed-check.py
11	6	scripts/kb/replace-once.py
6	6	scripts/mutation/crates-mutation-light-rows.py
```

最后一版 sha256：

```
f363326498d34890d09fd190604f2247e8f54f2994036c3eecddc4c0a95ae0bf  .claude/hooks/completion-cleanup-guard.sh
eb0ecabace851d72c439e810a3b786bf4b7baa40db190629c0c1b8f694dca165  .claude/hooks/lib_prompt_scratch_line.py
52f8b43a8d6a2dcd8e0d4b9f1392cc3a769efc65d05b17a45d03907a525670c6  .claude/hooks/write-guard.sh
a57f561a7e165079f28334e50ad6fb3881abf114cc040e4baf8876856a1d65e0  scripts/crash/check-segment-registry.py
f826baba0a09976d76ec69771feff1ae76701b539736432676e74e5cb9735996  scripts/crash/cpu-judging-sites-check.py
a8894600f290044015fa52eea52db2d01d7e0a7dd41fb037b8242967ab5132be  scripts/crash/crash-amplification-lib.sh
5bc968b4be9d78a6e00b29d89ebf121453a44848c9d0fd06582c1615a653b0f1  scripts/crash/crash-flow-coverage.py
134129138d9610b6fdbbf42daf915e9389040ad31afe560e5d20da36478f77b6  scripts/crash/gpu-exemptions-check.py
163250b33241fe2997f6634574a4f7f9b0fffe112a1c94616d0224167cd53e26  scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py
0a1eb51aac3418f0dc0784e05a8ff3be0c15b7ec5d102bdaadec628393415bc2  scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py
ab8007103c090ba8ccaec75a8d6c582025ef4f2ae2c8ece43857ba994144720f  scripts/experiment-apparatus/replay.sh
5c326e352647ad2ff76d461fc32e36983c5af7c0a68ac59f2ea5b09173a40a38  scripts/gate/changed-paths.sh
0f5d2abea5ecd1734e54ce47711070e2afebaeb552e5087f6ba055655e3efc69  scripts/gate/gate-structure-check.py
a5777e755ee6fbcdad23848a8fbea46a522b9e12ba13d966e44c12fa8e46b332  scripts/gate/stage-mine.py
7707070345a39080419cc5ca8d18137cda4958237a181c4e26caff7443236341  scripts/hosts/detached-run.sh
5c4e2c1257d0f3a9cc8e41e920c2267fcf3622c3961704b5730217dc8fa3c046  scripts/kb/kb-sections.py
f5f8323fc6406c928ab66f7818461c0c3f43a56159d3795fac9731666abea994  scripts/kb/migrate-changelog-format.py
e8b2132c5952e47844d63166a980921fc6affd4128c986c640a519d1398c9850  scripts/kb/milestone-close-owed-check.py
24f4b47a1ab22e4dfca9a90e924885c68c9e5a0dc6a67a09db7014fda0b51a5f  scripts/kb/replace-once.py
ce070bfda3f3007fa73dee40689595cbf59571834dc5074e3043ac8d7d1d1a87  scripts/mutation/crates-mutation-light-rows.py
```

## 要主 agent 跑的命令

工作目录仓根 `/home/fy5090/code/singlefs`。不起 cargo，内存上限不涉及。

1. 提交前那一趟门禁里 code-tooling 的 break-switch-replay 格会按新登记的 18 行复跑（detached-run.sh 53 条单条最慢约 3 分钟，并行 8 时这一格多十来分钟）。要单看这一格：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格; 范围=1 格; 为什么=核新登记的 18 份逐格相同' bash .claude/gate.d/code-tooling.sh --check break-switch-replay "$PWD"`
   看末行「复跑了 N 条、M 份」与有没有「✗」；负载过百时 detached-run.sh 的计时格可能红一次，格会单独重跑一次再定。
2. 命名检查不用跑：这一批没改 `.rs`。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没跑门禁阶段与各类 lint；没走定义三方；没提交。
- 各份的 `--selftest` 都是登记成 light 的，每份的基准趟就是不设开关的 `--selftest`，都退 0；语法检查用 `bash -n` 与 `ast.parse`（没用 py_compile，免得在仓里写 `__pycache__`）全过。
- detached-run.sh 里与负载有关、这一条没点名要修的计时格没动：Ⓡ 同名重起（watch 每 8 秒一轮、限时 90）、Ⓠ 连续 3 次查不到报警（查不到状态那一格在 query-failure-as-gone 下）、Ⓕ watch 全部结束（限时 60）、Ⓔ watch 被 TERM 停、in-caller-tree 下 Ⓥ 命令退出后登记写不进。负载过百时在某条开关下偶发多红或少红，单独重跑一次就对（`recheck-dr-final.txt`）。exit-dropped 下 `detached-run.sh stop` 等退出码等满自己的上限，也没动。
- replay.sh 的「编译目录 build-directory-removed-when-stopped：…应当 5 秒内退 143」一格在负载过百时在 build-output-dropped 下多红过一次、gz-read-raw 一条过了 120 秒；这份别的会话在改，只动了点名那四行，登记限时取 300。
- 不碰、只报的四份（multi-host-run.sh、resource_release_broadcast.py、verus-proof-check.py、run-with-memory-cap.sh）与慢的七份没跑、没改。
- 复跑多在负载 50 到 180 时跑（别的会话在跑 checker 档的 cargo test 与 qemu 样本档），`--jobs` 多数是 8，负载过百时降到 2 到 7；有一次在负载 100 时用了 9。
- 草稿目录里没有仓副本、编译目录与库拷贝（`du -sh` 50M，大头是 `runs/` 续跑表），留着给接着跑的人；本批起的 systemd 用户单元收尾后核过 0 个（`systemctl --user list-units --all 'singlefs-detached-selftest-*'` 无输出）。

## 复跑表

脚本、开关、改前（开工时那一版，兄弟任务的续跑记录）、改后一趟、改后二趟（最后一版 sha256 见上）。「改后新加的开关」是别的会话这期间加的。detached-run.sh 改后两趟之外，前面两版各另有两趟，判法相同。

| 脚本 | 开关 | 改前 | 改后一趟 | 改后二趟 |
|---|---|---|---|---|
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=all-sessions` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=attribution-by-window` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=background-ignored` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=build-directory-declarable` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=build-directory-unrecognized` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=creator-alive-ignored` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=deliverables-ignored` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=guard-off` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=handover-exempt-for-main` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=ledger-ignored` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=library-copy-unrecognized` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=notifications-ignored` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=open-files-ignored` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=prompt-scratch-ignored` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=prompt-scratch-relative-by-cwd` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=release-never` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=release-on-any-continuation` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=reminder-counts-as-running` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=repository-copy-unrecognized` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=run-holders-leaf-only` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=run-windows-ignored` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=scratchpad-ignored` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=size-omitted` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=subagent-run-session-level` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=subagent-sees-whole-session` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=subagents-ignored` | 相同 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=unjudged-pass-silent` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=unjudged-skip-silent` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=code-blocks-read` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=fence-info-with-backtick` | 相同 | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=relative-by-cwd` | 相同 | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=roots-unfiltered` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=unclosed-fence-read` | 不同（不同） | 相同 | 相同 |
| `.claude/hooks/write-guard.sh` | `LIB_WRITER_LOCK_BREAK=takeover-sweeps-new-claims` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/check-segment-registry.py` | `CHECK_SEGMENT_REGISTRY_BREAK=accept-extra-path` | 不同（不同） | 相同 | 相同 |
| `scripts/crash/check-segment-registry.py` | `CHECK_SEGMENT_REGISTRY_BREAK=path-rows-to-product` | 不同（不同） | 相同 | 相同 |
| `scripts/crash/check-segment-registry.py` | `CHECK_SEGMENT_REGISTRY_BREAK=product-required` | 相同 | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=judge-self-test-covers-whole-file` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=judge-self-tests-not-followed` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=judging-bodies-not-followed` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=local-function-of-the-same-name-followed` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=marker-anywhere-in-file` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=receiver-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=table-targets-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=unmarked-sites-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=rocksdb-entry-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=term-not-forwarded` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=term-not-waited` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=term-trap-not-restored` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=any-expansion-covers` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=block-count-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=empty-reason-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=entry-absence-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=ignored-growth-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=incomplete-judged-as-complete` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=reuse-without-recall` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=step-kind-coverage-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=unregistered-entry-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=unseeded-marker-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=zero-growth-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=marker-column-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=row-fields-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=unmarked-rows-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=unregistered-markers-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=cap-not-raised` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=lost-cell-dropped` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=missing-peak-as-zero` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=s1-always-same` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=segment-not-skipped` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=digest-mismatch-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=empty-cell-not-subtracted` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=line-ignores-the-sign` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=median-replaced-by-minimum` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=pc-red-threshold-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=slope-uses-the-top-point-only` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=archived-not-itemized` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-directory-kept` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-everything` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-log-single` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-output-dropped` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=cards-not-from-quiesced-list` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=copy-build-directories-kept` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=copy-lines-without-registry` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=driver-build-output-dropped` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=emitted-unchecked` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=exit-code-ignored` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=gpu-features-dropped` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=gpu-without-quiesce` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=guard-off` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=gz-read-raw` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=inputs-not-checked` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=jobs-from-nproc` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=load-errors-unchecked` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=load-field-ignored` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=not-run-as-broken` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=not-run-not-itemized` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=previous-output-kept` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=quiesce-failure-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=reuse-ignored` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=reuse-key-by-row-only` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=row-key-by-experiment` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=run-lines-without-features` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=run-lines-without-quiesce` | 不同（没点名） | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=term-not-forwarded` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=thread-registry-not-copied` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=unknown-experiment-accepted` | 相同 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REUSE_CHECKS_BREAK=rejudged-sentinel-passes` | 相同 | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LAG_BASELINE_BREAK=slash-prefix-rewritten` | （改后新加的开关） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LAG_BASELINE_BREAK=window-ignored` | （改后新加的开关） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=binary` | 相同 | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=empty-merge-base` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=fallback` | 不同（没点名） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=header-anywhere` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=moved-kept` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=quotepath` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=swallow` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=untracked` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=untracked-lines` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=heavy-ignored` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=list-ignored` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=table-ignored` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=unknown-ignored` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/stage-mine.py` | `STAGE_MINE_BREAK=by-row-takes-block` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/stage-mine.py` | `STAGE_MINE_BREAK=first-occurrence-anchor` | 不同（不同） | 相同 | 相同 |
| `scripts/gate/stage-mine.py` | `STAGE_MINE_BREAK=write-index-unchecked` | 不同（不同） | 相同 | 相同 |

| 脚本 | 开关 | 改前 | 改后一趟 | 改后二趟 |
|---|---|---|---|---|
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=archive-not-restored` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=assignment-name-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=break-honored-anywhere` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=broadcast-exit-swallowed` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=broadcast-none` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=broadcast-test-world` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=core-usage-unwatched` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=default-stop-timeout-unset` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=environment-dropped` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=exit-dropped` | 不同（超时） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=in-caller-tree` | 相同 | 不同（不同） | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=linger-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=log-unchecked` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=long-run-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=observe-overwrites-unit` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=path-home-dropped` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=query-check-after-archive` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=query-failure-as-gone` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=ran-without-pid-trusted` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=record-write-failure-as-vanished` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=same-label-allowed` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=same-second-restart-allowed` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=scopes-left-running` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=serial-declarations-unset` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=start-failed-trusted` | 不同（不同） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=start-failure-not-rechecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=start-failure-unmarked` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=stop-kill-unrecorded` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=stop-nonblocking` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=stop-timeout-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=unknown-alarm-missing` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=unregistered-units-hidden` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=wall-clock-optional` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-keys-by-path` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-misses-short-runs` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-skips-unknown` | 不同（没红） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-in-trap` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-units` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-units-on-hup` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-units-on-int` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-trusts-registry` | 相同 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=exempt-sha256-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=header-live-reading` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=hosts-canonical-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=hosts-suffix-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=hosts-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=live-exempt-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=partial-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=script-unresolved-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=two-logs-not-unwrapped` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=unshardable-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=wrappers-not-skipped` | 不同（没点名） | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_SHELL_WORDS_BREAK=detached-run-force-kept` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/kb-sections.py` | `KB_SECTIONS_BREAK=raw-arguments` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=abort-on-conflict` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=content-lenient` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=copies-ignored` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=fill-skip-one` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=fill-touch-existing` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=fold-single-dates` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=no-recheck` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=ordinal-always` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=untouched-off` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=write-order-file` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=file-form-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=filename-dates-counted` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=item-suffix-kept` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=paid-heading-optional` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=paid-rows-counted` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=status-ignored` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=whole-row-scanned` | 不同（没点名） | 相同 | 相同 |
| `scripts/kb/replace-once.py` | `REPLACE_ONCE_BREAK=no-clock-guard` | 相同 | 相同 | 相同 |
| `scripts/kb/replace-once.py` | `REPLACE_ONCE_BREAK=no-nearest` | 不同（不同） | 相同 | 相同 |
| `scripts/kb/replace-once.py` | `REPLACE_ONCE_BREAK=no-session-guard` | 相同 | 相同 | 相同 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=duplicate-targets-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=missing-targets-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-rows-accepted` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-targets-required` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=target-column-unchecked` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=unknown-rows-accepted` | 不同（没点名） | 相同 | 相同 |
