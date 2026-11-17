# m4-gpugc-mount-runtime-code-r1：攻方腿满盘探针修补（2026-11-09）

承接 `research/prompts/m4-gpugc-mount-runtime-code-r1-opus-output.md`。只修 `attack_full_pool` 的建池。**这条腿没编、没跑；探针只过了 `rustfmt`（语法），可能编不过，下面的挂钟全是推的。**

复跑命令见「要主 agent 跑的命令」。模型目录 `research/prompts/m4-gpugc-mount-runtime-code-r1-opus-s2-model/` 每个文件的 sha256：

```
3f8cf2f75cb9a7d274234fba442eb672519e8635a6212582a9c05372653c1c09  src/attack_full_pool.rs
2fd7c6f44725887272e817b85f036116d4992ad56231477a2b76fd60634413f6  tools/make_the_small_pool_fixture.py
407e31a9625e258ff43ad76278f6fcda0060cda6b06b809cbeb4fb859b72a48b  tools/rerun-from-a-clean-work-directory.sh
f8459f1d1efe8522a864a961c72a0fde9fe5fb117f735e26bc9bf46b70474357  tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 证据状态 |
|---|---|---|
| T3 只用 accept 写的走不出来（`full_pool_*`） | 没有新判定：探针修好待跑；前任报告里这一格仍是推的 | 建池崩溃是任务 11092 量的；修法没跑过 |
| 派发提示的两条路（只放大盘、只放小环） | 都不取，对不上的数见下一节 | 算术，读码 |
| 建池之外，这份探针读不准的两处 | 报出来并加了读数臂，见「建池之外动了什么」 | 推的 |

## 改法：盘 320 MiB、环 64 MiB

约束：`crates/singlefs-core/src/make_filesystem.rs:489`：`if ring_bytes > smallest / 4 {`。单元区从环末尾的下一个槽起（`crates/singlefs-core/src/allocator.rs:275` 的 `following_the_journal_ring`），环从 16 MiB 起，一段 1 MiB。探针写死 `FILES_AT_MOST = 400`、一份文件 32 单元占一段；两块盘同槽存，容量按一块盘算（`crates/singlefs-core/src/allocator.rs:2488` 那段文档，读码）。

**对不上的是派发提示的二选一**：环放小而盘留 1 GiB、或只放大盘，单元区都超过 400 段，400 份文件写不满，`full_pool_fill` 的 `refusal` 是空串，后面 `accept` 一拍就过，读出来是一次假的「推翻」。要留住 400 份就得连盘一起缩。第三条路就是下表末行，它满足「别的不动」：探针设计的 240 段（前任按 1 GiB 减默认起点 784 MiB 算，文件头写的「约 200 段」）不变。

| 盘、环 | 环不超过盘的四分之一 | 单元区段数（算术） | 400 份写得满 | 只算数据落点扫描的挂钟（推的） |
|---|---|---|---|---|
| 1 GiB、768 MiB（任务 11092） | 否，量的：mkfs 拒 | | | |
| 3 GiB、768 MiB（只放大盘的下限） | 是 | 2288 | 否 | 约 4.5 小时 |
| 1 GiB、256 MiB（只放小环的上限） | 是 | 752 | 否 | 约 10–48 分钟 |
| **320 MiB、64 MiB（取这个）** | 是，64 ≤ 80 | 240 | 是 | 约 20–97 秒 |

末列算法：单元数 × 2 块盘 × 平均扫的候选数（段数 × 16）× 每个候选比一遍已开的段（`crates/singlefs-core/src/allocator.rs:972` 起的循环，`crates/singlefs-core/src/allocator.rs:980` 那一判；240 段取平均约 33 次比较、752 段约 100、2288 段约 300，都是拍的）× 每次 10–50 纳秒（dev 编译，拍的；3 GiB 那一格只取 10 纳秒）。

改的文件（新生成一份搭建，冻结副本那一份不动）：`tools/make_the_small_pool_fixture.py` 换四处，每处旧串都在冻结副本里 `grep -cF` 命中 1 次：

- `crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices/mod.rs:31`：`pub const DEVICE_BYTES: u64 = 4 << 30;` 换成 `320 << 20`。
- `journal_ring_bytes: default_journal_ring_bytes(),` 换成 `64 << 20`。
- `crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices/mod.rs:363`：`DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),` 与盘 1 那一行，换成 `with_unit_area_start(…, genesis.unit_area_start)`。不换的话环一放小，mkfs 同一进程里的分配器还按默认起点 784 MiB 建，第一步就 panic（读码）。产品入口 `allocator_after_make_filesystem` 也是这样建的（`crates/singlefs-core/src/make_filesystem.rs:867`）。

环放小量歪了什么（推的）：`crates/singlefs-core/src/transaction.rs` 的一次发布记录条数上限从 65536 变 5461；环长在四份核心文件（事务、准入、会话、挂载）里 `grep` 到的用处都是记录落点、条数与挂载读环，准入文件没有命中，所以这一格的空间判定不随环长变。**推翻这一句的现象**：读数里出现 `JournalRecordsOfThePublishExceedTheLimit`。非默认环走挂载有入库用例：`crates/singlefs-verification/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:208`（128 MiB 的环，这条腿没跑）。

## 建池之外动了什么（超出「别的不动」，主 agent 定留不留）

原来那一条历史一个字没改：`topped_up=false` 的三组就是它。多了三样：

1. **填盘估时并超时就停**（定义「模型先跑一小段」）：腿跑不了短样，只能放进探针。每 16 份文件打一行 `name=full_pool_fill_estimate`，按最近两段的每份耗时线性外推，超过 2400 秒 panic。只读钟，不碰盘。
2. **多三组 `topped_up=true`**：重挂之后在这一次挂载里先写 1 MiB 的文件、再写与 `overwrite` 同样 35 字节的文件，都写到被拒，打一行 `name=full_pool_top_up`（`accept_before_unlink_went_through` 是对照），再删文件、走同样的 40 拍。用户动作放开扫，只固定前缀。
3. 每组末尾打 `name=full_pool_arm_seconds`，下一次估时用。

**为什么要第 2 样（两处都是读码推的，没量）**：

- 分配器每次挂载新建，开过的段记在只增不减的集合里（`crates/singlefs-core/src/allocator.rs:1951`，`crates/singlefs-core/src/allocator.rs:2596`；分配器、挂载、会话三份文件里 `grep` 到的写入只有这一处）。数据单元只避开这个集合（`crates/singlefs-core/src/allocator.rs:980`）。所以重挂之后，上一次挂载的段里放出来的成对空槽数据单元又能用，「写满」跨挂载不算数。
- 1 MiB 的文件被拒时还可能剩不到一份的余量，一单元的 `overwrite` 落得下。

**按原样跑会出什么问题**：`topped_up=false` 那三组在第 1 拍、`total_publishes=0` 时 `accept_went_through=true`。这说明重挂或余量给了空间，不说明维护线程把写者救出来了，却满足前任「40 拍内 accept 过了」的字面。

## 翻面条件（沿用，两组各自判）

`research/prompts/m4-gpugc-mount-runtime-code-r1-opus-output.md:286`，整行抄：

`| 只用 accept 写的走不出来 | `name=full_pool_*` | 40 拍 `accept_went_through=false`，之后 `name=full_pool_publish_user_change` `went_through=true` | 40 拍内 accept 过了；或填盘挡在别处 |`

这一行不改，`topped_up=false`、`topped_up=true` 两组分开各判一遍。跑之前写下的读法（改的是读数的分组，发生在产物之前）：

| 读数 | 我接受的结论 |
|---|---|
| `name=full_pool_fill` 的 `refusal` 是空串 | 装置没写满，整趟作废，不算翻面 |
| `topped_up=true` 且 `accept_before_unlink_went_through=true` | 这一组没写满，这一组作废 |
| `topped_up=true`（对照 false）40 拍全 false、之后 `went_through=true` | 打中坐实 |
| `topped_up=true`（对照 false）40 拍内过了，且过之前 `total_publishes` 非 0 | **我撤回「走不出来」**：维护线程救得出来 |
| `topped_up=false` 第 1 拍过、`total_publishes=0`，而 `topped_up=true` 打中 | 打中只在写满的挂载上成立；重挂之后的余量是另一件事，一起交主 agent 判。这一组字面算推翻，由主 agent 定是否采信我这一读 |
| 六组都打中 | 打中更强：连重挂之后的余量都救不出 |

## 估的挂钟与内存（全部推的，没量）

| 段 | 估 | 依据 |
|---|---|---|
| 拷树与编译 | 不到 1 分钟 | 量的：任务 11092 用时 526.842 秒，减 `attack_probes` 482.06 秒、`attack_gpu_opener` 2.10 秒，剩约 43 秒，那一趟连 GPU 包一起编。这一趟不编 GPU 包 |
| 填盘 | 1–6 分钟 | 至多 240 份、480 次发布；扫描 20–97 秒，留 3 倍给发布路径的别的开销（拍的） |
| 六组 | 3–12 分钟 | 每组拷盘、挂载、至多 40 拍，每组 0.5–2 分钟（拍的） |
| **合计** | **约 5–20 分钟，不超过 40 分钟** | 上面三段相加；超出时填盘那一道拦得住，六组那一段没有拦 |

内存：一块盘写满约 49 万个扇区，两块约 98 万个，每个约 570 字节（扇区加 `Vec`、树节点、分配器开销，拍的），一个池约 560 MiB；底池加一份拷贝约 1.1 GiB，峰值约 1.5 GiB（推的）。任务 11092 第一次尝试上限 4096M（量的），放得下；撞上限由服务放大重排，重排安全。

崩溃状态：探针不枚举，全量状态数 0。估出超 40 分钟（填盘 panic）的缩法：盘 176 MiB、环 40 MiB（40 ≤ 44），单元区 120 段，改生成脚本两个数与探针 `WHOLE_SEGMENTS_OF_THE_UNIT_AREA`，别的不动。

## 要主 agent 跑的命令

**先拷模型目录**（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」核查第 3 条），在仓根：

```
cp -r research/prompts/m4-gpugc-mount-runtime-code-r1-opus-s2-model /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model
```

**再交这一条**（在哪个目录起：`--cwd` 给的拷贝；只跑 `attack_full_pool` 这一个目标，`attack_probes`、`attack_gpu_opener` 的源文件不拷进树里，也不碰任务 11092 的 `run/`；不碰卡）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model -- bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-run /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model
```

能安全重跑：`s2-run` 里有上一次留的记号才清，没记号的非空目录退 2、不碰。

**跑完看**：`TASK_SERVER_RESULT` 的 `exit=`，日志末行 `name=rerun_exit attack_full_pool=<码>`；读数在 `s2-run/readings.out`（`name=` 开头），原始输出在 `s2-run/attack_full_pool.out`。

| 结局 | 怎么读 |
|---|---|
| `exit=0` | 探针跑完，照上一节读 |
| `exit=1`，`panicked at` 带「填盘估出来」 | 估时那一道拦下，照上一节缩法改 |
| `exit=1`，`panicked at` 带「可写挂载」 | 写满的盘重挂被拒。这本身是读数，不算翻面，带着日志新派同立场的腿 |
| `exit=1`，`panicked at` 带「写成了文件」 | 一份文件都没写成，读 `full_pool_fill` 的 `refusal`，建池要再调 |
| `exit=1`，有 `error` | 编不过，带任务号新派同立场的腿 |

`tools/rerun.sh` 从模板拷，只改 `rerun_steps`。模板 `--selftest` 用假冻结副本跑 `rerun_steps`，改过的这一段里 `cargo test` 在假树上跑不通，所以它不适用于这一份，没写进命令。

## 没打中的形状

这一趟不攻新形状。

## 这条腿自己的限度

- 没编没跑。类型按前任探针已编过的同样写法写（任务 11092 编过前任那一份），新加的是两个小函数与一个结构体，编不编得过要跑了才知道。
- 挂钟的单价、段比较次数都是拍的；线性外推在每份耗时超线性增长时估低，靠每 16 份重估一次补。
- 六组那一段没有超时拦截。
- `topped_up=true` 的写满按「一次 35 字节的新文件写被拒」判；`overwrite` 走的计划与新文件写不完全相同，二者需求差多少没算，所以留了 `accept_before_unlink_went_through` 当对照。
- 写满时的空间形状（有没有成对空槽、段是否跨文件）都没量，判读全靠 `name=` 里打出的 `empty_by_the_records`、`openable` 与拒绝成员原文。

## 没做什么

- 没跑任何代码，没往服务交任务；判决要引的数都没有。上面的「打中坐实」要主 agent 复跑之后才算线索以上。
- 没改 `attack_probes`、`attack_gpu_opener`，没重读任务 11092 的别的读数。
- 改法没提新的产品改动；前任给的甲乙丙三个改法没碰、仍是被攻过零轮。
- 没建仓副本与编译目录；格式检查用的小目录（只有探针拷贝与一份空文件）已删，没有要留的。
