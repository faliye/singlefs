# rot-defs-r1 云端攻方（Opus）报告：K2、K3（2026-10-17）

立场：假设这批「事实同步」改完的八份定义是错的，找照它做会被钩子拒、会漏做、会做错的步，和该改没改的句。没建副本、没起编译（模型目录里的原型包只写了 `Cargo.toml` 与空 `main`，只喂给钩子读清单，没编）。

## 复跑命令与文件哈希

模型只做一件事：把「照改后定义写出的命令」当成 hook 输入喂给仓里此刻的 `run-scope-guard.sh` 与 `heavy-test-guard.sh`，记退出码与拒绝说明前几行；不起 cargo、不跑门禁、检出记录写进 `--work`。

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/rot-defs-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新的草稿目录>
```

末尾打 `name=scope_probe same_as_model_out=yes` 与 `name=heavy_probe same_as_model_out=yes` 即复现（我这一次第二趟原样输出就是这两行，见下）。判的是仓里此刻的钩子：钩子改了，输出跟着变。

```
43fdd099e6337bf88d232e5285d0221fcd550535b21e74295c52a509ddce124c  probes/scope_probe.py
b50f0be8140cd70aff72f933f46f8641477692fb7191f8247a69b7e303745eec  probes/heavy_probe.py
9c91da192c8aad7110aa5ea43e98f22c0cade4c11923a6a8859d6bd6a99fe091  tools/rerun.sh
99543fd58b51585f8a2c6fe43b135bc9ae4bd98dd177f296bee0c424eb3fd04f  out/scope_probe.out
5ebd5291f3036dd8079a441154f80e2d37cb436c379bf8b815b3629e1e8edbe1  out/heavy_probe.out
```

第二趟复跑的末三行原样：

```
name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/code/singlefs model=/home/fy5090/code/singlefs/research/prompts/rot-defs-r1-opus-model work=/home/fy5090/.local/share/singlefs/work/rot-defs-r1-opus/rerun-second
name=scope_probe same_as_model_out=yes
name=heavy_probe same_as_model_out=yes
```

## 各格判定一览

| 编号 | 格 | 打中的地方 | 判定 | 量过 / 推的 |
|---|---|---|---|---|
| A1 | K2 | 写回员照 kb-scribe 第 2、3 步跑 `doc-decisions.sh --write`，范围声明闸按整轮拒子 agent，带不带声明都拒 | 打中（重） | 量过（钩子输入） |
| A2 | K2 | 攻方腿照 3b 调 `enumerate_crash_states` 的原型包必须链接 checker 档包，`cargo run` / `cargo test` 被重型测试闸拒；能过的只有「先 build 再直接执行二进制」，正是闸自认罩不住的那条缝 | 打中（重） | 量过（钩子输入） |
| A3 | K2 | 执行员照 1b 在 `scripts/experiments` 下裸跑 `cargo test`，按整轮拒子 agent | 打中 | 量过 |
| A4 | K2/K3 | 变异分诊第 5 步这一批改过，仍没有「点名的测试 ok 又打了 NOT_RUN_NO_GPU_ADAPTER 记本次未跑」那一分支（规则与门禁都有），会把这几条记成没红；也没有「基线不绿」 | 打中 | 推的（按规则与门禁样本原文对读，没跑门禁） |
| A5 | K2 | main-agent 第 10 行把 D13 已定项 9 的全量弹窗条件摘短了：丢了「对不上的键里有判器单元变了」那一半，也没指到能数出它的入口 | 打中 | 推的 |
| A6 | K2 | main-agent 第 94 行「不换树」与驱动文件头、commit-stages 第 2b 步 ③「换进库目录的专用树」相反 | 打中（轻） | 推的 |
| A7 | K2 | main-agent 验证段第 3 步「只改注释的不开代码轮」丢了「文档注释与 SAFETY 注释照代码算」 | 打中（轻） | 推的 |
| A8 | K2 | main-agent 第 7、29、50、90 行四处合入验证不是同一套：验证段第 1 步无条件 `--skip-apply`、写「补丁已打」，写段只给有依赖的那批打补丁；`--skip-apply` 不核补丁打没打 | 打中 | 推的 |
| A9 | K2 | agent-common 第 56 行这次只给轻型三条补了范围声明，同一句里的 harness 档 `cargo test -p <crate> --test …` 没带；实现员第 4 步「合成一条命令」用一份 export 会被按种类对不上拒 | 打中（轻，拒一次可改） | 量过 |
| B1 | K3 | 八份里没改到的命令句：main-agent 第 29、94、96 行，three-way-attack 第 53 行，implementation-writer 第 34 行（rerun-failed-tests 打出的命令不带声明） | 打中（轻） | 量过（第 94 行、第 53 行那两种）/ 推的（其余） |
| B2 | K3 | 别的定义：investigator 第 31 行「用输入给的命令原样复现」，给的是整包命令时子 agent 被拒，定义没说要收窄或交回 | 打中（轻） | 量过 |
| B3 | K3 | 钩子文件头 run-scope-guard.sh 第 32 行仍写 merge-and-verify「还不认只跑改到的包」，与判法与定义新写法对不上（主 agent 已知，归 tooling-writer） | 已知，不重报 | 推的 |

没打中的形状与这条腿的限度在末尾两节。

## K2：照改后定义做会走错的步

### A1　写回员跑不了自己定义里的 `doc-decisions.sh --write`

步骤（写回员照定义走）：

1. 规格翻了分项状态，走第 3 步，`.claude/agents/kb-scribe.md:33`：「然后 `bash .claude/gate.d/doc-decisions.sh --write`。」；不翻状态的走 `.claude/agents/kb-scribe.md:35`：「写完正文自己跑一次 `bash .claude/gate.d/doc-decisions.sh --write` 再往下」；第 2 步同样，`.claude/agents/kb-scribe.md:32`：「同一次一并跑 `bash .claude/gate.d/doc-decisions.sh --write`」。这一批改的第 22 行也指着它：`.claude/agents/kb-scribe.md:22`：「由第 3 步的 `doc-decisions.sh --write` 重生成」。第 4 步还明说它照跑：`.claude/agents/kb-scribe.md:36`：「第 2、3 步里的 `--write` 是写回流程的生成一步，照跑」。
2. 范围声明闸把门禁阶段不带 `--check` / `--item` 的跑法判成整轮：`.claude/hooks/run-scope-guard.sh:27`：「带 --check / --item 是 items」，不带的在判法里落到 `.claude/hooks/lib_run_scope.py:171`：「return RunShape("whole", f"门禁 {stage} 的全部格（不带 --check / --item）"」；`--write` 不在免声明的查询参数里（`.claude/hooks/run-scope-guard.sh:46`：「门禁阶段带 --list、--list-items、--selftest、--help、-h、--tool-versions」）。
3. 子 agent 的整轮一律拒：`.claude/hooks/run-scope-guard.sh:44`：「子 agent（hook 输入带 agent_type）的 whole 一律拒」；共用约束这一批新加的那条同样写了：`.claude/agent-common.md:87`：「子 agent 起 whole 一律拒」。

量出来的（`out/scope_probe.out` 原样，前三格）：

```
[2] kb-scribe 照第 3 步 doc-decisions --write，不带声明
           ✗ 子 agent 不跑整轮：这一条是门禁 doc-decisions.sh 的全部格（不带 --check / --item）。交回主 agent——在交回报告「要主 agent 跑的命令」一节写明这条命令、四段声明与为什么要整轮；自己只跑点名的格、项或测试目标
[2] kb-scribe doc-decisions --write，带 whole 声明
[0] kb-scribe doc-decisions --write --check decision-items-sync，带 items 声明
```

能过的写法只有带 `--check decision-items-sync`、声明成 items 的那一种；门禁文件头允许这么写：`.claude/gate.d/doc-decisions.sh:39`：「--write [--check decision-items-sync] [项目根]」。定义三处都没写这一种，写回员照定义写会被拒，照拒绝说明「交回主 agent」就交不出产出一节要的「第 2、3 步 `--write` 的原样末行与退出码」。主 agent 自己照 `.claude/main-agent.md:96` 跑同一条也会被拒一次（没带声明；`out/scope_probe.out` 第四格 `[2] 主 agent doc-decisions --write，不带声明`），能补声明再跑。

四句：
- 分不分辨臂：这一轮没有臂；它分辨「定义写的命令」与「钩子放行的命令」，两者在这一格上不同。
- 被判的系统看不看得到：看得到，判法只看命令文本与 `agent_type`。
- 满足判据的哪一分句：K2 第一条「哪一步会被钩子拒」。
- 改法在这一格上中不中：见下。

改法（只在我的模型上量过、被攻过零轮）：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| kb-scribe 三处写成 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=…; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync` | A1 写回员那一半 | 量过（上面第三格 `[0]`） |
| 钩子把 `--write` 当生成一步、不要声明（同 `--list`） | A1 两半 | 推的（钩子归 tooling-writer，没改没跑） |
| main-agent 第 29、96 行补声明 | A1 主 agent 那一半 | 推的 |

推翻条件：范围声明闸的判法改成认 `--write` 为免声明形态，或 `agent_type` 不再随子 agent 的 Bash 调用送进钩子，第一、二格就会变 `[0]`。

### A2　攻方腿 3b 许可的原型跑法被重型测试闸拒

步骤（攻方腿照自己的定义走）：

1. 3b 要它调 checker 档包里的枚举函数：`.claude/agents/three-way-attack.md:35`：「调 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 里每一段都展开的」；又说这样跑不算重型：`.claude/agents/three-way-attack.md:36`：「在自己的原型里这样调枚举函数跑小流不算重型」；测试目标不能放进 checker 档包（同一行）。harness 包放不进去（checker 档包依赖 harness，`crates/singlefs-checker-tier/Cargo.toml:12`：「singlefs-harness = { path = "../singlefs-harness" }」），所以原型只能是另一个依赖 checker 档包的包。
2. 写范围许可的跑法：`.claude/agents/three-way-attack.md:53`：「副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`」。
3. 重型测试闸把链接 checker 档包的包上起的 `cargo test` / `cargo run` 判成重型：`.claude/hooks/heavy-test-guard.sh:29`：「链接它的包（探针包、原型包：Cargo.toml 的依赖表里写它、或经 path 依赖间接依赖它）：cargo test、cargo run、cargo bench 的范围」；共用约束同样：`.claude/agent-common.md:82`：「依赖 checker 档包 `singlefs-checker-tier` 的别的 crate（工作区外的探针 crate 也算）跑测试、`cargo bench` 或 `cargo run`，同样按重型拒」。

量出来的（`out/heavy_probe.out` 原样）：

```
[2] three-way-attack 在调 enumerate_crash_states 的原型包里 cargo run（照 3b 与写范围）
      ✗ 重型测试被拒：cargo run 起的包 <输出目录>/proto-crate 依赖 singlefs-checker-tier（探针包链接 checker 档，里面能起崩溃枚举，默认只在提交时跑；按包判，不看它挑哪个目标）（checker 档）：three-way-attack 不跑「checker 档」
[2] three-way-attack 同一原型包 cargo test --test，带 target 声明
[0] three-way-attack 同一原型包只 cargo build
[0] three-way-attack 直接执行原型包编出来的二进制
```

放行的只剩「build 之后直接执行二进制」，而闸的文件头说这是它看不见的地方：`.claude/hooks/heavy-test-guard.sh:35`：「直接执行探针包编出来的二进制看不出链接了什么，罩不住」；共用约束又不许换写法绕：`.claude/agent-common.md:56`：「被拒了不换写法绕过去」。照定义走的攻方腿两条路都错：照写范围跑被拒；改成直接执行，等于钻闸的空子。这个矛盾在这一批之前就有（旧句「调同一份文件里」也指 checker 档包里的文件），这一批改的正是 3b 这一句的路径，没碰它。

四句：分辨的是「定义说不算重型」与「闸判重型」；闸看得到（读原型包的 `Cargo.toml`）；满足 K2 第一条「会被钩子拒」，以及第二条「改后句与别的定义、规则冲突」；改法见下。

改法（只在我的模型上量过、被攻过零轮）：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 3b 与写范围明写「调枚举函数的原型只 `cargo build`，经内存包装直接执行编出来的二进制」，并在 heavy-test-guard 文件头把这种用法登成许可而不是「罩不住」 | A2 | 推的（量过的只是闸现在放行它：上面第三、四格 `[0]`） |
| 3b 改成只调 harness 侧的计数与盘（`closed_form_state_count`、`SparseDevice`），不调 checker 档的枚举函数 | A2，但丢掉崩溃状态全展开那一条，与共用约束「崩溃点测试准确率优先」冲突 | 推的 |
| 闸给「攻方原型、状态数在约 10⁶ 以内」开例外 | A2 | 推的；闸归 tooling-writer |

推翻条件：重型测试闸改成不判工作区外的原型包、或 3b 改成不调 checker 档包，第一、二格变 `[0]` 或这一格没有对象。

### A3　执行员照 1b 裸跑 `cargo test` 被拒

`.claude/agents/experiment-runner.md:31`：「跑单测（`cargo test`）、`cargo run` 与装置二进制」——在 `scripts/experiments/` 下不点名目标的 `cargo test` 是那一个包的全部测试目标，判成整轮（`.claude/hooks/run-scope-guard.sh:44` 同上），子 agent 拒。量出来的：

```
[2] experiment-runner 照 1b 在 scripts/experiments 下 cargo test，不带声明
           ✗ 子 agent 不跑整轮：这一条是cargo test 不挑目标（整工作区，或当前目录那个包的全部测试目标）。交回主 agent——在交回报告「要主 agent 跑的命令」一节写明这条命令、四段声明与为什么要整轮；自己只跑点名的格、项或测试目标
[0] experiment-runner scripts/experiments 下 cargo test --bin，带 target 声明
[0] experiment-runner 第 3 步 mutate.sh，不带声明
```

第 3 步的 `mutate.sh` 不受这道闸管（`.claude/hooks/run-scope-guard.sh:64`：「scripts/mutation/ 下的变异驱动（不在要声明的那张表里）」），所以执行员撞墙的只在单测那一步。实验装置的单测本来就只在一个 bin 里，改成 `cargo test --release --bin e<号>-<英文名>` 加 target 声明就过（上面第二格，量过）；这一句这一批没改。推翻条件：钩子不再按 `agent_type` 拒整轮，或 `scripts/experiments` 下裸跑被认成点名目标。

### A4　变异分诊第 5 步漏了「本次未跑」与「基线不绿」

这一批给第 5 步补了 `⚓` 与「锚点失效」，同一句里判 crates 变异门禁每一条的次序仍是：`.claude/agents/mutation-triage.md:34`：「那一行出现了（跑了没红，或跑到它、跑完它之后进程被杀）记没红」。规则在这之前多一支：`.claude/rules/mutation-sampling.md:79`：「那一行是 `ok`、输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它」（记「本次未跑」，不算抓到、不算没红）；门禁实现它：`.claude/gate.d/checker-tier-crates-mutation-replay.sh:103`：「点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红，整道不因它判红」；样本里另起一行报：`.claude/gate.d/fixtures/checker-tier-crates-mutation-replay.sh/not-run/expect:5`：「本次未跑：1 条（点名的测试在本机打了 NOT_RUN_NO_GPU_ADAPTER 后返回成功」。

历史：在没卡的机器上跑整表（主 agent 照提交阶段跑、把输出路径给分诊）→ GPU 对拍那几条点名的测试打 `NOT_RUN_NO_GPU_ADAPTER` 后 `ok` → 门禁整道绿、另起一行「本次未跑：N 条」→ 分诊照第 5 步，测试那一行出现了，记没红；它的「几栏加起来要等于表的条数」（`.claude/agents/mutation-triage.md:34`：「几栏加起来要等于表的条数」）也因为多出来的这一类对不上。结果：分诊报出门禁自己没报的「没红」，交给主 agent 去补取样点。

同一道门禁派变异之前先核基线：`.claude/gate.d/checker-tier-crates-mutation-replay.sh:90`：「所以派变异之前先跑基线，不是恰好一个 ok 的行记」（「基线不绿」、整道判红）。分诊第 4、5 步一个字没提（`grep -c '基线不绿\|本次未跑\|NOT_RUN' .claude/agents/mutation-triage.md` 输出 `0`），整道红在基线上时分诊照「计数：」行的三个数可能全是抓到、照报绿。

四句：分辨「门禁的判法」与「分诊的判法」；分诊读得到那一行（门禁输出里有）；满足 K3 第一条（这八份里引着已变的现状而这一轮没改）与 K2「会漏做」；改法：第 5 步在「那一行出现了」之前插入规则第 79 行那一支，第 4 步补「基线：…基线不绿 N 行」那一行要为 0（推的，没实现没跑）。推翻条件：门禁的「本次未跑」分支被删，或分诊另有一处（别的规则被它整份读）规定了这一支——我查的是 mutation-triage.md 全文与它点名的 mutation-sampling.md。

### A5　全量弹窗条件被摘短

改后：`.claude/main-agent.md:10`：「按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上」。D13 原文是两半：`.claude/kb/decisions/13-验证路线.md:256`：「对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算」，同一行接着说清单由阶段打：`.claude/kb/decisions/13-验证路线.md:256`：「阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判」。

走错的两处：
1. 条件放宽：只加了一条新流、从没判过的块（键对不上而判器没变）照 main-agent 也算「动了」，照 D13 不算。主 agent 会多弹窗；方向偏保守，但它与引的条款不是同一句，按 `.claude/rules/three-way-inference.md`「引 kb 里的条目要整行抄，不许摘句——三处都管」算摘句。
2. 不知道去哪儿数：main-agent 没点名入口。仓里能数这一类的是崩溃点重放入口的只读查询，`scripts/crash/crash-point-replay.sh:21`：「同上，末尾再打几行」（`--list-items --with-keys`），`scripts/crash/crash-point-replay.sh:22`：「一种判器的结论都没有）、几块键相同，与两种判器这一份的判法单元指纹」。它打的是块数与指纹，不是「变了的判器单元清单」：`grep -rc '判器单元'` 在 `.claude/gate.d/checker-tier-crash-point-replay.sh`、`scripts/crash/crash-point-replay.sh`、`crates/singlefs-checker-tier/src/bin/crash_library_reader.rs` 三份里都是 `0`。所以 D13 那半句「阶段把…清单打出来」今天没有逐字对上的输出，主 agent 要么照 with-keys 的「只在别的判法单元指纹下判过」那一栏判，要么照 main-agent 的放宽版判，两种读法的弹窗次数不同（推的）。

改法（推的，被攻过零轮）：main-agent 第 10 行写成「只指路不写数」——「全量弹窗的条件见 D13（验证路线） 已定项 9 第 256 行那一句，数法是 `bash scripts/crash/crash-point-replay.sh --list-items --with-keys` 的『只在别的判法单元指纹下判过』那一栏」。推翻条件：有阶段真打出「变了的判器单元」清单（我只 grep 了上面三份）。

### A6　「不换树」与驱动做的事相反

改后：`.claude/main-agent.md:94`：「不换树、不按提交选条」。驱动文件头：`scripts/mutation/broad-mutation.py:12`：「换进库目录的专用树」（不带选条参数的剪枝跑法，把 `--source` 那棵工作树换进去）；commit-stages 第 2b 步 ③ 同样：`.claude/skills/crash-test/commit-stages.md:26`：「驱动先把工作区逐文件换进库目录的专用树」。skill 说的是「不换版本」，main-agent 写成了「不换树」。照 main-agent 读的主 agent 会以为驱动判的就是暂存区，不必先核工作区等于暂存区；而 2b 步 ① 正是为这个设的：`.claude/skills/crash-test/commit-stages.md:24`：「驱动判的是主工作区（`--source` 默认本仓根），要它等于暂存区」。main-agent 那一句指到了第 2b 步，主 agent 照 skill 做就不会错，所以算轻。改法（推的）：「不换版本、不按提交选条；判的是主工作区，先照第 2b 步 ① 核它等于暂存区」。推翻条件：驱动改成直接读暂存区、不再换进专用树。

### A7　「只改注释」丢了例外

改后：`.claude/main-agent.md:52`：「整段只改测试代码或只改注释的不开代码轮」。规则原文：`.claude/rules/implementation-workflow.md:10`：「文档注释里会编成 doctest 的代码块、文档注释与 `// SAFETY:` 注释的增删照代码算」。历史：一段实现只在 `crates/singlefs-core/src/` 加了几条 `///` 文档注释 → 主 agent 照 main-agent 第 52 行不开代码轮 → 提交时门禁 doc-process-records 的 crates-adversarial-review 格（照同一份 comment-only-change.py 判）判红，要么回头补一轮、要么登豁免。括注指到了规则，读规则的主 agent 不会错；只读这一句的会错（推的）。改法：「只改测试代码、或只改了注释（射程与例外照 … 第 2 步）」——把「只改注释」也收进括注的指路里，不在这里写短版。推翻条件：comment-only-change.py 不再把文档注释增删算代码。

### A8　四处合入验证的写法不是同一套

- `.claude/main-agent.md:7`：「一段实现写完之后的合入与验证（验证段，「写与验分两段」一节）」——这一批把第 7 行指到验证段，写法是 `[--skip-apply]`（可选）。
- `.claude/main-agent.md:29`：「（写阶段已打过的补丁带 `--skip-apply`）」——有条件。
- `.claude/main-agent.md:50`：「（补丁已打，只编、只验；给全补丁目录是为了重编核对）」——验证段第 1 步无条件 `--skip-apply`。
- `.claude/main-agent.md:90`：「打全部补丁（`apply-writer-patch.py`，核不过整批停）」——不带 `--skip-apply`，由它来打。
- 写段只给有依赖的那批打补丁：`.claude/main-agent.md:41`：「有依赖的：前一批实现员交回后，补丁只打不验」。

历史：一段实现只有一批、彼此不依赖 → 写段第 2 步不适用，没人打补丁 → 进验证段，照第 1 步带 `--skip-apply` → 脚本只读补丁目录算改到的包、不打：`scripts/gate/merge-and-verify.sh:44`：「--skip-apply 时照样读给的补丁目录，只是不打」→ 它把补丁改过的文件刷新时刻、重编（补丁没打，刷的是没改过的原文件）→ 在没打补丁的树上跑完测试报绿。脚本里我没找到「`--skip-apply` 时核补丁已经在树里」的那一步（`grep -n 'skip-apply\|SKIP_APPLY' scripts/gate/merge-and-verify.sh` 命中的都是用法、算包与自证，推的，没跑）。第 90 行那一张表的写法能避开这条，验证段第 1 步的写法避不开；main-agent 第 36 行要主 agent「严格照这一节排次序」，照的就是第 1 步。

四句：分辨「补丁在树里」与「不在」；脚本当时看不到（不核）；满足 K2 第二条「改后句与同一文件别处冲突」（body 点名的例子正是第 7 行与第 29、90 行）；改法（推的）：验证段第 1 步写成「写段打过的带 `--skip-apply`，没打过的不带」，或脚本在 `--skip-apply` 时对每份 `crates.patch` 跑 `git apply --check --reverse`、不过就红。推翻条件：写段另有一处要求每一批交回都先打补丁（我读的是 main-agent 第 36–55 行全段）。

### A9　范围声明只补了一半

这一批给 agent-common 第 56 行补的是轻型三条：`.claude/agent-common.md:56`：「三条开头都带范围声明 `SINGLEFS_RUN_SCOPE=」；同一句里 harness 档那一截没带：`.claude/agent-common.md:56`：「harness 档照跑：自己动到的 `singlefs-harness` 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）」。钩子对这类也要声明（target），没带就拒（`out/scope_probe.out`：`[2] implementation-writer 跑动到的测试二进制，不带声明`、`[2] three-way-attack 照写范围 cargo test -p --test，不带声明`）。第 87 行的拒绝清单写了这一类，所以照共用约束整份读的子 agent 能自己补；但同一句里一处带一处不带，容易照不带的抄。

实现员第 4 步要把验证「合成一条命令」：`.claude/agents/implementation-writer.md:40`：「交回前的验证合成一条命令在末尾跑一次」。只在开头 `export` 一份声明时，形式化那一段种类对不上被拒（`out/scope_probe.out`：`[2] implementation-writer 第 4 步一条命令，开头 export 一份 target 声明`，拒绝说明「声明的是点名的测试目标（target），命令跑的是轻型（quick）」）；每段各带自己的声明就放（`[0] implementation-writer 第 4 步一条命令，每段各带自己的声明`）。第 4 步没说要逐段带。

仓外检出记录（`~/.local/share/singlefs/work/agent-hook-detections.jsonl`，run-scope-guard.sh 的行，脚本数的：共 366 行，被拒 37 行，其中子 agent 28 行）里，`three-way-attack`、`three-way-forward`、`three-way-defense`、`investigator`、`implementation-writer` 都在 2026-10-15、2026-10-16 因「没带范围声明」或范围数写错被拒过，形状就是上面这一类。这是改动之前的记录，只说明这条路有人走，不当改后的证据。

改法（推的）：第 56 行 harness 那一截写成带声明的样子；实现员第 4 步加「串成一条时每一段各带自己的声明，不用一份 export」。推翻条件：钩子不再要 target 类声明。

## K3：漏改的地方

### B1　八份文件里没改到、照抄就被拒的命令句

| 位置与句子 | 照抄会怎样 | 标 |
|---|---|---|
| `.claude/main-agent.md:94`：「SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-crash-point-replay.sh --full <它的根>」 | 没带范围声明，主 agent 被拒一次（`out/scope_probe.out`：`[2] 主 agent 照 main-agent 第 94 行起 --full，不带声明`） | 量过 |
| `.claude/main-agent.md:29`：「各自不跑 `bash .claude/gate.d/doc-decisions.sh --write`，主 agent 在全部交回后跑一次 `--write`」 | 主 agent 照跑被拒一次（A1 第四格） | 量过 |
| `.claude/main-agent.md:96`：「主 agent 自己写规格、跑 `scripts/kb/replace-batch.py` 与 `bash .claude/gate.d/doc-decisions.sh --write`」 | 同上 | 量过 |
| `.claude/agents/three-way-attack.md:53`：「副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`」 | 不带声明被拒（A9），链接 checker 档包时被重型闸拒（A2） | 量过 |
| `.claude/agents/experiment-runner.md:31`：「跑单测（`cargo test`）、`cargo run` 与装置二进制」 | A3 | 量过 |
| `.claude/agents/implementation-writer.md:34`：「打印命令（带 `--include-ignored`，慢档用例也跑得到），经内存包装起」 | 打出的命令不带声明（`scripts/gate/rerun-failed-tests.py:4`：「从一份 cargo test 日志里取出失败的用例，按测试目标各打印一条只跑它们的命令」，`grep -c SINGLEFS_RUN_SCOPE scripts/gate/rerun-failed-tests.py` 输出 `0`），原样起会被拒一次 | 推的 |
| `.claude/agents/kb-scribe.md:32`、`:33`、`:35`，见 A1 | 子 agent 带不带声明都拒 | 量过 |
| `.claude/agents/mutation-triage.md:34`，见 A4 | 记错一类结局 | 推的 |

`.claude/main-agent.md:50` 那一句列了要带声明的命令：「起验证的命令（这一条、轻型三条、`cargo test` 点名的包与目标、`scripts/gate/gate-staged.sh`）开头都带范围声明」——没列门禁阶段的单跑（`doc-decisions.sh --write`、六道重阶段、`--full`），而 main-agent 别处正是这样起它们的。

### B2　别的定义该同步而没同步

在 sweep、investigator、experiment-designer、three-way-forward、three-way-defense、clerk、prior-art、three-way-materials 八份里数这一批改过的说法，全是 0：

```
KS_THREADS	0
五栏	0
已还原，基线仍全绿	0
crash.rs	0
sync-tree	0
路径清单	0
merge-and-verify	0
doc-decisions	0
cargo test	0
```

（命令：`for p in …; do cat <八份> | grep -cF -- "$p"; done`，末一格 `grep -c` 命中 0 退 1，整条退 1。）这八份没有旧说法要改。漏的只有一处做法：`.claude/agents/investigator.md:31`：「先原样复现：用输入给的命令在仓副本里」——输入给的是整包或整工作区的 `cargo test` 时（`out/scope_probe.out`：`[2] investigator 原样复现整包 cargo test，带 whole 声明`），子 agent 跑不了，定义没说要收窄到目标、还是写进「要主 agent 跑的命令」交回。改法（推的）：加一句「输入命令是整包、整工作区或整道门禁的，收窄到点名的目标或格再复现；收不窄的写进要主 agent 跑的命令」。

### B3　已知的钩子文件头旧句

`.claude/hooks/run-scope-guard.sh:32`：「合入后验证  scripts/gate/merge-and-verify.sh：whole（还不认只跑改到的包」——判法里拒绝说明已经是新写法（`lib_run_scope.py` 的 NARROWING_LINES 那一行写「挑包在脚本里做…种类照旧写 whole」），只有文件头旧。主 agent 在正文第二节已列，归 tooling-writer，这里只确认还在。

## 没打中的形状

逐处对着现文试过、没打中的（取样范围是正文第五节 diff 的每一处）：

- 线程变量 `E168_THREADS`：`scripts/hosts/capped.sh` 第 21 行的变量清单里有它，E168 装置读它，照 agent-common 第 62 行设成 N 不会错。
- 副本拷 `.claude/heavy-thread-default`：checker 档构建脚本闭包读到的 `crates/` 之外的数据文件只有这一份（`crates/singlefs-checker-tier/src/crash_identity.rs` 里 `the_data_files_outside_the_crates_include_the_heavy_thread_registration` 那条测试点名的）；`[workspace] members` 全在 `crates/` 下；照新拷贝清单编得过（推的，没编）。
- `mutate.sh` 的「全部复用」与「已还原，基线仍全绿」：两句都在（第 1961、2342 行），全部复用那一支的「计数：」行同样带「锚点失效」一栏；复用哨兵每趟抽一条强制现判，所以「全部复用」只在哨兵抽不出时出现，分诊与执行员两种收尾都认，不会走错。
- crates 变异门禁「计数：」行的栏：样本 `not-run/expect` 那一行有「被总上限挤掉」「排不上没跑」「scope 起不来没跑」「锚点失效 0 行（共 2 条）」，与分诊第 5 步新写的四栏一致。
- 崩溃枚举用例登记：`crash-case-check.py` 的 placement 照合表点名判，与实现员 7a、verification.md 同一套；合表在 tooling-writer 的写范围里（`.claude/gate.d/**`）。
- 实验页指针写法：`.claude/product-line-prefixes` 登了 `E7RESULT` 前缀，文件头写「判它的检查还没接」；执行员第 7 步的写法与它一致，门禁 quoted-result-lines 只判以 `E7RESULT ` 起头的整行，指针行不判，不会被判红。
- 定时闸挂的六个事件：`.claude/settings.json` 里 matcher 是 `CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash`，与 agent-common 新写的那条一致；它不拒 `proc.py wait` 与带真活的等待循环，与 agent-common 第 74 行给的出路不冲突。
- 派发闸新写的「许可跑重型的句子」：判法去掉引号里的转述与「重型测试：不跑」那一截，提到主 agent 的子句不判；照骨架写的派发提示（含这一轮给我的）不会误中（推的，没喂判法）。
- 实现员 4a 的「受影响的层 0 流与崩溃枚举用例」：旧名，但 `scripts/gate/apply-writer-patch.py` 第 78 行 `CHECKER_SECTION` 认的就是这几个字，代码里的字符串按 term-renames 豁免；只改定义一边才会断，所以不算漏改。
- `merge-and-verify.sh` 的 `--all-packages`、`--skip-apply`、`--memory`、`--threads`：文件头第 6 行都有，与 main-agent 四处写的参数名一致（参数名一致，用法不一致的是 A8）。

## 这条腿自己的限度

- 「量过」只量到钩子这一层：我把命令当 hook 输入喂给钩子，看的是放不放、拒绝说明写什么；没有真让一个写回员、执行员、攻方腿去跑，没看它们被拒之后会不会照拒绝说明自己改对。A1、A2 的「重」是因为照拒绝说明改也走不通（A1 的出路「交回主 agent」交不出定义要的产出；A2 能过的那条是闸自认罩不住的缝），不是因为被拒一次。
- 钩子输入里的 `agent_type` 是我按类型名填的；Claude Code 真送进来的字段名与值我没核（判法读的是 `agent_type`，仓外检出记录里子 agent 的行带着这个字段，相容）。
- A4、A5、A7、A8 是对读原文推的：没跑 crates 变异门禁、没跑 `--list-items --with-keys`、没跑 merge-and-verify。A8 那段「在没打补丁的树上报绿」是按脚本文件头与 grep 推的，没造一棵树真跑。
- 仓外检出记录只用来说明「这类拒绝真发生过」，它在仓外、是改动之前的、不进 kb。
- 只攻了正文第五节的 diff 与它引到的脚本、钩子；没读 R2 通读报告（`research/prompts/rot-2026-10-17/reports/read-R2.md`），K3 第一条「R2 判『不改』的句子」是按八份文件现文自己找的，不是逐条复核 R2 的判定。
- 这是第一轮攻方的一次抽样；「没打中」那一节照 `.claude/rules/three-way-inference.md` 一次不算数。

## 没做什么

- 没判 K1（正推那一格）；没替主 agent 采纳；没改任何定义、钩子与规则。
- 没建仓副本、没起编译、没跑门禁阶段、没跑重型与轻型。
- 要主 agent 跑的命令：没有。核我量过的那几格只要复跑上面那一条 `rerun.sh`。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-defs-r1-opus/` 里留着探针的第一版（`probe/`，含只有 `Cargo.toml` 与空 `main.rs` 的 `probe/proto-crate`，没编、没有 target）与两趟复跑的输出（`rerun-first/`、`rerun-second/`，各含同样的 `proto-crate` 桩）；都不是编译目录或仓副本，留给主 agent 核对时对照，用完可删。
