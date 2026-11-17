# m4-gpugc-precheck-b-a-code-r2：里程碑四七节 GPU 垃圾回收，抬 F 先算改法乙、甲，代码轮第二轮正文（2026-11-12）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

第一轮判决 `research/prompts/m4-gpugc-precheck-b-a-code-r1-main-verification.md` 第三节「改法」的第 1、2、4、5 条已写进主工作区（第 128、129 两批补丁）：

1. 第 1 条：`mounted_session.rs` 两处文档不再写「推了这次写也是被合取 7 拒」，改成推串里的空发布会换下固定点单元、可能把全空段数推高，判的是写之前那一刻的数。只改注释。
2. 第 2 条：`publish_user_change` 那一处与走空那一处各传哪一种，各加一行变异与一条真池用例。
3. 第 4 条：槽对数立 newtype `SlotPairCount`，与槽数分开。
4. 第 5 条：两条 600 步伪随机历史用例上各写一行不并行的理由。

第 3 条（kb 射程补写）等三轮判完由写回员做；第 6 条（先算拒绝时带不带卸载重挂那条出路）标着被攻过零次、不改代码，交用户。

这是这一题的第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。这一轮派辩方腿复核第一轮判决。不重复第一轮已攻过的角度（乙的增量计数各路跟没跟上、改 size 与建快照不先判、先乙后甲的次序）。

改动全文：`research/prompts/_m4-gpugc-precheck-b-a-code-r2-diff.md`。被判的产品代码两份：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mounted_session.rs`；`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs` 是测试代码，给腿读、不被判。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-precheck-b-a-code-r2-snapshot/crates-sha256.txt`，冻结副本 `sha256sum -c` 两份 OK。
- 冻结副本行号：`allocator.rs:356` `pub struct SlotPairCount(pub u64)`；`allocator.rs:511` `pairs_inside_the_cluster_segments`；`allocator.rs:800` `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments` 返回 `SlotPairCount`；`allocator.rs:779` `unblocked_slot_pairs_outside_the_cluster_segments` 的返回值留在 `u64`（实现员停下交主 agent：换它要动 `admission.rs` 的公开字段 `PlacementRoomOnOneDevice::slot_pairs_for_data_units` 与实验装置 E180、E181 两份源码；主 agent 定这一批不换）；`allocator.rs:4033`、`:4053` 两行不并行的理由。`mounted_session.rs:6056` `slot_pairs_once_everything_is_released: SlotPairCount`；`mounted_session.rs:6075` `most_a_floor_raise_could_make_available` 只在乘一个数据单元的槽数那一处取 `.0`；`mounted_session.rs:2116` `refusal_is_short_of_space` 与 `:6177` 甲那一判的文档已改。
- 新用例（冻结副本 `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs`）：第 1493 行 `a_publish_of_a_write_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series`、第 1546 行 `a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for`。
- 量到的：第 128 批证红 13955「跑了 2 条，抓到 2 条」，主工作区复判 14105「计数：抓到 2 条、没红 0 条、无效 0 条、锚点失效 0 行」；第 129 批改写的 7 行变异证红 14167「抓到 1 条」、14187「跑了 6 条，抓到 6 条」。合入后验证 14552：「测试全绿（2 个测试二进制，共 13 条测试；……按条复用：现判 13、复用 440、拒算 0）」、fmt 过、build 过；clippy default 绿，`--all-features` 红在别的会话的新包 `crates/singlefs-background-gpu-tests`（不归这一批），这一批改到的两个包单独的全特性 clippy 是任务 14703（写正文时在排，结局见判决）。
- 第一轮攻方探针（主 agent 复跑，任务 13926，整行在第一轮判决第一节第 2 条）：`fired_by_slot_pairs=1907 misrefused_slot_pairs=0 base_published_after_a_series=19351 fired_by_conjunct_before_any_series=4 misrefused_conjunct_before_any_series=0 remount_published_slot_pairs=12 remount_published_conjunct_before_any_series=2 series_product=31219 series_product_raised=24 series_product_crossed_the_floor=0`。
- 崩溃点重放带 `--item`：只有缩到展开深度 4 的那一趟（任务 13913，三项判器红数 0，不算门禁通过）；不缩域的留给提交时。
- 压着的条款：`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 10「射程」那一段（第 276 行，还没补写乙、甲）；欠账 `.claude/kb/checks-owed.md` C1002（抬 F 先算对槽对那一判挡得少）、C1003（被宽护卡住那条出路没量过）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　第一轮的修补本身
两处文档改完之后与代码、与任务 13926 的读数逐句对不对得上，有没有把话说过头（「可能把全空段数推高」是量到的 24/31219，「越不越过 `N₀` 不另估」是代码不估，不是量到越不过）。`SlotPairCount` 换得干不干净：有没有该换没换的槽对数（`unblocked_slot_pairs_outside_the_cluster_segments` 留 `u64` 是主 agent 定的，腿判这一处留着会不会让两个槽对数一个带类型一个不带、在哪一处被混着比或混着加）；`.0` 取在哪几处、取出来之后有没有与槽数相加相比。两条真池用例钉的是不是它们名字说的那一形：走空那条的期望 `NoSpaceAfterRaisingTheFloor` 是实现员推的、跑出来是绿的，腿核它红的时候红的是不是「走空那一处传成前台写入」这一件，别的改坏会不会也让它红或让它照样绿。

### T2　产生结论的方法
第一轮判决靠任务 13926 的探针下了四个结论（乙误拒这一份取样没撞上、甲的前提不成立、甲误拒分母太小、先拒之后卸载重挂写得进）。辩方腿复核：探针数的是不是判决用它说的那件事（`misrefused_*`、`remount_published_*`、`series_*_raised`、`series_*_crossed_the_floor` 各怎么算，对照模型源码 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/`）；读成 0 的那几项是不是装置走不到（造不出那一形就数不出，分母是多少）；O 那一臂是不是一个站得住的判别子（它关掉先算照推，推的上限与产品相同吗）；「打中：拒绝报里少一条卸载重挂出路」这一条判得够不够得着——现行（乙、甲之前）上界同一形丢不丢，这一条是乙、甲带来的还是原来就有。

### T3　从未被看过的地方
第一轮没看的：`accept` 那一处传的 `change_of_the_refused_batch` 在一批里并了几样改动时取的是哪一样、分类会不会因此错；甲判出时 `devices_short` 为空，读这个字段的调用方（挂载运行库、测试帮手 `assert_refused_with_nothing_pushed` 一类）拿到空表会不会走错支；`SlotPairCount` 的 `Sum` 与算术有没有溢出或下溢的新路；`DeviceFreeMap` 克隆时新位图与计数是不是一起克隆、重建分配器（重开、管理员回退）时这三样是不是从记录重算而不是沿用旧值。

### 检查单：走不到的理由二次查实
第 128、129 两批实现员报告「走不到的理由」一节原样：第 128 批「本轮没有改产品代码（`crates/<包>/src/` 一行没动）……」（它实际改了 `mounted_session.rs` 的注释与一条库单测的 panic 文字，没加断言）；第 129 批报告写补丁里没有新加的 `assert!`、`unreachable!`、`todo!`，那一节是空的。腿在改动全文里现查这两批加号行里有没有新的 `assert!`、`expect`、`unreachable!`、减法。

### 检查单：类型与名字
新加：`SlotPairCount`（`pub` 字段、`Sum`）；两条用例名。腿核 `SlotPairCount` 的公开字段该不该私有、`unblocked_slot_pairs_outside_the_cluster_segments` 留 `u64` 算不算「同一个量不同表示」，新名字有没有缩写。

### 检查单：不兼容旧代码与旧格式
本轮改动没有这一类：没碰盘上字节、没加版本分支（查过 `_-diff.md` 两份补丁全文）。腿现查。

### 检查单：不新建判定库
本轮改动没有这一类（查过同一份全文）。腿现查。

### 检查单：所有任务多机可用
两条新用例在内存池上现造，不读本机状态（第 128 批报告「拒算的测试」一节；拒算清单任务 13959 里没有它们）。腿现查。

### 检查单：测试优先多线程
两条伪随机历史用例上的理由已写（冻结副本 `allocator.rs:4033`、`:4053`）；两条新真池用例各是单场景。腿判理由成不成立。

### 检查单：带 --item 的命令与项
这两批没有新的崩溃点重放项与形式化项；变异行 9 行（第 128 批 2 行、第 129 批改写 7 行）都点名过。腿核改写的 7 行旧名在 `crates/mutations.tsv` 里是不是都删了、没有别的登记还点名旧名。

### 检查单：新崩溃流多罩了什么
本轮改动没有新崩溃流。腿判要不要。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1–T3 与检查单八格，T2 主攻 | 复核第一轮判决：四个结论各自够不够得着、探针读数的口径对不对、「打中」的两条是不是同样打中乙、甲之前的现行形态；替被判「打中」的甲文档与拒绝报辩护，辩不住的照实写；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T3 与检查单八格，T1、T3 主攻 | 攻第一轮的修补本身与没看过的地方；能造就在冻结副本的拷贝上造探针；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑（服务撞内存上限会重排同一条命令，草稿目录非空时先清再跑），探针的内存与时间写进报告；长跑的探针每个配置往标准错误打一行进度（服务看日志 10 分钟不涨判卡住）；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
