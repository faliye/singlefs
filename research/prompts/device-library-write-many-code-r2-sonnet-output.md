# device-library-write-many-code-r2 辩方腿报告（2026-11-10）

立场：辩方。复核对象：`research/prompts/device-library-write-many-code-r1-main-verification.md` 第二、三节，与背景材料 `research/prompts/_device-library-write-many-code-r2-background.md` 第四节八格。没有承接的前一份报告。
读数标记：「量的」是我自己用只读命令（grep、awk、读文件）在此刻主工作区量的；「推的」是我读代码推的，或来自主 agent 在背景材料里写的读数；模型的输出一行都没跑出来，全部「推的，待主 agent 跑」。

## 模型与复跑

模型目录（草稿目录里，不入仓）：`/home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-sonnet/model`。只读仓里的转交脚本与 prune-server 客户端，不起装置、不连服务、不写仓、不起子进程。

```
sha256  367a00e659c26fe6a7736db16c7f345ea763738f4ec9f7895a978bf57747523c  model/probe.py
sha256  d6dc3842a14374e478d6c6225ad712261fd22f61c9368b0ee8a0ebe70df21355  model/tools/rerun.sh   （从 scripts/agents/three-way-rerun-template.sh 拷，只改了 rerun_steps）
```

复跑命令写在文末「要主 agent 跑的命令」一节。`probe.py` 三件事：P0 转交一侧相对键的字符表是不是恰好是 ASCII 字母、数字与 `. _ - /`；P1 一条 `read` 请求的键里带换行时转交把后面的字节当不当请求办；P2 客户端 `write_many` 在几种一批里发几帧。

## 一、结论一览

1. 第二节五处「打中」：五处都站得住，没有一处要撤。两处要带限定：「一次提交」那一条是文字上的打中，「E158 的实际批量下只差措辞」这句话推不出（最大一个单位的条数没有产物）；`expect` 那一条要取严读法才成立，字面读法下它只是缺了「凭什么」。
2. 第三节五条改法：第 1 至 4 条修到了点名的那一格；第 5 条里的反向测试与内存键测试修到了，多线程测试只修到一半（抓到一次、没有第二次样，且变异表的复用规则会把一次抓到记成永久抓到）。
3. 改法比机制窄的两处：第 1 条只核 `write`、`write_many`，`read` 的键同样原样拼进请求；第 5 条的多线程测试只罩「请求与 absent 回应之间」，不罩 `found` 行与内容之间。改法比点名宽的一处（`found` 长度、`give_up` 写标准错误）站得住。
4. 第四节八格：七格成立，「带 --item 的命令与项」一格推不出（`--list-items` 我没跑，命令在文末）。
5. 判决没够到的邻格四处写在第五节，每处标了是量的还是推的。

## 二、第二节逐条（r1 判决「逐格判」表）

行号全是此刻主工作区的；r1 判决引的行号是当时那一版的，对不上的以此刻为准。

| 判决条目 | 辩方判定 | 证据 | 什么现象会推翻它 |
|---|---|---|---|
| 文件头协议与服务侧约定（打中 1：「一次提交」过度外推） | 站得住，是文字上的打中。「E158 的实际批量下只差措辞」这句话推不出。 | 客户端自己写「每帧一个写批，原子性只在一帧之内」（`scripts/prune-server/prune_server_client.py:346`），`write_many` 文档「按帧上限切成几帧写」（`scripts/prune-server/prune_server_client.py:857`）；服务端把几帧合进一个写批、同步一次，见「写走组提交」（`prune-server/src/storage.rs:22`），所以「一条请求只等一次 WAL 同步」还有第二个站不住的理由。E158 一批里每条都是自带头行、可独立读回的键（「key judgement_key=」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_judgement_library.rs:99`）），打断后只补没判的（「一块判完才落盘，写入是原子的；被打断后重跑同一条命令就是续跑，只补没判的。」（`.claude/rules/test-script-discipline.md:63`）），多帧落下一个前缀不会读错，行为上近乎只差措辞；但规则字面是「写入是原子的」，一个单位超过 8192 条就不满足字面。一个单位多少条没有数：运行参数行会打「units={}」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_domain_runs.rs:35`），而 `research/results/` 下 e158 产物里 `grep -l 'cut_depth\|units='` 0 个文件（量的） | 量到 E158 最大单位条数 ≤ 8192 且值都小：「只差措辞」成立、判决的打中降成纯文字。量到 > 8192：字面违反「写入是原子的」，打中升级为行为 |
| 攻击面 1 原子性与确认（打中 2、3：坏键之后把剩下的字节当请求读、大批两头互等） | 站得住。「单条 write 键不合时同一个病」也对。 | 修补前后的分叉点就是 `serve` 里认不出之后接着读还是停：此刻是回完就 return，只有弄坏开关才继续（「if break_is_set("relay-reads-on-after-refusal"):」（`scripts/experiment-apparatus/device-code-key.py:621`））；`write` 与 `write-many` 的坏键走同一个出口（「return self.unrecognized_request(line)」（`scripts/experiment-apparatus/device-code-key.py:671`）、「return self.unrecognized_request(header)」（`scripts/experiment-apparatus/device-code-key.py:690`））。「E158 的键是十六进制走不到」成立：键是「items/{key}.out」（`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:24447`），key 是 sha256 的「.hexadecimal()」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_judgement_library.rs:198`）。判决引的 `store_writes=items/injected.out`、`device_blocked_in_write_all=True` 是主 agent 任务 12060 的读数，我没复跑 | 在 r1 那一版转交脚本上跑坏键在前、内容像 write 的一批，库里没有 `items/injected.out`、装置不卡在写请求上 |
| 攻击面 2 字节计数、攻击面 3 抽出的发请求函数（成立） | 站得住 | 变异行仍在：「判定库口子 write-many 每条报字符数而不是字节数」（`crates/mutations.tsv:7394`）、「判定库口子 写请求不等 written 就回来」（`crates/mutations.tsv:7396`） | 这两行在此刻的 lib.rs 上锚点失效（我量过锚点各命中 1 次，见第四节格 7） |
| 攻击面 4 内存那一种（打中 4） | 站得住；内存那一种是测试替身，替身比真身宽就是替身不忠 | 「装置只在单测里调它」（`crates/singlefs-device-library/src/lib.rs:19`）；修补后两种后端过同一个谓词，调用在拿锁与分支之前（「give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:235`）、同一句在 `write_many` 的循环里（`crates/singlefs-device-library/src/lib.rs:260`））。有一个更窄的挡法：内存那一种 `panic!` 只让那一条测试红，`exit(3)` 会带走整个测试进程；但本包一贯用 `give_up`，判决选的是同一风格，不算错 | 找到一处装置单测用了真管道那一侧会拒的键而且单测绿、真跑不绿 |
| 攻击面 5 多线程（成立、欠测试） | 站得住；判决把它放进第三节第 5 条处理，前后一致 | 攻方那条测试在「锁在读请求与读回应之间放过」变异下 `rc=0` 是主 agent 任务 12060 的读数；此刻第三节第 5 条的新测试见第三节 | 同第三节第 5 条 |
| 检查单：类型与名字（打中：三处 `expect` 没写凭什么不中毒） | 站得住，但要取严读法；字面读法下只是「写了不变量名、没写凭什么」 | 规则那一格写「`expect` 写清依赖哪条不变量，有一处不是就是打中」（`.claude/rules/implementation-workflow.md:36`）。r1 的 `expect` 写了「锁没有中毒」，字面上点了名。严读（要写凭什么不中毒）有先例：全仓 `expect(".*中毒` 38 处（量的：`grep -rn 'expect(".*中毒' --include=*.rs crates scripts`），其中 7 处带「中毒：」后的理由，含产品代码「账的锁没有中毒：持锁的几处只做加法与表操作」（`crates/singlefs-core/src/allocator.rs:1982`），31 处不带。判决只管新加的，没有对别处双标；但判决正文没说它取了严读，建议补一句 | 主 agent 把这一格的意思定成字面读法：这一条降成建议 |
| 检查单：带 --item 的命令与项（正文写错，实测成立） | 站得住 | 新项此刻是变异表第 7590 至 7598 行；r1 的是第 6983、6984、7393 至 7399 行（「判定库口子 found 之后少读一个字节」（`crates/mutations.tsv:6983`）、「判定库口子 write-many 不核相对键」（`crates/mutations.tsv:7591`）） | 同第四节格 7 |
| 最脆弱的一处（`reply != "written"` 没有测试喂过 `failed`） | 站得住 | 那一句此刻在「if reply != "written" {」（`crates/singlefs-device-library/src/lib.rs:145`），由两条 `*_answered_failed_*` 测试与「判定库口子 回的不是 written 不退」（`crates/mutations.tsv:7594`）罩住 | 去掉那一行变异的锚点（`if reply != "written" {`）再跑，两条 `*_answered_failed_*` 测试仍全绿 |
| 其余「成立」行（走不到、不兼容、不新建判定库、多机、多线程、崩溃流） | 不争；抽查「走不到」一行我自己追了调用点 | `count_entries`、`count_write_many_calls` 在转交那一种上 `panic!`（「Backend::Relay(_) => panic!("单测只数内存那一种"),」（`crates/singlefs-device-library/src/lib.rs:285`）），调用点 E158 在 `mod fifth_run_tests`（「mod fifth_run_tests {」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/second_run_instrumented_device/tests.rs:181`））里、E174 在 `#[cfg(test)]`（`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs:3870`）之后的第 4187 行、E178 在 `#[cfg(test)]`（`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:7075`）之后的第 7693 行（量的：`grep -n 'count_entries\|count_write_many_calls'`） | 找到一处非测试代码调这两个函数 |

背景材料两处不准（正推腿报的）对不对：不争。

## 三、第三节五条改法：修到没有、比改法窄还是宽

判语：「修到」「没修到」「修过头」；窄、宽另列。

| 改法 | 判 | 证据 | 窄 / 宽 | 什么现象会推翻它 |
|---|---|---|---|---|
| 1 装置这一侧发请求前核相对键（write、write_many，两种后端，不合退 3、不发） | 修到 | 两处调用都在拿锁与按后端分支之前：「give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:235`）、同一句在循环里（`crates/singlefs-device-library/src/lib.rs:260`），拿锁在其后（「let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);」（`crates/singlefs-device-library/src/lib.rs:264`））。谓词是「!relative.is_empty()」（`crates/singlefs-device-library/src/lib.rs:91`）加按字节的 ASCII 字母数字与 `. _ - /`；转交一侧是「RELATIVE_KEY_PATTERN = re.compile(r"[A-Za-z0-9._/-]+")」（`scripts/experiment-apparatus/device-code-key.py:212`）。空串两边都拒；`..` 与开头 `/` 两边都收，且在这个存储里无害，键只是字节拼接（「return self.prefix + relative.encode("utf-8", "surrogateescape")」（`scripts/prune-server/prune_server_client.py:1284`））。两边字符表一致：推的，P0 待主 agent 跑 | 窄：`read` 不核。「format!("read {relative}\n").as_bytes()」（`crates/singlefs-device-library/src/lib.rs:195`）把键原样拼进请求，转交一侧是一行一请求，键里带 `\n` 时后面那一行会被当下一条请求（`read` 分支「if len(words) == 2 and words[0] == "read" and RELATIVE_KEY_PATTERN.fullmatch(words[1]):」（`scripts/experiment-apparatus/device-code-key.py:650`），下一行走 write 分支）。P1 第一格预测 `store_writes=['items/injected.out']`：推的，待跑。键里只有空格而无换行时转交回 failed 3、装置照退 3，无害（推的）。判决只点名 write 与 write_many，所以不是修补做错，是判决的射程比机制窄。宽：无 | P1 第一格 `store_writes=[]`：窄这一条撤；P0 `equal=False`：两边字符表不一致，这一条改回「没修到」 |
| 2 转交脚本判成认不出之后回 failed 3 就停止服务（工具员） | 修到 | 「except RefusedRequest as refused:」（`scripts/experiment-apparatus/device-code-key.py:618`）之后写回应、return；只有弄坏开关继续读（`scripts/experiment-apparatus/device-code-key.py:621`）。认不出的出口有三个，都抛同一个异常：未知请求（`scripts/experiment-apparatus/device-code-key.py:671`）、write-many 里一条头不合（`scripts/experiment-apparatus/device-code-key.py:690`）、`unrecognized_request` 自己（`scripts/experiment-apparatus/device-code-key.py:678`）。装置写卡在请求管道上时，转交退出会关掉读端，写得 EPIPE，装置按「写不进转交判定库的管道」退 3（推的，Rust 把 SIGPIPE 当错误返回） | 宽：同一批改里把 `written`、`hits` 由列表改字典（「self.written = {}   # 当有序集合用：一趟几万条，列表查找是平方级」（`scripts/experiment-apparatus/device-code-key.py:526`）），判决没点名；读代码看迭代次序与长度语义没变（撤回处「keys = list(relay.written)」（`scripts/experiment-apparatus/device-code-key.py:710`）），没跑 | 转交自证里「relay-reads-on-after-refusal」那三格在不开弄坏开关时红 |
| 3 两处文件头改成实话（整批一帧内一次提交，超过切几帧，written 在全部帧之后才回） | lib.rs 修到；转交脚本头修到点名的那一格，但新写的「一帧以内是一次提交」在两个边角上又过度外推一次 | lib.rs：「一帧（8192 条或 16 MiB）以内是一次提交，超过切成几帧各自提交」（`crates/singlefs-device-library/src/lib.rs:9`）、「整批的 written 在全部帧都确认落盘之后才回」（`crates/singlefs-device-library/src/lib.rs:10`）。转交脚本头：「以内一次提交、一次 WAL 同步」（`scripts/experiment-apparatus/device-code-key.py:55`）。边角一：服务端是组提交，几帧合一个写批同步一次（「写走组提交」（`prune-server/src/storage.rs:22`）），「一次 WAL 同步」是这一帧等一次、不是独占一次。边角二：客户端 `write_many` 在一帧之内也会多发帧——值长过一帧或以分段清单魔数起头（「return len(value) > SEGMENT_BYTES or value.startswith(SEGMENT_MANIFEST_MAGIC)」（`scripts/prune-server/prune_server_client.py:1126`）），或这把键留着登记的旧版本——先写登记键（`scripts/prune-server/prune_server_client.py:912`）、再写段（`scripts/prune-server/prune_server_client.py:913`）、最后写清单（`scripts/prune-server/prune_server_client.py:915`）。一条小值写一个留有登记旧版本的键，我读代码算是 2 帧加 1 次删前缀；P2 末格预测 `frames=2 prefix_deletes=1`（推的，待跑） | 窄：判决点名的「超过切几帧」「written 在全部帧之后才回」都改到了。E158 的值头是 `key judgement_key=…`，不会以魔数起头，边角二对 E158 走不到；边角一不影响 E158 的正确性。这是第二轮才出现的新形态，被攻过零轮 | P2 各格 `frames` 与上面推的数一致才算我读对了；P2 末格 `frames=1` 则边角二撤 |
| 4 持锁处的 `expect` 写明不会 panic 的理由 | 修到；同批的另两处改动（`found` 长度、`give_up` 写标准错误）是为了让这句理由成真，不算修过头 | 理由在「const RELAY_LOCK_IS_NEVER_POISONED: &str」（`crates/singlefs-device-library/src/lib.rs:31`），八处 `expect` 共用两条常量。逐样核持锁时做的事：拼请求、`write_all`/`read_line`/`read_exact` 都是返回 Err 再 `give_up`；`give_up` 现在是「let _ = writeln!(standard_error, "  ✗ 判定库读写没成：{sentence}");」（`crates/singlefs-device-library/src/lib.rs:80`）不再 panic，收尾「std::process::exit(exit_code)」（`crates/singlefs-device-library/src/lib.rs:86`）不展开。这两处额外改动有独立理由：退出码 75 的意思是「prune-server 不可用，重交」（「75 是 prune-server 不可用，3 是别的错」（`crates/singlefs-device-library/src/lib.rs:11`）），标准错误写不进时若 panic 会把它变成 101，调度方读到错的信号 | 窄一点：`found` 报的数 ≤ isize::MAX 而分配失败时，「let mut contents = vec![0_u8; length];」（`crates/singlefs-device-library/src/lib.rs:214`）是中止进程（SIGABRT），不是退 3；常量里写明了「分配失败是中止进程、不展开」，是明说的取舍。`try_reserve_exact` 一处同时罩住 > isize::MAX 与分配失败，可免掉 `.filter`（「.filter(|length| isize::try_from(*length).is_ok())」（`crates/singlefs-device-library/src/lib.rs:210`）），这是 std 文档的行为，我没跑，推的 | 持锁处找到一条真会 panic 的路径 |
| 5 收进攻方两条 `*_answered_failed_*` 测试与内存键测试并各配变异行；多线程测试换成抓得到「锁在请求与回应之间放过」的造法并配变异行 | 反向测试、内存键测试：修到。多线程测试：修到一半 | 变异行与测试一一对上：「判定库口子 write 不核相对键」（`crates/mutations.tsv:7590`）、「判定库口子 回的不是 written 不退」（`crates/mutations.tsv:7594`）、「判定库口子 give_up 写标准错误写不进就 panic」（`crates/mutations.tsv:7597`）、「判定库口子 读的锁在请求与回应之间放过」（`crates/mutations.tsv:7598`）。多线程那一行：主 agent 只跑了一次、抓到；规则要求「跑至少 5 轮。要说「通过」就得每轮都通过，要说「失败」就得每轮都失败；」（`.claude/singlefs-ai-sop/rules/test-discipline.md:6`）。变异复跑只复用「抓到」：「只有「抓到」与基线 ok 复用。」（`.claude/gate.d/crates-mutation-replay.sh:120`），所以一次碰巧抓到会被按行键存下来、之后不再抽样（推的，读上面这一行与第 106 行的行键组成）。检出机会靠 2 毫秒排队让装置线程睡在锁上、放锁走内核叫醒（「const QUEUEING_TIME: std::time::Duration = std::time::Duration::from_millis(2);」（`crates/singlefs-device-library/src/lib.rs:771`），共「const ROUNDS: usize = 200;」（`crates/singlefs-device-library/src/lib.rs:769`）轮），为什么放锁有一个窗口是推的（std 互斥的实现细节我没读） | 窄：测试只罩 `read` 里「请求与 absent 回应之间」；回应一律是「.write_all(b"absent\n")」（`crates/singlefs-device-library/src/lib.rs:817`），`found` 行与内容读完之间放锁没有测试也没有变异行。`write`、`write_many` 不需要：放锁要在调用方的守卫上做，而「fn send_write_request_and_wait_for_written(ends: &mut RelayEnds, request: &[u8], asked: &str) {」（`crates/singlefs-device-library/src/lib.rs:137`）借着这把守卫，借用期内放不了锁（推的，编译器保证）。宽：无 | 变异行 7598 连跑 5 轮以上每轮抓到、不变异每轮绿：「修到」；任一轮没抓到：该行是「不稳定」 |

## 四、背景材料第四节八格（成立 / 不成立 / 推不出）

| 格 | 回 | 证据 | 什么现象会推翻它 |
|---|---|---|---|
| 走不到的理由二次查实 | 成立 | 产品代码段（`crates/singlefs-device-library/src/lib.rs` 第 1 至 308 行）里 `todo!`、`unreachable!`、`assert!`、`unimplemented!`、`.unwrap(` 共 0 处，`.expect(` 8 处，`panic!` 2 处（量的：`awk 'NR<=309' … \| grep -cE`）。2 处 `panic!` 是 `count_entries`、`count_write_many_calls` 的转交分支（「Backend::Relay(_) => panic!("单测只数内存那一种"),」（`crates/singlefs-device-library/src/lib.rs:299`）），调用点全在测试模块里（见第二节末行）。8 处 `expect` 的理由逐样核过（第三节第 4 行）。唯一不是 panic 的出口是 `found` 长度在 isize 范围内的分配失败，会中止进程（SIGABRT），常量里明写了 | 持锁处找到一条真会 panic 的路径，或非测试代码调用 `count_*` |
| 类型与名字 | 成立 | 新名字：「fn give_up_unless_the_relay_accepts_the_key(relative: &str, asked: &str) {」（`crates/singlefs-device-library/src/lib.rs:98`）、`OTHER_ERROR_EXIT_CODE`（`crates/singlefs-device-library/src/lib.rs:29`）、「fn is_relative_key_the_relay_accepts(relative: &str) -> bool {」（`crates/singlefs-device-library/src/lib.rs:90`），没缩写；`_ =>` 全文件 0 处（量的）；退出码这个量只有 `OTHER_ERROR_EXIT_CODE` 一个名字，另两个码来自回应里的整数。`expect` 一项取严读法成立，字面读法也成立（第二节倒数第四行） | 主 agent 判 `OTHER_ERROR_EXIT_CODE` 与回应里的退出码是「同一个量不同语义」要 newtype |
| 不兼容旧代码与旧格式 | 成立 | 现跑背景材料给的命令 `grep -nE 'legacy\|old\|migrat\|兼容' research/prompts/_device-library-write-many-code-r2-diff.md`，命中 2 行（量的）：第 591 行是测试名里的 `holds`，第 732 行是 git 的 `old mode 100755`；没有认旧版本的分支。第 732 行带出一件事，见第五节第 3 条 | diff 里出现认旧格式的分支 |
| 不新建判定库 | 成立 | 本包 lib.rs 第 1 至 308 行里没有 sqlite、rocksdb、`File::create`、`create_dir`；唯一的 `OpenOptions` 在 `from_environment` 里开管道描述符（`crates/singlefs-device-library/src/lib.rs:164`），r1 就有；内存那一种只在单测（「装置只在单测里调它」（`crates/singlefs-device-library/src/lib.rs:19`）） | 出现按路径落盘的存放处 |
| 所有任务多机可用 | 成立（按这一格的字面判据）。检出力在第二台没量 | lib.rs 里读环境的只有两个描述符变量与子进程那一半的标记（`std::env::var_os(CHILD_HALF_VARIABLE)` 在 `crates/singlefs-device-library/src/lib.rs:512`），没有主机名、地址、`/home/`、`NFS_SHARED`（量的：`grep -nE` 见上）；起子进程用「std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"));」（`crates/singlefs-device-library/src/lib.rs:536`）。附注：多线程测试在变异下抓不抓得到取决于核数与调度，不属于这一格的判据，但派到哪一台它的检出力可能不同；命令里不绑机器，由服务分到的机器跑，若要看两台各自的读数，要主 agent 在两台各提交一次 | 在第二台上，未变异的代码跑这些测试红（判不对），或没有第二台的读数就不能说多机检出力一致 |
| 测试优先多线程 | 成立 | 新测试里只有多线程那一条是并发测试，线程数是场景的一部分（「// 线程数是场景的一部分，不是切片并行」（`crates/singlefs-device-library/src/lib.rs:762`）），不是切片并行，规则这一格问的是「该并行而没并行、线程数写死或覆盖」（`.claude/rules/implementation-workflow.md:40`）；其余八条毫秒级。七条起子进程的测试各起一个子进程、顺序等，不是并行的候选 | 找到一条分片并行该做而没做的 |
| 带 --item 的命令与项 | 推不出（算打中，同 r1 对这一格的处理） | 我量的：变异表末九行是第 7590 至 7598 行，名字全文件不重复（`cut -f1 crates/mutations.tsv \| sort \| uniq -d` 无输出），第 6983、6984、7393 至 7399、7590 至 7598 共 18 行的锚点在 lib.rs 里各命中 1 次（量的：python 读两份文件数 `src.count`）。能不能被 `--list-items` 列出、点名后抓不抓得到，我没跑。背景材料引的任务 12083 `点名 9 条…抓到 9 条` 是主 agent 的读数，且多线程那一行只跑了一次（第三节第 5 行） | 主 agent 跑 `--list-items` 列出这九行、点名跑多轮都抓到：这一格改成成立 |
| 新崩溃流多罩了什么 | 成立 | 本轮 diff 里的「崩溃」只有转交脚本里一个旧弄坏开关名 `after-key-crash-propagates` 的一行（量的：`grep -n '崩溃\|crash'` 命中第 790 行），没有新的崩溃点重放流或枚举用例 | diff 里出现新的崩溃枚举用例 |

## 五、判决没够到的邻格（辩方顺带看到的，每条标量的还是推的）

1. `read` 的键带换行：第三节第 1 行「窄」那一条。推的；P1 第一格能坐实。E158、E174、E178 的 `read` 键都是它们自己拼的 `items/<键>.out`、`codes/<键>.txt`、`index.tsv`，走不到，同 r1 对 `write` 的「走不到」。
2. 读回的值不是 UTF-8 时当「键不在」：「String::from_utf8(contents).ok()」（`crates/singlefs-device-library/src/lib.rs:221`）。文档写「读失败照退出码退，不交 None」（`crates/singlefs-device-library/src/lib.rs:187`），头里也写「不当「键不在」现判」（`crates/singlefs-device-library/src/lib.rs:11`）。这一行 HEAD 里就有（量的：`git show HEAD:crates/singlefs-device-library/src/lib.rs | grep -nF 'String::from_utf8(contents).ok()'` 命中第 165 行），r1、r2 都没碰它。所有写入口都是 `&str` 或 ASCII 撤回标记，只有库里的值被改坏才会走到：走到就是现判一次、写回，自愈但与文档矛盾。不是这两轮修补引进的，也不是这两轮判决该管的，只是「read 的回应解析在修补后有没有变」这个问题的答案：没变，旧的这一处还在。
3. 转交脚本的文件模式从 100755 变成 100644：HEAD 里是 100755（量的：`git ls-tree HEAD scripts/experiment-apparatus/device-code-key.py`），暂存区里是 100644（量的：`git ls-files -s`），工作区 `-rw-rw-r--`（量的：`ls -l`）。第一行还是「#!/usr/bin/env python3」（`scripts/experiment-apparatus/device-code-key.py:1`）。我找到的调用写法都是 `python3 scripts/…`（「python3 scripts/experiment-apparatus/device-code-key.py run . --binary e174-lease-drop-wait-memory-return」（`scripts/experiment-apparatus/replay.sh:4167`）），所以不会断；同目录里 100755 有 68 个、100644 有 416 个（量的：`git ls-files -s scripts/experiment-apparatus`），这一个原先是可执行的。多半是「写到临时文件再 mv 换上」留下的副作用，没人说明过。要不要改回，主 agent 判。
4. 撤回把这一趟写过的全部键一次 `write_many`：「relay.store.write_many([(key, WITHDRAWN_VALUE) for key in keys])」（`scripts/experiment-apparatus/device-code-key.py:714`），一趟几万条（「一趟几万条，列表查找是平方级」（`scripts/experiment-apparatus/device-code-key.py:526`）），也是多帧、各帧各自提交。撤到一半服务没了，报的是「这一趟写进库的 {len(keys)} 条没撤成」（`scripts/experiment-apparatus/device-code-key.py:717`），已撤的那几帧也算在里面。推的；撤回写同一个标记，重做同一条命令只补没撤的这一点我没验。

## 六、要主 agent 跑的命令

模型文件 sha256（补记第三个文件，前两个与文首相同）：

```
3cc91589390c8c0e20bb03d3ceb0b5c1c6f5f25912d7b7de75f2ba901f76166e  model/tools/mutant-rounds.sh
```

1. 原型 P0、P1、P2（只读，不起服务、不起子进程）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-sonnet -- python3 -B /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-sonnet/model/probe.py --repo /home/fy5090/code/singlefs`
   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-sonnet`（submit 的 `--cwd`）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`name=p0_python_key_alphabet`、`name=p0_python_key_edge`、三条 `name=p1_…`、七条 `name=p2_client_write_many`。
   - 我推的读数（待跑，对不上就是我读错）：P0 `accepted=66 documented=66 equal=True`，边角 `empty=False trailing_newline=False dotdot=True leading_slash=True surrogate_escaped_bytes=0`；P1 第一格 `store_writes=['items/injected.out']`，第二、三格 `store_writes=[]`；P2 `three_small_values frames=1`、`8192_small_values frames=1`、`8193_small_values frames=2`、`two_9_MiB_values frames=2`、`one_small_value_that_starts_with_the_manifest_magic frames=3`、`one_17_MiB_value frames=4`、留有登记旧版本的那一格 `frames=2 prefix_deletes=1`。
2. 同一份原型按三方腿的复跑形态（判决要引读数时用；`--frozen` 给只读副本或仓根都行，原型只读它）：
   - 命令：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-sonnet/model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-verify/rerun-work`
   - 在哪个目录起：随意（路径都是绝对的）；`--work` 要是不存在或空的目录。
   - 看哪几行：同第 1 条，另有 `name=rerun_roots`。
3. 多线程测试在锁变异下的抓法，变异行 7598 与未变异代码各 5 轮（变异复跑只复用「抓到」，不能用 `--item` 重复抽样，所以另做一份包副本）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-device-library --lib 点名多线程测试一条; 范围=变异表第 7598 行与未变异代码各 5 轮; 为什么=单次观测不算数，核一次抓到够不够' task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-sonnet/model/tools/mutant-rounds.sh --repo /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-verify/mutant-rounds`
   - 在哪个目录起：随意（路径都是绝对的）；`--work` 要是不存在或空的目录；编译目录在 `--work` 底下的 `target`，跑完经 `task-server/task clean <号>` 删。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`name=mutant_applied row=7598 anchor_hits=1`；`name=rounds_mutant_summary caught=<N> of=5`（`rc=101` 是被抓到）；`name=rounds_unmutated_summary red=<N> of=5`。每轮都抓到且未变异 0 轮红才算「修到」；否则该行是「不稳定」。
4. 变异表新行能不能列出（与 r1 的任务 12056 同形）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=只列不跑，不起 cargo; 为什么=核变异表第 7590 至 7598 行是否作为可点名的项列出' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   - 在哪个目录起：`/home/fy5090/code/singlefs`（仓根）。
   - 看哪几行：逐行列出里有「判定库口子」起头的 9 行新加项，末了「锚点失效」0 行、键对不上的条数。
5. E158 一个单位最多写多少条（第二节第 1 行的「推不出」要它）：我没有现成命令。运行参数行已有 `units=` 与 `judgements_fresh=`，平均数能从它出；最大单位要在 `process_unit` 里加计数，那是改代码，不归我，主 agent 若要这个数另派实现员。

## 七、没做什么

- 没往统一任务服务交任何任务、没起 `run_in_background`；第六节五条一条都没跑，第六节的预测与第三、四节里标「推的」的读数都只是推的。
- 没复跑 r1 判决与背景材料引的任务读数（12055 至 12060、12073、12074、12078 至 12085），它们当主 agent 的读数用。
- 没读禁读清单里的两份（r2 攻方报告、r2 判决）；r1 的两份腿报告没读（判决已经摘了它们的结论，我直接对原代码）。
- 没读 Rust 标准库的互斥与 `Vec::try_reserve_exact` 的源码，涉及它们的两句是推的。
- 没量 E158 单位条数、没起服务端、没验转交脚本自证。
- 没改仓里任何文件、没动暂存区；模型在草稿目录里不入仓，主 agent 要留的话自己拷进 `research/prompts/` 并登记。
- r1 判决引的是 r1 那一版文件的行号，那一版不在工作区；我只对此刻的行号核了引文（r1 版的「一次提交」那两句只能在 r2 diff 的减行里看到）。
