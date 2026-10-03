# 第 4 批代码轮：第一轮判决（2026-10-02）

<!-- doc-lint:not-numbers R-换 R-独 C1 C2 C3 C4 C5 C6 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_m3-batch4-code-r1-body.md` | — |
| 正推腿（Sonnet） | `research/prompts/m3-batch4-code-r1-sonnet-output.md`（341 行），模型 `m3-batch4-code-r1-sonnet-model/` | 084db498 |
| 攻方腿（Opus） | `research/prompts/m3-batch4-code-r1-opus-output.md`，模型 `m3-batch4-code-r1-opus-model/` | 6840af27 |

- 快照：冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/` 对 `research/prompts/m3-batch4-code-r1-snapshot/tree-sha256.txt` 跑 `sha256sum -c`，退 0。
- `cite-check.py` 两份报告一起跑（`--root` 冻结副本与主仓，带背景材料）：「核了 53 处引文，对上 53 处，没判 0 处」。
- 主 agent 现查（冻结副本）：
  - I-8.1 的 F：`crates/singlefs-checker/src/image.rs:838`「safety_factor >= JOURNAL_SAFETY_FACTOR_LOWER_BOUND,」；`crates/singlefs-core/src/recovery.rs:847`「if u64::from(recorded.safety_factor) != JOURNAL_SAFETY_FACTOR {」；`.claude/kb/invariants.md:90` I-8.1 那一行：「安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，读者要求 F ≥ 2），任一不等判红」；
  - GPU 判器覆盖：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:61` 起 `POOL_CHECKER_COVERED_INVARIANT_INDEXES` 只列下标 0–48；对拍 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1101` 两侧都 `& covered`；
  - 指针 extent 偏移：`.claude/kb/invariants.md:133` I-2.4 那一行「core 读指针头部的两个读者同改」。
- 这一轮判的 `crates/*/src/` 文件：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/pointer.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/crash_identity.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`、`crates/singlefs-harness/src/history.rs`。
- 两条腿的数都在冻结副本的拷贝上量；攻方原型里的崩溃状态合计不到 0.2 万个，枚举时单元写段整段落盘、不取子集。两条腿各格的「没打中」都是一次抽样。checker 档一样没跑。

## 二、逐条判决

| # | 格 | 两条腿 | 判 |
|---|---|---|---|
| 1 | C1 十二条与四处改写 | 正推逐句对：I-1.12、I-8.10、I-8.11、I-4.9、I-3.13、I-9.26、I-3.12、I-7.16、I-7.17、I-8.12、I-7.15、I-7.5 与 I-1.2、I-3.9（B7）、I-7.8、I-2.4 的 checker 一侧都与 kb 对得上。攻方：mkfs 收下的 87 组几何、400 个崩溃状态、六档 288 段随机历史上 checker 0 红（一次抽样） | 认 |
| 2 | C1 I-9.25、I-4.1、I-8.5 | 正推：代码自加了 kb 没写的收窄或不适用（报告第二节） | 记：kb 行写回时把这几处收窄补进对应那一行，或改代码到 kb 字面——随第 4 条一并由写回员照正推第二节逐条定 |
| 3 | C2-1 / C1 / C4 I-8.1 的 F | **两条腿同时打中**：四槽 F 改成 2（攻方另量 4、只坏一个非择到的槽），在飞上限与最坏占用按槽里的 F 算齐、重封：checker 0 红，core 只读与可写挂载都整池拒 `JournalSafetyFactorNotTheFirstVersionValue`。条款字面「F = 格式常量，任一不等判红」站在 core 一边 | **打中**。改 checker：F 那一格改成等于 `JOURNAL_SAFETY_FACTOR`（攻方甲，副本上量过：三种都变红、既有用例照过）。零轮，下一批 |
| 4 | C2-1 附带：三个字段不等时 | 攻方：core 走「读者不收」整池拒；checker 只红 I-8.1、其余照判，与 I-7.13 那八项「其余一律不适用」不同口径 | 采攻方乙（推的）：checker 把 I-8.1 三格不等归进与 I-7.13 同一口径的整池不作保。零轮，下一批 |
| 5 | C4 指针 extent 偏移 | 正推打中：extent 树根里那条数据指针 extent 偏移改 1，checker 红 I-2.4，core 只读挂载与读第一个字节都成功；core 只判 journal 新根段、码 2 内部条目、根记录三类读者，不判 `records.rs` 345 / 374 / 386、`instance_table.rs:206`、`extent_tree.rs:327`、`allocation_record_tree.rs:774` 六处 | **打中**。core 改成每个读指针头部的读者都判 extent 偏移（与第 4 批交回时已定「照 D19（块指针的结构与宽度预算） 已定项 3 是全部指针」同一件，进下一批）；I-2.4 行「两个读者」一句写回时改成全部读者 |
| 6 | C4 根记录 extent 偏移 | 正推：只把最新根实例表指针的偏移改 1，core 退到上一条根、靠 journal 重放回同版、挂载成功；checker 红 I-2.4、I-7.2 成立 | 不记打中：条款没写拒还是退。记进交用户表之外的写回清单：I-2.4 行写明「根记录里指针偏移非 0 按自证不过处置、退回上一条根」（第 4 批 core 报告第 ③ 问主 agent 已认），checker 红 I-2.4 而 core 挂载成功两边各判各的不矛盾 |
| 7 | C4 C463 | 正推：只做了只读挂载那一半 | 记进下一批：可写挂载那一半与 `RecoveryReport` 字段（第 4 批 core 报告第 ④ 问）一起做 |
| 8 | C2-2 I-4.1 读环 | 攻方打中（手造镜像）：盘 0 计数器 4 换成实例 2 的一条记录、盘 1 仍是 B：checker 重放落在 (1, 3)，core 落在 (1, 4)；再坏盘 1 那份 B 的树表指针，core 恢复失败而 checker 0 红。可达性是推的：第一版两盘池上不可达 | 打中（形态，推为今天不可达）。checker 读环改成与 core 同样按 (实例, 计数器) 建键，消掉分歧。零轮，下一批 |
| 9 | C2-3 R-独 对 `effective_root` | 攻方：两段历史 400 个崩溃状态逐个差分全部一致（一次抽样）；层 0 上的逐状态差分今天没有能跑的入口 | 没打中。层 0 差分要先开口子（攻方 `walk-differential-hook.patch`），记进下一批清单 |
| 10 | C2-4 R-换 三处读法 | 攻方：384 个二次崩溃状态读出内容相同、再挂被拒 0、读不出 0（一次抽样）；读代码推：二次崩溃之后再挂被拒只计数不判，而第二截对第一次挂载被拒按模型判，两截不对称 | 没测出误红。提交时主 agent 跑崩溃注入快档看「二次崩溃之后再挂」那一行「不同」= 0 与 TheRemountWasRefused 计数；不对称那一处记进下一批清单 |
| 11 | C3 违例位 | 正推：位序、偏移、字数两侧逐位对得上；判法摘要 v4 生产路径不读、旧判定不沿用。**第 49–64 位 16 条新不变量 GPU 内核没判、位恒 0，对拍两侧都 `& covered` 屏蔽**（主 agent 现查坐实） | 位宽认。GPU 没判 16 条：崩溃放量走 GPU 判器时这 16 条一条都没判，与「崩溃点测试准确率优先」不符——GPU 内核补这 16 条，进下一批；补齐之前崩溃放量全量的结论写明「新 16 条只在 CPU 一侧判」 |
| 12 | C5 测试改动 | 攻方：同步助手八个调用点在调它之前都没改过记录，抹不掉故障；两条改期望按条款推得出；线索：新用例把「非最新候选根的水位压低」钉成不红（一次抽样） | 认。线索那一条：非最新候选根水位压低在哪条不变量的射程里，写回 I-9.6 行时核 |
| 13 | C6 变异表 | 正推：harness 档 45 行 43 抓到、2 行没红（第 1782、1783 行，那份坏镜像同时破 I-7.16 两格，任一格单独恒真用例照绿）；checker 档第 1661 行按读法不红（结论缓冲填全 1，种假位的用例看不出）；31 个弱化点 22 个整套 harness 不红（I-4.1 重放六条规则、I-8.5 一类、I-9.26 下界、I-8.11 点名项四段、I-8.10 三段、I-2.4 三个调用点、I-7.5、I-7.17 槽格、I-8.1 相等边界、I-9.25 inode 树那一半） | 打中。1782、1783 拆成各破一格的两份坏镜像；1661 换一条在全 1 缓冲上也看得出的写法；22 个弱化点各补一份只红它的用例与变异行。下一批 |
| 14 | kb 状态列 | 正推：`grep -c '^| I-.*已实现' .claude/kb/invariants.md` 49，`IMPLEMENTED_INVARIANTS` 65 | 第 4 批新立 16 行状态从「未实现（实现在途）」改「已实现」，与第 2、5、6 条的措辞一起写回 |

## 三、改法（定下的，第 5 批下一波）

实现（三簇，文件不相交）：
- 池级 checker：I-8.1 F 改等于（第 3 行）、三格不等整池不作保（第 4 行）、I-4.1 读环按 (实例, 计数器)（第 8 行）、第 1782 / 1783 行拆镜像与 22 个弱化点补用例与变异（第 13 行）。
- core：全部读指针头部的读者判 extent 偏移（第 5 行）、C463 可写挂载那一半与 `RecoveryReport` 字段（第 7 行，连带 checker 档 `e158_root_choice_repair.rs`）、第 4 批 core 报告第 ① 问的改名。
- checker 档：GPU 内核补 16 条新不变量、`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 补齐、第 1661 行变异改写（第 11、13 行）。
写回（一份规格）：第 2、5、6、12、14 行。
提交时：第 10 行崩溃注入快档那一行；GPU 补齐之前崩溃放量全量的结论限定。
I-7.8「同一树 ID 发两次号」第二份坏镜像条款没写（第 4 批收拢），这一轮不涉及，仍在下一批清单。

## 四、交用户

无：两处 checker 与 core 的分歧都由条款字面判出（第 3、5 行），其余改法是让实现对齐已定条款。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D19（块指针的结构与宽度预算） 已定项 3 | 支撑 | 2026-10-02 不受影响：extent 偏移判全部指针，core 对齐它 |
| D23（journal 的角色与格式） 已定项 4 | 支撑 | 2026-10-02 不受影响：I-8.1 的 F 照格式常量判，与 core 同 |
| D13（验证路线） 已定项 9 | 备料 | 2026-10-02 不受影响：GPU 判器补齐 16 条之前，崩溃放量全量对新不变量只有 CPU 一侧 |
