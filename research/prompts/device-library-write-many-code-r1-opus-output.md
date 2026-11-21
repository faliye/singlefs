# device-library-write-many-code-r1：攻方腿（Opus）报告（2026-11-10）

立场：假设判定库口子 `write_many` 这一份改动是错的，造输入、回应序列与线程交错去打它。被判文件 `crates/singlefs-device-library/src/lib.rs`（sha256 `60dd1dfda44ff8c55876058a11ea4a6c28a3af5000406d2ebca9d759252dbd29`，与 `research/prompts/device-library-write-many-code-r1-snapshot/crates-sha256.txt` 第 1 行相同，量的：`sha256sum` 现跑）。管道那一头 `scripts/experiment-apparatus/device-code-key.py` 只当上下文读（此刻 sha256 `ebdfc09b85ba0d42e2941d4da3d955f928162fcc657736b94f7262e5a187ef6b`，它不在快照里，主 agent 判时以复跑输出里 `name=relay_source` 那一行的 sha256 为准）。

## 复跑命令与模型文件

模型目录 `research/prompts/device-library-write-many-code-r1-opus-model/`，复跑命令见末尾「要主 agent 跑的命令」第 1 条。文件 sha256（量的，`sha256sum` 现跑）：

| 文件 | sha256 |
|---|---|
| `relay_desync_model.py` | `9b8a95d640bc0b7a4a454480747db6bcc1df411a561b59b493e40d4d547455c7` |
| `device_library_attack_tests.rs` | `7415ac1b9bbd36343149e561cd3b58f2aba52d441c7f53cfb0689a6544f8819c` |
| `mutants/reply-check-dropped.anchor` | `480d44cbd03771822f9da897d5f7dddbcb204546256a840cd31ef497969a614e` |
| `mutants/reply-check-dropped.replacement` | `968c27e6c1a935f26b89f0defa2e100b10b8ee27c33cbe712fb0867fceb71d6a` |
| `mutants/lock-released-between-read-request-and-reply.anchor` | `7d686a16f8b2fe3a0b5d2254352c24e9e2122ed37ff5172fc631c625af8bc261` |
| `mutants/lock-released-between-read-request-and-reply.replacement` | `e8a3d5ba45ad7ff17e61835e3f4bd370d6c7bad960ed41c0d8c98ee8634c249e` |
| `tools/apply_once.py` | `fa14c697f07c5325f48efe52b6185d64fe4e6932e78529efa0e26dc4f1b55aa2` |
| `tools/rerun.sh` | `74e939d69f616d88c2971980c95bcb8d07f2a54f3e0862e964a5aea6088b14d1` |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`（量的：模板第 1–79、85–164 行与 rerun.sh 第 1–79、111–190 行 `diff` 无差）。模型一个数都还没跑出来：下文所有「模型会打出」都是推的，待主 agent 跑。

本题没有崩溃状态可枚举（被判的是一条进程间管道协议，不落盘、没有写流），第 3b 步的崩溃枚举与 10⁶ / 10⁷ 状态预算不适用，这一条腿跑的状态数是 0。

## 各格判定一览

| 格 | 判定 | 依据（细节在下面各节） |
|---|---|---|
| 攻击面一 原子性与确认：整批落盘才回 | 成立 | 转交整批读全、写完服务才回 written |
| 攻击面一 原子性与确认：「一条请求合成一次提交」 | **打中**（过度外推） | 客户端超过 8192 条或 16 MiB 就切成几帧，每帧一次提交；中途服务没了，前几帧已落盘、装置退 75、转交不撤 |
| 攻击面一 原子性与确认：整批拒时装置退、不当写了 | **打中**（条件可达） | 一批里有不合的相对键时，转交只回一个 failed 3，然后把这一批剩下的字节当新请求接着执行：内容长得像 `write …` 的那一段真会写进库，转交不撤；一批又大又多行时两头互等、装置卡死、不退 |
| 攻击面一 管道半途断开 | 成立 | 写一半断了退 3、回应读一半退 3；装置写到一半死了转交整批不写 |
| 攻击面二 字节计数 | 成立 | 报的是 UTF-8 字节数，转交按字节数读；带换行、空串照样分得开 |
| 攻击面三 抽出的发请求函数 | 成立 | 单条 `write` 改走它前后，请求字节、读的回应行数、退出码与话逐字相同 |
| 攻击面四 内存那一种不 panic 真管道 | 成立 | `count_write_many_calls` 只在 `cfg(test)` 里调到 |
| 攻击面四 空批与「条数至少 1」 | 成立 | 两头对得上 |
| 攻击面四 内存那一种与真管道一致 | **打中**（老问题，被这份改动带进成批写） | 内存那一种不核相对键，转交那一侧会整批拒的键它照收；装置单测绿、真跑退 3 |
| 攻击面五 多线程共用管道 | 成立（推的，没有测试罩着） | 写请求加读回应都在同一把锁里；三条新测试都是单线程，草稿补了一条多线程用例 |
| 检查单 走不到的理由二次查实 | 成立 | 两处 `panic!` 的调用点全在 `cfg(test)` 里 |
| 检查单 类型与名字 | **不成立** | 新加的三处 `expect` 只写「锁没有中毒」，没写靠哪条不变量不中毒；仓里别处的写法带理由 |
| 检查单 不兼容旧代码与旧格式 | 成立 | 现跑 grep 0 处 |
| 检查单 不新建判定库 | 成立 | 内存那一种是改动前就有的进程内单测替身，这份改动只换了它的值类型 |
| 检查单 所有任务多机可用 | 成立 | 只读两个环境变量与 `/proc/self/fd/<号>` |
| 检查单 测试优先多线程 | 成立（按这一格的判据） | 三条新测试都是毫秒级、没有可切的量；缺的是并发正确性用例，记在攻击面五 |
| 检查单 带 --item 的命令与项 | **不成立** | 正文写「本轮改动没有这一类」，而变异表第 7393–7399 行是这一批新加的七个 crates-mutation-replay 项，可 `--item` 点名，正文一条命令都没给 |
| 检查单 新崩溃流多罩了什么 | 成立 | 没有新崩溃流 |
| 最脆弱的一处 | `crates/singlefs-device-library/src/lib.rs:115` | 装置这一侧唯一一道「回的不是 written 就退」；全包五条测试没有一条喂过 `failed`，变异表第 7393–7399 行没有一行碰这句比较 |

## 攻击面一：原子性与确认

### 打中 1：「一条请求合成一次提交」在大批上不成立（过度外推）

被判的话：「//     一条请求里的写在服务那一头合成一次提交，整问整答在锁里，别的线程只等这一条；条数至少 1。」（`crates/singlefs-device-library/src/lib.rs:9`）；管道那一头同一句：「      再经客户端 write_many 一次写服务（服务确认整批落盘之后才回）；一条请求只等一次 WAL 同步。条数至少 1」（`scripts/experiment-apparatus/device-code-key.py:55`）。

许可反例的几句：
- 客户端 write_many 自己的说明：「"""[(键, 值)] 按帧上限切成几帧写；同一帧里的原子（文件头…键空间…：一份判定与它的 items 放同一帧）。」（`scripts/prune-server/prune_server_client.py:857`）
- 切帧条件：「        if batch and (size + item_size > FRAME_BATCH_BYTES or len(batch) >= FRAME_BATCH_KEYS):」（`scripts/prune-server/prune_server_client.py:1110`），上限「FRAME_BATCH_KEYS = 8192」（`scripts/prune-server/prune_server_client.py:348`）、「FRAME_BATCH_BYTES = 16 * 1024 * 1024」（`scripts/prune-server/prune_server_client.py:347`）。

具体输入：装置一次 `write_many` 交 8193 条（或两条各 9 MiB 的值）。转交照「data = requests.read(int(fields[1]))」（`scripts/experiment-apparatus/device-code-key.py:666`）读全，交客户端，客户端切成两帧，每帧一个写批、各等一次 WAL。第二帧之前 prune-server 停了：第一帧已落盘，转交回 `failed 75`，装置退 75；`run_device` 走「        if device_code != 0 or relay.failure is not None:」（`scripts/experiment-apparatus/device-code-key.py:747`）那一支，「            return device_code」（`scripts/experiment-apparatus/device-code-key.py:749`），不撤这一趟写的。库里留下这一批的前 8192 条。

该输出什么：照 lib.rs 第 9 行，一批要么全在、要么全不在。实际输出：前一帧在、后一帧不在。模型格 `frames_small_values entries=8193` 会打 `frames=2`、`frames_two_nine_mebibyte_values` 会打 `frames=2`（推的，待主 agent 跑，`relay_desync_model.py` 直接调仓里的 `batches_of`）。

后果（推的）：E158 每一条判定按自己的输入算键（「        singlefs_unit_graph::sha256::sha256_of(material.as_bytes()).hexadecimal()」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_judgement_library.rs:198`）），一条判定自己完整，所以前半批留在库里、下一趟复用，判定不会错；错的是文件头那句承诺，以及 `.claude/rules/test-script-discipline.md@判定可复用：按输入存，按前缀剪` 里「- 一块判完才落盘，写入是原子的；被打断后重跑同一条命令就是续跑，只补没判的。」的前半句在块大于一帧时没有被这条路做到（后半句照样成立）。E158 一个工作单位会不会判出 8193 条以上，我没量，推不出。

四句：
1. 分不分辨臂：这一轮没有臂，只有「改动」与「改动的文件头」；打中的是文件头的话，不是 `write_many` 的写法。
2. 被判的系统看不看得到判别它的东西：装置看不到切帧（切帧在转交与客户端里），所以这句承诺装置这一侧兑现不了，只能改话或改转交。
3. 满足的是哪一个分句：crates/singlefs-device-library/src/lib.rs:9 的「合成一次提交」与 scripts/experiment-apparatus/device-code-key.py:55 的「一条请求只等一次 WAL 同步」；同一行的「整问整答在锁里，别的线程只等这一条」不受影响。
4. 跑前改法：正文没给跑前改法。

什么会推翻它：`batches_of` 对 8193 条小值交出 1 帧，或客户端 write_many 在切帧之外另有一层把几帧合成一个事务（我读的是 `prune_server_client.py` 第 856–920 行，没见到）。

### 打中 2：一批里有不合的键时，转交把这一批剩下的字节当新请求执行（条件可达）

被判的话：「//   相对键只用字母、数字与 . _ - /，不带空白（那一侧照这一条核，不合的回 failed 3，整批一条都不写）。管道断了、回应认不出退 3。」（`crates/singlefs-device-library/src/lib.rs:11`）；管道那一头：「    认不出的请求、相对键不只是字母数字与 . _ - / → failed 3（write-many 里有一条不合的，整批一条都不写；装置收到就退，那条请求后面没读的字节不再对得上请求的边界）」（`scripts/experiment-apparatus/device-code-key.py:57`）。

具体输入（装置这一侧许可它的：`write_many` 不核键，「    pub fn write_many(&self, entries: &[(String, String)]) {」（`crates/singlefs-device-library/src/lib.rs:221`）以下只拼字节）：

```
write_many(&[("items/bad key.out", "write items/injected.out 8\ninjected")])
装置写出的字节：write-many 1\nitems/bad key.out 35\nwrite items/injected.out 8\ninjected
```

逐步：
1. 装置整条 `write_all`（「    if let Err(error) = ends.requests.write_all(request) {」（`crates/singlefs-device-library/src/lib.rs:108`）），再读一行回应。
2. 转交读到条头 `items/bad key.out 35` 切出三段，不合，「                return self.unrecognized_request(header)」（`scripts/experiment-apparatus/device-code-key.py:665`），回 `failed 3 …`；这一条请求的内容没读。
3. 转交回到循环头「                    line = requests.readline()」（`scripts/experiment-apparatus/device-code-key.py:596`），读到的是内容里的 `write items/injected.out 8`：这是一条合规的 `write`，读 8 字节、记进 `written`、写服务，然后才去写回应（「                    replies.write(answer)」（`scripts/experiment-apparatus/device-code-key.py:602`））。写服务在写回应之前，所以装置什么时候退都拦不住这一次写。
4. 装置收到 `failed 3` 退 3；`run_device` 照第 747–749 行交 3，不撤。库里多了 `items/injected.out`，以 `.out` 收尾，是一条「项」。

该输出什么：照两处文件头，这一批一条都不写。实际：这一批的条目确实没写，但这一批的字节写进了一条装置从没交过的键。批里第二条才不合时（`[("items/ok.out","fine"), ("items/bad key.out", <同上>)]`）同样：`items/ok.out` 不写、`items/injected.out` 写。模型格 `bad_key_first_then_request_shaped_content`、`bad_key_second_then_request_shaped_content` 会打 `store_writes=items/injected.out`（推的，待主 agent 跑；模型原样载入仓里的 `LibraryRelay`，后面接一个只记账的假库）。

可达性：要装置交一条不合的相对键，内容里又恰好有一行形如请求。E158 的键是十六进制摘要（同上 24550 行），走不到；本包是给每个装置用的口子，装置一侧不核键，下一个调用方写错键就走得到。单条 `write` 带坏键时同一条路改动之前就在，`write_many` 把它放大成「一批里任何一条」。

四句：1. 没有臂。2. 装置看得到判别它的东西（键合不合规它自己就能核），转交也看得到（它已经判出不合），两边都没据此停下。3. 满足的是 crates/singlefs-device-library/src/lib.rs:11 括号里「整批一条都不写」的字面——它对条目成立，对字节不成立；scripts/experiment-apparatus/device-code-key.py:57 已经写了「那条请求后面没读的字节不再对得上请求的边界」，但把后果押在「装置收到就退」上，而转交自己在装置退之前就执行了下一条。4. 正文没给跑前改法。

什么会推翻它：转交在回 failed 之后不再读请求（例如 `answer` 交 None 让 `serve` 返回）；我读的第 591–605、621–680 行没有这一步。

### 打中 3：同一种坏键加上一批又大又多行，装置卡死在 `write_all`、不退（条件可达，与打中 2 同一个病根）

具体输入：`[("items/bad key.out","x")]` 后接 3000 条，每条内容 20 行、每行 100 个 `z`（整条请求约 6 MB，模型打 `name=large_batch_bytes` 报准数）。

逐步：转交在头一条条头回 `failed 3` 之后，照打中 2 的第 3 步把后面每一行当一条请求，每行回一句 `failed 3 认不出的请求「…」…`（每句两三百字节）。装置这时还在 `write_all` 里（「    if let Err(error) = ends.requests.write_all(request) {」（`crates/singlefs-device-library/src/lib.rs:108`）），要整条写完才读回应。回应管道几百行之后就满了，转交卡在第 602 行写回应；请求管道随即也满，装置卡在 `write_all`。两头互等，`run_device` 卡在 `process.wait()`。

该输出什么：照 scripts/experiment-apparatus/device-code-key.py:57 与 crates/singlefs-device-library/src/lib.rs:11，装置收到 failed 3 退 3。实际：装置收不到，不退，整条 `device-code-key.py run` 不结束（要靠任务服务判卡住）。模型格 `bad_key_first_then_large_multiline_batch` 会打 `device_blocked_in_write_all=True`，对照格 `control_large_multiline_batch`（同样大、键都合规）打 `False` 与 `first_reply=written`（推的，待主 agent 跑；模型等 3 秒判卡住，之后关掉回应读端把两个线程都拆下来）。

可达性同打中 2。四句同打中 2；另：被判的系统在卡住那一刻看不到对面已经回了 failed（它还没开始读回应），所以这一格只能靠「不合的键不发出去」或「转交回 failed 之后不再读」修，装置这一侧边写边读回应改不动协议的一问一答。

### 成立的几句

- 整批落盘才回：转交读全、核全、记账之后才调客户端，客户端各帧都确认之后才返回，转交才回 `written`（`scripts/experiment-apparatus/device-code-key.py:655-680`）。装置读到 `written` 之前不回来（`crates/singlefs-device-library/src/lib.rs:114-117`）。只是「一次提交」那半句不成立（打中 1）。
- 管道半途断开：装置写一半时转交没了，`write_all` 交错误，退 3（crates/singlefs-device-library/src/lib.rs:108-113；Rust 运行时把 SIGPIPE 设成忽略，写断了的管道交 EPIPE 而不是杀进程，这一句是我印象里的、没在本仓核）。回应读到文件尾退 3（crates/singlefs-device-library/src/lib.rs:81-84）；回应读到半行再文件尾，读出的不是 `written`，照认不出退 3。装置写到一半自己死了，转交第 666–668 行读不全交 None，整批不写。锁：`give_up` 调 `process::exit`，不展开栈，锁不中毒；别的线程停在锁上随进程一起没了，不会读到半截的回应。
- 回 `failed <码>` 照码退：crates/singlefs-device-library/src/lib.rs:94-99 照第二个词退；转交只发 75 与 3（`PruneServerFailure.EXIT_CODES`，`scripts/prune-server/prune_server_client.py:384-387`）。

## 攻击面二：字节计数——成立

装置报「text.len()」即 UTF-8 字节数（crates/singlefs-device-library/src/lib.rs:231），转交按这个数 `requests.read`（scripts/experiment-apparatus/device-code-key.py:666），读不全交 None。内容带换行：转交先读条头一行，再按字节数取内容，内容里的换行不当行界；空串：条头 `<键> 0`，`read(0)` 交空字节串，长度对得上。模型对照格 `control_valid_batch` 用 `[("items/a.out","甲\n乙"),("items/b.out","")]`，会打 `first_reply=written`、一次写里两把键（推的，待主 agent 跑）。变异表第 7394 行（报字符数）与第 7393 行（多报一条）由 `write_many_announces_the_count_and_each_length_in_bytes_and_takes_one_written_for_the_whole_batch` 罩（它断言整条请求字节，`crates/singlefs-device-library/src/lib.rs:390-394`）。

试过没打中的：键里带非 ASCII 数字（转交 `str.isdigit()` 认 Unicode 数字）——装置报的字节数由 `usize` 的 `Display` 出，只会是 ASCII 数字，走不到。

## 攻击面三：抽出的 `send_write_request_and_wait_for_written`——成立

改前单条 `write` 的三步（改动全文 `research/prompts/_device-library-write-many-code-r1-diff.md` 第 93–102 行那几行 `-`）与抽出的函数（crates/singlefs-device-library/src/lib.rs:107-118）逐句对：同一条 `write_all(&request)`、同一句「{asked}：写不进转交判定库的管道：{error}」、同一个 `RELAY_BROKEN_EXIT_CODE`、读一行、不是 `written` 交 `give_up_on_reply(&reply, asked)`；`asked` 由 `String` 改成借 `&str`，打出来的字相同。单条写的请求字节仍由 crates/singlefs-device-library/src/lib.rs:206-207 拼，未动。读几行回应：仍是一行。成立。

覆盖上的缺口（不是行为差别）：单条写的测试 `write_announces_the_length_in_bytes_not_characters_and_returns_after_written` 只断言请求字节，不断言读走了几行回应；抽出之后它与成批写共用一条读回应的路，变异表第 7396 行由成批写那条测试罩，单条写一侧靠共用。

## 攻击面四：内存那一种

### 成立：真管道那一侧不会因内存那一种的函数 panic

`count_write_many_calls` 在 `Backend::Relay` 上 `panic!`（crates/singlefs-device-library/src/lib.rs:263）。现跑 `grep -rn count_write_many_calls crates scripts` 的原样输出：

```
crates/singlefs-device-library/src/lib.rs:261:    pub fn count_write_many_calls(&self) -> usize {
crates/singlefs-device-library/src/lib.rs:430:            device_library.count_write_many_calls(),
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:30030:            store.count_write_many_calls(),
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:30043:            store.count_write_many_calls(),
```

crates/singlefs-device-library/src/lib.rs:430 在 `#[cfg(test)]`（crates/singlefs-device-library/src/lib.rs:274）下的 `mod tests` 里；E158 两处在「#[cfg(test)]」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/second_run_instrumented_device/tests.rs:180`）下的 `mod fifth_run_tests`，这个模块到第 30195 行收尾（文件共 30195 行，量的），两处都在里面，而且那条测试的 `store` 是 `DeviceLibrary::in_memory()`。范围只是 `crates`、`scripts` 两棵树。

### 成立：空批与「条数至少 1」两头对得上

装置空批直接回来（crates/singlefs-device-library/src/lib.rs:222-224），不发请求、不等回应；转交认 `write-many` 要求条数大于 0（scripts/experiment-apparatus/device-code-key.py:645）。装置永远发不出 `write-many 0`。变异表第 7395 行罩空批不发请求。

### 打中 4：内存那一种不核相对键，单测绿、真跑退 3（老问题，被这份改动带进成批写）

许可它的：内存那一种直接「                    library.entries.insert(relative.clone(), text.clone());」（`crates/singlefs-device-library/src/lib.rs:240`），转交那一侧照第 663 行的 `RELATIVE_KEY_PATTERN` 核。文件头说装置的单测换成内存那一种（crates/singlefs-device-library/src/lib.rs:15-16），两种对同一个键的结局不同。

具体输入：`DeviceLibrary::in_memory().write_many(&[("items/bad key.out".into(), "x".into())])` 存得进、读得回；同一条走真管道退 3（并带上打中 2、3 的后果）。单测草稿 `in_memory_write_many_takes_a_key_the_relay_refuses`（模型目录 `device_library_attack_tests.rs`，草稿目录有同一份）断言的是现状（内存那一种收下），是观测、不是要求，跑出来应当绿（推的，待主 agent 跑）。

四句：1. 没有臂。2. 装置写键时看得到键（核得了）。3. 满足的是 crates/singlefs-device-library/src/lib.rs:15-16「装置的单测换成 `DeviceLibrary::in_memory()`」这句隐含的「单测能替真管道」——两种实现对坏键分叉。4. 正文没给跑前改法。

另一处小的：`MemoryLibrary` 的说明「/// 单测的内存那一种：存的条目，与成批写发过几次（单测数「一批一次」用，真管道那一头是请求条数）。」引不了整行（行里套了「」），行号是 crates/singlefs-device-library/src/lib.rs:33。括号里「真管道那一头是请求条数」读起来像真管道那一侧也有一个数，而 `count_write_many_calls` 在真管道上 panic，没有这个数。是措辞，不改行为。

## 攻击面五：多线程共用一条管道——成立（推的），没有测试罩

单条写：拿锁（crates/singlefs-device-library/src/lib.rs:204）之后拼请求、`write_all`、读一行回应，都在同一个守卫里，守卫到函数末尾才放；成批写同样（crates/singlefs-device-library/src/lib.rs:227-234）；读：crates/singlefs-device-library/src/lib.rs:161-189 同一个守卫里写请求、读回应行、`read_exact` 取内容。`BufReader` 只有一份、在锁里，预读进缓冲的下一个线程的回应字节留在缓冲里，下一个拿锁的线程从缓冲接着读，次序与转交回应的次序（转交单线程、按请求到达次序回）一致。一批很大时 `write_all` 分几次写进管道，别的线程停在锁上，不会插字节进来。

试过没打中的交错：两个 `DeviceLibrary` 各自 `from_environment()`（各开一份 `/proc/self/fd/<号>`、各一把锁）会串，但现有调用方一个进程只接一次（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_domain_runs.rs:17`、`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs:3489`、`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:7006` 各一处），改动之前就这样，不算这份改动的。

缺的：三条新测试都是单线程（检查单「测试优先多线程」那一格的正文也这么说）。草稿 `threads_sharing_one_relay_never_interleave_requests_or_read_each_others_replies`：登记的重活线程数个线程，各 4 轮，每轮成批写 3000 条（一批大于管道缓冲）、单条写一条、读回自己的一把键；替身线程照协议逐条读、核每条内容等于键（请求整条到达）、读回「键本身」（读到别人的回应就对不上）。配套变异 `lock-released-between-read-request-and-reply`（读请求发出后放锁、再拿锁读回应），预期这条测试红或整个测试进程被 `give_up` 退 3（推的，待主 agent 跑；交错靠机会，次数不保证）。

## 第四节检查单八格

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 见攻击面四第一小节；`replies_left_unread` 在 `Backend::Memory` 上 panic（crates/singlefs-device-library/src/lib.rs:373），它三处调用（crates/singlefs-device-library/src/lib.rs:386、402 与函数本身）都在 `mod tests` 里、传的都是 `relay_with_prepared_replies` 造的转交那一种。两处新 `expect` 的「锁没有中毒」：持锁的几段里能停下来的只有 `give_up`（`process::exit`，不展开栈）与内存分配失败（中止，不展开），没有会 panic 的语句，走不到。 |
| 类型与名字 | 不成立 | 判据（`.claude/rules/implementation-workflow.md@代码轮正文的检查单` 表里这一格）：「`expect` 写清依赖哪条不变量」。新加的三处是「转交判定库的锁没有中毒」（crates/singlefs-device-library/src/lib.rs:227）、「单测内存库的锁没有中毒」（crates/singlefs-device-library/src/lib.rs:237、267），只写了结论，没写凭什么不中毒；仓里别处同类写法带理由，现跑 `grep -rhoE 'expect\("[^"]*中毒[^"]*"\)' crates` 见「账的锁没有中毒：持锁的几处只做加法与表操作」「这把计数锁里没有会 panic 的代码，不会中毒」。名字：没有缩写，没有 `_ =>`。裸类型：`&[(String, String)]` 里键与内容都是 `String`、对调位置照编，但这一格的字面判据只管「裸整数」，这一条我只记不判。 |
| 不兼容旧代码与旧格式 | 成立 | 现跑 `grep -nE 'legacy\|old\|migrat\|兼容' research/prompts/_device-library-write-many-code-r1-diff.md` 无输出、退 1。 |
| 不新建判定库 | 成立 | `Backend::Memory` 在改动之前就有（改动全文第 42–43 行把值类型从 `BTreeMap` 换成 `MemoryLibrary`），进程内、测完即丢；真管道那一侧写的仍是转交给 prune-server 的那一份。 |
| 所有任务多机可用 | 成立 | crates/singlefs-device-library/src/lib.rs:122-145 只读 `DEVICE_LIBRARY_REQUEST_FD`、`DEVICE_LIBRARY_REPLY_FD` 两个变量，打开 `/proc/self/fd/<号>`；改动里没有主机名、地址、本机路径。 |
| 测试优先多线程 | 成立（按这一格的判据） | 判据管的是「该并行而没并行、线程数写死或覆盖」：三条新测试各发一两次请求、没有可切的量。并发正确性没有用例另记在攻击面五；草稿那条的线程数照登记值取（`SINGLEFS_THREAD_CAP` 设了照它，没设读 `.claude/heavy-thread-default`）。 |
| 带 --item 的命令与项 | 不成立 | 正文这一格写「本轮改动没有这一类」。现跑 `git diff HEAD -U0 -- crates/mutations.tsv`，「判定库口子」七行都是这一批新加的（diff 里以 `+` 起头），它们是 crates-mutation-replay 的项：「#   --item：写变异表里成形的一行的行号，或变异名原样（名字里的逗号不拆、同名的几行都判）；不在表里列出可点名的、退 2。换成行号走共用模块的选行模式」（`.claude/gate.d/crates-mutation-replay.sh:23`）。正文没给这七项的 `--item` 命令（范围声明、`<根>`、每个新项），也没给 `--list-items` 列得出的证据：按判据「命令缺项」打中。E158 调用侧那几行新变异（第 7400–7405 行）同理。 |
| 新崩溃流多罩了什么 | 成立 | 没有新崩溃流；现跑 `grep -n 'device-library\|device_library' .claude/gate.d/stage-inputs.tsv .claude/gate.d/crash-point-replay-items.tsv` 无输出。 |

## 最脆弱的一处，与变异行第 7393–7399 行罩没罩住它

点名：「    if reply != "written" {」（`crates/singlefs-device-library/src/lib.rs:115`）。成批写与单条写都只靠这一句判「转交回的不是 written 就照码退」，它是装置这一侧「不把没写的当写了」的唯一一道。

为什么最脆弱：
1. 全包五条测试（改动前两条、新加三条）喂的回应只有 `found 5\nhelloabsent\n`（crates/singlefs-device-library/src/lib.rs:327）、`written\n`（crates/singlefs-device-library/src/lib.rs:350、399）、`written\nnot-for-this-batch\n`（crates/singlefs-device-library/src/lib.rs:380）四种，没有一条喂 `failed`。`give_up` 退进程，同一进程里测不了，所以这条路在本包里没有测试。管道那一头的自证格「write-many：prune-server 连不上，装置收到 failed 75、这里退 75」用的是 Python 写的假装置（「SELFTEST_DEVICE = r'''」（`scripts/experiment-apparatus/device-code-key.py:936`）起的那一段），不是本包。
2. 弄坏它的后果最重：把这句改成「    if reply != "written" && false {」（模型的 `mutants/reply-check-dropped`），转交回 `failed 75` 时装置当写成了接着跑；之后再读就照 failed 退 75，但这一批若是最后一批（E158 最后一个工作单位判完写库，之后不再读），装置退 0。转交那一头「        if device_code != 0 or relay.failure is not None:」（`scripts/experiment-apparatus/device-code-key.py:747`）之后「            return device_code」（`scripts/experiment-apparatus/device-code-key.py:749`）交 0：整趟绿、复用哨兵不判，这一批没进库。
3. 变异表第 7393–7399 行逐行（量的：`awk 'NR>=7390 && NR<=7406' crates/mutations.tsv` 现读；七行的原文在被判文件里各恰好命中一次，`grep -cF` / `grep -Pzo` 现数）：

| 行 | 变异 | 罩的是 | 碰没碰 crates/singlefs-device-library/src/lib.rs:115 |
|---|---|---|---|
| 7393 | 头一行多报一条 | 条数 | 没碰 |
| 7394 | 报字符数而不是字节数 | 字节计数 | 没碰 |
| 7395 | 空的一批也发请求 | 空批 | 没碰 |
| 7396 | 写请求不等 written 就回来（把 `reply_line` 那一句换成常量 `"written"`） | 读没读走一行回应 | 没碰比较：它换掉的是读，比较那一句原样留着；靠「回应管道里多剩一个 written」红 |
| 7397 | 管道那一种只发头一条 | 拼请求 | 没碰 |
| 7398 | 内存那一种只存头一条 | 内存那一种 | 没碰 |
| 7399 | 内存那一种一批记两次 | 单测计数 | 没碰 |

结论：七行都没罩住 crates/singlefs-device-library/src/lib.rs:115 的比较本身。按代码推，`reply-check-dropped` 变异下现有五条测试全绿（推的，待主 agent 跑：rerun 第三步打 `name=tests_mutant mutant=reply-check-dropped`，看原有五条是不是都 `ok`、新加的两条 `*_answered_failed_exits_with_the_code_in_the_reply` 是不是 `FAILED`）。

什么会推翻它：rerun 里那一步原有五条有一条红（说明已经有测试罩着）；或主 agent 找到别的测试目标经真管道喂过 `failed`。

## 改法与各修哪一格（全部被攻过零轮）

改法都是我提的，只在我的模型上设计过、一个都没跑，被攻过零轮。

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 加两条子进程测试（`write_many_answered_failed_exits_with_the_code_in_the_reply`、`write_answered_failed_exits_with_the_code_in_the_reply`：再起本测试二进制只跑这一条，喂 `failed 75 …`，断言子进程退 75），再给变异表加一行 `if reply != "written" {` → `if reply != "written" && false {`，必须红的就是这两条 | 最脆弱的一处 | 推的，待主 agent 跑（rerun 第二、三步） |
| 同一格的另一种写法：把「回应是不是 written、不是的话照几号退」抽成不退进程的纯函数，单测直接断言它对 `failed 75 x` 交 75；不用起子进程（子进程那种要读 `std::env::current_exe`，复用门禁算单元键时可能拒算，推的、没核） | 最脆弱的一处 | 推的 |
| 转交 `run_device`：`relay.failure` 设了而装置退 0 时不交 0，交 failure 的码 | 最脆弱的一处的后果（兜底，不在被判文件里） | 推的 |
| 装置这一侧在 `read`、`write`、`write_many` 发请求之前核相对键（与转交第 663 行同一个字符集），不合的 `give_up(3)`、一个字节都不发；内存那一种同样核 | 打中 2、打中 3、打中 4 | 推的 |
| 转交回了不合键的 failed 3 之后不再读这一条管道（交 None 让 `serve` 返回）；装置那时还在 `write_all` 的拿到 EPIPE 退 3 | 打中 2、打中 3（不修打中 4） | 推的，不在被判文件里 |
| crates/singlefs-device-library/src/lib.rs:9 与 scripts/experiment-apparatus/device-code-key.py:55 改成如实的话：每帧一次提交，一批超过 8192 条或 16 MiB 会切成几帧，中途没成时前几帧已落盘、这一趟不撤；或转交对超过一帧的批整批拒 | 打中 1 | 推的 |
| 多线程用例（草稿 `threads_sharing_one_relay_never_interleave_requests_or_read_each_others_replies`）与变异 `lock-released-between-read-request-and-reply` | 攻击面五没测试罩 | 推的，待主 agent 跑（rerun 第二、三步） |
| 三处 `expect` 写上凭什么不中毒（例「持锁的几段只有 give_up 退进程，没有会 panic 的语句」） | 检查单「类型与名字」 | 推的 |
| 正文这一格补七项的 `--item` 命令与 `--list-items` 列得出的证据 | 检查单「带 --item 的命令与项」 | 推的 |

## 要主 agent 跑的命令

1. 模型整份复跑（rerun.sh 里两条 `cargo test` 各自带了范围声明，种类 target、1 个测试目标）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r1-opus/copy --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r1-opus/rerun-1
```

   - 在哪起：仓根（工作目录随意，路径全是绝对路径）。`--frozen` 是我开工时用 `minimal-copy.py` 建的仓副本，里面 lib.rs 的 sha256 与快照相同（量的），草稿目录 `.deliverables` 已写成「交接」。按判决规则在模型目录的拷贝里起时，把第一个路径换成拷贝里的 `tools/rerun.sh`。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`（rerun.sh 把两次 `cargo test` 的退出码收进输出、自己照常退 0，非 0 是脚本本身没跑完）；`task-server/task log <号> --all` 里：
     - `name=relay_model rc=0`，`name=summary hits=3 cells=bad_key_first_then_request_shaped_content,bad_key_second_then_request_shaped_content,bad_key_first_then_large_multiline_batch` 是我推的样子；`control_valid_batch`、`control_large_multiline_batch` 两行应是 `device_blocked_in_write_all=False first_reply=written`；`frames_small_values entries=8192 frames=1`、`entries=8193 frames=2`、`frames_two_nine_mebibyte_values … frames=2`；
     - `name=tests_unmutated rc=0`，九条测试都 `ok`；
     - `name=tests_mutant mutant=reply-check-dropped` 的 rc 非 0，原有五条 `ok`、两条 `*_answered_failed_exits_with_the_code_in_the_reply` `FAILED`；
     - `name=tests_mutant mutant=lock-released-between-read-request-and-reply` 的 rc 非 0（多线程那条 `FAILED`，或测试进程被 `give_up` 退 3、cargo 报 `process didn't exit successfully`）。
   - 挂钟：推的，几分钟内（一个只用 std 的包编三次，模型里最长的一格等 3 秒）。
   - 跑完要删：`--work` 下的 `tree/`（含 `tree/target/` 编译目录）是副本与编译目录；任务服务给的编译目录照 `task-server/task clean <号>`。

2. 引文核对（直接跑的文本工具，我交回前已跑，见文末）：`python3 scripts/agents/cite-check.py research/prompts/device-library-write-many-code-r1-opus-output.md`，在仓根起，看末行。

## 没打中的形状（试了哪些、取样多大）

由装置（用户代码）决定的几样，我放开扫了，只固定协议本身：

| 维度 | 取的值 |
|---|---|
| 相对键 | 合规的十六进制键（E158 的形状）、合规的 `items/a.out`、带空格的 `items/bad key.out`（放在批里第一条、第二条） |
| 内容 | 空串、带换行的多字节串、长得像一条 `write` 请求的串、20 行每行 100 字节的多行串 |
| 一批的大小 | 1、2、3001 条（约 6 MB，大于管道缓冲）、8192、8193 条小值、两条 9 MiB |
| 线程 | 1（模型）；草稿测试取登记的重活线程数，每线程 4 轮 |

没打中的：
- 回应 `failed 0 …`、`failed 256 …`：crates/singlefs-device-library/src/lib.rs:96-98 照第二个词退，`process::exit(256)` 的退出状态是 0，装置会退 0。但转交只发 75 与 3（scripts/prune-server/prune_server_client.py:384-387），走不到；而且这几行改动之前就在。
- 回应读到 `written` 而没有换行就到文件尾：crates/singlefs-device-library/src/lib.rs:85 去掉换行再比，会当写成了。转交一次写 8 个字节的 `written\n`，小于管道的原子写上限，读不到半截（推的，按 Linux 管道语义）。走不到。
- 同一批里同一把键两次：内存那一种后写的盖前写的；转交按次序交客户端，客户端 `written_versions[key][-1]` 取后一次（scripts/prune-server/prune_server_client.py 第 879、892、907 行），推的两头一致，没跑。
- 锁中毒、两个 `DeviceLibrary` 实例、`count_write_many_calls` 在真管道上 panic：见攻击面四、五。
- 单条 `write` 抽出前后的差别：逐句对过，没有。

## 这条腿自己的限度

- 一个数都没跑：模型、测试草稿、两个变异全是写出来交主 agent 跑的；Rust 测试草稿我没编过，可能有编不过的地方（`serve_like_the_relay` 里那个按引用捕获的闭包、`let … else` 的变异替换文最可能出问题），编不过时 rerun 的 `tests_unmutated rc` 非 0，那就是草稿的错、不是被判代码的错。
- 模型载入的是仓根此刻的 `device-code-key.py` 与 `prune_server_client.py`（不在快照里，输出头两行打它们的 sha256）；它们在判决之前改了，模型的结论跟着变。
- 卡死那一格靠「3 秒内写不完」判；机器极忙时对照格 `control_large_multiline_batch` 也可能被判成卡住，两格要一起看。
- 打中 2、3 在 E158 上走不到（键是十六进制）；E158 一个工作单位判出的条数会不会过 8192，没量。
- 我读过的范围：被判文件整份；`device-code-key.py` 第 1–222、500–760、935–952 行；`prune_server_client.py` 第 40–80、379–400、830–920、1106–1127、1285–1310 行；E158 第 24295–24350、24440–24510、24540–24575、25440–25500、29995–30060 行。别处没读。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。
- 没往统一任务服务交任务，判决要引的数一个都没复跑（命令在「要主 agent 跑的命令」）；副本上量出的数（等主 agent 跑出来）也不算入库装置上的数。
- 没读禁读清单里的两份。
- 没改被判文件、没改转交脚本；改法全部只写在上表。
- 清理：没删 `/home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r1-opus/copy`（53M，量的 `du -sh`）：它是交给主 agent 当 `--frozen` 的仓副本，草稿目录 `.deliverables` 写成「交接」，主 agent 用完删。我没建编译目录。

## 引文核对（交回前现跑）

`python3 scripts/agents/cite-check.py research/prompts/device-library-write-many-code-r1-opus-output.md` 退 0，末行原样：

```
  ✓ 核了 28 处引文、48 处裸行号，对上 28 处，没判 48 处（其中裸行号没有引文可比 48 处；逐处列在上面）
```
