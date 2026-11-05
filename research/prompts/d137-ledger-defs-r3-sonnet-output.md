# d137-ledger-defs-r3 辩方腿（格 R）报告（2026-11-02）

立场：复核前两轮判决与落地，核数。被复核：`research/prompts/d137-ledger-defs-r1-main-verification.md`（8b9b48ad）、`research/prompts/d137-ledger-defs-r2-main-verification.md`（1ed468e2）。没读禁读清单里的攻方报告。

## 此刻文件的 sha256 前 8 位（与开工快照 `research/prompts/d137-ledger-defs-r3-snapshot/files-sha256.txt` 逐份对过）

19 份被判文件此刻的 sha256 与快照逐份相同，没有对不上的：

| 文件 | 前 8 位 | 文件 | 前 8 位 |
|---|---|---|---|
| `.claude/agent-common.md` | 54ff4c79 | `.claude/rules/verification.md` | c10aa10d |
| `.claude/agents/clerk.md` | bebb5887 | `.claude/rules/implementation-workflow.md` | 4b57216e |
| `.claude/agents/experiment-designer.md` | 2a2fa725 | `.claude/rules/mutation-sampling.md` | 3dad8a4b |
| `.claude/agents/experiment-runner.md` | 83d9a62d | `.claude/skills/crash-test/commit-stages.md` | 52b08d58 |
| `.claude/agents/implementation-writer.md` | cc371914 | `.claude/skills/crash-test/SKILL.md` | aa33115e |
| `.claude/agents/investigator.md` | f7cd45cb | `scripts/hosts/compile-then-swap.py` | 6328769d |
| `.claude/agents/kb-scribe.md` | a7f6c098 | `scripts/gate/merge-and-verify.sh` | cc7ae95d |
| `.claude/agents/mutation-triage.md` | 4c81979a | `.claude/gate.d/doc-experiments.sh` | 7e0f7a6b |
| `.claude/agents/three-way-attack.md` | 003ee15e | `.claude/main-agent.md` | 589c7787 |
| `.claude/agents/tooling-writer.md` | f0028544 | | |

另引到的、不在快照里的文件：`.claude/gate.d/checker-tier-crash-point-replay.sh` e8ce42b9，`records/2026-10-29-subagent拆分提案.md` b86bf854，`scripts/mutation/crates-mutation-rows.py` 4a31ff70，`.claude/hooks/runner-dispatch-guard.sh`（只读它第 68 行）。

## 一、判决里报的任务号与数（`task-server/task log <号> --all` 现读，日志拷在 `~/.local/share/singlefs/work/d137-ledger-defs-r3-sonnet/log-<号>.txt`）

| 判决里的话 | 现读 | 判 |
|---|---|---|
| 第一轮：任务 4769 末行「✓ 快照 d137-ledger-defs-r1：清单点名 11 份，倒推出 11 份，sha256 对上 11 份」 | 日志第 24 行就是这句，结局 succeeded 退 0 | 站得住 |
| 第一轮：任务 4770 造七个世界，每个世界一行 `world_built … untracked=1` | 日志 7 行 `name=world_built`，world 名依次 control-single-line-red、control-single-line-green、continued-bin-on-next-line、continued-features-on-next-line、marker-without-judge、alias-import-judge、declared-cargo-test-green | 站得住 |
| 第一轮：任务 4870 七行 rc 依次 1、0、77、1、1、0、0，两个对照是 1 与 0 | 日志 7 行 `name=judge_world … rc=` 依次 1、0、77、1、1、0、0；对照（control-single-line-red=1、control-single-line-green=0） | 站得住 |
| 第二轮：任务 4929 清单 16 份、倒推出 15 份、sha256 对上 15 份，`.claude/main-agent.md` 倒推不出 | 日志第 20 行「清单点名 16 份，倒推出 15 份、sha256 对上 15 份；倒推不出 1 份、对不上 0 份」，第 12 行点名 `.claude/main-agent.md`；结局 command-failed 退 1（判决没写退码，这是倒推不出一份的正常结局） | 站得住 |
| 第二轮：任务 4927 复跑 agent-definition-shape 绿 | 日志「agent-definition-shape：绿」，14 份定义 | 站得住 |
| 第二轮：任务 4930 造样本与变异体 | succeeded 退 0；日志含三个世界与 `mutant_built … changed_lines=2` | 站得住 |
| 第二轮：任务 4932 三行 rc 为 77、0、1 | 日志依次 `rc=77`、`rc=0`、`rc=1`，世界名依次 comment-backslash-swallows-command、comment-backslash-hides-second-command、comment-without-backslash-control（对照判红）；`judge_inputs gate_sha256=690a35ef` 是第二轮快照那一份门禁，不是此刻的 7e0f7a6b | 站得住（跑的是旧门禁，见下「核不动」第 1 条） |
| 第二轮：任务 4931 探针 `features-twice-gpu-first rc=0 … gpu_in_every_call=no`，`features-equals-gpu`、`short-flag-gpu`、`all-features` 都 rc=2 | 日志逐字如此（features_words=extra|extra；其余三行 rc=2 calls=0）；自证「查了 16 项」是改前的脚本 | 站得住 |
| 第二轮：任务 4933 变异体探针 `features-gpu … gpu_in_every_call=no`、`selftest_in_process rc=0` | 日志 `spelling=features-gpu rc=0 calls=2 gpu_in_every_call=no features_words=-|-`、末行 `name=selftest_in_process rc=0` | 站得住 |
| 第三轮背景：`compile-then-swap.py` 任务 5090–5095，基线退 0、开关红格与点名逐格相同 | 5090 基线 succeeded 退 0（日志「自检通过（查了 23 项，假 cargo 一共被调 36 次」，此刻自证表也是 23 项）；5091 红 1 格 features-twice-gpu-first；5092 红 7 格（gpu-feature-passed-through、features-twice-gpu-first、features-equals-spelling、features-short-flag、features-short-flag-attached、features-quoted-list、all-features-passed-through）；5093 红 4 格（features-equals-spelling、features-short-flag、features-short-flag-attached、all-features-passed-through）；5094 红 1 格 features-missing-value。与文件头 `scripts/hosts/compile-then-swap.py:117-128` 写的红格逐个相同；5095 探针五种写法 rc=0，`all-features` 一行 gpu_in_every_call=no 是因为探针只数 `--features gpu`、`--all-features` 本来不带这个词 | 站得住 |
| 第三轮背景：`merge-and-verify.sh` 任务 5119–5123，基线退 0、开关红格与点名逐格相同 | 5119 succeeded 退 0（「自检通过（查了 55 种）」）；5120 红 1 格（读不出 PRODUCT_GPU_PACKAGE_NAME）、5121 红 1 格（名不是成员）、5122 红 2 格（补丁只改产品 GPU 包、名不是成员），5123 红 12 行；与文件头 `scripts/gate/merge-and-verify.sh:212-230` 逐格对：product-gpu-unreadable-as-unaffected 1、non-member 1、package-ignored 2，checker-quick-skipped「十一格＋连带一格」=12 | 站得住 |
| 第三轮背景：门禁单格 rules-named-by-definitions、agent-definition-shape、fixture-claims、break-switch-headers 绿（任务 5106、5126） | 5106 日志「2 格里 2 格绿（rules-named-by-definitions agent-definition-shape）」；5126 日志「2 格里 2 格绿（fixture-claims break-switch-headers）」 | 站得住 |

没核到的：第一轮「攻方模型复跑」一节写任务 4770 造世界、任务 4870 判；`task status` 里 4770 的命令是 `rerun.sh --repo … --frozen … --work …/rerun-1`，4870 的命令是 `judge-worlds.sh …/rerun-1/worlds`，与判决写的一致。判决没写 4930/4931/4932/4933 之外的第二轮任务号，没有别的号要核。

## 二、第一轮判「采」并已落地的各条（逐条现查此刻的文件；sha256 前 8 位见开头一表）

结论词：站得住 / 站不住 / 核不动。「引」都是我 `grep -nF` 或 `sed -n` 在此刻文件里现查到的行；行号是此刻的行号，判决里写的行号（例 `agent-common.md:98`）在改过之后已挪位，以此刻为准。

| 判决条 | 此刻落点（路径:行，sha8） | 做到了判决那一句吗 | 判 |
|---|---|---|---|
| 入库装置经 `compile-then-swap.py` 编、带登记的特性（正推 1、攻 A1） | `scripts/hosts/compile-then-swap.py:330-336`（6328769d）`build()` 在 `features` 非空时把 `--features <合起来的>` 交给 `cargo build`，`all_features` 时交 `--all-features`；`swap()`（:382）把两条 cargo（build 与 test --no-run）都带上；判决写的旧位置 `:300` 是改前的行 | 做到。任务 5090 自证 23 项绿、5091-5094 四个开关逐格红（见第一节） | 站得住 |
| 按判段入口认，不按 `crash_amplification::` 字面（攻 A2） | `.claude/gate.d/doc-experiments.sh:2977-2981`（7e0f7a6b）`JUDGE_MODULES` 两个模块、`JUDGE_ENTRIES` 13 个名字；`:3070-3076` `judge_entries_called` 要「提到两个模块之一」且「写着表里的名字」，别名导入因此认得出；`:3220` `has_gpu` 判特性；`experiment-designer.md:31`、`experiment-runner.md:40` 的两句也改成按判段入口认 | 做到（字面）。门禁此刻版本的样本档还没跑，见「核不动」 | 站得住（读代码）；样本核不动 |
| 跑前登记那一格认反斜杠续行（攻 A3） | `doc-experiments.sh:3065-3067` `continued()`；`:3181-3184` 围栏里、围栏外行首是命令的行都并，整行注释（`comment_line`，:3060）断开；`:3185` 起命令行尾注释截掉 | 做到：并成一条再认，注释里的反斜杠不续行 | 站得住（读代码）；样本核不动 |
| `grep -c NOT_RUN` 数不到另写文件与 `.gz`（攻 A4） | `experiment-runner.md:40`（83d9a62d）：任务日志 `task log <号> --all | grep -c NOT_RUN`，文件 `grep -c`、`.gz` 用 `zgrep -c`，目录逐份数、不对目录本身 `grep -c`，另写的 `--out`、`--output-directory` 文件也算 | 做到 | 站得住 |
| 设计员「整条写成 submit」要带范围声明（攻 A5） | `experiment-designer.md:31`（2a2fa725）「整条写成 `SINGLEFS_RUN_SCOPE='…' task-server/task submit -- <命令>`（`cargo test` 点名包与目标的带四段范围声明，不点名目标的整包 `cargo test` 也要声明…；`cargo run` 不要声明）」；`.claude/hooks/run-scope-guard.sh`（c520949e）`:43` 写「不挑目标的整包、整工作区）是 whole」即整包 `cargo test` 要声明，`:58` 把 `run` 列在不要声明的子命令里 | 做到，与钩子文件头相符 | 站得住 |
| `implementation-writer.md` 「五处」（正推 2、攻 A7） | `implementation-writer.md:47`（cc371914，7a 项）：「每条都要的三处…有条件的四处…」，共七处名字，不是判决字面的「五处、不写条件」。第二轮已把这个差异判「不采，落地比判决字面更准」 | 我对着 `scripts/crash/crash-case-check.py` 判法段（placement ③④、must-red、provenance）核：三处必要（stage-inputs 的 `crash-case:` 行、合表点名项、`crates/mutations.tsv` 变异行）对应 placement 与 must-red；ITEMS、轻型变异登记表、`OBSERVER_ENTRIES`、`PROVENANCE_OF_THE_FLOWS` 四处的条件，在 `crates-mutation-light-rows.tsv:4-6`、`crash_point_replay_provenance.rs:97-141`、`crash-case-check.py` provenance 段里都能找到出处。落地比「五处」准 | 站得住（与第一轮判决字面不同，第二轮已判过） |
| `commit-stages.md` 第 19 行出路只重跑一格（正推 3、攻 A8） | `.claude/skills/crash-test/commit-stages.md:19`（52b08d58）「要卡的有三处——crash-case-replay 格查库那一批里带 `gpu` 特性的命令、feature-gated-tests 格里带 `gpu` 特性那一趟、product-gpu-real-card-tests 格的真卡对拍…`--check crash-case-replay,feature-gated-tests,product-gpu-real-card-tests` 只重跑这三格」；对照 `.claude/gate.d/checker-tier-crash-point-replay.sh:189`（e8ce42b9）「要上卡的三处」，三处逐个相同，三格名在 `:591-597` 的 `*_CELL_NAME` 里 | 做到 | 站得住 |
| `main-agent.md` 第 32 行、第 101 行，`SKILL.md` 第 21 行补那一格（正推 3.2、攻 A9） | `main-agent.md:32`（589c7787）「crash-case-replay、feature-gated-tests、product-gpu-real-card-tests 三格…补丁没碰 checker 档包、它依赖的包与产品 GPU 包 `singlefs-background-gpu` 时报本次未跑」；`:101` 有「另有 product-gpu-real-card-tests 一格」；`SKILL.md:21`（aa33115e）「快档两趟…、product-gpu-real-card-tests 一格（产品 GPU 包 `singlefs-background-gpu` 带 `tests-on-a-real-gpu` 的真卡对拍）」 | 做到。`main-agent.md:101` 里特性名两侧是「」、前后各留一个空格（「带 「tests-on-a-real-gpu」特性 的」），只是排版，不影响读法 | 站得住 |
| `merge-and-verify.sh` 只改产品 GPU 包被判没碰（攻 A10） | `scripts/gate/merge-and-verify.sh:648-657`（cc7ae95d）`product_gpu_packages()` 现读门禁里 `PRODUCT_GPU_PACKAGE_NAME` 那一行（真门禁 `checker-tier-crash-point-replay.sh:600` 是 `singlefs-background-gpu`）；`:685-702` 把产品 GPU 包的依赖闭包并进射程，读不出（:688-690）与不是成员（:693-696）都「affected，照跑」并点名 | 做到，两个方向相同；任务 5119-5123 基线绿、四个开关逐格红（见第一节） | 站得住 |
| 「没卡报本次未跑」把服务没交卡与测试找不到显卡并成一种（攻 A11） | `verification.md:35`（c10aa10d）分写两句：「服务没交卡的那一格判红、那一趟不起」与「测试找不到显卡打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报「本次未跑」」；对照门禁 `checker-tier-crash-point-replay.sh:2532-2538` `require_service_card` 没卡打 ✗ 返回 1 | 做到 | 站得住 |
| 分诊定义「六栏」与 `mutate.sh` 九栏对不上（攻 A12 后一半） | `mutation-triage.md:32`（4c81979a）与 `mutation-sampling.md:79`（3dad8a4b）都写「报九栏：抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏，后接…现判、复用、拒算三栏」；对照 `scripts/mutation/mutate.sh:2046` `verdict_count_text` 六栏、`:2030` `LIBRARY_COUNT_TEXT` 三栏，`:2163`、`:2432` 拼成一行 | 字面做到：六加三等于九，次序与 `mutate.sh` 打的一致。但同一句话的后半还留着一个对不上的东西，见第三节 F1 | 九栏站得住；整句有残留（见 F1） |

继续第一轮判「采」：

| 判决条 | 此刻落点 | 做到了吗 | 判 |
|---|---|---|---|
| `agent-common.md` 删半句后仍写「分两种出路」，派发闸现分三条（攻 A6、正推 5） | `.claude/agent-common.md:100`（54ff4c79）「退 2 的拒绝说明分三种出路：…的（环境里设着已删的强制重跑变量）…；有它自己报「没登记」「登记有错」的句子的，先接准入；这几句都没有的，先让 `admission.py` 能跑」；对照派发闸 `.claude/hooks/runner-dispatch-guard.sh:68` 「按输出分三条出路」，及 `:1351-1355`、`:1387-1400` 的两类退 2 | 做到，出路数与闸一致；全仓 `grep -nE '分两种|两种出路'` 在 `.claude/agent-common.md`、`.claude/agents/`、`.claude/rules/`、`.claude/main-agent.md` 里 0 处 | 站得住 |
| read-once 规则点名与定义相冲（攻 A13，正推 6 核不动）：改成按各定义的活取用到的那一节；`clerk.md` 那一句理顺 | `three-way-attack.md:17`（003ee15e）点名 `pushback-discipline.md`「对不上就当场说」「分清「我核过」和「我印象里」」，不点名要写 `.claude/warnings/` 的「警告写在哪」（该规则 `:8`、`:31` 是这两节，`:44` 才是警告节）；`implementation-writer.md:16`、`tooling-writer.md:16` 点名 `machine-first.md`「遇到一条通行准则，走这个流程」「判据」（`:52`、`:61`），不点名带兼容条的「留下的」节（`:84`，兼容条是 `:87` 「对外格式与协议兼容」）；`clerk.md:16`（bebb5887）改成「规则读三样，别的不读：…；派发提示另点名的规则；改 `records/` 下的文件时…」 | 做到：被点名的小节标题都在规则文件里现查得到，带冲突的节没有被点名；`clerk.md` 那一句仍是「别的不读」后面接两个例外，读得通 | 站得住 |
| 第一轮「采」的补充：三份只在共用约束括注里提到的规则，记欠账（一百八十 ②） | 三份规则 `.claude/singlefs-ai-sop/rules/session-wrapup.md`（a4d80b9d）`:68` 「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」由 `implementation-writer.md:16`、`investigator.md:16`、`three-way-attack.md:17` 点名；`.claude/rules/timer-only-remind.md`（14534cc4）`:13` 「不用」由 `tooling-writer.md:16`、`implementation-writer.md:16`、`investigator.md:16` 点名；`.claude/rules/path-moves.md`（cd5fd4c6）`:8` 「怎么做」由 `tooling-writer.md:16`、`:33` 「改一个全仓术语：正文之外还有五处会红」由 `kb-scribe.md:16` 点名；账本 `records/2026-10-29-subagent拆分提案.md:884`（b86bf854）去向句与此逐项相同 | 做到，账本去向句与实际点名一一相同 | 站得住 |

## 三、第一轮判「不采」、记欠账、交别的会话的去向

| 判决条 | 去向与落点（现查） | 判 |
|---|---|---|
| 第一轮：分诊定义只认 `NOT_RUN_NO_GPU_ADAPTER` 一种本次未跑，另一种在 `crates-mutation-rows.py`，等那份落地再跟（攻 A12 前一半，记欠账） | 账本节在：`records/2026-10-29-subagent拆分提案.md:879` 标题「一百八十、三方 d137-ledger-defs-r1 留下的两样欠账」，`:882` 的 ①；调度表 `:99` 一行「一百八十 新登 … ① 等 crates-mutation-rows.py 落地；② 三份规则由定义点名」。判决里的文件名没有行号，现查 `scripts/mutation/crates-mutation-rows.py:207` 「带 gpu 特性的格另有一种记本次未跑（给定状态全交 CPU 判）」、`:4762` 打「本次未跑：N 条（…NOT_RUN_NO_GPU_ADAPTER…或带 gpu 特性而多机配置不在、给定状态全交 CPU 判…）」，欠账说的那一种在源码里确实存在。落点完整，但「等落地」的触发条件现在看是满足了：那份文件此刻在暂存区（`git status` 为 `M ` 即工作区与暂存区相同，sha256 4a31ff70），已是一份新写的版本；分诊定义 `mutation-triage.md:32` 判本次未跑的那一句仍只写 `NOT_RUN_NO_GPU_ADAPTER` 一种。这条欠账现在可以付，由主 agent 判 | 去向站得住；欠账未付、触发条件已到 |
| 第一轮「不采」：定义写 `NOT_RUN`、门禁出路写 `NOT_RUN_`（正推 1 小差异） | `experiment-runner.md:40` 末尾「`NOT_RUN_<原因>` 行与 `CRASH_AMPLIFICATION_NOT_RUN` 行都算」，计数用 `grep -c NOT_RUN`（前缀匹配，两种都数到），不漏判。同形状在别的臂：把 `grep -c NOT_RUN_` 当计数的话漏掉 `CRASH_AMPLIFICATION_NOT_RUN`，所以不采是对的 | 站得住 |
| 第一轮「不采」：工具员报告里 `crash-case-check.py:25` 指不存在小节对不上（正推 2） | 判决说那三处已由跟随改动那一件改成现存小节名，出处 `~/.local/share/singlefs/work/d137-b19-follow-ups/result.md` 第 7 条（该文件 `:136` 标题「### 7 crash-case-check.py 三处不存在的小节名」存在）。现查：`scripts/crash/crash-case-check.py` 里 `grep -c 崩溃枚举用例住哪` 得 0，`:25`、`:598`、`:600` 现写 `.claude/rules/verification.md「门禁管哪一半」`，该小节在 `.claude/rules/verification.md:131`、对应表行在 `:139`；`:602` 的 `test-script-discipline.md「用例原子化：每一条都能单独跑」` 与 `path-moves.md「怎么做」`（`:8`）也都在 | 站得住 |
| 第二轮：入库装置裸二进制要卡（攻 B2、辩 A1），交做统一任务服务的会话；换上后这一路在要卡的入库装置文件头各写一行 | 服务一侧：`task-server/task_server_lib.py:239-244`（4d3f51a3）「编出来的二进制…读仓里唯一那一份同名源…前同样行数里的 `// task-server-gpu-arguments: <词>…`」，`:1921-1952` `declared_gpu_binary`；`gpu_card_configuration.rs:385-398`（ff98c3ce）没给卡又没清场且配置开了 GPU 时退 78，出路写「要卡的入库装置二进制在 src/bin/<名字>.rs 文件头写 `// task-server-gpu-arguments: …`」（判决写的 `:394` 那一支已改，行号挪了）。装置一侧：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1`（`*`）、`e158_root_choice_repair.rs:1`、`e161_crash_state_dedup_and_time_split.rs:5`、`e168_delete_ring_window_borrow.rs:1`，都在前 60 行内 | 服务与四份装置站得住。E171 两份见 F2 |

## 四、第二轮判「采」并已落地的各条

| 判决条 | 此刻落点（路径:行，sha8） | 做到了吗 | 判 |
|---|---|---|---|
| B1 设计员末句仍说门禁按 `crash_amplification::` 认判段 | `experiment-designer.md:31`（2a2fa725）「按判段入口认：调到没编进 `gpu` 就打 NOT_RUN 行的入口，即 `crash_amplification` 的 `run_from_environment` 一族、`GivenStatesEngine`、…；经 `use … as <别名>` 导入再调的照认，只取崩溃点…的不算」。全仓 `crash_amplification::` 字面在 `.claude/agents/`、`rules/`、`skills/`、`agent-common.md`、`main-agent.md` 里 0 处（`grep -rn` 现查） | 做到 | 站得住 |
| B2 裸二进制要卡（交做统一任务服务的会话） | 见第三节最后一行；设计员/执行员侧：`experiment-runner.md:30` 续派提示写二进制绝对路径、产物命令写成 `task-server/task submit -- <二进制…>` | 服务与四份装置做到；E171 两份见 F2 | 部分站得住 |
| B3 执行员数 NOT_RUN 要 `task log <号> --all`；目录产物 `grep -c` 退 2 | `experiment-runner.md:40`（83d9a62d）：`task-server/task log <号> --all | grep -c NOT_RUN`，「不带 `--all` 默认只给最后 200 行，数不全」；目录「列出目录里的每份文件、逐份数」。现查 `task-server/task:25`、`:58`（dfed1f94）`LOG_TAIL_LINES = 200` | 做到，200 这个数与服务一致 | 站得住 |
| B4 续行并接把以反斜杠收尾的注释并进下一条；围栏外续行不并；`-Fgpu` 与带引号多特性误红；只点名 `--test` 的 `cargo test` 不判（攻 B4、辩 A3） | `doc-experiments.sh:3060-3067` `comment_line` / `continued`（注释里的反斜杠不续行）；`:3181-3184` 围栏里与围栏外行首是命令的行都并，遇整行注释断开；`:3117-3156` `command_options`：`--features`/`-F` 分开写（:3146）、`--features=`（:3150）、`-F<值>`（:3152）、`--all-features`（:3154）、`--test` 目标（:3125）；`:3106` `command_words` 用 `shlex.split` 切引号；`:3220` `has_gpu` | 读代码做到。对判决写的数「攻方世界三行 77、0、1」：任务 4932 日志逐字如此（见第一节），但那一趟读的是第二轮的门禁（`gate_sha256=690a35ef`）。此刻门禁 7e0f7a6b 之后加的样本档（`fixtures/doc-experiments.sh/preregistration-judge-features-*` 十三份、`judge-entries-in-sync-*` 三份）全是未跟踪目录，没有一次跑过 | 站得住（读代码）；样本核不动 |
| B5 特性写法三处不一致：门禁认 `--features=`、`-F`、写几次、`--all-features`，`compile-then-swap.py` 前几种退 2、写两次只留后一个 | `compile-then-swap.py:918-960`（6328769d）`feature_names_union`、`parse_command_line`：`--features <值>`、`--features=<值>`、`-F <值>`、`-F<值>`、`--all-features`，写几次取并集、值里逗号与空白都分隔；与 `doc-experiments.sh:3146-3154` 逐写法相同（门禁多一条 `--package=`、`-p` 不相干）。任务 5095 探针：`features-equals-gpu`、`short-flag-gpu` 现为 rc=0 gpu_in_every_call=yes，`features-twice-gpu-first` 现为 `gpu,extra`，对照任务 4931 的改前读数 rc=2 与 `extra` | 做到，改前改后读数可对 | 站得住 |
| B6 自证新项直接调 `swap()`，丢掉命令行 `--features` 的变异体自证照样全过 | `compile-then-swap.py:785-814` 的新项经 `check_features_through_command_line`（:765）走 `run_command_line`；`:827-` `SELFTEST_CASES` 共 23 项（`sed -n 820,875p | grep -c '^    ("'` 得 23，与任务 5090「查了 23 项」相同）；开关 `command-line-features-dropped`（:120）。任务 5092 红 7 格，与文件头 `:120-122` 点名的 7 格逐个相同 | 做到 | 站得住 |
| B7 读不出产品 GPU 包名照跑的那一支没有自证与开关；不是成员方向相反（攻 B7、辩 A10） | `merge-and-verify.sh:688-696`；自证 `:1472-1526`；开关 `product-gpu-unreadable-as-unaffected`（文件头 `:228`）、`product-gpu-non-member-as-unaffected`（`:230`）。任务 5120、5121 各红 1 格，任务 5122 红 2 格 | 做到 | 站得住 |
| B8 `main-agent.md` 第 55 行验证段第 1 步补产品 GPU 包 | `.claude/main-agent.md:55`（589c7787）「补丁没碰 checker 档包、它依赖的包与产品 GPU 包 `singlefs-background-gpu` 时报本次未跑」 | 做到 | 站得住 |
| B9 `verification.md` 第 63 行末句仍说门禁那一格里面自己经 `gpu-quiesce-run.sh` 起；第 35 行补产品 GPU 那一格「跑到加复用 0 条也报本次未跑」 | `verification.md:63`（c10aa10d）末句现为「崩溃放量入口…与门禁 `checker-tier-crash-point-replay` 的 feature-gated-tests 格自己都不清场、不挑卡，经服务交（服务在最外层清场、分卡），不在外面套」；`:35` 有 product-gpu-real-card-tests 一格与「一个对拍目标跑到加复用 0 条的也报「本次未跑」」 | 第 63 行与第 35 行的产品 GPU 句做到。但第 35 行同一格里还留着「特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起」，与第 63 行和共用约束相反，见 F3 | 判决那两句站得住；同形状还有一处没改（F3） |
| B10 `commit-stages.md` 第 23 行靠驱动文件头 `*`，带 `--skip-gpu` 的那一片服务照样清场；第 19 行重跑列三格 | `commit-stages.md:23`（52b08d58）「驱动文件头声明 `# task-server-gpu-arguments: *`，服务照样给带 `--skip-gpu` 的那一片清场分卡」；对照 `scripts/mutation/broad-mutation.py:4`（文件头 `# task-server-gpu-arguments: *`）与 `task_server_lib.py:244`（`*` 是任何参数）；`:19` 三格见第二节 | 做到，与服务的认法一致 | 站得住 |
| B11 `mutation-triage.md` 第 15 行写分类以 `mutation-sampling.md` 为准，那份仍是六栏 | `mutation-triage.md:15`「`.claude/rules/mutation-sampling.md` 全篇（分类以它为准）」；`mutation-sampling.md:79` 现为九栏；与 `mutation-triage.md:32` 同一段 | 做到（九栏）；残留见 F1，两处逐字复制见 F4 | 站得住 |
| 辩 A2 判段入口表没有一格核同步 | `doc-experiments.sh:328`（`stage_cell judge-entries-in-sync`）与 `:3262-3460` 的格体（`:3449`、`:3451`、`:3455`、`:3460` 是红绿两种输出）；文件头 `:29`；`stage-owners.tsv:123`；弄坏开关 `judge-entries-new-unreported`（:215）、`judge-entries-stale-unreported`（:217）；样本 `judge-entries-in-sync-green/-red/-stale-red`（未跟踪）。我对着 `crash_state_enumeration.rs:187、246、266、287、307、335、368、400` 与 `crash_amplification.rs:1470、1482、17418、17442` 的 `pub fn` 手核，表里 13 个名字都在，源码里带记号的公开入口我没找到表外的 | 读代码做到；格没跑 | 站得住（读代码）；格与样本核不动 |
| 辩 A5 设计员范围声明括注补整包 `cargo test` | 见第二节「设计员…范围声明」一行 | 做到 | 站得住 |
| 判决不采：第一轮判决第 20 行文字与落地（三处必要加四处有条件）不同 | `implementation-writer.md:47` 照落地 | 第二节已核该落地比「五处」准 | 站得住 |
| 第一轮留的两样欠账登进账本（采，已登记） | `records/2026-10-29-subagent拆分提案.md:879-884` 一百八十节存在，① ② 两样都在，② 带去向 | 做到 | 站得住 |

## 五、判决里报的引文数（我能复现的）

| 判决里的话 | 我复现的 | 判 |
|---|---|---|
| 第一轮：cite-check「核了 49 处引文、99 处裸行号，对上 17 处，没判 131 处」，没判里 93 处是快照树之外的脚本、38 处是裸行号没有引文可比 | 对着第一轮快照树 `~/.local/share/singlefs/work/d137-ledger-defs-r1-main/d137-ledger-defs-r1-tree` 重跑：`python3 scripts/agents/cite-check.py research/prompts/d137-ledger-defs-r1-sonnet-output.md research/prompts/d137-ledger-defs-r1-opus-output.md --root <该树> --background research/prompts/_d137-ledger-defs-r1-body.md`，末行逐字同「核了 49 处引文、99 处裸行号，对上 17 处，没判 131 处（其中裸行号没有引文可比 38 处…）」，退 0。算术 49+99=148=17+131；131-38=93 | 站得住 |
| 第二轮：辩方「核了 27 处引文…对上 27 处」 | 辩方报告（67ee4e34，sha8 与判决写的相同）对着第二轮快照树重跑：「核了 27 处引文、30 处裸行号，对上 16 处，没判 41 处」，退 0、对不上 0 处。引文数 27 复现；「对上 27」复现不出：快照树只收被判的 16 份文件，倒推不出的 `main-agent.md` 与树外的脚本不在树里，被引到的行走「没判」。对着此刻的主工作区重跑（不带 `--root`）是「11 处对不上」，原因是那些文件第二轮修补时又改过、行挪了。判决写「对上 27」的那一趟读的是当时的工作区，现在复现不了 | 引文数站得住；「对上 27」核不动 |
| 第二轮：攻方「核了 39 处引文…对上 39 处」 | 攻方报告（5f6a32b5，sha8 与判决写的相同）对着第二轮快照树重跑：「核了 39 处引文、15 处裸行号，对上 27 处，没判 27 处」，退 0；引文数 39 复现，「对上 39」复现不出，原因同上一行。对着此刻工作区（不带 `--root`）是「23 处对不上」。两条腿合在一起重跑：「核了 66 处引文、45 处裸行号，对上 43 处，没判 68 处」（27+39=66，30+15=45 对得上）。输出在 `cite-r2-opus-root.txt`、`cite-r2-opus.txt` | 引文数站得住；「对上 39」核不动 |

核对时我在草稿目录留了三份输出：`~/.local/share/singlefs/work/d137-ledger-defs-r3-sonnet/cite-r1-root.txt`、`cite-r2-sonnet-root.txt`、`cite-r2-sonnet.txt`。

## 六、前两轮没看到的、同形状还在的地方（站不住的与去向不全的）

F1　分诊定义写着一条脚本不再打的「清场没过没跑：N 条」行（站不住，同一句话里的残留）。
- 现查：`.claude/agents/mutation-triage.md:32`（4c81979a）「基线不绿与清场没过没跑不在「计数：」那一行里，各自另起一行报（「基线：…基线不绿 N 行」「清场没过没跑：N 条」）」，`:41` 「有「清场没过没跑：」「本次未跑：」行的一并贴原样」；`.claude/rules/mutation-sampling.md:79`（3dad8a4b）同一段逐字相同。
- 脚本现状：`grep -rn '清场没过没跑' scripts .claude/gate.d .claude/hooks task-server` 0 处。`scripts/mutation/crates-mutation-rows.py:4744-4768`（4a31ff70）收尾只打「计数：」「基线：」「复用：」「覆盖：」「本次未跑：」五类行；这一行在 HEAD 版本里有 1 处（`git show HEAD:scripts/mutation/crates-mutation-rows.py | grep -c '清场没过没跑：{len'` 得 1），暂存区版本里 0 处（`git show :scripts/mutation/crates-mutation-rows.py | grep -c 清场没过没跑` 得 0）；它是 `git log -S` 找到的提交 f4605214d 里加的（`git show f4605214d -S清场没过没跑` 命中 `crates-mutation-rows.py` 的 `say(f"  清场没过没跑：…")`），那之后清场改由服务在最外层做，脚本里这一支没了。
- 这是第一轮「采后一半」改九栏时动过的那一句，九栏改对了，句尾留着旧行；同时也是账本 `一百八十 ①`（`records/2026-10-29-subagent拆分提案.md:882`）要等落地后补的那一件。落地的条件现在看已满足（上一节），所以该一并付：本次未跑认两种来历、删掉「清场没过没跑」。
- 什么现象会推翻：若某个脚本在我没搜的目录里仍打这一行（我只搜了 `scripts`、`.claude/gate.d`、`.claude/hooks`、`task-server`；`.claude/singlefs-ai-sop/` 没搜），就不是死引用。

F2　E171 两份入库装置的要卡声明：去向有落点，但落点是别的会话手上的补丁与一句散文，账本里没有一节（去向不全）。
- 现查：两份装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`（第 1860 行起用 `GivenStatesEngine`）与 `e171_three_oracle_collusion_after_the_oracle_fixes.rs`（第 7037 行用 `GivenStatesEngine`）此刻文件头没有 `// task-server-gpu-arguments:` 行（`grep -l` 现查，只有 e156、e158、e161、e168 四份有）。
- 落点：`~/.local/share/singlefs/work/compat-removal-e171-crates/patch/crates.patch:5`、`:15` 就是给这两份各加一行（`baseline row known-bad-images`、`baseline row confirmed-loss-form`），`records/2026-10-31-删兼容旧版本代码.md:209` 说它「补丁只交、等三方轮 m4-gpugc-code-r3 判决后真打」，而那一轮判决已写完（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:728`）。`records/2026-10-29-一百三十七节那一轮调度记录.md:314` 一句「E171 两份归做 E171 的会话」。拆分提案账本里没有哪一节登这件事。
- 失败是吵的、不是静默的：`gpu_card_configuration.rs:385-398` 在没给卡又没清场且配置开了 GPU 时退 78 并写出路，所以这两份在补丁打上之前裸交服务只会退 78，不会悄悄自己挑卡。
- 什么现象会推翻：补丁已经打到主工作区别处（我只看了此刻主工作区这两份文件头）。

F3　`verification.md` 第 35 行同一格里还留着第二轮 B9 判掉的那种说法（同形状没改全）。
- 现查：`.claude/rules/verification.md:35`（c10aa10d）「特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起，照旧一条 `cargo test`」。对照：同文件 `:63` 「清场由服务做…调用方经 `task-server/task submit -- <命令>` 交，不在外面套清场」；`.claude/agent-common.md:64` 把 `gpu-quiesce-run.sh` 列在交服务的命令不许带的包装前缀里；`.claude/gate.d/checker-tier-crash-point-replay.sh:187`「这一道不自己清场、不挑卡」、`:2549` 「清场归 task-server，这里不另套清场包装」；`.claude/main-agent.md:101` 同一件事已改成「带 `gpu` 的照旧一条 `cargo test`，哪几张卡由服务分」。全仓 `.claude/agents`、`rules`、`skills`、`agent-common.md`、`main-agent.md` 里 `gpu-quiesce-run` 只剩 `verification.md:35` 这一处是照旧说法（另两处 `agent-common.md:64`、`commit-stages.md:17` 是禁用清单）。
- 什么现象会推翻：这一句是在描述快档命令的形状而不是教人怎么交（读法上留得住）。但同文件第 63 行与共用约束都明说不套，读的人会被两头拉。

F4（零轮）　`mutation-triage.md:32` 与 `mutation-sampling.md:79` 有 481 个字符的连续相同段，门禁放行。
- 现查：`difflib.SequenceMatcher.find_longest_match` 在这两行上得 481（两行各 799、878 个字符）。门禁 `agent-definition-shape` 的「不抄规则的长行」按整行逐字比（`.claude/gate.d/code-tooling.sh:2282`、`:2449`），行首前缀不同，所以任务 4927、5106 都绿。共用约束要求定义不抄规则。推的：属于第二轮 B11 修补之后才有的东西，没经过前两轮；由主 agent 判要不要拆。

F5（零轮，推的，没量过）　`crash_amplification.rs:1402` 的 `pub fn judge_not_run_without_the_gpu_feature` 不在 `JUDGE_ENTRIES` 里。
- 依据：`judge-entries-in-sync` 的认法（`doc-experiments.sh:3254-3260` 格上方注释的 ②④）只把函数体里带 `cfg!(feature = "gpu")` 或调这个函数的单元当入口；这个函数本身体内两样都没有（`:1402-1410` 现查），所以它不在表里；`preregistration-judge-features` 又只数表里的名字（`ENTRY_WORD`，`:2993`）。装置若直接调它自己拼 NOT_RUN 行，会漏判。我没找到哪份装置这样调，所以只是形状（`grep -n judge_not_run_without_the_gpu_feature crates/singlefs-checker-tier/src/bin/*.rs` 现查只有 `e161_crash_state_dedup_and_time_split.rs:3020` 一处注释提到它，没有调用）。
- 什么现象会推翻：上面那条 grep 出现函数调用而不是注释，或别处（`scripts/experiment-apparatus/` 下）有装置直接调它。

## 七、任务服务读数与文件改动先后

每个任务的起跑都晚于它判的文件的最后一次改动（用 `ls -la` 的修改时刻与 `task-server/task status <号>` 的「用时」倒推，报告里不写时刻）：`compile-then-swap.py` 最后一次改动早于任务 5090–5095；`merge-and-verify.sh` 最后一次改动早于任务 5119；`doc-experiments.sh` 最后一次改动早于任务 5106、5126；`agent-common.md` 与九份定义最后一次改动都早于任务 5106（这一趟 `rules-named-by-definitions`、`agent-definition-shape` 两格绿）。所以第三轮背景写的「绿」不是对着改之前的文件跑的。

## 八、核不动的

1. `doc-experiments.sh` 此刻版本的样本档：`fixtures/doc-experiments.sh/preregistration-judge-features-*` 十三份与 `judge-entries-in-sync-*` 三份在 `git status` 里是 `??` 未跟踪，背景材料也写了「要先暂存才喂得了样本档，还没跑」。续行、注释、`-F<值>`、带引号多特性、`--test` 目标、别名导入这些读代码都对得上，但没有一次对此刻版本的门禁跑出读数；任务 4870、4932 跑的是第一、二轮的旧版本（4932 的 `gate_sha256=690a35ef`）。
2. `judge-entries-in-sync` 格在真树上的结局：没跑。我手核了 13 个名字都在源码里是公开入口，源码里没找到表外带记号的公开入口，但格的判法是它自己的切函数单元与调用边，我没复现。
3. 第二轮攻方「对上 39 处」：引文数 39 复现，对上数复现不出（快照树上 27），见第五节。
4. 第二轮辩方「对上 27 处」：见第五节，复现的是 16。
5. `e158_root_choice_repair.rs:1` 的「只有 r2-all r2-seventh-batch r4-seg2 三个模式起 `GivenStatesEngine`」：构造函数 `given_states_engine_of_this_experiment` 的调用点 6940、7007 在 `run_seventh_batch_reproductions`（:6922，由 :7410、:7416 的 r2-all、r2-seventh-batch 调），22068 在 `run_segment_two`（:22027，由 :17042 的 r4-seg2 调）对得上；21663、21865 在 `run_the_seventh_fault_family`、`run_a_crash_family` 里，我没追到它们是不是只从 `run_segment_two` 来。
6. `merge-and-verify.sh` 自证「查了 55 种」：没有逐条数此刻自证的种数，只对了基线退 0 与四个开关的红格数。
7. 判决里的行号会漂：第一轮 `agent-common.md:98`（此刻在 `:100`）、`compile-then-swap.py:300`（此刻 `build()` 在 `:330`）、第二轮 `gpu_card_configuration.rs:394`（那一支已改成退 78，在 `:385-398`）。判决原样保存，不算错；这里写的是以此刻为准。

## 九、要主 agent 跑的命令

都在仓根 `/home/fy5090/code/singlefs` 起（不带 `--cwd`）。前两条要先暂存 `.claude/gate.d/fixtures/doc-experiments.sh/` 下的 16 个新样本目录（主 agent 的 git 写操作，我不做）。

1. `task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh /home/fy5090/code/singlefs/.claude/gate.d --item doc-experiments`。看 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`，日志里没有 `✗` 行；有 `✗` 就是某个样本判错，点名的样本名对照 `doc-experiments.sh` 文件头。回答核不动第 1 条。
2. 对 `doc-experiments.sh` 里和这一批直接相关的弄坏开关各交一条：`task-server/task submit -- env EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=<开关> bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh /home/fy5090/code/singlefs/.claude/gate.d --item doc-experiments`，`<开关>` 依次取 `judge-entries-new-unreported`、`judge-entries-stale-unreported`、`continuation-unjoined`、`shell-comments-ignored`、`features-spelling-narrow`、`test-targets-unjudged`、`judge-by-module-marker`。每条该退 1，`✗` 点名的样本集合与 `.claude/gate.d/doc-experiments.sh` 文件头 `:183-217` 里该开关下列的样本逐个相同（`judge-entries-new-unreported` 对应 `judge-entries-in-sync-red`，`judge-entries-stale-unreported` 对应 `judge-entries-in-sync-stale-red`）。
3. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=核判段入口表 JUDGE_ENTRIES 与两份判段源文件此刻逐个相同' task-server/task submit -- bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync /home/fy5090/code/singlefs`。绿的句子是「判段入口表 JUDGE_ENTRIES 的 {len(entries)} 个名字与两份判段源文件里认出的公开判段入口逐个相同」（`doc-experiments.sh:3460`，运行时 `{len(entries)}` 换成名字个数，此刻表里是 13）；红会点名文件、行与限定名。回答核不动第 2 条。
4. 直接跑的文本工具（不经服务）：判决写「对上 27」「对上 39」的那一趟读的是当时的工作区，要复现只能回到当时的树；主 agent 若要落实，在改前的树上跑 `python3 scripts/agents/cite-check.py research/prompts/d137-ledger-defs-r2-sonnet-output.md research/prompts/d137-ledger-defs-r2-opus-output.md --background research/prompts/_d137-ledger-defs-r2-body.md`（此刻的树上是对不上的，这是预期，不是新问题）。可不做。
5. 直接跑的只读查：`grep -rn '清场没过没跑' /home/fy5090/code/singlefs/scripts /home/fy5090/code/singlefs/.claude/gate.d /home/fy5090/code/singlefs/.claude/hooks /home/fy5090/code/singlefs/task-server /home/fy5090/code/singlefs/.claude/singlefs-ai-sop`，要 0 行才支撑 F1。
6. 直接跑的只读查：`grep -rn 'gpu-quiesce-run' /home/fy5090/code/singlefs/.claude/rules /home/fy5090/code/singlefs/.claude/agents /home/fy5090/code/singlefs/.claude/skills /home/fy5090/code/singlefs/.claude/agent-common.md /home/fy5090/code/singlefs/.claude/main-agent.md`，应命中 `verification.md:35`、`agent-common.md:64`、`commit-stages.md:17` 三行，才支撑 F3。

## 十、没做什么

- 不替主 agent撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务；判决要引的数（门禁样本档、`judge-entries-in-sync` 格在真树上的结局）没复跑，命令在上一节；这些读数在主 agent 跑出之前都是「推的，待主 agent 跑」。
- 没读第三轮攻方报告（禁读清单）。前两轮的判决整份读了；前两轮的腿报告只用 `cite-check.py` 与 `sha256sum` 核了引文数和文件指纹，没逐条读内容。
- 没改任何被判文件；报告之外只在草稿目录 `~/.local/share/singlefs/work/d137-ledger-defs-r3-sonnet/` 留了任务日志拷贝与 cite-check 输出，没有仓副本、编译目录。
- 没动手修 F1 到 F5：F1、F3 是各一句的字句修，F2 要主 agent 判要不要在账本里为 E171 立一节，F4、F5 是零轮形态，按多轮规则写进交用户表标「零轮」。
