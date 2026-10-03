# m3-code-r2 云端攻方腿（Opus）报告：Y2（H1–H8 哪一格会错）、Y3（别的入口上的根在别的盘、根环转过一圈）

日期 2026-10-01。被判代码：冻结副本 `/tmp/claude-1000/m3-code-r2/tree/`（开工时 `sha256sum -c` 对 `research/prompts/m3-code-r2-snapshot/crates-sha256.txt` 全部 OK），全部在它的拷贝上编、跑；下文的数都是副本上的，不是入库装置上的。只跑了 harness 档测试二进制与 `singlefs-core --lib`；checker 档、层 0、变异整表、广谱变异一样没跑，原型里没有调层 0 的枚举函数（这一轮的盘面都是换盘与注入读错，不枚举崩溃态）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y2-① H1–H3 新用例 | **打中**（两个错实现放得过） | RG「把区域号当盘号」：区域 2（txg ≡ 2 mod 3）的缺根从不核，entries 21 条与 harness 全包全绿，会话发布放过、池级 checker 红；RP「卸载、抬 F、回退那条路不重读」：全绿，一次瞬时读错就把健康盘拒成落后支 |
| Y2-① 题面点名的三个错实现 | 没打中 | 只比实例不比 txg（IO）被转圈那条红掉（量过）；「只在会话发布入口重读」没单独跑（推的：可写挂载不重读时 H1 可写挂载那条会红，同变异行 2 的机理）；「重读次数写死成 2」读法下的 R3（读三次）放得过，只偏离字面、无数据后果（推的） |
| Y2-② H4 改名 | 没打中 | 7 份文件按词换名、去空白（两份再去注释行）后与第一轮逐字节相同；旧标签字符串没有产物或脚本在认 |
| Y2-③ H5 长度 0 | 计数与三后端没打中；**打中一个放得过的错实现** | 四个后端 22 格逐格相同、计数不动；「长度 0 早退挪到 `check_request` 之前」（Z0）放得过全部用例，越界、没对齐的长度 0 文件后端交 `Ok`、另两个后端拒 |
| Y2-④ H6 删除守卫 | 没打中 | SIGTERM 停掉时镜像留下（量过），但条款只要「成功、失败、panic 都删」；名字带进程号与进程内计数，删不到别人的 |
| Y2-⑤ H8 `EINTR` | 上界与开关没打中；**打中一个放得过的错实现** | 循环没有上界（照 std 口径，持续信号下的活锁是推的）；开关只有 `cfg(test)` 里调；「被打断第二次就报错」（E1）放得过全部用例 |
| Y2-⑥ 整行换的 2 行变异 | 没打中 | 两行都红在点名用例上（prove-red「跑了 2 条，跳过 0 条，跑的都抓到了」） |
| Y3 原样代码四个入口 | 没打中 | 27 个 txg × 两种切法 × 四个入口，结局相同，切在根槽之前的 108 格全拒成缺根、盘不变 |
| Y3 用例取样 | **打中**（弱读法） | 转圈那一形上缺根那一半是卸载、准入抬 F、回退唯一的一道（拿掉它，每个入口 9 格做成、池级 checker 红；EC 下查过是 I-3.1）；这三个入口的用例只取 txg 6 不转圈那一形（有第二道兜底）；只在那条路上改错的 EC、AC 放得过全部用例 |

## 复跑

模型目录 `research/prompts/m3-code-r2-opus-model/`。`proto_entries_append.rs` 接在 entries 用例文件后面成 `tests/r2opus_entries_proto.rs`，`r2opus_zero_length_on_three_backends.rs` 原样放进 `crates/singlefs-harness/tests/`；`wrongs.py` 是错实现表，`wrongrun.py apply|restore <副本> <冻结副本> <名>` 套上与写回；脚本里的路径写死在 `/tmp/claude-1000/m3-code-r2-opus/`，换目录改脚本头的 `D=`。

```
D=/tmp/claude-1000/m3-code-r2-opus; R=research/scripts
rsync -a /tmp/claude-1000/m3-code-r2/tree/ $D/work/ && cp -r research/prompts/m3-code-r2-opus-model $D/model
T=$D/work/crates/singlefs-harness/tests
cat $T/entries_after_mount_refuse_swapped_or_behind_devices.rs $D/model/proto_entries_append.rs > $T/r2opus_entries_proto.rs
cp $D/model/r2opus_zero_length_on_three_backends.rs $T/
# 原样扫描（Y3 全量 + Y2R 一次读坏）：先编 r2opus_entries_proto，再
bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 12 bash $D/model/run-sweeps.sh orig 1 <r2opus_entries_proto 测试二进制>
# 错实现：entries + 扫描
bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 12 bash $D/model/run-wrongs.sh 10 A_only_region_zero_device:y3 RG_region_number_as_device:y3 RP_no_reread_in_the_three_other_entries:y2r R3_reread_twice_after_the_mount:y2r
# 全包、恰好连坏两次、长度 0、H6 停进程
bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 12 bash $D/model/chain4.sh
```

原样扫描（`logs/orig/`）与第一批错实现（`logs/wrong/{A,E,T01,RG,IO,RP,R3}…/`）跑的是原型的早一版：那时 Y3 用 entries 文件自己的 `go_through`（回退目标固定 txg 3），没有 `R2_CUTS`、`R2_ROLLBACK_PREV`、`Y3-VIOLATIONS` 与 `r2opus_y2_exactly_two_failures_in_a_row`；之后的跑用的是模型目录里的定稿。默认参数下两版的 Y3 每格做的事相同。

源文件 sha256（日志逐个的 sha256 在 `SHA256SUMS-logs.txt`，291 行）：

```
7cbbc74656964d03df9eee29fab022a173fced8c4ea4074807f69c7d9b62e8e6  proto_entries_append.rs
4743b58bc6b65f7c082d0016b4610cedbad618f27fff668ab9c895093af2e610  r2opus_zero_length_on_three_backends.rs
46a156b5b1b63bbcf5bb5d78ac6c211404c8ef671c40101de027dd3ac31d960f  wrongrun.py
1b9ef610e54fe6c7b1065b24d689c625ccda4b713230d2507fa8853cf355d286  wrongs.py
cc958faaca9a943592893d1f5057ecbce23f048083254ce5a93950f8f35fedca  chain3.sh
45bedf08ab915f0aaf626bbef1a2ce0f6f428387b56934c992882198adcdda12  chain4.sh
8f4ee2a642f512d27f10bd325a0b8e476418cbc2b90d4012300063b07a0646e2  ec-violations.sh
a42d220b14151df6cf8be8f1124ae343599bf86fd7a5e6414dedd70a6d0392f9  h6-kill.sh
bf977c9d9c1b4c55ce9e0e9d2964227a753282aaf194ed332784afef644ea37d  run-full-par.sh
2f9f7334c9fa029571bd3d1ddea921342fc5adfe53088b38e828068efd0950c1  run-sweeps.sh
ded97d540e3fafd77864704f5dac1dcba95e48d62b532fde5405b8f79c46e388  run-wrongs2.sh
ebf4f8c8fbaae9e0eeed3fd6390fa20316f404c268055b7546324b0ee282b54a  run-wrongs.sh
a0c347820b3d15bee7de1be39868dc7196be54b3e0b652fe967d1bcf04123edd  SHA256SUMS-logs.txt
```

### 我提的改法（都只在我的模型上量过错实现的行为、用例本身没写成入库形态，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| H2 加 P 的 txg ≡ 2 (mod 3) 一版（txg 8） | RG | 推的（用例没写）；RG 下 txg 8 会话发布做成、checker 1 条是量过的（Y3 扫描） |
| 卸载、准入抬 F、回退各加 H1 那一形（n = 1） | RP | 推的；RP 下这三个入口 n = 1 拒成落后支是量过的（Y2R 扫描） |
| 「恰好连坏两次」一格 | R3 | 推的；R3 下做成、原样拒是量过的（Y2T 扫描） |
| 卸载、准入抬 F、回退各加 txg 27 转圈一条 | EC、AC、Y3 那一格 | 推的；EC、AC、T01 下这三个入口转圈那几格做成、checker 红是量过的（EC 下 9 格查过名字，都是 I-3.1） |
| H5 单测加越界、没对齐的长度 0，改比「四个后端逐格相同」 | Z0 | 推的；Z0 下 6 格不一致是量过的 |
| H8 开关改成打断前 k 次，单测 k = 2 | E1 | 推的；没跑 |

## Y2　H1–H8 哪一格会错

装置：冻结副本整份拷进 `/tmp/claude-1000/m3-code-r2-opus/work/`，加两份原型测试文件（`r2opus_entries_proto.rs` = entries 用例文件原样 + 模型目录 `proto_entries_append.rs`；`r2opus_zero_length_on_three_backends.rs`），错实现由 `wrongs.py` 列出、`wrongrun.py` 套上与从冻结副本写回。每格是一段历史：起点（两块 4 GiB 内存稀疏盘、mkfs、取号、暖机、第一个文件 txg 3、会话里覆盖写到 P 的前一版），P 是会话里下一次覆盖写，录下 P 的写流，P 的根住的那块盘换成「P 之前 + 流切在 P 的根槽 FUA 之前 / 之后」。数都是副本上的。

### Y2-①　H1–H3 新用例自己（打中两个错实现放得过；另一个放得过但只偏离字面）

造法：H1、H3 用「切在根槽 FUA 之后」那块盘，是真崩溃留得下的盘面（r1 攻方 Z4-② 已论证，这一轮不重复）；H2 两条用「切在根槽之前」那块盘，是换盘造的（C585 本题），不是崩溃态，与改法 C 的题面一致。断言钉的：H2 盘 1 那条断言点名盘 1、两次读都是 `Bad(NotSelfVerified)`；转圈那条两次读都是 `AnotherRoot { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }`；H3 断言写调用 0、逐扇区不变、真读坏恰好 2 次。

错实现在 entries 整个文件（21 条）上的结局（`logs/wrongs-batch1b.out` 原样）：

```
WRONG A_only_region_zero_device entries rc=101 test result: FAILED. 20 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.35s failed=[a_session_publish_onto_device_one_holding_the_publish_root_taken_before_its_root_slot_write_is_refused_before_any_write ]
WRONG E_any_self_verified_root entries rc=101 test result: FAILED. 20 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.18s failed=[a_session_publish_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write ]
WRONG RG_region_number_as_device entries rc=0 test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.64s failed=[]
WRONG IO_instance_only entries rc=101 test result: FAILED. 20 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.80s failed=[a_session_publish_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write ]
WRONG RP_no_reread_in_the_three_other_entries entries rc=0 test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.80s failed=[]
WRONG R3_reread_twice_after_the_mount entries rc=0 test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.56s failed=[]
```

| 名 | 改哪里（冻结副本 `crates/singlefs-core/src/mount.rs`） | entries 21 条 | 它错在哪 |
|---|---|---|---|
| A（r1 的，对照） | 第 3416 行起那两行换成 `region_devices[0]` | 红 1 条（H2 盘 1） | — |
| E（r1 的，对照） | 第 3429 行守卫前加 `true \|\|` | 红 1 条（H2 转圈） | — |
| IO 只比实例不比 txg（题面点名） | 第 3429 行守卫去掉 txg 那一半 | 红 1 条（H2 转圈：上一圈那条根也是实例 1） | — |
| **RG 把区域号当盘号** | 第 3416 行起那两行换成 `DeviceIdentity(区域号)` | **全绿** | **量过**：区域 2（txg ≡ 2 mod 3）归盘 0，被当成没有的盘 2，缺根那一半从不核。Y3 扫描里切在根槽之前、txg 8–23 的区域 2 那几格，会话发布照常做成、池级 checker 红；txg 26、29、32（区域 2、根环转过一圈）四个入口全放过、checker 红（见下） |
| **RP 抬 F、准入抬 F、卸载、回退不重读** | 第 3444 行重读改成按线程标志跳过，`caller_inputs_agreeing_with_the_disk`（第 3834 行那一次调用）前后置标志；会话发布与可写挂载照旧重读 | **全绿** | **量过**：切在根槽之后的健康盘、那一槽第 1 次读坏一次，原样照常做成，RP 下拒成落后支（下引） |
| R3 挂着之后四个入口读三次 | 第 3444 行之后再读一次（只在 `devices_behind_the_current_version` 里），可写挂载照旧 | 全绿 | **量过**：那一槽恰好连坏两次（第 1、2 次读），原样拒成缺根，R3 下照常做成。盘上真有 P 的根，放过不造坏盘面；错在偏离被判函数自己的文档「知道住着根的槽重读一次仍坏才算」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:3397`）（推的：没有数据后果） |

RG 的两行原样（`logs/wrong/RG_region_number_as_device/y3-6..14-first.log` 与 `y3-24..32-first.log`）：

```
Y3 txg=8 region=2 slot=2 root_dev=DeviceIdentity(0) cut=BeforeItsRootSlotWrite entry=SessionPublish outcome=ok unchanged=false checker=1 remount=Ok(InstanceGeneration(1),CheckpointTxg(9)) checker_after_remount=0 ms=2855 FLAG
Y3 txg=26 region=2 slot=0 root_dev=DeviceIdentity(0) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=ok unchanged=false checker=2 remount=Ok(InstanceGeneration(1),CheckpointTxg(28)) checker_after_remount=1 ms=4126 FLAG
```

RP 与原样同一格（`logs/orig/y2r-6.log`、`logs/wrong/RP_no_reread_in_the_three_other_entries/y2r-6.log`）：

```
Y2R txg=6 entry=NormalUnmount occ=once n=1 fired=1 writes=44 outcome=ok checker=0
Y2R txg=6 entry=NormalUnmount occ=once n=1 fired=1 writes=0 outcome=behind-root(DeviceIdentity(0),Bad(Unreadable),Bad(Unreadable)) checker=0
```

RP 下抬 F（第 1、3 次读，抬 F 核两遍）、回退（第 1 次读）同样，txg 6、7、8、27 四个 txg 都是。原样的「一次读坏」整张表（`logs/orig/y2r-*.log`，四个入口 × 四个 txg × n 从 1 扫到那一槽被读的次数或 12）除回退目标被根环盖掉的那几格（txg 27 回退到 txg 3，`TargetNotACandidate`，与读错无关）外全部照常做成、checker 0。

四句（RG、RP）：
- 分不分辨臂：这里没有臂，判的是 H2、H1 的用例够不够。
- 被判系统看得到判别它的东西：RG 要 P 的根落在区域 2，H2 只取了区域 0（txg 6、27）与区域 1（txg 7）；RP 要「挂着之后、经 `caller_inputs_agreeing_with_the_disk` 的入口上一次瞬时读错」，H1 只取了会话发布（n = 1）与可写挂载（n = 8）。盘面都在用例手里，看得到。
- 满足判据字面的哪一句：「写出一种错实现放得过新用例」（`research/prompts/_m3-code-r2-body.md:67`）。RP 下健康盘被拒，同时也是同一行的「合法历史被拒」那一类，只是错实现造出的，不是 H1–H8 引进的。
- 改法（推的，零轮）：RG——H2 加一版 P 的 txg ≡ 2 (mod 3)（txg 8，会话里多覆盖写两次），断言点名盘 0；RP——H1 的会话那一形对正常卸载、准入抬 F、管理员回退各加一格（n = 1，盘面同 H1，断言照常做成、注入真发生了一次）。R3 要钉就加「恰好连坏两次」一格（两层注入盘，见 `r2opus_y2_exactly_two_failures_in_a_row`）。这几条只在我的模型上量过错实现的行为，用例本身没写成入库形态，被攻过零轮。

### Y2-②　H4 改名之后行为变没变（没打中）

做法：第一轮冻结副本 `/tmp/claude-1000/m3-code-r1/tree/` 的 7 份文件里把旧名按词换成新名、去掉全部空白，与这一轮冻结副本同样去掉空白之后逐字节比；`mount.rs`、`mounted_session.rs` 另去掉注释行再比（这两份的改动里删了一句注释）。原样输出：

```
SAME-modulo-whitespace crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
SAME-modulo-whitespace crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
SAME-modulo-whitespace crates/singlefs-harness/src/history.rs
SAME-modulo-whitespace crates/singlefs-harness/src/model_comparison.rs
SAME-modulo-whitespace crates/singlefs-harness/tests/common_admission/mod.rs
SAME-noncomment crates/singlefs-core/src/mounted_session.rs
SAME-noncomment crates/singlefs-core/src/mount.rs
```

- 即 `history.rs`、`model_comparison.rs` 的匹配臂与 checker 档两个 bin 只换了名字、重排了空白，代码一个记号没变；匹配臂的分组（落进 `ObservedRefusalReason::Unexplained` / `None` / `&[]` 的那几组）原样。
- `history.rs` 的标签字符串从 `"DevicesBehindTheCurrentVersionAndMissingItsUnits"` 变成带 `OrItsRoot` 的：在冻结副本里旧名零处；主仓 `research/results`、`research/scripts`、`research/mutations`、`.claude/gate.d` 零处；主仓里唯一一处带引号的旧名在 `research/prompts/m2-impl-r2-fixes-b-evidence/edits/04_matches.py` 第 26 行，是冻结证据，不读 history 的输出。没有产物或脚本按旧标签字符串认它。
- 没跑 checker 档两个 bin（重型）；「行为没变」靠上面的逐记号比，不靠跑。

### Y2-③　H5：长度 0（计数与三后端没打中；放得过一个错实现）

原型 `r2opus_zero_length_on_three_backends.rs`：文件后端、`O_DIRECT` 后端两臂、内存稀疏盘，同一张请求表逐个比交回、盘变没变、文件后端落地计数前后。偏移 11 个（0、512、中间、末尾前一块、末尾、末尾后一块、4 倍盘长、1、256、`u64::MAX - 511`、`u64::MAX`）× 长度 {0, 512}，原样输出末行（`logs/h5-orig.log`）：

```
H5-SUMMARY cells=22 disagreements=0
```

- 长度 0 的 11 格：文件后端计数前后都是 `ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0, punch_attempts_interrupted_by_signal: 0 }`，打洞与退路两个计数（连同被打断那个）一个都没加；在设备里与末尾那 5 格四个后端都交 `Ok`、不动盘；越过末尾、溢出的交 `OutOfRange`，没对齐的交 `Unaligned`，四个后端逐格相同。稀疏盘那一臂（实现员报告的缺口）这里补比了，同样相同。
- 错实现 Z0「长度 0 的早退挪到 `check_request` 之前」（冻结副本 `block_device.rs` 第 439 行之前插 `if length == 0 { return Ok(()); }`）：没对齐、越界的长度 0 文件后端交 `Ok`，另两个后端交 `Unaligned` / `OutOfRange`。新单测只用了对齐、在设备里的三处偏移（0、中间、末尾），稀疏盘对拍用例 `sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs` 的 `requests()`（第 34 行是它最后一项）没有长度 0 的项。Z0 下的跑数见下。

### Y2-④　H6：删除守卫（没打中：被杀时删不掉，在条款射程之外；删错别人的镜像没造出）

- 被信号停掉：模型目录 `h6-kill.sh` 起 mkfs 打洞那条用例的测试二进制，6 秒后 `kill -TERM`（测试超时被杀、内存包装限时到点送的都是这个信号），原样输出（`logs/chain4.out` 末尾）：

```
before-kill pid=2378114 files:
52 -rw-rw-r-- 1 fy5090 fy5090 4294967296 Oct  1 09:00 /tmp/singlefs-step67-punch-zeroed-ranges-2378114-0-dev0.img
48 -rw-rw-r-- 1 fy5090 fy5090 4294967296 Oct  1 09:00 /tmp/singlefs-step67-punch-zeroed-ranges-2378114-1-dev1.img
after-kill files:
52 -rw-rw-r-- 1 fy5090 fy5090 4294967296 Oct  1 09:00 /tmp/singlefs-step67-punch-zeroed-ranges-2378114-0-dev0.img
48 -rw-rw-r-- 1 fy5090 fy5090 4294967296 Oct  1 09:00 /tmp/singlefs-step67-punch-zeroed-ranges-2378114-1-dev1.img
```

  两份镜像留着，脚本随后删掉（`removed …` 两行）。停在 6 秒时实占只有 52K、48K；跑到写零之后才停，留下的是 r1 量过的各 769M。`std::process::exit`、abort（panic 途中再 panic）同理不跑 `Drop`（推的，没另跑；工作区 `Cargo.toml` 与各包没有设 `panic`，测试照常 unwind）。这不违 H6，也不违条款：条款要的是 `.claude/singlefs-ai-sop/rules/command-safety.md@测试镜像一律放临时目录`：「成功、失败、panic 都删」，同一节末段写着门禁「它看不见的：…门禁被打断之后还在跑的子进程建的」（`.claude/singlefs-ai-sop/rules/command-safety.md:155`）。记成现象：被杀那一形今天只能靠人或下一次清理删。改法（推的，零轮）：开跑前删掉进程号已不在的同前缀镜像。
- 并行删错：mkfs 用例的镜像名 `singlefs-step67-{tag}-{进程号}-{进程内计数}-dev{盘号}.img`（`crates/singlefs-harness/tests/common/mod.rs` 第 41 行的格式串），同一进程里两条用例计数不同、两个进程进程号不同，守卫只记自己建成的那几份（冻结副本 mkfs 用例第 79 行 `images.paths.push(path);` 在建成之后），删不到别人的。没打中。
- 一处窄的：`block_device.rs` 的两条新单测先建守卫、后排他建镜像（第 1130 行、第 1241 行），`O_DIRECT` 那一臂用 `std::fs::write` 建（第 1272 行，不排他）。同名文件已在（上一个被杀的、进程号被复用的进程留下的）时：排他建失败、守卫照删那一份；`O_DIRECT` 那一臂直接覆盖再删。删的是死进程留下的同名镜像，不是活着的别人的；推的，没造（要抢到同一个进程号）。

### Y2-⑤　H8：`EINTR` 重试（上界与开关没打中；放得过一个错实现）

- 上界：冻结副本 `block_device.rs` 第 449 行起的循环只在打成洞、报不支持、报别的错三处出去；被打断那一支（第 489 行起）只加计数、不设上界。注释写的上界是「这期间送到的信号数」，与 std `write_all_at` 对 `Interrupted` 同口径。持续不断的信号（例如每次 `fallocate` 期间都到一个的高频定时器信号）会让它一直转：推的，没造出——本机 ext4 上普通文件打洞造不出 `EINTR`（r1 攻方 Z3-b 同样没造出），这一轮没在别的文件系统上试。
- 开关在非测试路径上：`with_test_only_zero_fill_landing` 是 `pub`、没有 `cfg(test)`，但全 `crates/` 里调它的只在 `block_device.rs` 第 734 行 `#[cfg(test)]` 之后（第 958、1037、1137 行），`block_device.rs` 之外零处（`grep -rn` 计数 0）；默认落地方式是 `PunchHoleOrWriteZeroBytesIfUnsupported`，两个构造入口都从它起。开关在生产路径上走不到；计数 `punch_attempts_interrupted_by_signal` 在生产路径上只在真 `EINTR` 时加，是本意。这一格与 H8 之前已有的两个开关同一形态，不是 H8 新引进的。
- 开关的「第一次」按一次写零算（第 452 行 `if punch_attempts_interrupted == 0 =>`，那个计数是 `write_zeroes_at` 里的局部量），同一块盘上每次写零的第一次打洞都被当成 `EINTR`。文档注释写的就是「这一次写零的第一次打洞」，相符。
- 错实现 E1「被打断第二次就报错」（第 489 行那一支里加 `if punch_attempts_interrupted > 1 { return Err(…) }`）：新单测只让第一次打洞当成 `EINTR`，第二次就是真打洞，E1 放得过。E1 下 core 单测与 harness 全包的结局见下节。它错在「连着两次被打断就把写零报成 I/O 错」，与判决 H8「`EINTR` 重试打洞，不退回写零、不报错」不合；今天本机造不出真 `EINTR`，后果是推的。要钉它得让开关打断前 k 次（k ≥ 2）。

### 放得过的那几个错实现在 harness 全包与 core 单测上（量过，组合着跑）

harness 档 `tests/` 下 90 个测试二进制（两份 `r2opus` 原型除外；每个二进制 `--skip r2opus --test-threads 3`，4 路并行）加 `singlefs-core --lib`。为省时间，改的行互不重叠的错实现组合成两组一起套（模型目录 `wrongs.py` 末尾两行），`logs/chain4.out` 原样：

```
FULL COMBO_RG_RP_Z0_E1 streams=4 binaries=90 rc_files=90 nonzero=[] core_lib=test result: ok. 138 passed
FULL COMBO_RG_RP_Z0_E1 failed_tests=[]
FULL COMBO_EC_R3 streams=4 binaries=90 rc_files=90 nonzero=[] core_lib=test result: ok. 138 passed
FULL COMBO_EC_R3 failed_tests=[]
```

RG、RP、Z0、E1 一起套，EC、R3 一起套，90 个二进制退出码全 0、core 单测 138 条全过。组合套全绿推得出每个单套也全绿，除非两个错实现互相掩盖一条用例的红（推的：它们改的是不同的判定，没见到同一条用例同时依赖两处）。AC 只跑了 entries 一个文件（21 条全绿）。Z0 下长度 0 原型的结局（`logs/chain4.out`）：

```
WRONG Z0_zero_length_before_the_request_check h5 rc=0 H5-SUMMARY cells=22 disagreements=6 2 ok-binaries
```

6 格不一致全在长度 0、越界或没对齐的那 6 个偏移上：文件后端交 `Ok`，稀疏盘与 `O_DIRECT` 两臂交 `OutOfRange` / `Unaligned`；同一次跑里稀疏盘对拍用例（2 条）照样全过。

Z0 的四句：不分辨臂；判别子看得到（越界、没对齐的长度 0 请求在单测手里造得出）；满足「写出一种错实现放得过新用例」；改法（推的，零轮）：`a_zero_length_zero_fill_ends_the_same_on_the_file_backend_and_both_arms_of_the_direct_backend` 的偏移加「末尾后一块」「没对齐」两处，比的从「交回 `Ok`」改成「四个后端逐格相同」；或往 `requests()` 加长度 0 的越界、没对齐两项（实现员报告停下交主 agent 的第 1 条已提过往那里加长度 0，加的是在设备里的两项，抓不到 Z0）。

E1 的四句：不分辨臂；判别子看得到（开关只打断第一次，打断两次要改开关）；满足同一句；改法（推的，零轮）：开关改成「前 k 次打洞当作 `EINTR`」，单测取 k = 2，断言计数 `punch_attempts_interrupted_by_signal: 2`、交回 `Ok`。

### Y2-⑥　整行换掉的那 2 行变异（证红，没打中）

`research/scripts/prove-red.sh --copy <攻方的第二份副本> --memory 3G singlefs-core <两个名>`，原样末三行（`logs/prove-red.out`）：

```
文件后端打洞报不支持的退路一个 0 都不写（交回成功而段里还是旧字节）	抓到	an_unsupported_hole_punch_falls_back_to_writing_zero_bytes_across_chunk_boundaries 红了（日志 /tmp/claude-1000/m3-code-r2-opus/logs/prove-red/001.log）
文件后端打洞报别的错却交回成功（错被吞掉、一个 0 都不写，段里还是旧字节）	抓到	a_hole_punch_failing_with_an_input_output_error_is_reported_and_writes_no_zero_bytes 红了（日志 /tmp/claude-1000/m3-code-r2-opus/logs/prove-red/002.log）
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
```

两行的原文在冻结副本里各恰好命中一次（prove-red 先核了才改）；红的就是变异表那一行点名的用例。日志拷进了模型目录 `logs/prove-red/`。

## Y3　别的入口上的「根在别的盘」「根环转过一圈」

原型 `r2opus_y3_entries_sweep_root_elsewhere_and_lap`：P 的 txg 扫 6–32（27 个：区域 0/1/2 各 9 个，盘 0/1；txg 24–32 这 9 个的根槽里住着上一圈自证得过的根），切在根槽 FUA 之前 / 之后，四个入口各走一次；每格之后（拒没拒都）拿这组盘的拷贝可写挂载一次，入口之后与再挂之后各跑一次池级 checker。管理员回退的目标是用户定的，扫两种：第一个文件那一版（txg 3）与现行那一版的前一版。只固定前缀（P 之前的覆盖写）与故障（换盘），用户动作（入口、回退目标、之后再挂）放开。

### 原样代码：四个入口结局相同（没打中）

`logs/orig/y3-{a,b,c}.log` 汇总（命令数出）：

| 切法 | 入口 | 结局 | 格数 |
|---|---|---|---|
| 根槽之前 | 四个入口各 | 拒成缺根那一半，两次读 `Bad(NotSelfVerified)` | 各 18 |
| 根槽之前 | 四个入口各 | 拒成缺根那一半，两次读 `AnotherRoot`（txg 24–32） | 各 9 |
| 根槽之后 | 会话发布、卸载、准入抬 F | 照常做成，checker 0 | 各 27 |
| 根槽之后 | 管理员回退（目标 txg 3） | 做成 21；6 格 `TargetNotACandidate`（txg 27–32，txg 3 的根被盖掉，与盘无关） | 27 |

根槽之前那 108 格全部 `unchanged=true checker=0`、再挂之后 `checker_after_remount=0`。三个入口在「根在盘 1」「根环转过一圈」上与会话发布结局一样，没有用例不钉的不同结局。

### 打中：转圈那一形上，缺根那一半是卸载、抬 F、回退唯一的一道；这三个入口的用例只取了有第二道的那一形

把缺根那一半整个去掉（变异表 C585 第一行的替换文，T01）再扫一遍，三个入口分成两种（`logs/wrong/T01_no_root_half/`）：

```
Y3 txg=10 region=1 slot=3 root_dev=DeviceIdentity(1) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=other:NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { ring_slot: RootRingSlot { region: 1, slot: 3 }, first_reading: NotSelfVerified, reread: NotSelfVerified }) unchanged=true checker=0 remount=Ok(In
Y3 txg=28 region=1 slot=1 root_dev=DeviceIdentity(1) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=ok unchanged=false checker=2 remount=Ok(InstanceGeneration(1),CheckpointTxg(31)) checker_after_remount=1 ms=4041 FLAG
```

| T01 下 | 卸载 | 准入抬 F | 回退（目标前一版） | 会话发布 |
|---|---|---|---|---|
| 根槽之前、槽里没写过（18 格） | 18 格拒成 `NewerStateStillUnreadableAfterOneReread` | 18 格拒成 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread` | 18 格拒成 `CandidateJudgementStillUnreadableAfterOneReread` | 做成、checker 红 |
| 根槽之前、槽里是上一圈的根（9 格） | 9 格做成、checker 2 条、再挂之后 1 条 | 9 格做成、checker 1 | 9 格做成、checker 1 | 做成、checker 红 |

槽里没写过时，三个入口另有一道「这个进程知道住着根的槽，读坏重读仍坏就拒」（D16（发布语义）已定项 1 表里「根槽这一次读坏」（`.claude/kb/decisions/16-发布语义.md:37`）那一行），换上来的盘照样被拒，只是拒因不同；槽里是上一圈自证得过的根时这一道认为读得出，缺根那一半是唯一挡住它的。卸载、准入抬 F、回退今天的用例（`a_normal_unmount_onto_…`、`a_floor_raise_onto_…`、`an_administrator_rollback_onto_…`）只取 txg 6（`PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO`，冻结副本 entries 文件第 695 行）、槽里没写过那一形；H2 的盘 1 与转圈两条只走会话发布。

只在这三个入口那条路（`caller_inputs_agreeing_with_the_disk`，冻结副本 `mount.rs` 第 3834 行那一次调用）上改错、会话发布与可写挂载不动的两个错实现：

| 名 | 改法 | entries 21 条 | 扫描（切在根槽之前，回退两种目标） |
|---|---|---|---|
| EC | 那条路上「根槽里自证得过就算是它」 | 见下节全包 | 会话发布 27 格全拒；卸载 9 格、准入抬 F 9 格、回退 13 格（两种目标合计）做成、checker 红；其余拒成缺根 |
| AC | 那条路上只查 region 0 那块盘 | 全绿（`WRONG AC_only_region_zero_device_in_the_three_other_entries entries rc=0 test result: ok. 21 passed; …`） | 会话发布 27 格全拒；区域 1 且转圈的 txg 25、28、31 上卸载、准入抬 F、回退做成、checker 红；区域 1 不转圈的被第二道拒 |

EC 一行（`logs/wrong/EC_any_self_verified_root_in_the_three_other_entries/y3-24..32-first.log`）：

```
Y3 txg=28 region=1 slot=1 root_dev=DeviceIdentity(1) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=ok unchanged=false checker=2 remount=Ok(InstanceGeneration(1),CheckpointTxg(31)) checker_after_remount=1 ms=4382 FLAG
```

四句：
- 分不分辨臂：没有臂，判的是三个入口的用例取样。
- 判别子看得到：转圈那一形（根槽里是上一圈自证得过的根）在用例手里造得出，H2 已经为会话发布造过（txg 27）。
- 满足判据字面的哪一句：「造出别的入口上结局不同而用例不钉」（`research/prompts/_m3-code-r2-body.md:68`）的弱读法——原样代码四个入口结局相同，不同的是「缺根那一半被弄坏时」三个入口在两形上的结局（不转圈有第二道兜住、转圈没有）；按强读法（原样代码结局不同）这一格不中。
- 改法（推的，零轮）：卸载、准入抬 F、管理员回退各加一条 txg 27 转圈的用例（`root_device_taken_before_the_publish_root_slot_write_through(入口, PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE, ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE)`，已有的辅助函数直接调）；回退那条照现有辅助函数用目标 txg 3 也能红（原样代码缺根那一半排在判候选之前，扫描里 txg 27–32 切在根槽之前的回退全是缺根那一半拒的；错实现下被 `TargetNotACandidate` 挡住，拒因不对、断言照样红），但那样看不到错实现真放过；取现行那一版的前一版，错实现下是做成、checker 红。只改测试，不改 `crates/singlefs-core`。

EC 下 txg 27、28、29 三格的卸载、准入抬 F、回退（目标前一版）共 9 格，入口之后 checker 报的第一条都是 I-3.1（`logs/ec-violations.log`），卸载那几格另有 I-7.9，例（整行）：

```
Y3-VIOLATIONS txg=27 entry=NormalUnmount ["I-3.1: 盘 0：记账的已分配 Some(557056)，遍历全部有效根得到 425984（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 24、最新根 txg 29、环里自证过的根槽 24 个、最老的自证过的根 txg 3、遍历的候选根槽 2 个、并进遍历的由记录施加出来的版本 0 个、被实例表判抛弃的根槽 0 个、回退下界 F 27、低于 F 的根槽 22 个", "I-7.9: 实例 1 txg 28 那条带卸载记号的根把回退下界 F 从 0 抬到 27，高于根环里 txg 比它小的最新那条根的 txg 26"]
```

## 没打中的形状

| 试过的形状 | 取样范围 | 结果 |
|---|---|---|
| 原样代码，换上切在 P 的根槽 FUA 之前 / 之后的那块盘，经四个入口，之后再可写挂载 | P 的 txg 6–32（区域 0/1/2、盘 0/1、转圈 9 个），216 格，回退目标 txg 3（错实现那几批另扫「前一版」） | 切在之前的 108 格全拒成缺根那一半、盘不变；切在之后的全做成（回退到被盖掉的 txg 3 那 6 格除外）；入口之后与再挂之后 checker 全 0 |
| 原样代码，那一槽第 n 次读坏一次 / 从第 n 次起都坏 | 四个入口 × txg 6、7、8、27 × n 1–12（读不到第 n 次就停） | 一次读坏：除 txg 27 回退到被盖掉的 txg 3（`TargetNotACandidate`，与读错无关）外全做成、checker 0；从第 n 次起都坏：n 落在缺根那一半第一次读上拒成缺根，别的位置照常做成或拒成各自的「重读仍坏」成员 |
| 原样代码，那一槽恰好连坏两次 | 同上 | 落在缺根那一半的两次读上拒成缺根，别处照常做成或拒成各自的「重读仍坏」成员 |
| 错实现 A、E、IO、T01 | entries 21 条 | 各红 1、1、1、7 条 |
| H4 改名前后逐记号比 | 7 份文件 | 相同 |
| 长度 0 四个后端 | 11 个偏移 × 长度 {0, 512} | 22 格逐格相同、计数不动 |
| 整行换的 2 行变异 | 2 行 | 都抓到 |
| 守卫删错别人的镜像 | 读名字格式与守卫记账的时点 | 删不到；进程号复用那一形是推的、没造 |

## 这条腿自己的限度

- 一条腿抽一次样；「没打中」的格不支撑「没有错」。
- 全部取样在两块盘、`region_devices` 是 [盘 0, 盘 1, 盘 0]（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/src/scenario.rs:44`）的池上；三块盘以上的池没造，「区域号对盘数取模」这类错实现在两块盘上与正确实现同值，这里分不出。
- Y3 只扫了 P 前面全是覆盖写的历史（txg 6–32），前缀里没有抬 F、回退、建 inode；回退目标只扫两种。Y2R / Y2T 的 n 只扫到 12，准入抬 F 那一槽被读的次数超过 12（一次读坏 12 格全做成），第 13 次以后没扫。
- harness 全包用的是 4 路并行直接跑测试二进制，不经 `cargo test` 的调度；harness 的 `--lib` 单测（56 条）没在错实现下跑。错实现在全包上是组合着套的（两组），单套全绿是推的。没另跑一次不套错实现的全包作基线：合入后验证报 harness 档全绿（背景材料第一节），组合套之后 90 个二进制退出码全 0，没有要对基线的红。
- 第一轮原样扫描与第一批错实现用的是原型早一版（见「复跑」那一段）。
- `EINTR` 没在真信号下造出，H8 的上界与 E1 的后果都是推的。
- H6 只用 SIGTERM 停过一次（停在 6 秒），没在写零之后停、没试 SIGKILL 与 abort。
- 跑的过程：第三批（`chain3.sh`）被后台时限默认 30 分钟停在 Z0 那一格，Z0 当时已套上，我从冻结副本写回之后用 `chain4.sh` 从那一格接着跑（`logs/progress.md` 第 1 行）；一次全包串行跑（基线）看出要约一小时，停掉改成 4 路并行；内存包装两次排不上（252，slice 被别的会话占着），上限从 8G 降到 3G 排上，一次因为同会话已有一条在跑被拒（252），设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 之后排上；prove-red 第一次退 251（上限超过峰值表 2 倍），设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 之后跑。这几次被拒的都是一行没跑的。
- 开工时 `ps` 看到别的会话的 `cargo test`（`-p singlefs-format` 等）与广谱变异 `cargo nextest … -j 31` 在跑，没有性能测量；我这边一律 `nice -n 19`、经 `capped.sh 12`。
- 原型跑的崩溃态 0 个（没调层 0 枚举）。

## 没做什么

- 不判 Y1（辩方那一格）；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 重型测试没跑：checker 档包的测试与两个 bin、层 0 门禁、QEMU、herd7、变异整表、广谱变异。H4 的「checker 档两个 bin 行为没变」靠逐记号比，没跑。
- 没改主工作区、冻结副本与 merge-wt 的任何文件；写的只有本报告、模型目录与草稿目录。

## 删掉的

- 删了：仓副本 `/tmp/claude-1000/m3-code-r2-opus/work`（11M）、`/tmp/claude-1000/m3-code-r2-opus/work-pr`（11M，证红用的第二份）、编译目录 `/tmp/claude-1000/m3-code-r2-opus/target`（14G）、`/tmp/claude-1000/m3-code-r2-opus/target-pr`（175M）；H6 那一次停进程留下的两份镜像 `/tmp/singlefs-step67-punch-zeroed-ranges-2378114-{0-dev0,1-dev1}.img`（脚本当场删）。原型的两份镜像（`r2opus-zero-length-*`）由守卫删了，`/tmp` 里没留。
- 留着：草稿目录 `/tmp/claude-1000/m3-code-r2-opus/`（6.2M）里的日志、原型、报告分段与 rc 文件，已整份拷进模型目录，是复核用的材料，不是仓副本或编译目录。
