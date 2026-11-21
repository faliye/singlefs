# device-library-write-many-code-r1：判定库口子成批写，代码轮第一轮正文（2026-11-10）

## 一、题面

`crates/singlefs-device-library/src/lib.rs` 给实验装置读写判定库的口子加了成批写 `write_many`：一条请求带多条（相对键, 内容），整批只回一个 `written`；另加单测用的计数 `count_write_many_calls`，把单条写与成批写共用的「发请求、等 written」抽成 `send_write_request_and_wait_for_written`。门禁 crates-adversarial-review（任务 11978）报这一份没被三方判决点名；用户 2026-11-10 定走一轮代码三方。这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_device-library-write-many-code-r1-diff.md`（一份文件，对 HEAD）。开工快照：`research/prompts/device-library-write-many-code-r1-snapshot/crates-sha256.txt`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的文件：`crates/singlefs-device-library/src/lib.rs`（快照 sha256 `60dd1dfd…`）。文件头写着协议：`read`、`write`、`write-many <条数>` 三种请求，三种都可能回 `failed <退出码> <一句>`，装置照退出码退。
- 管道那一头（转交给 prune-server 的一侧）：`scripts/experiment-apparatus/device-code-key.py` 文件头第 53–57 行写 `write-many` 的服务侧：整批读全、核全之后才动手，每条照 `write` 记账（写过的键、复用哨兵），再经客户端 `write_many` 一次写服务；有一条相对键不合整批一条都不写；条数至少 1。它的弄坏开关在同一文件头第 141–152 行（`write-many-first-entry-only` 等六个）。这份脚本不是这一轮被判的文件，当上下文读。
- 调用方：`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_judgement_library.rs:132` `self.store.write_many(&items);`（E158 按单位成批写）。
- 变异行：`crates/mutations.tsv` 第 7393–7399 行（判定库口子 write-many 七条）与第 7401、7405 行（E158 调用侧）。
- 测试：`crates/singlefs-device-library/src/lib.rs` 的 `mod tests` 新加三条：`write_many_announces_the_count_and_each_length_in_bytes_and_takes_one_written_for_the_whole_batch`、`write_many_of_nothing_sends_no_request_and_takes_no_reply`、`write_many_in_memory_stores_every_entry_and_counts_one_call_per_non_empty_batch`。
- 压着的条款：`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」（装置不开本地库，读写经服务）；同一份规则「一个门禁：两种跑法」里复用与「一块判完才落盘，写入是原子的」（`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 原子性与确认：`write_many` 回来时整批是否都已落盘；服务侧整批拒（一条键不合）时装置这一头是否退、不会把没写的当写了；管道读写半途断开（写了一半请求、回应读到一半）时，锁与后续请求的边界对不对。
2. 字节计数：头一行报条数、每条报内容字节数（UTF-8 字节，不是字符数），内容里带换行或空串时服务侧按字节读得回原样吗。
3. 抽出的 `send_write_request_and_wait_for_written`：单条 `write` 改走它之后行为与改前逐字相同吗（请求字节、读几行回应、失败的退出码与话）。
4. 内存那一种：`MemoryLibrary` 与 `count_write_many_calls` 只给单测用，会不会让真管道那一侧（`Backend::Relay`）在某条路上 panic；空批不发请求、不等回应，与服务侧「条数至少 1」两头对得上吗。
5. 多线程：几个线程共用一条管道，`write_many` 整问整答在一把锁里；一批很大时别的线程只是等，不会读到别人的回应吗。

## 四、检查单

### 检查单：走不到的理由二次查实

这一份没有新加 `todo!`、`unreachable!`、`assert!`。新加的 `panic!` 有两处：`count_write_many_calls` 在 `Backend::Relay` 上 `panic!("单测只数内存那一种")`，测试辅助 `replies_left_unread` 在 `Backend::Memory` 上 panic（在 `mod tests` 里）。主 agent 判前一处走不到：`count_write_many_calls` 全仓只在单测里调：本包 `mod tests` 一处、`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 第 30030、30043 行两处（在同文件第 29232 行 `#[cfg(test)]` 之后）（腿现跑 `grep -rn count_write_many_calls crates scripts` 核）。新加的 `expect` 两处是锁没中毒（`"转交判定库的锁没有中毒"`、`"单测内存库的锁没有中毒"`）。腿逐处追调用点判「走不到」成不成立。

### 检查单：类型与名字

新加：结构 `MemoryLibrary`（字段 `entries`、`write_many_calls`），函数 `send_write_request_and_wait_for_written`、`write_many`、`count_write_many_calls`，测试辅助 `replies_left_unread`。`write_many` 的入参是 `&[(String, String)]`（相对键, 内容），两个都是裸 `String`。腿判：键与内容不共用一个 newtype 算不算「同一个量不同语义共用裸类型」；名字有没有缩写；有没有 `_ =>`；每处 `expect` 写没写依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支：单条 `write` 与 `read` 的协议没变，`write-many` 是新加的一种请求。腿现跑 `grep -nE 'legacy|old|migrat|兼容' research/prompts/_device-library-write-many-code-r1-diff.md` 核。

### 检查单：不新建判定库

这一份就是装置写判定库的口子；改动只加成批写的请求，写的仍是管道那一头转给 prune-server 的那一份（文件头第 1–2 行）。内存那一种只在单测里用、测完即丢、不落盘（文件头「装置的单测换成 `DeviceLibrary::in_memory()`」那一句）。腿判内存那一种算不算新的判定存放处。

### 检查单：所有任务多机可用

改动不读本机独有状态、不写主机名与地址；管道描述符号从环境变量 `DEVICE_LIBRARY_REQUEST_FD`、`DEVICE_LIBRARY_REPLY_FD` 读，由起装置的一侧给。腿核。

### 检查单：测试优先多线程

三条新测试各自只发一两次请求、毫秒级，不切片；并发是被测对象之一（锁里整问整答），但三条测试都没起多线程。腿判该不该补一条多线程用例（几个线程同时 `write` 与 `write_many`，回应不串）。

### 检查单：带 --item 的命令与项

本轮改动没有这一类：没有新的崩溃点重放项、门禁项或实验复跑项；三条新测试经 `cargo test -p singlefs-device-library --lib` 点名跑，变异行第 7393–7399 行点名的测试目标是 `-p singlefs-device-library --lib`。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新的崩溃点重放流或崩溃枚举用例（查过 `.claude/gate.d/stage-inputs.tsv` 与 `.claude/gate.d/crash-point-replay-items.tsv` 的这一批改动不涉及本包）。

## 五、格

1. 代码做的是不是条款与文件头协议说的：「判定库只有一份」、「一块判完才落盘，写入是原子的」、文件头 `write-many` 那三行。
2. 第三节五条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
