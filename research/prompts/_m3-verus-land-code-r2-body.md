# Verus 核接回 `crates/`：代码轮第二轮正文（2026-10-01）

<!-- doc-lint:not-numbers M1 M2 M3 M4 G2 G4 G5 G6 G08a K02 K19 K20 C1 C2 C4 C5 C6 H1 H2 H3 H4 H5 H6 S1 S2 S10 -->

## 一、题面

`.claude/rules/implementation-workflow.md` 三步里的第 2 步，第二轮：代码轮第一轮判决 `research/prompts/m3-verus-land-code-r1-main-verification.md` 第五节第 1、2、3 条的改法已由第 4 批三个实现员落进 `crates/` 并合入（`records/2026-10-01-里程碑三形式化接入.md`「第 4 批开工」之后各条）。这一轮只攻第 4 批的改法本身（被攻过零轮），不重复第一轮的角度（C1 等价、C2 spec 太弱、C3 编进二进制的与 Verus 看的、C4 胶水、C5 测试与变异、C6 编码纪律、C7 判法版本闭包、C8 E165 装置各自第一轮的攻法）。另一条输入：题面第三轮判决 `research/prompts/m3-formal-r3-main-verification.md` 第二节 S2（句法层 spec 要含连续、链首、在飞上限、提交标记）与 S10（必红不认 rlimit 用尽、verified 数判不出 spec 变弱、空真）。

被判的文件（判决逐份写去向）：

| 文件 | 第 4 批改了什么 |
|---|---|
| `crates/singlefs-replay-prefix/src/lib.rs` | 4-1：spec 补进 D23（journal 的角色与格式） 已定项 4、7、8、14、18、22 与点名校验、C540、报告字段、计数器加一不溢出；核报错带出计数器；计数器 / txg / 事务号 / 实例代号各立 newtype，两个布尔换枚举，改名，死子式删除，46 处 `[原文 N]` 注释删除；`#[verifier::opaque]`；`requires` 严格升序 |
| `crates/singlefs-replay-prefix/verus-mutations.tsv` | 4-1：整表 94 行（81 行改注解、13 行改可执行判定） |
| `crates/singlefs-core/src/recovery.rs` | 4-1：胶水直接拼 `RecoveryFailure`、删 `unreachable!` 与两份手抄；建表前 `assert_eq!` 键等于字段；断言 `counter < u64::MAX` |
| `crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs`（新） | 4-1：K19 回归 4 条 |
| `crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs` | 4-2：`rewrite_record` 重算后面记录的反向链；K20 一格 |
| `crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs`（新） | 4-2：K02，直接调 `scan_journal`、`choose_root`、`replay_journal`，阳性对照走 `recover` |
| `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` | 4-2：G08a 两条用例 F 抬到 4、5 并断言 |
| `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` | 3-6：`--pinned-cases`、三处 clippy、两处改名 |
| `crates/mutations.tsv` | 4-1 改写 18 行、追加 5 行；4-2 改第 43、723、738、1039 行锚点、追加 4 行（K20、K02 主 agent 合入时照 4-1 对照表改到新写法）、删第 1296 行；3-6 追加 3 行 |

另有 4-2 对 5 份 harness 测试只跑了 `cargo fmt`，不判。

## 二、实现今天的样子（主 agent 的观测，2026-10-01，主工作区合入之后）

- 冻结副本 `/tmp/claude-1000/m3-verus-land-code-r2/tree/`（`crates/`、`Cargo.toml`、`Cargo.lock`），sha256 清单 `research/prompts/m3-verus-land-code-r2-snapshot/crates-sha256.txt`（215 份）；读冻结副本，不读主仓 `crates/`。第一轮的冻结副本 `/tmp/claude-1000/m3-verus-land-code-r1/tree/` 是改前；两者之间这一批的 diff 在 `research/prompts/_m3-verus-land-code-r2-diff.md`（`crates/mutations.tsv` 里别的会话追加的行不判）。
- 主 agent 现跑：Verus 0.2026.09.20 `--no-cheating` 新核「32 verified, 0 errors」；`verus-mutations.tsv` 94 行逐行套上全红，其中 14 行夹一处 rlimit 用尽、每行另有至少一处真证明错（`/tmp/claude-1000/merge-4/verus-rows/`）；harness 档 95 个测试二进制全绿、clippy 退 0（`/tmp/claude-1000/merge-and-verify/2026-10-01-339319/`、`/tmp/claude-1000/merge-4/finish/`）；`crates-mutation-rows.py static-check .`「CHECKED 1503 199 9」、没有 ANCHOR。
- 实现员报告：4-1 `/tmp/claude-1000/m3-verus-core-4-1/report.md`（第二节 ③ 逐格写了没验到的：胶水的字段映射、回调本身判得对不对、C540 只在 K10 下与条款字面相同、M4 不在这一件）；4-2 `/tmp/claude-1000/m3-verus-tests-4-2/report.md`；3-6 `/tmp/claude-1000/m3-e165-pinned-3-6/report.md`。
- 没做的：4-1 没重做改后核与 fcc4d340 那一版的对拍；它改写的 18 行变异只证了第 355 行（其余留给提交时 checker-tier-crates-mutation-replay）。
- E165 第一段在改后树上逐字节重跑归第 5 批 3-2，不在这一轮。

## 三、格

### H1　改后核与改前等价
第一轮 C1 在改前核（第 3 批）上量过等价。第 4 批又改了核（newtype、枚举、改名、死子式、`judge_the_candidate` 把逐条 `break` 换成 `return CandidateVerdict::…`）与胶水（报错直接拼、删再调与手抄、两条新断言）：改后与 fcc4d340 的 `replay_journal` 在产品输入上还给同一个结局吗（返回的根、报告五字段、报错种类与内容、读盘次序）；两条新断言（键等于字段、`counter < u64::MAX`）在产品路径上会不会被触发（`scan_journal` 造得出的表）。

### H2　spec 补完之后证到了什么
spec 现在写进了哪些条款、哪几句与 D23 已定项 4、7、8、14、18、22 的字面对不上（过宽或过窄）；`#[verifier::opaque]` 有没有让某条后置条件变成空真或靠 reveal 才成立而 reveal 写漏；13 行改判定的变异之外，还有没有一处改可执行判定的变异今天照样 verified（S10 说的 spec 强度）；回调两判（反向链、点名单元）只能写成后置条件，今天写了没有、写对没有。

### H3　报错路径与 G6
核报错带出计数器、胶水拼 `RecoveryFailure` 之后：每一种核失败映射到的 `RecoveryFailure` 成员与内容，和改前（再调原函数拿清单那一段）一样吗；有没有一种核失败今天没有对应的成员、被吞或被映射错。

### H4　测试与变异（第 4 批新加的）
K19 回归 4 条、K20 一格、K02 新文件、G08a 两条、3-6 的 3 条单测，以及 4-1 / 4-2 / 3-6 追加与改写的变异行：每一条证不证得到它的名字说的那件事；合入时主 agent 照对照表改了锚点的 K20、K02 两行，改后的替换文还是不是原来那一判的变异；K02 那份测试绕过 `recover` 直接调三个函数，测到的是不是产品路径。

### H5　编码纪律（第 4 批新写的代码）
`.claude/singlefs-ai-sop/rules/code-discipline.md`：新 newtype 与枚举、改名、`assert_eq!` 消息、`#[allow]` 的 reason（3-6 的 `large_enum_variant`）、测试里的魔法数；逐条给文件:行。

### H6　E165 装置的 `--pinned-cases`（次要）
按 ②③④ 找状态、重算 ① 的写法与钉表文件头约定是不是一致；它与 `research/scripts/e165-pin-red-examples.py --replay` 的接口有没有一处两边各读一种。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | H1、H3、H6 为主 | 逐处核改后的代码做的是不是条款与第一轮判决要的；H1 在冻结副本的拷贝上对改后核与 fcc4d340 那一版逐个比结局（第一轮正推的对拍可以照用） |
| 云端攻方（Opus） | H1–H6，重点 H2、H4、H5 | 攻每一处改法：造反例、造变异，在冻结副本的拷贝上量；Verus 可以在自己草稿目录的副本上直接起（`~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus`，`--no-cheating`，经内存包装），把 rlimit 写成 10 |

第二轮，派正推不派辩方：第一轮判决里没有判出局的一方要辩护，要核的是改法本身。

## 五、跑前写死的判据

- 改后与 fcc4d340 在某个产品输入上结局不同 ⇒ 打中 H1，写回代码。
- 一处只改可执行判定的变异在改后核上照样 verified（rlimit 10）⇒ 打中 H2，记「spec 太弱」，补 spec 并把那一行进 `verus-mutations.tsv`。
- 一种核失败映射错或没有成员 ⇒ 打中 H3。
- 一行变异或一条测试证不到它的名字说的那件事 ⇒ 打中 H4。
- 编码纪律违例给出文件:行就算打中 H5。
- 腿提的新改法算零轮，标「零轮」。
- 不提前优化 `crates/` 里 core 的实现代码（用户 2026-10-01 定）：只攻正确性、证明、测试与编码纪律；为了快去改 core 的改法不提，提了判决不采。

交付：腿的报告写 `research/prompts/m3-verus-land-code-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-verus-land-code-r2-<腿名>-model/`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈，读冻结副本。
