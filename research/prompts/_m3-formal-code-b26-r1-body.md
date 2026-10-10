# 第 26 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers J1 J2 J3 J4 R1 R2 F3 A2 B3 C2 KX1 KX2 RR8 T5 T9 -->

## 一、题面

第 26 批（`records/2026-10-01-里程碑三形式化接入.md#26`）合入了三份补丁，改动全文 `research/prompts/_m3-formal-code-b26-r1-diff.md`（照实现员交回的原样，按合入次序）：

- 26-3：照判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第三节第 1 条改。池级 checker `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`：I-9.16 加 ③ ④；I-5.5 判不了的几格改「部分没判」或跳过；I-2.4 加根槽 428..457 全 0；带树一版分配记录树根全零挂在 I-5.4。挂载透明 `crates/singlefs-harness/src/mount_transparency.rs`：三根指针并进树表条目根指针那一行，`mapping_root` 与分配记录树根的空性关系，journal_tail 与系统配置 journal 实例代号两行，按字节区间的字段表；core `crates/singlefs-core/src/recovery.rs` 加读原字节的只读函数，`crates/singlefs-core/src/allocator.rs` 加回落计数。主 agent 对它的取舍在记录第十八节「26-3 交回」那一条（I-5.4 挂法、`user_visible_tree_root_pointers` 不扩、magic 不比等）。
- 26-1r：26-1 在新树上的重打。C664 拒因分开（`crates/singlefs-core/src/mount.rs` 新成员 `MountError::SystemConfigurationUnreadableAfterTheReadStage`，删 `WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`，挂载透明里拒绝合法的复算）；C696 候选「丁」（用户定）：随机历史新步「每块盘最新那一槽系统配置读回全 0 再可写挂载」与新比重、已有比重的指纹钉住、checker 档新用例 `crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`；F3-e 三条取样用例（判决 `research/prompts/m3-formal-code-b24-r1-main-verification.md` 第三节第 2 条，被攻过零轮）。主 agent 收的两处收严：注入读故障的态上新拒因复算为假不算不合法；N-配置 复算去掉「有盘一份都没有就按真」那一支。变异表替换 5 行、删第 9 批 RR8 一行。
- 26-2：HEAD 上 8 条红的钉值（#1 `crates/singlefs-checker-tier/src/crash_identity.rs` 补登记；#2–4 系统配置轮换用例的判法按盘放行 mkfs 清零；#6 门槛 10 → 8；#7 段序列 `[…,24,5]`、选择器 `<= 3` → `<= 5`、计数 1 → 9；#8 69 条期望值；#9 打印行与计划哈希，进度文件哈希留待重录）与 C686 用例（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`）。

条款：上面三份判决；`.claude/kb/decisions/13-验证路线.md`「#### 已定项 1」挂载透明逐字段表；`.claude/kb/invariants.md` I-2.4、I-5.4、I-5.5、I-9.16 四行；`.claude/kb/checks-owed.md` C655、C664、C686、C687、C696 五行。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b26-r1/tree/`（主仓第 26 批合入后的 `crates/`、`Cargo.toml`、`Cargo.lock`，sha256 清单在同目录 `crates-sha256.txt`，297 行）。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录。
- 合入验证：26-2、26-3 那一趟 harness 档 153 个测试二进制全绿（`/tmp/claude-1000/merge-and-verify/2026-10-05-3861651/`），26-1r 那一趟 155 个全绿（`…/2026-10-05-4130269/`），fmt、clippy 都过。checker 档一条没跑（根环线在跑合入前那棵树上的 checker 档快档、那 8 条点名重跑还没回）；新加的 checker 档用例与各份变异行只编过。
- 实现员证红：26-3 证了 2 行（其余 21 行留提交时）；26-1r 8 条全抓；26-2 C686 证过。

方案按 `crates/` 今天的实现来谈。

## 三、格

### J1　26-3：C655、C687 的改法
代码做的是不是判决说的；判决第五节那六条零轮形态（I-9.16 ③ ④、根槽归 I-2.4、journal_tail 不等式与系统配置 journal 实例代号、`mapping_root` 与分配记录树根的空性关系、运行配置「R2 = 内置默认」、回落看门）哪一条在合法历史上误红、或放过一个坏实现；字节区间字段表的归类与区间有没有错、漏；I-5.4 挂法与「部分没判」的报法会不会让层 0 按判定种类钉的值或崩溃注入的计数出错（读码，checker 档不跑的写推的）。

### J2　26-1r：C664、C696 丁、F3-e
拒因分开之后两条拒因各自的合法判法对不对（尤其注入读故障的态上「复算为假不算不合法」与 N-配置 去掉那一支，是不是把该判不合法的放过了）；新步与新比重会不会改已有比重的取样（指纹钉住了七组）；二次崩溃镜像在注入态下照旧判那一处会不会误红；F3-e 三条取样（零轮）在更长历史、更多步数上罩不罩得住；替换的 5 行与删掉的 RR8 是否还各罩着原来那个行为。

### J3　26-2：8 条钉值与 C686
每个新钉值的出处（日志哪一行或哪段代码）站不站得住；#7 选择器放宽到 `<= 5` 有没有让那条用例失去它本来要判的东西；#2–4 按盘放行清零有没有放过一个本该停下的写序；C686 用例断言的落点、层级、出生序号是不是从条款推得出来。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | J1、J2、J3 | 从判决与条款正推每处改动做的是不是条款说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | J1、J2、J3，重点 J1、J2 的零轮形态 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b26-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b26-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
