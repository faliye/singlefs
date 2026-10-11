# 第 10z 批第二十、二十一块：代码轮第一轮正推腿报告（2026-10-09）

轮名 m3-batch10z-tu-code-r1，立场正推。冻结副本 `~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/`（下文代码引文的路径写成 `crates/…`，按 `--root` 冻结副本找），kb 引文相对仓根。
模型与日志：`research/prompts/m3-batch10z-tu-code-r1-sonnet-model/`（`README.md` 有目录与重建）。读了上一轮判决与上一轮本腿的报告与模型；没读本轮攻方的产出（禁读清单）。

## 0. 开工核对与做法

```
$ cd <冻结副本> && sha256sum -c research/prompts/m3-batch10z-tu-code-r1-snapshot/crates-sha256.txt | awk '{n++; if ($NF!="OK") bad++} END {print "共", n, "行，非 OK", bad+0}'
共 338 行，非 OK 0
```

做法（冻结副本没改，下面都在拷贝上做）：

- K1–K2：把上一轮的随机历史探针（`src/p10z_probe.rs.txt`）改到新接口。新增：每次「被宽护卡住」报告读 `ways_out()`（出路二列不列、次数、槽数、放出来几个）与预演结局，再真推环、再试同一个写，逐条对；`D` / `E`（unlink）、`T`（直接调预演，核会话与盘不动）、`Z`（读错注入）、`WAY1`（照出路一卸载再挂再试）。攻方上一轮的 `atk_k1_ways_out_sweep` 等原型拼进拷贝，只把「Stuck」改成 `Stuck:A`（列推环）/`Stuck:R`（不列）。
- K3：harness 档 25 + 6 行逐行套到拷贝、按各行自己的命令跑；上一轮 11 个活变异与我新造的 19 个对整份 harness 测试文件跑；第二十一块 13 行（含主 agent 改过的那一行）与我造的 31 个变异套在扫源码模型上跑。
- K4：把 `gpu_unit_checks.rs`、`crash_amplification.rs` 测试模块里的扫源码判法与用例体从冻结副本按行区间机械抽出，做成两个独立小程序（`src/scan-gpu-main.rs.txt`、`src/scan-crash-main.rs.txt`、`src/replay-*_model.rs.txt`），不进 checker 包，不起 checker 档测试。
- K5：往拷贝的 `mounted_session.rs` 加一个差分单元测试（`src/core-mounted-session-debug.diff`），拿文件后端逐请求对拍。
- 内存与线程：每条编译与跑编译产物的命令都经 `run-with-memory-cap.sh`（3G 到 12G）、`capped.sh`，线程不超过 15；同时起的件数乘上限不超过 12G。

## 1. 用户定案与判决第三节逐条：实现同不同义

用户原话与判决条文的位置：`research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md:44` 「用户 2026-10-09 弹窗原话「甲：预演推环与准入，推不通就不列」」，第 46 行是第二十块的条目，第 47 行是第二十一块的条目。

| 条 | 实现（冻结副本:行） | 判 |
|---|---|---|
| 报之前在分配器拷贝上预演推环 | 「let mut rehearsed_allocator = allocator.clone();」（`crates/singlefs-core/src/mounted_session.rs:497`）；写走 `DeviceKeepingWritesInMemory`（同文件 293 行起），底下的盘借成只读的 `&dyn BlockDevice` | 同义；一个写都发不到真盘是类型保证（没有 `&mut` 的路） |
| 按推完的根环表算上限、预演一次这次改动的准入（连同抬 F） | 推完接着走会话发布那一段：`publish_the_change_raising_the_floor_for_space`（同文件 961 行起）；上限由抬 F 那一串自己的私有函数在盖着推环写的盘上读 | 同义；`WalkTheQueue` 那一支（534 行）自然历史里没造出，见 K1-1 |
| 放得下才列推环、放不下只列卸载再挂 | `ways_out` 里推环只在 `ChangeAdmittedAfterThePush` 时进（`crates/singlefs-core/src/mounted_session.rs:248` 起；文字尾「预演过：推完之后这次改动放得下。」在 270 行） | 同义 |
| 推环入口先判冻结 | 「if let Some(frozen) = allocator.frozen_publish() {」（`crates/singlefs-core/src/mounted_session.rs:1132`），在核盘表、预演、读写之前 | 同义 |
| C 记着落点而一个槽都没隔离：不报宽护 | 报告一侧「*isolated == 0)」（`crates/singlefs-core/src/mounted_session.rs:645`）交 `NotWhatStopsTheAdmission` | 同义 |
| 同上：推环入口在任何写之前拒 | 入口一侧同一句（同文件 1163 行）后 `return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot)`（1165 行） | 同义 |
| 出路一按 R 有没有根分两种说法、写明 R 的账永久读不出的后果 | 有根一支「R 的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时已断开。」（同文件 251 行）；没有根一支「没有读得出的有效根：再挂时择的是卸载推出来的最后一版」（255 行） | 同义；后果只写在有根一支，没有根一支实测没有这个后果，见 K2-1 |
| 出路二写推几次、写掉几槽，N 等于推完隔离实际降了多少 | 次数与槽数同一个量；N 取预演里推完前后每块盘隔离数之差（「(*device, before.saturating_sub(after))」，同文件 518 行） | 同义；两槽注入与自然的「C 加被抛弃根」形 N 都等于真推环降的数，见 K2-1 |
| 「被宽护卡住」的文档注释写明宽护是挡着的之一 | 「宽护是挡着准入的之一，不是唯一」（同文件 203 行） | 同义 |
| 正推注入级两形（重叠时放出来的数、C 有落点无隔离时推环入口的结局）与 M1、M2、M7、M12、loop-span1 补用例 | 25 + 6 行逐行重跑全红，见 K3 | 同义；仍有没盯的写法，见 K3 |
| 不碰 `mount.rs` | 改动全文里第二十块只动 `mounted_session.rs` 与 harness 测试文件 | 同义 |
| 第二十一块：`!=` 误判改掉 | 「rest.starts_with("!=")」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2581`）整个一个词 | 同义；L5 判绿 |
| 第二十一块：别的合法写法判红的写明或改掉 | `use … as`、两层花括号、`self` 组、`extern "C"` 改成判绿（L1 到 L4）；通配、条目层调宏、另起模块写明故意拒（用例名里带「故意」） | 同义；但「条目层调宏」的字面把属性与常量初始值里的表达式宏也判红，见 K4-2 |
| 第二十一块：崩溃放量名单外三种能拦的补 | 按叫停类型找层（`crates/singlefs-checker-tier/src/crash_amplification.rs:4892`）、数每个 `let`（5307 行起）；我造的 C01、A06 到 A10 判红 | 同义；类型换写法与 `#[cfg]` 的绕过仍在，见 K4-1 |
| 第二十一块：原始字符串那一处不再 `assert!` | `cr`、`br`、`r` 都认（`crates/singlefs-checker-tier/src/crash_amplification.rs:5046`） | 同义；A13、A14 与 26 个层数用例判对 |

## 2. K1　第二十块 预演定出路

### K1-1 预演与真结局

判法：每次「被宽护卡住」报告里读 `ways_out()` 有没有第二条（`Stuck:A` 列推环，`Stuck:R` 不列），然后真调推环入口、再试同一个写，看结局对不对得上；另有「放出来几个」「推几次」「写掉几槽」三个数对真推环的数。

**攻方上一轮的 `atk_k1_ways_out_sweep`（9 形，203 格）在新实现上**（拼进拷贝，命令 `atk_run.sh k1w atk_k1_ways_out_sweep ATK_PUSH_ATTEMPTS=3 ATK_PRES=2,4,8 ATK_FILLS=6,11,16,21`；日志在模型目录）：

```
$ grep -o 'field=[^ ]* push_then=[a-z]*[^ ]*' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k1w-new2.log | sed -E 's/\(pushed\([0-9]+\),iso_after=[0-9]+\)/fail/' | sort | uniq -c
     20 field=Stuck:A:adm push_then=ok@1
    182 field=Stuck:R:budget push_then=failfail
      1 field=Stuck:R:budget push_then=ok@2
$ grep -o 'way_one_then=[a-z_]*' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k1w-new2.log | sort | uniq -c
    203 way_one_then=ok
```

- 上一轮同一批是 203 格全报推环、其中 182 格推完仍被拒（155 格对照臂放得下）。现在列推环的 20 格推完同一个写第一次就放得下（20/20），不列的 183 格里 182 格推完三次都被拒；203 格出路一卸载再挂都放得下。「列了推环而推完被拒」0 格。
- 实现员三形长写（`atk_k1_implementer_forms_sweep`）：

```
$ grep -o 'first=Stuck:[AR] push_then=[a-z]*' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k1-impl.log | sed -E 's/@[0-9]//' | sort | uniq -c
     18 first=Stuck:A push_then=ok
      6 first=Stuck:R push_then=fail
```

  长 2 到 40 个单元列推环、推完放得下；60、80 个单元不列、推完仍拒；三形的 `way_one=ok` 24/24。
- **一个例外，预演说放不下而推完其实放得下**：`pre=8 fill=16 g=17` 一格，不列推环，真推环之后第一次再试被拒、第二次放得下（`push_then=ok@2`）。183 格不列的格子里预演的拒绝停因全是「一次准入的发布数做满」（`field=Stuck:R:budget`）：被拒的那一次已经把抬 F 推成的几串落了盘，第二次再试接着往下。预演只试一次这次改动，所以对「推完再试一次」是准的，对「推完再试几次」在这一格偏严（1/183）。后果只是少列了一条本来走得通的出路，出路一照列且走得通。

**随机历史**（384 槽盘，前缀覆盖写 4 到 160 次，R 取回退 / 建 inode / 覆盖写，故障 R 的账 / 谱系片 / 全部单元，动作串取自覆盖写、建 inode、长写、顺序写、推环、卸载再挂、崩了再挂、回退；a、c、e、f 四批加注入多槽 C 的 jk 批；全程没有注入块设备错）：

```
$ python3 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/sum_new.py <a1..a4 c1..c4 e3..e6 f20 f40 f60 jk-* ff 各日志> | grep -v '^---\|^jk-\|^$'
304 ('pushed', 'retryNS', 'R')
598 ('pushed', 'retryOk', 'A')
818 NotWide
1035 Stuck
52 flagrNe
731 reh=A
304 reh=R
902 withCMP
```

- 902 条带「预演结局对真结局」比对的报告里：列推环（`A`）598 条推完再试全放得下，不列（`R`）304 条推完再试全被拒；`CMP` 非空 0 条（脚本对任何对不上的格单列一行 `CMP-nonempty`，没有这一行）。「预演说推完放得下、照推环走完同一个写却被拒」0 条，「预演说放不下而推完其实放得下」在这一批里 0 条（攻方那批的 1 格是第二次尝试放得下）。
- `flagrNe` 52 条全在 jk 批（往根环表手工注入 C 槽）：探针对「R 的 txg」的期望只对自然历史有定义，注入的槽它不认。注入形的 R 另用独立读法核：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/rr-r-txg-check.txt
offs=3,9  独立读（P 动作，探针读根环表）: rOcc=Some((19, Some(4))),needs=Some(9)   报告里的: S(r=Some(4),n=6
offs=5,30,41  独立读（P 动作，探针读根环表）: rOcc=Some((19, Some(12))),needs=Some(17)   报告里的: S(r=Some(12),n=14
offs=20,46  独立读（P 动作，探针读根环表）: rOcc=Some((19, Some(17))),needs=Some(22)   报告里的: S(r=Some(17),n=19
```

  R 那一槽的占用者 txg 对上（4、12、17）；n 比 needs 少 3，是报告那一步前面的长写已经把现行 txg 推进了 3。

**我上一轮写的推翻条件在新实现上**：「384 槽盘上推环、重试被拒，而同一历史无 C 的对照里写得下」。上一轮的 `ff` 形（前缀覆盖写 12 次，长写步长 20）现在的原样输出：

```
$ grep -h DIAG research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/ff.log | sed -E 's/DIAG label=ff prefix=12 //' | cut -c1-250 | head -2
r=o faults=racct actions=HHHH step=2H NS[Ceil(14,14);S(r=Some(16),n=15,iso=24,rEq,reh=R:NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingThe,rel=12+12);hold=1;isoC=24;needs=Some(15)]|pushed15|stillIso=0/24|retryNS[Pub(6+3);N;hold=0;isoC=0;needs=None]|C
r=o faults=racct actions=HHHH step=3H NS[Pub(6+3);N;hold=0;isoC=0;needs=None]|pushRef:Nothing|stillIso=0/0|retryNS[Pub(6+3);N;hold=0;isoC=0;needs=None]|CMP[]
```

同一步（第 3 次 `H`）上一轮推 15 次后仍被拒、出路里列着推环；现在 `reh=R`，`ways_out()` 只剩出路一（`Stuck:R` 不列推环）。同形三批（f20、f40、f60，上一轮 39 行 `retryNS` 的来源）：

```
$ python3 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/sum_new.py <f20 f40 f60 日志> | grep -v '^---\|^$'
109 ('pushed', 'retryNS', 'R')
17 ('pushed', 'retryOk', 'A')
168 NotWide
126 Stuck
17 reh=A
109 reh=R
126 withCMP
```

列推环的 17 条全放得下，被拒的 109 条全不列。推翻我这条判断的现象：任何一条 `Stuck:A` 报告之后真推环再试被拒（上面 598 + 20 + 18 条 0 例），或一条 `Stuck:R` 之后真推环一次就放得下（0 例，只有攻方那批 1 格要第二次）。

**出路一在不列推环的格子里走得通吗**（`WAY1=1`：被拒后不推环，卸载再可写挂载，再试同一个写）：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/w2.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/w3.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/wf20.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/wf40.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/wf60.log | grep DIAG | grep -o 'WAY1:[^|]*|retry[A-Za-z]*|rehearsal=[A-Z]' | sed -E 's/WAY1:(X:remount-[a-z]+)[^|]*\|/\1 /' | sort | uniq -c
    212 X:remount-ok retryOk|rehearsal=A
     47 X:remount-ok retryOk|rehearsal=R
```

- 不列推环的 47 条（预演说推完仍被拒）出路一照样放得下 47/47，列推环的 212 条出路一也放得下 212/212。读回全零一直在（`S2_PERSIST=1`）时：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/wp1.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/wp2.log | grep DIAG | grep -o 'S(r=\(Some\|None\)([0-9]*)\?[^|]*|WAY1:[^|]*|retry[A-Za-z]*|rehearsal=[A-Z]' | sed -E 's/S\(r=(Some|None).*\|WAY1:(X:remount-[a-z]+)[^|]*\|(retry[A-Za-z]*)\|rehearsal=([A-Z])/r=\1 \2 \3 rehearsal=\4/' | sort | uniq -c
    132 r=Some X:remount-refused retryNoSession rehearsal=A
     12 r=Some X:remount-refused retryNoSession rehearsal=R
```

  144 条有根（`Some`）的格子再挂全被拒、会话已断开，与出路一那句「R 的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时已断开。」（`crates/singlefs-core/src/mounted_session.rs:251`）同义；其中 132 条同时列了推环，12 条没列（此时只剩出路一，而出路一在永久读不出时走不通：条文写了这一半，报告里没有第三条出路，这是预演不管的一格，见没做什么）。

**`WalkTheQueue` 那一支（unlink 之后走空被卡）**：自然历史里没造出来。`D`（unlink 最近建的 inode）与 `E`（unlink 第一个文件）在 a4、c3、e3 到 e6 的随机动作串里都出现过，被拒报告里没有一条出在这两步（a4、c3、e3 到 e6 的日志里 `grep DIAG | grep -E 'step=[0-9]+[DE] ' | grep -c 'NS\['` 得 0，那几批共 1479 条第一次被拒的报告里没有）。这一支只经直接调预演走到（下面 K1-2 的 `T[walk]`），预演里 `WalkTheQueue` 那一支的 `expect(…)`（「推环接在有树的一版后面」，`crates/singlefs-core/src/mounted_session.rs:536`）没触发。

### K1-2 预演不落盘；预演里出错

- **一个写都发不到真盘是类型保证**：预演的盘 `DeviceKeepingWritesInMemory` 只持 `&'disk dyn BlockDevice`（`crates/singlefs-core/src/mounted_session.rs:293` 起），共享引用调不了 `write_at(&mut self, …)`、`write_zeroes_at`、`barrier`；这三条在预演盘上只落进内存的 `written_blocks` 或什么都不做。所以「屏障、写零穿到真盘」不存在这条路；读穿到真盘，读不到的块走底下盘的错。
- **读回自己写过的块的拼接、对齐与越界**：往拷贝加了差分测试 `rehearsal_device_agrees_with_the_file_backend_on_every_request`（`src/core-mounted-session-debug.diff`），拿真的 `FileBackedBlockDevice` 逐请求对拍（物理块 512 与 4096，设备 512 B 到 128 KiB，读 / 写 / 写零各占三分之一，偏移与长度含 0、越界、`u64::MAX` 附近、没对齐），错误成员与字段、读出的字节、底下那块盘有没有被写、整盘读回，都要相同：

```
$ cd <拷贝> && TMPDIR=<草稿目录>/tmp bash scripts/run-with-memory-cap.sh 6G bash scripts/capped.sh 15 cargo test --release --offline -p singlefs-core --lib sonnet_k5 -- --nocapture
SONNET_K5 requests=100000 refused=63106
test mounted_session::sonnet_k5_differential::rehearsal_device_agrees_with_the_file_backend_on_every_request ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 216 filtered out; finished in 0.96s
```

  10 万个请求 0 个不同；把判越界的 `end <= device_size` 改成 `<`、把写零的对齐判去掉，这条各红一次（`panicked at … mounted_session.rs:1499` 与 `:1505`，行号是拷贝里的）。
- **会话与分配器不被弄动**：直接调预演（`T` 动作，把私有函数放成 `pub` 的拷贝上，不经「被拒」那条路），前后比分配器 `{:?}`、盘的整份镜像、现行根；472 次调用含冻结着、没有文件的一版、没有 C：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/t1.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/t2.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/t3.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/t4.log | grep -o 'T\[[a-z]*\]:[AP]\|T\[[a-z]*\]:R:[A-Za-z]*' | sort | uniq -c
     44 T[change]:A
    104 T[change]:P
     24 T[change]:R:NoFileVersionToChange
    100 T[walk]:A
    200 T[walk]:P
$ cat <同上四份> | grep -c 'unchanged=true'
472
$ cat <同上四份> | grep -c 'unchanged=false'
0
```

  `P` 的结局成员有 `NothingHeldUntilThisProcessWritesARingSlot`（288 次）与 `PublishFrozenAfterAWriteFailureIsNotResentYet`（16 次，第一道冻结判）；没有一次 panic。
- **预演里出错时报什么**：预演里推环被拒报 `PushRefused`（原成员），推完这次改动被拒报 `ChangeStillRefusedAfterThePush`（带原 `UserChangeRefused`），两种都不列推环，出路一照列；不会把一个真能放下的改动报成卡死（上面 K1-1 的 20 + 18 + 598 条 `A` 全放得下），也没有一次把会话弄动。读盘错注入（`Z` 动作）我没造成：注入的读错那一段没被读到（`Z:readerr_on=2` 之后写照样放得下），记「没造出来」。
- **真发半路块设备错之后**（`P_FAIL_AT` 取 0、20、60、90，动作串 `oPPKoooKoo`：第一次推环中途失败，第二次再推，再崩了再挂）：

```
$ for n in 0 20 60 90; do echo "n=$n: $(grep -h DIAG research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k$n.log | grep -o 'pushRef:[A-Za-z0-9/]*' | paste -sd' ') | $(grep -h DIAG research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k$n.log | grep -o 'K:remount-ok:sessionTxg=[0-9]*:selected=[0-9]*' | paste -sd' ') | first_red: $(grep -o 'first_red=[^ ]*' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k$n.log | sort -u | paste -sd' ')"; done
n=0: pushRef:AfterRehearsal1/20 pushRef:Frozen | K:remount-ok:sessionTxg=28:selected=28 K:remount-ok:sessionTxg=34:selected=34 | first_red: first_red=-
n=20: pushRef:AfterRehearsal2/20 pushRef:Frozen | K:remount-ok:sessionTxg=29:selected=29 K:remount-ok:sessionTxg=34:selected=34 | first_red: first_red=-
n=60: pushRef:AfterRehearsal4/20 pushRef:Frozen | K:remount-ok:sessionTxg=31:selected=31 K:remount-ok:sessionTxg=37:selected=37 | first_red: first_red=-
n=90: pushRef:AfterRehearsal6/20 pushRef:Frozen | K:remount-ok:sessionTxg=33:selected=33 K:remount-ok:sessionTxg=38:selected=38 | first_red: first_red=-
```

  第二次推环拒成 `Frozen`（上一轮是预演在冻结着的拷贝上照样过、报成 `PublishFailedAfterTheRehearsal`，已改）；再挂选到的 txg 等于会话 `current` 的 txg，池级 checker `first_red` 全 `-`。

推翻 K1-2 判的现象：直接调预演前后任一比对为 `false`；预演盘在 `&dyn BlockDevice` 之外多出一条写真盘的路；差分测试对拍出一个不同。

### K1-3 预演的代价

每报一次整串推一遍环加这次改动剩下的发布。在预演函数里加了计时与计块数的 `eprintln!`（拷贝里，`src/core-mounted-session-debug.diff`），384 槽盘上往根环表注入一槽 C 后直接调预演，长写的单元数取 1 到 6000：

```
$ for u in 1 3 10 20 30 40 60 100 500 2000 6000; do echo "units=$u $(grep -h REHEARSAL research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/cost-units-$u.log | tail -1 | sed -E 's/REHEARSAL //') $(tail -1 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/cost-units-$u.time)"; done
units=1 secs=0.0193 clone_secs=0.0000 blocks=9093 bytes=4655616 result=A units=1 wall=0.26s maxrss=78952KB
units=3 secs=0.0194 clone_secs=0.0000 blocks=9445 bytes=4835840 result=A units=3 wall=0.26s maxrss=79044KB
units=10 secs=0.0204 clone_secs=0.0000 blocks=10453 bytes=5351936 result=A units=10 wall=0.25s maxrss=79044KB
units=20 secs=0.0200 clone_secs=0.0000 blocks=11893 bytes=6089216 result=A units=20 wall=0.26s maxrss=79712KB
units=30 secs=0.0206 clone_secs=0.0000 blocks=13333 bytes=6826496 result=A units=30 wall=0.25s maxrss=79876KB
units=40 secs=0.0214 clone_secs=0.0000 blocks=14773 bytes=7563776 result=A units=40 wall=0.27s maxrss=81116KB
units=60 secs=0.0234 clone_secs=0.0000 blocks=17653 bytes=9038336 result=A units=60 wall=0.25s maxrss=85580KB
units=100 secs=0.0270 clone_secs=0.0000 blocks=23541 bytes=12052992 result=A units=100 wall=0.26s maxrss=92776KB
units=500 secs=0.0665 clone_secs=0.0000 blocks=81717 bytes=41839104 result=A units=500 wall=0.30s maxrss=169216KB
units=2000 secs=0.0633 clone_secs=0.0000 blocks=8308 bytes=4253696 result=R units=2000 wall=0.31s maxrss=143164KB
units=6000 secs=0.3841 clone_secs=0.0000 blocks=8308 bytes=4253696 result=R units=6000 wall=0.67s maxrss=271028KB
```

- 内存：推环那一串约 4.6 MB 常数；推完放得下的话，用户这次改动要写的每个数据单元（32 KiB，两块盘各一份）全进 `written_blocks`，每单元约 74 KB（从 3 到 500 个单元斜率 (41.84 − 4.84) MB / 497），也就是约 2.5 倍于用户内容，另加用户自己持着的那一份；放不下时（`result=R`）在准入就拒、不写数据，只剩推环那一串的常数。随改动大小线性涨，不随盘槽数涨（`clone_secs` 都是 0.0000，到本装置能造的盘大小）。
- 时间：几十毫秒（20 到 66 毫秒），2000 个单元以上被拒时为释放换下的单元要读盘核校验和，6000 个单元 0.38 秒。
- 没有量到「报错这一步本身失败或慢到不可用」的形；没造出来的是几十万单元的真大盘与长删除队列（后者在预演里要把队列走空，内存与时间随剩下的批数涨，自然历史里没造出被卡的删除，没量）。

## 3. K2　第二十块 出路措辞、冻结与无隔离

### K2-1 两种 R 的出路一说法、出路二的次数与槽数、N、多槽 C 与重叠

脚本对每条「被宽护卡住」报告核：`ways_out()` 有没有第二条等于不等于「预演结局是不是放得下」（`!ways2`）；出路二文字里的「发 N 次」「轮到的 N 槽」对独立数出的次数（`!txt12`）；「放出来 M 个」对预演报的数（`!txt3`）；出路一的两种说法对 R 那一槽有没有读得出的有效根（`!way1`、`!way1b`）；报的次数对暴力数（`!needs`）；挡着的槽数对独立算的（`!iso`）。上面 K1-1 那条 `sum_new.py` 的输出就是这些：1035 条里除了 jk 批的 `flagrNe`（见上）没有一条带这几个标记。

- **有根（`Some`）**一支：自然历史里 R 那一槽挂载那一刻记成有效根，a、c、e 批 856 条（`scripts/stuck_by_fault.sh` 的输出两行：`racct,rlin` 345 条、`rall` 511 条，全是 `Some`）里文字都是「按新实例的实例表判成被抛弃」，永久读不出的后果（上面 `wp` 批 144/144 再挂被拒）与文字同义。
- **没有根（`None`）**一支：只在「崩在 R 的根落盘之前」这一形出现。攻方上一轮 `atk_k1_fill_sweep` 的那一格，新实现下：

```
$ grep -h 'way_one:' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/k1-fill.log | cut -c1-420
ATK k1 pre=4 fill=16 step=5 units=16 arm=crash@63/100 way_one: refused_first=nospace(stop=ceiling(F=7,C=7),last=admission[0:1163223/1163264,1:1163223/1163264],field=Stuck(r_slot=2.3,r_txg=None,n=19,iso=[51, 51]),raises=1) unmount=ok(3) remount_c_slots=0 remount_c_iso=0 grow13_after=ok(raises=0) viol=0
```

  `r_txg=None` 时文字是第二种（「没有读得出的有效根：再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，C 那一圈照它重算、不再有这一槽」），再挂后 C 没了（`remount_c_slots=0`）、同一个写放得下；上一轮攻方指出的「R 按新实例的实例表判成被抛弃」在这一形不准，现在不再这么说。这一支我只造出这一格，没有自然随机批；`None` 一支的永久读不出后果（读根槽读坏而不是账读坏时再挂）：`rroot` 故障下没有宽护（C 空）、不出报告，所以这一支没有后果可写，也没有后果可造。
- **N 等于推完隔离实际降了多少**：自然的「C 加被抛弃根」形（攻方 `pool_whose_newest_root_was_abandoned_by_a_crash_recovery` 的搭建，往上加长写，`son_abandoned_and_c_stuck_sweep`）里被拒后列了推环的 22 格，逐格比文字里的次数、槽数、放出来的数对独立数出的次数与真推环降的隔离总数：

```
$ bash research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/son_check.sh research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/son-*.log
     22 text=real released>C-only retry=ok
```

  22/22：三个数都对，放出来的数大于只数 C 那一圈罩着的挡着的槽数（例：`iso_field=180`、`text_released=204`、`real_released=204`，多出来的 24 个是推环途中写掉被抛弃根那一槽时它独占的），推完同一个写放得下。这是上一轮「放出来的数偏大」那一形的反面：现在报的是真降的数。
- **多槽 C**：自然的两槽 C（攻方丙并中间那一形，Q 顺序写加 R 建 inode，`son_two_c_stuck_sweep`）上被拒 59 格，列推环 0 格（推环救不了）；所以出路二文字在自然两槽 C 上没被读到。往根环表注入多槽 C 的 jk 批里，C 槽数 2 与 3 的格子列推环 12 条、不列 10 条，文字与次数、槽数、放出来的数全对（`CMP` 全空），列了的推完全放得下：

```
$ bash research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/jk_holds.sh research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/jk-*.log
     22 reh=A hold=1 pushed stillIso=0 retryOk
      8 reh=A hold=2 pushed stillIso=0 retryOk
      4 reh=A hold=3 pushed stillIso=0 retryOk
      8 reh=R hold=1 pushed stillIso=0 retryNS
      6 reh=R hold=2 pushed stillIso=0 retryNS
      4 reh=R hold=3 pushed stillIso=0 retryNS
```

  注入的 C 是往内存表里塞的，盘上没有对应记录，所以这一支只证推环算与摘，不证挂载读出多槽 C。
- **C 与被抛弃根重叠**：`J_OVERLAP`（C 的落点取被抛弃根的落点），原样输出里 R 那一槽之前先轮到的被抛弃根还护着重叠的槽，N 是 0，k=20 越过被抛弃根那一槽才放：

```
$ grep -h 'NS\[' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/jo-overlap-new.txt | grep -o 'S(r=[A-Za-z0-9()]*,n=[0-9]*,iso=[0-9]*,rNe,reh=R[^,]*,rel=[0-9+]*\|stillIso=[0-9/]*' | paste -sd' ' | sed 's/ S(/\nS(/g'
S(r=Some(4),n=3,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=0+0 stillIso=28/28
S(r=Some(4),n=3,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=0+0 stillIso=28/28
S(r=Some(9),n=8,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=0+0 stillIso=28/28
S(r=Some(9),n=8,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=0+0 stillIso=28/28
S(r=Some(15),n=14,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=0+0 stillIso=28/28
S(r=Some(15),n=14,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=0+0 stillIso=28/28
S(r=Some(21),n=20,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=14+14 stillIso=0/28
S(r=Some(21),n=20,iso=28,rNe,reh=R:NoSpaceAfterRaisingThe,rel=14+14 stillIso=0/28
```

  （`H_STEP` 取 40 到 130 的五批里落到被拒点的格子，同一个 k 出两遍是两个步长落到同一个被拒点。）上一轮这一形出路二会说「放出来 28 个」而真推完一个没放（`stillIsoC=28`）；现在 k=3、8、14 这几格预演报的 N 是 0（`rel=0+0`）、真推完 `stillIso=28/28` 一个没放，推环不列（`reh=R`）；k=20 越过被抛弃根那一槽，`rel=14+14` 对 `stillIso=0/28`，隔离总数降 28 个，报的就是 28。这一支是注入级，自然历史里没造出 C 与被抛弃根重叠。
- **有盘读坏的槽**：故障取 R 的账 / 谱系片 / 全部单元（`racct,rlin;rall`）时有宽护报告，取 R 的根槽读回全零（`rroot`）时 C 是空的、不出报告（`stuck_by_fault.sh` 的输出里没有 `rroot` 一行）。

推翻 K2-1 的现象：任一条报告的 `!txt12`、`!txt3`、`!way1`、`!way1b` 标记，或 `son_check.sh` 出现 `text!=real`，或 `released<=C-only` 而文字写了更大的数。

### K2-2 冻结之外，预演过、真发第一次被拒的状态

入口第一道冻结判（`crates/singlefs-core/src/mounted_session.rs:1132`）改完以后，预演（报告一侧）与真发（入口一侧）走的是同一个函数 `push_the_root_ring_on`（`crates/singlefs-core/src/mounted_session.rs:500` 在预演里调它，1123 行是它本体），区别只有盘：预演是读穿到真盘的内存盘，真发是调用方交进来的盘。所以两边在第一次发布上能分叉的只有：真盘的写 / 屏障报错（`PublishFailedAfterTheRehearsal`，K1-2 的 `P_FAIL_AT` 四点已核）、调用方交进来的盘表与会话持着的不一致（`DevicesHandedIn`，M2 那一行 2356 钉）、两次调用之间用户又写了别的（不是同一个状态）。预演不读盘核换下的那一份只在入口内层那一遍整串预演里跳过（`SkippedByTheDryRunBeforeAcquisition,`，`crates/singlefs-core/src/transaction.rs:6400`；入口里 1182 行用它），报告一侧的预演后面还有一遍走 `publish_version` 的真发（读盘核照做），所以那一份读坏会让预演就报 `PushRefused`，不会过了预演才在真发被拒。读错注入（`Z`）没造成，这一格是读码推的。

逐格实测：1000 多条 `A` 报告（K1-1）里「预演过、真推环第一次就被拒」0 条；`Frozen` 在 P_FAIL_AT 四点之后第二次推环全部拒成 `Frozen`，没有第二次真发。推翻的现象：一条 `Stuck:A` 之后真推环在第一次发布上被拒，且不是真盘的写 / 屏障错、盘表不一致或两次调用之间用户又写过。

### K2-3 C 记着落点而一个槽都没隔离

两处处置：报告一侧不报宽护（`crates/singlefs-core/src/mounted_session.rs:645`），推环入口在任何写之前拒（同文件 1163 到 1165 行）。判据同一句：C 那一圈罩着的槽里隔离着的数全为 0。

- 该报宽护而被这一判吞掉的形：只要有一个槽是隔离着的就报，所以「部分隔离、部分分配着或在 defer 窗口里」照报，且推环放得出那几个隔离着的；被抛弃根也把同一批槽置了隔离位的（C 与被抛弃根重叠）算隔离着，照报（上面重叠的 28 格）。没有找到被吞掉的形：自然历史里「C 在、停因是 F 在上限、C 罩着的槽一个都没隔离」0 条（`bash research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/notwide_hold.sh <a、c、e 三批日志>` 的输出只有两行：`122 Ceil hold=0 isoC=0`、`501 Pub hold=0 isoC=0`，a、c、e 三批 1479 条第一次被拒的报告里 856 条报成宽护、其余 623 条 C 全是空的）；注入级的那一形入库两条用例各钉一处（行 2357 报告一侧、行 2358 入口一侧，都红）。
- 报告一侧还有停因的两道（预算用完、抬 F 取不到落点）：`NotWhatStopsTheAdmission`，在 C 在且隔离着的注入形上入库用例钉了（行 2362、2363 红）；自然历史里这两种停因都出在 C 已空（`Pub hold=0`），所以这是注入级的钉住，不是条款说的自然形。

## 4. K3　判别力

### K3-1 实现员追加 25 行、整行替换 6 行（harness 档）逐行重跑

把改动全文第 2339 到 2363 行（追加 25）、2369 到 2374 行（替换 6）逐行套到冻结副本的拷贝（旧串恰好命中一次），按各行自己的命令跑（release，`cargo test --release --offline` 加那一行的 `-p … --test … -- <过滤>`）：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/rows/rows-a1.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/rows/rows-a2.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/rows/rows-a3.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/rows/rows-b1.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/rows/rows-b2.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/rows/rows-b3.log | sed -E 's/^行([0-9]+) .*: rc=([0-9]+) 红 ([0-9]+) 条.*/\1:rc\2:红\3/' | sort -n | paste -sd' '
2339:rc101:红8 2340:rc101:红1 2341:rc101:红1 2342:rc101:红1 2343:rc101:红1 2344:rc101:红1 2345:rc101:红1 2346:rc101:红1 2347:rc101:红1 2348:rc101:红1 2349:rc101:红1 2350:rc101:红1 2351:rc101:红1 2352:rc101:红1 2353:rc101:红1 2354:rc101:红1 2355:rc101:红1 2356:rc101:红1 2357:rc101:红1 2358:rc101:红1 2359:rc101:红1 2360:rc101:红1 2361:rc101:红1 2362:rc101:红1 2363:rc101:红1 2369:rc101:红14 2370:rc101:红1 2371:rc101:红1 2372:rc101:红1 2373:rc101:红1 2374:rc101:红1
```

31 行全红（追加 25 + 替换 6）。**上一轮不红的行 1524（第九块 loop-span1，现行 2374）现在红 1 条**，点名的 `unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_…` 用例拦住。攻方 M1（行 2355）、M2（行 2356）、M7（行 2353）、M12（行 2361）各红 1 条。

### K3-2 上一轮我自己活下来的 11 个变异，在新的整份 harness 测试文件上重跑

整份文件跑（54 条加 1 条慢档，慢档不跑）。上一轮的旧串在新代码里有的已不再恰好命中一次，那几个在下面写明：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut2-a.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut2-b.log | sed -E 's/ \| .*//' | cut -c1-150
maxmin: rc=101 红 2 条 pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up the_slots_the_way_out_by_the_push_releases_with_two_ri
noIsoCheck: 旧串命中 2 次，跳过 (0s)
budgetAsStuck: rc=101 红 1 条 no_space_when_the_publishes_of_one_admission_run_out_while_the_ring_sl
spaceAsStuck: rc=101 红 1 条 no_space_when_the_floor_raise_finds_no_placement_while_the_ring_slots_
failNoCurrent: rc=101 红 1 条 pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refu
slotsWrittenEmpty: rc=101 红 3 条 a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_write pushing_the_root_ring_with_two_ring_slots_h
textCountZero: 旧串命中 2 次，跳过 (0s)
textRegionSlot: 旧串命中 2 次，跳过 (0s)
skipDevCheck: 旧串命中 0 次，跳过 (0s)
emptyHeld: rc=101 红 1 条 a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_write
notRecordedAsNotAllocated: rc=0 红 0 条 
```

- 红了 6 个：`maxmin`、`budgetAsStuck`、`spaceAsStuck`、`failNoCurrent`、`slotsWrittenEmpty`、`emptyHeld`。
- `noIsoCheck` 现在有两处（报告一侧、入口一侧），各改一处整份文件跑：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut3-e.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut3-f.log | cut -c1-120
noIsoReport: rc=101 红 1 条 no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolate
noIsoPush: rc=101 红 1 条 pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_whi
```

- `textCountZero`、`textRegionSlot`、`skipDevCheck` 的旧串变了：出路二文字的次数、槽数在两处各出现一遍，我改成分别改第一处、第二处（`textCount1Zero`、`textCount2Zero`）、换槽号区域号（`textNoneSlotSwap` 改的是没有根那一支）、出路一的 txg 加一（`textRPlusOne`），全红（见 K3-3 表）；`skipDevCheck` 的代码搬到 `mount.rs`，用行 2356（M2）代；行 2356 红。
- 仍活的 1 个：`notRecordedAsNotAllocated`（表里没记着的被抛弃根改走不整段隔离，C762 那一支），用户已知留着，不在本轮判范围。

### K3-3 我新造的 19 个变异（对整份 harness 测试文件）

写在 `scripts/mut3_body.py`，套 `scripts/mut3.py`，日志 `logs/mut/mut3-*.log`：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut3-a.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut3-b.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut3-c.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/mut3-d.log | sed -E 's/^([A-Za-z0-9]+): (rc=[0-9]+ 红 [0-9]+ 条).*\| \[.test result: ([a-zA-Z]+)\. ([0-9]+) passed; ([0-9]+) failed.*/\1 \2 \3 \4p \5f/; s/^([A-Za-z0-9]+): rc=0 红 0 条 +.*(test result: ok\. [0-9]+ passed).*/\1 rc=0 红0 \2/' | cut -c1-120
textCount1Zero rc=101 红 5 条 FAILED 49p 5f
textCount2Zero rc=101 红 5 条 FAILED 49p 5f
textReleasedMin rc=101 红 5 条 FAILED 49p 5f
textDeviceZero rc=101 红 5 条 FAILED 49p 5f
textRPlusOne rc=101 红 9 条 FAILED 45p 9f
textNoneSlotSwap rc=101 红 3 条 FAILED 51p 3f
releasedReversed rc=101 红 5 条 FAILED 49p 5f
releasedZero rc=101 红 5 条 FAILED 49p 5f
releasedFirstDeviceOnly rc=101 红 5 条 FAILED 49p 5f
walkSkipped rc=0 红 0 条 ok 54p 0f
overlayIgnored rc=0 红 0 条 ok 54p 0f
keepOffByOne rc=0 红 0 条 ok 54p 0f
zeroesNotKept rc=0 红 0 条 ok 54p 0f
writeNotKept rc=0 红 0 条 ok 54p 0f
lengthUnaligned rc=0 红 0 条 ok 54p 0f
rangeEndLt rc=0 红 0 条 ok 54p 0f
barrierErr rc=101 红 13 条 FAILED 41p 13f
rehearseSkipsChange rc=101 红 8 条 FAILED 46p 8f
frozenCheckDropped rc=101 红 1 条 FAILED 53p 1f
```

红 12 个，活 7 个。活的 7 个：

1. `walkSkipped`（预演里 `WalkTheQueue` 那一支改成恒放得下）：没有一条用例的被拒点出在 unlink 之后的走空（K1-1 自然历史也没造出）。
2. `overlayIgnored`、`keepOffByOne`、`zeroesNotKept`、`writeNotKept`（预演盘的读不认自己写过的块、记块号错一位、写零不记、写不记）：入库用例与端到端都不依赖预演盘读回自己写过的块——我把 `writeNotKept` 编进探针，a1 到 a3 三批与 `T` 直接调预演的输出和不改的版本逐个相同：

```
$ python3 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/sum_new.py research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/a1.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/a2.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/a3.log | grep -v '^---\|^$'
26 ('pushed', 'retryNS', 'R')
263 ('pushed', 'retryOk', 'A')
162 NotWide
356 Stuck
330 reh=A
26 reh=R
289 withCMP
$ python3 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/sum_new.py research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/pm-a1.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/pm-a2.log research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/pm-a3.log | grep -v '^---\|^$'
26 ('pushed', 'retryNS', 'R')
263 ('pushed', 'retryOk', 'A')
162 NotWide
356 Stuck
330 reh=A
26 reh=R
289 withCMP
$ diff <(grep -o 'T\[[a-z]*\]:[^ ]*' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/t1.log) <(grep -o 'T\[[a-z]*\]:[^ ]*' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/pm-t1.log) | wc -l
0
```

  所以这四种改坏在今天的产品路径上看不出来（预演盘写下去的东西没有后来的读要读回）；只有我加的差分测试（K1-2，10 万个请求）钉得住预演盘「读回自己写过的块」这条合同。若以后有读回自己写过的块的新路径（比如预演里多一步读盘核），这四种就是缺口。
3. `lengthUnaligned`、`rangeEndLt`（预演盘的对齐判只判偏移、越界判 `<`）：产品从不发这样的请求，今天等价；差分测试能红。

推翻 K3 判的现象：这 7 个里有哪个被入库用例拦住；或这 31 + 19 行里有哪行改用别的过滤名后不红。

### K3-4 注入级用例的那几形：自然历史里有没有、钉的是不是条款说的那件事

- C 记着落点而一个槽都没隔离：自然历史里没有（K2-3：623 条没报成宽护的第一次被拒报告 C 全是空的）。钉的是同一句判据在报告一侧与入口一侧各一处（行 2357、2358 各红 1 条，整份文件跑 `noIsoReport`、`noIsoPush` 各红 1 条），与用户定案「C 记着落点而一个槽都没隔离时不报宽护、推环入口在任何写之前拒」同义。
- 停因是预算用完 / 抬 F 取不到落点而 C 在且隔离着：自然历史里没有（C 在时第一次被拒的停因全是 `Ceil`，`Pub` 的 501 条 C 都是空的）。钉的是「只在 F 已在上限时报」（D16 第 37 行「准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时」），行 2362、2363 红。
- 两槽 C 的 N、重叠：自然的两槽 C（Q 加 R）没有一格列推环（59 格全 R）；自然的「C 加被抛弃根」有（22 格，N 对）；两槽重叠的 C 与被抛弃根自然历史里没造出。入库用例钉的是「放出来的数 = 推完前后每块盘隔离数之差」，用户定案是这一句。
- 第九块 loop-span1（行 2374）：跨两槽的落点后一槽空闲且根槽读坏，自然历史里我上一轮与这一轮都没造出；用例清掉后一槽的隔离位注入，钉的是 D28 第 27 行第九项「只除去分配位置着的槽（含已释放而还在 defer 窗口里的」（`.claude/kb/decisions/28-挂载期承诺量.md:27`）所要的「补隔离补满整段跨度」。

### K3-5 第二十一块 13 行（含主 agent 改过的那一行）在扫源码模型上重跑

把第二十一块各行的 旧串→新串 套到 `src/replay-gpu_model.rs.txt`、`src/replay-crash_model.rs.txt`（把真用例体连同判法机械抽出，`include_str!` 换成读文件），编、跑，看哪几条用例红：

```
$ sed -E 's/^(行[0-9]+) .*: (红 [0-9]+ 条) \[(.*)\] 绿 ([0-9]+)$/\1 \2 绿\4/' research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/replay-gpu-rows.txt research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/replay-crash-rows.txt | cut -c1-80
行3554 红 1 条 绿5
行3555 红 2 条 绿4
行3556 红 1 条 绿5
行3557 红 1 条 绿5
行3558 红 1 条 绿5
行3559 红 1 条 绿5
行3560 红 1 条 绿5
行3572 红 1 条 绿5
行3561 红 1 条 绿4
行3562 红 1 条 绿4
行3563 红 1 条 绿4
行3564 红 1 条 绿4
行3565 红 1 条 绿4
行3566 红 1 条 绿4
```

14 行全红（3556 与 3572 是同一行的两个写法：主 agent 把 `"\u{7b}"` 换成 `as_bytes() == [0x7b_u8]` 之后的版本）。第 3566 行改的是产品源码（没编进 gpu 特性那一份 `judge_my_blocks_on_gpu`），套在被扫的源码上，`every_layer_handed_the_stop_request_hands_it_on_or_asks_it` 红。这是我在模型上的重推，不是 checker 档真跑；「真用例在 checker 档里的行为」复核不了。

## 5. K4　第二十一块 扫源码

做法见第 0 节。GPU 那条（`ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call`，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2714`）喂 49 个我造的卡池模块（`src/gpu-variants.tsv`、`src/gpu-variants2.tsv`，头上照真源码带那四条 `use`，`RAW:` 开头的是整个模块）；崩溃放量那条（`ways_a_layer_handed_the_stop_request_could_drop_it`，`crates/singlefs-checker-tier/src/crash_amplification.rs:5176`）喂 36 份在冻结副本源码上改过的文本（`scripts/gen_crash_variants.py` 生成，不入库，生成脚本入库）；层数函数（`brace_depth_at_the_start_of_each_line`，同文件 4968 行）喂 26 个手算期望的 case。真源码两条都判绿（`REAL GREEN`）。

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/scan-gpu-variants.txt research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/scan-gpu-variants2.txt | sed -E 's/^([A-Za-z0-9_]+): (GREEN|RED|PANIC).*/\1 \2/; s/^REAL (GREEN|RED).*/REAL \1/' | awk '{printf "%s %s; ", $1, $2} NR%4==0 {print ""} END {print ""}'
REAL GREEN; L1_use_as_other GREEN; L2_nested_braces GREEN; L3_self_group GREEN; 
L4_extern_c_block GREEN; L5_const_ne GREEN; L6_plain_items GREEN; L7_attr_include_str RED; 
L8_const_assert RED; L9_concat_in_static RED; L10_precise_capture_use PANIC; L11_char_brace GREEN; 
L12_raw_string_mod GREEN; L13_c_raw_string GREEN; L14_raw_ident_mod RED; L15_lifetimes GREEN; 
L16_doc_use_as_std GREEN; L17_trailing_comment_use GREEN; L18_block_comment_nested GREEN; B1_use_as_std RED; 
B2_use_wgpu_as_std RED; B3_nested_as_std RED; B4_self_as_std RED; B5_use_last_seg_std RED; 
B6_mod_std RED; B7_extern_crate RED; B8_glob RED; B9_macro_rules RED; 
B10_struct_std GREEN; B11_type_alias_wgpu GREEN; B12_cfg_use_then_fn GREEN; B13_cfg_test_use_then_fn GREEN; 
B14_fn_name_via_local_macro_in_fn GREEN; B15_use_inside_fn_as_std RED; B16_extern_crate_after_attr RED; REAL GREEN; 
L13b_c_raw_string_with_inner_quote PANIC; L19_pub_crate_use GREEN; L20_use_super_glob RED; L21_derive_and_attrs GREEN; 
L22_same_name_use_as_same_name GREEN; L23_unary_not_in_const GREEN; L24_never_return GREEN; L25_where_clause_closure GREEN; 
L26_trailing_expr_macro_in_fn GREEN; L27_byte_literals GREEN; L28_lifetime_before_quote GREEN; L29_escaped_quote_char GREEN; 
L30_number_suffix GREEN; L31_cfg_any_extern_crate RED; L32_unicode_ident_and_strings GREEN; 
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/scan-crash-variants.txt | sed -E 's/^([A-Za-z0-9_.]+): (GREEN|RED|PANIC).*/\1 \2/; s/^REAL (GREEN|RED).*/REAL \1/; s/\.rs / /' | awk '{printf "%s %s; ", $1, $2} NR%3==0 {print ""} END {print ""}'
REAL GREEN; A01_cfg_any_pinned_line_with_static GREEN; A02_match_arm_rebind GREEN; 
A03_closure_param_rebind GREEN; A04_for_rebind GREEN; A05_early_return_before_pin GREEN; 
A06_tuple_let_rebind RED; A07_let_extra_spaces RED; A08_let_split_across_lines RED; 
A09_let_with_comment_between RED; A10_if_let_rebind RED; A11_trailing_comment_on_pin RED; 
A12_string_with_let_in_body RED; A13_raw_string_with_brace GREEN; A14_cr_raw_string_with_inner_quote GREEN; 
A15_if_false_block_comment_brace GREEN; A16_char_brace_literals GREEN; A17_second_pin_name_use_line RED; 
A18_type_shadow_use_line RED; A19_stop_request_shadow_by_nested_fn_param GREEN; A20_lifetime_labels GREEN; 
B01_dead_branch_copy RED; B02_trailing_comment_in_short_body RED; C01_new_layer_one_line RED; 
C02_new_layer_sync_first GREEN; C03_new_layer_dyn_fn_ref GREEN; C04_new_layer_impl_fn GREEN; 
C05_new_layer_generic GREEN; C06_new_layer_type_alias GREEN; C07_new_layer_comment_in_type GREEN; 
C08_new_layer_static_bound GREEN; C09_new_layer_mut_ref GREEN; C10_new_layer_send_sync GREEN; 
C11_new_layer_static_ref GREEN; C12_new_layer_box GREEN; C13_new_layer_fn_pointer_param RED; 
D01_type_reflowed_in_existing_layer GREEN;
$ python3 research/prompts/m3-batch10z-tu-code-r1-sonnet-model/scripts/depth_check.py
26 个 case，不同 0
```

（26 个层数 case 里第 16 个我手算期望时多写了一个数，改正后全对；case 含转义引号字符、字节字符、`\u{7d}` 字符、生命周期与标签、字符串续行、`r##"…"##`、`cr#"…"#`、`c"…"`、`b"…"`、`r#fn`、文档注释里的右花括号、块注释嵌套与跨行、字符串里的 `//`、多字节字符。）

### K4-1 判绿的写法里有没有真能遮住钉住路径、绕过叫停的

- **GPU 那条：`use … as`、两层花括号、`self` 组、`extern "C"` 这四类现在判绿（L1 到 L4），它们引进来的名字不叫 `std`、`wgpu`，也不是钉住那四个函数名，遮不住。** 同样写法引进叫 `std` / `wgpu` 的名字（B1 到 B5）、另起模块（B6）、`extern crate`（B7 与 B16）、通配（B8）都判红；`use 别处::某函数 as 钉住的名字` 判红（`sources_brought_in_under_the_name != [expected_source.as_str()]`，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2795`）。`extern "C" { … }` 块里在稳定版只能声明函数与静态，进的是值名字空间，遮不住按路径首段解析的 `std`、`wgpu`。
- **有一种判绿的写法真能绕过，GPU 与崩溃放量两条都有：给被钉住的那一句挂 `#[cfg(...)]`。** GPU 一侧：把钉住那条 `use` 写成 `#[cfg(any())] use super::super::{ device_found_idle_by_a_poll };`，另在卡池模块里定义同名函数 `fn device_found_idle_by_a_poll(_x: &u8) -> bool { false }`（B12，`#[cfg(test)]` 加 `#[cfg(not(test))]` 同形，B13）：扫源码不认属性，看见那条 `use` 的词、来源对得上，判绿；编译时 `use` 被去掉，按名字调的是模块里这一个。崩溃放量一侧：长的那一层里钉住那一行前面加 `#[cfg(any())]`，模块级另放一个叫 `stop_request` 的静态（A01）：用叫停整名的行只有钉住那一行、`let` 绑了一次、类型名只在那一行、层数是第 1 层，全判绿；编译时那一行没有，下面用的 `stop_request` 是静态。同形的小程序 rustc 编过，跑出来叫停被丢掉、按名字调的是遮住的那一个：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/final/rustc-shadow.txt
== cfg_shadow
pinned-like call returned false (outer fn returns true)
== cfg_stop
layer(&|| true) = false (a layer that hands the stop on would return true)
== struct_std
error[E0223]: ambiguous associated type
== type_wgpu2
error: expected type, found variant `PollType`
== type_std2
error[E0223]: ambiguous associated type
```

  两条用例的「拦不住」名单（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2826` 起，`crates/singlefs-checker-tier/src/crash_amplification.rs:5157` 起）里没有「被钉住的那一句挂 `#[cfg]`」这一条（崩溃放量那份写的是「在函数上面加 `#[cfg]` 让这样另写类型的同名函数编进来」，是另一件）。B10（`struct std;`）、B11（别名）判绿但编不过（上面后三行），不是静默的。
- **崩溃放量按类型找层：名单外换写法的新层判绿。** 文档列的 `&(dyn Sync + Fn() -> bool)`（C02）、`&dyn Fn() -> bool`（C03）、`impl Fn() -> bool`（C04）、泛型（C05）、类型别名（C06）都判绿；文档没列、也判绿的还有类型里夹注释（C07）、多一个 `+ 'static`（C08）、`&mut`（C09）、`+ Send + Sync`（C10）、`&'static`（C11）、`Box<dyn Fn() -> bool + Sync>`（C12）。其中 C10 是随手就会写的样子。已钉住的层把类型重排版（D01）判绿，不误红。
- **文档已列的**：钉住那一行之前提早返回（A05）、`match` 的臂（A02）、闭包参数（A03）、`for`（A04）判绿。嵌套 `fn` 的参数叫 `stop_request`（A19）判绿，但嵌套函数捕获不了外层的绑定，不是绕过。
- **会红的**：解构、多空格、`let` 拆行、`let` 与名字之间夹块注释、`if let`（A06 到 A10）数成第二次 `let`，另起一行用叫停整名（A17）、块里另 `use` 同名类型（A18）红；死分支里留一份原样调用（B01）红；原始字符串、块注释里的右花括号、字符字面量、生命周期标签（A13 到 A16、A20）不改层数、判绿，是对的。

推翻 K4-1 判的现象：B12 / B13 / A01 在真用例里判红（我在模型上跑，真用例复核不了）；或 rustc 编不过上面前两个小程序。

### K4-2 新的找层与数 `let` 的判法、块注释与原始字符串：骗得过的与合法产品写法判红的

- **合法产品写法判红或 panic 的**（GPU 那条）：
  1. 属性里和常量 / 静态初始值里的表达式宏：`#[doc = include_str!("x.md")]`（L7）、`const _: () = assert!(1 < 2);`（L8）、`static NAME: &str = concat!("a", "b");`（L9）判红，理由是「条目这一层调宏或定义宏」。用例名里写的故意拒是 `thread_local!` 这类定义条目的宏；这三种展开出来的是表达式、定义不了条目，被一并拒了。要放行只拒条目位置的宏（`macro_rules!` 与后面跟 `{` 或 `(…);` 的条目宏）。
  2. `impl Trait + use<'a>`（精确捕获，Rust 1.82 起，本机 rustc 1.98）：`use` 当成 `use` 语句读，读到分号才收尾，当场 panic（L10，`.expect("use 以分号收尾")`，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2754`）。
  3. C 风格原始字符串 `cr#"say "{" here"#`（Rust 1.77 起）：分词只认 `r`、`br`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2560`），不认 `cr`，字符串里再有一个 `"` 就当成字符串收尾、里面的花括号当成真花括号，panic（L13b）；崩溃放量那条的数层数认 `cr`（`crates/singlefs-checker-tier/src/crash_amplification.rs:5046`），两份不一致。内容里没有 `"` 的 `cr#"…"#`（L13）碰巧判绿。
  4. 原始标识符 `r#mod` 被当成关键字 `mod`（L14），判红：分词把 `r#` 去掉了。
  5. 通配 `use super::*;`（L20）、`#[cfg(any())] extern crate`（L31）判红：前者是用例名里写明故意拒的，后者无害但被拒。
- **合法产品写法判红的**（崩溃放量那条）：钉住那一行或短层的函数体上带行尾注释（A11、B02）、钉住的长层函数体里的字符串里出现 `let stop_request`（A12）判红：整行比、数 `let` 的是文本，不读字面量；行尾注释里写这句同理（读码推的，没造）。新层的参数前有函数指针类型时层名读成空串（C13），本来就该红，只是消息里的层名不准。
- **块注释与原始字符串的处理**：26 个层数 case 全对，包括 `cr`、`br`、`r##`、`c"…"`、字符串续行与嵌套块注释；GPU 的分词对这些（`r#"…"#`、`b"…"`、字符 `'{'`、生命周期、嵌套块注释、字符串里的 `//`）也都对（L11、L12、L15 到 L18、L27 到 L30、L32 判绿）。
- **这些子判法有没有单独的用例钉**（这一问回到 K3）：在扫源码模型上对判法自己造变异，真用例体与真源码一起跑：

```
$ cat research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/replay-gpu-mut.txt research/prompts/m3-batch10z-tu-code-r1-sonnet-model/logs/mut/replay-crash-mut.txt | sed -E 's/\[(.)(.{0,200}).*$/[\1\2/' | cut -c1-120
G1_char_literal_never: 全绿（活）
G2_raw_string_off: 全绿（活）
G3_block_comment_off: 全绿（活）
G4_block_comment_not_nested: 全绿（活）
G5_line_comment_off: 全绿（活）
G6_string_escape_off: 全绿（活）
G7_raw_ident_off: 全绿（活）
G8_byte_string_off: 全绿（活）
G9_lifetime_as_char_start: 全绿（活）
G10_escaped_char_end: 全绿（活）
G11_depth_rule_zero_removed: 红 1 条 ['the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_fu']
G12_macro_alnum_removed: 全绿（活）
G13_expected_source_ignored: 红 1 条 ['the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_broug']
G14_self_group_trailing_comma: 红 2 条 ['the_card_pool_scan_accepts_renames_nested_groups_and_self_th', 'the_card_pool
G15_double_colon_token_off: 红 6 条 ['the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_fu', 'the_card_pool_sc
C1_char_literal_off: 全绿（活）
C2_cr_raw_off: 全绿（活）
C3_block_comment_not_nested: 红 1 条 ['brace_depth_reads_past_raw_strings_block_comments_and_string']
C4_string_escape_off: 全绿（活）
C5_line_comment_off: 全绿（活）
C6_lifetime_branch_off: 全绿（活）
C7_raw_closing_hashes_ignored: 红 1 条 ['brace_depth_reads_past_raw_strings_block_comments_and_string']
C8_escape_char_literal_end: 全绿（活）
C9_ident_skip_off: 全绿（活）
C10_let_pattern_end_only_equals: 全绿（活）
C11_whole_word_neighbor_ignored: 全绿（活）
C12_ref_type_prefix_any_amp: 全绿（活）
C13_layer_name_from_first_fn: 红 4 条 ['the_stop_request_scan_finds_a_new_layer_whose_stop_parameter', 'the_stop_request_scan_finds_a_new_layer_whose_stop_parameter', 'the_stop_request_scan_finds_the_remembered_stop_request_rebo
C14_depth_must_be_1_removed: 全绿（活）
C15_type_name_count_removed: 全绿（活）
C16_pinned_lines_compare_removed: 红 1 条 ['every_layer_handed_the_stop_request_hands_it_on_or_asks_it']
```

  31 个里 `C12` 是我写的无效变异（旧串等于新串），不算；其余 30 个红 8 个、活 22 个。活的：GPU 分词里的字符字面量、原始字符串、块注释（含嵌套）、行注释、字符串转义、原始标识符、字节字符串、生命周期与转义字符的收尾（G1 到 G10）、条目层宏的首词字符判（G12）；崩溃放量里的字符字面量、`cr`、字符串转义、行注释、生命周期分支、转义字符收尾、词跳过、`let` 的模式收尾（只认 `=`）、整词判的前后邻字符、层数必须是 1、类型名行数（C1、C2、C4、C5、C6、C8、C9、C10、C11、C14、C15）。原因是用例体里没有一个被扫的文本带这些写法，真源码里也没有，所以去掉对应分支照绿。这些分支今天的产品代码用不上，是以后写了才会错：判法在文档里声明了「注释、字面量里的 mod、use、花括号都不算」，这句没有一条用例钉。

推翻 K4-2 判的现象：上面任一条「合法写法判红」的变体在真用例里判绿，或任一活变异被真用例拦住（我在模型上跑，真用例复核不了）。

## 6. K5　抄的那一份对齐判

三份同一条判据：`crates/singlefs-core/src/block_device.rs:314`（私有函数，文件后端与 O_DIRECT 后端共用）、`crates/singlefs-harness/src/memory_pool.rs:116`（稀疏盘那一份，文档写明照 core 的写、由 `tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs` 拿文件后端逐个请求对拍钉住）、`crates/singlefs-core/src/mounted_session.rs:315`（预演盘这一份）。逐句读：先判偏移与长度是不是物理块的整数倍（不是 → `Unaligned`，字段 `offset`、`length`、`physical_block_size`），再 `checked_add`，不越过设备末尾放行，越过或溢出 → `OutOfRange`（字段 `offset`、`length`、`device_size`），三份相同；差别只有取值的来源（预演盘的物理块大小在建盘那一刻读一次，设备大小每次读底下那块盘的 `size_in_bytes()`；文件后端把两个值存在结构里）。

实测：K1-2 的差分测试拿真文件后端对拍预演盘，10 万个请求、5 种盘（物理块 512 / 4096，设备 512 B 到 128 KiB）、读 / 写 / 写零各一份：错误成员与字段、读出的字节、底下盘有没有被写、整盘读回，0 个不同（输出在 K1-2）。把判越界的 `end <= device_size` 改成 `<`、把写零前的对齐判去掉，各红一次。

分叉：不同的请求形一个也没有找到。预演与真发在这条判据上不会分叉；会分叉的是预演盘不模拟的东西：写 / 屏障的 I/O 错、O_DIRECT 后端对用户缓冲区地址对齐的要求（真盘在缓冲内部拷到对齐暂存区，请求层面不拒）、真盘读到的随时间变的内容。另有一处潜在差别：预演盘的 `write_zeroes_at` 先分配 `length` 字节的全 0 再记（`crates/singlefs-core/src/mounted_session.rs:434`），文件后端按 4 MiB 一块；发布路径不写零（`grep -rn '\.write_zeroes_at(' crates/singlefs-core/src` 去掉 `block_device.rs` 之后只剩 `make_filesystem.rs` 的三处，即 mkfs），今天不触发，以后预演里若有整段清零，内存要随长度涨。

## 7. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 与用户定案「甲」及判决第三节逐条 | 同义（两处补充） | 14 条逐条对上；补充：出路一永久读不出的后果只写在有根一支（实测没根一支没有这个后果）；「条目层调宏」把属性与常量初始值里的表达式宏也拒了（K4-2） |
| K1-1 预演与真结局 | 没打中（一格偏严） | 攻方 203 格：列推环 20/20 推完放得下、不列 183 格里 182 格推完仍被拒；随机 902 条对比 `A`→放得下 598/598、`R`→仍被拒 304/304，`CMP` 非空 0；我上一轮的推翻条件在新实现上 0 例；例外 1 格（停因是一次准入发布数做满，第二次再试放得下）预演偏严 |
| K1-2 预演不落盘；预演里出错 | 没打中 | 写不到真盘是类型保证；472 次直接调预演前后会话与盘都没动；差分测试 10 万请求 0 不同；真发半路块设备错后第二次推环拒成 `Frozen`；读错注入没造成 |
| K1-3 预演代价 | 没打中；随改动大小线性涨 | 20 到 66 毫秒；内存约 4.6 MB 加每单元约 74 KB（约 2.5 倍于改动内容，放得下时才有）；大盘与长删除队列的数没量 |
| K2-1 措辞、次数、槽数、N、多槽 C、重叠 | 没打中 | 两种说法与实际同义；次数 / 槽数 / N 对独立量与真推环 0 不符；N 是真降的数（自然「C 加被抛弃根」22/22，重叠注入形 N=0 与真推 `stillIso=28/28` 对上）；自然的两槽 C 上没有一格列推环 |
| K2-2 冻结之外 | 没打中（读码加实测） | 预演与真发是同一个函数，分叉只在真盘 I/O 错、盘表不一致、两次调用之间用户又写；读错注入没造成 |
| K2-3 无隔离两处处置 | 没打中；注入级 | 自然历史里 C 在而一个槽都没隔离 0 例；两处各由一条用例钉住（整份文件跑各红 1 条） |
| K3 判别力 | 打中（窄） | 31 + 14 行逐行全红（含上一轮不红的 loop-span1）；上一轮 11 个活变异 6 个现在红、4 个旧串变了改造后红、1 个（C762）仍活；新造 19 个活 7 个：预演盘读回自己写过的块相关 4 个与对齐 / 越界判 2 个今天产品路径等价、`WalkTheQueue` 那一支 1 个没造出被拒点；第二十一块判法子分支（字符字面量、原始字符串、行注释、字符串转义等）22/30 个变异活 |
| K4-1 判绿写法能否遮住 / 绕过 | 打中（窄） | `use … as`、两层花括号、`self` 组、`extern "C"` 本身遮不住；但被钉住的那一句挂 `#[cfg]` 两条用例都判绿，rustc 小程序证实真丢叫停、真换掉被调函数；崩溃放量按类型找层对类型加 `+ Send`、`+ 'static`、`&mut`、`&'static`、夹注释、`Box<dyn …>` 的新层判绿，文档没列 |
| K4-2 找层、数 `let`、块注释与原始字符串 | 打中（窄） | 层数 26 个 case 全对、`cr` 认得；合法写法判红或 panic：属性 / 常量初始值里的表达式宏、`impl … + use<'a>`（panic）、GPU 分词不认 `cr#"…"#`（里面有 `"` 时 panic）、`r#mod`、行尾注释 / 字符串里的 `let stop_request` |
| K5 抄的那一份对齐判 | 没打中 | 三份同一条判据；对拍 10 万请求 0 不同；唯一差别是预演盘写零先分配整段长度，产品路径今天不写零 |

## 8. 没做什么

- 不判别的腿的格、不替主 agent 采纳或出判决；上面的「判」是这一条腿读码加造反例得出的，被攻过零轮。
- 没跑门禁阶段、checker 档测试、带 gpu 特性的 cargo test、崩溃枚举、QEMU。第二十一块 14 行变异与我造的变异都套在从源码机械抽出的扫源码模型上，不是 checker 档里的真用例；「真用例在 checker 档里的行为」复核不了，要主 agent 照合入验证那一趟的做法认。
- 没量：几十万单元的大盘上预演的时间与内存；长删除队列上预演的代价（自然历史里没造出被拒的 unlink，`WalkTheQueue` 那一支只经直接调预演走到）；两进程竞态；G2 乙；tri 判法本身。
- 没造出来的形：读错注入（`Z`，注入的读错那一段没被读到）；预演过而真发因读核换下的那一份被拒；`None` 一支（R 那一槽没有有效根）的随机批（只有攻方 `atk_k1_fill_sweep` 那一格）；两槽 C 加被抛弃根的自然形；C 与被抛弃根重叠的自然形。
- 384 槽盘以外的几何：e3 到 e6 批取了 256、288、320 槽，`CMP` 同样全对，只当线索。
- 扫源码的绕过与合法写法判红：只在模型上量过，没有编进产品代码验类型（`#[cfg]` 的两个用独立小程序验了行为）。
- 随机批里 `flagrNe` 52 条是探针对注入形 R 的期望没有定义（R 另用独立读法核了三个偏移）。
- 不读攻方腿本轮的提示与产出（禁读清单），没读。

## 9. 与规则的出入、清理

- 出入：K5 / K1-2 的差分测试用 `TMPDIR` 指到草稿目录的 `tmp`；其余编出来的二进制（探针、harness 测试二进制、扫源码模型、rustc 小程序）都经 `bash scripts/run-with-memory-cap.sh`（1G 到 12G）起，一次一件、同时不超过 4 件；线程不超过 15。`cargo` 编译在内存包装里；`rustc` 编扫源码模型与小程序没经包装（不跑编出来的代码之外的事，规则不要求）。
- 草稿目录里的拷贝与编译目录已删（删前 `du -sh`）：`tree-a`、`tree-b`、`tree-m`、`tree-m2`、`tree-pm` 各 22M，`target-a` 572M、`target-b` 559M、`target-m` 553M、`target-m2` 553M、`target-pm` 541M、`target-scan` 1.2M、`replay`（rustc 编的小程序）208M、`shadow` 13M、`tmp` 260K；路径都在 `~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet/` 下。留着的是日志、脚本与模型源码草稿（该目录合 9.9M，没有编译目录与仓副本）；会话暂存目录里我放的两份临时文件也已删。
- 冻结副本没改。入库的只有本报告与 `research/prompts/m3-batch10z-tu-code-r1-sonnet-model/`。

## 10. 引文核对

```
$ python3 scripts/cite-check.py research/prompts/m3-batch10z-tu-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree --root /home/fy5090/code/singlefs | tail -1
  ✓ 核了 6 处引文，对上 6 处，没判 0 处（逐处列在上面）
```

脚本只认「引文」加括号里位置那一种写法，上面其余的文件:行号我另用 `sed -n <行>p <文件> | grep -qF <句>` 逐处核过（mounted_session.rs、transaction.rs、block_device.rs、memory_pool.rs、gpu_unit_checks.rs、crash_amplification.rs 共 33 处，行号都对；D16 第 37 行、D28 第 27 行、判决第 44 / 46 / 47 行的句子各 `grep -nF` 命中）。
