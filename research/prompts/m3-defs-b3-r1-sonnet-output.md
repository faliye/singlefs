# m3-defs-b3-r1 正推腿（Sonnet）报告：D1、D5、D6、D7

立场：逐条核「定义与规则写的就是钩子与脚本做的」。判 一致 / 不一致 / 文字没写到。引文位置写成冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/` 里文件的绝对路径加行号（绝对路径是为了交回闸按仓根核也找得到同一份文件）；行号是我在冻结副本里 `grep -nF` 现取的。

## 0. 现场与喂法

- 冻结副本核对：在冻结副本根目录跑 `sha256sum -c research/prompts/m3-defs-b3-r1-snapshot/frozen-copy-sha256.txt`，21 行 OK、0 行非 OK。
- 我的实验树 `~/.local/share/singlefs/work/m3-defs-b3-r1-sonnet/tree/` 是冻结副本的 `.claude/`、`scripts/`、`records/`、`crates/`、`CLAUDE.md` 的拷贝；`diff -rq` 对 `.claude/hooks` 与 `scripts` 无差异（输出为空，退出 0）。D6 的实验另在树里加了假轮（`research/prompts/r-*`），冻结副本本身没动。
- 喂法：
  - 重型测试闸与合入锁：`feed-heavy.sh <agent_type 或 -> <cwd> <命令>`，把 JSON `{"tool_name":"Bash","cwd":…,"tool_input":{"command":…},"agent_type":…}` 喂给 `heavy-test-guard.sh`（`CLAUDE_PROJECT_DIR` 指实验树，`AGENT_HOOK_DETECTIONS` 指草稿目录）。
  - 派发闸：`runner-dispatch-guard.sh --check-prompt <提示文件> --type <类型> --root <树>`；要看建锁的才喂 stdin JSON（`dispatch16.py`）。
  - 写闸第九道：`wlcases.py` 把 JSON `{"tool_name":"Write","tool_input":{…},"agent_id":…,"transcript_path":…}` 喂给 `write-guard.sh`。
  - 合入锁的现场：`lockcases.py`、`lockcopy.py` 调 `lib_workspace_cargo_lock.py` 的 `build_held_workspace`（只用它造现场，不跑 `--selftest`）。
  - 脚本都在草稿目录 `~/.local/share/singlefs/work/m3-defs-b3-r1-sonnet/`，没入库（不是装置、没有数进结论）。
- 没跑任何登记的 `--selftest`，没起 cargo，没跑重型测试。

## D1　线程数

### D1-1　「默认 31、固定不看忙闲」之外，可达工作流里实际线程数不是 31 而规则让人以为是

规则原文：「机器忙时也固定这个数，不看忙闲」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53`）；「N 照派发提示，派发闸 ⑮ 要它等于登记值」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:61`）。逐条：

| 路径 | 实际线程 | 判 | 依据 |
|---|---|---|---|
| `crates/` 里没设变量时 | `available_parallelism`（本机 32 核，不是 31） | 一致（排除表承接） | 「crates/singlefs-checker-tier/src/crash_amplification.rs:available_parallelism」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/heavy-thread-exempt:65`）；「crates/singlefs-harness/tests/random_histories.rs:.min(16)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/heavy-thread-exempt:77`）。「13 处」的点数见表后 |
| E161 装置 | `capped.sh N` 设了也不管：它只读 `E161_THREADS` | 一致，但没有任何东西查 | 「没设取 `available_parallelism`（5.6）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2632`），取值在第 2641 行，变量名在第 120 行；`capped.sh` 的变量表（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/capped.sh:29-38`）没有它。文字靠「实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:61`）兜；复现见下 |
| 变异门禁 | 32 核机器上 16 个工作进程 | 一致（排除表承接）；规则文字没提 | 「by_processor = max(1, min(16, processor_count // 2))」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/mutation/crates-mutation-rows.py:1855`）；排除表「scripts/mutation/mutate.sh:MAXIMUM_WORKERS_BY_PROCESSOR=16」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/heavy-thread-exempt:60`）；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/skills/crash-test/commit-stages.md:12` 只写广谱变异驱动 |
| E171 | 按此刻空闲线程取小，上限 31 | 不一致（与「不看忙闲」），已登记 | 「threads = max(0, min(found.available_threads, HOST_THREADS_CEILING))」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py:279`）；`available_threads` 是核数减在用核（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/host-resources.py:356`）。本机现测见下：忙时是 0 |
| 门禁 layer0 的 gpu 那一趟 | `RUST_TEST_THREADS=31`，没有 `--test-threads 1` | 不一致（与 `verification.md` 第 59 行） | 见 D1-4 |
| `capped.sh <N>` 显式给数 | N | 一致；没有东西比 N 与提示里的数 | 下面喂法：`capped.sh 8` 与提示里 31 并存，闸放行 |
| 子 agent 命令没经 `capped.sh` | cargo 与 libtest 各自的缺省（核数） | 文字没写到闸 | 全仓钩子里 `capped` 只作启动器被认，没有一条要求它；喂法见下 |
| `SINGLEFS_THREAD_CAP` 已设时的 `capped.sh default` | 取该环境变量，不是登记值 | 文字没写到（主 agent / 共用约束两处只说「取登记值」） | 「local cap="${SINGLEFS_THREAD_CAP:-}"」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/heavy-threads.sh:30`） |

复现与原样输出（实验树里跑）：

```
$ bash scripts/hosts/capped.sh 8 printenv E161_THREADS ; echo rc=$?
rc=1
$ env -u SINGLEFS_THREAD_CAP bash scripts/hosts/capped.sh --print default | head -3
CARGO_BUILD_JOBS=31
RUST_TEST_THREADS=31
SINGLEFS_LAYER0_THREADS=31
$ SINGLEFS_THREAD_CAP=5 bash scripts/hosts/capped.sh --print default | head -3
CARGO_BUILD_JOBS=5
RUST_TEST_THREADS=5
SINGLEFS_LAYER0_THREADS=5
$ ./feed-heavy.sh implementation-writer $T "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib" 2
rc=0
$ ./feed-heavy.sh implementation-writer $T "bash scripts/hosts/capped.sh 8 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib" 2
rc=0
```

本机现测 `python3 scripts/hosts/host-resources.py --json` 的本机一项（原样截取）：`"cpu": {"cores": 32, "busy_cores": 32.0, "loadavg": [179.4, 181.83, 148.0], "available_threads": 0}`。E171 在这样的机器上拿到的线程数是 `max(0, min(0, 31))` = 0（读代码推的，没起 E171）。

`crates/` 里的「13 处」：`grep -rn 'available_parallelism' crates` 命中很多文档行；取线程默认的代码行我数出来的与主 agent 的「13 处」没有逐一对过，不替它背书，只确认这些文件在排除表里各有一行（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/heavy-thread-exempt:65-77`）。

推翻条件：E161 那条被推翻需要 `capped.sh` 也设 `E161_THREADS`，或装置读 `SINGLEFS_THREAD_CAP`（现在都没有）；E171 那条被推翻需要该文件的 `available_threads` 不是 `host-resources.py` 给的忙闲数。

### D1-2　删掉「N 变了发消息」之后，用户临时让出 CPU，规则里还有没有一条管

判：文字没写到。

- 主 agent 的两处只留了「改了定义、共用约束或规则」这一种消息：「除此之外不给在跑的子 agent 发消息」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:39`）；「只在改了定义、共用约束或规则时收一条消息」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:49`）。
- 改前有一条管「重活数变了」：补丁里被删的行是 `~/.local/share/singlefs/work/m3-tooling-b3-defs/patch.diff` 第 66 行与第 80 行（`grep -nF '重活数变了' patch.diff` 命中 66、80；转述，不在冻结副本里）。它管的是重活数变化，不是用户叫人让出 CPU，所以改前也没有一条直接管「用户临时让出」；这一格的缺口改前就在，不拿它否这次改动（跑前条款第三行）。
- 在跑的命令已经把线程变量设死（`capped.sh` 起命令时 export，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/capped.sh:164`），没有机制能从外面把一条在跑命令的线程降下来。能做的只有三样，规则文字都没写：① 让在跑的子 agent 的后续命令改线程（要发消息，与第 39 行冲突）；② 在派发提示里写「线程上限已判」另给数（只对新派的有效）；③ 在主 agent 自己的环境设 `SINGLEFS_THREAD_CAP`（`capped.sh default` 与广谱变异驱动认它，见 D1-1 的原样输出；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/implementation-workflow.md:71` 提到它，主 agent 与共用约束两份没提）。
- 推翻条件：能在冻结副本里找到一句管「用户临时要求让出 CPU」的话（对 `.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/`、`.claude/skills/` 搜 `让出`、`降下`、`忙闲`：我搜到的只有第 53 行「不看忙闲」那一句，是反方向）。

### D1-3　派发闸 ⑮ 与「可以输入参数调整」

判：一致（有放行行）；另有一处钩子不拒文字说拒的写法（不一致）。

- 文字：「用户给了参数、或内存装不下要用别的数的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53`）另起一行「线程上限已判：<理由>」；钩子：理由非空、不以 `<` 起头就放行（`valve_reason`，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:355`）。喂法（`--check-prompt`，只数输出里「线程上限」出现的行数）：`线程上限：16` + `线程上限已判：用户给了参数` 为 0；`线程上限已判：<理由>` 与空理由仍拒（3 行）；`线程上限已判：x` 放行。
- 不一致：钩子文件头写「不等于、写成公式、认不出数的拒」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:52`），文字写「N 不等于登记的每台默认重活线程数」才拒（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:85`）。实际只取开头的整数：「THREAD_LIMIT_LEADING_NUMBER = re.compile」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:888`），比较行「if number is None or int(number.group(1)) != registered」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:920`）。所以开头是 31 的公式放行。
- 喂法与原样结果（`bash $T/.claude/hooks/runner-dispatch-guard.sh --check-prompt exp/thr/p<i>.md --type clerk --root $T`，每份提示只换「线程上限：…」那一行，输出里「线程上限」行数）：

| 写法 | 线程上限相关行数 | 判 |
|---|---|---|
| `线程上限：31` | 0 | 放行 |
| `线程上限：31 ÷ 同时在跑的重活数` | 0 | 放行（应拒：值 ≠ 31） |
| `线程上限：31/2` | 0 | 放行（应拒） |
| `线程上限：31 的一半，即 15` | 0 | 放行（应拒） |
| `线程上限：031` | 0 | 放行 |
| `线程上限：32` / `30` / `310` / `31.5` / `N = 31` / `default` | 3 | 拒 |
| `线程上限：整机核数 ÷ 同时在跑的重活数` | 3 | 拒 |
| `线程上限: 31`（半角冒号）、`- 线程上限：31`、`**线程上限**：31` | 0 | 放行 |

- 没写这一行的提示不判：「没写这一行的不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:53`）；而 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53` 的主句是「任务里带重活……的，派发提示写一行」。写不写靠主 agent 自己，钩子只在写了的时候判数；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:61` 的写法「派发提示里有……的」是有条件的，与钩子一致。
- 推翻条件：把 `线程上限：31 ÷ 同时在跑的重活数` 喂 ⑮ 得到拒绝（需要比较前先去掉非整数尾巴或拒带运算符的值）。

### D1-4　GPU 那一趟

判：不一致（与 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:59`）。已由实现员自报，这里复核到行。

- 规则：「GPU 测试二进制带 `--test-threads 1` 跑」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:59`）。
- 门禁：「cargo test --release」那一趟的前缀是 `quiesce_prefix` 加 `layer0_thread_prefix`（`bash capped.sh default`，第 212 行取），命令后面只有 `-- --nocapture`：「${quiesce_prefix[@]+"${quiesce_prefix[@]}"} ${layer0_thread_prefix[@]+"${layer0_thread_prefix[@]}"} cargo test --release」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:760`）。`capped.sh` 把 `RUST_TEST_THREADS` 设成 31（变量在 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/capped.sh:31`，export 在第 164 行）。所以带 gpu 特性那一趟的 libtest 并发是 31，不是 1。
- 一并的疑问：同一句规则「CPU 一侧的并行写在用例里」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:59`）需要测试二进制逐个并发数为 1；门禁里这一趟是「--lib」加若干目标（第 760 行），一次起多个目标，各目标内部的并发才是 1 的意思。是否改成每个 GPU 目标带 `--test-threads 1` 是改法，不在这一腿。
- 推翻条件：第 760 行的命令里出现 `--test-threads 1`，或 `capped.sh` 不设 `RUST_TEST_THREADS`。

### D1-5　D1 小结

- 规则让人以为是 31 而实际不是的可达路径：E161（只读自己的变量）、变异门禁 16 个工作进程、E171（忙闲数）、GPU 一趟的 libtest 并发；`crates/` 的 `available_parallelism` 默认在排除表里登记。
- 「用户临时让出 CPU」没有规则管（D1-2）。
- ⑮ 与参数的关系一致，但「写成公式的拒」钩子做不到（D1-3）。

## D5　执行前拒绝的写法与看门狗两样

### D5-1　重型测试闸：依赖 checker 档包的别的 crate

文字：「依赖 checker 档包 `singlefs-checker-tier` 的别的 crate（工作区外的探针 crate 也算）跑测试或 `cargo run`，同样按重型拒」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:81`）。

判：大部分一致；`-p <探针自己的包名>` 一种写法钩子不拒（不一致）；几种文字没写到的边。

喂法（实验树里造了三个探针：`probes/linked`、`probes/gitdep` 在工作区外，依赖 checker 档包分别用 path 与 git 写法；`probes/plain` 不依赖；`probes/ws` 是自带 `[workspace]` 的）。以 `implementation-writer` 经 `bash scripts/hosts/run-with-memory-cap.sh 4G` 起，除注明的外：

| 命令 | 退出码 | 判 |
|---|---|---|
| `cargo test --manifest-path probes/linked/Cargo.toml` | 2 | 一致 |
| `cargo run --manifest-path probes/linked/Cargo.toml` | 2 | 一致 |
| `cargo test --manifest-path probes/gitdep/Cargo.toml`（git 依赖，只认名字） | 2 | 一致 |
| `cargo nextest run --manifest-path probes/linked/Cargo.toml` | 2 | 一致 |
| 当前目录 `probes/linked`，`cargo test` | 2 | 一致 |
| 主 agent 不带前缀，`cd probes/linked && cargo run --release` | 2 | 一致 |
| 主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 的 `cargo run`（探针） | 0 | 一致（前缀放行，文字「主 agent 不带前缀也拒」） |
| `cargo test --manifest-path probes/plain/Cargo.toml` | 0 | 一致 |
| 当前目录 `probes/linked`，`cargo test -p attack-probe` | **0** | **不一致** |
| `cargo test -p attack-probe --manifest-path probes/linked/Cargo.toml`（当前目录 `$W`） | **0** | **不一致** |
| 当前目录 `probes/linked`，`cargo run -p attack-probe` / `cargo test --package attack-probe` / `cargo run --package=attack-probe` | **0** | **不一致** |
| `cargo bench --manifest-path probes/linked/Cargo.toml` | 0 | 文字没写 bench |
| `cargo build --manifest-path probes/linked/Cargo.toml` | 0 | 一致（只编不跑不算） |
| 直接执行 `probes/linked/target/release/attack-probe` | 0 | 文字没写到（钩子自己写罩不住） |
| 当前目录 `probes/ws`（虚拟工作区根），`cargo run` | 0 | 文字没写到（钩子自己写不判） |

不一致的缘由：`-p` 的值只在工作区成员表里找，独立包（自己没有 `[workspace]`、也不是别人的成员）找不到就让范围为空，随后「for package_directory in scope:」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_heavy_tests.py:1090`）一次也不进。成员表来自 `scope = [members[name] for name in sorted(named)]`（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_heavy_tests.py:1071`），`named` 来自 `named |= (package_specification_members`（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_heavy_tests.py:1069`）。而文件头明写「（`--manifest-path` 指到它的 Cargo.toml、-p 点名它、当前目录在它里面裸跑）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_heavy_tests.py:61`）。钩子自己写的罩不住只有两条：直接执行探针编出来的二进制、虚拟工作区根不带 -p 的 run（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_heavy_tests.py:63`）；`-p <独立探针>` 不在其中。

复现：

```
mkdir -p $W/probes/linked/src; printf '[package]\nname="attack-probe"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nsinglefs-checker-tier={path="$T/crates/singlefs-checker-tier"}\n' > $W/probes/linked/Cargo.toml   # $T 换成实验树绝对路径
./feed-heavy.sh implementation-writer $W/probes/linked "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p attack-probe" 2     # rc=0
./feed-heavy.sh implementation-writer $W/probes/linked "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test" 2                    # rc=2
```

推翻条件：第一条得到退出码 2。这个缺口改前同样在（批 2 的钩子），文字是批 3 才写上「工作区外的探针 crate 也算」；按跑前条款第三行，它是这句文字站不住的证据，改法落钩子那一边（让 `-p` 命中不到成员时回落到 `--manifest-path` 或当前目录那一个包），不是否这次文字改动。

### D5-2　重型测试闸：主工作区合入锁

文字：「主工作区里起 cargo（只查版本与清单的子命令除外）而主工作区合入锁被别的还活着的进程占着」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:81`）。

判：核心一致；子命令的集合与「命令位置」射程文字没写全。

用 `build_held_workspace` 造主工作区（git 仓）与一个活着的持锁假进程，`SINGLEFS_WORKSPACE_LOCK_FILE` 指锁文件，各命令在主工作区目录喂（`lockcases.py`，原样输出，每行 `标签: rc=`）：

```
子 agent 主工作区 cargo build: rc=2
主 agent 主工作区 cargo build: rc=2
子 agent cargo --version: rc=0
子 agent bash build.sh（脚本里起 cargo）: rc=0
子 agent make 里起 cargo: rc=0
子 agent cargo fmt: rc=2
子 agent cargo clippy: rc=2
子 agent cargo metadata: rc=2
子 agent cargo tree: rc=2
子 agent cargo clean: rc=2
子 agent env FOO=1 cargo build / rustup run stable cargo build / ~/.cargo/bin/cargo build: rc=2
子 agent rustc --version: rc=0
```

- 放行的子命令集合是写死的：「NON_BUILDING_SUBCOMMANDS = {"version", "help", "locate-project", "pkgid", "read-manifest", "verify-project"}」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_workspace_cargo_lock.py:53`）。文字「只查版本与清单的子命令除外」没说是哪几个：`cargo metadata`（读清单）、`cargo tree`、`cargo fmt --check` 这类只读命令在锁被占着时被拒。判：文字没写到（二义）。
- 脚本里的 cargo 不判：「不读脚本正文：`bash x.sh` 里的 cargo 不在这一道的射程里」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_workspace_cargo_lock.py:19`）；上面 `bash build.sh`、`make build` 都是 0。文字「主工作区里起 cargo」没限「命令位置」。判：文字没写到。
- 出路句「在自己的副本里编」：不带 `.git` 的副本里 `cargo build` 退 0；`cp -r` 把 `.git` 一起带过去的副本在默认锁位置（`<common-dir>/singlefs-main-workspace.lock`）下退 2（锁文件跟着拷了，里面的持有者还活着；`lockcopy.py` 输出：主工作区 rc=2、带 .git 的副本 rc=2、不带 .git 的副本 rc=0）。共用约束对副本的拷法写的是「副本只拷要编的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:46`），照那样拷出路走得通；文字没单写「别拷 `.git`」。判：照做走得通，边上有坑。
- 推翻条件：`cargo metadata` 在锁被占着时退 0，或脚本里的 cargo 被拒。

### D5-3　写闸第九道与派发闸 ⑯

文字：「项目子 agent 往别的子 agent 持写者锁的草稿目录里写」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:82`）；「写者锁的持有者还在跑，又没写」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:85`）「写者锁已判：<理由>」（⑯）。

判：与钩子一致；三处文字没写到。

- 只判带 `agent_id` 的子 agent，目标在草稿根里：「主 agent（输入里没有 agent_id）、没有锁的目录、草稿根之外的目标不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/write-guard.sh:50`），与文字一致。
- ⑯ 点名的目录只认行首「草稿目录：<路径>」，路径在工作根或旧根底下、工作根本身不算：`WRITER_LOCK_DIRECTORY_LINE`（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:936`）。同目录再派、持锁的还在跑就拒；已判行接手（`dispatch16.py` 原样：a 第一次派发 rc=0 锁在；b 同目录再派（不同提示）rc=2，说明写「持锁的是 一个刚派出去、派发回执还没落进主会话记录的子 agent」；c 加「写者锁已判：…」rc=0，锁换成新的）。
- 嵌套目录（文字没写到）：⑯ 只看被点名的那一个目录；写闸从目标往上找第一把锁。`wlsub.py` 原样：

```
agentA 写 wlsub-dir/a.md: rc=0 九道拒=False
agentE 写 wlsub-dir/sub/e.md: rc=0 九道拒=False
agentA 写 wlsub-dir/sub/a-in-sub.md: rc=2 九道拒=True
agentE 写 wlsub-dir/e-in-parent.md: rc=2 九道拒=True
```

  A 持 `wlsub-dir` 的锁、E 持它的子目录 `wlsub-dir/sub` 的锁（⑯ 对 `dispatch16.py` 的 e 例放行了子目录再派），A 写自己目录下的子目录被拒。
- 钩子文件里残留一处旧编号：「持锁的还在跑就拒（⑮）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_writer_lock.py:4`），文字与派发闸文件头都写 ⑯。无害，记一笔。
- 推翻条件：`wlsub.py` 第三行得 `rc=0`。

### D5-4　共用约束「开工先认锁」照做会怎样

文字（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:45`）：「第一个被认作持锁者的是派发你的那一份提示」；「你的第一次 Write / Edit 被写范围闸拒」且拒绝说明点名了别的持锁者的，停下、交回主 agent、不换 Bash 写进去。

判：对有 Write / Edit 的子 agent 大体一致；四处与钩子对不上或文字没写到。

喂法：`wlcases.py` 造草稿目录、写者锁（`lib_writer_lock.py` 的 `new_lock`、`create_lock`）与一份主会话记录（`async_launched` 回执行），逐个 `agent_id` 把 Write 的 JSON 喂给 `write-guard.sh`。原样输出（截取，`九道拒` 是我数的「写者锁相关的拒绝」）：

```
1a B(别的提示)先写: rc=2 九道拒=True 认领文件=False
1b A 之后写: rc=0 九道拒=False 认领文件=True
2a 旧进程(回执在记录里、早于锁): rc=2 九道拒=True 认领文件=False
2b 新派发: rc=0 九道拒=False 认领文件=True
3a 旧进程(记录里找不到它的回执)先写: rc=0 九道拒=False 认领文件=True
3b 新派发后写: rc=2 九道拒=True 认领文件=True
4a 记录读不了，旧进程先写: rc=0 九道拒=False 认领文件=True
4b 记录读不了，新派发后写: rc=2 九道拒=True 认领文件=True
5a B 无回执写子目录: rc=0 九道拒=False 认领文件=True
```

1. **Bash 里的写、没有 Write / Edit 的子 agent（文字没写到）**。14 份定义里 6 份的 tools 有 Write 或 Edit，8 份只有 Read / Bash（`for f in .claude/agents/*.md; do grep -m1 '^tools:' $f; done | grep -vc 'Write\|Edit'` 得 8，总数 `ls .claude/agents/*.md | wc -l` 得 14）：三方三条腿、`sweep`、`mutation-triage`、`experiment-designer`、`prior-art`、`three-way-materials`。它们的写全走 Bash，写闸看不见（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/write-guard.sh:54` 「拦不住 Bash 里的写」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_writer_lock.py:23` 「Bash 里的写这里看不见」）。所以「你的第一次 Write / Edit 被拒」这一触发条件对它们永远不发生，它们也没有读锁文件的指示可循；锁一直没人认领。后果见 5a：别的有 Write / Edit 的子 agent 如果它的派发回执不在记录里，会在这把锁上先到先得地认领。
2. **「第一个被认作持锁者的是派发你的那一份提示」比钩子强（不一致）**：钩子只在回执找得到时比提示 sha256 与时刻；找不到（回执还没落进记录、记录读不了）是先到先得：「找不到（回执还没落进记录、记录读不了）：先到先得」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_writer_lock.py:17`），代码「owner = try_claim(scratch, lock, agent_id)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_writer_lock.py:211`）。5a 是别的提示的 B 在无回执时认领成功。
3. **重启后的旧进程**：回执在记录里时拒（2a，比较行「if digest != lock.get("prompt_sha256") or not late_enough」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_writer_lock.py:209`）；回执不在这份记录里时旧进程先写就先得（3a），新派发的正主随后被拒，拒绝说明点名旧进程（3b），按这一句文字正主要停下交回。读不了记录同样（4a、4b）。我没法复核重启之后旧进程的回执是否还在主会话记录里（复核不了：需要真重启一次客户端），所以这一条只到「机制上成立」。
4. **拒绝说明不一定点名持锁者（文字条件与钩子对不上）**：回执对不上的（1a、2a）说明里写的是「派发提示对得上这把锁的那一个子 agent（你的派发回执不是这把锁登记的那一份提示，或早于这把锁）」，没有 agent 号：「owner = (f"子 agent {holder}" if holder」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/write-guard.sh:223`）。文字的触发条件是「拒绝说明点名了别的持锁者的」，按字面这种拒绝不满足，可能被读成普通写范围拒绝；说明里的 → 怎么办 同样写了「停下对这个目录的写」，所以实际会停，是文字窄于钩子。
5. **子 agent 自己再派的子 agent**：不存在，「你手里没有 Agent 工具，不派 subagent」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:10`）。

推翻条件：5a 得 `rc=2`（回执不在记录里的子 agent 不能认领别人提示建的锁）；或 8 份只有 Bash 的定义里任何一份在派发时拿到读锁文件的指示。

### D5-5　看门狗新报的两样

文字：「有子 agent 交回时列草稿根里超阈值」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:37`）……没动的编译目录与仓副本，只报不删；重启前旧进程留下的子 agent 记录标「状态=已失效」，不按无动静报。

判：一致，两处文字比代码窄。

- 草稿根里超阈值：「被看的子 agent 里有交回（或被停）的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/agent-watch.py:116`），文字只写「有子 agent 交回时」，漏了被停；配置 `scratch-idle-hours=6` 在 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/watch.conf:31`；「只进报告、不叫醒、不删」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/agent-watch.py:119`）与文字一致。我喂了 `scratch_hog_report`（`hogs.py`）：10 小时前最后修改的带 `.git` 的目录与带 `Cargo.toml` 加 `crates/` 的目录各报一条（「仓副本 …：最后修改 10h00m 前」），只有 `Cargo.toml` 与 `src/` 的、只有 `notes.md` 的不报，1 小时前的不报。编译目录那一例我自己造数据时把 `CACHEDIR.TAG` 的修改时刻写成了现在，没判成过期，所以没有喂到编译目录的过期判，这一半只读代码未喂。
- 已失效：「不按「无动静」「工具调用过长」这类报」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/agent-watch.py:125`）与「已失效的在报告里标「状态=已失效」」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/agent-watch.py:124`）与文字一致。没喂（需要造 `~/.claude/sessions` 登记，我不碰 `~/.claude/`）；判定的两种情形我读了代码，不替它背书。
- 推翻条件：交回的子 agent 存在而报告里没有「草稿根里超阈值的编译目录与仓副本」一节。

### D5-6　D5 小结

| 句 | 判 |
|---|---|
| 重型闸：依赖 checker 档包的别的 crate 跑测试或 `cargo run` | 不一致（`-p <独立探针包名>` 不拒） |
| 重型闸：主工作区合入锁 | 一致；子命令集合、脚本里的 cargo 文字没写到 |
| 写闸第九道、派发闸 ⑯ | 一致；嵌套目录文字没写到 |
| 派发闸 ⑮ | 一致；「写成公式」钩子不拒（D1-3） |
| 「开工先认锁」 | 对 8 份只有 Bash 的定义无从触发；比钩子强（先到先得）；拒绝说明不一定点名 |
| 看门狗两样 | 一致 |

## D6　叫停轮写 halted

文字：「文件空、没出处、原话在那份文件里找不到的不收，那一轮照旧算开着」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/three-way-inference.md:121`）；「派发闸 ⑥ 认它」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/three-way-inference.md:121`）。

### D6-1　一份 halted 文件能不能把一个其实还要判的轮关掉

判：能。钩子做的就是文字说的，文字本身只要求「原话在那份文件里找得到」，没有东西能验那句话是用户叫停的话。

钩子的判法：先把标题行去掉，剩下的行里找「仓里 records / research / .claude 起头的 .md 路径」，路径在仓里存在，同一行里有引号「」括起的、去掉空白后至少 6 个字的一句，且这一句在那份路径的文件里（去掉空白）能找到，就收：「body = [line for line in text.splitlines() if line.strip() and not line.lstrip().startswith("#")]」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:704`）、「for cited in HALTED_SOURCE_PATH.findall(line):」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:711`）、「HALTED_QUOTE_MINIMUM_CHARACTERS = 6」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:694`）、「elif any(quote in cited_text for quote in quotes):」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:720`）。轮要不要算开着只看：快照目录在、正文 `_<轮>-body.md` 在、没有 `<轮>-main-verification.md`（`has_verdict = os.path.exists`，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:739`）、halted 过得了上面这一关（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:743`）；不看腿的产出在不在、有没有交回。

喂法（在实验树的 `research/prompts/` 里造七个假轮，每轮一个 `_<轮>-body.md`、一个 `<轮>-snapshot/kb-sha256.txt`（列 `.claude/kb/<轮>.md`），再派 `kb-scribe`、规格点名 `.claude/kb/<轮>.md`；`--check-prompt --type kb-scribe --root $T`，数输出里「开工快照」出现的行数，0 是放行、3 是被快照拦住）：

| 假轮 | halted 文件的那一行 | 快照拦 | 判 |
|---|---|---|---|
| r-real | `出处：records/2026-10-01-删除与并发事务.md 第 1595 行「增加流程限制  测试代码不需要打， 改了直接测。」`（仓里真有这句） | 0 | 收，符合本意 |
| r-self | `出处：research/prompts/r-self-halted.md「这句话是我随手写的六个字以上」`（出处就是它自己） | 0 | 收，**一个其实还要判的轮被一句自己写的话关掉** |
| r-unrel | `出处：.claude/main-agent.md「本机常有别的会话在同一个仓里干活」`（与叫停无关的一句） | 0 | 收，同上 |
| r-nobracket | 路径加原话，原话不用「」括 | 3 | 不收 |
| r-short | 原话「增加流程」（4 个字） | 3 | 不收 |
| r-open | 没有 halted 文件 | 3 | 拦着（对照） |
| r-ctrl | 没有 halted 文件 | 3 | 拦着（对照） |

- 改前同样的缺口：派发闸本来就有放行行「快照冲突已判：<轮名> <理由>」，理由只要非空：`SNAPSHOT_CONFLICT_RULED_LINE = re.compile`（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:682`）。所以「靠一句话关掉快照」改前就有，只是它每次派发写一行；halted 是一次写下、对之后所有派发都生效。按跑前条款第三行，这个打中也打中改前，另立一笔账，不拿它否 halted 这一笔。
- 推翻条件：把 r-self 那一行喂进去时快照仍拦着（需要钩子拒绝出处等于 halted 文件自己、或要求出处文件不是 `research/prompts/` 下这一轮自己的产物）。

### D6-2　派发闸 ⑥ 的判法与规则文字对不对得上

判：文字窄于钩子（文字没写到四条要求）；其余一致。

- 规则文字写「一行指到叫停原话的出处（仓里 `records/`、`research/`、`.claude/` 起头的 `.md` 路径，加用户叫停的原话）」，没写：原话要用「」括起来（r-nobracket 不收）、至少 6 个字（r-short 不收）、标题行不算正文、路径和原话要在同一行。照文字写的人容易写出被拒的 halted 文件。
- 文字写的三种不收（空、没出处、原话找不到）钩子都按文字办：空与只有标题的文件收不了，原因在拒绝说明里写出（`halted_round_problem`，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:696`）。
- 一行里有多处出处时，任何一处合格就收（`return None` 在循环里）。
- 推翻条件：把 r-nobracket 与 r-short 两种写法喂进去得到放行，或规则文字里找得到「」、6 个字这两条要求。

### D6-3　与 `research/prompts/abandoned-rounds.tsv` 是不是同一件事两处登记

判：是，而且两边互不读。

- 门禁读那张表：「发过腿、没有判决、也没登记 ⇒ 红」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/gate.d/doc-process-records.sh:99`）。halted 文件它不认：在仓里搜 `halted`（排除 `research/`、`target`、`fixtures`）命中的只有三方规则、派发闸、几份 E 系列实验文件，`.claude/gate.d/` 与 `scripts/` 里没有任何一处读 `-halted.md`。
- 派发闸读 halted 文件，不读那张表：`.claude/hooks/runner-dispatch-guard.sh` 里没有 `abandoned`。
- 已经有一个双登记的实例：`research/prompts/m3-batch63-code-r1-halted.md` 第 5 行写出处与原话，`research/prompts/abandoned-rounds.tsv` 第 9 行同一轮写了日期、理由（也引了用户那句话）、去向。照规则文字只写 halted 文件的叫停轮，发过腿的话门禁 66 号（`abandoned-rounds` 格）仍红；只在表里登记，派发闸 ⑥ 仍拦。规则文字（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/three-way-inference.md:121`）只提 halted，没提那张表。
- 推翻条件：能找到一处读 halted 文件的门禁代码，或读 `abandoned-rounds.tsv` 的钩子代码。

## D7　长活不写死台数与额度

文字：「长活默认分片、能断点续跑」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:55`）；「不写死台数与额度（几台、哪几张卡、每台多少内存）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:55`）；「预计超过 2 小时的照派发闸 ⑭」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:55`）；「N 取登记的每台默认重活线程数」类的线程上限一行（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53`）。

### D7-1　照做的人写不写得出同时满足三条的提示

判：写得出，钩子放行；三条之间有一处二义、两处钩子窄于文字。

- 写得出：`exp/d7-ok.md`（experiment-runner；有「线程上限：31」「预计挂钟：超过 2 小时，…」「分片：」「续跑：」，「多机：」一行点名 `layer0-shard-configuration-check.sh --emit-hosts` 与 `host-resources.py --json`、不写数字）`--check-prompt` 的原样输出：`✓ 派发闸照这份提示放行（experiment-runner，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口…、⑯ 草稿目录写者锁…`，退出码 0。
- 二义：「多机」一行要求「每台的线程与内存从 host-resources.py --json 现算」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/dispatch-prompt.py:69`；钩子拒绝说明同句 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_long_run.py:98`），「可用线程 = 核数 − 此刻在用的核」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/host-resources.py:27`）是看忙闲的数；「线程上限」一行是固定 31、不看忙闲（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53`）。同一份提示里对「每台开多少线程」有两个出处，没有一句说哪个管哪件事。本机现测 `--json` 报 `"available_threads": 0`（D1-1 原样）。判：文字没写到。
- 钩子窄于文字之一：只判 `experiment-designer` 与 `experiment-runner`：「LONG_RUN_AGENTS = {」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:1029`）。喂 `--type clerk`、`investigator`、`tooling-writer` 同样写「预计挂钟：超过 2 小时」不带三行，长活声明拒绝行数都是 0，`experiment-runner` 是 1。文字「预计超过 2 小时的照派发闸 ⑭」没限类型。
- 钩子窄于文字之二：「写死台数与额度」只在「多机」一行里按数字加单位判：「HARD_CODED_QUOTA = re.compile」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/lib_long_run.py:57`）。喂法与结果（长活声明拒绝行数，0 是放行）：`多机：两台，每台三十一线程、一百二十八吉内存，用 --emit-hosts 与 host-resources.py --json 核` 0；`多机：用 3 张卡（0、1、2 号），…` 0；`多机：…每台 31 线程…` 1；`分片：按 2 台各 31 线程、每片 16G 切` 0。「哪几张卡」没有任何一格拦；分片与续跑两行里的数不判。判：文字没写到钩子的射程。
- 推翻条件：D7-1 第一条被推翻要求 `exp/d7-ok.md` 被拒；二义被推翻要求冻结副本里有一句说明「多机」行里的线程与「线程上限」的关系。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| D1 默认 31 的可达反例 | 文字没写到 / 不一致 | E161 只读自己的变量、变异门禁 16 个进程、E171 按忙闲取小（本机现测 0）、GPU 一趟 `RUST_TEST_THREADS=31`；`crates/` 默认与变异门禁在排除表里登记 |
| D1 用户临时让出 CPU | 文字没写到 | 删掉「N 变了发消息」后，没有一句管在跑子 agent 的线程怎么降；改前也只管重活数变化 |
| D1 ⑮ 与参数 | 一致，另有一处不一致 | 放行行与参数一致；开头是 31 的公式（`31 ÷ 重活数`、`31/2`）钩子放行，文件头与 `agent-common.md:85` 说拒 |
| D1 GPU 一趟 | 不一致 | `verification.md:59` 要 `--test-threads 1`，`checker-tier-layer0-replay.sh:760` 没带，`RUST_TEST_THREADS=31` |
| D5 依赖 checker 档包的 crate | 不一致 | `-p <独立探针包名>` 退 0；其余写法退 2；bench、直接执行二进制、虚拟根 run 文字没写 |
| D5 主工作区合入锁 | 一致，文字没写到 | 核心一致；放行的子命令只六个、脚本里的 cargo 不判、`cp -r` 带 .git 的副本照拒 |
| D5 写闸第九道、⑯ | 一致，文字没写到 | 嵌套目录互相拦；`lib_writer_lock.py:4` 残留旧编号 ⑮ |
| D5 开工先认锁 | 文字没写到 / 不一致 | 8 份只有 Bash 的定义无从触发；回执找不到时先到先得，比文字强；对不上回执的拒绝说明不点名 |
| D5 看门狗两样 | 一致 | 文字漏写「被停」也算；已失效的判定没喂，只读代码 |
| D6 halted 能否关掉还要判的轮 | 能（文字允许） | 自引自己、引无关文件都收；改前的「快照冲突已判」行同样能关，另立一笔账 |
| D6 ⑥ 判法与文字 | 文字没写到 | 「」、≥6 字、标题行不算、同一行四条钩子要求文字没写 |
| D6 与 abandoned-rounds.tsv | 同一件事两处登记 | 两边互不读，`m3-batch63-code-r1` 两处都登记了 |
| D7 同时满足三条 | 写得出；二义一处、钩子窄两处 | 多机行的「线程现算」与固定 31 二义；⑭ 只判设计员与执行员；额度只在多机行按数字判 |

## 没做什么

- 没判 D2、D3、D4（不归这条腿）；没替主 agent 采纳或出判决；没给出改法的最终措辞（只在个别条里点了落点）。
- 没起 cargo，没跑任何登记的 `--selftest`，没跑重型测试。
- 复核不了：① 重启后旧进程的派发回执还在不在主会话记录里（要真重启客户端）；② 「已失效」判定（要造 `~/.claude/sessions` 登记，我不碰 `~/.claude/`）；③ 第二台的核数与它上面的 `available_threads`（私有主机信息不进报告，也没连）；④ 前台派发的子 agent 在主会话记录里有没有 `async_launched` 回执（没有的话，写者锁全是先到先得）。
- 没喂：编译目录的过期判（我自己造数据把 `CACHEDIR.TAG` 的修改时刻写成了现在）、E171 与 E161 装置本身（只读代码与读环境变量，没起）。
- 草稿目录里的 `tree/`、`probes/` 在交回前删掉，脚本（`feed-heavy.sh`、`lockcases.py`、`lockcopy.py`、`wlcases.py`、`wlsub.py`、`dispatch16.py`、`hogs.py`）与 `exp/` 留着；重建 `tree/`：把冻结副本的 `.claude`、`scripts`、`records`、`crates`、`CLAUDE.md` 拷到 `tree/`，D6 的假轮按 D6-1 一节的描述造。
