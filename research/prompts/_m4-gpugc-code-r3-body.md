# m4-gpugc-code-r3：里程碑四七·3 GPU 垃圾回收，代码轮第三轮正文（2026-11-02）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

代码轮第二轮（判决 `research/prompts/m4-gpugc-code-r2-main-verification.md`）第三节九条与第五节、「四之一」用户定的几处，由第 59 批（`~/.local/share/singlefs/work/m4-gpugc-code-r2-fix-writer/patch/`）与第 60 批（`~/.local/share/singlefs/work/m4-gpugc-batch60-writer/patch/`）改完，都已打进主工作区。这一轮是最终轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核；只攻前两轮站住的形态与这两批的改法本身，这一轮里新冒出来的零轮形态不为它另开一轮，写进判决的交用户表、标「零轮」。前两轮攻过、这两批没改的角度不再攻（第一轮正文 `research/prompts/_m4-gpugc-code-r1-body.md` 第三节 T1–T6，第二轮正文 `research/prompts/_m4-gpugc-code-r2-body.md` 第三节 T1–T4）。

改动全文：`research/prompts/_m4-gpugc-code-r3-diff.md`（两份补丁里产品代码的块按打进主工作区的次序原样抄录，1674 行、8 个文件块；测试、变异表的块不在里面，整份补丁在各自的目录里）。第 60 批叠在第 59 批上，两批改同一份文件的，以冻结副本里的现状为准。

## 二、实现今天的样子（主 agent 的观测，2026-11-02）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r3/repo/`（此刻主工作区，含这两批补丁与别的会话这期间合入、没提交的改动；去掉了 `.git`、编译目录、私有配置 `multi-host.env` 与 `.env`）。被判的 7 份产品代码文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m4-gpugc-code-r3-snapshot/crates-sha256.txt`，冻结副本逐份对上。
- 被判的产品代码：`crates/singlefs-background-gpu/src/` 的 `background_gpu.rs`、`gpus_of_this_host.rs`、`waiting_for_the_gpu.rs`；`crates/singlefs-core/src/` 的 `background_compute.rs`、`mounted_session.rs`、`mount.rs`、`compaction_plan.rs`。`mount.rs` 与 `mounted_session.rs` 里不是这两批的块（别的会话暂存区里的改动）当上下文读、不攻；腿只攻 `_m4-gpugc-code-r3-diff.md` 里有的块。
- 冻结副本里的落点（第二轮判决第三节的条号在括注里）：
  - 切换校验只现枚举一次（第 1 条）：`background_gpu.rs` 里外壳的 `GpuInventory` 实现，`devices_not_opened_at_the_mount` 读挂载那一刻定下的那份。
  - 重交次数只数真交出去的（第 2 条）：`waiting_for_the_gpu.rs` 第 170 行 `note_a_wait_past_the_limit`、第 213 行 `resubmissions += 1`。
  - 排空待发被拒照样计叫起、带报警（第 3 条）：`mounted_session.rs` 第 5088 行 `PublishOfThePendingChangesRefused`、第 5116 行带报警字段的拒绝成员；「排空不成就不跑整理」照留（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs` 整理那道门的前提）。
  - 泄漏账按次累加（第 4 条）：`gpus_of_this_host.rs` 第 55–94 行 `bytes_leaked_by_earlier_mounts` 一带。
  - 回读暂存挪进显存不足的错误范围（第 5 条，第 60 批第 1 条换方法造了会红的测试）：`background_gpu.rs` 第 547 行 `trait DeviceCreatingTheBuffersOfAChunk`、第 1025 行 `buffers_created_within_the_free_memory`、第 1051 行 `dispatch_both_kernels_and_read_back`。
  - 停着表由 `enum_listing_every_member!` 生成（第 7 条）：`background_compute.rs` 第 38 行起。
  - 叫起计数（第 9 条，用户 2026-11-01 定「改代码」：出错停下那一趟不算，下一次叫起是 1）：`background_compute.rs` 第 1423 行 `begin_a_wake`、第 1440 行 `alarm_at_the_end_of_a_wake`；`mount.rs` 第 9019 行只供测试的挂载入口（C921（只供测试的开关在发布构建里够得着） 已登记）。
  - `compaction_plan.rs` 只改了一条测试闭包的参数名（clippy）。
- 验证段的数（量的，`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 59、60 批各行）：合入后验证「✓ 测试全绿（335 个测试二进制，共 3229 条测试，其中慢档 77 条；跑了 3 个包…」、fmt、build 过；三个包的 clippy 绿；证红第 59 批 18 行、第 60 批 9 行加第 5265 行全部抓到；形式化轻型三格绿；崩溃轻型 53 项 0 红；变异轻型两批点名的 16 项全部抓到。真卡用例（每张卡 1 GiB；一张、两张、三张卡；没有卡拒挂；挂着出错报警加 CPU 补跑）只写没跑，延到七·3 收口（用户 2026-11-01 定）。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13 八条（第 245–252 行，含 2026-11-01 补的两句）与它的射程段；`.claude/kb/milestone/04-fourth-txn.md` 七·3 那一行（第 149 行，含真卡测法那一句）；`.claude/kb/mount-parameters.md`；`.claude/kb/checks-owed.md` C921（只供测试的开关在发布构建里够得着） 那一行；`.claude/rules/fs-design.md`「五条硬要求」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　「已定」项撑不撑得住
D26（后台整理与放置回收） 已定项 13 第 2–4、8 条（第 246–248、252 行）在两批改过之后的代码上逐句对：每一句条款，代码里哪一行做它、哪一条测试或变异行钉它；有没有条款写了、代码今天做不到或没有测试钉的句子（尤其 2026-11-01 补的两句：叫起计数不算出错停下那一趟、重切时已做完的片不留）；第二轮判决打中的九条改完之后，有没有哪一条把别的已定句子改坏了（例：只现枚举一次之后，挂着时卡掉了还能不能按第 4 条拒切换；泄漏按次累加之后，第 3 条「上限不超过那张卡的显存总量」怎么扣）。

### T2　未定项清单的完整性
两条腿与主 agent 前两轮判决里写成「读法」「照收」「交用户」「零轮」「核不动」「只抽了一次样」的每一条，列一张清单：哪些已有去向（用户定、kb 写回、欠账行号），哪些没有去向；七·3 那一行验收里还有哪一句没有实现或没有测试（不算延到收口的真卡用例）；`.claude/kb/checks-owed.md` 里点到七·3 或后台 GPU 的欠账行，哪些该在七·3 收口前关、哪些可以留到七·4、七·5。

### T3　前两轮报出的数字能不能核
前两轮判决与这两批报告里写进结论的数（探针原行、证红计数、测试条数、变异行号、泄漏字节、片数），逐个问：产生它的命令与产物还在不在、能不能照原样重跑出同一个数；只在腿的副本上量过、主 agent 没复跑的有哪些；第 60 批第 1 条「回读暂存」那条测试用替身设备判先后次序，它判到的是不是真卡上 wgpu 错误范围的行为（替身照什么规矩记错）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T3，T1 主攻 | 核「代码今天做的是不是已定项 13 每一句说的那件事」：逐句列条款行号、代码文件:行号、钉它的测试或变异行名；T2 的清单逐条写去向；T3 逐个数写出处与复跑命令 |
| 云端攻方（Opus） | T1–T3，T1、T3 主攻 | 在冻结副本的拷贝上造反例：两批改法把别的已定句子改坏的输入；条款写了而代码做不到的时序；前两轮报出的数里重跑不出同一个数的那一个；替身判先后与真卡行为不同的那一种 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；引补丁块写 `_m4-gpugc-code-r3-diff.md` 那一节的补丁目录名，不写那份文件的行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改。读码推出来、没量过的写「推的，没量过」。
- 不起 GPU：真卡用例（特性 `tests-on-a-real-gpu`）不写进命令；不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令），不跑 checker 档测试。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
