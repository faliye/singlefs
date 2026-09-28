# governance-defs-r3 云端攻方（Opus）报告

攻击面：G1、G2、G3 为主，G4、G5 余力（正文 `research/prompts/_governance-defs-r3-body.md` 第四节分工表「云端攻方」一行）。前几轮判决 `research/prompts/governance-defs-r2-main-verification.md`；上一轮攻方 B1–B7 的角度不重复，这一轮攻的是它们的写回本身。
开工 2026-09-27。开工核过快照：`sha256sum -c research/prompts/governance-defs-r3-snapshot/sha256sums.txt` 41 行无一不符；`git diff --stat bfc447e HEAD -- .claude research/scripts crates` 无输出。负载：`ps -o pid,args -u "$(id -u)"` 里 cargo / gate.sh / qemu / fio / vm-bench 0 个。
没跑任何重型测试，没编译；钩子的判定用合成 PreToolUse JSON 喂真钩子；59 号的判档从今天的源文件用 ast 现抽函数再喂合成输出。这个容器里内存包装 `--check` 起不来（slice 总上限设不上），cargo 一条都跑不了，cargo 的输出形状是推的（见 D1）。

## 复跑

```
bash research/prompts/governance-defs-r3-opus-model/run-all.sh <草稿目录>
```

输出原样存在 `research/prompts/governance-defs-r3-opus-model/run-all.out`（200 行，7 段各 `rc=0`）；在另一个草稿目录再跑一次、把 `/tmp/…` 路径抹掉后 `diff` 无差别。模型目录 22 个文件：

```
5cfdf05a41270e003fe9ceefc6689ffe03d8e8d63bcb77cb807797f9b2bf82f0  ./feed.sh
0ed3a3060869096c329f41929e1be9b1c1ac69a16ace9232e63932dd1a969824  ./g1-judge.py
acf37b260023ebe8ba2bd1cae4c8c20462ebe1ba2c90ecdae56e9bf6541f02e7  ./g2-untracked.sh
09033528b612e16bb354341b97c9c7f60417f1deecdde976cf656a8a04fe06f5  ./g3-git-blind.sh
2c48910540759d5f237b939f1b5bf9e88b04698312891450187c926d25efd56c  ./g3-who-records-time.sh
e073d49fd086b70c0702f84ea4e7f0c231e6e2126484a84fbad68748c5f56fa0  ./g4-driver-name.sh
b10702805e1f79f86d8d90dfbab922eafcadd51d01e13d62bfef4eaf0a9b8828  ./g5-exit-two.sh
780288f04b6c53dc92a3515303d362059cbc4dfbe68750c34cb67bb6dc07257b  ./run-all.out
c0d12bee18170aaffacbf33c9d1a516360478902436e768420ebcc26edd84f75  ./run-all.sh
649d69e4b9c75881397312e46c891c5d4300e726217b14ffc339e7a5632da12c  ./run-hooks.sh
615b867e1568cb33abf9aaf6f7adbe1bf966d7254313e8d72acff4d2d3acaed3  ./cases/crash-verifier--check-57.cmd
0349decedcf0fd8388efddc4fcacea4fae6ca80744468dd25b8bbcb4b42637af  ./cases/crash-verifier--check-59.cmd
52d0eea16020dee9bc1451f45f00aae09212de56a55c83b6744e58a35f904f10  ./cases/crash-verifier--inputs-59.cmd
8deb0a9091006a742bc2f94453fdb595e5b655f270982228a53c411a6b5e8afe  ./cases/crash-verifier--run-55-qemu-first-transaction-logged.cmd
c0ccf4434353acb053b3fa011ec87385b25ee224cd3e4d32fd9ea66ce620041d  ./cases/crash-verifier--run-57-lkmm-logged.cmd
83a357cd5768d3fe69a4a784aebc442e388661a2207715e91881bc410a4b12cf  ./cases/crash-verifier--run-59-crates-mutation-replay-logged.cmd
8756956cd7e09c62f07b2ffe775f6b71a48f170e306107f21149e7118bb20377  ./cases/experiment-runner--replay-one.cmd
b43065ff2f7c849b9edf6014e77fe41d205a5194dd74cab13915bb38724b0161  ./cases/implementation-writer--cargo-bench-bare.cmd
16aa32d04fe40fecd83dff49eb1a2e4ff72ea0d30316bb2a07d02096b1d1378e  ./cases/implementation-writer--cargo-bench-wrapped.cmd
a09a383f927011fba2a8c4892c066fa6e6b855b9da4e606320e41c685ca9a65e  ./cases/implementation-writer--cargo-test-bare.cmd
07dd95e7dcb9cbe5b384ef001d892b4d8f458106daf24e39425c9cf79644d52a  ./cases/mutation-triage--read-59-log.cmd
30fb38e4ae59b72cc749c498d3c3497578a8c81920a3001736270b2e233530a4  ./cases/three-way-verifier--changed-since.cmd
```

模型只跑 bash、python3 与 git；临时仓建在给定的草稿目录下，不碰仓里任何文件。

## 各格判定一览

编号用这一轮新的一族 D（上一轮是 B）。「分辨」一列答的是：这条历史让改前、改后的写法一起出局，还是只让改后的出局。

| 编号 | 格 | 打中了什么 | 分辨改前 / 改后 | 我的判定 |
|---|---|---|---|---|
| D1 | G1 | 写回的那句「测试进程没跑完的，有 `error:` 行就记无效，否则记没红」与 59 号的判档次序相反：点名测试先打出 `... ok`、之后测试进程被信号杀，输出里有 `error: test failed`，59 号记「没红」不记「无效」。这正是上一轮 B1 自己列的 `crash-after-named-test-passed` 那一格，写回没采 B1 的改法甲（按「点名测试打没打出那一行」分） | 是：改前那句（「编译错误记无效、没有记没红」）在这一格预言对，改后预言错；`crash-before` 那一格正好反过来 | 站不住（两处文件给出相反做法），后果轻 |
| D2 | G1 | 「无效」除了编不过、进程被杀还有第三个来源：点名测试那一行没被认出、别的测试或点名测试照常红了，输出只有 `error: test failed, to rerun pass …`，没有 `process didn't exit successfully`。照第 5 步的尾巴读法，分诊员把它读成「替换文编不过」（第八类），出路是改替换文，改错了地方。今天的表里就有 11 行必走这一格：`crates/mutations.tsv` 第 6 列测试名尾巴带 `$`，59 号 `re.escape` 之后永远认不出点名测试红了 | 否：改前改后都把它读成编不过 | 不拿它判这一批（正文第五节末那条）；**11 行是 59 号在下一次真跑时必红的潜伏问题，另记、优先交主 agent** |
| D3 | G2 | 未跟踪文件那条核对命令三道共用 `crates litmus .lkmm-static-only`：别的会话的实现员起草一条新 litmus（写范围闸放行 `litmus/**`），55、59 两道都停，而它们不读它 | 是：改前按每道自己的输入取，55、59 照跑 | 站不住（照改后的定义做不下去，要等别的会话），后果中 |
| D4 | G3 | 核查员「没改过记 ✗」靠 `git log --since` 与 `git status --short` 判；`.claude/singlefs-ai-sop/` 整个目录被 `.gitignore` 忽略、一个文件都没跟踪，腿常引的 SOP 规则在开工之后被上游同步改了，两条命令都空，核查员记「没改过」、把对的引文记 ✗ | 部分：改前清单外那一支用同样两条命令但只说「照实写」，改后写死「没改过记 ✗」，并把它推到设计轮 | 站不住（判别子观测不到：被判的现查看不见被忽略的文件） |
| D5 | G3 | 「腿开工时刻」只有一个数，而各腿派出的时刻不同、续做腿更晚；只有代码轮那一节要求主 agent 记「派腿的时刻」，main-agent.md 与 three-way-inference.md 一处都没有。上一轮核查员用的是自己估的开工时刻，攻方腿自记的开工更早：两者之间被改的清单外文件照 D4 同样落 ✗。设计轮不给时刻时，第 1 步判别力自证挑不出合格的那一条，只能停在「核查方法不分辨」 | 部分：改前设计轮一律落「分不清」，自证一样停；改后多了一支，但没人被要求喂它 | 站不住（一步要的输入没人给） |
| D6 | G4 | 第 5 步照字面「另写一个 `driver_e<号>`」给同一个入库装置的第二份产物写驱动函数：bash 里后定义的盖掉先定义的，第一行登记跑的是第二个函数，没有任何闸查重名 | 是（改后新写的模板） | 轻：同一句又说「照 E156、E158 的先例」，E158 的 13 个驱动函数都带后缀 |
| D7 | G5 | 共用约束「2 是包装的用法写错，改写法再跑」：包装把被包命令自己的退出码 0–249 原样传出，执行员第 5 步经包装跑的 `replay.sh` 在构建失败、登记表第 4 列写错时自己退 2；照这句，执行员去改包装的写法、重跑，不去看构建为什么失败 | 是（改后新写的一句） | 站不住（两处文件给出相反做法：包装文件头说 0–249 是命令自己的） |

钩子：照改后定义写的 10 条命令（崩溃验证员取输入、两条核对、带指纹与日志跑 55 / 57 / 59；分诊员读日志；核查员两条现查；执行员经包装跑 `replay.sh`；`cargo bench` 经包装）前台、后台各喂三个钩子，全部 `rc=0`；对照组裸 `cargo bench`、裸 `cargo test` 被 `heavy-test-guard.sh` 以 `rc=2` 拒，与共用约束那一句一致。构造不出「照定义做被钩子拒」。

## G1 59 号计数的读法

被判的句子（两处同文，`grep -nF` 现查）：
- `.claude/agents/mutation-triage.md:28` 与 `.claude/rules/mutation-sampling.md:82` 都含：「测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」」与「「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过」。
- `.claude/agents/mutation-triage.md:27` 含：「编不过的与测试进程被杀的在「无效」一栏（第 5 步那一句分得开两者）」。

59 号今天的判档（`.claude/gate.d/59-crates-mutation-replay.sh`）：
- `:326` `    tail = "\n".join(output.splitlines()[-8:])`，而 `output = run.stdout + run.stderr`：分诊员看到的尾巴是 stderr 的末 8 行。
- `:338` `    red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)`
- `:342` `    if ran_pattern.search(output):` 之下记 `failure`（没红）——**先于** `:344` `    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:`。
- `:348` `        return line_number, "invalid", f"{table}:{line_number} {name}：替换文写进源码之后编不过（退出码 {run.returncode}）\n{tail}"`：凡落进无效的，59 号打出来的头一句都是「编不过」。

模型 `g1-judge.py` 用 ast 从今天的 59 号里抽出 `judge_one`，喂合成 cargo 输出（`run-all.out` 原样）：

```
case	59 号判档	定义第 5 步对「测试进程没跑完」预言的判档	分诊员照定义读「无效」	真实发生的
compile-error	invalid	（进程跑完了，不适用）	替换文编不过（第八类）	替换文编不过
named-test-failed	caught	（进程跑完了，不适用）	—	点名测试红了（抓到）
named-test-failed-dollar-row	invalid	（进程跑完了，不适用）	替换文编不过（第八类）	点名测试红了（抓到），表里测试名尾巴带 $
other-test-failed-named-absent	invalid	（进程跑完了，不适用）	替换文编不过（第八类）	别的测试红了，点名的那个名字在输出里不存在（测试改过名）
crash-after-named-test-passed	failure	invalid	—	点名测试跑完没红，之后测试进程被信号杀  <- 定义预言与 59 号不符
crash-before-named-test	invalid	invalid	进程被杀	点名测试没跑完，测试进程被信号杀
named-test-not-run	failure	（进程跑完了，不适用）	—	点名测试没跑到
```

合成输出的形状是推的：cargo 1.94.1 的二进制里 `strings` 得到「`test failed`」「`, to rerun pass `」「`"process didn't exit successfully: `」「`test exited abnormally; to see the full output pass --no-capture to the harness.`」这几个串；普通测试失败（libtest 退 101）时 cargo 只打 `error: test failed, to rerun pass …`，不接 `Caused by: process didn't exit successfully`，被信号杀才接——这一条按 cargo 源码的记忆推，没真跑。多线程默认下 libtest 在测试结束时才打「`test 名字 ... `」那一行。

### D1：写回那句的判档次序与 59 号相反

照定义做：分诊员拿到一次 59 号输出，里面有一条变异让同一个测试二进制里另一条测试栈溢出，点名测试先跑完、没红。照第 5 步那句（进程没跑完、输出里有 `error:` 行 ⇒ 无效），它去「无效」一栏找这一条，找不到；这一条在「没红」一栏，尾巴里是 `process didn't exit successfully`，而定义只说这条尾巴出现在「无效」里。

上一轮 B1 已把这一格列出（`research/prompts/governance-defs-r2-opus-output.md:97` `crash-after-named-test-passed.txt	59 号：failure（整道判红：是）…`），并给了改法甲（同一文件 `:114`：「点名测试没打出行就落进「无效」、打出了就落进「没红」」）。写回用的是「有没有 `error:` 行」，不是「点名测试打没打出那一行」，这一格因此从对变成错。

四问：
1. 分辨：是。改前「编译错误记无效、没有记没红」在这一格预言「没红」，对；改后预言「无效」，错。`crash-before-named-test` 那一格正好反过来。
2. 看不看得到：部分。分诊员看得到这一条在哪一栏、尾巴里有没有被杀的字样；看不到点名测试那一行（在 stdout，尾巴只取 stderr 末 8 行）。
3. 字面：正文第五节「站不住」那条的「两处文件给出相反做法」（定义的描述与 59 号 `:342`–`:344` 的次序相反）。
4. 改法（都没实现，被攻过零轮）：

| 改法 | 修哪一格 | crash-after | crash-before | 量过 / 推的 |
|---|---|---|---|---|
| 甲一：两处那句改成「点名测试那一行打出来了就记没红（不论后面有没有被杀）；没打出来、输出里有 `error:` 行或 `could not compile` 就记无效；都没有记没红（没跑到）」 | 描述与 `:338`–`:349` 的次序一一对上 | 不再中 | 不再中 | 推的（按代码推；上表是在今天的 59 号上量的） |
| 甲二：再补一句「没红里尾巴带 `process didn't exit successfully` 的，点名测试跑完了、之后进程被杀，照没红分类，另列」 | 分诊员在没红一栏见到被杀字样时有出路 | 不再中 | — | 推的 |

推翻条件：拿一条让同一二进制里别的测试 abort 的变异在 59 号里真跑一次（提交时的崩溃验证员），它若落进「无效」，D1 撤回。

### D2：「无效」的第三个来源，今天的表里有 11 行

照定义做：59 号在 `crates/mutations.tsv:688` 那条（`E156 M31…`，cargo 参数 `-p singlefs-harness --bin e156_allocation_basis_counts`，第 6 列 `accounting_after_first_transaction_matches_the_registered_anchor$`）上，变异如期让点名测试红了；输出里是 `test tests::accounting_after_first_transaction_matches_the_registered_anchor ... FAILED`，`:338` 的正则要的是字面的 `…anchor$ ... FAILED`，认不出；`:342` 同样认不出；stderr 有 `error: test failed, to rerun pass …`，于是 `:344` 记无效，`:348` 打「替换文写进源码之后编不过」。分诊员照第 5 步看尾巴：没有 `process didn't exit successfully` ⇒ 不是进程被杀 ⇒ 编不过，按 `mutation-sampling.md` 第八类出路「改这一行替换文」。替换文没有问题，要改的是第 6 列。

`g1-judge.py` 对今天整张表扫第 6 列（`run-all.out` 原样，节选首尾两行与合计）：

```
  crates/mutations.tsv:202	referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite$	源文件里有 fn referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite：是	59 号的 red_pattern 认不认「test tests::referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite ... FAILED」：不认
  crates/mutations.tsv:694	hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test$	源文件里有 fn hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test：是	59 号的 red_pattern 认不认「test tests::hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test ... FAILED」：不认
  共 11 行 / 表里成形的 724 行
```

11 行（第 202、297、298、684–689、693、694 行）的 cargo 参数都是整个 `--bin e156_allocation_basis_counts`、没有过滤，第 6 列都带 `$`；`git log -S` 显示它们在 a1f4691（2026-09-24）与 346f5e6（2026-09-25）进表。它们在 59 号里只可能落「无效」（任何测试红了）或「没红·没跑到」（没有测试红），永远落不进「抓到」，所以下一次真跑 59 号必红。仓里没找到这两次提交之后 59 号跑过的记录；这个容器跑不了 59 号，这一条是按代码推加合成输出量的。

另一条走同一格的路：点名测试改了名，而第 6 列没跟着改（锚点只核原文、不核测试名，`.claude/gate.d/33-mutation-tables.sh` 与 59 号都不核）。

四问：
1. 分辨：否。改前「编译错误记无效」同样把它读成编不过。按正文第五节末那条，不拿它判这一批，另记。
2. 看不看得到：看得到。尾巴里是 `error: test failed, to rerun pass …`，没有 `could not compile`，与编不过分得开；定义没叫分诊员看这一处，还写了「第 5 步那一句分得开两者」，像是两种已经穷尽。
3. 字面：不按第五节判；是判据（定义的二分法）本身漏了一类。
4. 改法（都没实现，被攻过零轮）：

| 改法 | 修哪一格 | dollar-row | other-test-failed-named-absent | 量过 / 推的 |
|---|---|---|---|---|
| 乙一：第 5 步与 `mutation-sampling.md:82` 补第三种读法：无效里尾巴有 `error: test failed` 而没有 `could not compile`、也没有 `process didn't exit successfully` 的，是点名测试的名字没对上，先核第 6 列 | 分诊员不再误归第八类 | 不再误归（59 号仍红） | 不再误归 | 推的 |
| 乙二：11 行第 6 列去掉 `$`（或 59 号把第 6 列当正则、不 `re.escape`） | 59 号能认出点名测试红了 | 不再中 | 仍中（换名那一路） | 推的；改表归 `experiment-runner` / 主 agent，改 59 号是改门禁 |
| 乙三：59 号在锚点预扫里同时核第 6 列：去掉 `$` 后在第 2 列的源文件里有没有 `fn <名字>` | 换名与 `$` 两路都在开跑前红 | 不再中 | 不再中（点名测试在第 2 列那份源文件里时） | 推的 |

推翻条件：59 号在这 11 行上真跑出「抓到」，或者仓里有别的机制在跑前把第 6 列的 `$` 去掉。

## G2 崩溃验证员

被判的句子：`.claude/agents/crash-verifier.md:24` 含「`git ls-files --others --exclude-standard -- crates litmus .lkmm-static-only`（cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层，要没有输出；别处的未跟踪文件不挡）」与「两样有一样不过就不跑那一道」。`git diff --quiet` 那一条按每道取输入，未跟踪那一条三道共用一组路径。

### 钩子：照定义做的命令都放行

`run-hooks.sh` 以 `agent_type=crash-verifier` 喂：取 59 号输入的 `awk`、57 / 59 两条核对、第 6 步指纹与 `{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<阶段>.sh; echo "exit=$?"; } > <草稿>/<阶段>.log 2>&1` 写在同一条命令里的 55 / 57 / 59 三条（「前台跑也写」）；前台、后台各一次，三个钩子全部 `rc=0`（`run-all.out` 的 `#### run-hooks.sh` 段）。分诊员 `grep -n '计数：' <草稿>/59-crates-mutation-replay.log` 同样放行。日志路径 `<草稿目录>/<阶段>.log` 与分诊员输入一节「提交时那一次门禁 59 号的输出路径（缺它不开工）」接得上。

### D3：未跟踪那条三道共用，55、59 被一条它们不读的 litmus 挡住

历史（`g2-untracked.sh`，临时仓，核对命令照定义写；`run-all.out` 原样）：

```
== U1 实现员在别的会话里起草一条新 litmus：别的会话留下未跟踪的 litmus/wip-fsync-order.litmus
  55-qemu-first-transaction.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：照跑
  57-lkmm.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：停
  59-crates-mutation-replay.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：照跑
== U2 别的会话调 57 号判别力时在仓根放了标记：别的会话留下未跟踪的 .lkmm-static-only
  55-qemu-first-transaction.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：照跑
  57-lkmm.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：照跑
  59-crates-mutation-replay.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：照跑
== U3 别的会话新建一个测试文件（该停）：别的会话留下未跟踪的 crates/singlefs-harness/tests/wip.rs
  55-qemu-first-transaction.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：停
  57-lkmm.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：停
  59-crates-mutation-replay.sh	git diff 退 0	改后未跟踪那条：停	改前（按这道自己的输入）：停
== 今天的仓里谁读 litmus/ 下的文件
  55 号脚本里 litmus 出现次数：0
  crates/ 里在运行时读 litmus 的测试文件：crates/singlefs-harness/tests/publish_order_matches_litmus.rs 
  crates/mutations.tsv 里跑这个测试文件的行数：0
  那个测试读哪几个 litmus："first-txn-root-implies-units.litmus" "first-txn-journal-implies-units.litmus" "first-txn-root-implies-units-nofence.litmus" "first-txn-journal-implies-units-nofence.litmus" 
  57 号读仓根标记：18:[[ -f "$ROOT/.lkmm-static-only" ]] && static_only=(--static-only)
```

59 号把 `litmus` 拷进每一片（`.claude/gate.d/59-crates-mutation-replay.sh:176` `COPIED_INTO_EACH_SHARD = ["crates", "Cargo.toml", "Cargo.lock", "litmus", "research/results"]`），但片里唯一读 litmus 的测试只读四个写死名字的已跟踪文件，而且表里 724 行没有一行跑它；cargo 不把 `.litmus` 认作目标。55 号脚本里一次都不出现 `litmus`。所以 U1 里 55、59 那两停是白停。U1 可达：`.claude/hooks/agent-write-scope.tsv:7` `implementation-writer	litmus/**`，别的会话的实现员被允许在那里新建文件。U2 靠有人把判别力样本才用的标记放进仓根，可达性低；而且 57 号在静态档全过时退 3（`.claude/gate.d/57-lkmm.sh:22` `  exit "$rc"`，`.claude/scripts/lkmm.sh:296` `  exit 3`），本来就不会假装绿。

停下之后的出路：`.claude/main-agent.md:26` 「9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。」——接得住，但要等别的会话把那条 litmus 暂存、提交或撤掉，提交流程停在这里。

四问：
1. 分辨：是。改前未跟踪那条按每道自己的输入取，U1、U2 上 55、59 照跑；改后三道一起停。
2. 看不看得到：看得到（`git ls-files --others` 列出的就是那个文件）。
3. 字面：第五节「站不住」那条的「照改后的定义做……做不下去」（等别的会话）。这与上一轮 B2 是同一种形状（核对范围比那一道读的宽），落在 B2 写回新加的那一条上。
4. 改法（都没实现，被攻过零轮）：

| 改法 | 修哪一格 | U1 | U2 | U3 | 量过 / 推的 |
|---|---|---|---|---|---|
| 丙一：未跟踪那条也按每道取：55、59 取 `crates`，57 取 `crates litmus .lkmm-static-only` | 55、59 不再白停 | 55、59 照跑，57 停 | 55、59 照跑，57 停 | 三道都停 | 推的（U 格里「改前」一列是按每道输入量的，与丙一在 U1、U3 上同，U2 上 57 停这一格是推的） |
| 丙二：保留三道共用，定义里写「litmus 下的未跟踪文件只挡 57」 | 同丙一 | 同丙一 | 同丙一 | 同丙一 | 推的；比丙一多一条要记的例外 |

推翻条件：今天的 59 号或 55 号有一条路径在运行时按目录枚举 `litmus/`（而不是读写死的名字）。

### G2 其余几问：构造不出

- 每道取的输入与 57 号实际读的：`lkmm.sh` 读 `litmus/*.litmus`、litmus 头里 `singlefs-models:` 指的文件（今天 `litmus/*.litmus` 里去重后 4 个路径全在 `crates/` 下，另一种写的是 `none`）、`crates/` 下 `*.rs` 里有没有写出 litmus 名字；57 号自己读仓根 `.lkmm-static-only`。都在 `litmus crates .claude/scripts/lkmm.sh` 加阶段脚本加未跟踪那一条里。
- 59 号拷进每一片的 `research/results`：`crates/` 里 `CARGO_MANIFEST_DIR` 只在 `publish_order_matches_litmus.rs` 出现，`research/results` 只出现在 `//` 注释里，没有测试在运行时读它；不在 59 号那一行输入里不造成误判。
- 55 号的仓根标记 `.qemu-prerecorded`（`.claude/gate.d/55-qemu-first-transaction.sh:122`）不在未跟踪那一条里，与 `.lkmm-static-only` 同形；但预录档全过退 3（同文件 `:303` `  exit 3`），崩溃验证员照第 3 步记非 0，不会记成通过。只记一笔，不算打中。

## G3 核查员

被判的句子（`grep -nF` 现查）：
- `.claude/agents/three-way-verifier.md:21` 含「没改过记 ✗，改过或没给开工时刻记「分不清：文件可能在腿开工之后被改过」」。
- 同文件 `:22` 起头「- 腿开工时刻……」；`:27` 含「挑一条文件在快照里、或腿开工之后没被改过的」；`:28` 含「`git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>`」。
- `.claude/rules/implementation-workflow.md:26` 含「连同派腿的时刻（`date -u`）交核查员当输入」，这一节标题是 `:24` `## 代码轮派腿之前记一份开工快照`。

五种情形各走哪一支（按上面几行推）：给了快照且文件在清单里 → `sha256sum -c`，对得上比主树、对不上只比倒推副本、都没有记分不清；给了快照、文件不在清单里 → 主树加两条 git 现查；没给快照的设计轮 → 同上一支；没给快照的代码轮 → 停下要；报告 sha256 对不上 → 整份分不清；没给开工时刻 → 清单外一律分不清。五支之间没有重叠。落空的是下面两处：两条 git 现查看不见的文件（D4），与「开工时刻」这个输入本身（D5）。另有一处小缺口：核查员的输入里没有「这一轮是代码轮还是设计轮」，代码轮漏给快照时它分不出该停还是该照设计轮核。

### D4：被 git 忽略的文件，两条现查永远是空的

`.gitignore:6` `/.claude/singlefs-ai-sop/`；`git ls-files .claude/singlefs-ai-sop | wc -l` → `0`。`.gitignore:14` `/research/scripts/memory-peaks.tsv`。前者装着腿最常引的 SOP 规则与 `gate.sh`、钩子脚本，由上游同步整份换掉（`.singlefs-ai-sop-version` 在 git 里改过 31 次，今天是 `0.0.57`）；后者每跑一次内存包装就被改写。

历史（`g3-git-blind.sh`，临时仓；提交时刻用 `GIT_*_DATE` 写死）：腿甲先开工、读到四份文件的第 10 行；之后别的会话同步上游 SOP、跑了一次包装，各在第 3 行前插一行（被忽略的文件）；再之后别的会话在一份跟踪着的钩子第 3 行前插一行并提交；腿乙最后才派。`run-all.out` 原样：

```
.claude/singlefs-ai-sop/rules/evidence-discipline.md	T=2026-09-27（腿甲开工）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行已变成「rule line 9」
.claude/singlefs-ai-sop/rules/evidence-discipline.md	T=2026-09-27（腿乙派发）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行已变成「rule line 9」
research/scripts/memory-peaks.tsv	T=2026-09-27（腿甲开工）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行已变成「peak 9」
research/scripts/memory-peaks.tsv	T=2026-09-27（腿乙派发）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行已变成「peak 9」
.claude/hooks/tracked-changed-late.sh	T=2026-09-27（腿甲开工）	git log=[2b38302 ]	git status=[]	改过 ⇒ 分不清	真相：第 10 行已变成「tracked line 9」
.claude/hooks/tracked-changed-late.sh	T=2026-09-27（腿乙派发）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行已变成「tracked line 9」
.claude/hooks/bash-command-detector.sh	T=2026-09-27（腿甲开工）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行没动
.claude/hooks/bash-command-detector.sh	T=2026-09-27（腿乙派发）	git log=[]	git status=[]	没改过 ⇒ 对不上记 ✗	真相：第 10 行没动
看得见被忽略文件的两种现查（推的改法，不是定义里的）：
.claude/singlefs-ai-sop/rules/evidence-discipline.md	git status --short --ignored=[!! .claude/singlefs-ai-sop/]	git check-ignore=[.claude/singlefs-ai-sop/rules/evidence-discipline.md]
research/scripts/memory-peaks.tsv	git status --short --ignored=[!! research/scripts/memory-peaks.tsv]	git check-ignore=[research/scripts/memory-peaks.tsv]
```

（最后一对是对照：没被改过的跟踪文件，引文对得上时记 ✓，这一列的「对不上记 ✗」不会触发。）

腿真引 SOP 文件的行号：这一系列四份腿报告里 `grep -oE '\.claude/singlefs-ai-sop/[^ `:）]+:[0-9]+'` 数出 r1 攻方 3 处、r1 正推 1 处、r2 攻方 1 处（`.claude/singlefs-ai-sop/scripts/gate.sh:415`）、r2 正推 0 处。这一轮的快照清单（41 行）一个 SOP 文件都没有，这几处全走「清单外」那一支。

四问：
1. 分辨：部分。改前清单外那一支用的是同两条命令，但只写「照实写」；改后写死「没改过记 ✗」，并把同一判法推到没给快照的设计轮。
2. 看不看得到：被判的现查看不到——`git log` 与 `git status --short` 对被忽略、没跟踪的文件在「改过」「没改过」两种情形下输出逐字相同（都是空）。这是「判别子观测不到」那一形：要求把这两种情形判得不同的判据，用这两条命令满足不了。核查员自己看得到别的东西（`git check-ignore`、`stat -c %Y`、SOP 目录里的 `MANIFEST.sha256`）。
3. 字面：第五节「站不住」那条的「结果错」（对的引文被记 ✗）。
4. 改法（都没实现，被攻过零轮）：

| 改法 | 修哪一格 | SOP 行、峰值表行 | 跟踪文件被提交改过那行（T=腿乙派发） | 量过 / 推的 |
|---|---|---|---|---|
| 丁一：先 `git check-ignore -q -- <文件>` 或 `git ls-files --error-unmatch -- <文件>`；被忽略或没跟踪的，不用两条 git 现查，改用 `stat -c %Y` 与开工时刻比，拿不到就记分不清 | 被忽略文件不再误记 ✗ | 不再中 | 仍中（那是 D5） | 推的（`check-ignore` 那两行输出是量的） |
| 丁二：快照清单罩进腿会引的 SOP 文件（或整个 `MANIFEST.sha256`） | 把它们挪进「在清单里」那一支 | 不再中（SOP）；峰值表仍中 | 仍中 | 推的 |

推翻条件：`.claude/singlefs-ai-sop/` 下有文件被 git 跟踪、或核查员定义另有一句叫它对被忽略的文件改用别的判法。

### D5：「腿开工时刻」只有一个数，而设计轮没人被要求给

`g3-who-records-time.sh`（`run-all.out` 原样）：

```
.claude/main-agent.md	「派腿的时刻」0 处	「腿开工时刻」0 处
.claude/rules/three-way-inference.md	「派腿的时刻」0 处	「腿开工时刻」0 处
.claude/rules/implementation-workflow.md	「派腿的时刻」1 处	「腿开工时刻」0 处
.claude/agents/three-way-verifier.md	「派腿的时刻」0 处	「腿开工时刻」3 处
.claude/agents/three-way-materials.md	「派腿的时刻」0 处	「腿开工时刻」0 处
要求记时刻的那一节的标题：
24:## 代码轮派腿之前记一份开工快照
这一轮（governance-defs-r3）的快照目录里有什么：
sha256sums.txt
上一轮核查员用的开工时刻、攻方腿自记的开工时刻、正推腿报告的提交时刻：
腿开工于 2026-09-27
开工 2026-09-27
107b79f 2026-09-27 三方 governance-defs-r2：云端正推腿报告
```

两处落空：
- **时刻取晚了**：上一轮核查员拿的那个时刻比攻方腿自记的开工晚一刻钟，与正推腿报告的提交时刻几乎同时。用它查 `git log --since`，两者之间被提交改过的清单外文件看不出来（上面 D4 表 `tracked-changed-late.sh` 那两行：T=腿甲开工那一行记分不清，T=腿乙派发那一行记 ✗，而那一行确实变了）。各腿派出的时刻本来就不同，按「撞了限额……新开一条接着做」接上的续做腿更晚；定义只收一个数，没说取最早的那一个。
- **设计轮没人喂**：唯一要求记时刻的是代码轮那一节；这一轮（设计轮）的快照目录里只有 `sha256sums.txt`，派给我的提示里也没有开工时刻。设计轮若连快照也不给（`three-way-inference.md` 没要求设计轮给），第 1 步判别力自证要挑「腿开工之后没被改过的」那条，没有时刻就一条都挑不出；挑任一条加 1 行去核，照 `:21` 只能落「分不清」，判不出 ✗，停在「核查方法不分辨」——而方法本身没毛病。

四问：
1. 分辨：部分。改前设计轮一律落分不清、自证一样停；改后多写了「拿腿开工时刻现查」一支，这一支只有在别人给时刻时才走得通，而没有一份文件要求设计轮给。
2. 看不看得到：看不到。核查员手里没有各腿真正读文件的时刻。
3. 字面：第五节「站不住」那条的「一步要的输入没人给」。
4. 改法（都没实现，被攻过零轮）：

| 改法 | 修哪一格 | 时刻取晚 | 设计轮没时刻 | 量过 / 推的 |
|---|---|---|---|---|
| 戊一：main-agent.md「派出去之后」或 three-way-inference.md「核查员按轮派」补一句：凡要派核查员的轮，派第一条腿之前 `date -u` 写进这一轮快照目录（例 `dispatched-utc.txt`），续做腿不改它；核查员输入改成「全部腿里最早的派发时刻」 | 两格都修 | 不再中 | 不再中 | 推的 |
| 戊二：只在核查员定义里写「取最早的那一个」 | 时刻取晚 | 不再中 | 仍中 | 推的 |

推翻条件：主 agent 在某处（记录、派发模板、材料员定义）已经对每一轮记下各腿最早派发时刻，而上一轮用的那个时刻是核查员自己估的、没拿它。

（D5 补一条现查：`grep -c '快照'` 在 `.claude/rules/three-way-inference.md`、`.claude/main-agent.md`、`.claude/agents/three-way-materials.md` 三份里都是 `0`。）

## G4 执行员入库装置（余力）

被判的句子：`.claude/agents/experiment-runner.md:32` 含「入库装置照 E156、E158 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 `E<号>|@driver_e<号>||<产物>|exact`」。`research/scripts/replay.sh:637` `    "${bin#@}" >"$fresh" 2>"$OUT_DIR/$tag.err"` 按名字调函数。

### D6（轻）：模板名给第二份产物用会与第一份重名，bash 静默盖掉

`g4-driver-name.sh`（`run-all.out` 原样）：

```
今天 replay.sh 里 @driver 登记行按实验号数：
     14 E158
      1 E9
      1 E156
      1 E142
今天 replay.sh 里名字不是恰好 driver_e<号> 的驱动函数：13 个（例：driver_e158_q3_1_g0() driver_e158_q3_1_s16() ）
今天 .claude/gate.d/ 与 .claude/hooks/ 里提到 driver_ 的文件：0 个；replay.sh 里查函数重名（declare -F / type -t）的行：0
E900 @driver_e900 期望比 e900-first.out，实际跑出：second-mode output
E900 @driver_e900 期望比 e900-second-mode.out，实际跑出：second-mode output
```

照字面做：重跑已有的入库装置、换一个模式出第二份产物，执行员再写一个 `driver_e<号>()` 与一行 `E<号>|@driver_e<号>||<第二份产物>|exact`；第一行登记从此跑的是第二个函数，它的比对报「对不上」，看着像装置坏了。没有闸查重名。

四问：分辨是（模板是这一批新写的）；看得到（`grep -c '^driver_e<号>()' replay.sh`）；字面「结果错」；但同一句点名「照 E156、E158 的先例」，E158 的 14 行里 13 个函数都带后缀，照先例做的执行员不会撞上，所以只记轻。改法（推的，被攻过零轮）：模板写成 `driver_e<号>` 或 `driver_e<号>_<模式>`（同一实验第二份产物起带后缀的新名）；要会红就在 87 号或 `replay.sh` 开头加一道「`^driver_[a-z0-9_]*()` 去重」。

### G4 其余几问：构造不出

- 写范围闸：`.claude/hooks/agent-write-scope.tsv:16` `experiment-runner	research/scripts/replay.sh	复跑登记…`，整份文件放行，写驱动函数不被拒；执行员 tools 有 Edit。
- 「编在仓根 `target/`」与「不设 `CARGO_TARGET_DIR`」：`research/scripts/replay.sh:456` `  (cd .. && cargo run -q -p singlefs-harness --bin e156_allocation_basis_counts)`，在仓根跑 cargo、不设变量，编进仓根 `target/`，两句相容。
- 第 5 步的复跑命令经包装跑、以 `experiment-runner` 喂三个钩子，前台后台都 `rc=0`。

## G5 其余（余力）

### D7：「2 是包装的用法写错」与包装的退出码表相反

被判的句子：`.claude/agent-common.md:46` 含「2 是包装的用法写错，改写法再跑」。

`g5-exit-two.sh`（`run-all.out` 原样）：

```
包装不给参数：退 2
包装文件头与传出那一行：
18:#   0–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 250–254）；
27:#   2      用法错（上限、余量、总上限不是 正整数[KMGT] 的写法，没给命令）。
132:exit "$command_exit"'
执行员第 5 步经包装跑的 replay.sh 自己退 2 的地方：
602:    exit 2
616:  exit 2
619:cargo build --release --manifest-path e7-index-bench/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; exit 2; }
共用约束那一句：
46:2 是包装的用法写错，改写法再跑
```

历史：执行员照第 5 步跑 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`；`research/` 工作区此刻编不过（它自己的新 bin，或别的会话的半截改动），`replay.sh:619` 把 cargo 的输出丢进 `/dev/null`、只打一句「replay: 构建失败」、退 2；包装在 `run-with-memory-cap.sh:132` 原样传出 2。照共用约束那一句，执行员判「包装的用法写错」，去改包装的写法再跑，同样退 2。另一条同形的路：`crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs` 在「宿主重跑写路失败」「读不了日志」这类真失败上 `std::process::exit(2)`（grep 数出这两个 bin 目录里 `exit(2)` 18 处），经包装跑时同样被读成用法错。这一步没真跑（这个容器里包装走不到起命令那一步），传出那一行是读源码的。

四问：
1. 分辨：是。改前共用约束没提 2。
2. 看不看得到：看得到。包装的用法错在起命令之前、stderr 头一行是「  ✗ …」接「  → 怎么办：…」（`run-with-memory-cap.sh` 的 `reject_usage`），`replay.sh` 的是「replay: 构建失败」。
3. 字面：第五节「站不住」那条的「两处文件给出相反做法」（共用约束说 2 是包装的，包装文件头说 0–249 是命令自己的）。
4. 改法（推的，被攻过零轮）：那一句改成「2 且 stderr 里是包装的「✗ … / → 怎么办」用法行，才是包装的用法写错；否则是被包命令自己退的 2，照那条命令的报错办」。改包装、让用法错换一个 0–249 之外的码，能根治，但那是改脚本，越出这一轮。

### G5 其余几处：构造不出

- 共用约束的 `cargo bench`：裸 `cargo bench` 被 `heavy-test-guard.sh` 以 `rc=2` 拒（「不经内存包装跑编译出来的代码被拒：cargo bench」），经包装的放行，与「不经它的由 `heavy-test-guard.sh` 拒」一致。
- main-agent.md「退出码是经内存包装的那条 `--full` 命令的，250–254 是包装自己的结局」：`.claude/gate.d/54-layer0-replay.sh:233` 打印的第三行在 worktree 没建好时退 1、建好时退 `--full` 那条的码，最后 `( exit "$layer0_full_rc" )`，与这句一致。包装用法错的 2 这里不会出现（命令写死在 54 号里）。
- 书记员两处（标题状态 N、21 / 30 号先红）没攻，见「这条腿自己的限度」。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 照定义写的命令被钩子拒 | 10 条命令 × 前台 / 后台 × 3 个钩子 = 60 次判定（另 2 条对照） | 60 次全 `rc=0`；对照组裸 `cargo bench` / `cargo test` 各前台后台被 `heavy-test-guard.sh` 拒（`run-all.out` 里 `rc=0` 共 68 行 = 60 + 对照组在另两个钩子上的 8 行） |
| 59 号八栏与分诊员第 4 步的「共 N 条」相加对不上 | `59-crates-mutation-replay.sh:407`–`:408` 计数行的八栏 | 八栏都在一行里，定义要求全加，构造不出漏栏 |
| 57 号读了崩溃验证员输入之外的东西 | `lkmm.sh` 全部 `$ROOT/` 读法、litmus 头里的锚点路径 | 都在输入里 |
| 59 号读了 `stage-inputs.tsv` 那一行之外的东西而判错 | 拷进每一片的 `litmus`、`research/results`；`crates/` 全部 `CARGO_MANIFEST_DIR`、`include_str!`、`research/` 字样 | 只有一个测试读 4 个写死名字的 litmus，表里没有一行跑它 |
| 仓根标记让 55 / 57 假装绿 | `.qemu-prerecorded`、`.lkmm-static-only` | 两个都退 3，不会记成通过 |
| 核查员五种情形两支重叠 | 输入一节第 21、22 行与第 2、6 步 | 没有重叠；落空见 D4、D5 |
| 执行员写范围闸拒写驱动函数 | `agent-write-scope.tsv` 执行员 13 行 | 放行 |

## 这条腿自己的限度

- cargo 与 libtest 的输出形状（D1、D2 的合成输出）是推的：这个容器里内存包装 `--check` 起不来，照共用约束不绕开包装去裸跑 cargo。`strings` 只证明 cargo 二进制里有那几个串，不证明普通失败时不接 `Caused by`。
- D2 的 11 行「下一次真跑 59 号必红」按代码推、在今天的 59 号函数上用合成输出量过；没真跑 59 号（重型，子 agent 不跑）。
- D4、D5 的历史是临时仓里造的，时刻用 `GIT_*_DATE` 写死；「上游同步在一轮中间发生」的频率没量，只数了版本戳文件在 git 里被 31 次提交改过。
- D6、D7 只读源码、没跑 `replay.sh`。
- 每个改法都只在我的模型上量过或只是推的，被攻过零轮。
- G5 里书记员（kb-scribe.md 第 2、3 步与写入后钩子那一段）一处都没攻。
- 上一轮 B1–B7 的写回里，B2 / B3（D3）、B1（D1）、B7（D5）攻了；B4、B5、B6 的写回只顺带核了 B4（54 号第三行）。

## 没做什么

- 没跑任何重型测试（54、55、57、59、87、整轮门禁、`gate-staged.sh`），没编译，没跑 cargo。
- 没改仓里任何被判的文件；只写了这份报告与 `research/prompts/governance-defs-r3-opus-model/`，草稿在派发给的草稿目录。
- 没判正推那几格，不替主 agent 采纳；D2 里 11 行的改表、改 59 号都不归我。
- 没读禁读清单里的 `research/prompts/governance-defs-r3-sonnet-output.md` 与 `research/prompts/governance-defs-r3-sonnet-model/`。
