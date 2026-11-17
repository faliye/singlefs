# device-library-write-many-code-r3 正推腿报告（Sonnet，2026-11-10）

批次：records/2026-11-10-门禁与实验欠账收尾调度.md#5。立场：正推（从 `crates/` 今天的实现与已定条款推出「应该是什么」，再与被判对象逐格比）。
没写模型，没起脚本，没往任务服务交任务；要跑的命令在末尾「要主 agent 跑的命令」一节。
被判文件现状：`crates/singlefs-device-library/src/lib.rs` 的 sha256 与开工快照一致（`sha256sum` 与 `research/prompts/device-library-write-many-code-r3-snapshot/crates-sha256.txt` 同为 `773f90a766cb5826…`）。
没读：`device-library-write-many-code-r3-opus-output.md`、`device-library-write-many-code-r3-main-verification.md`。
读过的前两轮材料：`research/prompts/device-library-write-many-code-r2-main-verification.md`（第三节四条的原文）。

## 先说一件影响读数的事（时序）

量的：`stat` 看 `crates/singlefs-device-library/src/lib.rs` 的最后修改，比任务 12234 日志的结束只早 0.6 秒；任务 12234 自己跑了 30.164 秒（`task-server/task status 12234`），任务 12233 跑了 39.234 秒、任务 12232 跑了 210.827 秒，结束时刻都晚于 lib.rs 的最后修改，开始时刻都早于它。
所以这三个任务开跑时读到的 lib.rs 不是此刻这一版。我把 r2 实现员补丁（`device-library-write-many-r2-fix/patch/crates.patch`）打在它的 baseline 上，再与此刻文件逐行 `diff`：

```
$ sha256sum patched.rs     # 14b72704b259ec571f3bcd36b71016d905a8f6f4b3c1edabe08073c65853114c，等于 apply-outcome.tsv 回读行
$ diff patched.rs crates/singlefs-device-library/src/lib.rs
851a852,854
>     /// 装置线程每一轮调一次口子的那一步：读的交回读到的，写的交 None。
>     type AskTheLibrary = Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>;
> 
855c858
<         ask: Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>,
---
>         ask: AskTheLibrary,
```

即只差主 agent 抽出来的那个测试内的类型别名。任务 12237（clippy）、12238（测试）开始在别名之后（各只跑 1.0 秒、2.0 秒，结束时刻晚于 lib.rs 修改 15 秒左右）。
结论：12232、12233、12234 的读数量的是别名之前的一版；别名只在 `mod tests` 里、不改产品代码，行为等价是推的（`diff` 只有上面这些行是量的）。要在此刻文件上坐实，见末尾命令 1。

## 一、正文第五节第 1 格：第二轮判决第三节四条

第二轮判决原文（逐条）在 `research/prompts/device-library-write-many-code-r2-main-verification.md` 的「三、交实现员」一节。

### 条 1：`read` 发请求之前核相对键，转交与内存两个后端都核；不合退 3、一个字节都不发；配三条测试与变异行

判：**修到**。

- 核键在 `read` 里排在拿锁、写请求、按后端分叉之前：「give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:192`），下一句才是「match &*self.backend {」（`crates/singlefs-device-library/src/lib.rs:193`）。内存那一支（同文件 `lib.rs:234-239`）在 `match` 里面，所以也被这一句罩住。
- 三处用同一个函数、同一个次序（先核键、再拿锁、再拼请求）：`read` 在 `lib.rs:192`，`write` 在「give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:246`），`write_many` 在循环里「give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:271`）。函数本体在「fn give_up_unless_the_relay_accepts_the_key(relative: &str, asked: &str) {」（`crates/singlefs-device-library/src/lib.rs:100`）。
- 三条测试：带换行（`lib.rs:761`，断言退 3 且 `requests_sent` 为空串）、带空格（`lib.rs:785`，同样两条断言）、内存坏键（`lib.rs:808`，断言退 3）。
- 变异行三条：`crates/mutations.tsv:7605` 「判定库口子 read 不核相对键」点名带换行那条；`:7606` 「判定库口子 相对键规则放过空白」点名带空格那条；`:7607` 「判定库口子 read 只在转交那一种核相对键」点名内存坏键那条。
- 证红：任务 12233 日志里这三行都是「抓到」（点名测试各自「红了」）；任务 12232（变异表静态核）退 0，末尾「CHECKED 7606 783 12」。
- 修过头？没有。下游内存库测试在改后仍绿：E158 任务 12235 `158 passed`，E174、E178 任务 12236 `30 passed`、`54 passed`。三处调用方的键都是 `items/<键>.out`、`codes/<键>.txt`、`index.tsv`、`<轮键>.out` 拼成的（`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs:2929-2939`，`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:24446-24448`）。一处旁注：E174 读 `previous_index.get(...)` 取出的 `previous_key` 来自库里存的 `index.tsv`（`e174_lease_drop_wait_memory_return.rs:3117`），库里那份若被写坏，`read` 现在退 3 而不是发出一个畸形请求——这是条 1 要的行为，不算过头。
- 推翻条件：任一个后端的 `read` 在坏键上仍交 `None` 或发出字节；或三处核键的函数不是同一个。

### 条 2：`found` 报的长度分配不出时退 3，用可失败的分配；配测试与变异行

判：**修到**，带一处旁注（推的，没量）。

- 预留改成可失败的：「let mut contents = Vec::new();」（`crates/singlefs-device-library/src/lib.rs:218`）、「if let Err(error) = contents.try_reserve_exact(length) {」（`crates/singlefs-device-library/src/lib.rs:219`），预留不成走 `give_up(OTHER_ERROR_EXIT_CODE, …)`，随后「contents.resize(length, 0_u8);」（`crates/singlefs-device-library/src/lib.rs:225`）。预留成功之后 `resize` 不再分配，不会 abort。
- `isize::MAX` 那条路仍在：「.filter(|length| isize::try_from(*length).is_ok())」（`crates/singlefs-device-library/src/lib.rs:213`），失败走 `give_up_on_reply`（打「回应认不出」）。两条路分得开：超过 `isize::MAX` 的测试多加了一句断言要求标准错误含「回应认不出」（`lib.rs:709` 起的测试，断言在 `lib.rs:730`），`2^62` 的测试（`lib.rs:737`）只断言退 3、且 `2^62` 不过 `isize::MAX`、过滤放行，只能由预留那一支退出。
- 变异行：`crates/mutations.tsv:7608`「判定库口子 found 报的字节数照不可失败的分配」把 `Vec::new()` 换成 `Vec::with_capacity(length)`，任务 12233 日志「抓到」（被 SIGABRT 杀掉时退出码是 `None`，不等于 `Some(3)`）。`:7596` 的老行「判定库口子 found 报的字节数过大也照分配」（把过滤换成恒真）现在只靠新加的「回应认不出」断言抓得到（预留对超过 `isize::MAX` 的长度也回错，退出码同样是 3，两支只差那句话），任务 12233 日志也是「抓到」。
- 走不到的理由：实现员报告写「无」（`device-library-write-many-r2-fix/result.md` 的「走不到的理由」一节），产品代码里 `unreachable!`、`todo!`、`assert` 在第 1–330 行零命中（见第三节第 1 格）。
- 旁注（推的，没量）：旧写法 `vec![0_u8; length]` 走零页分配（不摸页），新写法 `resize` 会把预留下来的每个字节写一遍 0，也就是摸遍所有页。回应报的长度很大但地址空间放得下（例如几十 GB）时，旧写法在 `read_exact` 读到文件尾退 3，新写法先摸页，可能在内存上限里被杀（退出码不是 3）。可达性：长度来自转交脚本从库里读出的真值长度，现有调用方的值是 KiB 到 MiB 量级，走不到；仅当转交那一侧说谎时才碰得到。我没起探针，不判它「修过头」，只记一笔。
- 推翻条件：把预留换回不可失败的分配后，`lib.rs:737` 的测试仍绿；或转交一个 `found 2^62` 的回应时进程不是退 3。

### 条 3：放锁另三处（`read` 读 `found` 内容、`write`、`write_many`）各配一条抓得到的测试与变异行，造法照 `a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply`

判：**修到**。

- 四条探锁测试共用一个函数：「fn round_where_the_lock_was_free_in_the_middle_of_the_exchange(」（`crates/singlefs-device-library/src/lib.rs:895`），别名 `AskTheLibrary` 在「type AskTheLibrary = Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>;」（`crates/singlefs-device-library/src/lib.rs:853`）。四条测试：`read`（`lib.rs:1061`）、`read` 回 `found`（`lib.rs:1077`）、`write`（`lib.rs:1093`）、`write_many`（`lib.rs:1113`）。`found` 那条把回应拆成两段（`found 5\n` 与 `hello`），探锁的一方在第一段写完之后、第二段许可之前抢锁，所以「放锁在 found 行与内容之间」才判得出；放锁点在 `lib.rs:218` 前后（变异把 `let mut contents = Vec::new();` 前面插入「放锁再拿锁」）。
- 变异行三条加原有一条：`crates/mutations.tsv:7609`「判定库口子 读的锁在 found 那一行与内容之间放过」、`:7610`「判定库口子 写的锁在请求与 written 之间放过」、`:7611`「判定库口子 成批写的锁在请求与 written 之间放过」，`:7598` 原有的读那一行仍在。
- 证红有两路：任务 12233 里 `:7609`、`:7610`、`:7611` 三行「抓到」（日志行 13、14、15，都标「现判：行新加」）；任务 12234 的多轮量法里四个放锁变异各 5 趟全红、没改坏的 5 趟全绿（第二节逐字对）。`:7598` 不在任务 12233 的十条里，它的证红只来自任务 12234（`variant=read-lock-between-request-and-reply` 的 `named_failed=5`）：探锁测试被重构过、这一行的点名测试内容变了，门禁复跑路径（`--item`）上它这一趟没单独点名过——见末尾命令 2，把它加进 `--list-items` 点名。
- 绿的一侧不依赖时序：没改坏的实现里，探锁抢到锁就意味着装置线程已读完最后一段回应，而「写到第几段」的计数在写之前就记上了（探锁函数的文档注释写了这一点：「装置线程读完最后一段回应才放锁，那一段在写之前就记上了」（`crates/singlefs-device-library/src/lib.rs:894`）；实现是 `reply_parts_written` 先 `store`（`lib.rs:963`）再 `write_all`），所以不会误报；时序只影响红的一侧抓不抓得到。
- 推翻条件：把 `lib.rs:195`（`read` 的 `ends.lock()`）之后任一处改成放锁再拿锁，而四条测试仍全绿；或没改坏的实现下这四条出现红。

### 条 4：文件头与转交脚本文件头里「一帧以内一次提交」改成不外推的说法；读回非 UTF-8 交 `None` 的文档改成实话

判：**修到**，带一处指路小偏差。

- `lib.rs` 文件头：「整批经 prune-server 客户端的 write_many 写服务，客户端可能把一批切成几帧、各帧各自提交：条数、字节数、值的形态（长值与以分段清单魔数起头的值分段存）、」（`crates/singlefs-device-library/src/lib.rs:9`）、「一批写到半截出了事，服务里可能只有其中几帧。整批的 written 在全部帧都确认落盘之后才回。」（`crates/singlefs-device-library/src/lib.rs:11`）。全仓（`.rs`、`.py`、`.md`、`.sh`，不含 `research/prompts/`）`grep -rn '一帧以内'` 零命中。
- 转交脚本文件头：「再经客户端 write_many 写服务：客户端可能把一批切成几帧各自提交（条数过 FRAME_BATCH_KEYS、字节过 FRAME_BATCH_BYTES、值的形态」（`scripts/experiment-apparatus/device-code-key.py:55`）、「与键留着的旧版本都会让它多切，切法以 prune_server_client.py 为准），原子性只在一帧之内；整批的 written 在全部帧都确认落盘之后才回。条数至少 1」（`scripts/experiment-apparatus/device-code-key.py:56`）。
- 三个「会多切」的因素各有出处：条数（`scripts/prune-server/prune_server_client.py:348` 的 `FRAME_BATCH_KEYS = 8192`）、字节（同文件 `:347` 的 `FRAME_BATCH_BYTES = 16 * 1024 * 1024`）、值的形态与旧版本（同文件文件头「写」一节第 2 步，`:47` 起：「2. 登记了而不是此刻清单那一版的」）。
- 非 UTF-8：文档两处——`read` 的文档注释（`crates/singlefs-device-library/src/lib.rs:189`）与文件头（`crates/singlefs-device-library/src/lib.rs:16`「found 后面的内容不是 UTF-8 时 read 交 None（与键不在一样，调用方去现判），不退。」）；行为：「String::from_utf8(contents).ok()」（`crates/singlefs-device-library/src/lib.rs:232`）。文档与行为一致。
- 指路小偏差：`lib.rs:10-11` 写「切法以 scripts/prune-server/prune_server_client.py 文件头为准」，但 8192 条这条切法只在该文件的常量与注释里（`prune_server_client.py:346-348`、`:1110` 的 `batches_of` 里「if batch and (size + item_size > FRAME_BATCH_BYTES or len(batch) >= FRAME_BATCH_KEYS):」），文件头（第 4–272 行的文档字符串）里写了字节上限与分段，没写条数上限（`awk` 数过，文档字符串里没有 `8192`）。转交脚本那一句只写「以 prune_server_client.py 为准」，没这个偏差。不影响第二轮判决点名的那一格（不外推原子性），只是指路差一步。
- 没有测试或变异行钉住「非 UTF-8 交 `None`」这一句（`grep -n 'from_utf8' lib.rs` 只有 `lib.rs:232` 一处产品代码，测试里没有喂非 UTF-8 内容的）。判决没要求，记一笔：把它改成 `from_utf8_lossy` 不会被任何现有测试抓到。
- 推翻条件：任一份文件头仍写「一帧以内是一次提交」；或 `read` 在非 UTF-8 内容上不交 `None`。

### 第 1 格小结

| 条 | 判 | 主要证据 |
|---|---|---|
| 1 `read` 核键 | 修到 | `lib.rs:192` 在 `lib.rs:193` 的 `match` 之前；测试 `lib.rs:761`、`:785`、`:808`；变异 `mutations.tsv:7605-7607`；任务 12233 全「抓到」 |
| 2 `found` 可失败分配 | 修到（旁注：`resize` 摸页，推的） | `lib.rs:218-225`；测试 `lib.rs:709`、`:737`；变异 `mutations.tsv:7596`、`:7608` |
| 3 放锁另三处 | 修到 | `lib.rs:1077`、`:1093`、`:1113`；变异 `mutations.tsv:7609-7611`；任务 12233、12234 |
| 4 文件头与 UTF-8 | 修到（指路小偏差，无测试钉 UTF-8 一句） | `lib.rs:9-11`、`:16`、`:189`、`:232`；`device-code-key.py:55-56` |

没有「没修到」，没有量到的「修过头」。

## 二、正文第三节第 3 条：第二节每个读数在任务日志里逐字找得到；`rounds.sh` 每趟一个编译目录

日志都在 `/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-<号>.attempt-1.log`，下表「行」是该日志的行号（`grep -nF` 逐字命中的行）。结局与上机台数取自 `task-server/task status <号>`。

| 第二节的读数 | 日志里逐字的那一行 | 结局、台 | 判 |
|---|---|---|---|
| 12238 `test result: ok. 21 passed` | 行 28：`test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s`；行 6–27 列了 21 条 `... ok` | succeeded 退 0、本机 | 对上 |
| 12237 clippy 退 0 | 日志只有两行（`Checking singlefs-device-library`、`Finished dev profile … in 0.14s`），没有 `✗`；命令是 `cargo clippy -p singlefs-device-library --all-targets --all-features -- -D warnings -D clippy::wildcard_enum_match_arm …` | succeeded 退 0、本机 | 对上（退 0 取自 status，日志里没有单独的退出码行） |
| 12232 变异行静态核退 0 | 行 518：`MULTI_HOST_RUN_STEP step=peer-command event=end … exit=0`；行 509 的 `CHECKED` 行第一个数是 7606（我另数 `crates/mutations.tsv` 为 7606 行变异，同数）；命令是 `python3 scripts/mutation/crates-mutation-rows.py static-check /home/fy5090/code/singlefs` | succeeded 退 0、第二台 | 对上 |
| 12233 `✓ 点名 10 条：现判 4 条、复用 0 条、拒算 6 条；跑了 10 条，抓到 10 条` | 行 20：该句后面还有「（其中复用 0 条），本次未跑 0 条，跑了的都抓到了」，读数是行首的一段 | succeeded 退 0、本机 | 对上（背景材料是行首一段的逐字摘录） |
| 12234 `name=variant_summary variant=unmutated … named_failed=0 any_failed=0 of=5` | 行 6：`name=variant_summary variant=unmutated named_test=- named_failed=0 any_failed=0 of=5` | succeeded 退 0、本机 | 对上 |
| 12234 四个放锁变异各 `named_failed=5 any_failed=5 of=5` | 行 12（`read-lock-between-request-and-reply`）、行 18（`read-lock-between-found-and-contents`）、行 24（`write-lock-between-request-and-reply`）、行 30（`write-many-lock-between-request-and-reply`），每行末尾都是 `named_failed=5 any_failed=5 of=5` | 同上 | 对上 |
| 12235 E158 `158 passed` | 行 584：`test result: ok. 158 passed; 0 failed; …; finished in 345.33s`；行 111 `Running unittests src/bin/e158_root_choice_repair.rs` | succeeded 退 0、第二台 | 对上 |
| 12236 E174 `30 passed`、E178 `54 passed` | 行 119：`test result: ok. 30 passed`（行 85 `Running … e174_lease_drop_wait_memory_return.rs` 之后）；行 179：`test result: ok. 54 passed`（行 121 `Running … e178_leaf_and_record_death_counts.rs` 之后） | succeeded 退 0、本机 | 对上 |

读数本身全部逐字找得到。但读数周围的三句话对不上或要加限定：

1. **「第二节的任务都派在第二台跑过」（第四节「所有任务多机可用」格的前提）不成立。** `task-server/task status` 的「第 1 次：…台」：12232、12235 是第二台，12233、12234、12236、12237、12238 是本机。新加的四条探锁测试、四条新子进程测试，都只在本机跑过（12234、12238）。推翻它的现象：`task status` 里这五个任务显示第二台。
2. **读数不是此刻文件上量的**：见开头「时序」一节，12232、12233、12234 量的是别名之前的一版（只差测试内类型别名）；12235 跑了 402.988 秒，它依赖的 `singlefs-device-library` 是什么时候编的我没法从日志判定（编译目录在第二台本地），记「推不出」。12237、12238 在别名之后。
3. **12233 的「拒算 6 条」不是漏洞**：日志行 3–8 写的是「行键算不出（照跑、不复用、不写库）」，原因是点名测试用 `std::env::current_exe()` 再起自己（`crates/singlefs-device-library/src/lib.rs:547` 的 `std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))`）。那 6 条照跑且「抓到」，只是不会被复用。另：这个包的子进程测试现在是 11 条（`lib.rs:591`、`:615`、`:642`、`:661`、`:685`、`:709`、`:737`、`:761`、`:785`、`:808`、`:826`），`.claude/gate.d/refused-tests.tsv` 里 `singlefs-device-library` 零命中（`grep -c` 为 0），r2 实现员报告也记了这一笔，没有人登记。

### `rounds.sh` 每趟一个编译目录是不是真的

文件：`/home/fy5090/.local/share/singlefs/work/device-library-write-many-r2-fix/rounds/rounds.sh`。

- 每个变异一棵树：「local tree="$WORK/$slug/tree"」（`rounds.sh` 第 39 行），`minimal-copy.py` 从 `--repo` 拷（第 40 行），`apply_row.py` 换变异（第 45 行）；变异名唯一、原文恰好命中一次才换（`apply_row.py` 的 `hits != 1` 退 1，脚本据此退 2）。
- 每趟一个编译目录：`rounds.sh` 第 53 行的 `CARGO_TARGET_DIR="$WORK/$slug/target-round-$round"`，目录名带变异名与趟号；每趟跑完第 69 行的 `rm -rf "$WORK/$slug/target-round-$round"`，每个变异跑完第 72 行的 `rm -rf "$tree"`。`--work` 目录已存在就退 2（第 27 行），不会复用上一趟的目录。
- 判法没有把编译错误算成「抓到」：`named_failed` 只在日志里有 `test tests::<名> ... FAILED` 时加一（`rounds.sh` 第 59–61 行，`named="missing"` 在第 65 行），编译失败的 `rc` 非 0 但 `named=missing`，不进 `named_failed`。
- 任务 12234 的 status 是「编译目录 无」：服务没给这个任务指编译目录，所以起作用的只有脚本自己设的 `CARGO_TARGET_DIR`。
- 间接证据（量的）：各变异的失败集不同——读（请求回应之间）变异 `19 passed; 2 failed`、另三个 `20 passed; 1 failed`、没改坏的 `21 passed`（12234 日志行 1–30）。若变异之间共用编译目录，后跑的二进制会盖掉前面的，失败集会趋同；这正是 r2 辩方脚本栽的地方。
- 没法直接坐实的：`rounds-run-1` 运行目录已经不在（`ls` 不到），脚本留下的 `logs/*-round-*.log` 里的 `Compiling singlefs-device-library` 行本来是最直接的证据，现在看不到。命令 1 重跑后一条 `grep -c` 就能坐实 25 个日志各编了一次。

判：**成立（读脚本得出；运行留痕已被清掉，待命令 1 复核）**。推翻条件：命令 1 跑完后 `logs/*-round-*.log` 里有任何一趟没有 `Compiling singlefs-device-library`（说明那一趟复用了别处编好的二进制）。

## 三、正文第四节八格

### 检查单：走不到的理由二次查实 —— 成立

- 产品代码里的 `panic!` 只有两处：「Backend::Relay(_) => panic!("单测只数内存那一种"),」在 `count_entries`（`crates/singlefs-device-library/src/lib.rs:296`）与 `count_write_many_calls`（`crates/singlefs-device-library/src/lib.rs:310`）的转交那一支。其余 `panic!` 在测试模块里（`lib.rs:429`、`:985`、`:1034`）。本轮没有新加的产品 `panic!`、`unreachable!`、`todo!`、`assert`（`grep -nE 'assert|unreachable!|todo!' lib.rs` 只列第 330 行之后的测试代码）。
- 逐处追调用点（`grep -rn 'count_write_many_calls\|count_entries' --include=*.rs`，排除 `target/`）：`lib.rs:484`、`:486`（本包测试，库是 `DeviceLibrary::in_memory()`）；`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:29955`、`:29959`（库来自 `:29945` 的 `DeviceLibrary::in_memory()`）、`:30030`、`:30035`、`:30043`（库来自 `:30019` 的 `DeviceLibrary::in_memory()`）；`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs:4187`（在 `:3871` 起的 `mod tests` 里，`round_files_in` 的库来自 `test_round_library(&DeviceLibrary::in_memory(), …)`，调用在 `:4241`、`:4256`、`:4274`、`:4275`、`:4295`、`:4314`）；`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:7693`（在 `:7076` 起的 `mod tests` 里，库来自 `:7727`、`:7737`、`:7751`、`:7767`、`:7814` 的 `DeviceLibrary::in_memory()`）。生产路径上的库只有 `DeviceLibrary::from_environment()`（`e158_root_choice_repair.rs:26775`、`e174_lease_drop_wait_memory_return.rs:3489`、`e178_leaf_and_record_death_counts.rs:7006`），没有一处把它交给 `count_*`。
- 推翻条件：任何非 `mod tests` 的调用点，或把 `from_environment()` 造的库交给 `count_*` 的路径。

### 检查单：类型与名字 —— 成立（一处措辞小毛病）

- `AskTheLibrary`（`lib.rs:853`）有文档注释，名字说得出是什么；`ProbedExchange` 的四个字段各有文档注释；探锁函数名 `round_where_the_lock_was_free_in_the_middle_of_the_exchange`（`lib.rs:895`）说得出返回什么。空泛词表（`.claude/naming-vague-words`）要求名字的**每个**词都在表里才判违规，新名字没有这种。序号词（`.claude/naming-ordinal-tokens`）没有命中：新名字里是 `part_number`、`round_number`、`step_seen_at`，不是 `step1`、`part1` 那种。我没跑命名检查（那是门禁，交主 agent）。
- `expect` 理由：产品代码里的 `expect` 只有 `RELAY_LOCK_IS_NEVER_POISONED`（`lib.rs:195`、`:249`、`:275`）与 `MEMORY_LOCK_IS_NEVER_POISONED`，理由常量在「const RELAY_LOCK_IS_NEVER_POISONED: &str =」（`crates/singlefs-device-library/src/lib.rs:33`），本轮改成「可失败地预留（…预留不成照别的错退掉）」，与代码一致：持锁时的 `try_reserve_exact`（`lib.rs:219`）失败走 `give_up` 退进程，之后 `resize`（`lib.rs:225`）与 `String::from_utf8`（`lib.rs:232`）都不会 panic。测试里的 `expect` 写的是「这一步在测试里为什么成立」（例：`lib.rs:993` 的「这把锁只在本测试的三个线程之间传，没有线程握着它 panic」）。小毛病：`lib.rs:1042` 的 `expect("装置线程交回口子交回的")` 语句不通，不影响判断。
- `_ =>`：`grep -n '_ =>' lib.rs` 零命中；clippy 任务 12237 的命令带 `-D clippy::wildcard_enum_match_arm` 且退 0。
- 推翻条件：新名字全由空泛词组成；产品 `expect` 没有理由；`lib.rs` 里出现 `_ =>`。

### 检查单：不兼容旧代码与旧格式 —— 成立

- 现跑正文给的命令，`grep -nE 'legacy|old|migrat|兼容' research/prompts/_device-library-write-many-code-r3-diff.md` 退 0，命中 5 行（第 252、458、475、491、511 行），全是测试函数名里的 `holds`（`old` 的子串）：`a_read_holds_the_relay_lock_…`、`a_read_answered_found_holds_…`、`a_write_holds_…`、`a_write_many_holds_…`。`legacy`、`migrat`、`兼容` 零命中。同一条命令对 `lib.rs` 现文件：命中也只有 `lib.rs:1061`、`:1077`、`:1093`、`:1113` 四行测试名。
- 改动里没有认旧版本的分支：`from_environment` 没动（`lib.rs:154` 起），请求/回应格式只收严（`read` 核键）、没加兼容路径。
- 推翻条件：diff 里出现按版本号或旧格式分支的代码。

### 检查单：不新建判定库 —— 成立

- 本轮没有新的存储：产品代码里唯一的文件操作仍是 `from_environment` 里打开两根已存在的管道（「let requests = std::fs::OpenOptions::new()」`crates/singlefs-device-library/src/lib.rs:166`），`grep -nE 'std::fs::(create|write|remove|rename)|create_dir|File::create'` 零命中；测试用 `std::io::pipe()`，不落盘。
- 推翻条件：出现落盘或自己开库目录的代码。

### 检查单：所有任务多机可用 —— 成立（读码）；正文前提不成立

- 新测试读不读本机独有状态：产品代码里只有一处 Linux 路径 `/proc/self/fd/{text}`（`lib.rs:162`），两台都是 Linux；测试里没有 `/home`、`/tmp`、主机名、核数、`HOME`（`grep -nE '/home|/tmp|hostname|HOME|temp_dir|available_parallelism'` 零命中，只有 `/proc` 那一行）。`1_usize << 62` 的分配失败测试靠「2^62 字节大于任何用户地址空间」，对 x86_64 与 aarch64 成立，与 `vm.overcommit_memory` 的取值无关（本机是 0）。
- 前提不对：见第二节的第 1 条，五个任务是本机。新测试没在第二台跑过；想让服务派到第二台，调用方不许写 `--peer`、`--local-only`（共用约束「子 agent 不交任务」），所以我也没法替主 agent 指定。复核不了第二台实跑的结果，原因：机器选择归任务服务。
- 推翻条件：测试里出现本机路径、主机名；或同一条 `cargo test -p singlefs-device-library --lib` 在第二台红。

### 检查单：测试优先多线程 —— 成立

- 四条探锁测试是真多线程：装置线程、替身线程与探锁的本线程各一个（`lib.rs:890-891` 的注释写了这是场景的一部分，不是切片并行），共用 `lib.rs:895` 的函数。
- 绿的一侧不靠时序（见第一节条 3）。红的一侧靠探锁一方抢到放锁的空当：任务 12234 里四个放锁变异各 5 趟全红（每趟内部 200 轮），前两轮另有 30/30 与探针 300/300（`research/prompts/device-library-write-many-code-r2-main-verification.md` 的「一、核查」一节）。20 次里 0 次漏抓；这个次数只能把「每趟漏抓率」压到约 15% 以下（3/20，95% 置信，推的），不能说漏抓率是零。门禁 `--item` 一趟只跑一次，所以「偶尔漏抓」会表现成门禁偶发红——这条没法在本轮坐实，量法上要靠多跑几趟的读数，命令 1 能再给 25 趟。
- 其他 17 条测试里有 11 条起子进程（见第二节第 3 条），与探锁测试在 libtest 默认并行下同跑，任务 12238 一趟 21 条全绿（行 28）。
- 判：够。推翻条件：命令 1 里某个放锁变异有一趟 `named_failed` 小于 5，或没改坏那一份有一趟红。

### 检查单：带 --item 的命令与项 —— 成立（静态）；`--list-items` 的实列待主 agent 跑

- 我用只读脚本（`python3 -I`，只读 `crates/mutations.tsv` 与 `lib.rs`，不是项目代码）数：变异表共 7606 行，点名 `crates/singlefs-device-library/src/lib.rs` 的有 25 行；这 25 行每行的原文在 `lib.rs` 里都恰好命中 1 次（多行原文把 `\n` 还原后数），25 个变异名在表里各只出现 1 次，必须红的测试名都是 `lib.rs` 里存在的函数。与任务 12232 日志行 509 的 `CHECKED 7606` 同数。
- 本轮新、改的行共 10 条（`mutations.tsv:6983`、`:7590`、`:7596`、`:7605`–`:7611`），正是任务 12233 点名的 10 条；`:7598` 的探锁测试被重构过但没点名（见第一节条 3 的最后一点），命令 2 把它一起列。
- 推翻条件：命令 2 的 `ITEM_PLAN` 少列了某一项，或 `ITEM_CARD_DONE` 的项数不等于点名数。

### 检查单：新崩溃流多罩了什么 —— 成立

- 改动全文 `_device-library-write-many-code-r3-diff.md` 里 `crash`、`崩溃` 零命中（`grep -ciE 'crash|崩溃'` 为 0）；`.claude/gate.d/crash-point-replay-items.tsv` 里 `device-library` 零命中（`grep -c` 为 0）。没有新的崩溃点重放流或崩溃枚举用例。
- 推翻条件：diff 或登记表里出现这一类条目。

## 四、顺手记的三笔（不在八格里，没量）

1. `reply_line`（「fn reply_line(replies: &mut std::io::BufReader<std::fs::File>) -> String {」`crates/singlefs-device-library/src/lib.rs:110`）用 `read_line` 读一行回应，没有长度上限；回应不带换行地一直写下去时 `String` 无限增长，内存耗尽是 abort、不是退 3。与条 2 同一类，但回应来自自己的转交脚本（`device-code-key.py`），走不到；推的，没起探针。
2. `mutations.tsv:7596` 那一行的两支路（`isize::MAX` 过滤与预留失败）现在退出码相同，行能被抓到只因为新加了「回应认不出」那句断言（`lib.rs:730`）；这是在断言报错文字，不是行为。实现员报告已说明留下过滤是为了分开两句话（`device-library-write-many-r2-fix/result.md` 的「停下交主 agent 的设计问题」一节，「超过 `isize::MAX` 的 `filter` 留下了」）。
3. 子进程测试 11 条不在 `.claude/gate.d/refused-tests.tsv`（零命中），本轮又加了 4 条。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第五节第 1 格 条 1（`read` 核键） | 修到 | `lib.rs:192` 在 `lib.rs:193` 的 `match` 之前，两个后端都罩住；三条测试与三行变异，任务 12233 全抓到 |
| 第五节第 1 格 条 2（`found` 可失败分配） | 修到 | `lib.rs:218-225` 预留不成退 3；`resize` 会摸页是推的旁注，没量 |
| 第五节第 1 格 条 3（放锁另三处） | 修到 | 四条探锁共用 `lib.rs:895` 的函数，三行新变异与原有一行在任务 12234 里各 5/5 抓到；`:7598` 没进任务 12233 的点名 |
| 第五节第 1 格 条 4（文件头与 UTF-8） | 修到 | 两份文件头都改成「可能切几帧」；指路写「文件头」而 8192 条在常量处；无测试钉 UTF-8 一句 |
| 第三节第 3 条 读数逐字 | 对上 | 8 个读数在 7 个任务日志里都逐字找得到（行号见第二节表） |
| 第三节第 3 条 「都派在第二台」 | 不成立 | `task status`：只有 12232、12235 在第二台，另五个在本机 |
| 第三节第 3 条 读数对的是不是此刻文件 | 部分 | 12232、12233、12234 开跑早于 lib.rs 最后一次改动，量的是只差测试内类型别名的前一版（`diff` 量的），12235 推不出 |
| 第三节第 3 条 `rounds.sh` 每趟一个编译目录 | 成立（读脚本） | `rounds.sh` 第 39、53、69、72 行；运行留痕已被清，待命令 1 一条 `grep -c` 坐实 |
| 检查单：走不到的理由 | 成立 | 产品 `panic!` 只在 `count_*` 的转交支（`lib.rs:296`、`:310`），所有调用点是 `mod tests` 里的内存库 |
| 检查单：类型与名字 | 成立 | `AskTheLibrary` 与新名字不空泛，产品 `expect` 有理由，无 `_ =>`；`lib.rs:1042` 一句措辞不通 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 正文那条 grep 命中 5 行全是 `holds` 里的 `old`，`legacy`、`migrat`、`兼容` 零命中 |
| 检查单：不新建判定库 | 成立 | 没有新存储，只有管道 |
| 检查单：所有任务多机可用 | 成立（读码），前提不成立 | 测试不读本机独有状态；第二台实跑没有，机器选择归服务，复核不了 |
| 检查单：测试优先多线程 | 成立 | 四条探锁是三线程场景；绿的一侧不靠时序，红的一侧 20/20，漏抓率上界约 15% 是推的 |
| 检查单：带 --item 的命令与项 | 成立（静态） | 25 行锚点各命中 1 次、名字唯一、7606 行同数；`--list-items` 实列待命令 2 |
| 检查单：新崩溃流多罩了什么 | 成立 | diff 与登记表里都没有这一类 |

## 要主 agent 跑的命令

没有模型，没有 `rerun.sh`，不需要 sha256 清单。以下命令都没有起过。

命令 1（在此刻文件上重量多轮，并留下每趟编译日志）。起目录：仓根 `/home/fy5090/code/singlefs`。工作目录 `rounds-run-1` 还不在。

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标，没改坏的一棵树加四个放锁变异各一棵树; 范围=1 个测试目标 × 5 棵树 × 5 趟; 为什么=任务 12234 开跑早于 lib.rs 的最后一次改动，在此刻的文件上重量一次并留下每趟的编译日志，核每趟各编一次' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/device-library-write-many-r2-fix/rounds/rounds.sh --repo /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-sonnet/rounds-run-1
```

看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；任务日志里五行 `name=variant_summary`：`unmutated` 一行是 `named_failed=0 any_failed=0 of=5`，四个放锁变异各是 `named_failed=5 any_failed=5 of=5`。随后在仓根直接跑只读的 `grep -c 'Compiling singlefs-device-library' /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-sonnet/rounds-run-1/logs/*-round-*.log`，25 个文件每个都是 `:1` 才算每趟各编一次。

命令 2（`--item` 点名列出，不起 cargo）。起目录：仓根。

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=crates-mutation-replay 的 --list-items --with-keys 点名; 范围=11 项; 为什么=核本轮新、改的十行变异加探锁重构过的一行都能被 --item 点名列出，不起 cargo' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys --item '判定库口子 read 不核相对键' --item '判定库口子 相对键规则放过空白' --item '判定库口子 read 只在转交那一种核相对键' --item '判定库口子 found 报的字节数照不可失败的分配' --item '判定库口子 读的锁在 found 那一行与内容之间放过' --item '判定库口子 写的锁在请求与 written 之间放过' --item '判定库口子 成批写的锁在请求与 written 之间放过' --item '判定库口子 found 之后少读一个字节' --item '判定库口子 write 不核相对键' --item '判定库口子 found 报的字节数过大也照分配' --item '判定库口子 读的锁在请求与回应之间放过' /home/fy5090/code/singlefs
```

看哪几行：退 0；标准输出 11 行 `ITEM_PLAN item=<名>`，随后 `ITEM_CARD` 11 行与 `ITEM_CARD_DONE items=11`。

命令 3（把 `:7598` 那一行按门禁 `--item` 写法真跑一次）。起目录：仓根。

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=crates-mutation-replay 的 --item 点名一行; 范围=1 项; 为什么=探锁测试被重构过，任务 12233 的点名里没有这一行，补一次门禁写法的证红' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/crates-mutation-replay.sh --item '判定库口子 读的锁在请求与回应之间放过' /home/fy5090/code/singlefs
```

看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志里该项「抓到」。

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务、判决要引的数没复跑（命令在上面「要主 agent 跑的命令」一节）。
- 没读攻方腿本轮报告与判决；没跑命名检查、门禁、clippy、测试；没起探针量 `resize` 摸页的旁注。
- 第二台的实跑结果复核不了（机器选择归任务服务）；任务 12235 读到的是哪一版 `lib.rs` 推不出。
