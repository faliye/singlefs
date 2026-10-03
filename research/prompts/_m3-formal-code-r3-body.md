# 形式化验证第 12、13 批合入的代码：代码轮第三轮正文（2026-10-03）

<!-- doc-lint:not-numbers M1 M2 M3 M4 O1 O2 O3 K1 K2 K3 K4 K5 L1 L2 L3 L4 L5 L6 H1 G1 G2 U1 U2 U5 U6 F6 V1 KF KF3 KF9 KFW KG KA KJ1 KJ1T R2 -->

## 一、题面

形式化验证这一线第 12、13 批合入 `crates/` 的改动走代码轮第三轮（最终轮）：第二轮判决（`research/prompts/m3-formal-code-r2-main-verification.md`）打中的改法落了没有、落得对不对，加上第 12 批 RefFS 接调用点与三个实验装置新写的代码做的是不是条款说的、哪一格会错。第三轮只攻前两轮站住的形态与这两批新落地的改法；这一轮新冒出的零轮形态不再开轮，写进判决的交用户表。

被判的改动全文在 `research/prompts/_m3-formal-code-r3-diff.md`（基是第二轮冻结副本）：

| # | 改动 | 出处 | 压着的条款（零轮的标出） |
|---|---|---|---|
| L1 | RefFS 照 H1 改（发布调用、fsync、回退都抬 lo，回退算已确认发布，原样重发结局，m 改版本号覆盖全部写根入口与恢复读出）；harness 翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs`（操作序列、崩溃时刻按切段屏障归属、只给 RefFS）；崩溃注入两截与故障注入重开的调用点判 RefFS ∧ O1（任一红即红） | 12-2，报告 `/tmp/claude-1000/m3-formal-12-2/report.md` | `research/prompts/m3-formal-r6-main-verification.md` 用户定案 H1；`research/prompts/m3-formal-r5-main-verification.md` 用户定案 G1、G2；D13（验证路线） 已定项 3 |
| L2 | 故障注入六档 RefFS 一律弃权（`TheStateCarriesAnInjectedFault`） | 13-4，报告 `/tmp/claude-1000/m3-formal-13-4/report.md` | U1 射程句「带故障的状态不适用」；主 agent 已判「注入点没撞上的历史不带故障，该照判」，改法排第 14 批（不必再攻这一句） |
| L3 | 二次崩溃的 F：抬 F 的写落没落盘定判不判（界取挂载交回的第一串抬到的 F 那一次系统配置槽写，按持久集合判），暖机那几态判 F 不变；相位摆点仍三段 | 13-4 | 第二轮判决 K2-c 一行与第三节第 4 条（零轮）；r5 判决 U5 前半句 |
| L4 | checker 逐档读到可用槽先按不收之值表判（含 I-8.1 三格），当场交回、不再核偏移比世代号（`image.rs` 与 GPU 判器 `crash_judge_checker.wgsl`）；checker 择根键相同取读根环次序里最前（`walk.rs`）；借槽距用例加问 checker | 13-3，报告 `/tmp/claude-1000/m3-formal-13-3/report.md` | 第二轮判决第三节第 1、2、3 条（零轮）；D22（单元原子性怎么合成） 已定项 16、24 |
| L5 | E165 装置 `--profile r4 --family root_choice_faults`（带故障的合法历史一族 KF、KF3、KF9、KFW，拿掉保护的对照 KG，KJ1T 判据，闸取今天实现的取甲） | 登记 `research/prompts/e165-r4-prereg.md`（修订 1–3） | 问题单 `research/prompts/e165-r4-questions.md`；C331（择根倒挂压过已确认的写） 改取 R2 还没进代码（C662） |
| L6 | E170 导出装置 `bad-states-second-run` 模式与 P10 一行；E171 合谋测试装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`（坏实现表逐行打变异、O1 与 O2 两列、关一条 oracle 的开关） | E170 第 1 次重跑第三段、E171 第一段报告 `/tmp/claude-1000/e171-segment1/report.md` | 登记 `research/prompts/e170-r1-prereg.md`、`research/prompts/e171-preregistration.md`；D13（验证路线） 已定项 6 |

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-r3/tree/`，清单 `research/prompts/m3-formal-code-r3-snapshot/crates-sha256.txt`（244 份）。
- 合入验证（本机，7G、4 线程）：harness 档 117 个测试二进制全绿、fmt 过；clippy 第一趟红在 `e165_replay_bounded_enumeration.rs` 第 15389 行一条常量断言（主 agent 包进 `const { }`），复跑 `cargo clippy --keep-going --all-targets --all-features -- -D warnings` 退 0。checker 档（崩溃注入、层 0、三个装置的单测、变异整表）归提交时，没跑。
- `crates/mutations.tsv` 第 1898、1903 行（E165 r3 第二段）锚点命中 2 次（r4 新族抄了旧函数），排第 14 批；E168 那八行锚点红是别的会话的。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　RefFS 接调用点（L1、L2）
H1 六条（a 到 f）与 U2 在包里各落在哪、用例钉住没有；翻译层「崩溃时刻按切段屏障归属」在每一种步（发布、覆盖写、零单元发布、回退、挂载、抬 F、冷启动恢复）上是不是只有一种读法；RefFS ∧ O1 合起来之后，合法历史上 RefFS 一侧会不会先误红（判不了的该弃权却投了红票）、坏实现上会不会漏（该判的却弃权）；故障注入一律弃权之后 O1 一票撑不撑得住。

### K2　二次崩溃 F 的界（L3）
界按内容认（挂载交回的抬 F 值在写表里第一次出现）在合法历史上会不会认错位置；按持久集合判比「抬 F 第一个写起」细的那一格在撕裂第三态上是不是同一个结局；摆点三段不变、推了抬 F 的挂载上暖机只摆一点，这一缺口多大。

### K3　checker 逐档与择根（L4）
逐档先判值之后，checker 与实现在第二轮攻方那 13 种手放槽上还剩哪几种结局不同（R、S 越界与 incompat 加不收之值那几种已记欠账，不重攻）；同一种坏槽在普通读法报 I-8.1、逐档读法报 I-7.13 会不会让 I-7.13 的计数或依据它的判法出错；`walk.rs` 新 fold 与 `choose_root` 在所有入参次序上同值；GPU 判器那一份与 CPU 同改。

### K4　三个实验装置（L5、L6）
E165 r4 生成器造的「带故障的合法历史」是不是今天写路径真写得出（闸、取号、系统配置轮换与 `crates/singlefs-core/src/mount.rs`、`transaction.rs` 对不对得上），KG 拿掉的两道保护是不是条款里那两道；E170 新模式与旧模式输出不变；E171 装置逐行打变异时基线、阳性对照、关一条开关会不会让某一列静默不判（判成「没红」）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1–K4 | 复核第二轮判决：打中的改法落地后够不够、有没有改过头；替被改掉的旧写法辩护（checker 逐档先核偏移、故障注入里读故障档 RefFS 照判、F 不判的相位整段算） |
| 云端攻方（Opus） | K1–K4，重点 K1、K4 | 在 `crates/` 的拷贝上造反例：RefFS ∧ O1 在合法历史上误红、翻译层归属错、E165 r4 生成器造出写路径写不出的历史、E171 某一列静默不判；不重复前两轮攻过的角度（挂载透明十样误红、checker 与实现逐档 13 种槽、mkfs 写零钉值、胶水差分、择新键摊入） |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；这是第三轮（最终轮），不再开第四轮，这一轮新冒出的零轮形态写进判决的交用户表。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-formal-code-r3-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-r3-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。代码引文写冻结副本的路径与行号。
