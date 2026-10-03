# Verus 核接回 `crates/`：代码轮第一轮正文（2026-10-01）

<!-- doc-lint:not-numbers M1 M2 M3 M4 G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 C1 C2 C3 C4 C5 C6 C7 C8 -->

## 一、题面

`.claude/rules/implementation-workflow.md` 三步里的第 2 步：第 3 批合入（`records/2026-10-01-里程碑三形式化接入.md` 第五节「第 3 批开工」）改过的 `crates/` 代码走一轮三方。用户 2026-10-01 原话：「写完后 走三方捶打代码 从这几次的反馈来看 这部分代码质量一般」。要答：代码做的是不是条款说的；哪一格会错；证明与胶水之间漏了什么；测试与变异证不证得到它们声称的东西。

被判的文件（这一轮按路径点名，判决逐份写去向）：

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-replay-prefix/src/lib.rs`（新） | Verus 核：取重放前缀的判定、spec 与证明、两条普通单测 |
| `crates/singlefs-replay-prefix/Cargo.toml`（新）、`verus-mutations.tsv`（新，38 行） | 依赖 `vstd =0.0.0-2026-09-20-0158`、check-cfg、Verus 元数据；Verus 必红世界表 |
| `crates/singlefs-core/src/recovery.rs` | `replay_journal` 改成调核，读盘、反向链、点名校验成回调，报错换回、拼根留在原地 |
| `crates/singlefs-core/Cargo.toml`、仓根 `Cargo.toml`、`Cargo.lock` | 依赖新 crate、工作区成员 |
| `crates/singlefs-checker-tier/src/crash_identity.rs` | 判法版本闭包相关的两条测试清单加新 crate |
| `crates/mutations.tsv` | 21 行原位改锚点（行号、名字不变）、第 448 行换成真变异、追加 4 行 |
| `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（新，同一次提交里没过代码轮） | E165（恢复重放有界穷举与变异杀伤界） 的入库装置，次要对象 |

## 二、实现今天的样子（主 agent 的观测，2026-10-01，主工作区合入之后）

- 开工快照与冻结副本见第五节；读冻结副本，不读主仓 `crates/`。
- 实现员报告 `/tmp/claude-1000/m3-verus-land-3-1b/report.md`（sha256 f1e88273…2b3e）：Verus 0.2026.09.20.aef82ed `--no-cheating`「14 verified, 0 errors」；19 个 harness 目标前后 315 行无差；核里 `grep -n 'external_body\|assume\|admit'` 零命中；改锚点 21 行里 514、515、613 三行底子上就不红（报告第 4 条）；`tsv-448` 用 Verus 证了等价、原位换成真变异。结论句当线索。
- 接回前的可行性报告 `/tmp/claude-1000/m3-formal-verus-wire-in/report.md` 的 Q4 末表列了核之外没证的胶水 G1–G12。
- 已知、不算这一轮新打中的：核的 `m2_holds` 不要求计数器从链首起逐一连续，删掉 `expected_key` 不等即断那一判照样验过（主 agent 2026-10-01 用 Verus 0.2026.09.20 现跑坐实，第二轮判决 `research/prompts/m3-formal-r2-main-verification.md` 第二节第 2 行），补 spec 已排进第 4 批。攻它的推论（例如由此还漏了什么）算新的。
- `crates/mutations.tsv` 第 43、723、738、1039、1296 行锚点在 HEAD 上就腐化（主 agent `research/scripts/crates-mutation-rows.py static-check .` 现查），不是这一批的改动，不在这一轮的射程里。

## 三、格

### C1　核与原文等价
`recovery.rs` 改前（HEAD fcc4d340 第 3057–3229 行）与改后（`replay_journal` 调 `replay_core`）在一切输入上给同一个结局（返回的根、报告五个字段、报错的种类与内容、读盘次数与次序）吗？`is_above_the_chosen_root` 的抽取、两个回调的懒求值、`unreachable!` 新加的 panic 路径、多条末条标志时再调原函数拿清单那一段，逐处核。

### C2　证明证到了什么
spec 与 `ensures` 说的是不是 D13（验证路线） 已定项 1 M2 与 D23（journal 的角色与格式） 已定项 4、已定项 14 的那几句；`requires` 在产品路径上谁担保（计数器不是 u64 最大值、回调的 `requires`）；有没有一处证明靠的是空真、恒真的 spec 或过宽的循环不变式；`verus-mutations.tsv` 38 行改的是不是有意义的判定（只改 spec 让它红不算证明有判别力）。

### C3　编进二进制的与 Verus 看的
平时编用 crates.io 的宏把 `verus!` 擦成普通 Rust、验用 Verus 自带的 09-20 发布；两份源码同源（第五节）。还有没有别的差别：cfg、`#[verifier::…]` 标记在平时编时的语义、幽灵代码擦掉之后留下的可执行代码。

### C4　胶水
G1–G12 里哪几条今天最可能出错，各给一个能造出错的输入或改法；新 crate 的公开 API 被别的调用方误用的可能。

### C5　测试与变异
两条新单测、21 行改锚点的变异（每行改的是不是原来那一判）、514 / 515 / 613 不红的原因（实现员说是取样点不敏感）、`crash_identity.rs` 的两条测试与追加两行变异，证不证得到它们声称的东西。

### C6　编码纪律
`.claude/singlefs-ai-sop/rules/code-discipline.md`：名字、`expect` 消息、穷举 `match`、`as` 转换、注释只写为什么、`#[allow]` 写 reason；逐条给文件:行。

### C7　判法版本闭包与崩溃放量
改核之后判法版本变不变（`crash_identity.rs` 收法、`use` 自动进闭包这句成不成立）；GPU 判器另写的 WGSL `replay_journal` 与改后的核还一致吗（主 agent 的观测：GPU 判器不调 core）。

### C8　E165 装置（次要）
它判的是不是跑前登记 `research/prompts/e165-preregistration.md` 写的；入库产物逐字节复跑的前提（线程数无关、枚举次序确定）有没有漏洞。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | C1、C2、C3、C7 为主 | 逐处核代码做的是不是条款说的、改前改后是不是等价；能在冻结副本的拷贝上量的就量（例如改前改后对同一批输入逐个比结局） |
| 云端攻方（Opus） | C1–C8，重点 C1、C2、C4、C5、C6 | 用户判「这部分代码质量一般」：攻每一处会错、证明漏掉的、测试与变异证不到的、编码纪律的违例；造反例、造变异，在冻结副本的拷贝上量；Verus 不跑（主 agent 判决时现跑要核的那几条） |

第一轮，不派辩方。

## 五、跑前写死的判据

- 改前改后在某个输入上结局不同 ⇒ 打中 C1，写回代码（除非能指出原文那一判本身违反条款，那时走第二轮判决的题面改法）。
- 一条 `ensures` 或 spec 能被一个违反条款的实现满足 ⇒ 打中 C2，记「spec 太弱」，并进第 4 批补 spec。
- 一行变异或一条测试证不到它的名字说的那件事 ⇒ 打中 C5，补取样点或换变异。
- 编码纪律违例给出文件:行就算打中 C6。
- 腿提的新改法算零轮，标「零轮」。
- 不提前优化 `crates/` 里 core 的实现代码（用户 2026-10-01 定）：只攻正确性、证明、测试与编码纪律；为了快、省内存、少读盘去改 core 的改法不提，提了判决不采。改法本身要动 core 的（修一个错、补一条判），写成最小的那一改。

交付：腿的报告写 `research/prompts/m3-verus-land-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-verus-land-code-r1-<腿名>-model/`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的路径与行号。方案按 `crates/` 今天的实现来谈。不跑 checker 档与重型测试，不跑 Verus。

## 六、冻结副本与 diff

腿读 `/tmp/claude-1000/m3-verus-land-code-r1/tree/`（合入之后的 `crates/` 与仓根 `Cargo.toml`、`Cargo.lock`；sha256 清单 `research/prompts/m3-verus-land-code-r1-snapshot/crates-sha256.txt`，212 份）。这一批相对 HEAD fcc4d340 的 diff 在 `research/prompts/_m3-verus-land-code-r1-diff.md`（新 crate 与 E165 装置是整份新文件，在冻结副本里读）。改前的 `recovery.rs` 用 `git show fcc4d340:crates/singlefs-core/src/recovery.rs` 取。主 agent 合入后现跑：harness 档 97 个测试二进制全绿；新 crate 与 `singlefs-core` 的门禁 clippy 旗标与 fmt 退 0；E165 装置 clippy 三处红（第 188、782、893 行，排进第 4 批）。
