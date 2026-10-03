# 第 16 批代码轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers O1 O2 O3 K1 K2 K3 K4 U6 R2 -->

## 一、题面

第 16 批（`records/2026-10-01-里程碑三形式化接入.md#16`）合入了三份补丁，改动全文 `research/prompts/_m3-formal-code-b16-r1-diff.md`：

- 16-2（C673（三 oracle 合谋测试的补 oracle 改法没做） ④）：理想模型加卸载的答 `answer_unmount`，对拍加 `observed_unmount`，抬 F 那一串的循环抽成共用函数。
- 16-3（C673 ①、C670（无读故障的崩溃态上可写挂载见证判据为真没有判官））：`mount_transparency.rs` 新入口 `judge_writable_mount_refusal_of_a_crash_state`，崩溃态没注入读故障、拒因 N-配置、见证判据在崩溃镜像上复算仍为真 ⇒ 拒得不合法；`crash_injection.rs` 加 `StreamMarks::read_faults_in_a_crash_state` 并接到第二截。
- 16-4（C674（GPU 判器少了 CPU 判器有的最新根水位一格））：`crash_judge_checker.wgsl` 在 I-7.8 取 max 那一格后加最新那条根自己的水位一格。
- `crates/mutations.tsv` 追加 15 行。

这些改法来自 E171（三oracle合谋测试） 结论三轮判决（`research/prompts/e171-conclusion-r3-main-verification.md`），全部被攻过零轮。这一轮攻代码做的是不是条款说的、哪一格会错。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 合入验证：harness 档四个包 132 个测试二进制全绿，fmt、clippy（`--all-targets --all-features`）退 0；checker 档一样没跑（16-3 的 3 条内联用例与 4 条变异、16-4 的挂 `gpu` 用例与 1 条变异都只编过，归提交时）。
- 16-2 停下的一格：树表 0 条那一版上卸载，条款没写（D16（发布语义） 已定项 1「正常卸载」只写推一串空发布），实现一个字节都不写（`crates/singlefs-core/src/mount.rs` 里卸载那一段的注释），模型在这一格报「答不了」并有用例钉住。卸载记号（D22（单元原子性怎么合成） 已定项 7）没进对拍。
- 16-3 的读故障认定：崩溃窗口末尾之前，`CrashRecoveryAbandoningTheNewestRoot` 那一步往录制流里发过东西才算注入过读故障；撕裂系统配置那一支按「判据从哪一支变真」分开，仍走 U6 照甲判合法（U6 那一路的复算仍是甲，core 已是 R2，归 C331（择根倒挂压过已确认的写）、C664（挂载拒绝合法复算把两种拒因报成一个））。
- 16-4：GPU 判器那一格照抄 CPU 判器（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image` 末尾 I-7.8 两格）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　16-2 卸载的答对不对
模型答的卸载那一串（新 F 取现行那一版的 txg、推空发布直到每块盘都有一条、做完会话关掉）与 D16 已定项 1「正常卸载」一行和射程、已定项 7 卸载那一段逐句对得上吗；`observed_unmount` 交给对拍的是哪几条根、漏没漏；抽出的共用循环对抬 F 那条路径是不是逐字不变；树表 0 条那一格模型报「答不了」是不是唯一的读法。

### K2　16-3 判官会不会误判
新判官在合法的崩溃态上会不会判出「拒得不合法」（误红），在 E171 第 242、269 行那两类坏实现上会不会漏；读故障的认定（只认 `CrashRecoveryAbandoningTheNewestRoot` 那一步发过东西）漏没漏别的注入读故障的步；撕裂那一支分得干不干净（与 C664 的两种拒因、R2 跳过撕裂槽对得上吗）；接线（调用点把 `read_faults_in_a_crash_state` 交给新判官）错了有没有东西会红。

### K3　16-4 GPU 那一格与 CPU 是不是同判
`newest_root` 与 CPU 判器的「最新那条根」是不是同一条（择根序、平局、撕裂的根）；`highest_seen` 两边口径一样吗；判法摘要随内核原文变、复用的旧判定会不会被误用。

### K4　变异行与测试的判别力
15 行变异各自改的是不是它点名要证的那一处，点名的测试会不会红；第 242、269 行两条改指崩溃注入用例的行，在今天的实现上那条崩溃注入用例基线绿、变异下红，推得出吗（没跑，归提交时）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K4 | 逐句核代码做的是不是条款说的，给出每格的判定与引文 |
| 云端攻方（Opus） | K2、K4，兼 K1 | 攻哪一格会错：造误红的合法崩溃态、造漏判的坏实现、找认不出的读故障步；量得了的在副本上写小模型量，量不了的写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；打中而坐实 ⇒ 改法并进下一批实现员；分歧 ⇒ 开第二轮。
- checker 档没跑的那几格，腿的结论只能是「推的」，主 agent 判决里照实标。

交付：腿的报告写 `research/prompts/m3-formal-code-b16-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b16-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
