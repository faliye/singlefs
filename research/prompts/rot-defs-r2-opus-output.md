# rot-defs-r2 云端攻方（Opus）报告：K2、K3（2026-10-19）

攻击面：K2（照改后定义走一遍会被钩子拒或漏做的步、戊-A56 与丁-42 两处判法与定义句是不是一套说法、「一轮怎么开、怎么收」第 14 条）与 K3（`04ceaa904` 四份定义的改动、十二份定义里两轮都没判到的过时句）。不重复第一轮攻过的角度（第一轮那二十处改动与漏改的句）；第一轮自己的格只在「修补本身」出了新问题时再碰。

## 复跑命令与文件哈希

模型目录 `research/prompts/rot-defs-r2-opus-model/`：把命令文本当 PreToolUse(Bash) 的输入喂给 Bash 上判命令的三道钩子（bash-command-detector、heavy-test-guard、run-scope-guard），不执行命令本身；另照门禁源码的 echo 模板造一份快档日志、量 `capped.sh` 把声明当命令词时的退出码。检出记录与上次红的清单都指到 `--work` 目录。

```
cd /home/fy5090/code/singlefs
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/rot-defs-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/rot-defs-r2-snapshot --work <新的草稿目录>
```

末尾打四行 `name=compare file=… same=yes|no`，与 `out/` 里留底的四份逐字节比；本腿第三趟复跑四行都是 `same=yes`。`out/snapshot-drift.out` 记开工快照之后变了的被判文件（见「这条腿自己的限度」）。

```
01ef8d0952bf1ab3398715028205020f9ca336fc59bd6ae5f6aeb346d3bf6e0f  ./cases/crash-quick-array-appended-in-function.txt
0976234b42e3001b232ff5d1b5e9c6dc45a012a2087a0e84123444f2e7c3fa78  ./cases/crates-mutation-quick-with-break-switch.txt
7a51944152e046cfa0b967ddfd16651e21fdf970fa00de1a42dfd3b43b226c00  ./cases/device-deps-e165-pinned-cases.txt
4ba1763d4c68b0db27727fb0105f0cd564fc2d7c61d8544d0cc231998ab5fedc  ./cases/device-main-e165-pinned-cases.txt
9336fda07afd9aa909e2d1ddaf531ac74e5ec0e16943dc93fb727a2419ebba49  ./cases/formal-quick-with-break-switch.txt
6e84733ee85cd573079825b720297196e0cd7bb656553239ce49a2ce2ece2098  ./cases/implementer-combined.txt
f6d168e97a8c338991347e4eb83611e46716b8b699b773dfbfaf8ba6628c71a2  ./cases/kb-scribe-write.txt
4c8092d65a2178efd3239b4f6fb27d6ff32692969eb24c239b97734f6ce8164e  ./cases.list
f895cd3cb4af99c2eda408fd892ebf7461f81a68644198cb3dc416af19a2a1ee  ./cases/pin-script-replay.txt
54263cfb9619ec585c4e5b0a6738e99d5157d91470e3b009f3d0af384b97b34d  ./cases/rerun-wrapped-declaration-inside.txt
b7da308c1814e67b7b6502fe36a2ae808b488b497e859d339167f771eb4de326  ./cases/rerun-wrapped-declaration-outside.txt
70052f3852fed49bd39dcdab667e5c9a48411283883a9485b122cd6e166ee137  ./cases/runner-checker-device-unit-test-package.txt
ee5e4c6ba972e647313b88137924be4d2107d2c14b8c82bd622baf0e585bffd0  ./cases/runner-checker-device-unit-test.txt
35aea31f8c9fba3b293f68f5278a994159eb35d3f9731cfe2be011c226fb0d64  ./cases/runner-research-unit-test.txt
c264efeabf81faea9f9658df1ea2ed80fd156412d8093556e8b3b6b3235e80cf  ./out/capped.out
f411fdf6dc632ea9a7775c41c081df77a63b0a058b54d2230882cd684958ddb4  ./out/probe-deps.out
7dbeb08fcf4b12eeb5bdcf98f54cb2ddf21784a23d95ca2ac8eb01bf5e0b2aa8  ./out/probe.out
5c92603b11df298cf08f5a0949a9c8a563b066eb3c99abe8386ef4c126110e21  ./out/quick-sim.out
bfbebd2e2120409a04774059a9f238a05a1c6760b635ff9141dec1ef9a92b8e2  ./out/snapshot-drift.out
04e4374f108cfbc0aee4ad61e94af7a3db8d4a199d68214f0d38dd94ad8ea563  ./probe.py
29d3602c706dc6c6d829542ae2f88d0b56f0fdfd3af3e6cbcad8fc47d623fb0f  ./tools/rerun.sh
```

`cases/device-deps-e165-pinned-cases.txt` 里写死了本机那份 `deps/e165_replay_bounded_enumeration-<哈希>` 的文件名；本机重编之后哈希变，这一格要照 `ls -i target/release/e165_replay_bounded_enumeration target/release/deps/e165_replay_bounded_enumeration-*` 换成同一个 inode 的那一份再喂。

## 各格判定一览

| 名字 | 格 | 打中的地方 | 判定 | 量过 / 推的 |
|---|---|---|---|---|
| 入库装置单测 | K2 | 执行员照 1b 新加的那条带声明的单测命令：入库装置那一支（checker 档包 `src/bin/`）被重型测试闸拒，带不带 `-p` 都拒；定义没给出路，产出又要「单测数」。同一句的 bin 名写成 `e<号>-<简称>`，与第 2 步「中文简称只用在 kb 页」相反 | 打中 | 量过（钩子输入）/ bin 名那一半是对读原文 |
| 装置本体放行重放 | K2 | 戊-A56 放行直接执行 checker 档装置本体（`target/release/<名>` 与 deps 下同一份）；`e165_replay_bounded_enumeration --pinned-cases <钉表>` 就是形式化门禁红例钉表重放那一格的活，经 `e165-pin-red-examples.py --replay` 起是重型、直接起本体谁都放 | 打中 | 量过（钩子输入）；「同一份活」是按脚本文件头推的 |
| 轻型开关 | K2 | 两道带 `--quick` 的门禁在平常跑时照认弄坏开关：`GATE_MUTATION_BREAK=quickrunsallrows` 让 crates 变异轻型判整张表，`FORMAL_VERIFICATION_BREAK=quick-core-unselected` 让形式化轻型照整道跑；重型测试闸只看参数，照轻型放行子 agent | 打中 | 量过（钩子输入）；门禁那一侧按源码推的 |
| 监控行 | K2 | 第 14 条给的 `grep -cE "^  (✓\|✗) crash-case:"` 在提交时那一趟（快档逐条核用例）数出 `✓ 0 ✗ 0`，库里有红块、块没罩住的用例都不进这两个数；这一行也没有「本次未跑」「作废」的位置，与 crash-test skill 的抄法不是一套 | 打中 | 量过（照门禁 echo 模板造的日志）/ 真日志同形是推的 |
| 混合补丁 | K2 | 验证段第 1 步要一条命令里「打过的带 `--skip-apply`、没打过的不带、给全补丁目录」，脚本的 `--skip-apply` 是整条命令一个开关；有依赖的一段写段只打前几批、最后一批没打，照这一步写的一条命令必退（1 或 2），出路只在脚本拒绝说明里 | 打中（轻） | 推的（读脚本，没跑） |
| 声明夹进包装 | K2 | 实现员 1c：`rerun-failed-tests.py` 打出的命令开头已带声明，照共用约束在前面套 `capped.sh N` 与内存包装，三道钩子都放行，跑起来 `capped.sh` 把声明当命令词 exec，退 127 | 打中（轻） | 量过（钩子输入与 `capped.sh` 真跑） |
| E171 修订 | K3 | `main-agent.md:91` 仍写「今天是 E171 修订 8」；实验页 2026-10-13 已有修订 12 重钉段长 | 打中 | 推的（对读原文） |
| 产物名用简称 | K3 | `experiment-runner.md:38` 新产物名「按 `e<号>-<简称>.out` 算」，与第 2 步「中文简称只用在 kb 页」相反；`research/results/` 177 份产物没有一份带非 ASCII 字符 | 打中（轻） | 量过（计数）|
| 正推腿冻结副本 | K3 | `04ceaa904` 改了 `three-way-forward.md:37` 那一行，同一句在辩方、攻方两份里带「冻结副本只读」，正推这一份没有；辩方、正推都指到 `<模型目录>/tools/rerun.sh`，却不要求报告开头给复跑命令与哈希 | 打中（轻） | 推的（对读原文） |
| 小节指到段落 | K3 | `main-agent.md:89` 把三方规则里一段加粗的段首当小节引（「核查与判决由主 agent 做」），那份规则里没有这个标题 | 打中（轻） | 量过（标题扫描脚本） |
| 04ceaa904 四处 | K3 | 四份定义改的引用：小节标题今天都在、同一文件别处的写法一致 | 没打中 | 量过（标题扫描脚本） |

另有两处不在十二份定义里、顺手记下的旧句（门禁与钩子的文字，归 tooling-writer）在 K3 末尾。没打中的形状与这条腿的限度在末尾两节。

## K2：照改后定义走一遍会走错的步

### 入库装置单测

步骤（执行员照改后的定义走，跑前登记把装置写死成入库装置）：

1. 第 2 步把源文件放进 checker 档包：`.claude/agents/experiment-runner.md:34`：「时改写 `crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs`」（前面是「跑前登记把装置写死成入库装置」），bin 名：`.claude/agents/experiment-runner.md:34`：「入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`」。
2. 1b 这一批新加的命令：`.claude/agents/experiment-runner.md:31`：「跑单测（`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e<号>-<简称> 的单测; 范围=1 个测试目标; 为什么=…' cargo test --release --bin e<号>-<简称>`）」。
3. 装置的内联单测编在 checker 档包里，重型测试闸按包拒：

```
== runner-checker-device-unit-test（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；按包）（checker 档）：experiment-runner 不跑「checker 档」
[run-scope-guard.sh] exit=0
== runner-checker-device-unit-test-package（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；按包）（checker 档）：experiment-runner 不跑「checker 档」
[run-scope-guard.sh] exit=0
== runner-research-unit-test（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

（`out/probe.out` 原样；前两格在仓根起，第二格加了 `-p singlefs-checker-tier`；第三格是 research 那一支，在 `scripts/experiments` 下起。）范围声明闸三格都放（声明的种类对得上），拦下的是重型测试闸。定义没有哪一句说入库装置的单测不跑、交给谁：第 3 步只给变异写了去处（「三个数写「待提交时 checker-tier-crates-mutation-replay」」），单测没有；产出又要它：`.claude/agents/experiment-runner.md:53`：「单测数（一条命令数出来）」。照共用约束「被拒了不换写法绕过去」，执行员只能交回、交不出单测数。

同一句的 bin 名还写成了简称：`.claude/agents/experiment-runner.md:34`：「中文简称只用在 kb 页」；`scripts/experiments/Cargo.toml` 的 166 个 `[[bin]]` 名没有一个带非 ASCII 字符（`grep -A1 '^\[\[bin\]\]' scripts/experiments/Cargo.toml | grep '^name' | grep -cP '[^\x00-\x7f]'` 输出 `0`）。照 1b 抄 `e<号>-<简称>`，research 那一支 cargo 报没有这个 bin（推的，没跑 cargo），入库装置那一支的名字连下划线都不对。

四句：
- 分不分辨臂：这一轮没有臂；它分辨「定义要执行员跑的单测」与「闸放行的单测」，在入库装置那一支上不同、research 那一支上相同。
- 被判的系统看不看得到：看得到，闸读命令文本与当前目录下工作区成员的清单。
- 满足判据的哪一分句：K2 第一条「会被钩子拒、会不会漏做」两样都中。
- 改法在这一格上中不中：见下表。

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 1b 分两支写：research 那一支 `cd scripts/experiments && … cargo test --release --bin e<号>-<英文名里的下划线换成连字符>`；入库装置那一支不跑单测，产出的单测数写「待提交时 checker-tier-crash-point-replay 快档」（`.claude/rules/verification.md` 快档那一行写明快档带上装置二进制的内联单测） | 两半 | 推的（research 那一支的写法量过：上面第三格 `exit=0`） |
| 入库装置那一支的单测写进「要主 agent 跑的命令」，主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 起 | 被拒那一半 | 推的；要不要为一个装置的单测弹窗，归主 agent 定 |

推翻条件：重型测试闸改成放行 checker 档包里 `--bin <装置>` 的内联单测，或定义另有一处给入库装置单测写了去处（我读的是执行员定义全文与它点名的几份规则的点名小节）。

### 装置本体放行重放

步骤：

1. 戊-A56 写进闸的文件头：`.claude/hooks/lib_heavy_tests.py:68`：「直接执行装置的本体与它同判、不算」；判法：`.claude/hooks/lib_heavy_tests.py:2899`：「return None   # 装置的本体（cargo build 编出）」。攻方原型那一种也登成了许可：`.claude/hooks/lib_heavy_tests.py:71`：「攻方原型 `cargo build` 之后经内存包装直接执行编出来的二进制是许可的用法」。
2. 形式化门禁红例钉表重放那一格起的就是这个装置本体：`scripts/experiments/e165-pin-red-examples.py:62`：「再用装置 e165_replay_bounded_enumeration --pinned-cases <表> 在今天的实现上逐行判」。经脚本起是重型：`.claude/agent-common.md:56`：「与 `scripts/experiments/e165-pin-red-examples.py --replay`」（同一句在 `.claude/rules/verification.md:136`）。
3. 同一份活换成直接起本体：

```
== device-main-e165-pinned-cases（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== pin-script-replay（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：scripts/experiments/e165-pin-red-examples.py --replay --binary target/release/e165_replay_bounded_enumeration（门禁 checker-tier-formal-verification 的判法：起 Verus 或编 checker 档装置重放；只有字面写明的 toolchain、fingerprint、tool-versions、list-items 与红例钉表的追加、--check、--backfill、--list-items 不算；红例钉表参数里的变量只在可能展开成
[run-scope-guard.sh] exit=0
== device-deps-e165-pinned-cases（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

（`out/probe.out` 与 `out/probe-deps.out` 原样。）第一格是 `target/release/<名>`（批 4 之前就不判），第三格是 deps 下与它同一个 inode 的那一份（批 4 新放行的）。调查员、执行员、攻方腿、正推与辩方腿都不在拒绝的名单里，闸对它们一样放。

与定义句是不是同一套说法：不是。共用约束与 `verification.md` 按入口脚本列重型（`e165-pin-red-examples.py --replay`），闸按二进制是不是装置本体放行，两套在「直接起 e165 本体、喂钉表」这一种上给出相反的判定。定义里许可直接执行装置的句子只写了「自己的」装置：`.claude/agents/experiment-runner.md:31`：「`cargo run` 与装置二进制（例 `scripts/experiments/target/release/e<号>-<简称>`）」、`.claude/agents/three-way-attack.md:36`：「调枚举函数的原型只 `cargo build`，经内存包装直接执行编出来的二进制（不 `cargo run`、不 `cargo test`）」，没有一句说别的门禁格起的装置本体归谁。照定义走的子 agent 不会因为这一句去起 e165 本体；会去起它的是要复现「钉表重放红」的调查员（输入给的就是那一格的 ✗ 行，定义第 2 步要它收窄再复现），收窄到「直接起本体、喂钉表」这一步，闸放、共用约束的清单拒。

四句：
- 分不分辨臂：分辨「按入口判」与「按二进制判」两套说法，在这一条命令上两者相反。
- 被判的系统看不看得到：看得到，参数 `--pinned-cases` 与装置名都在命令文本里。
- 满足判据的哪一分句：K2 第二条「放行一侧会不会让子 agent 跑到本该归主 agent 的重活」。
- 改法在这一格上中不中：见下表。

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 闸把「装置本体带 `--pinned-cases`」与 `e165-pin-red-examples.py --replay` 同判（按装置名加参数，同 E152 按名字判那一种） | 这一格 | 推的；闸归 tooling-writer |
| 共用约束重型那一条补一句「门禁格起的 checker 档装置本体，直接执行同那一格算」，调查员第 5 步照它判 | 定义一侧的说法 | 推的；只靠文字，闸照旧放 |

装置那一侧：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:31`：「按数据行的次序每行判一个状态、打一行 `name=state`（与 `--replay-state` 打的那一行是同一段代码）」；逐态行与钉表比对由脚本做（同文件第 34 行），判状态这份活在装置里。

推翻条件：形式化门禁的重放格不再算重型，或闸改成按「装置名加 `--pinned-cases`」判。

### 轻型开关

丁-42 那一侧（项经数组传）我先试了能不能让数组多出一个让轻型变重的词：三个入口带 `--quick` 时多出来的词要么是用法错（崩溃点重放入口 `--quick --full` 退 2、形式化 `--quick` 与 `--check` 一起退 2），要么本来就是轻型许可的（`--item`、根），闸的数组模型又按文本次序保守取值（函数体里追加 `--full` 的那一种照拒，下面第三格）。数组这一条没打中，详见「没打中的形状」。

打中的是同一道「认轻型」判法的另一个口子：两道门禁在平常跑时照认自己的弄坏开关，而闸只看参数。

1. crates 变异门禁：`.claude/gate.d/checker-tier-crates-mutation-replay.sh:121`：「GATE_MUTATION_BREAK=quickrunsallrows  --quick 不读轻型变异登记表，整张表都判」；判法里不论是不是自证都认：`.claude/gate.d/checker-tier-crates-mutation-replay.sh:189`：「if ((quick)) && ((list_items == 0)) && [[ ",${GATE_MUTATION_BREAK:-}," != *,quickrunsallrows,* ]]; then」。
2. 形式化门禁：`.claude/gate.d/checker-tier-formal-verification.sh:93`：「quick-core-unselected  --quick 不把核心部分那三项放进点名，照整道跑」；同样不论场合：`.claude/gate.d/checker-tier-formal-verification.sh:196`：「if [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,quick-core-unselected,* ]]; then」。
3. 崩溃点重放入口有拦这一类的那一步（平常跑时带了只供自证的变量就退），两道门禁没有：`scripts/crash/crash-point-replay.sh:306`：「平常跑时带了只供自证的变量」。

```
== crates-mutation-quick-with-break-switch（implementation-writer）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== formal-quick-with-break-switch（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== crash-quick-array-appended-in-function（implementation-writer）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：scripts/crash/crash-point-replay.sh --quick --item core --full（崩溃点重放的重型：不带参数是剪枝、--item 是逐条、--full 是全量，只有仓内那一份带 --quick（可加 --item <项>）的轻型、单独的 --list-items 与登记表里的单独 --selftest 不算）（崩溃点重放）：implementation-writer 不跑「崩溃点重放」
[run-scope-guard.sh] exit=0
```

（`out/probe.out` 原样。）前两格的命令原样在 `cases/` 下：`GATE_MUTATION_BREAK=quickrunsallrows SINGLEFS_RUN_SCOPE='种类=quick; …' bash scripts/hosts/run-with-memory-cap.sh 8G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick $PWD` 与 `FORMAL_VERIFICATION_BREAK=quick-core-unselected … checker-tier-formal-verification.sh --quick $PWD`。照门禁源码推，前一条判的是 `crates/mutations.tsv` 整张表（共用约束列的「crates 变异整表」），后一条照整道跑（含红例钉表重放那一格），都是重型；闸与范围声明闸都按轻型放（范围声明闸那一侧本来只要求 quick 的范围非空）。

不是照定义走就会撞上的：定义没有一句叫子 agent 设这两个变量。它答的是 K2 第二条的后半句「放行一侧会不会让子 agent 跑到本该归主 agent 的重活」——会，而且要的只是一个环境变量，不是换写法。照 `.claude/rules/verification.md`「两档：轻型与重型」那一句（「夹了别的参数的、不带 `--quick` 的照重型拒」）读，环境变量不在「参数」里，说法本身就没罩住。

四句：分辨「闸认的轻型」与「门禁这一趟真跑的范围」；闸看得到（变量写在命令文本开头）；满足 K2 第二条；改法：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 两道门禁照 `crash-point-replay.sh` 那一步，平常跑时带了 `*_BREAK` 就退 2 | 两格 | 推的；门禁归 tooling-writer |
| 闸的 `light_tier_command` 见命令环境里带那道门禁的 `*_BREAK` 就不认轻型 | 两格（只罩命令文本里写出来的那一种） | 推的 |

推翻条件：两道门禁在别处（共用库 `lib/stage-cells.sh` 或 preflight）已经在平常跑时摘掉或拒绝这两个变量——我只读了这两道文件里的那几行，没读 preflight。

### 监控行

改后：`.claude/main-agent.md:33`：「14. **每轮监控固定抄一行」（后面接那一行的格式「这一趟 ✓ N ✗ M，✗ 的是：…」），数法给的是 `grep -cE "^  (✓|✗) crash-case:" <日志>`。

崩溃点重放门禁两种跑法打的逐条用例行不是一个样子：
- `--full`：每条用例收尾一行 `  ✓ <键> 判绿…` 或 `  ✗ <键> …`（`.claude/gate.d/checker-tier-crash-point-replay.sh:1435`：「echo "  ✓ $case_key 判绿（${threads_text}；${blocks_text}），判日志认出的行原样："」），第 14 条的 grep 数得对。
- 提交时那一趟（快档，逐条查库）：每条用例一行，前面四个或七个空格、不带 ✓ ✗（`.claude/gate.d/checker-tier-crash-point-replay.sh:941`：「echo "    $case_key：库里有判红的块（$blocks_output）"」，`.claude/gate.d/checker-tier-crash-point-replay.sh:942`：「echo "       $case_key：块没罩住（退 $blocks_exit：$blocks_output）"」）；判红只在汇总一行里，开头是条数不是键（`.claude/gate.d/checker-tier-crash-point-replay.sh:947`：「echo "  ✗ ${#red_cases_here[@]} 条崩溃枚举用例在崩溃判定库里有键相同而判红的块：${red_cases_here[*]}"」）。

照这几处 echo 的模板造一份三条用例的日志（一条罩住、一条有红块、一条块没罩住），用第 14 条的 grep 数：

```
name=item14_grep N=0 M=0
```

（`out/quick-sim.out` 原样，日志在 `tools/rerun.sh` 里现造。）照第 14 条抄出来是「这一趟 ✓ 0 ✗ 0」，而这一趟有一条判红、一条本次未跑。

与 crash-test skill 的说法对不上的两处：
- skill 要原样抄、不数：`.claude/skills/crash-test/commit-stages.md:30`：「`checker-tier-crash-point-replay` `--full` 逐条用例各抄一行」；第 14 条要数成一行。
- skill 有「本次未跑」「作废」两种结局：`.claude/skills/crash-test/commit-stages.md:29`：「退出码 77 记…，不记通过」（中间省掉的是「本次未跑」），`.claude/skills/crash-test/commit-stages.md:30`：「输出里读不到判定行的阶段记…，不记通过」（省掉的是「作废」）；第 14 条那一行只有 ✓ 与 ✗ 两格，grep 一行没数到时写出来是 `✗ 0`，读的人当成没有红。

与看门狗不冲突：看门狗那一节不规定监控时抄什么（`main-agent.md`「派出去之后」一节），`watch.conf` 的 `ask-every-minutes=0` 也不另起整点询问；第 14 条加的是主 agent 被叫醒之后自己做的一步，两边不打架。

四句：分辨「快档」与「全量」两种日志；主 agent 读日志时看得到哪一种（`--full` 开跑那一行写着）；满足 K2 第三条；改法：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 第 14 条改成「逐条报用例的跑法（`--full`）照 grep 数；快档照它末尾那一句原样抄（罩住、有红、本次未跑三个数）；数不到判定行的记作废」 | 这一格 | 推的 |
| 门禁快档逐条那一行也打 ✓ ✗ 前缀 | 这一格 | 推的；门禁归 tooling-writer，会动它的样本档 |

推翻条件：第 14 条说的「监控」只指 `--full`（条文里没写，「逐条报用例的」那几个字我读成举例）。

### 混合补丁

这一格碰的是第一轮 A8 的修补本身（批 4 照判决改了验证段第 1 步），不是重报 A8。

1. 写段只打「有依赖的」那几批：`.claude/main-agent.md:42`：「有依赖的：前一批实现员交回后，补丁只打不验」。一条依赖链里最后一批没有下一批依赖它，写段不打它。所以凡是写段里有过依赖的一段，进验证段时手里一定同时有打过的与没打过的补丁目录。
2. 改后的验证段第 1 步要一条命令同时照顾两种：`.claude/main-agent.md:51`：「写段打过的补丁目录带 `--skip-apply`（只编、只验，脚本先逐份核补丁已打在树上，核不过退 1），没打过的不带（脚本来打）；给全补丁目录是为了重编核对」。
3. 脚本的 `--skip-apply` 是整条命令一个开关：`scripts/gate/merge-and-verify.sh:787`：「--skip-apply) skip_apply=1; shift ;;」，带了就不打任何一份、逐份反着核：`scripts/gate/merge-and-verify.sh:832`：「if [[ $skip_apply -eq 1 ]]; then dirs=(); check_patches_applied "$logs"; fi」。

历史：一段写段有两批（乙依赖甲）→ 写段打了甲、派乙、乙交回 → 验证段第 1 步照「给全补丁目录」写一条命令。带 `--skip-apply`：乙反着核不过，退 1，拒绝说明给的出路是分两趟（`scripts/gate/merge-and-verify.sh:197`：「没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）」）；不带：甲再打一次，打补丁那一步退 2、整批停（`scripts/gate/merge-and-verify.sh:20`：「每个补丁目录经 scripts/gate/apply-writer-patch.py 打（退 2 就整批停）」）。两条路都停在脚本里，没有坏结果；坏在定义：第 1 步写的是一条命令，照它做必退一次，出路「分两趟」只在脚本的拒绝说明里，分两趟之后「给全补丁目录是为了重编核对」那半句也不再成立（每趟只核自己给的那几份）。

四句：分辨「一条命令」与「两趟」；脚本看得到（逐份反着核）；满足 K2 第二条「合入后验证那几句与判法是不是同一套说法」；改法（推的，被攻过零轮）：第 1 步写成「写段打过的与没打过的分两趟：先 `--skip-apply <打过的…>`，再 `<没打过的…>`；只有一种的一趟」，或脚本改成逐个目录认「已打」（反着核得上就跳过打），一条命令照给全。推翻条件：写段另有一句要求最后一批也先打进树（我读的是 `main-agent.md`「写与验分两段」一节全文）。

### 声明夹进包装

1. 实现员 1c：`.claude/agents/implementation-writer.md:34`：「每条开头已带 `SINGLEFS_RUN_SCOPE='种类=target; …'` 声明，没带的补上 target 声明再起），经内存包装起」。
2. 共用约束又要重活一律套 `capped.sh`：`.claude/agent-common.md:62`：「子 agent 起重活（编译、测试、变异、产物、原型扫描）的每条命令一律用 `bash scripts/hosts/capped.sh N <命令>` 起」。
3. 声明要留在最前、包装插在声明与 `cargo` 之间，只写在脚本文件头：`scripts/gate/rerun-failed-tests.py:13`：「（`SINGLEFS_RUN_SCOPE='…' bash scripts/hosts/run-with-memory-cap.sh <上限> cargo test …`），范围声明闸把外面的声明带进包装里面那一条。」实现员定义没写这一层。

照 1c 与第 62 行在打出来的整条前面套包装（`cases/rerun-wrapped-declaration-inside.txt`）与照脚本文件头插在中间（`cases/rerun-wrapped-declaration-outside.txt`），三道钩子都放：

```
== rerun-wrapped-declaration-outside（implementation-writer）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== rerun-wrapped-declaration-inside（implementation-writer）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

前一种真跑起来，`capped.sh` 把 `SINGLEFS_RUN_SCOPE=…` 当命令词 exec（`scripts/hosts/capped.sh:214`：「exec "$@"」）：

```
/home/fy5090/code/singlefs/scripts/hosts/capped.sh: line 214: exec: SINGLEFS_RUN_SCOPE=种类=target; 跑什么=x: not found
name=capped_declaration_as_command exit=127
```

（`out/capped.out` 原样，命令换成 `true` 量的。）范围声明闸把包装里面那个 `名=值` 词当成声明认了，所以一条跑不起来的命令被记成「放行、带对的声明」。实现员看到 127 会自己改，算轻；钩子那一半（把包装参数里的 `名=值` 当声明）是闸的误认。改法（推的）：1c 补「声明留在最前，包装插在声明与 `cargo` 之间」，或 `rerun-failed-tests.py` 直接打出带 `capped.sh` 与内存包装的整条。推翻条件：`capped.sh` 改成认 `名=值` 开头的参数并 export。

## K3：没被看过的地方

### 04ceaa904 四处（没打中）

四份定义在 `04ceaa904` 改的引用，逐个对今天的标题行（`grep -n '^#'`）：
- `experiment-designer.md` 第 16 行把 `test-script-discipline.md` 的路径从 `.claude/singlefs-ai-sop/rules/` 改到 `.claude/rules/`：后者在，「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」两节都在（第 87、62 行的二级标题），前者不在（`ls` 报 No such file）；同一文件第 34 行只写文件名，指的是同一份，一致。全仓 `.claude/`、`scripts/` 下不在 CHANGELOG 里的旧路径命中 0 处。
- `investigator.md` 第 16 行改成「假设要写得出能否掉它的观测」：`evidence-discipline.md` 第 73 行的二级标题逐字是它；旧标题「一个假设要能被观测否掉」全仓只剩上游 CHANGELOG 里一处历史记录。
- `three-way-defense.md`、`three-way-forward.md` 把 `tools/rerun.sh` 改成 `<模型目录>/tools/rerun.sh`：与 `three-way-attack.md` 同一句的写法一致。
- 十一份被判文件里「文件「小节」」这种引法，我用一份脚本逐个对被引文件的标题行（去掉井号与 `**` 后相等或以它起头），只报出「小节指到段落」那一处（下文）；反引号里的仓内路径逐个核在不在，0 处不在。

### E171 修订

`.claude/main-agent.md:91`：「有实验登记钉死了常量、还没拍树的（今天是 E171（三oracle合谋测试） 修订 8）」。实验页今天已到修订 12：那份实验页第 955 行是三级标题「第 1 次重跑·修订 12：流三段长重钉（2026-10-13）」，标题下正文第一句：`.claude/kb/experiments/171-三oracle合谋测试.md:957`：「这一节只属于装置里一个常量 `LOCAL_THIRD_STREAM_SEGMENT_LENGTHS` 的两个数」。修订 12 重钉的正是装置里钉死的常量；「今天是修订 8」这句话的「今天」已经过时，主 agent 照它判「这一批要不要先跑常量核对」时拿的是旧的修订号（要不要跑、跑哪一条 `stream-constants`，我没判，推的）。第一轮三份报告与判决里「修订 8」「stream-constants」命中都是 0（`grep -c`），两轮都没看过。

改法（推的）：这一句不写「今天是哪一次修订」，写成指路「钉死常量而还没拍树的实验见 `.claude/kb/experiments.md` 索引里状态列」，或干脆写「E171 实验页最新那一次修订」。推翻条件：修订 12 之后 E171 的常量已拍树、这一行整行该删（那就是更大的过时，不是推翻）。

### 产物名用简称

`.claude/agents/experiment-runner.md:38`：「新实验没有旧名，按 `e<号>-<简称>.out` 算」，与同一份定义第 2 步「中文简称只用在 kb 页」（`.claude/agents/experiment-runner.md:34`）相反。今天的产物都用英文名：`ls research/results/ | wc -l` 输出 `177`，`ls research/results/ | grep -cP '[^\x00-\x7f]'` 输出 `0`。这一句与「入库装置单测」那一格的 `e<号>-<简称>` 同出一源（第 38 行那句是 2026-09-27 写的，英文名那一条早一天，`git log -S` 查的），批 4 在 1b 新写的声明又照抄了一遍。改法（推的）：两处都换成「`e<号>-<英文名里的下划线换成连字符>`」。推翻条件：有新产物按简称起名且门禁不判它（我只数了今天的文件名）。

### 正推腿冻结副本

`04ceaa904` 改的正是这一行：`.claude/agents/three-way-forward.md:37`：「- 报告文件、模型目录、草稿目录。除此之外不写。模型目录里的」，同一句在辩方与攻方两份里多一截：`.claude/agents/three-way-defense.md:36`：「冻结副本（主 agent 建的开工快照、`~/.local/share/singlefs/work/<轮>-frozen/` 这类）只读」，`.claude/agents/three-way-attack.md:53` 同一句逐字也在。正推腿要编冻结副本里的代码时，定义不说先拷出来；照写范围「除此之外不写」读，它不会写进冻结副本，但 `cargo build` 在冻结副本里起会建 `target/`（写进去了）。另一处：辩方、正推现在都点名 `<模型目录>/tools/rerun.sh`，却没有攻方第 8 步那句「写模型的，报告开头给复跑命令与每个文件的 sha256」，主 agent 照三方规则复跑模型时，这两条腿的报告里不一定有可抄的命令。改法（推的）：正推那一句补上冻结副本只读那一截；辩方、正推的「产出」补一句写模型时报告开头给复跑命令与哈希。推翻条件：正推腿按定义永远不编代码（它的「做什么」第 3 步「能用命令核的事实复跑」没排除编译）。

### 小节指到段落

`.claude/main-agent.md:89`：「主 agent 自己核、自己写判决（`.claude/rules/three-way-inference.md`」，后面接的小节名是「核查与判决由主 agent 做」——那份规则里没有这个标题，它是一段加粗的段首：`.claude/rules/three-way-inference.md:123`：「**核查与判决由主 agent 做，不派子 agent 去核子 agent。**」。`git log -S` 看，这一句 2026-09-28 进仓起就是加粗段首，从没做过标题。照共用约束「引规则与 kb 的条款优先写 `文件@小节标题`」把它写成 `@` 引法，`cite-check.py` 会判标题不在。轻：人读得懂指的是哪段。改法（推的）：写成「`three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」一节里「核查与判决由主 agent 做」那一段」。

### 两处门禁与钩子的旧句（不在十二份定义里，归 tooling-writer）

- 写回员照定义写 `--write --check decision-items-sync` 之后回读红了，门禁的出路句仍叫它整道 `--write`：`.claude/gate.d/doc-decisions.sh:450`：「改完正文跑 bash .claude/gate.d/doc-decisions.sh --write」（同样的句子还在第 254、474 行），照抄就是第一轮 A1 量过的整轮拒。与写回员定义第 4 步那句「第 2、3 步里的 `--write` 是写回流程的生成一步，照跑」（`.claude/agents/kb-scribe.md:36`）读在一起，写回员不会照它跑，所以只是文字旧。
- 重型测试闸的拒绝说明列红例钉表不算重型的几种用法时漏了 `--check-table`：`.claude/hooks/heavy-test-guard.sh:818`：「与红例钉表的追加、--check、--backfill、--list-items 不算」，判法已经认它：`.claude/hooks/lib_heavy_tests.py:694`：「E165_PIN_LIGHT_MODES = frozenset({"append", "check", "check-table", "backfill", "list-items"})」。只影响说明文字。

## 没打中的形状

照改后定义的命令例子逐条喂钩子、填上占位后都放的（`out/probe.out`，全部 exit=0）：
- 写回员第 2、3 步与新立分项那一处的 `SINGLEFS_RUN_SCOPE='种类=items; …; 范围=1 格; …' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync`（`cases/kb-scribe-write.txt`）；`为什么=…` 原样不填的照拒（为什么太短），这是该拒的。写回员第 1 步那条写范围预检命令照原样在不设 `CLAUDE_PROJECT_DIR` 的 Bash 里跑：`.claude/kb/decisions.md` 退 0，`CLAUDE.md` 与 `research/prompts/x.md` 退 2，能用。
- 实现员第 4 步「串成一条时每一段各带自己的范围声明」：fmt、build、动到的测试二进制、形式化轻型四段各带声明、各写退出码文件，一条命令（`cases/implementer-combined.txt`）。
- 执行员 research 那一支的单测（`cases/runner-research-unit-test.txt`）。
- 主 agent 照 `main-agent.md` 新写的 `--full` 那一条（带 whole 声明与 `SINGLEFS_HEAVY_TESTS=user-request`，不带 agent_type）：三道都放（这一格在草稿目录里量的，没进 `cases/`）。

试过、判成没问题的：
- 丁-42 数组：能不能让数组在跑的时候比闸看到的多出一个变重的词。三个入口带 `--quick` 时，多出来的 `--full`、`--check`、`--list-items`、多一个根要么让入口退 2（崩溃点重放入口 `--quick --full`，形式化 `--quick` 配 `--check` / `--list-items`），要么本来就算轻型（`--item`、根）；crates 变异门禁 `--quick --check crates-mutation-rows` 照旧只判轻型的行。闸那一侧按文本次序取数组的值，函数体里 `items+=(--full)` 没调用也照拒（偏严；调用了的那一格在 `cases/` 里，没调用的那一格在草稿目录里量的）。没找到变重的写法。数组这一条与定义句的关系：定义与 `verification.md` 都没写数组，闸多认一种写法，方向是放宽写法、不放宽范围。
- 第一轮 A5 的修补（`main-agent.md` 第 10 行的数法 `--list-items --with-keys`）：`crash-point-replay.sh` 文件头说它数的是「只在别的判法单元指纹下判过」的块，正是判器单元变了的块，条件与数法对得上；它不列「变了哪几个判器单元」，D13 那一句要的清单仍不在（这一半第一轮已报）。
- 调查员第 2 步新加的「收窄到点名的目标或格再复现」与共用约束「子 agent 交回前不跑门禁阶段」：一个是复现、一个是交回前的验证，定义压共用约束，不冲突；收窄到 checker 档那几道的格照旧重型，由第 5 步交回，走得通。
- 变异分诊第 2 步「打 `⚓`、不跑，别的条照跑」：分诊不改表，`⚓` 由 `mutate.sh` 自己按条记（它文件头「锚点失效」一段），第 4 步要的「基线不绿 N 行」在 crates 变异门禁的「基线：」一行里有这几个字，对得上。
- 攻方腿 3b 点到的名字（`enumerate_crash_states`、`enumerate_crash_states_versions`、`crash_state_count_with_torn_in_place_overwrites`、`SparseBlockDevice`、`SparseDevice` 与 `MemoryPool` 的 `Clone`）今天都在；`closed_form_state_count` 在 harness 里两处。
- 共用约束第 46 行副本拷贝清单：`crates/` 外被编进去的只有 `.claude/heavy-thread-default`（`singlefs-harness` 与 `singlefs-unit-graph` 各一份 `include_str!`），清单里有它。
- 第 14 条与看门狗：看门狗那一节只管叫醒与处置，不管抄什么；`watch.conf` 不起整点询问。不冲突（冲突在 crash-test skill 一侧，见「监控行」）。

## 这条腿自己的限度

- 「量过」只到钩子这一层：命令文本当 PreToolUse 输入喂给钩子，看放不放；没有真起执行员、调查员去跑，没跑任何门禁、cargo 与装置。「装置本体放行重放」「轻型开关」两格里「这一趟真跑的是重型」那一半按脚本与门禁源码推的。
- 钩子输入里的 `agent_type` 是我按类型名填的，与第一轮同。
- 开工快照之后 `.claude/hooks/lib_heavy_tests.py` 又变了（`out/snapshot-drift.out`：快照记的 sha256 是 `7cae21b8…`，今天是 `30f7e232…`），十一份定义与 `run-scope-guard.sh` 没变。探针量的是今天的闸；照提示「按今天的实现来谈」，没回到快照那一版重量。
- 「监控行」的日志是照门禁 echo 模板造的，不是一趟真的提交时日志；门禁真打出来的 `$blocks_output` 那一截我编了样子，不影响那一条 grep 的结果。
- 「混合补丁」「E171 修订」「正推腿冻结副本」「小节指到段落」是对读原文推的。
- 写 `tools/rerun.sh` 时被重型测试闸拒过一次：我在脚本里用 `[[ -f "$candidate" && … && "$candidate" -ef "$uplifted" ]]` 找 deps 下那一份，闸把 `[[ ]]` 里 `&&` 后面的 `"$candidate"` 当成命令词，判成「直接执行 checker 档测试二进制」（`[[ ]]` 里的比较被当成命令执行，误判）。照拒绝说明的出路把那一步从脚本里拿掉，改成用例文件里写死本机那一份的文件名，没换写法绕。攻方腿照第 8 步写 `rerun.sh` 时同样会撞到，算轻，留给主 agent 判要不要记。
- 这是第二轮攻方的一次抽样；「没打中」那一节照 `.claude/rules/three-way-inference.md` 一次不算数。

## 没做什么

- 没判 K1（辩方那一格）；没读禁读清单里的两份；没替主 agent 采纳；没改任何定义、钩子、门禁与规则。
- 没建仓副本、没起编译、没跑门禁阶段、没跑重型与轻型（只跑了 `checker-tier-formal-verification.sh --list-items` 看可点名的项，只读）。
- 要主 agent 跑的命令：没有；核我量过的格只要复跑开头那一条 `rerun.sh`。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-defs-r2-opus/` 里留着探针第一版（`probe/`，含造的 `fake;run=1.log` 与 `rerun-failed-tests.py` 的输出）、两份 `rerun.sh` 草稿、两趟复跑的输出（`rerun-second/`、`rerun-third/`；`rerun.sh` 拷进去的冻结快照拷贝已删）与两份扫描脚本；都不是编译目录或仓副本，用完可删。
