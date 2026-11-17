# m4-gpugc-floor-raise-precheck-code-r3：里程碑四七节 GPU 垃圾回收，抬 F 之前先算，代码轮第三轮正文（2026-11-10）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

判决 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-main-verification.md` 第三节第 1 条：撤回第 122 批补丁里 `mounted_session.rs` 的聚簇段那一截（槽对那一判的上界不再加这次挂载开过的聚簇段里没挡的槽对），连同它的判法用例与变异行；文档「九种」、运行库两条崩溃循环用例上的不并行理由留着。主 agent 自己改的，已在主工作区。

这是这一题的第三轮，也是最后一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。只攻前两轮站住的形态（撤回之后的先算判法，就是第一轮被判的那一版加文档两处）；这一轮新冒出来的形态不再开轮，写进判决的交用户表。

改动全文：`research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-diff.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-snapshot/crates-sha256.txt`。
- 被判的产品代码：`crates/singlefs-core/src/mounted_session.rs`。先算判法在 `push_one_floor_raise_for_a_refusal_for_space` 开头（3 处调用共用：`accept`、`publish_user_change`、走空），上界：式子那一判是可用加 defer 加隔离；槽对那一判是段外槽对加每个放得开的槽各补一对；上界恰好等于需求照推；提交内生块那一判照推。
- 量到的：撤回之后判法模块 `mounted_session::everything_a_floor_raise_could_release_tests` 10 条任务 11966 绿；clippy 任务 11967（写正文时的结局见判决）。
- 第二轮读数（主 agent 复跑，整行在第二轮判决第一节）：攻方探针 372 个配置 `vain_series_by_the_inside_term=18105 vain_remount_lets_it_through=0 outlet_lost_after_a_series=0 outlet_lost_first_round=0 published_where_the_old_bound_fired=0 stuck_reports=7730`；辩方探针 `SUMMARY cells=200 same=200 lost_path_in_old=0 recovered_in_new=0 white_push_new=40`。
- 第一轮判决写进射程、不改的三形：先算不罩另发的搬迁批、不判提交内生块那一判、短缺混着来时推过一串才判出。
- 已知红，不归这一批：可写挂载回退相关两条（别的会话在修 `mount.rs`）。
- 压着的条款：`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 1（「根槽这一次读坏」「抬 F 的上限」两行）、已定项 10（射程那一段）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　「已定」项撑不撑得住
撤回是不是干净：撤回之后的先算与第一轮被判的那一版（第 119 批，`research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-diff.md`）逐处同不同，有没有残留（字段、参数、`expect`、用例里的字段）；D16 已定项 10 射程那一段（kb 文件自己的行号现查）与撤回之后的代码逐句对不对得上。

### T2　未定项清单的完整性
第二轮判决登的欠账（判出放不下时 C 那一圈罩着、卸载重挂写得进的格只有取样没撞上；攻方改法一零轮）全不全：先算还有哪一形会推白串或吃掉出路，前两轮没列；「defer 每槽算一对」的松（第二轮攻方 T1-4）要不要单列一行。

### T3　那一轮报出的数字能不能核
第二轮判决第一节两份探针的读数：探针数的是不是判决用它说的那件事（`vain_series_by_the_inside_term`、`outlet_lost_first_round`、`vain_remount_lets_it_through` 各自怎么算，对照模型源码），读成 0 的那几项是不是装置走不到（造不出那一形就数不出）；两份探针共用了哪些前提。

### 检查单：走不到的理由二次查实
撤回删掉了第 122 批唯一一处新 `.expect`；第 119 批那两处 `expect` 第一轮已追过。腿现查撤回之后还剩几处、调用点变没变。

### 检查单：类型与名字
撤回之后 `SlotsAFloorRaiseCouldRelease` 只剩槽数字段。腿核。

### 检查单：不兼容旧代码与旧格式
本轮改动没有这一类（只删）。腿现查。

### 检查单：不新建判定库
本轮改动没有这一类。腿现查。

### 检查单：所有任务多机可用
本轮改动没有新用例。腿现查。

### 检查单：测试优先多线程
本轮改动没有新用例。腿现查。

### 检查单：带 --item 的命令与项
删掉的变异行「抬 F 先算：槽对那一判的上界不算聚簇段里没挡的槽对」不在 `crates/mutations.tsv` 了；腿核没有别的登记还点名被删的用例 `an_unblocked_slot_pair_inside_the_cluster_segments_counts_toward_the_slot_pair_bound`（读，不跑）。

### 检查单：新崩溃流多罩了什么
本轮改动没有新崩溃流。腿判要不要。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1、T3 与检查单八格，T1 主攻 | 核「撤回之后的代码是不是第二轮判决第三节说的、与第一轮被判那一版逐处相同」；T3 对照两份探针源码核读数口径；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T3 与检查单八格，T2 主攻 | 找前两轮没列的先算缺口（白推一串、吃掉出路、误拒）；能造就在冻结副本的拷贝上造探针；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑（服务撞内存上限会重排同一条命令，草稿目录非空时先清再跑；第二轮攻方探针峰值约 7 GiB），探针的内存与时间写进报告；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
