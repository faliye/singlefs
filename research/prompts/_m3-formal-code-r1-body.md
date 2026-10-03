# 形式化验证第 7–10 批合入的代码：代码轮第一轮正文（2026-10-02）

<!-- doc-lint:not-numbers M1 M2 M3 M4 O3 R3 H08 H09 C1 C2 C3 C4 C5 C6 C7 XE1 XE2 XE3 J6 J7 Qn -->

## 一、题面

形式化验证这一线第 7 批与第 10 批合入 `crates/` 的改动（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 7、8、10 批）走一轮代码三方（`.claude/rules/implementation-workflow.md` 第 2 步）：代码做的是不是条款说的、哪一格会错。上一轮代码三方是 m3-verus-land-code-r3（核接回 `crates/` 那三轮，已判），这一轮只判它之后的改动。

被判的改动全文在 `research/prompts/_m3-formal-code-r1-diff.md`（9 753 行）：

| # | 改动 | 出处 | 压着的条款 |
|---|---|---|---|
| C1 | 变异行与用例（R2-4 I-8.3 那一判进 `crates/mutations.tsv`、R3-9 三处 `verify_named_units`、R3-10 `scan_journal` 先到先得、R3-1 重建根卸载记号与其余字段、C648 可写挂载读阶段读盘次数、H08 / H09 两种形状），产品代码不改 | 7-1b 补丁，报告 `/tmp/claude-1000/m3-formal-7-1/report.md` | D13（验证路线） 已定项 1 M2；D23（journal 的角色与格式） 已定项 14；I-8.3（重放前缀严格连续） |
| C2 | M3 幂等两读（记录级、不动点）与单调在核 `replay_core` 上证出，可执行代码不改；必红表加 39 行 | 7-2b 补丁，报告 `/tmp/claude-1000/m3-formal-7-2/report.md` | D13（验证路线） 已定项 1 M3；I-8.4（重放幂等）；I-8.12（同一实例里 journal 记录的 txg 随计数器不减），还没登进 `invariants.md` |
| C3 | 择根抽成核 `choose_root_slot`、扫环建表抽成核 `first_copy_of_each_record_key`（先查后插）、合判推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；`recovery.rs` 的 `choose_root` 与 `scan_journal` 改成胶水；扫环胶水先把读到的每一份收进 Vec 再建表；两个键完全相同的槽取前面那个（照原行为，条款没写） | 10-3 补丁，报告 `/tmp/claude-1000/m3-formal-10-3/report.md` | D22（单元原子性怎么合成） 已定项 7、已定项 24；D13（验证路线） 已定项 1 M2、M3、M4；C650（择根与重放没有合判） |
| C4 | E165（恢复重放有界穷举与变异杀伤界） 装置：r2 档（生成器根字段随版本变、J6Qn、J7、Z5）、多盘一族、扫盘建表与反盘序、读实现结局的启动自检挪进单测 | 装置 diff（底：m3-verus-land-code-r3 冻结副本），登记 `research/prompts/e165-r2-prereg.md` 修订 1–4 | 同 C2；E165 页 |
| C5 | E170（O3规约执行器第一版） 导出装置（新文件）：两条层 0 流导出、`bad-states` 模式 | 文件全文，登记 `research/prompts/e170-preregistration.md` | D13（验证路线） 已定项 6 |
| C6 | `crates/mutations.tsv` 追加：E165 r2 装置 23 行（第 1634–1644、1657–1659、1668–1673、1813–1818 行）、E170 XE1–XE3 3 行，与 C1、C3 的行 | 各段报告 | `.claude/rules/mutation-sampling.md` |
| C7 | `crates/singlefs-replay-prefix/verus-mutations.tsv` 整表 153 行；`.claude/gate.d/verus-pins.tsv` verified 数 56 | 主 agent 逐行核 `/tmp/claude-1000/m3-verus-rows-10-3/rows.out`（153 / 153 对上） | `.claude/rules/verification.md`「门禁管哪一半」形式化验证那一行 |

## 二、实现今天的样子（主 agent 的观测，2026-10-02）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-r1/tree/`，清单 `research/prompts/m3-formal-code-r1-snapshot/crates-sha256.txt`。`crates/singlefs-replay-prefix/src/lib.rs` 4 281 行（Verus `--rlimit 10`「56 verified, 0 errors」），`crates/singlefs-core/src/recovery.rs` 4 635 行。
- 合入验证：`merge-and-verify.sh` 打上 10-3 补丁之后 harness 档 110 个测试二进制、fmt、clippy 全绿；checker 档（E165、E170 装置单测、变异整表）归提交时，没跑。
- E165 第 2 次重跑四段都够判（实验页「第 2 次重跑」一节）；E170 三段都够判（实验页）。这一轮判的是代码，不是实验结论。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　C3 胶水与核对不对得上
`choose_root` 与 `scan_journal` 改成胶水之后，行为与改前逐条相同吗（读盘次数与次序、错误怎么交、自证失败的槽怎么跳）；Vec 收齐再建表的内存形状（journal 环第一版 1024 槽，最坏多少）；两个键完全相同的槽取前面那个——条款没写，这一处写死在核里还是胶水里，改了谁会红；「胶水拿所选槽的那条根去调重放」那一步不在 Verus 里，谁罩。

### K2　C2、C3 的证明在证什么
M3 不动点的前件（I-8.12、P0–P5）与条款字面（D13 已定项 1 M3 一行）是不是同一句；合判推论的前提「只有两个核的 ensures」——核的 ensures 本身有没有弱到让推论空真的地方（空真格只在 `requires` 函数体头插 `assert(false)`）；必红表 99、125 两行期望 2 → 3 的理由（新 spec 进上下文、择根核共用 S5 序）站不站得住。

### K3　C1、C6 的变异行与用例
每条新加的变异行改的是不是条款说的那一处、点名的用例真能红；7-1b 报 C648 上 S03、S07 原样在健康镜像上不红（读缓存命中），补了把上一条记录在 0 号盘写成全零的用例——这算补敏感取样点还是换了一个场景；`crates/mutations.tsv` 第 173、174、177 行改前就 0 命中（不归这一轮，只点名）。

### K4　C4、C5 装置
E165 装置读实现结局的启动自检挪进单测之后，变体下还有没有别的启动路径会读实现；多盘一族的「在」改成任一份在即在、直接建表按设备号升序读——与实现 `scan_journal` 按 `device_identities()` 次序读不同（第四段量到反盘序时 R 域 3 482 处表不同），装置的哪一栏代表实现；E170 导出装置的 `bad-states` 补丁「补丁不算写」与记录核对器把补丁当后写看（E170 第三段报告偏离第 1 条），会不会让 Q3-f 那一格的数误导。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K4 | 核「代码做的是不是条款说的」：每一处改动对到压着的条款原句，给出对上 / 对不上与依据行号 |
| 云端攻方（Opus） | K1–K4，重点 K1、K2 | 攻「哪一格会错」：在 `crates/` 的拷贝上造反例（胶水与核行为不同的输入、推论空真的改法、变异行改了却红在别处），量过的写数 |

## 五、跑前写死的判据

- 一处改动两腿都判对上 ⇒ 站住。
- 一条腿造出反例、主 agent 现查坐实 ⇒ 打中，改法进下一批代码，改完再攻一轮。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-formal-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
