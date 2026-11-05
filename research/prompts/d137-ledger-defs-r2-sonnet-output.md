# d137-ledger-defs-r2 辩方腿报告（格 D：复核第一轮判决，2026-11-02）

被复核：`research/prompts/d137-ledger-defs-r1-main-verification.md`（此刻 sha256 前 8 位 8b9b48ad）「逐条判」表十四条：十二条判「采」，两条判「不采」。
本腿没读第二轮攻方腿报告（禁读清单）；第一轮两条腿的报告也没读，判「采」的现查证据一律从被判文件、脚本、门禁与 crates 源码里自己现查。

## 一、结论先说

- 十二条「采」里，现查证据够得着结论的：十二条都够（站得住）。其中五条的改法只修了被打中的那一种写法、同一形状在别的写法上仍打中：A1（重复写的 `--features`）、A3（围栏之外的续行等）、A4（stdout 的取法、目录产物）、commit-stages 第 19 行（还有第三处要卡的）、A10（包名读到却不是工作区成员）。A2 的入口表没有跟源码同步的机械核对。
- 两条「不采」都站得住；第一条还有判决没写的更强理由：`CRASH_AMPLIFICATION_NOT_RUN` 这一行后面没有下划线，门禁出路里写的 `NOT_RUN_` 数不到它。
- 判决文字与落地对不上的一处：implementation-writer 第 47 行判决写「照字面列五处、不写条件」，落地的是「三处都要 + 四处有条件」（见下表 A7）。
- 跟改漏掉的一处：`experiment-designer.md` 第 31 行末尾那句还在说门禁「按源文件里有没有 `crash_amplification::`、`crash_state_enumeration::` 判特性」，门禁已改成按判段入口认。

## 二、复跑命令与文件 sha256

我自己跑的只有一样：把 `.claude/gate.d/doc-experiments.sh` 里 preregistration-judge-features 格内嵌的 python 抽出来，喂手造的登记，看判成什么。纯读文件、纯 python、没起 cargo、没起进程、没往服务交任务；世界建在草稿目录。

```
bash ~/.local/share/singlefs/work/d137-ledger-defs-r2-sonnet/cases.sh /home/fy5090/code/singlefs ~/.local/share/singlefs/work/d137-ledger-defs-r2-sonnet
```

草稿目录 `~/.local/share/singlefs/work/d137-ledger-defs-r2-sonnet/` 里的文件（sha256 前 16 位，量的）：

```
bde8118cbce5d746  cases.sh          八个手造登记的驱动
c88242f0c21aa106  judge_features.py 从门禁抽出来的内嵌 python（门禁此刻 sha256 前 8 位 690a35ef）
7b2b8d7050d5f6dd  cases.out         上面那条命令的输出
5363d92532a35c75  scan_bins.py      按门禁的认法扫 crates 与 scripts/experiment-apparatus 下的 bin，列谁调了判段入口
00b7e25a75c406bc  run_case.sh       单个世界的跑法（cases.sh 里有同样的内容）
```

被判的 16 份此刻与开工快照 `research/prompts/d137-ledger-defs-r2-snapshot/files-sha256.txt` 逐份相同（`sha256sum -c` 没有一行不是 OK，量的）。我引到的、不在快照里的文件此刻 sha256 前 8 位：
`.claude/gate.d/checker-tier-crash-point-replay.sh` e8ce42b9，`.claude/hooks/run-scope-guard.sh` c520949e，`.claude/hooks/runner-dispatch-guard.sh` acde6287，`scripts/crash/crash-case-check.py` 2ce57160，`scripts/mutation/mutate.sh` 1e064d0d，`.claude/rules/mutation-sampling.md` 0ebfe2eb，`crates/singlefs-checker-tier/src/crash_amplification.rs` 672e599d，`crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 80c2d27a，`crates/singlefs-checker-tier/Cargo.toml` 6c3d6cb3，`task-server/task` dfed1f94，`.claude/singlefs-ai-sop/rules/machine-first.md` 92e21312，`.claude/agent-common.md` 7abe0915。

## 三、逐条复核表（判决表的行号是 `d137-ledger-defs-r1-main-verification.md` 的行）

判定三档：站得住 / 够不着 / 同样打中别的写法。「推翻它的现象」写在每条末尾。

### 判「采」的十二条

#### 判决第 15 行：入库装置经 `compile-then-swap.py` 编、不带特性（正推 1、攻方 A1）

- 判定：站得住（证据够得着）；改法同样打中别的写法（一处）。
- 证据：判决写的「`scripts/hosts/compile-then-swap.py:300` 的 cargo 参数是」那一行，在暂存区那一版里是第 300–301 行（`git show :scripts/hosts/compile-then-swap.py` 里 `"cargo", "build"` 在第 300 行、`"cargo", "test"` 在第 301 行，量的）；HEAD 那一版是第 212–213 行。判决读的是暂存区版，行号没错。现在 `scripts/hosts/compile-then-swap.py:306`：「if features and mode != "features-dropped":」与 `scripts/hosts/compile-then-swap.py:307`：「profile += ["--features", ",".join(features)]」，两条 cargo 共用 `profile`（此刻 sha256 前 8 位 22ced829）。
- 同样打中：门禁认 `--features` 可写几次、取并集（`.claude/gate.d/doc-experiments.sh:3053`：「features.update(item for item in re.split(r"[,\s]+", following.strip("'\"")) if item)」，此刻 690a35ef）；`compile-then-swap.py` 把选项放进一个字典，后写的盖掉先写的（`scripts/hosts/compile-then-swap.py:839`：「options[argument] = arguments[position + 1]」，读代码推的）。登记里写 `--features verdict-store --features gpu` 门禁判绿（`cases.out` 的 C7），执行员照「原样给」抄成 `--features verdict-store --features gpu` 交给 `compile-then-swap.py`，只剩 gpu；反过来 `--features gpu --features verdict-store` 的 gpu 被吞掉，判段只打 NOT_RUN、没有任何一道报错。`--features=gpu`、`-F gpu` 落进位置参数、报「要给草稿副本与目标两个路径」，是响的，不算漏。
- 判决没交代：攻方 A1 的另一半（服务认要不要卡只看命令、裸二进制带了 gpu 也不分卡不清场）判决那一行没写去向；兄弟脚本报告写了「没修攻方 A1 的另一半」（`~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-scripts/result.md`，sha256 前 8 位见下）。本腿没读攻方报告，核不动 A1 另一半原文说了什么；只能说判决表里它没有去向。
- 推翻它的现象：有一份登记只写一次 `--features gpu`、执行员照抄喂 `compile-then-swap.py` 之后二进制里判段仍打 NOT_RUN（改法本身就没用）；或 `--features` 重复写在登记里的装置一个也没有而且以后也不会有（缺口无对象）。

（兄弟脚本报告 `~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-scripts/result.md` 此刻 sha256 前 8 位 67d5a285。）

#### 判决第 16 行：「源文件里有 `crash_amplification::` 或 `crash_state_enumeration::`」比「走到判段」宽（A2）

- 判定：站得住；改法只在现有入口上准，入口表没有跟源码同步的核对。
- 证据（判决自己写了「它们进不进判段没逐个核」，这里补核）：
  - 判段路由只有一个取路点：`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:1088`：「let route = crash_point_replay_judge_route()?;」，在 `enumerate_crash_states_through_the_library` 里；这个函数只有 `enumerate_crash_states_with_every_kind_of_observer`（`crash_state_enumeration.rs:400`）一处调，其余六个 `enumerate_crash_states*` 都经它。入口表里这七个名字对得上（`.claude/gate.d/doc-experiments.sh:2932-2934`）。
  - `crash_state_enumeration.rs:436` 的 `sample_crash_states_on_the_cpu` 文档写「不复用、不上卡」，函数体不碰路由（全文件 `crash_point_replay_judge_route()` 只有 1088 与测试里 3469 两处调用，量的）。e170、e184 只走这一支和 `plan_crash_points` 一类，所以「改前算、现在不算」是对的。
  - crash_amplification 一侧：`crates/singlefs-checker-tier/src/crash_amplification.rs:17428` 的 `run_from_environment_with_observers`（`run_from_environment` 转调它）、`:18442` 的 `GivenStatesEngine`、`:1470` 的 `judge_not_run_in_this_binary`、`:1482` 的 `judging_test_not_run_without_the_gpu_feature` 都读多机配置并比 `cfg!(feature = "gpu")`；我按「函数体里有 `multi_host_configuration_of_this_process`、`.uses_gpu()`、`judge_not_run`、`NOT_RUN`」扫了两份源文件的全部 `pub fn`，除了入口表里的名字，只多出 `judge_not_run_without_the_gpu_feature`（`:1402`，纯函数，收两个布尔）、`not_run_lines_without_the_gpu_feature`（`:1499`）、`given_states_not_run_line_without_the_gpu_feature`（`:1515`）、`multi_host_configuration_of_this_process`（`:1427`）、`judge_my_blocks_on_gpu_cards`（`:4169`，整个函数在 `#[cfg(feature = "gpu")]` 下，没编 gpu 时不存在）。那几个纯函数是拼 NOT_RUN 行的零件，不是路由，没有 bin 直接调（`scan_bins.py` 的输出里 bin 只用到表里的名字）。
  - 按门禁的认法扫此刻全部 bin：判段入口被调的是 e156、e158、e161、e168、e171（两份）；`crash_library_reader.rs`、e170、e184 提到模块名但不调入口（`scan_bins.py`，量的）。
- 缺口：全仓只有 `.claude/gate.d/doc-experiments.sh` 用 `JUDGE_ENTRIES`（`grep -rln JUDGE_ENTRIES` 只在它与 code-tooling.sh 里一个无关的同名片段命中，量的），没有任何一格核「两个模块里加了新的公开入口」；门禁注释自己写「两个模块里加了新的公开入口会走判段的，把名字加进 JUDGE_ENTRIES」（`.claude/gate.d/doc-experiments.sh:2913`，靠人记）。新入口漏登记时，旧认法（模块名记号）会判到、新认法判不到，方向是漏判。这是这次改法引入的方向变化，判决没单列。
- 推翻它的现象：`scan_bins.py` 点名「走判段」的 bin 里有一个其实不进判段（多要 gpu，不致命）；或有一个不在表里的 pub 入口被某个 bin 调用而且它会打 NOT_RUN（漏判）——后一种我没找到。

#### 判决第 17 行：跑前登记那一格按单行认命令，反斜杠续行漏判或误红（A3）

- 判定：站得住；改法只修了围栏里的反斜杠续行，同形状的别的写法仍打中（响的与静的各一）。
- 证据：判决记的七个世界与对照是主 agent 跑的（任务 4870）。本腿另跑的 `cases.out`：C5/C6 是设计员新写法 `SINGLEFS_RUN_SCOPE='…' task-server/task submit -- cargo test -p pkg --bin e990_judge`，不带 gpu 判红、带 gpu 判绿，范围声明里的汉字和 `cargo test` 字样没有造成误判；C8 只取崩溃点的 bin 不判。续行并行的实现在 `.claude/gate.d/doc-experiments.sh:3084`：「while (in_fence and "continuation-unjoined" not in broken and continued(line)」。
- 同样打中：
  1. 围栏之外的续行（列表项里的行内代码、缩进代码块）不并：`.claude/gate.d/doc-experiments.sh:3084` 的条件头一个就是 `in_fence`。`cases.out` 的 C3 把 `--bin` 拆到下一行放在列表项里，门禁退 77、报「没点名 --bin 的 1 条」，不判；同一份登记里还有别的被判的命令时，这一条只进「没点名 --bin 的 N 条」的计数，不红。这是静的，但有计数可见。
  2. `-Fgpu`（`-F` 紧贴值）与 `--features "verdict-store gpu"`（引号里带空格）判成没带 gpu，误红（`cases.out` 的 C1、C2）。这两种在改前就是这样，不是这次改法打出来的；是响的（误红），出路是改写成 `--features gpu`。
  3. 点名测试目标的 `cargo test -p <包> --test <目标>`（不点名 `--bin`）不判（`cases.out` 的 C4，退 77）：门禁注释 ① 写明「点名了 --bin <名>（--bin=<名>）的才判」。但定义（`.claude/agents/experiment-designer.md:31`）对 `cargo test` 一律要求带 gpu，`judging_test_not_run_without_the_gpu_feature`（`crash_amplification.rs:1482`）本来就是给测试用例用的；只要有装置写成集成测试目标，门禁判不到。现仓没有这样的登记，没有对象。
- 推翻它的现象：一份登记把 `--bin` 拆在围栏外的续行里、同一份登记里另有被判的命令，门禁绿而且没报计数（计数现在会报，所以不成立）；C1、C2 在真登记里出现过（`research/prompts/` 下 `grep` 不到，没有对象）。

（上一条里「真登记里 `grep` 不到」是我 grep 了 `research/prompts/e*prereg*.md` 里 `-F<字>`、`--features "…"`、`--features` 重复写三种，三种都 0 处，量的。第 15 行那条里重复写 `--features` 同样 0 处，缺口现在没有对象，是潜伏的。）

#### 判决第 18 行：`grep -c NOT_RUN <产物>` 数不到另写的文件与 `.gz`（A4）

- 判定：站得住；改法只修了「文件」「`.gz`」两种，stdout 与目录产物两种同形状仍打中。
- 证据：`.claude/agents/experiment-runner.md:40`：「装置的标准输出（任务日志）与它写出的每份产物」，这就是现在的写法，`.gz` 用 `zgrep -c`。此刻 `research/results/` 有 152 项、其中一个 `.gz`（`zgrep -c NOT_RUN` 得 0），全目录 `grep -rl NOT_RUN` 0 处（量的）。
- 同样打中：
  1. 「任务日志」：`task-server/task:25`：「task-server/task log <号> [--attempt <第几次>] [--all]  打那一次尝试」，默认只打最后 LOG_TAIL_LINES 行，要整份得带 `--all`。同一份定义的另一处写对了：`.claude/agents/experiment-runner.md:35`：「整份输出用 `task-server/task log <号> --all` 读」；第 40 行数 NOT_RUN 那一句没带 `--all`，照字面读默认日志，前面的 NOT_RUN 行会漏掉。这和 A4 打中的是同一类「数不全」，只是换了一个地方。
  2. 目录产物：`research/results/` 里有三个目录（`e10-aging-compaction-2026-10-13`、`e156-alloc-basis-counts-2026-10-18-support`、`e175-product-path-inode-watermark-regression-2026-10-08`）。第 40 行点名 `--output-directory` 这类，若字面 `grep -c NOT_RUN <目录>`，GNU grep 3.11（`/usr/bin/grep`，我的 shell 里 `grep` 是 ugrep 包装，所以用全路径）打「Is a directory」并打 0、退 2（量的，草稿目录里造了一个含 `NOT_RUN_X` 的目录与 `.gz` 试过，试验目录已删）。`zgrep -c NOT_RUN 目录/*` 才逐份数得出。「逐份」两个字能读出这一层，但「合计」没说怎么合。
- 推翻它的现象：执行员读 `task log` 默认输出就满足了第 40 行的「0」，而 `--all` 下 NOT_RUN 不为 0；或没有装置写目录产物（现仓已有三个目录，不是零）。

#### 判决第 19 行：设计员「整条写成 submit」那句没带范围声明（A5）

- 判定：站得住；括注里只写了点名目标的 `cargo test`，没写整包的。
- 证据：`.claude/hooks/run-scope-guard.sh:41` 的表头「#   cargo test  cargo test 与 cargo t」那一段：点名 `--test`/`--bin`/`--example`/`--bench`、`--lib`、`--doc` 的是 target，「别的（不挑目标的整包、整工作区）是 whole，范围写「整包」或「整轮」」；不要声明的形态在 `run-scope-guard.sh:58`：「cargo test 带 --no-run（只编不跑）或 libtest 的 --list」与 cargo 的别的子命令（run、build……）。设计员括注「`cargo test` 点名包与目标的带四段范围声明，写法照 `.claude/hooks/run-scope-guard.sh` 文件头；`cargo run` 不要声明」（`.claude/agents/experiment-designer.md:31`）与表头一致，没有写错。
- 缺口：不点名目标的 `cargo test -p <包>`（whole）同样要声明、种类写 whole，括注没提；`cargo test … --no-run` 不要声明，也没提。装置登记里几乎都点名 `--bin`，影响小。
- 推翻它的现象：设计员按这句写出一条不挑目标的 `cargo test -p X` 且没带声明，执行前被拒；或带了声明的 `cargo run` 被拒（表头没这条）。

#### 判决第 20 行：`implementation-writer.md:47` 的「五处」（转述：改前那句「它要登记五处」）漏 must-red 与崩溃放量 ITEMS（正推 2、攻方 A7）

- 判定：站得住；落地与判决字面不一致，但与现行判法一致。
- 证据：`scripts/crash/crash-case-check.py:11`：「must-red      每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标」（此刻 2ce57160）；`scripts/crash/crash-amplification.sh` 里确有 `ITEMS=(`（第 336 行）、`OBSERVER_ENTRIES`（`crates/singlefs-checker-tier/src/judging_units_entries.rs:146`）、`PROVENANCE_OF_THE_FLOWS`（`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs:139`）都在；轻型表挑法与定义的条件逐字同向（`.claude/gate.d/crates-mutation-light-rows.tsv` 第 4 行起：改坏对象在 `crates/singlefs-core/src/` 下的行才取）。第 47 行现在是「每条都要的三处」加「有条件的四处」（`.claude/agents/implementation-writer.md:47`，此刻 76ec207c）。
- 不一致：判决第 20 行写的做法是「第 47 行照字面列五处、不写条件」；落地的是三加四、写了条件。落地比判决字面更准（来历表那一行本来就带条件，`crash-case-check.py` 文件头 provenance 一段），但判决与落地对不上，主 agent 写回判决时要改字句或认可落地。
- 推翻它的现象：`python3 scripts/crash/crash-case-check.py --only placement,modules,provenance,must-red` 在一个只照这句登记了「三处」的新用例上红（说明条件写宽了，有条件的实际是必须的）。

#### 判决第 21 行：`commit-stages.md` 第 19 行一带的出路只重跑 feature-gated-tests 一格（正推 3、攻方 A8）

- 判定：站得住（漏 product-gpu 那一格是真的）；改法同样打中第三处。
- 证据：门禁文件头列的要上卡的地方是三处：`.claude/gate.d/checker-tier-crash-point-replay.sh:189`：「要上卡的三处（查库那一批里有带 gpu 特性的命令、feature-gated-tests 带 gpu 特性那一趟、product-gpu-real-card-tests 的对拍那一趟）开跑前都经 require_service_card 核环境里有这张卡」（此刻 e8ce42b9，不在开工快照里）。现在 `.claude/skills/crash-test/commit-stages.md:19` 的新句子是「再 `--check feature-gated-tests,product-gpu-real-card-tests` 只重跑要卡的这两格、不停那个进程」（此刻 1581ca99，与快照相同）。
- 同样打中：三处里的第一处在 crash-case-replay 格内：`checker-tier-crash-point-replay.sh:1944`：「require_service_card "$CRASH_CASE_REPLAY_CELL_NAME" "查库那一段（带 gpu 特性的用例）" || return 1」。所以「要卡的这两格」仍少了 crash-case-replay 的查库段。门禁文件头还说：覆盖、ignore、0 次三格读 crash-case-replay 格与 feature-gated-tests 格两趟的日志（同文件 `:23`，「快档读 crash-case-replay 格那一趟与 feature-gated-tests 格那一趟的日志」），其中任一没点就「本次未跑」或「行不全」（`:25`）。所以只重跑那两格，出口 4 那几格不会补出绿。清场回读没过的那一次到底卡住了哪几处，从这些文字里读不出来，核不动。
- 推翻它的现象：清场回读没过时，crash-case-replay 的查库段并没有被卡住（它的第一次运行已经完成），只需重跑后两格（这样新句子是对的）；反过来，重跑后出口 4 的三格报「本次未跑」。

#### 判决第 22 行：main-agent 第 32、101 行与 SKILL.md 第 21 行「快档两趟」没补那一格（正推 3.2、攻方 A9）

- 判定：站得住。
- 证据：`.claude/main-agent.md:32`：「crash-case-replay、feature-gated-tests、product-gpu-real-card-tests 三格只现判单元键对不上的测试函数」；`:101` 有「另有 product-gpu-real-card-tests 一格」；`.claude/skills/crash-test/SKILL.md:21`：「product-gpu-real-card-tests 一格（产品 GPU 包 `singlefs-background-gpu` 带 `tests-on-a-real-gpu` 的真卡对拍）」（此刻 d3fb6fd6 与 aa33115e，与快照相同）。`git diff` 外另查：`implementation-workflow.md` 第 44、68 行与 `verification.md` 第 35 行都已带这一格。
- 同样打中：`main-agent.md:32` 写这一格在补丁没碰产品 GPU 包时「报本次未跑」，与 `merge-and-verify.sh` 的新射程一致（见 A10）。没有别的活文档仍写「两格」：`grep -rnE 'crash-case-replay[、,，和 ]+feature-gated-tests|快档两趟'` 在 `.claude/`（除 kb 与上游 sop）下剩的命中都是已带那一格的句子。`.claude/kb/decisions/13-验证路线.md` 里 feature-gated-tests 的提及不在本轮射程，没查字句。
- 推翻它的现象：`grep -rn 'feature-gated-tests' .claude/skills .claude/rules .claude/main-agent.md` 出现一句只列两格的描述。

#### 判决第 23 行：`merge-and-verify.sh` 只看 checker 档闭包，只改产品 GPU 包的补丁判「没碰」（A10）

- 判定：站得住；改法在一个角上是失败时放过（fail-open）。
- 证据：`crates/singlefs-checker-tier/Cargo.toml` 里 `singlefs-background-gpu` 0 处（`grep -c`，量的，6c3d6cb3）；全仓只有 checker-tier 与 background-gpu 两个包带 gpu 类特性（`grep -n gpu crates/*/Cargo.toml` 的命中），没有第三个同类包；`crates/singlefs-background-gpu/Cargo.toml` 的 `tests-on-a-real-gpu = []` 挂四个 `required-features` 测试目标；包名在门禁里是 `.claude/gate.d/checker-tier-crash-point-replay.sh:600` 的 `PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu"`。`scripts/gate/merge-and-verify.sh:640`：「PRODUCT_GPU_PACKAGE_LINE = re.compile」现读那一行，读不出就照跑（安全方向）。
- 同样打中：`scripts/gate/merge-and-verify.sh:683`：「product_closure = set().union(*(closure_of(name) for name in guarded if name in manifests))」。读到了包名、但包名不是工作区成员时（门禁里的名字改了、工作区没跟上，或反过来），`guarded` 非空、`product_closure` 为空，下一步 `:687` 判「unaffected」（读代码推的，没起脚本）。读不出那一行是「照跑」，读到却对不上成员是「不跑」，两种失败的方向不一致。自证里没有这一格（报告里的自证是「补丁只改到产品 GPU 包」一格，包名是对的）。
- 推翻它的现象：`MERGE_AND_VERIFY_CHECKER_QUICK_STAGE` 指到一份 `PRODUCT_GPU_PACKAGE_NAME="不存在的包"` 的假门禁，补丁改任何 crates 文件，`merge-and-verify.sh` 不判「照跑」。

#### 判决第 24 行：「没卡报本次未跑」把服务没交卡（判红）与测试找不到显卡（本次未跑）并成一种（A11）

- 判定：站得住；改法漏列一种本次未跑。
- 证据：服务没交卡判红：`checker-tier-crash-point-replay.sh:189` 同一句后半「没有就那一格判红、那一趟不起」；测试找不到显卡：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1024` 起「本机没有 NVIDIA 卡时打一行 `NOT_RUN_NO_GPU_ADAPTER`」。`.claude/rules/verification.md:35` 的新句子把两种分开（此刻 0038f835，与快照相同）。
- 同样打中：`checker-tier-crash-point-replay.sh:163`：「本次未跑（不判红、不记库）：没卡时对拍打的 NOT_RUN_NO_GPU_ADAPTER 逐条报一行（按测试函数）；一个对拍目标跑到加复用 0 条也报。」`verification.md:35` 里 feature-gated-tests 那一格写了「挂特性的测试文件跑到加复用 0 条的」，product-gpu-real-card-tests 那一格没写「对拍目标跑到加复用 0 条也报本次未跑」这一种。
- 推翻它的现象：product-gpu 一格出现「0 条」的情形，报告者没把它报成「本次未跑」，而门禁其实报了——即规则句子少列一种导致主 agent 误判为绿。

#### 判决第 25 行：分诊定义的计数栏「六栏」与 `mutate.sh` 九栏对不上（A12，采后一半）

- 判定：站得住；另一支计数行不在分诊的射程里（够不着）；前一半欠账没找到落点。
- 证据：`scripts/mutation/mutate.sh:2163`：「echo "计数：$(verdict_count_text)$LIBRARY_COUNT_TEXT"」（同样的一句在 `:2432`）；`verdict_count_text` 打六栏（抓到、没红、无效、没跑完、抓名字有盲区、锚点失效），`LIBRARY_COUNT_TEXT`（`:2030`）接现判、复用、拒算三栏，共九栏。`.claude/agents/mutation-triage.md:32` 现在写「报九栏：抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏，后接判定库那一侧的现判、复用、拒算三栏」（此刻 4c81979a，与快照相同）。
- 够不着：`mutate.sh:2130` 另有一行「计数：派出去 N 条，收回 M 份判定：编得过…编不过…锚点失效…」，形状不同；它在 `--compile-only`（门禁 checker-tier-research-build-and-replay 的 mutation-tables 一格用）那一支，分诊员读的是平常跑的收尾行，所以这一支不影响「报九栏」这句。
- 没落地：`.claude/rules/mutation-sampling.md:79` 仍写「抓名字有盲区、锚点失效六栏」（此刻 0ebfe2eb，已知，正文第二节写了）；判决那一行说「前一半（另一种本次未跑）在 `crates-mutation-rows.py` 里……等它落地再跟」，我在 `records/2026-10-29-一百三十七节那一轮调度记录.md` 里 grep `ledger-defs` 只有第 311 行一处，且说的是「包装清单」，没有这一条欠账的落点（推翻它：主 agent 指给我欠账登记在哪）。
- 推翻它的现象：分诊员读到的收尾「计数：」行不是九栏。

#### 判决第 26 行：`agent-common.md:98` 删半句后仍写「分两种出路」，派发闸现分三条（正推 5、攻方 A6）

- 判定：站得住。
- 证据：`.claude/hooks/runner-dispatch-guard.sh:68`：「退 2 一律照拒，拒绝说明按输出分三条出路」（此刻 acde6287，不在快照里），认法 `:1356`：「ADMISSION_RETIRED_VARIABLE_SENTENCE = re.compile(rf"^✗ {ADMISSION_RETIRED_VARIABLE} 删了")」。`.claude/agent-common.md:98` 现在写「退 2 的拒绝说明分三种出路」（此刻 7abe0915，与快照相同）。
- 同样打中：同一句判的是三类，定义里的三类与钩子逐类同向（去掉已删变量／接准入／让 `admission.py` 能跑）；没有别的定义复述这句话（`grep -rn '分两种出路' .claude/agents .claude/agent-common.md` 0 处）。
- 推翻它的现象：钩子再加第四类出路而定义没跟。

#### 判决第 27 行：四份 read-once 规则点名与定义相冲（正推 6 核不动、攻方 A13，采前三样）

- 判定：站得住；另三份欠账没找到落点（核不动）。
- 证据：冲突两处现查在：`.claude/singlefs-ai-sop/rules/machine-first.md:87`：「- 对外格式与协议兼容」，对共用约束「里程碑 17 收尾之前不兼容任何旧东西」（`.claude/agent-common.md:52`）；`pushback-discipline.md` 的「警告写在哪：`.claude/warnings/<日期>.md`」（第 44 行起的一节）与攻方腿只能写报告、模型目录、草稿目录相冲。判决选的办法按小节点名，与共用约束的读法一致：`.claude/agent-common.md:21`：「- 规则类文件（`.claude/rules/` 与 `.claude/singlefs-ai-sop/rules/` 下的 `.md`）一律整份读，不受读入大小的限制：定义「开工先读：」一行点名的是其中几个小节时，照样读全那份文件，点名的小节是这件活要照做的那几条」。改后的四处点名里的节名我逐个对了标题行都在：engineering-philosophy 的「AI 友好 ≠ 不可读」（16 行）与「推论：人的注意力该花在哪」（36 行），machine-first 的「遇到一条通行准则，走这个流程」（52 行）与「判据」（61 行），pushback-discipline 的「对不上就当场说」（8 行）与「分清「我核过」和「我印象里」」（31 行），design-doc-discipline 的「1. 正文只写当前现状，历史一律放到文末」（9 行）与「2. 长度要配得上改动的重量」（34 行）。别的定义没有再整份点这四份规则（`grep -rn` 四个文件名，只在这四份定义里命中，量的）。
- 核不动：判决说「另三份只在共用约束括注里提到的规则不在这一轮射程，记欠账」；我在 `.claude/kb/checks-owed.md` 与调度记录里 grep 不到 `d137-ledger-defs-r1` 的欠账行；「记欠账」落在哪个文件，判决没写。
- 推翻它的现象：攻方腿或书记员照改后的点名只读小节、却仍被整份文件里没点名的小节要求写 `.claude/warnings/`（定义里没有一句说「没点名的小节不适用」，靠共用约束第 21 行那句的读法）。

### 判「不采」的两条

#### 判决第 28 行：正推 1 的小差异（定义写 `NOT_RUN`、门禁出路写 `NOT_RUN_`），不采

- 判定：站得住，而且有判决没写的更强理由。
- 证据：判决的理由是「定义的写法数得更宽，不漏判」。现查：`crates/singlefs-checker-tier/src/crash_amplification.rs:1523`：「pub const CRASH_AMPLIFICATION_NOT_RUN_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_NOT_RUN";」，判段打的那一行是 `CRASH_AMPLIFICATION_NOT_RUN step=judge …`，`NOT_RUN` 后面没有下划线。门禁出路里的 `NOT_RUN_` 数不到这一行；定义的 `NOT_RUN` 数得到。设计员与执行员定义里已把两种并列写（`.claude/agents/experiment-designer.md:31`：「`NOT_RUN_<原因>` 行与 `CRASH_AMPLIFICATION_NOT_RUN` 行都算」）。反方向的误计风险：`research/results/` 现存 152 项产物里 `NOT_RUN` 0 处（`grep -rl`，量的），当前没有无害的 `NOT_RUN` 字样会造成误报。
- 同样打中别的臂：门禁出路句子（`.claude/gate.d/doc-experiments.sh` 里 preregistration-judge-features 的「怎么办」，写「登记里写核 NOT_RUN_ 0 行」）仍是窄写法，会让设计员按门禁出路写登记时少数一种。这句在门禁，判决没动。
- 推翻它的现象：有一份产物里 `NOT_RUN` 作为无害字样（变量名、说明文字）出现而被定义的宽写法数成「本次未跑」。

#### 判决第 29 行：正推 2 说工具员报告里 `crash-case-check.py:25` 指不存在小节对不上，不采

- 判定：站得住。
- 证据：判决说那三处已改成现存小节名。现查：`grep -c '崩溃枚举用例住哪' scripts/crash/crash-case-check.py` 得 0（此刻 2ce57160）；第 25 行现在是「② 它要住在 crates/singlefs-checker-tier/tests/（checker 档，D13 已定项 15，.claude/rules/verification.md「门禁管哪一半」崩溃枚举用例住在 checker 档那一行）」，`.claude/rules/verification.md:131` 有标题「## 门禁管哪一半」。另两处同形状：`crash-case-check.py:602`、`:604` 引的 `test-script-discipline.md`「用例原子化：每一条都能单独跑」（该文件第 9 行有这个标题）、`path-moves.md`「怎么做」（该文件第 8 行有这个标题）、`verification.md`「harness 档里再分快档用例与慢档用例」（第 23 行有这个标题），也都在。
- 推翻它的现象：`scripts/crash/crash-case-check.py` 里还有一处引的小节名在被引文件里找不到标题行。

## 四、判决没点到、本腿顺带核到的跟改漏

- `experiment-designer.md` 第 31 行末尾：`.claude/agents/experiment-designer.md:31`：「门禁 doc-experiments 的 preregistration-judge-features 格按源文件里有没有 `crash_amplification::`、`crash_state_enumeration::` 判特性」。门禁现在按 `JUDGE_MODULES` 加 `JUDGE_ENTRIES` 两样都有才算（`.claude/gate.d/doc-experiments.sh:2931` 的 `JUDGE_MODULES="crash_amplification crash_state_enumeration"`，`:2947` 起的 `ENTRY_WORD`），不再按 `模块名::` 记号认。这句成了过时的描述：它还在教设计员「门禁按模块名记号判，对不上就报主 agent」，而门禁已经和定义用同一种认法。工具员报告里「留下的不一致」写的就是这一条（`~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-defs/result.md`），门禁改完之后没人回头改定义。推翻它的现象：把该句改成「门禁按判段入口认，入口表 `JUDGE_ENTRIES` 与定义里列的入口一致」之后，没有别的句子再依赖旧说法。

## 五、要主 agent 跑的命令

本腿没往统一任务服务交任务。下面都是轻型的，没有重型；`task-server/task submit` 的整条：

1. 复跑我的八个手造登记（纯 python，读仓不写仓；也可以直接在仓根起，它是文本工具，不起进程）：
   `task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-sonnet/cases.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-sonnet`
   看结果：与 `cases.out` 逐行相同（sha256 前 16 位 7b2b8d7050d5f6dd）；开头若出现「! 抽出的 python 与草稿目录里那份 judge_features.py 不同」，说明门禁在我之后又改过，C1–C8 的读数要重判。
2. 核 A10 的失败方向（我推的、没起脚本）：假一份 `PRODUCT_GPU_PACKAGE_NAME="不存在的包"` 的门禁，经 `MERGE_AND_VERIFY_CHECKER_QUICK_STAGE` 指给 `merge-and-verify.sh`，补丁只改 `crates/singlefs-background-gpu/src/` 下一个文件，看是不是判「unaffected」。这条要造样本，整条命令我拼不出来（自证格的样本造法在 `scripts/gate/merge-and-verify.sh` 的 narrow_case），建议派工具员加一格再交；主 agent 要的话，请交 `bash scripts/gate/merge-and-verify.sh --selftest` 的基线趟（登记 light，要的 `--server-binary` 与兄弟报告第 5 条一样）。
3. 核 A4 stdout 那一半：执行员读 `task-server/task log <号>` 默认输出就判 NOT_RUN 为 0，而 `--all` 下不为 0。造法：一条先打 NOT_RUN 再打超过 LOG_TAIL_LINES 行输出的命令，`task-server/task submit -- bash -c 'echo NOT_RUN_X; seq 1 3000'`，然后读 `task-server/task log <号> | grep -c NOT_RUN` 与加 `--all` 各一次。

## 六、没做什么

- 不替主 agent撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务；判决要引的数没复跑；本腿的读数里「推的」有：A1 重复 `--features` 被吞（读 `compile-then-swap.py:839`）、A10 包名对不上成员时判 unaffected（读 `merge-and-verify.sh:683`）、commit-stages 第 19 行少了第三处的后果。跑出来的有：`cases.out` 八个登记的读数、`scan_bins.py` 的 bin 清单、`/usr/bin/grep -c` 打目录与 `.gz` 的行为。
- 没读攻方腿报告，也没读第一轮两条腿的报告；没重议「子 agent 不交任务」的做法。
- 没跑重型测试，没起 cargo、没起进程，没改仓里任何文件（只写报告与草稿目录）。
- 没核 A1 另一半（服务不认裸二进制要卡）的攻方原文，也没读 `task-server/task_server.py` 里 `gpu_need` 的实现，只读了它的说明文字。
- 没逐条重跑判决第 17 行提到的七个世界（任务 4870 是主 agent 跑的）。
