# 形式化验证第 11 批合入的代码：代码轮第二轮正文（2026-10-02）

<!-- doc-lint:not-numbers M1 M2 M3 M4 O1 O3 K1 K2 K3 K4 L1 L2 L3 L4 L5 F1 F2 F3 F4 F5 F6 G1 G2 H1 U1 U5 U6 V1 V2 V3 W1 Q KJ0 KJ1 KJ2 KJ3 KA -->

## 一、题面

形式化验证这一线第 11 批合入 `crates/` 的改动（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 11 批）走代码轮第二轮：第一轮（`research/prompts/m3-formal-code-r1-main-verification.md`）打中的改法落地了没有、落得对不对，加上这一批新写的代码做的是不是条款说的、哪一格会错。

被判的改动全文在 `research/prompts/_m3-formal-code-r2-diff.md`：

| # | 改动 | 出处 | 压着的条款（零轮的标出） |
|---|---|---|---|
| L1 | RefFS 新包 `crates/singlefs-reference-filesystem`（只依赖 std，18 条规则各一单测一变异） | 11-3，报告 `/tmp/claude-1000/m3-formal-11-3/report.md` | r4 / r5 / r6 三份判决第四节与「用户定案」（F1 Q、F3 另起、F4 单独一个包、G2、H1，零轮）；D13（验证路线） 已定项 3；D16（发布语义） 已定项 4、6、7、12、17 |
| L2 | 挂载透明判法（harness 新模块 `mount_transparency.rs`）与崩溃注入接入三处、拒绝合法按 D23 已定项 14 在镜像上复算、O1 判据 ③ 两行变异 | 11-4，报告 `/tmp/claude-1000/m3-formal-11-4/report.md` | r5 判决第二节 U5、U6，r6 判决 U5、U6 两行（零轮）；D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14 |
| L3 | C651：checker ③ 逐档、GPU 判器 checker 那一份同改、mkfs 每盘先把 [0, 1 MiB) 整段写 0、io_min 8192 两张坏镜像与一条层 0 流、harness 钉值 | 11-5，报告 `/tmp/claude-1000/m3-formal-11-5/report.md` | `research/prompts/m3-c651-r1-main-verification.md` 第四节、用户定案 W1 与「改成逐档」（推翻 2026-09-14 的甲）、r6 判决 V1 两个出口（零轮）；D22（单元原子性怎么合成） 已定项 8、16 |
| L4 | 代码轮第一轮的改法：攻方用例、胶水对调与胶水取错根三处变异、`newest_root_among` 平局取前一个、R3-1 三处拷贝的变异；合判推论去掉 J1 前提那一行不加（J1 由 M2 推得出） | 11-10、11-11，报告 `/tmp/claude-1000/m3-formal-11-10/report.md`、`/tmp/claude-1000/m3-formal-11-11/report.md` | 代码轮第一轮判决第三节与「改法落地」 |
| L5 | E165 装置 r3 档：对抗一族按 I-8.12 拆行（第一段）、择根合判族 `--family root_choice`（第二段） | 登记 `research/prompts/e165-r3-prereg.md` 修订 1、2 | 问题单 `research/prompts/e165-r3-questions.md`；C650（择根与重放没有合判） |

## 二、实现今天的样子（主 agent 的观测，2026-10-02）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-r2/tree/`，清单 `research/prompts/m3-formal-code-r2-snapshot/crates-sha256.txt`。
- 合入验证：harness 档 115 个测试二进制全绿、fmt 过；全工作区 clippy 红在别的会话暂存的 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 第 181、182 行（不归这一线）；checker 档（崩溃注入、层 0、E165 装置单测、变异整表）归提交时，没跑。
- RefFS 还没接调用点：G1 合取（调用点 = RefFS ∧ O1 全四条）、H1 的翻译表与崩溃时刻归属在下一批写。这一轮判 L1 包本身。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　RefFS 包（L1）
18 条规则各对上哪一句条款；条款没写处它返回错误成员不动模型（实现员报告第五节七条）——这样做会不会让调用点接上之后把合法历史判成「不支持」而漏判；H1 定的「发布调用、fsync、回退都抬 lo、回退算一次已确认发布」与包今天「只有 fsync 抬 lo」不同，接调用点之前包要改哪几处。

### K2　挂载透明（L2）
十样逐样判与 r5 / r6 判决的写法一不一致；「推的」几样（树 ID 水位、两根指针、txg 下界、扫描报告）哪一样在合法历史里会先误红（实现员推树 ID 水位，`crates/singlefs-core/src/recovery.rs` 第 1322 行一带取根环与记录的 max）；拒绝合法的镜像复算与 D23 已定项 14 的单子是不是同一句；二次崩溃三个相位的字段一半取法。

### K3　C651（L3）
checker 逐档与实现逐档在每一种输入上是不是同一个结局（越界槽整池拒、incompat 不认得的槽不核偏移、平局世代号最大同号偏移小）；mkfs 写零插在什么位置、录制流多出的步对层 0 与崩溃放量钉值的影响（实现员只改了 harness 两份，checker 档推「从 mkfs 之后起枚举、不变」）；GPU 判器 `crash_judge_checker.wgsl` 改法与 CPU 一致。

### K4　第一轮改法与装置（L4、L5）
第一轮打中的几格落地后真的会红；现有用例分不开 ② 借槽距与 ③ 逐档（11-11 推的，第三类）；E165 r3 两段装置的新族与拆行有没有让旧档输出变（执行员报 profile-check 逐字节同）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1–K4 | 复核第一轮代码轮判决：打中的改法落地后够不够、有没有改过头；替被改掉的旧写法辩护（`newest_root_among` 取后一个、checker ③ 取 4096、合判推论不加 J1 那一行） |
| 云端攻方（Opus） | K1–K4，重点 K2、K3 | 在 `crates/` 的拷贝上造反例：挂载透明在合法历史上误红、checker 与实现逐档结局不同的输入、mkfs 写零之后钉值漂移；不重复第一轮攻过的角度 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；这是代码轮第二轮，第三轮只攻这一轮站住的与改法落地。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-formal-code-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-r2-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
