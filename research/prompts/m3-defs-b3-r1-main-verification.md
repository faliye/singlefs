# 工具与治理批的定义与规则改动：第一轮判决（2026-10-12）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 D6 D7 A1 A2 A3 B1 B2 B3 C1 C2 E1 E2 E3 F1 F2 F3 G1 G2 G3 H1 H2 H3 -->

正文 `research/prompts/_m3-defs-b3-r1-body.md`；腿：正推 `research/prompts/m3-defs-b3-r1-sonnet-output.md`（sha256 60ed02db…a8a8）、攻方 `research/prompts/m3-defs-b3-r1-opus-output.md`（sha256 afb4a1fb…6645，模型 `research/prompts/m3-defs-b3-r1-opus-model/`）。岔路单 `research/prompts/m3-defs-b3-r1-forks.md`。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/rules/three-way-inference.md`、`.claude/skills/crash-test/commit-stages.md`。

## 核查记录（主 agent 做）

- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b3-r1/repo/` 对 `research/prompts/m3-defs-b3-r1-snapshot/frozen-copy-sha256.txt` 21 份 `sha256sum -c` 全过，腿跑着的时候没被改。
- `python3 scripts/agents/cite-check.py <两份报告> --root <冻结副本> --background research/prompts/_m3-defs-b3-r1-background.md`：「✓ 核了 88 处引文，对上 88 处，没判 0 处」。
- 攻方模型拷进主 agent 草稿目录（`find … | sha256sum` 与报告里抄的逐行相同），经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh`：五行 `name=compare file=d1|d3|d4|d6|d6-cell same`。
- 主 agent 另走一条路现查正推的几条：`bash scripts/hosts/capped.sh --print 8 | grep -c E161` 输出 0，`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` 第 120 行只读 `E161_THREADS`；`.claude/gate.d/checker-tier-layer0-replay.sh` 第 750–765 行 `--test-threads` 0 处；`.claude/agents/` 里 `tools:` 只有 Read、Bash（prior-art 另有 WebFetch、WebSearch）的 8 份；在依赖 `singlefs-checker-tier` 的独立探针目录里调冻结副本 `lib_heavy_tests.classify`：`cargo test`、`cargo test --manifest-path Cargo.toml` 判 `checker-tier-cargo`，`cargo test -p attack-probe`、`cargo run -p attack-probe` 返回 `None`。
- E1 主 agent 现查（只数命中、不打印值）：`records/2026-09-30-里程碑三开工第一轮.md` 第 1004 行含 `multi-host.env` 里 `PEER_SSH_HOST` 的值，已提交（3e3cdc3c）；`research/prompts/m3-batch10x-code-r1-opus-output.md` 第 295、302 行含本机 `hostname`，已提交（ce1457c3）；`git status -sb` 是 `## master...origin/master`，HEAD 与 origin/master 同为 ce1457c3，即两处都已推到远端。
- 核不动的：GPU 那一趟借不到显存时「本次未跑」不判红（B1 的后果）要跑带 gpu 特性的快档才能坐实，没跑；重启后旧进程的派发回执、看门狗「已失效」判定两腿都只读了代码。

## 逐格判决

| 格 | 判 | 依据（腿与主 agent 现查） | 去向 |
|---|---|---|---|
| D1 | 站不住 | ⑮ 只比开头整数，「线程上限：31 ÷ 2」「31 的一半」「31（本机），第二台 16」都放行，没写这一行的派发不判（两腿各自量到，攻方 A3 复跑一致）；共用约束许可的 `capped.sh N` 与 `capped.sh default` 在设了 `SINGLEFS_THREAD_CAP` 时交不同的数（攻方 A1，量过）；E161 只读 `E161_THREADS` 而 `capped.sh` 不设（主 agent 现查）；GPU 那一趟与「`--test-threads 1`」不一致（主 agent 现查）；临时让出 CPU 没有规则管（两腿；改前的文字也中，另立一笔账，岔路 H3） | 改法 G1–G5，进下一批；H3 交用户 |
| D2 | 站不住 | B1 与 D1 的 GPU 那一条同一件事（量过）；B2「配置里的机器与卡都用」与单个 GPU 测试二进制只用本机点名的卡不符（推的，读码）；B3「20 万态」没写单位、射程，和 `.claude/main-agent.md`「禁止」第一条「挂钟长不是收窄它的理由」互相拉扯（推的） | 改法 G6 |
| D3 | 站不住（与用户 2026-09-28 的话冲突，交用户） | C1：按哈希「照现状登记」让别的会话没审过的 `crates/` 改动拿到豁免（`exempted=yes`），改用只暂存自己的块又让这一批自己的门禁红（`exempted=no`），攻方量过、复跑一致；C2 compile-then-swap 只核开跑之后的改动（改前也中，另立一笔账） | 岔路 H2 |
| D4 | 站不住 | E1：仓里已有两处私有值且已推到远端（主 agent 现查坐实）；E2：射程只写第二台，本机主机名经日志进仓；与「贴原样输出」「整行抄原话」「证据不许事后改」三条撞（推的）；E3：能做成只判新加行、按长词的会红检查（误报数量过） | 岔路 H1；E3 形态记下一批工具活 |
| D5 | 部分站不住 | 正推：`-p` 点名独立探针放行（主 agent 现查坐实），bench、直接执行探针二进制、虚拟工作区根 `cargo run` 文字没写；合入锁核心一致，脚本里的 cargo 不判、文字没写；写者锁嵌套目录互相拦、`lib_writer_lock.py` 第 4 行残留旧编号 ⑮；「开工先认锁」对 8 份只有 Read、Bash 的定义触发不了（主 agent 现查），回执找不到时钩子先到先得、比文字强，回执对不上时拒绝说明不点名持锁者；看门狗两样一致、文字漏「被停也算」 | 改法 G7–G9 |
| D6 | 站不住 | halted 引自己、引这一轮正文、引规则那一句、引无关文件里的一句都能关掉还要判的轮（两腿各自量到）；出处写 `research/../../…` 指到仓外照收（攻方 F2）；钩子另要求「」括、至少 6 个字、同一行，文字没写；halted 与 `research/prompts/abandoned-rounds.tsv` 是同一件事两处登记、互不读（两腿）。另：派发闸本来就有的「快照冲突已判：<轮名> <理由>」理由非空就放行（正推；改前也中，另立一笔账） | 改法 G10 |
| D7 | 部分站不住 | 三条同时满足的提示写得出（正推 `--check-prompt` 退 0）；但「多机」行要每台线程按 `host-resources.py` 现算、与固定 31 二义；⑭ 只判 experiment-designer、experiment-runner，文字写成对所有长活；「写死额度」只在多机行判 | 改法 G11 |

## 改法（全部被攻过零轮；进下一批工具与定义活，改完开第二轮）

| 改法 | 改什么 | 修的是哪一格 |
|---|---|---|
| G1 | ⑮：「线程上限：」后面只认一个十进制整数（后面只许接中文括注），出现 `÷`、`/`、「一半」、两个及以上数字的判红；任务带重活的类型（implementation-writer、investigator、tooling-writer、experiment-runner、mutation-triage、三方攻方腿）没写这一行判红 | D1 ⑮ 放行公式与缺行 |
| G2 | 共用约束：子 agent 起重活一律 `capped.sh N`（N 照派发提示）；`capped.sh default` 只给主 agent 与脚本内部用；登记文件头「`SINGLEFS_THREAD_CAP` 设了照它」与派发提示的关系写成一句 | D1 A1 |
| G3 | `capped.sh` 设的变量补 `E161_THREADS`；门禁格 heavy-thread-defaults 加判「装置自己的线程变量没设时不回退到 `SINGLEFS_THREAD_CAP` 或登记值」 | D1 E161 |
| G4 | `.claude/gate.d/checker-tier-layer0-replay.sh` 带 gpu 特性那一趟加 `--test-threads 1`（CPU 一侧线程照登记值）；轻型 `scripts/crash/crash-point-replay.sh --quick` 里带 gpu 的那几项同改 | D1 GPU、D2 B1 |
| G5 | 定义写明还没改成 31 的去哪查：`.claude/heavy-thread-exempt` 的「代码批」「与依赖产物共用一起改」行 | D1 规则让人以为是 31 |
| G6 | `verification.md`：「20 万态」写成「日常验证（harness 档与轻型三条）的崩溃状态数」、注明不管提交与全量；「配置里的机器与卡都用」改成说驱动（崩溃放量、对拍驱动）照多机配置用、单个 GPU 测试二进制只用本机点名的卡 | D2 B2、B3 |
| G7 | `lib_heavy_tests.py`：`-p`/`--package` 点名的包先在当前目录与 `--manifest-path` 那一份的包名里找，再找工作区成员；bench 同判；定义写明「直接执行编出来的二进制」「脚本里起的 cargo」两样不判 | D5 探针、合入锁射程 |
| G8 | 写者锁（2026-10-12 主 agent 更正：嵌套目录照旧按最近一层的锁判，钩子不改——正推 `wlsub` 例里 A 持父目录、子目录另派给 E 持锁，A 写 E 的子目录被拒、E 写父目录被拒都是对的，原句「只认最近一层」把它读成要改钩子是主 agent 判错；只在文字里写明子目录另派之后父目录持锁者不写它）；`lib_writer_lock.py` 第 4 行编号改对；回执对不上时拒绝说明点名持锁者；「开工先认锁」改写成只对有 Write/Edit 的定义、并写明回执找不到时先到先得 | D5 写者锁 |
| G9 | `main-agent.md`「派出去之后」补「被停的同样算」 | D5 看门狗 |
| G10 | 叫停轮只留一处登记：派发闸 ⑥ 改读 `research/prompts/abandoned-rounds.tsv`（门禁 doc-process-records 的 abandoned-rounds 格已读它；2026-10-12 主 agent 更正，原写 doc-experiments），不另立 halted 文件；登记行的出处必须是 `records/` 下现存文件的行号、那一行里要有轮名与「」括的用户原话，不许指这一轮自己的材料、规则文件或仓外路径；规则文字照钩子的实际判法写全 | D6 |
| G11 | ⑭ 的「多机」行改成台数取多机配置、每台线程取登记值，不现算；⑭ 判全部会带长活的类型，或文字收窄到它判的两种，二选一按改法 G1 那份类型表 | D7 |

另立一笔账（改前也中，不拿来否这次改动）：`compile-then-swap` 只核开跑之后的改动（C2）；派发闸「快照冲突已判」理由非空就放行（D6）。

## 交用户（岔路单 `research/prompts/m3-defs-b3-r1-forks.md`）

- H1：已推到远端的两处私有值怎么处置，私有值与「原样」三条规矩撞时谁优先。
- H2：按哈希登记的豁免表遇到别的会话没审过的改动，照现状登记还是不登记。
- H3：用户或别的会话要求临时让出 CPU 时，在跑的子 agent 怎么办。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则与 skill 的文字和钩子行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 更正（2026-10-13 第三轮判决）

这份判决里引的数有对不上产物的，以 `research/prompts/m3-defs-b3-r3-main-verification.md`「数字更正」一节为准，原句不改。
