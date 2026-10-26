# 第十二批任务 54：五份慢自证照复跑记录补开关表

日期：2026-10-24（开工时 2026-10-23）。草稿目录 `~/.local/share/singlefs/work/landing-audit-b12-switch-naming-final`（下称 F），续跑表与判法沿用兄弟任务 52 的 `~/.local/share/singlefs/work/landing-audit-b12-switch-naming-slow`（下称 S）：`S/tools/judge2.py`、`stable.py`、`replay.py`，没改它们，也没往 S 里写东西。

## 结论

- 开工核基线：五份的 sha256 与续跑表里那一版逐个相同（67645496…、948064e3…、18437ced…、7d530d22…、4ce4d097…）；`judge2.py` 判出的读数与派发表逐格相同。五份在 git 里都是「已暂存、工作区没有别的改动」（`M `），没有别的会话没暂存的改动要照留。
- 第 1 条：五份的开关表都按两趟记录补齐了点名。改后拿原来那两趟记录按新表判（`F/tools/build.py` 模拟，判法就是 judge2 / replay.py 的 `judge_entry`，同一套汇总行算法）：
  | 脚本 | 一趟 | #second | 还剩的（只一趟有的，没补点名，见「只一趟有的差异」） |
  |---|---|---|---|
  | agent-watch.py | 对 122 | 对 121、不同 1 | backgroundwait（#second） |
  | crash-amplification.sh | 对 50 | 对 50 | 无；开关轮读表改前改后 50 条逐条相同 |
  | crash-point-replay.sh | 对 41 | 对 41 | 无；开关轮读表改前改后 41 条逐条相同 |
  | broad-mutation.py | 对 73、不同 3、没红 1 | 对 76、没红 1 | named-items-always-rerun、select-by-commit、gpu-carry-ignored（一趟）；没红的那一条是第 2 条，旧记录上照旧没红 |
  | mutation-shard-run.sh | 对 16 | 对 14、不同 2 | peer-library-kept、break-honored-anywhere（#second） |
- 只一趟有的差异七条，判下来都不是点名不全，是时序或输出截窗随环境变：列在下面，没补点名。
- 第 2 条：`BROAD_MUTATION_BREAK=term-leaves-step-running` 两趟没红，是**格没判到**：开关接上了（`Runner.stop_step` 里不找 scope、只给包装发 TERM），但 selftest_stop_cells 用的是真包装，而真包装被 TERM 时自己停 scope，开关弄坏的那一处被包装兜住了。改法：selftest_stop_cells 里另起一趟，给包装设它自己的弄坏开关 `RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope`（改前的包装：被 TERM 不停 scope），照包装文件头「弄坏开关只在自证里认」建标记文件带过去。实测：只跑这一组格（`BROAD_MUTATION_SELFTEST_CELLS=selftest_stop_cells`）不设开关 3 格都对（hook-swap 的 `--verify`）；设这条开关 23 秒退 1，红在新格「✗ 自证：包装自己不停 scope 时被 TERM 停，在途那一步没有一起停」，拿新表判是「对」。判法单元指纹两把（CPU 趟、GPU 趟）改前改后相同，库里的判定照样沿用。
- 第 3 条：五份都经 `scripts/gate/hook-swap.py` 脚本规格换上（broad-mutation.py 两次），六次都退 0、回读一致。broad-mutation.py 的 `--verify` 跑的是 stop 那一组格；整份 `--selftest` 在 235 秒限时里没跑完（负载 20 到 39），交主 agent。agent-watch.py 换上之后单跑不设开关的 `--selftest` 176 秒退 0。另三份基准趟 535 到 895 秒，前台跑不了，交主 agent。
- 第 4 条：两趟复跑的命令、判法与登记行写在「要主 agent 跑的命令」一节，我没跑。

推翻条件：主 agent 照那一节跑完两趟之后，`judge2.py` 对这五份报出「不同」，而那一条不在「只一趟有的差异」列的七条里；或 crash-amplification.sh、crash-point-replay.sh 的整份 `--selftest`（含开关轮）不设开关退非 0。

## 第 1 条：逐份补了什么

点名的写法照各份文件头原来的样子：agent-watch.py、broad-mutation.py 是「」括住失败行开头；mutation-shard-run.sh 是一格一行、括号里「」括住细节；crash-amplification.sh、crash-point-replay.sh 的自证末尾开关轮从表里读格名（括号外的每一段算一格），失败行原文只写进括号里，照兄弟任务 51 对 crash-amplification-lib.sh 的写法。两趟都少红的项：能对上实际失败行的改成对得上的写法（中间随跑变的一段写成「…」），对不上的改成『』留作说明；两趟都多红的行补成项。

- **agent-watch.py**（3 条）：finished 补 selftest_kept_copies 两项、selftest_scratch_hogs 一项；pipeupstreamalways、stale 各补 selftest_reminder_timer 两项（stop、other 两个假脚本底下的 sleep 照报「进程无输出」）、selftest_unacknowledged_terminations 一项（看门狗真跑三次）。
- **crash-amplification.sh**（13 条）：第 34、40、45、46 格括号里的句首原来夹着失败行里没有的连写（实际失败行在中间插了一段括注），改成「…」隔开：「一行 exhaustive=false 只因登记的段…这一项却判了红」「登记段分片枚举…判了红、或 ITEM 行没带…或成功句没报全展开与分片已判几态」「全部复用的一趟…判了红」「分项驱动报判段没跑…的那一项没记本次未跑」；swallow-exit、ignore-red-count、item-78-as-failure、condition-before-red 四条第 22 格的括号里补「✗ 点名的卡用不了（假驱动）」（那一格失败行转述的假驱动输出）。括号外一个字没动，开关轮读表逐条相同。
- **crash-point-replay.sh**（18 条）：多红的那几格在格名后面的括号里补「✗ 自证 <格名>：」（local-only-keeps-gpu-drops-peer-cards 七处，fast-tier-red-stops-heavy、fast-tier-slice-states-sent、local-only、pipeline-entry-* 三格、full-writes-back-into-the-one-library、local-only-baseline-sees-peer、fast-tier-two-hosts-configuration-refused、fast-tier-two-hosts）；list-items-with-keys 的旧句首换成此刻的「--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo」；第二台连不上那一格的句首补上「写原因、」。另改了一处**失败行的写法**：`run_target_check run-target-green 0 正常退 0` 与 `run-target-red 1 快档红退 1` 的第三个参数没加引号，失败行里只剩「正常退」「快档红退」、少了退出码，表里点名的「（正常退 0）」「（快档红退 1）」对不上；两处加了引号（模拟时把旧记录里那两句照改后的写法换了再判）。
- **broad-mutation.py**（27 条，另第 2 条一条）：list-items-pipe-unhandled 的点名原来被拆成两项（「✗ 自证：--list-items 接 」「✗ 自证： head 时没照…」），并回一项「✗ 自证：--list-items 接 | head 时没照 SIGPIPE 惯例安静退」；stage-two-full-cargo-test、slow-step-keeps-old-name、doc-tests-skipped 三条里各两项「✗ 自证： (binary_id(=singlefs-format::ignored_ca」是截坏的残句（`git log -S` 查到它随一次整批提交进来，哪一行失败行都照不出），删了；rocksdb-entry-unchecked 说明里的「# rocksdb-prebuilt-entry: selftest-fake-cargo」「entry」不是失败行，改成『』；另有 25 条（含删残句的那三条）在块末各补一行「另红：」，补的是失败行里转述的驱动输出（「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」「✗ 复用失灵：…键相同却现判了」「✗ 汇总对不上：…」「✗ 第 3 条的 one-test 退 104（认不出的退出码）：不落行」「✗ 基线 two-test-checker-tier 在没改过的树上退 101：基线不绿」这类）与没点到的格（reuse-sentinel、reuse-three-questions、两台各跑一片的 GPU 导入格、第二台的 GPU 判定行随导入那一格……）。
- **mutation-shard-run.sh**（16 条）：import-skipped 原来「没点名」是因为「（「本机「」里有一行判了两次：…）」少一个」，后面的项全被吞成一项；这一句与两趟都少红的另 18 项改成『』。16 条每一条都补一行「① 经 checker-tier-crates-mutation-replay 跑的那几格先撞上门禁共用库『弄坏开关只在样本根上认』那一判、退 2（「✗ ① checker-tier-crates-mutation-replay 接驱动那一趟判绿，判定与单机那一趟逐字相同：退 2；diff：」）」；anchor-broken-dispatched 另补 ⑭ 那一行。

## 只一趟有的差异（没补点名）

逐条看了两趟的失败行原文（`F/tools/show.py <路径> <开关>`）：

| 脚本 | 开关 | 哪一趟 | 多出的失败行 | 判 |
|---|---|---|---|---|
| agent-watch.py | backgroundwait | #second | 「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明，实际 3」 | 负载下的时序。selftest_session_watch_in 里 progressstale 那份会话记录最后一条写在 9.8 分钟前，无动静的线是 10 分钟（`--idle-minutes` 默认），写完 12 秒之后就过线；这条开关下等后台任务的也报无动静，在盯的那个看门狗第一次整查要是过了这 12 秒就当场退 3、不建就绪文件，第二个看门狗拿到锁、照样退 3。第一趟 156.88 秒没过线，#second 174.51 秒过了 |
| broad-mutation.py | named-items-always-rerun | 一趟 | 三行「✗ 复用失灵：N 键相同却现判了（…种子 …）」 | 失败行转述的输出截窗随环境变。格的失败行转述驱动 stderr 的末尾几百字（`stderr[-300:]`、`[-500:]`），「✗ 复用失灵」那一行正落在截窗的边上：本机那一趟截进来了，第二台那一趟（临时目录路径更长）截在它后面一行「→ 怎么办」。基准趟一行都没有 |
| broad-mutation.py | select-by-commit | 一趟 | 「✗ 复用失灵：6 键相同却现判了」 | 同上：#second 那一趟同一格的转述从紧跟它的「→ 怎么办」起 |
| broad-mutation.py | gpu-carry-ignored | 一趟 | 「✗ 复用失灵：gpu:5 键相同却现判了（…种子 2441428078…）」 | 同上 |
| mutation-shard-run.sh | peer-library-kept | #second | 「✗ ⑫ 场景都跑在自证自己的 tmp/ 下：…外面多出来的「…/singlefs-mutation-replay-…」」 | 复跑装置的环境：第二台那一段是一个内存包装底下并行起几份自证，几份共用包装的那一个 TMPDIR，⑫ 数到的是同时在跑的另一份自证建的目录。本机那一趟没有 |
| mutation-shard-run.sh | break-honored-anywhere | #second | 汇总句里多出「另本次未跑 1 格：⑥ … 120 秒内假 cargo 没起来」 | 负载下的时序（⑥ 等假 cargo 起来限 120 秒） |

另：broad-mutation.py 的 rerun-done、no-carry-without-arguments、gpu-carry-ignored、ignored-scan-skips-nested-modules 里 stable.py 列成「只一趟多红」的「第二台的 GPU 判定行没随导入…」「两台各跑一片的 GPU 导入格前提不成立」，两趟其实都红了这一格，只是转述的尾巴不同（stable.py 按前 90 字认同一行），照两趟都有补了点名。

这几条在复跑里还可能再「不同」一次。要根治：agent-watch 那一格把 progressstale 的最后一条往前挪到离线足够远（或给看门狗传 `--idle-minutes`）；broad-mutation.py 的格不再整段转述带 ✗ 的驱动输出（转述前把「✗」换掉，或只转述到第一行 ✗ 之前）；mutation-shard-run.sh 的复跑每份自证给自己的 TMPDIR。都不在这一批「只改开关表与失败行写法」的范围里，没做。

## 第 2 条：term-leaves-step-running

- 查：`Runner.stop_step` 读这条开关（`own_scope_unit` 不调、只 `child.terminate()` 再等宽限），开关接上了；自证里被 TERM 停那一格（selftest_stop_cells）经真的 `scripts/hosts/run-with-memory-cap.sh` 起假一步，包装自己接 TERM、停它开的 scope（它文件头「被停」一段），所以只给包装发 TERM 也停得下，格判不到。是格没判到，文件头原来写的就是「此刻不让任何格红」。
- 改（只动 selftest_stop_cells 与文件头那一条，判法单元的闭包里一个函数都没动）：原来那几格之后另起一趟，假一步照样经真包装，环境里加 `RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope` 与 `RUN_WITH_MEMORY_CAP_SELFTEST_MARK=<工作目录>/memory-cap-selftest-mark`（标记文件头一行写包装的 `SELFTEST_MARK_CONTENT`，照 mutate.sh 的写法）；驱动起来、假一步挂住之后只给驱动发 TERM（自己起的进程号），判驱动退 143、假一步那两个进程都停了。判完写 release 文件，开关下停不下的那两个进程见到它就退，不留到收尾。库目录另起一个 `stop-scope-kept`。
- 文件头那一条改成「红：「✗ 自证：包装自己不停 scope 时被 TERM 停，在途那一步没有一起停」、「✗ 被 TERM / INT 停了：在途那一条没落行」」，后一项是那一格转述的驱动 stderr 末尾（驱动被停时必打的一行）。
- 判法单元指纹（`F/tools/judgekey.py`：两棵只放 broad-mutation.py、mutation-prune.py、lib_libtest_result_lines.py、lib_function_units.py 的小根，一棵改前一棵改后）：

```
…/keyroot/before 289af6ae6a4bdc2dda4d65bada343fd59ad822e504e6dfcb3c9eed15499cd595 f651d9783077e141f1dcc3e271b1d5113500daec09de13ff57671519784e35a8
…/keyroot/after 289af6ae6a4bdc2dda4d65bada343fd59ad822e504e6dfcb3c9eed15499cd595 f651d9783077e141f1dcc3e271b1d5113500daec09de13ff57671519784e35a8
```

- 绿样本：hook-swap 的 `--verify`（`BROAD_MUTATION_SELFTEST_CELLS=selftest_stop_cells nice -n 19 python3 -B scripts/mutation/broad-mutation.py --selftest`）退 0，3 格都对，0 行失败行。
- 红样本：同一条命令加 `BROAD_MUTATION_BREAK=term-leaves-step-running`，23 秒退 1（`F/bm-stop-red.txt`），失败行 3 行：

```
  ✗ 自证：包装自己不停 scope 时被 TERM 停，在途那一步没有一起停（包装带 RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope，改前的包装；那一步挂住了；驱动退 143；那一步的进程 [262856, 262865]、还有活着的）：u45l/stop-scope-kept/runs/run-261044-yryqjo6f/tree）开着工作目录
  ✗ 被 TERM / INT 停了：在途那一条没落行，在途那一步已一起停掉
  ✗ --selftest：3 格里 1 格判错
```

  拿新表与 replay.py 的 `judge_entry` 判这一份（基准是上面那一趟绿的）：`('对', [], [])`。跑完回查：那两个进程号已不在，`systemctl --user list-units 'singlefs_broad_mutation_selftest_*'` 没有残留的 slice。
- 整份 `--selftest`（123 格加这一格）不设开关在 `timeout 235` 下没跑完（退 124，负载 20 到 39）；它留下的 `/tmp/broad-mutation-selftest.cn90e_sr` 我删了。整份的两趟交主 agent（下一节第 1 条里本机那一段就含它）。

## 第 3 条：换上

六次都是脚本规格（不带 `--hook`、`--feeds`），规格与反向规格在 `F/specs/*.json`、`F/swap-<名>/hook-swap/reverse-*.json`（每个目录里时刻最晚的那一份是真换上的那一次，早的那一份是 `--dry-run` 留的）。原件留在 `F/originals/`（`scripts/agents/copy-originals.py`）。换上的输出原样：

```
  scripts/mutation/broad-mutation.py 换上之前 sha256 7d530d22aa7be0225f96fdccd5d734e2b7cef5eb9c7d1b5e5b647ce5916f35a1，换上之后应当是 19c05aaaeb91ca4a9424b325246b22e48dece9470e3889d73cc70477c4dde364
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 0 段编译、验证命令退 0（BROAD_MUTATION_SELFTEST_CELLS=selftest_stop_cells nice -n 19 python3 -B scripts/）
  scripts/mutation/broad-mutation.py 换上之前 sha256 19c05aaaeb91ca4a9424b325246b22e48dece9470e3889d73cc70477c4dde364，换上之后应当是 49273352a99610cea9ae89a8cd1c689d5201ea89a0bbfb67cf4dd2168f551b0f
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 0 段编译
  scripts/agents/agent-watch.py 换上之前 sha256 67645496e832ac1d272e1d295c192522e44b49a00fcaef410bd555251754fb5d，换上之后应当是 86ffa88eb97fc9c51b1ecd07b6897a37dcbf636496a6bc6682da8b85eedd0fa3
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 0 段编译
  scripts/crash/crash-amplification.sh 换上之前 sha256 948064e3e96efd3fe0e88cc3d5f59341587a7d2a5215d551b07a37057764dc82，换上之后应当是 265fe635a2d29699e03c24854d902314809e9dc3d476e5535da7fa900d5eb672
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 0 段编译
  scripts/crash/crash-point-replay.sh 换上之前 sha256 18437ced3fcbaf020356baef361649430cb35a8614ad1c490717f5e5f2bb3ff3，换上之后应当是 5bf17efea61cd79d487f368e4d65469db09044b04d0b389b93e0085a1aacc6e6
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 3 段编译
  scripts/mutation/mutation-shard-run.sh 换上之前 sha256 4ce4d0970182b442be740d8d4ec7c2bb960b05092a4d73651a23e9c2b08e2c67，换上之后应当是 25c2b7eaa497d7fa2eb26f92700c736cf775c16aaad11ebf47e6bf751a5a102a
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 2 段编译
```

换上之后回读：agent-watch.py、crash-amplification.sh、crash-point-replay.sh、mutation-shard-run.sh 与 `build.py` 出的改后预览逐字节相同（`cmp`）；`bash -n` 三份 .sh、`ast.parse` 两份 .py 都过（没用 py_compile，免得写 `__pycache__`）。

换上之后跑的自证：

| 脚本 | 跑了什么 | 结果 |
|---|---|---|
| agent-watch.py | 不设开关 `timeout 235 nice -n 19 python3 -B scripts/agents/agent-watch.py --selftest` | 176 秒退 0（`F/aw-full-green.txt`，唯一一处 ✗ 在成功句的文字里） |
| agent-watch.py | `AGENT_WATCH_BREAK=stale` 同上 | 235 秒限时到了没跑完（记录里这一条要 270 秒），留下的 `/tmp/selftest-scratch-pass-probe-bf_930ia` 我删了；这一条交复跑 |
| broad-mutation.py | 见第 2 条 | stop 那一组格绿、红各一份；整份没跑完 |
| crash-amplification.sh、crash-point-replay.sh、mutation-shard-run.sh | 没跑 | 基准趟 535、680、895 秒，前台跑不了；开关表改动由 `build.py` 拿开关轮自己的读表 python 在改前、改后各读一遍，逐条相同 |

## 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`，先设 `S=$HOME/.local/share/singlefs/work/landing-audit-b12-switch-naming-slow` 与 `F=$HOME/.local/share/singlefs/work/landing-audit-b12-switch-naming-final`。起的都是登记成 light 的 `--selftest`（fg.py 每条经 `nice -n 19`），不起 cargo；本机那一段不经内存包装，第二台那一段由 multi-host-run.sh 经第二台的 run-with-memory-cap.sh 包（上限取配置的 PEER_MEMORY_CAP）。并行度取登记值：本机写 `--jobs 31`（`.claude/heavy-thread-default`），第二台那一侧 peer-run.sh 自己取 `heavy_thread_default`。第 1、2 条互不相干，可同时用 run_in_background 起。

fg.py 的一个脾气要知道：一份脚本此刻的 sha256 还没有基准趟时，那一次调用只排基准趟、跑完就退 0。所以本机那一段先单跑一次 `--baseline-only`，第二台那一段把 peer-run.sh 连起两次（头一次跑出基准趟与 #second，第二次补上不带后缀那一趟与 broad-mutation.py 的 #second）。

1. 本机：agent-watch.py 两趟、broad-mutation.py 第一趟（每条限时 1800 秒）：
   ```
   python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py,scripts/mutation/broad-mutation.py --jobs 31 --seconds 1800 --deadline 100000 --baseline-only; python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py,scripts/mutation/broad-mutation.py --jobs 31 --seconds 1800 --deadline 100000; python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py --jobs 31 --seconds 1800 --deadline 100000 --suffix '#second'
   ```
   看三段各自的末行「这一趟记了 N 条；还没记的 M 条」，M 都要是 0（不是 0 的同一条命令重起只补没记的）。broad-mutation.py 的基准趟就是第 2 条要的「整份 `--selftest` 不设开关」那一次。
2. 第二台：crash-amplification.sh、crash-point-replay.sh、mutation-shard-run.sh 两趟，broad-mutation.py 的 #second（每条限时 1800 秒）：
   ```
   bash scripts/hosts/multi-host-run.sh --peer --label b12final-replay --send $S/peer-send .b12-replay --fetch .b12-replay/runs -- bash -c 'bash .b12-replay/tools/peer-run.sh scripts/crash/crash-amplification.sh,scripts/crash/crash-point-replay.sh,scripts/mutation/mutation-shard-run.sh scripts/mutation/broad-mutation.py; bash .b12-replay/tools/peer-run.sh scripts/crash/crash-amplification.sh,scripts/crash/crash-point-replay.sh,scripts/mutation/mutation-shard-run.sh scripts/mutation/broad-mutation.py'
   ```
   看末尾「第二台那条命令退 0」（bash -c 的退出码是第二次 peer-run.sh 的：0 是都记了，3 是还有没记的）与每一段「== 第二台：… 趟第 N 次」后面 fg.py 的「还没记的 0 条」。取回之后并进续跑表：`cp -n $HOME/.local/share/singlefs/work/multi-host-run/b12final-replay/.b12-replay/runs/*.json $S/runs/`。起不来（退 255、准入退 78、钩子拒）的，四份全放本机照第 1 条的写法跑（`--only` 换成四份）。
3. 两条都跑完之后判：
   ```
   python3 -B $S/tools/judge2.py $F/judge2-after.jsonl scripts/agents/agent-watch.py scripts/crash/crash-amplification.sh scripts/crash/crash-point-replay.sh scripts/mutation/broad-mutation.py scripts/mutation/mutation-shard-run.sh; python3 -B $F/tools/limits.py scripts/agents/agent-watch.py scripts/crash/crash-amplification.sh scripts/crash/crash-point-replay.sh scripts/mutation/broad-mutation.py scripts/mutation/mutation-shard-run.sh
   ```
   每份要「基准 退 0」，两组都是 `{'对': N}`：agent-watch.py 122、crash-amplification.sh 50、crash-point-replay.sh 41、broad-mutation.py 77、mutation-shard-run.sh 16。有「不同」的跑 `python3 -B $S/tools/stable.py <路径> --only-bad` 看是两趟都有的还是只一趟有的；只一趟有的、又是「只一趟有的差异」一节那七条之一，是已知的不定，那一份先不登记，交下一个 tooling-writer 照那一节末段的根治办法改了再复跑；别的（两趟都有的差异）同样新派 tooling-writer 接着补点名。crash-amplification.sh、crash-point-replay.sh 的基准趟另要看输出里开关轮那一段没有 ✗（它从改后的表读格名）。
4. 两组全对的那几份照下面登记（`limits.py` 打的「限时」那一栏就是两趟里最慢一条的两倍取整到百；下面括号里是按旧那一版记录推的，以 limits.py 为准；理由里的秒数换成它打的「最慢」）。一行一条，接在 mutate.sh 那一行后面，次序不要紧：
   ```
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/agents/agent-watch\.py\t' --line $'scripts/agents/agent-watch.py\t<限时（约 600）>\t# 一百二十二条开关逐个跑过两趟：点名的格与实际红的格逐格相同；单跑一条最慢 <秒> 秒'
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/crash/crash-amplification\.sh\t' --line $'scripts/crash/crash-amplification.sh\t<限时（约 1100）>\t# 五十条开关逐个跑过两趟：点名的格与实际红的格逐格相同（自证自带开关轮，读同一张表，括号里的失败行原文不读）；单跑一条最慢 <秒> 秒'
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/crash/crash-point-replay\.sh\t' --line $'scripts/crash/crash-point-replay.sh\t<限时（约 1400，第二台基准趟到 900 秒时 1800）>\t# 四十一条开关逐个跑过两趟：点名的格与实际红的格逐格相同（自证自带开关轮，读同一张表，括号里的失败行原文不读）；单跑一条最慢 <秒> 秒'
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/mutation/broad-mutation\.py\t' --line $'scripts/mutation/broad-mutation.py\t<限时（约 900）>\t# 七十七条开关逐个跑过两趟：点名的格与实际红的格逐格相同（term-leaves-step-running 红在给包装设 term-leaves-scope 的那一趟）；单跑一条最慢 <秒> 秒'
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/mutation/mutation-shard-run\.sh\t' --line $'scripts/mutation/mutation-shard-run.sh\t<限时（约 1800）>\t# 十六条开关逐个跑过两趟：点名的格与实际红的格逐格相同（经门禁跑的那几格在每条开关下都先撞上样本根那一判、退 2）；单跑一条最慢 <秒> 秒'
   ```
   插完核表里每行 3 栏、第二栏是整数。
5. 命名检查不用跑：这一批没改 `.rs`。

## 顺带看到的（交主 agent，没改）

- **mutation-shard-run.sh：经门禁跑的那几格在每条开关下都先被门禁共用库拒掉。** 十六条开关的每一趟里，经 checker-tier-crates-mutation-replay 跑的 16 格（单机那一趟、①、②、③、④、⑮）都红在「环境里设了弄坏开关 MUTATION_SHARD_RUN_BREAK=…，而被判的根 … 不是样本根」（退 2）：门禁共用库「弄坏开关只在样本根上认」那一判排在 MUTATION_SHARD_RUN_SELFTEST_MARK 之前。judge 的汇总行算法（一半以上的开关下都多红的同一行不要求点名）把这 15 行放过了，只有 ① diff 那一行带了开关名、每条都不一样，才要逐条点名（这一批补的就是它）。后果是：读进 checker-tier-crates-mutation-replay 的那三条（no-final-pass、environment-opens、break-honored-anywhere）在那几格里到不了要证的那一处，两趟都少红的那几项（「退 1；diff：」「退 0；判定库记录行：」「退 0；stderr 尾部：」）就是这么来的，这一批照规矩改成了『』。要它们重新证得出，改法在门禁共用库或自证起门禁的那几格（让样本根判认这份自证的标记），不在这一批的写范围里。
- **broad-mutation.py、agent-watch.py、mutation-shard-run.sh 的不定**：见「只一趟有的差异」末段。
- **复跑工具 fg.py**：基准趟还没记的那一次调用只跑基准趟就退 0（见上一节开头）；peer-run.sh 靠退 0 判一趟跑完，单起一次会漏掉不带后缀那一趟。这一批的命令连起两次绕过去了，工具没改（它在兄弟任务 52 的草稿目录里）。

## scripts/hosts/run-with-memory-cap.sh（不碰、只报）

此刻 sha256 `d56034516139eee00563fe20041e3a12dd0c3941bc346d08d27e8eced8a2bb1b`，与兄弟任务 52 报的那一版相同，git 里是已暂存（`M `）。没改它。broad-mutation.py 新加的那一趟用到它两样：弄坏开关 `term-leaves-scope`（它第 896 行那一判）与第 546 行 `SELFTEST_MARK_CONTENT="run-with-memory-cap.sh --selftest 起的包装"`。改它的会话动这两样时，broad-mutation.py 的 selftest_stop_cells 那一格会红（出路写在那一格里），请那个会话一并改 broad-mutation.py 里那一行。

## 改过的文件

仓内五份（原件在 `F/originals/`，`python3 scripts/agents/copy-originals.py check F` 报「改过 5 份、不在了 0 份、没改 1 份」，没改的那一份是 break-switch-replay-files.tsv）。`git diff --stat -- <这五份>` 原样（五份开工时都已暂存、工作区没有别的改动，所以这就是这一批的改动）：

```
 scripts/agents/agent-watch.py          | 12 ++++--
 scripts/crash/crash-amplification.sh   | 36 ++++++++---------
 scripts/crash/crash-point-replay.sh    | 46 ++++++++++-----------
 scripts/mutation/broad-mutation.py     | 74 ++++++++++++++++++++++++++++++----
 scripts/mutation/mutation-shard-run.sh | 51 +++++++++++++++--------
 5 files changed, 150 insertions(+), 69 deletions(-)
```

最后一版 sha256：

```
86ffa88eb97fc9c51b1ecd07b6897a37dcbf636496a6bc6682da8b85eedd0fa3  scripts/agents/agent-watch.py
265fe635a2d29699e03c24854d902314809e9dc3d476e5535da7fa900d5eb672  scripts/crash/crash-amplification.sh
5bf17efea61cd79d487f368e4d65469db09044b04d0b389b93e0085a1aacc6e6  scripts/crash/crash-point-replay.sh
49273352a99610cea9ae89a8cd1c689d5201ea89a0bbfb67cf4dd2168f551b0f  scripts/mutation/broad-mutation.py
25c2b7eaa497d7fa2eb26f92700c736cf775c16aaad11ebf47e6bf751a5a102a  scripts/mutation/mutation-shard-run.sh
```

新写的 `# gate-similar:` / `# hook-events:` 行：没有（没新建门禁阶段与钩子）。没跑 `gate-overlap.py --list`：这一批没加门禁、钩子与判据，只改已有脚本的开关表、一格自证与两处失败行的写法。弄坏开关一条没新加（term-leaves-step-running 是已有的，这一批让它红在点名的格）。

草稿目录里的工具（`F/tools/`）：`show.py`（打某几条开关两趟多出来的失败行与少红的项）、`build.py`（按改法表出 hook-swap 规格、拿两趟旧记录按改后的表判、crash 两份另核开关轮读表）、`judgekey.py`（判法单元指纹改前改后）、`limits.py`（登记的限时）；改法表在 `F/edits/`。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没跑门禁阶段与各类 lint；没走定义三方；没提交。
- 改后的两趟复跑没跑（上一节第 1、2 条）；break-switch-replay-files.tsv 没登记（第 4 条）。
- broad-mutation.py 整份 `--selftest` 不设开关与设 term-leaves-step-running 两趟都没跑完整份（235 秒限时），只跑了 stop 那一组格；crash-amplification.sh、crash-point-replay.sh、mutation-shard-run.sh 换上之后一次自证都没跑；agent-watch.py 只跑了不设开关的一趟。都由上一节第 1、2 条的基准趟与开关趟覆盖。
- 「只一趟有的差异」那七条没补点名，也没改那几格的写法（根治办法在那一节末段）。
- 没碰 scripts/hosts/run-with-memory-cap.sh。没上卡（五份的自证都不上卡；本机卡 0、1 在被别的会话的 e185 用着）。
- 开跑前看负载：没有 qemu-system、vm-bench.sh、e152-file-system-benchmark、fio；有别的会话的 rustc、变异复跑与 e185、e184 装置在跑，一分钟负载 39 降到 15。我的自证都在本机经 `nice -n 19` 前台跑，没用第二台，没等锁。
- 草稿目录里没有仓副本、编译目录与库拷贝：判法单元指纹那两棵小根（`F/keyroot/`，2.6M，只有四份 .py）用完删了；我起的两次超时的自证留下的 `/tmp/broad-mutation-selftest.cn90e_sr`、`/tmp/selftest-scratch-pass-probe-bf_930ia` 删了。`/tmp` 下别的 broad-mutation-selftest.*、selftest-scratch-pass-probe-* 是更早的别的趟留的，没动。
