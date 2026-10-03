# 第 4 批代码轮：二·1 不变量实现簇与违例位改宽，第一轮正文（2026-10-02）

<!-- doc-lint:not-numbers R-换 R-独 C1 C2 C3 C4 C5 C6 -->

## 一、题面

第 4 批（调度 `records/2026-09-30-里程碑三开工第一轮.md` 第 4 批与第 5 批 5-6）合入之后的代码，照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 2 步走一轮三方：核「代码做的是不是条款说的」、攻「哪一格会错」。

被判的改动（都已在主工作区，未提交）与压着的判决：

| 改动 | 补丁与报告 | 压着的判决与条款 |
|---|---|---|
| checker 档：违例位 64 → 128 位（GPU 判器、WGSL、CPU 对拍）、`crash_identity` 测试登记表不在时跳过、I-8.4 R-换 进崩溃注入二次崩溃 | `/tmp/claude-1000/m3-r1/batch4-checker-tier/patch/`（`report.md`） | `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 11 行；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节 I-8.4 那几行 |
| 池级 checker：两轮新立的不变量判法（清单 49 → 65） | `/tmp/claude-1000/m3-r1/batch4-checker/patch/`（`report.md` 第一节逐条落点） | 两份判决第二节、第三节；`.claude/kb/invariants.md` 那几行 |
| core：mkfs 拒不对齐几何、I-8.1 core 同判、指针 extent 偏移非 0 拒、C462 错误成员、C463 字段 | `/tmp/claude-1000/m3-r1/batch4-core/patch/`（`report.md`） | `research/prompts/m3-inv-a-r1-main-verification.md`；D19（块指针的结构与宽度预算） 已定项 3；`.claude/kb/checks-owed.md` C462、C463 |
| 合入后：测试助手同步 journal 新根段、改钉值、两处 clippy、六行锚点、助手合一 | `/tmp/claude-1000/m3-r1/batch4-postmerge/patch/`、`/tmp/claude-1000/m3-r1/batch4-anchors/patch/` | 同上 |
| 注释与 ignore 理由里的已删 agent 名 | `/tmp/claude-1000/m3-r1/batch5-agent-names/patch/` | `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行 |

## 二、实现今天的样子（主 agent 的观测，2026-10-02）

- 腿读冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`：合入之后的主工作区 `crates/`、`Cargo.*`、`.claude/kb/`，sha256 在 `research/prompts/m3-batch4-code-r1-snapshot/tree-sha256.txt`。合入之前的样子取 HEAD（`git show HEAD:<路径>`），或在冻结副本上反打补丁。
- 被判的 `crates/*/src/` 文件：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`；`crates/singlefs-core/src/make_filesystem.rs`、`mounted_read.rs`、`pointer.rs`、`recovery.rs`、`root_record.rs`、`system_configuration.rs`；`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crash_identity.rs`、`crash_injection.rs`、`crash_judge_dispatch.rs`、`crash_judge_gpu.rs` 与五份 `crash_judge_*.wgsl`；`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`（只改注释）；`crates/singlefs-harness/src/history.rs`。
- 合入后验证：harness 档四个包全部测试二进制跑过，红的 17 条改完在主工作区重跑全绿；fmt 过；`cargo clippy --all-targets --all-features -D warnings` 零告警；变异表 mutation-tables 格只剩 HEAD 就有的七处 `"../.."`。checker 档（层 0 快档、变异整表、GPU 对拍）一样没跑，归提交时。

## 三、格

### C1　逐条：判法是不是条款说的
池级 checker 新立与改写的每一条（清单见 checker 报告第一节，65 条里第 4 批新加与改写的那些）：判法与 `.claude/kb/invariants.md` 那一行逐句对，取哪个候选集、什么时候报不适用、读不出的怎么算。合法镜像上误红、坏镜像上漏红，各找一个就算打中。

### C2　实现员交出来的四处已知分歧
1. I-8.1 的 F：checker 判「F ≥ 2」、在飞上限按槽里的 F 算；core 拒 F ≠ `JOURNAL_SAFETY_FACTOR`（3）。哪边对着条款，另一边在哪张镜像上与它分开。
2. I-4.1 读环：checker `records_the_replay_reads` 按计数器取每个计数器第一块盘上读得出的那一份，core 按 (实例, 计数器) 建键。两块盘同一计数器坐着不同实例的记录时两边分不分开（造镜像）。
3. I-4.1（R-独）与 core `effective_root` 的逐状态差分没做：在层 0 能枚举到的状态上两边会不会选出不同的根（推的或造的都行，标清）。
4. I-8.4 R-换 的三处读法（checker 档报告「三处读法」）：基准取「挂载写全落在第一次崩溃镜像上再只读恢复」；一边读得出一边恢复失败判红；没崩过的挂载被拒 / 之后只读恢复失败 / 二次崩溃之后再挂被拒只计数不判。这三处有没有让合法崩溃态误红、或把该红的放过。

### C3　违例位改宽
128 位、结论 12 → 14 字、判法摘要 v4：CPU 与 GPU 两侧位序、偏移、字数逐位对不对得上；第 64 条以上的不变量在 GPU 侧有没有落进没人读的字；判法摘要变了之后旧判定是不是一条都不沿用。

### C4　core 五条
mkfs 拒不对齐几何、I-8.1 core 同判、指针 extent 偏移非 0 拒（两种挂载都拒）、C462 错误成员、C463 字段：每条与条款与 checker 一侧逐条对；根记录 extent 偏移非 0 按自证不过处置、最新根坏时退回上一条，这一条会不会把合法根当坏。

### C5　测试改动
合入后两份补丁改了测试助手（`tests/common/mod.rs` 新加 `copy_each_root_into_the_new_root_segment_of_its_journal_records`，`checker_known_bad_images.rs` 自带的那份删掉改调它）与几条用例的期望（实例表链第 1 片多 I-1.2、I-3.9；c533 那条 I-8.11 改坏后不适用）。攻：同步助手会不会把坏镜像「修好」、让该红的用例变成只红别的；改期望的两条，新判定的依据站不站得住。

### C6　变异表
这一批追加与改写的变异行（`crates/mutations.tsv` 末尾那几十行与六行锚点改写）：替换文是不是真把被测的行为改坏了；有没有改了判法而没有一条变异盯着的地方（盲区）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | C1、C3、C4、C6 | 核「代码做的是不是条款说的」：逐条对判法与不变量行、违例位两侧、core 五条与条款 |
| 云端攻方（Opus） | C2、C5，兼攻 C1 | 在冻结副本的拷贝上造：合法镜像误红、坏镜像漏红、checker 与 core 在同一张镜像上分开 |

## 五、跑前写死的判据

- 一条判法在合法镜像（core 写得出的、或崩溃枚举到的）上红 ⇒ 打中，判法要改。
- 一份只改一处的坏镜像该红的那一条不红 ⇒ 打中。
- checker 与 core 在同一张镜像上对同一件事判得不同 ⇒ 打中，交主 agent 判哪边对着条款。
- 一条腿报「没打中」只抽了一次样；腿提的新形态算零轮。
- 腿不跑 checker 档（`singlefs-checker-tier` 的测试、层 0、变异整表）：要它的数就写成要主 agent 提交时跑的命令。

交付：
- 腿的报告写 `research/prompts/m3-batch4-code-r1-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；
- 攻方的副本与用例放 `research/prompts/m3-batch4-code-r1-opus-model/`；草稿放 `/tmp/claude-1000/<腿名>/`；
- 引 kb 条款写 kb 文件自己的行号（去冻结副本里那份 kb 现查）；方案按 `crates/` 今天的实现来谈。
