# m4-gpugc-floor-raise-precheck-code-r2：里程碑四七节 GPU 垃圾回收，抬 F 之前先算，代码轮第二轮正文（2026-11-10）

<!-- doc-lint:not-numbers T1 T2 -->

## 一、题面

判决 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-main-verification.md` 第三节第 1 到 3 条（第 122 批，补丁目录 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-fixes-writer/patch/`，实现员报告同目录 `../result.md`）已合入主工作区。第一轮判决写明第二轮只攻第 1 条：槽对那一判的上界加上这次挂载开过的聚簇段里没挡的槽对。

这是这一题的第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。第一轮已判的格（T1-1 上界算小、T1-2 越位、T1-3 第四处推串、T1-4 提交内生块、T3-3 短缺混着来）不再攻。

改动全文：`research/prompts/_m4-gpugc-floor-raise-precheck-code-r2-diff.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-snapshot/crates-sha256.txt`。
- 被判的产品代码：`crates/singlefs-core/src/mounted_session.rs`：`SlotsAFloorRaiseCouldRelease` 新加一项 `unblocked_slot_pairs_inside_the_cluster_segments`，`of_the_device` 多收 `allocator.cluster_segments()`，这一项用已有的 `unblocked_slot_pairs_outside_the_cluster_segments` 算两次相减（传空集得全部槽对，传这次挂载的聚簇段得段外槽对）；槽对那一支在原有几项前面加上这一截；式子那一判没动。另：`UserChangeRefused` 文档「八种」改「九种」。
- 量到的（实现员副本上）：clippy 两个包任务 11943 退 0；判法模块 11 条任务 11944 绿；验证包 11 个目标 332 条任务 11945 绿（上界变大之后没有用例改判）；证红任务 11950 抓到 1 条。合入之前那一版（第 119 批）的合入后验证任务 11805：这一路改到的 75 个目标退 0，其中真写满那一族 22 条。
- 实现员留给这一轮的：新那一项读成 0 时判法模块抓不到（没写变异行），要一条集成用例才红；攻方腿第一轮说过加这一截会推一串白写。
- 已知红，不归这一批：可写挂载回退相关两条（别的会话在修 `mount.rs`）。
- 压着的条款：`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 1（「根槽这一次读坏」「抬 F 的上限」两行）、已定项 10。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　第一轮的修补本身
加的那一截是不是「卸载重挂放得出」的那一截（再挂时分配器重建的口径与这一项算法同不同）；会不会多算（聚簇段里那些槽对卸载重挂也放不出的情形）；宁大勿小之后白推一串的代价，在什么形上出现、多大；被宽护卡住那条出路在这一形上走不走得到（推到上限、报宽护、列卸载重挂）。

### T2　产生结论的方法与从未被看过的地方
第一轮攻方的 T3-4 只读码、可达性没量（宽护要读坏的被抛弃根才造得出）：这一轮能不能造出来；新那一项读成 0 时没有用例红，判法模块的用例是不是只验了算术、没验接到真分配器上。

### 检查单：走不到的理由二次查实
实现员报告「走不到的理由」写「无」，新加的唯一一处 `.expect` 依赖的不变量在报告里写明（`~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-fixes-writer/result.md`）。腿追调用点判。

### 检查单：类型与名字
新名字：`unblocked_slot_pairs_inside_the_cluster_segments`。槽对与槽两种单位有没有共用裸整数。

### 检查单：不兼容旧代码与旧格式
本轮改动没有这一类（读补丁文件清单）。腿现查。

### 检查单：不新建判定库
本轮改动没有这一类。腿现查。

### 检查单：所有任务多机可用
新用例是判法小用例，不读活树。腿核。

### 检查单：测试优先多线程
第一轮判「不成立」的两条崩溃循环用例已补一行不切片并行的理由（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs`）。腿核。

### 检查单：带 --item 的命令与项
这一批没有新的崩溃点重放项；新变异行 `抬 F 先算：槽对那一判的上界不算聚簇段里没挡的槽对` 在 `crates/mutations.tsv`。腿核项在登记里（读，不跑）。

### 检查单：新崩溃流多罩了什么
本轮改动没有新崩溃流。腿判要不要。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1、T2 与检查单八格，T2 主攻 | 复核第一轮判决 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-main-verification.md`：T3-4 那一条采纳够不够得着、修补是不是打中了它、有没有别的修法；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1、T2 与检查单八格，T1 主攻 | 在冻结副本的拷贝上造反例：多算、白推、宽护出路走不到；能造就造读坏的被抛弃根；探针能复跑；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑（服务撞内存上限会重排同一条命令，草稿目录非空时先清再跑），探针的内存与时间写进报告；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
