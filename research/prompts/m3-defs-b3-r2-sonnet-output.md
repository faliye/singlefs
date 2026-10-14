# m3-defs-b3-r2 辩方腿（Sonnet）报告（2026-10-13）

立场：复核第一轮判决 `research/prompts/m3-defs-b3-r1-main-verification.md`——改法 G1–G11 与用户定案 H1–H3 落到第二轮冻结副本里，对第一轮两腿每一条打中的历史是不是修到、有没有改过头；第一轮哪一格判重、判轻。负责格 R1、R3、R4、R6、R7、R8。引文路径写冻结副本的绝对路径；行号是这一次 `grep -nF` 现取的。

## 0 现场与喂法

- 冻结副本核对：第二轮 `sha256sum -c research/prompts/m3-defs-b3-r2-snapshot/frozen-copy-sha256.txt`（在冻结副本根里跑）30 行 OK、`grep -vc ': OK$'` 得 0；第一轮 `sha256sum -c research/prompts/m3-defs-b3-r1-snapshot/frozen-copy-sha256.txt` 同样 0 行非 OK。两份冻结副本我都没改。
- 两棵树：`tree-before` 拷自第一轮冻结副本（`~/.local/share/singlefs/work/m3-defs-b3-r1/repo`），`tree-after` 拷自第二轮冻结副本（`~/.local/share/singlefs/work/m3-defs-b3-r2/repo`），各拷 `.claude`、`scripts`、`records`、`crates`、`.cargo`、`CLAUDE.md`、`Cargo.*` 和 `abandoned-rounds.tsv`；不拷 `multi-host.env`、`research/` 大目录、`target`。同一份提示或同一条 JSON 在两棵树上各喂一次，打 `改前 rc→改后 rc`。
- 喂法：派发闸经 `--check-prompt <提示> --type <类型> --root <树>`（⑥ 的快照样本另建树里的 `research/prompts/`），重型闸与写闸经钩子入口喂 JSON（`CLAUDE_PROJECT_DIR` 指样本树），`capped.sh` 经 `--print`，判法函数只用在 `lib_workspace_cargo_lock.py` 与 `lib_writer_lock.py` 造现场、`scratch_hog_report` 三处。没跑任何 `--selftest`、门禁阶段、cargo。
- 模型目录 `research/prompts/m3-defs-b3-r2-sonnet-model/`：`feeds/`（八份重喂脚本加 `build_trees.py`、`make_gate_sample_roots.py`、`e1_private_hits.py`，另拷第一轮攻方的 `opus-r1-probes/` 三份）、`tools/rerun.sh`（模板改 `rerun_steps`）、`out/`（九份输出，工作目录换成 `<WORK>`，另有第一轮攻方探针 d1、d3 在两份冻结副本上的输出各一份）。
- 复跑：`bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-defs-b3-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-defs-b3-r2/repo --work <~/.local/share/singlefs/work/ 底下的新目录>`（改前冻结副本默认第一轮那份，换了设 `BEFORE_FROZEN`）。末尾九行 `name=compare file=… same` 即与 `out/` 逐字节相同；我连跑两次（第一次落盘、第二次比对）九行全 same。
- 判定词：修到 / 部分修到 / 没修到 / 改过头。表 1 共 41 行：修到 33、没修到 6、部分修到 2（括注写「文字」的是钩子没变、只改文字的，算修到）。**改过头 0 条**（第 2 节逐条列了改前改后）。表里 ⑥ 相关的行写「放/拦」：拦 = 开工快照挡了这次派发（退出码 2），放 = 没挡（退出码 0）。
- 一次性观测（不在复跑里）：主工作区那两份文件有没有还带私有值，我用 `multi-host.env` 的值与本机主机名只数命中行号，不打值。原样输出：`主工作区 PEER_SSH_HOST值 records/2026-09-30-里程碑三开工第一轮.md 命中行号 []`、`主工作区 本机hostname research/prompts/m3-batch10x-code-r1-opus-output.md 命中行号 []`；`git status --short` 列这两份是 `M`。冻结副本里这两份没带（见 `out/e1.txt`）。

## 1 第一轮每条打中的历史去向（正推 S01–S26，攻方 O01–O15）

| 编号 | 第一轮打中的历史 | 喂法 | 改前→改后 | 判 | 依据 |
|---|---|---|---|---|---|
| S01 | 正推 D1-1：E161 装置只读 E161_THREADS，`capped.sh N` 不设 | r2_capped：`capped.sh 8 printenv E161_THREADS` | 1→0（改后打出 8） | 修到 | capped.sh:43 |
| S02 | 正推 D1-1：E171 编排器按此刻空闲线程取小（本机现测 0） | 读码，没起 E171 | - | 没修到（预期内：sha256 钉住，留给 E171 下一次修订） | heavy-thread-exempt:58 |
| S03 | 正推 D1-1：变异门禁 16 个工作进程 | 读码 | - | 没修到（预期内：「与依赖产物共用一起改」行） | heavy-thread-exempt:64 |
| S04 | 正推 D1-1/D1-4、攻方 A4 前半、B1：门禁 layer0 带 gpu 特性那一趟 `RUST_TEST_THREADS=31`、无 `--test-threads 1` | opus-r1-probes/d1-threads.sh（layer0-gate-mentions-test-threads）；r3_r4_static | `--test-threads` 次数 0→5；`capped.sh default` 前缀仍交 31 | 修到（单机这一趟）；J05、J06、J07 另记 | checker-tier-layer0-replay.sh:732、checker-tier-layer0-replay.sh:766 |
| S05 | 正推 D1-1：子 agent 命令不经 `capped.sh`、或 `capped.sh 8` 与提示里 31 并存，闸都放行 | r2_capped：S4 三条 | 0→0、0→0、0→0 | 文字修到（一律 `capped.sh N`）；钩子照旧不要求、不比 N | agent-common.md:61 |
| S06 | 正推 D1-1：`SINGLEFS_THREAD_CAP` 已设时 `capped.sh default` 取它 | r2_capped：S5-default-cap-env-5 | RUST_TEST_THREADS 5→5（脚本没改） | 修到（文字：default 只给主 agent 与脚本） | agent-common.md:61 |
| S07 | 正推 D1-2、攻方 A2：用户临时让出 CPU，没有规则管 | 读文字（没有钩子） | - | 修到（H3 文字）；旧句并存见 J09 | main-agent.md:53 |
| S08 | 正推 D1-3、攻方 A3：⑮ 放过 ÷、/、「一半」、「第二台 16」 | r1_thread_limit：S7-*、A3-* | ÷ 0→2；/ 0→2；一半 0→2；A3 公式 0→2；A3 第二台 0→2 | 修到 | runner-dispatch-guard.sh:1001 |
| S09 | 正推 D1-3 末条、攻方 A3 第三例：不写这一行不判 | r1_thread_limit：S8-missing-line-* | implementation-writer 0→2；investigator 0→2；tooling-writer 0→2；experiment-runner 0→2；mutation-triage 0→2；three-way-attack 0→2；clerk 0→0 | 修到（六种）；表外类型照旧不判，见 J04 | runner-dispatch-guard.sh:1005 |
| S10 | 正推 D5-1：`-p <独立探针包名>` 不拒 | r5_heavy_guard：D5-1 五行 | cwd 探针 `test -p` 0→2；`run -p` 0→2；`--package` 0→2；`--package=` 0→2；`-p`+`--manifest-path` 0→2 | 修到 | lib_heavy_tests.py:1100 |
| S11 | 正推 D5-1：bench、直接执行探针二进制、虚拟工作区根 `cargo run` 文字没写 | r5_heavy_guard | bench 0→2；直接执行 0→0；虚拟根 run 0→0 | 修到（bench）；另两样钩子本来不判，文字改成「不判」 | lib_heavy_tests.py:953、agent-common.md:81 |
| S12 | 正推 D5-2：合入锁——脚本里 cargo 不判；`cargo metadata`/`tree`/`fmt` 被拒；`cp -r` 带 .git 的副本照拒 | r5_heavy_guard：lock 八行（`lib_workspace_cargo_lock.py` 两树同一份） | bash build.sh 0→0；metadata 2→2；tree 2→2；fmt 2→2；cp -r 副本 2→2 | 脚本里 cargo：文字修到；子命令集合二义与 `cp -r` 带 .git：没改，第一轮判决也没把它们列进改法（J10） | agent-common.md:81 |
| S13 | 正推 D5-3：嵌套目录互相拦 | r6_writer_lock：sub-* 四行 | A 写父 0→0；E 写自己的子目录 0→0；A 写 E 的子目录 2→2；E 写父 2→2 | 修到（文字；钩子按主 agent 更正不改） | agent-common.md:45、lib_writer_lock.py:20 |
| S14 | 正推 D5-3：`lib_writer_lock.py` 第 4 行旧编号 ⑮ | r6_writer_lock：stale-number | 第 4 行有 ⑮：True→False | 修到 | lib_writer_lock.py:4 |
| S15 | 正推 D5-4(1)：8 份只有 Read/Bash 的定义，「开工先认锁」无从触发 | r6_writer_lock：static-agents | 定义共 14 份，有 Write/Edit 6、只有 Bash 8；共用约束限定到有 Write/Edit 的 False→True | 修到（文字） | agent-common.md:45 |
| S16 | 正推 D5-4(2)(3)：先到先得比文字强；重启后旧进程回执不在记录里时先认领 | r6_writer_lock：3a/3b/4a/4b/5a | 3a 0→0；3b 2→2；4a 0→0；4b 2→2；5a 0→0 | 文字修到（如实写先到先得）；钩子行为没改，旧进程仍可先认领（J11） | agent-common.md:45、lib_writer_lock.py:17 |
| S17 | 正推 D5-4(4)：回执对不上的拒绝不点名持锁者 | r6_writer_lock：1a、2a、9 | 1a 点名持锁者 False→True；2a False→True；无回执对得上时写锁登记的提示 sha256 前 12 位 False→True | 修到 | lib_writer_lock.py:227 |
| S18 | 正推 D5-5：看门狗「被停也算」、超阈值目录报告 | s14_watchdog；读文字 | 报告 5 行→5 行（两个仓副本、一个编译目录、汇总、出路）；文字「被停的同样算」False→True | 修到（文字）；编译目录一例第一轮没喂到，这次喂到了 | main-agent.md:37 |
| S19 | 正推 D6-1、攻方 F1：halted 引自己/引这一轮正文/引规则句/引无关一句，把还要判的轮关掉 | r7_abandoned：同一段历史改后换成登记表一行 | 自引 放→拦；引正文 放→拦；引规则 放→拦；引无关一句 放→拦 | 修到 | runner-dispatch-guard.sh:756、runner-dispatch-guard.sh:809 |
| S20 | 攻方 F2：出处用 `research/../../` 指到仓外 | r7_abandoned：F2；另喂 `records/../`、records 下指到仓外的符号链接 | 仓外 放→拦；`records/..` 指仓内别处 放→拦；符号链接 放→拦 | 修到 | runner-dispatch-guard.sh:756 |
| S21 | 正推 D6-1 末段：「快照冲突已判：<轮> <理由>」理由非空即放行 | r7_abandoned：X-valve-snapshot-conflict-any-reason | 放→放 | 没修到（第一轮已另立一笔账，改前也中） | runner-dispatch-guard.sh:742 |
| S22 | 正推 D6-2：规则文字没写「」括、6 个字、同一行 | 读文字；r7：D6-2 两行 | 没「」括 拦→拦；不到 6 个字 拦→拦 | 修到（文字补全，钩子判法不变） | three-way-inference.md:121 |
| S23 | 正推 D6-3、攻方 F3：halted 与 tsv 两处登记、互不读 | r7_abandoned：D6-1-real 与 F3 | 真轮 m3-batch63-code-r1：改前靠 halted 文件放、改后靠登记表一行放 放→放；只留 halted 文件：改后拦并提示「已撤」（blocked_by_snapshot=True、halted_note_present=True） | 修到（只剩一处登记） | three-way-inference.md:121 |
| S24 | 正推 D7-1：多机一行「线程现算」与固定 31 二义 | r8_long_run | 旧写法（host-resources 现算）0→2；新写法（取 --emit-hosts 与登记值）2→0 | 修到（⑭ 走登记值读法；脚本文件头仍走现算读法，并存，文字已分开） | lib_long_run.py:63、main-agent.md:55 |
| S25 | 正推 D7-1：⑭ 只判设计员与执行员 | r8_long_run：S18-over2h-* | implementation-writer 0→2；investigator 0→2；tooling-writer 0→2；mutation-triage 0→2；three-way-attack 0→2；clerk 0→0 | 修到 | runner-dispatch-guard.sh:1176 |
| S26 | 正推 D7-1：写死额度只在多机行按数字判 | r8_long_run：中文数字、卡号、分片行带数字 | 中文数字（取登记值写法）2→0；「3 张卡」2→0；分片行带 `31 线程` 2→0 | 没修到（改前也中，G11 没含） | lib_long_run.py:121 |
| O01 | 攻方 A1：`SINGLEFS_THREAD_CAP=8` 时两种许可写法交 8 与 31 | opus-r1-probes/d1-threads.sh；r2_capped；r1_thread_limit A1-* | `capped.sh --print default` 与 `--print 31`（env=8）：8 与 31（改前改后同，脚本没改）；⑮ 提示写 8 2→2、写 31 0→0；骨架 0→0（仍填 31） | 部分修到：分叉靠文字消除（子 agent 只用 `capped.sh N`）；环境变量通道仍被 ⑮ 与骨架忽略，是把争议写成规则（J12） | agent-common.md:61、runner-dispatch-guard.sh:1018、dispatch-prompt.py:107 |
| O02 | 攻方 A2：用户临时让出 CPU，在跑的子 agent 没有路降线程 | 同 S07 | - | 修到（H3 文字，在跑的不动）；同 S07 | 同 S07 |
| O03 | 攻方 A3（同 S08、S09） | 同上 | 同上 | 修到 | 同 S08、S09 |
| O04 | 攻方 A4 后半：GPU 对拍里 CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`，不在 `capped.sh` 清单 | r3_r4_static | 清单里 0 处；测试里读 SINGLEFS_THREAD_CAP 0 处；没设取 available_parallelism | 没修到（落「代码批」行）；新写的注释说它照登记值，不实（J06） | crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:208、crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:265、heavy-thread-exempt:76 |
| O05 | 攻方 B1：轻型 `--quick` 与门禁带特性那一趟让 GPU 测试按 31 个线程跑 | r3_r4_static；opus d1 probe | 文件里 `--test-threads` 出现次数：crash-point-replay.sh 0→6、门禁 0→5；双机驱动、admission.py、放量脚本 0→0 | 部分修到：单机路径修到；双机分支、崩溃枚举 crash-case、崩溃放量没改，而规则写成无例外（J05） | crash-point-replay.sh:462、crash-point-replay.sh:477、verification.md:59 |
| O06 | 攻方 B2：「配置里的机器与卡都用」与单个 GPU 测试二进制只用本机点名的卡 | 读文字 | - | 修到（文字）；原句射程可读成驱动，判重一点（J13） | verification.md:59、verification.md:60 |
| O07 | 攻方 B3：「20 万态量级」没有单位，与「挂钟长不是收窄它的理由」拉扯 | 读文字 | - | 修到（文字：限定为日常验证、不管提交与全量）；harness 档残留（J08） | verification.md:58、verification.md:50 |
| O08 | 攻方 C1：照现状哈希登记，别人没审的代码改动拿到豁免 | opus-r1-probes/d3-exempt.sh（`user_exemption_reason` 原样抽出） | 三行输出改前改后逐字相同：register-current-hash exempted=yes；own-only exempted=no；staged exempted=no | 文字修到（H2 乙：不登记）；门禁判不出「别人没审」，输出没变 | main-agent.md:27、agent-common.md:64 |
| O09 | 攻方 C2：`compile-then-swap.py` 只核开跑之后的改动 | - | - | 没修到（第一轮已另立账，不在这一批） | - |
| O10 | 攻方 E1：已推到远端的两处私有值 | e1_private_hits.py（冻结副本）；主工作区一次性数命中 | 冻结副本：两棵都是 PEER_SSH_HOST 的值 1 份（行 1004）、本机主机名 1 份（行 295、302）；主工作区：两份文件命中行号 [] 与 [] | 工作区修到（H1 甲）；冻结副本集成时没带，不算没修 | records/2026-09-30-里程碑三开工第一轮.md、research/prompts/m3-batch10x-code-r1-opus-output.md（值不写出） |
| O11 | 攻方 E2：射程只写第二台；与「原样」规矩冲突 | 读文字 | - | 修到（文字）；上游 SOP 那一句仍写「一个字都不改」，两层各说各的（J14） | agent-common.md:47、evidence-discipline.md:28 |
| O12 | 攻方 E3：能做成只判新加行、按长词的会红检查 | 读 `doc-text.sh` 的 private-values 格；格子 agent 起不了 | - | 修到（形态一致）；要主 agent 跑 `--check private-values` | doc-text.sh 新格，见第 5 节命令 |
| O13 | 攻方 F1：自引/引正文/引规则句 | 同 S19 | 同 S19 | 修到 | 同 S19 |
| O14 | 攻方 F2：指到仓外 | 同 S20 | 同 S20 | 修到 | 同 S20 |
| O15 | 攻方 F3：只登记 tsv 则 ⑥ 照拒；只写 halted 则门禁红 | 同 S23 | 同 S23 | 修到（只剩一处登记） | 同 S23 |

编号 J 后面的数字补成两位（正文里写 J5 的就是 J05）。
推翻条件：任一行的「改前→改后」在复跑里变了，或依据列的 `文件:行` 在冻结副本里读不到那句话。

## 2 第一轮判对、改前钩子判对的历史，在改后重喂（找改过头）

| 组 | 喂法 | 改前→改后 | 判 |
|---|---|---|---|
| ⑮ 该拒的照拒 | r1_thread_limit：32、30、310、31.5、`N = 31`、`default`、整机核数 ÷ 重活数 | S7-32 2→2、S7-30 2→2、S7-310 2→2、S7-31.5 2→2、S7-N=31 2→2、S7-default 2→2、S7-cores-div 2→2 | 一致（没改过头） |
| ⑮ 该放的照放 | r1_thread_limit：`31`、`031`、半角冒号、列表记号、粗体、`31（本机）` | S7-31 0→0、S7-031 0→0、S7-halfwidth-colon 0→0、S7-list 0→0、S7-bold 0→0、X-ok-paren 0→0 | 一致（没改过头） |
| ⑮ 已判行 | r1_thread_limit：理由非空、占位、空 | S3-valve-reason 0→0、S3-valve-x 0→0、S3-valve-placeholder 2→2、S3-valve-empty 2→2 | 一致（没改过头） |
| ⑮ 环境变量 8 | r1_thread_limit：A1-env8-* | A1-env8-prompt31 0→0、A1-env8-prompt8 2→2、A1-env8-prompt8-valve 0→0 | 一致（没改过头） |
| `capped.sh` 取数 | r2_capped：default 无环境/环境 5、显式 31/8、`capped.sh 8` 里套 default | RUST_TEST_THREADS 31→31、5→5、31→31、8→8、8→8（只多出 E161_THREADS 一项） | 一致（没改过头） |
| 重型闸该拒的照拒 | r5_heavy_guard：`--manifest-path` 探针 test/run/nextest/git 依赖、cwd 在探针里裸跑、主 agent 不带前缀 | test --manifest-path linked 2→2、run --manifest-path linked 2→2、test --manifest-path gitdep 2→2、nextest --manifest-path linked 2→2、cwd linked: cargo test 2→2、main agent no prefix: cd linked && cargo run 2→2 | 一致（没改过头） |
| 重型闸该放的照放 | r5_heavy_guard：带 commit 前缀的主 agent、不依赖 checker 档的探针、只编不跑、harness 档包 | main agent with commit prefix: cargo run linked 0→0、test --manifest-path plain 0→0、D5-1 build --manifest-path linked (compile only) 0→0、X -p singlefs-harness from tree root (harness tier) 0→0、X cwd harness crate: cargo test --lib 0→0、X plain probe cwd: cargo test -p plain-probe 0→0 | 一致（没改过头） |
| 合入锁 | r5_heavy_guard：lock 八行（同一份库） | sub-agent cargo build in main workspace 2→2、main agent cargo build 2→2、sub-agent cargo --version 0→0 | 一致（没改过头） |
| 写者锁认领 | r6_writer_lock：1b、2b（持锁者自己写） | 1b 0→0、2b 0→0 | 一致（没改过头） |
| 派发闸 ⑯ | r6_writer_lock：d16 a–f | a 0→0、b 2→2、c 0→0、d 0→0、e 0→0、f 2→2 | 一致（没改过头） |
| ⑥ 该放的轮、该拦的轮 | r7_abandoned：真轮 m3-batch63-code-r1（登记表一行）、合格出处、没登记、不到 6 个字、没「」 | D6-1-real-user-quote 放→放、control-good-source 放→放、D6-1-open-no-registration 拦→拦、D6-2-short-quote 拦→拦、D6-2-no-bracket-quote 拦→拦 | 一致（没改过头） |
| ⑭ 写死额度照拒 | r8_long_run：`每台 31 线程、64G 内存` | 2→2 | 一致（没改过头） |
| ⑭ 设计员与执行员 | r8_long_run：超过 2 小时不写三行、不写预计挂钟 | S18-over2h-nothing-else-experiment-designer 2→2、S18-over2h-nothing-else-experiment-runner 2→2、S18-no-wall-clock-experiment-designer 2→2、S18-no-wall-clock-experiment-runner 2→2 | 一致（没改过头） |
| 看门狗 | s14_watchdog：仓副本与编译目录报告 | 报告 5 行→5 行，名字相同（`agent-watch.py` 两树同一份） | 一致（没改过头） |

读码类的「一致」（没有喂）：`crates/` 里自己定并行度默认值的写法登记在排除表「代码批」行，13 行改前改后逐字相同（`diff` 无输出）；排除表总行数 62→66，多出的 4 行是派发闸拒绝说明与自证样本里点名 `available_threads` 的文字（正文第二节已记）。

改后更严、但正是改法要的（不算改过头）：

| 写法 | 改前→改后 | 对应的改法 |
|---|---|---|
| `cargo bench` 在工作区根上 | r5_heavy_guard X tree root: cargo bench 0→2（`cargo bench -p singlefs-harness` 0→0、`--no-run` 0→0 不变） | G7 要的 bench 同判 |
| ⑭ 多机一行写 host-resources 现算 | 0→2 | G11 要的不现算 |
| ⑮ 括注里带数字 `31（每台 31 个）` | 0→2 | G1 要的括注里不许有数字 |
| ⑮ 尾部零宽字符 `31\u200b` | 0→2 | G1 只认整数加中文括注 |

推翻条件：找到一条第一轮判对的历史，改后退出码与改前不同而改法没要求变。

## 3 第一轮哪一格判重、判轻（依据 文件:行）

| 编号 | 格 | 判 | 内容 | 依据 | 打中改前吗 |
|---|---|---|---|---|---|
| J01 | R1 | 判轻 | 正文格 R1 把六份定义的「输入」列为 G1 的改动处，实际 investigator、tooling-writer 两份只写「线程上限、内存上限」，没写 `线程上限：N` 一行与「派发闸查」，而钩子对这两种缺行就拒（investigator 0→2、tooling-writer 0→2）；另四份（如实现员）写全了 | investigator.md:25、tooling-writer.md:25、implementation-writer.md:27、runner-dispatch-guard.sh:1005 | 不中：缺行拒是这一批新加的 |
| J02 | R1 | 判轻 | 写了 `线程上限：<随便>` 或 `<31>` 就算「写了这一行」，⑮ 不判值、⑬ 只抓 `<填 …>`：implementation-writer 0→0、0→0（`<填 登记值>` 2→2 被 ⑬ 拦） | runner-dispatch-guard.sh:1045、runner-dispatch-guard.sh:1046 | 中：改前这样写也放行 |
| J03 | R1 | 判轻 | 「线程上限已判：<任意非空理由>」同时放行公式与缺行：`31 ÷ 2` 加已判行 0→0；与「快照冲突已判」同形（S21） | runner-dispatch-guard.sh:742 | 中：已判行改前就在 |
| J04 | R1 | 判轻 | 表外类型带重活：clerk 定义允许编译、跑编译出来的代码，不写线程上限一行也放行（0→0），共用约束要的 N 无处可取；三方正推腿、辩方腿的定义带模型目录与 rerun.sh、跑模型加 nice，攻方在表内、这两种不在 | clerk.md:27、agent-common.md:54、three-way-defense.md:34 | 中 |
| J05 | R3 | 判轻 | 规则与注释写成无例外：crash-point-replay.sh 文件头「开了 GPU 的每一趟快档」带 `--test-threads 1`，规则写「单个 GPU 测试二进制……带 `--test-threads 1` 跑」；实际只有单机那一条 cargo 命令带（crash-point-replay.sh:482），双机分支（crash-point-replay.sh:477）、admission.py 拼的 crash-case 命令、放量脚本三处 `--test-threads` 出现 0、0、0 次；正文第二节已知这几处没改，规则文字没写例外 | crash-point-replay.sh:43、verification.md:59、crash-point-replay.sh:482、crash-point-replay.sh:477 | 不中：这一句是这一批写的 |
| J06 | R3 | 判轻 | CPU 一侧失实：门禁注释写「CPU 一侧判器的线程照 SINGLEFS_THREAD_CAP 与登记值」，crash-point-replay.sh 文件头写「照 capped.sh default 交的登记值」；GPU 对拍测试读自己的 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（capped.sh 清单里 0 处），没设就取 `available_parallelism`，不读 SINGLEFS_THREAD_CAP（0 处），本机 `nproc` 得 32，不是 31；该文件落在「代码批」行，门禁格不点名 | checker-tier-layer0-replay.sh:59、crash-point-replay.sh:43、crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:208、crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:265、heavy-thread-exempt:76、code-tooling.sh:1868 | 中（行为改前就这样）；注释是新写的，失实是新的 |
| J07 | R3 | 判轻 | 带特性那一趟是 `--lib` 加 16 份挂 gpu 的测试文件一条命令跑，`--test-threads 1` 让 `--lib` 的约 295 条与那 16 份文件的 312 条 `#[test]` 全串行（含不碰 GPU 的纯 CPU 用例）；挂钟没量，只在提交时那一趟，不进日常；G4 说「CPU 一侧线程照登记值」管的是用例内部线程，管不了用例之间 | checker-tier-layer0-replay.sh:766 | 不中 |
| J08 | R4 | 判轻 | v60 的前半句「GPU 对拍与内存对拍不压并发：开满默认线程」没动，与 v59 的 `--test-threads 1` 同时描述 GPU 对拍测试，只在末尾加了「驱动」的限定，读的人仍可读成矛盾；v58 把 harness 档也纳入 20 万，而 harness 档不在两档里、没有重型兄弟，要压只能收窄域或搬去 checker 档，规则没写选哪个（推的，没量 harness 档的最大态数） | verification.md:60、verification.md:59、verification.md:58、verification.md:42、main-agent.md:10 | 不中 |
| J09 | R1/H3 | 判轻 | H3 写成「在跑的子 agent 不发消息，照原样跑完」，同一份文件里还有两句允许给在跑的发消息（看门狗叫醒后「发消息让它改」；interim 通知「发消息让它接着做」），这两句改前就在，H3 没把它们收窄 | main-agent.md:53、main-agent.md:43、main-agent.md:49 | 中 |
| J10 | D5 | 判轻 | 第一轮判决 D5 没把合入锁的子命令集合二义（`cargo metadata`/`tree`/`fmt` 拒，文字「只查版本与清单」）和 `cp -r` 带 .git 的副本照拒列进改法；改后两样原样（S12） | agent-common.md:81 | 中 |
| J11 | R6 | 判轻 | G8 只改了文字：回执不在记录里时先到先得没改。`expected_holders` 已能在记录里找到「提示对得上这把锁」的 agent（1a、2a 的拒绝说明就用它），3a 里却没拿它去拒旧进程（3a 0→0、5a 0→0） | lib_writer_lock.py:17、lib_writer_lock.py:227 | 中：行为改前就这样 |
| J12 | R1 | 判轻 | A1 的修法是把争议写成规则：子 agent 照提示的 N，`SINGLEFS_THREAD_CAP` 只管 default 与脚本；若用户把环境变量当「给了参数」，⑮ 与骨架照样忽略它（A1-env8-prompt8 2→2、骨架 0→0），子 agent 仍跑 31；要靠主 agent 手写 N 与已判行。登记文件头仍写「SINGLEFS_THREAD_CAP 设了照它」 | heavy-thread-default:3、agent-common.md:61、runner-dispatch-guard.sh:1018、dispatch-prompt.py:107 | 不中 |
| J13 | D2 B2 | 判重（弱） | B2 的原句「配置里的机器与卡都用」在「GPU 对拍与内存对拍不压并发」那一条里，崩溃放量与对拍驱动本来就照配置用两台与全部卡；把它说成与「单个 GPU 测试二进制只用本机点名的卡」矛盾，要看「对拍」指驱动还是测试二进制。G6 的改写等于补射程，没有改规则 | verification.md:60 | - |
| J14 | E2 | 判轻 | 共用约束写了「这一条优先于」原样规矩，但上游 SOP 的证据规矩仍写「一个字都不改」；读哪层的腿会得到相反的指示 | agent-common.md:47、evidence-discipline.md:28 | 不中 |
| J15 | R1 | 判轻 | 括注判法只挡阿拉伯数字、÷、/、「一半」「除以」：`31（本机，第二台十六）` 放行 0→0、`31（二分之一）` 放行 0→0，与攻方 A3 的「31（本机），第二台 16」同形，G1 想挡的正是这种 | runner-dispatch-guard.sh:1001 | 中：改前也放行 |

判轻 14 条、判重 1 条（弱）。J01 是这一批新带进来的，按跑前条款第二行「钩子拒的定义没写」算站不住；J02–J04、J06、J10–J12、J15 改前也中，按第三行另立账，不拿来否这次改动；J05、J07、J08、J14 是这批新写的文字与注释本身的问题。
推翻条件：J01 被推翻要求 investigator.md、tooling-writer.md 的「输入」里出现 `线程上限：N`；J05 要求规则或文件头写出双机分支与崩溃枚举的例外；J06 要求测试读 `SINGLEFS_THREAD_CAP` 或 `capped.sh` 清单含 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`。

## 4 六格的辩方判定

| 格 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|
| R1 ⑮ 线程上限 | 站不住（部分） | 第一轮的 ÷、/、「一半」、缺行都修到（S08、S09、O03）；全角数字 0→2、上标 2→2、零宽在数字中间 2→2、括注里写数字 0→2、括注里写斜杠或一半、两行各写一个数 2→2 都拒。打中的：J01（定义与钩子不一致，新带的）、J02、J03、J04、J15、J12（改前也中，另立账）。H3「在跑的不动」与旧句冲突见 J09。表外类型：sweep、kb-scribe、prior-art、three-way-forward、three-way-materials 的定义里 `cargo／编译／capped／跑模型／跑编译` 命中 0 处（grep 现取），clerk 与辩方腿的情形见 J04 | 推翻 R1 的「站不住」：investigator.md、tooling-writer.md 的「输入」写出 `线程上限：N`，且 ⑮ 对 `<随便>`、`31（本机，第二台十六）` 拒；推翻「修到」那一半：复跑里 S08、S09 任一行不再是 0→2 |
| R3 GPU 那一趟 `--test-threads 1` | 站不住（部分） | 门禁一趟与轻型单机路径修到（S04、O05）；J05（规则与文件头写成无例外，双机分支与崩溃枚举 crash-case 没带）、J06（CPU 一侧失实）、J07（`--lib` 的约 295 条也串行，挂钟没量）。与「默认 31」不冲突：命令行 `--test-threads` 盖过 `RUST_TEST_THREADS`（推的，本机没有 libtest 源码，没编译验） | 规则与注释各补一句「双机分支与崩溃枚举 crash-case 例外」、CPU 一侧注释改成实情，J05、J06 就站住；带特性那一趟实测挂钟若与改前同量级，J07 不成立 |
| R4 verification 两句 | 站得住（原冲突）；同样打中别的臂（J08） | B2、B3 的原冲突修到（O06、O07）：verification.md:58 加了「不管提交与全量」，verification.md:50 说轻型是重型子集，所以压轻型的域不等于收窄枚举域，与 main-agent.md:10 不再打架；与「凡用 GPU 先清场」「两档」没有新冲突。残留 J08（verification.md:60 与 verification.md:59 并存；harness 档无重型兄弟） | 推翻 J08 后半：量出 harness 档没有超过 20 万态的崩溃枚举；推翻前半：verification.md 第 60 行前半加上「CPU 一侧」的限定 |
| R6 写者锁 | 站得住 | 撤回后钩子行为与第一轮逐条相同（嵌套四行 0→0、0→0、2→2、2→2）；回执对不上的拒绝点名持锁者（1a、2a、9）；「开工先认锁」收窄到有 Write/Edit 的定义（8 份只有 Bash 的不再被要求）；「子目录另派之后父目录持锁者不写它」只有文字：子目录有自己的锁之后钩子才拦（A 写 E 的子目录 2→2），另派之前 A 写子目录放行（0→0）。Bash 里的写、旧进程的先到先得是 J11，改前就有，文字已如实写 | 找到一段历史：回执在记录里的旧进程被放行，或 Bash-only 定义因为文字收窄而被锁挡了写；这次没找到 |
| R7 叫停轮一处登记 | 站得住 | 第一轮 F1–F3、D6 的历史全部修到（S19–S23、O13–O15）；真轮 m3-batch63-code-r1 仍收；同一行两个轮名收两轮 放→放、放→放，名字在而原话是别的事也收 放→放，文字已写「由主 agent 核」。⑥ 与门禁不对称：日期写坏、同一轮两行 ⑥ 收而门禁红；出处弱 ⑥ 拒而门禁绿（读 doc-process-records.sh:804、doc-process-records.sh:810、doc-process-records.sh:806，样本根与命令在第 5 节） | 找到 ⑥ 收而主 agent 判不是用户叫停的轮、且没经 ⑥ 以外的人看过——这一条本来就留给主 agent |
| R8 长活三行与多机线程 | 站得住（类型与登记值读法）；同样打中改前（额度判法）；够不着（最后跑出的数） | ⑭ 扩到七种类型、多机行改读登记值（S24、S25）；这些类型被逼写的只有一行预计挂钟（写「2 小时内」放行：S8-line-31-* 在带预计挂钟的基础提示下 0→0），超过 2 小时才多写三行。额度判法没动（S26）。「照提示做的执行员最后跑出的是哪个数」超出 ⑭：⑭ 只判文字；被起脚本文件头仍按现算读，双机驱动的 8 格现算线程没改（见 heavy-thread-exempt:29、heavy-thread-exempt:64），所以固定 31 到不了那几个驱动 | 找到一份提示通过 ⑭ 而其中的长活被起后线程数明显不是登记值，且不是正文第二节已列的几处 |

跑前条款对应：R1 的 J01 满足「钩子拒的定义没写」→ 改法进下一批；R3 的 J05/J06 是新写文字与实际不符 → 改法进下一批；R4、R6、R7、R8 两条腿里我这条没找到「可达历史违反用户定案」，没有「改过头」。

## 5 要主 agent 跑的命令（门禁格子 agent 起不了，这里不跑）

1. 复跑本报告的所有输出：见第 0 节命令，预期九行 `same`。
2. 装置读自己的线程变量那一格（S01、O04 的门禁一半）：在冻结副本根里 `bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults`；预期绿，`crash_judge_gpu_agrees…rs` 与 `crash_amplification.rs` 落在排除表「代码批」行（code-tooling.sh:1868）。
3. 私有值格（O12）：`bash .claude/gate.d/doc-text.sh --check private-values`；有本机 `multi-host.env` 时在冻结副本里跑，输出只报文件、行号、键名；没有配置退 77。
4. 叫停轮的门禁格与派发闸 ⑥ 对照（R7）：`python3 -I research/prompts/m3-defs-b3-r2-sonnet-model/feeds/make_gate_sample_roots.py <新空目录>` 造四个样本根，再在冻结副本里 `for r in <新空目录>/gate-*; do bash .claude/gate.d/doc-process-records.sh --check abandoned-rounds "$r"; echo "$r rc=$?"; done`；读格的代码得出：gate-weak-source 退 0（⑥ 拒），gate-bad-date 与 gate-duplicated 退 1（⑥ 收），gate-halted-only 退 1。
5. 带 gpu 特性那一趟的挂钟（J07）与「本次未跑」是否因 `--test-threads 1` 变少：要跑带 gpu 特性的快档，属重型，我没跑。

## 6 引文（每条的行号都是 grep -nF 现取的，原文逐字）

- `cap_e161`：「弄坏开关 e161-threads-unset：照改前的清单，不设 E161_THREADS（E161 装置只读这一个变量，capped.sh N 到不了它）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/hosts/capped.sh:43`）
- `ex_e171`：「scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py:min(found.available_threads, HOST_THREADS_CEILING)  # 按 sha256 钉住的冻结层，一个字节不改；每台线程与此刻空闲取小，上限 31」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/heavy-thread-exempt:58`）
- `ex_mut`：「scripts/mutation/mutate.sh:MAXIMUM_WORKERS_BY_PROCESSOR=16  # 与依赖产物共用一起改：与 worker_count_for 同一个上限」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/heavy-thread-exempt:64`）
- `lay_set`：「if [[ ",$feature_list," == *,gpu,* && "$layer0_break" != gpu-test-threads-unset ]]; then gpu_test_arguments=(--test-threads 1); fi」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/checker-tier-layer0-replay.sh:732`）
- `lay_cmd`：「${quiesce_prefix[@]+"${quiesce_prefix[@]}"} ${layer0_thread_prefix[@]+"${layer0_thread_prefix[@]}"} cargo test --release -p singlefs-checker-tier --features "$feature_list" --lib "${target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} 2>&1 \」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/checker-tier-layer0-replay.sh:766`）
- `ac_capped`：「子 agent 起重活（编译、测试、变异、产物、原型扫描）的每条命令一律用 `bash scripts/hosts/capped.sh N <命令>` 起」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:61`）
- `ac_default`：「`capped.sh default` 只给主 agent 与脚本内部用」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:61`）
- `ac_header`：「子 agent 照派发提示的 N」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:61`）
- `mc_yield`：「用户或别的会话要让出 CPU 时，只改新派的子 agent」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:53`）
- `mc_norun`：「在跑的子 agent 不发消息，照原样跑完」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:53`）
- `g_form`：「THREAD_LIMIT_VALUE = re.compile(r"[0-9]+(?:[ \t]*（[^（）\n]*）)?")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1001`）
- `g_req`：「THREAD_LIMIT_REQUIRED_AGENTS = {"implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack"}」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1005`）
- `lht_local`：「candidates[local_name] = os.path.normpath(os.path.dirname(manifest_path))」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_heavy_tests.py:1100`）
- `lht_bench`：「if subcommand == "bench" and not break_is_set("bench-unjudged"):」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_heavy_tests.py:953`）
- `ac_direct`：「直接执行它编出来的二进制、脚本里起的 cargo，这一样不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:81`）
- `ac_child`：「子目录另派给别的子 agent 之后，父目录的持锁者不写那个子目录」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:45`）
- `wl_nest`：「嵌套的草稿目录（目标往上走经过几处有 .writer-lock 的目录，find_lock_directory）：只认最近一层的锁，认领文件也只建在最近一层。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_writer_lock.py:20`）
- `wl4`：「排他建锁，持锁的还在跑就拒（⑯）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_writer_lock.py:4`）
- `ac_lock`：「tools 里有 Write / Edit 的，开工先认锁」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:45`）
- `ac_first`：「回执还找不到的，先到先得」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:45`）
- `wl_first`：「找不到（回执还没落进记录、记录读不了）：先到先得，排他建认领文件，建成了它就是持锁者，没建成读认领文件看是谁。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_writer_lock.py:17`）
- `wl_hold`：「def expected_holders(lock, requester, transcript_path):」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_writer_lock.py:227`）
- `mc_stopped1`：「交回之后后台还在跑（被停的同样算）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:37`）
- `mc_stopped2`：「有子 agent 交回时（被停的同样算）列草稿根里超阈值」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:37`）
- `g_src`：「ABANDONED_SOURCE = re.compile(r"(?<![A-Za-z0-9_\-./])(records/" + ABANDONED_SOURCE_CHARACTERS + r"\.md)" + ABANDONED_SOURCE_LINE)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:756`）
- `g_name`：「if not name_edges.search(cited_text) and not break_switch_is("snapshot-abandoned-round-name-unchecked"):」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:809`）
- `g_valve`：「SNAPSHOT_CONFLICT_RULED_LINE = re.compile(」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:742`）
- `twi_row`：「派发闸 ⑥ 读这张表（门禁 doc-process-records 的 abandoned-rounds 格也读它）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/three-way-inference.md:121`）
- `twi_check`：「那句话是不是用户为这一轮说的，由主 agent 核」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/three-way-inference.md:121`）
- `lr_live`：「LIVE_COMPUTE = re.compile(r"host[-_]resources|available_threads")   # 登记值读法下多机一行不许出现的现算」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_long_run.py:63`）
- `mc_tail`：「每台线程取登记的每台默认重活线程数，不现算。⑭ 判的类型：`implementation-writer`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:55`）
- `g_long`：「LONG_RUN_AGENTS = {"experiment-designer"} | THREAD_LIMIT_REQUIRED_AGENTS」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1176`）
- `lr_quota`：「quotas = [match.group(0).strip() for match in HARD_CODED_QUOTA.finditer(hosts)]」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_long_run.py:121`）
- `g_env`：「environment = {name: value for name, value in os.environ.items() if name != THREAD_OVERRIDE_VARIABLE}」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1018`）
- `dp_skel`：「临时覆盖（SINGLEFS_THREAD_CAP）不算：骨架填的是登记值本身。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/agents/dispatch-prompt.py:107`）
- `gpu_cpuvar`：「const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:208`）
- `gpu_fallback`：「.unwrap_or_else(|| std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:265`）
- `ex_gpu`：「crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:available_parallelism  # 代码批：GPU 对拍里 CPU 判器一侧的线程」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/heavy-thread-exempt:76`）
- `cpr_set`：「replay_break_is gpu-test-threads-unset || test_threads_arguments=(--test-threads 1)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/crash/crash-point-replay.sh:462`）
- `cpr_two`：「run_fast_tier_target_on_two_hosts "$target" "$owners"」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/crash/crash-point-replay.sh:477`）
- `v59`：「单个 GPU 测试二进制只用本机点名的卡，带 `--test-threads 1` 跑」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:59`）
- `v60b`：「崩溃放量与对拍驱动照多机配置用两台与全部卡」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:60`）
- `v58`：「日常验证（harness 档与轻型三条）的崩溃状态数压在 20 万量级」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:58`）
- `v50`：「轻型是重型的子集：轻型里有的每一项，重型里都有。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:50`）
- `mc_item9`：「遇到文件里有别的会话没审过的改动时不登记，请那个会话自己登记，或等它提交后再登记」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:27`）
- `ac_item9`：「遇到文件里有别的会话没审过的改动时不登记，请那个会话自己登记，或等它提交后再登记」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:64`）
- `ac_priv`：「门禁 doc-text 的 `private-values` 格判新加的行」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:47`）
- `evd`：「发给模型的提示、跑出来的原始输出、当时的日志，在这一轮里一个字都不改。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/singlefs-ai-sop/rules/evidence-discipline.md:28`）
- `inv25`：「线程上限、内存上限；报告路径与草稿目录，都在 `~/.local/share/singlefs/work/` 下。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/investigator.md:25`）
- `tw25`：「线程上限、内存上限；报告路径与草稿目录，都在 `~/.local/share/singlefs/work/` 下。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/tooling-writer.md:25`）
- `iw27`：「N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），后面只许接一段中文括注」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/implementation-writer.md:27`）
- `g_stated`：「stated = [value for value in written if not value.startswith("<")]」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1045`）
- `g_missing`：「missing = required and not written」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1046`）
- `clerk27`：「要编译、跑编译出来的代码的，照共用约束」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/clerk.md:27`）
- `twh`：「跑脚本、跑模型一律加 `nice -n 19`。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:54`）
- `def34`：「模型目录里的 `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/three-way-defense.md:34`）
- `cpr_single`：「cargo test --release -p singlefs-checker-tier --features "$features" --test "$target" -- --nocapture ${test_threads_arguments[@]+"${test_threads_arguments[@]}"}」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/crash/crash-point-replay.sh:482`）
- `cpr_hdr`：「开了 GPU 的每一趟快档测试参数带 `-- --nocapture --test-threads 1`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/crash/crash-point-replay.sh:43`）
- `lay_cmt`：「（GPU 对拍的测试各占卡的显存，不并行起；CPU 一侧判器的线程照 SINGLEFS_THREAD_CAP 与登记值，不受它管）；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/checker-tier-layer0-replay.sh:59`）
- `ct_gate`：「crates/ 里的（crash_amplification.rs、两份 GPU 对拍测试）与 scripts/mutation/ 的（与依赖产物共用）这几份落在原有的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/code-tooling.sh:1868`）
- `v60a`：「GPU 对拍与内存对拍不压并发：开满默认线程」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:60`）
- `v42`：「harness 档不在这两档里，照」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:42`）
- `ma_nonarrow`：「这一条管枚举域，不管跑的时机」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:10`）
- `mc_fixmsg`：「是问题就决定发消息让它改」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:43`）
- `mc_interim`：「还没交回、在干活或在等自己后台任务的，只在改了定义、共用约束或规则时收一条消息」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:49`）
- `htd_3`：「SINGLEFS_THREAD_CAP 设了照它，脚本的参数给了照参数」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/heavy-thread-default:3`）
- `old_v60`：「GPU 对拍与内存对拍不压并发：开满默认线程（登记的每台默认重活线程数），配置里的机器与卡都用」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:60`）
- `g_date`：「malformed_registry_rows.append(f"第 {line_number} 行日期不是 YYYY-MM-DD：{fields[1]}")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-process-records.sh:804`）
- `g_dup`：「duplicated_registrations = [f"{name}（第 {', '.join(map(str, numbers))} 行）" for name, numbers in sorted(registered_line_numbers.items()) if len(numbers) > 1]」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-process-records.sh:810`）
- `g_nosrc`：「registered_line_numbers.setdefault(fields[0].strip(), []).append(line_number)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-process-records.sh:806`）
- `ex_two`：「scripts/crash/crash-amplification-two-hosts.sh:看 crash_amplification_threads_now 里 max(1, available_threads)  # 自证失败行的出路文字，不起线程」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/heavy-thread-exempt:29`）

## 7 模型目录各文件 sha256（`cd research/prompts/m3-defs-b3-r2-sonnet-model && find . -type f | sort | xargs sha256sum` 原样）

```
5534bfdeb6daf6bf6dbfd1801399270d4d07271d0b6f244f424eaf62c7a1094c  ./feeds/build_trees.py
911c775f8868319d2f32bbb240493e79558f91aaf3973ec567296628c83606ca  ./feeds/common.py
335134dfa15817b5876fbc6fa0e90c926e99f8337183bc98623fabca8e6611fd  ./feeds/e1_private_hits.py
9dff34464160612662a8738bab81b0e6f4c6903ec8b4ec555880a259ed2347f4  ./feeds/make_gate_sample_roots.py
9cb000782666edaf7621b58c5a222eaf52f1136fd52b1411047aa649c007b176  ./feeds/opus-r1-probes/d1-threads.sh
5d331b95b31576f046969e79f5a281440896de9c294198f079029dbbc471aa54  ./feeds/opus-r1-probes/d3-exempt.sh
7a55d7a70e479cdeaae07fdfa2cddf001eb543e572094637c59535fe5e0d029a  ./feeds/opus-r1-probes/d4-private-values.py
b48d6696bd88454bb74e776e92ce6d76449a7cd99988ad2c8586ab2cd7c1bcda  ./feeds/r1_thread_limit.py
19a4eb279ede5ffda6bdfe80b3fc7c13c9d9d9886f6b97aa07e553022565987f  ./feeds/r2_capped.py
88ac08531b645a043964c72d2a6f02ad4a6eff0167b925c1be6e15df53740e3e  ./feeds/r3_r4_static.py
6f885f6af372a964d8faf21e7e81ee131d19fe963aeeb6502d48e1a042599709  ./feeds/r5_heavy_guard.py
7b3b5c960a004ccbaf4a7897c332ebbe1dd7c37f24ec9111029d12cbba4177e2  ./feeds/r6_writer_lock.py
c5faf4876b9f2a525d51b5d5ba3b89066ad58afd8d5c00fa866e545cefcc2aba  ./feeds/r7_abandoned.py
9be874c6643092a1c263f449d45f393de35d3b6fb85efabe81b00666ad2507c3  ./feeds/r8_long_run.py
c8b4c7f297d91eabc49a172070d2bfa1ecf4193137499f2f2d8117315857e88c  ./feeds/s14_watchdog.py
175f8e7c0195d8a692aa62ab31098dc4deb566c8ad92da8c490588e050032f74  ./out/e1.txt
7d9fd566a133bd784a32fa3740b77b4c8b5b4c65bd4b0011987b067d4309558c  ./out/opus-r1-d1-on-after.txt
ec74ed232c4963de582ae0946a9e82390bbb187ef114bef35449df89e9b6bbc4  ./out/opus-r1-d1-on-before.txt
e83c7d173192f4f5ef058f3173709745db1dcd410b145f0eec31ddf9c040a18d  ./out/opus-r1-d3-on-after.txt
e83c7d173192f4f5ef058f3173709745db1dcd410b145f0eec31ddf9c040a18d  ./out/opus-r1-d3-on-before.txt
beaed7ae5f7569250db3f2f6521d6405131d092db01af91699b3415c4fc26e07  ./out/r1_thread_limit.txt
1e5b1857904281edefa4fc4619c8c6c5b93901e9747efbec093e9ce6a51fec17  ./out/r2_capped.txt
b6a4e887146aa8878cced9507def5ec98a9f741ce5ce449487a1a39c1bc24358  ./out/r3_r4_static.txt
88e1acac140bd3848886d908972085c919d7bec6b6e4941d04e2c8c30e612af1  ./out/r5_heavy_guard.txt
106fb1146fec33180325330a21ccb638f861f0f57bb0ac945d4c8d7a9001fbda  ./out/r6_writer_lock.txt
f4abb6a52eede6e885a6f0f2857ded429f91c3287ae0cd98ce5faf5c93d15598  ./out/r7_abandoned.txt
5080316f841d12e3c247194236d2d7b0d0d3e6477ae94b741da8119215721efc  ./out/r8_long_run.txt
5c1b28197ad4a6179996c43f65c63588dc957d8c9e91fc0a129d72a7e66c7384  ./out/s14_watchdog.txt
06e85c81bd56b546c13353e836fa6c6ced553f3c93684276de7c258d07e6ea4a  ./tools/rerun.sh
```

## 8 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没跑任何登记的 `--selftest`、门禁阶段、cargo、重型测试；没起 E171、E161 装置；没跑 GPU。
- `--test-threads` 盖过 `RUST_TEST_THREADS` 是推的（本机没有 libtest 源码，没编译一个测试二进制去验）。
- 门禁格 `heavy-thread-defaults`、`private-values`、`abandoned-rounds` 只读了代码和自证说明，没跑格；J07 的挂钟没量；harness 档最大态数没量。
- 第一轮两腿报告里「没打中」「没判」「没喂」的小节不是打中的历史，不在复核范围。看门狗「已失效」判定与重启后旧进程的真实回执：我没造 `~/.claude/sessions` 登记、没重启客户端，只读码，与第一轮同。
- 没读禁读清单里的文件：调度记录、实现员自报、这一轮攻方腿的报告与模型。读了主工作区的只有第 0 节那一次数命中行号。

