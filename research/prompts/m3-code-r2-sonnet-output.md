# m3-code-r2 云端辩方腿报告（Y1：复核第一轮判决，替被改的一方辩护）

日期 2026-10-01。立场：辩方，替 H1–H8 之前的实现与测试辩护；每条同时回答「同样的打法打不打得中所有替代方案」。
只读与推导；唯一跑过的非读命令是草稿目录里的三段 python ctypes 小探针（`fallocate` 打洞，不编不跑仓里的测试），脚本与原样日志在模型目录 `research/prompts/m3-code-r2-sonnet-model/`。
被判代码读冻结副本 `/tmp/claude-1000/m3-code-r2/tree/`，引代码写副本绝对路径加副本行号；引 kb 与规则写仓内路径加那份文件自己的行号（每处都 `grep -nF` 现查过）。

## 0　读的对象对上了没有

```
$ cd /tmp/claude-1000/m3-code-r2/tree && sha256sum -c research/prompts/m3-code-r2-snapshot/crates-sha256.txt | 取末列 | sort | uniq -c
     17 OK
$ grep -rn 'DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot' --include=*.rs crates | wc -l   （副本）
30
$ grep -rn 'DevicesBehindTheCurrentVersionAndMissingItsUnits' --include=*.rs crates | grep -v 'MissingItsUnitsOrItsRoot' | wc -l
0
```

第一轮判决原文 63 行、正推腿 187 行、攻方腿 334 行、实现员报告 196 行，整份读过。下文「判决第 N 行」指第一轮判决第二节表里的 `#` 列。

## 1　14 行逐条复核

判定三档：站得住 / 够不着 / 同样打中别的臂。一条可以同时挂两档，写在证据里。

| 判决 # | 辩方判定 | 证据 |
|---|---|---|
| 1　D 重读一次（Z2-④、Z4-①D） | **站得住**；**同样打中别的臂**（单元那一半，条款字面不要求，判决没点到） | ① 重读不是实现员自拿主意：D16 已定项 1 那一行写的是「挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次」（`.claude/kb/decisions/16-发布语义.md:37`），现行那一版的根槽正是这个进程写过、FUA 返回过的槽；代码在「let reread = reading_other_than_the_version_root()?;」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:3444`）。改法 C 那一句只写「四个入口同一判」（`.claude/kb/decisions/18-块里携带什么信息.md:323`，整句「用户定改法 C——落后且缺现行那一版的单元，或现行那一版的根写在这块盘上而那一槽里不是它，才拒，四个入口同一判」），没写重读，所以是「条款没说、代码做了、测试没钉」，缺口是真的。② 变异不是等价变异：实现员 8 行逐条证红（实现员报告「证红」一节表第 1、2 行，转述），攻方副本上去掉重读全包 94 个二进制照绿（转述，第一轮攻方报告 Z4-① 表）。③ **同形没被点到**：单元那一半一次读不出就直接记缺，「.is_some_and(…bytes_on_this_device == unit.bytes)」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:3337`）无重读、无用例；D18 第 314 行字面是「落点读不出或字节与这个进程验得过的那一份不同，才拒」（`.claude/kb/decisions/18-块里携带什么信息.md:314`），所以不是实现错，但「一次瞬时读错不该把健康的落后盘拒成缺东西」这条理由按同一逻辑也罩得到它。④ 对替代方案：「重读 2 次」「重读 N 次」被 n = 1、n = 8 那两格同样放过（两格只钉「至少重读一次」），打法对它们不分辨，这是用例的口径不是判决的错。 |
| 2　A「只查盘 0 的根槽」 | **站得住** | 区域归属表是 `[0, 1, 0]`：「region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/common/mod.rs:49`），旧用例的 P 是 txg 6（区域 0、盘 0），根在盘 1 的那一形旧用例一格都没有；判不出来的错实现是真的（攻方量到 168 格 checker 红，转述）。守卫在「if device_of_the_ring_slot != device {」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:3418`）。替代方案（只查盘 1、取 `region mod 2`）被同一类用例同样分辨，打法不偏袒任何一臂。 |
| 3　E「自证得过就算是它」 | **站得住** | 旧用例的根槽是 mkfs 清过零的槽，两次读都是没自证过，分不出「读坏」与「上一圈的旧根」；攻方在转圈表上量到 226 格红（转述）。H2 转圈用例用 txg 27、断言 `AnotherRoot`（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:1048` 起那条）。替代方案「只比 txg 不比实例」（B）今天分不出，判决已把它记为没打中（第 11 行），不矛盾。 |
| 4　长度 0 的写零（Z3-a） | **站得住**，但只在接口一层「够得着」；改法（H5）没改坏合法行为；「三个后端逐项相同」只钉了两个 | ① 够得着的层次：契约只写「写零：把 `[offset, offset + length)` 整段变成 0。偏移与长度同样要按物理块对齐、不越过设备末尾。」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:127`），长度 0 对齐、在范围内，是合法请求，契约没说它的结局；调用方今天没有：全仓 `write_zeroes_at(` 的生产调用只有 mkfs 两处，长度是几何里的字节数（「device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;」，`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/make_filesystem.rs:499`）；`journal_ring_bytes` 是公开参数，我没查到拒 0 的校验（没查全，不当成有或没有）。所以「够不着」对今天的流量成立，「接口上三个后端对同一合法请求结局不同」也成立，判决写的正是后一句，站得住。② 它是这一批自己引进的回退：打洞之前拆块循环对长度 0 一次不转、交回 `Ok`（「while written < length {」，`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:360`，现在是退路共用的那一份），`O_DIRECT` 后端同一循环（「write_zeroes_in_chunks(offset, length, scratch.as_slice()」，`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:714`）。H5 让文件后端回到原来的结局，不是新定一条政策，所以不会改坏合法行为：「if length == 0 {」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:442`）在 `check_request` 之后，未对齐、越界的零长度请求照拒，与 `O_DIRECT` 后端同序。③ 缺口：对拍用的 `requests()`（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs:25` 起）里仍没有长度 0，而这份对拍正是文件后端对稀疏盘逐请求比结局的地方（「verdict_of(sparse_device.write_zeroes_at(offset, length))」，同文件:153）；新单测（`block_device.rs:1227`）只比了文件后端与 `O_DIRECT` 后端两臂，稀疏盘那一臂没钉。实现员报告把它列进「停下交主 agent 的」第 1 条（转述）。④ 对替代方案：「在 `check_aligned_and_in_range` 里三个后端一起拒长度 0」同样消掉三方不一致，同一打法不分辨两条；H5 动得最少，且不动 `read_at`/`write_at` 对空缓冲的结局。 |
| 5　用例红时留下镜像（Z3-d） | **站得住**；**同样打中别的臂**（同形在别处没改，且守卫罩不住被杀） | 规则原文「建镜像的装置在测试结束时自己删掉它：成功、失败、panic 都删。」（`.claude/singlefs-ai-sop/rules/command-safety.md:147`）；H6 的守卫在「struct ImageFilesRemovedOnDrop {」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs:43`），建成一份记一份（「images.paths.push(path);」，同文件:79），不会删别人的同名文件；各用例镜像名带进程号与序号，一个文件一条用例，并行不撞。**同形没改**：`block_device.rs` 的 `mod tests` 里还有 9 条用例只在末尾删（awk 数出：行 794、857、939、1021、1080、1321、1370、1420、1483 各一处「std::fs::remove_file(&path).expect("清理镜像");」，例 `/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:857`），其中 939、1021、1080 是同一批打洞改动自己新加的；`mkfs_bytes_judged_by_the_checker.rs`、`data_unit_allocation_and_write.rs` 也是末尾才删（例「std::fs::remove_file(path).expect("清理镜像");」，`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:101`）。量级：判决点名的是约 1.5 GiB 一次的那一条；`/tmp` 今天残留 `singlefs-step67-*` 共 77G（命令输出见本节末），记录里主 agent 早已记着「`/tmp/singlefs-*` 还有 334 项、合计 79G」（`records/2026-09-30-里程碑三开工第一轮.md:212`），那一堆的来源我没分清是红了留的、被杀留的、还是别的会话正在用的，不替它归因。Drop 守卫在进程被杀（超时 SIGKILL、`abort`）时不跑，这不是 H6 的回退，是该规则那一句「Rust 用 Drop 守卫」自带的射程。替代方案「建完立刻打开再 `unlink`」罩得住被杀，但这条用例末尾要按路径读 `metadata`，要改成 `fstat`；两条路都不被「红了留文件」这一打法偏袒。 |
| 6　SAFETY 里「即使越界内核也只返回 errno」（Z3-f） | **站得住**（旧句不成立）；**新句也不精确**，H7 引进了一处小错 | 旧句不成立：本机 ext4 上越过文件末尾打洞交回 0（判决第 6 行已现查）。**新句**写成「越过文件末尾的打洞内核交回 0、什么都不做，同样不碰内存。」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:388`）：「什么都不做」只对整段落在文件末尾之外那一形成立。我在 ext4 与 tmpfs 上各量了跨过文件末尾那一形，在文件内的那一半被打成了 0（`research/prompts/m3-code-r2-sonnet-model/logs/punch-straddling-eof.log` 原样：`straddle rc=0 errno=0 in-file part zeroed=True head untouched=True; beyond-EOF rc=0; size=65536`，两个文件系统各一行）。不碍内存安全，也到不了这里（`check_request` 先挡越界，设备大小等于文件大小）；但注释说了一个现象上不对的句子。unsafe-allowances 那一格读的是描述符、偏移、长度三类词（据第一轮正推腿引的表头，转述），不判这一句对不对。 |
| 7　`EINTR` 不重试（Z3-b） | **站得住**（吞掉 `EINTR` 不改坏合法行为、不会卡死）；**够不着**（在本机两种文件系统上量不出 `EINTR`）；与用户那一句字面有出入，见下「提给主 agent 的一条」 | ① 打洞在普通文件上会不会真出 `EINTR`：`man 2 fallocate` 的错误表里有「EINTR  A signal was caught during execution; see signal(7).」（本机 `man 2 fallocate`，命令输出，非仓内文件）。我在 ext4（`/`）与 tmpfs（`/dev/shm`）上用 CPython 装的 SIGALRM 处理器（装法不带 SA_RESTART）、间隔 50 微秒（小段打洞）与 20 微秒（整文件打洞）的定时器，让信号在 `fallocate(PUNCH_HOLE\|KEEP_SIZE)` 执行途中到达：1 536 + 768 次小段打洞、各 8 次整文件大打洞，`EINTR=0`、`other=0`（原样日志四份在 `research/prompts/m3-code-r2-sonnet-model/logs/eintr-*.log`，例 `punches_ok=1536 EINTR=0 other=0`；每个只抽了这一次样，是「没复现」不是「不会」）。所以今天这批测试机上这条重试走不到，只被测试开关进入；它防的是别的文件系统（我没有 NFS、FUSE 可测，没读内核源码，本机 `/usr/src` 只有 headers）。② 重试正确性：打洞幂等，已打过的部分再打一次结果相同，所以不论 `fallocate` 在返回 `EINTR` 之前动没动过文件，原样重打都对；注释「`fallocate(2)` 在动文件之前就返回了，原样再打一次洞」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:170`）这半句「在动文件之前」我核不了（没内核源码；man 页只说「信号被捕获」），但结论不依赖它。③ 卡死：循环在「被打断就原样再打，轮数的上界是这期间送到的信号数」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:446`）；一次 `EINTR` 要有一个已送达并跑过处理器的信号，所以没有信号风暴就不会一直转；信号风暴加一个每次进门就查信号、且从不推进的文件系统才会活锁，这与 std 的 `write_all_at` 对 `Interrupted` 的口径相同（后一句是转述，本机没装 rust-src，没核）。**不是新引进的卡死面**：旧实现在这一点上是直接报错。④ 分路：「Some(libc::EINTR) => HolePunchFailure::InterruptedBySignal,」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/block_device.rs:181`）位于 `EOPNOTSUPP`/`ENOSYS` 臂之后、`Some(_)` 通配之前，别的 errno 的分路没变（攻方 Z3-b 七个 errno 表，转述）。⑤ 对替代方案：「照报」（H8 之前）与「有限次重试再报」两条同样被「该文件系统真会出 `EINTR`」这一前提约束，打法不偏袒；有限次重试会把「被信号打断」变成假失败，比无界重试更坏。 |
| 8　可写挂载上的缺根一半（Z1 ①） | **站得住**（用户已定，没有新证据翻它） | 用户 2026-10-01 弹窗定「留着，条款补句、补用例」（`records/2026-09-30-里程碑三开工第一轮.md:210`）。判决让补的 H3 用例要两次连续读错才走到：「那一槽真读坏的恰好两次」断言在 `/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:1201` 起那条里；它是新成员「BehindTheSelectedVersionAndItsRootNotInItsRingSlot {」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:707`）的第一条有用例的入口。代价侧：正推腿推过「`Written` 那一支在无故障历史上空转」（第一轮正推报告 C① 第 4 点，推的），攻方 17 条流 364 911 个合法崩溃态零误拒（转述）。辩方没有更多可补。 |
| 9　成员改名（Z1 ②） | **站得住**；新名**比旧名说得清**，但有两处没收干净 | 旧名的最强辩护：kb 里「自证单元（journal 记录 / 根槽 / 系统配置槽）不用这张表」（`.claude/kb/decisions/18-块里携带什么信息.md:184`）把根槽也称作单元，按这个宽读「缺它的单元」罩得住根。站不住：代码自己的枚举把「单元」留给非根那一半（「UnitsMissing(Vec<(TransactionUnit, SlotNumber)>),」，`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:637`）、根另立一个成员（「RootNotInItsRingSlot(VersionRootNotInItsRingSlot),」，同文件:640），名字在代码里读出来就是窄的；规则判据是「判据：不看注释、不看调用点、不看实现，光看这个名字，说不说得出它是什么、做什么、不做什么。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:15`）。新名 `…MissingItsUnitsOrItsRoot` 把两半都写出来，与枚举两个成员一一对得上；30 处、8 个文件的改名是纯改名（`history.rs` 只是匹配臂与标签字符串：「"DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot"」，`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/src/history.rs:2392`；`research/results`、`research/scripts`、`.claude/gate.d` 三处下旧名零文件：`grep -rl … \| wc -l` 输出 0）。**没收干净**：① 同一个概念两个说法：可写挂载侧的同胞成员叫「BehindTheSelectedVersionAndItsRootNotInItsRingSlot {」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-core/src/mount.rs:707`），新名里写的是「缺根」（`MissingItsRoot`），而这个成员在根槽里是另一条自证得过的根时也进，那时根没「缺」只是不在它的槽里；规则里有「一个语义概念全仓一个名字」（`.claude/singlefs-ai-sop/rules/code-discipline.md:31`）。② 同一个文件里的旧注释没跟上：「有盘落后于现行那一版又缺它的单元」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/src/model_comparison.rs:780`）。③ `DeviceBehindTheCurrentVersion` 不改的理由（缺什么由 `lacking` 的类型说）成立，与判决第 9 行「三处」一致。替代名（例 `…LackingWhatItNeeds`）说得更少，不比新名好；要消掉 ① 只能把新名改成 `…MissingItsUnitsOrItsRootNotInItsRingSlot` 那一类更长的名字，规则里「长度不设限」（`.claude/singlefs-ai-sop/rules/code-discipline.md:24`）不拦。 |
| 10　打洞成功那一支不另加开关 | **站得住**，但判决给的理由答的是另一件事，该换一句 | 规则字面：「每条分支必须能被测试强制进入。」（`.claude/rules/fs-design.md:146`）、「留一个只供测试使用的开关，能无视判据直接走指定分支。」（`.claude/rules/fs-design.md:147`）、「进不去就等于没测」（`.claude/rules/fs-design.md:148`）。判决的理由是「那条用例本身就证明这一支被进入了」，这句答的是「环境支持时进得去」，没答「强制」。真正的理由：这一支的判据是 `fallocate` 的返回，「强制进入」的开关只能是「假装它返回 0」，那等于跳过系统调用，什么真行为都没测到（只剩计数），而 mkfs 用例用真 `fallocate` 打真洞并比 `wchar`、实占块数、计数三样，比一个假开关钉得更多：「录制层记的每一步写零都落成了打洞」（`/tmp/claude-1000/m3-code-r2/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs:133`）。另两条分支的开关本来就是「假一个 errno 送进真判据」，落到真退路上；H8 新加的开关同理，第二次重打真调 `fallocate`，等于在测试里又走了一遍打洞成功那一支。不支持打洞的环境里这一支没法进、mkfs 用例判红，是环境前提，已写在用例文件头。同一打法对替代方案：「加一个假装成功的开关」只让计数可测，不让行为可测，两个方案都买不到「强制进入真打洞」。 |
| 11　Z2-①②③、Z3-b 其余、Z3-e、Z4-②③④ 没打中 | 不涉及，辩方不动 | 判决写「只抽了一次样，不支撑『没有错』」，与「一条腿只抽一次样不算一次观测」那一节相合；这一行没有拿「没打中」去支撑任何采纳，没有可辩的。 |
| 12　Z2-⑤ W1 / W3 放得过 | **站得住**（不改） | 判决依据是「树号只在第一个文件那一版发」（推的），现有可达历史分不出；辩方没有反例。若以后多文件建树，这一行要回头补用例，判决已记进 C586「怎么拦」。 |
| 13　Z2-③ 撕裂系统配置槽被拒 | **站得住**（不属于这一批） | 攻方自己写明是改动之前就有的拒因（`NewerStateStillUnreadableAfterOneReread`，转述）；与 H1–H8 无关。 |
| 14　Z3-c 真「不支持」文件系统没造出来 | **站得住**（照实记） | 本机命名空间被拒、能写的只有 ext4 与 tmpfs，两者都支持打洞；我自己的探针同样只在这两个文件系统上跑。 |

第 5 行残留量的命令与原样输出（辩方只读，没删任何文件；这些里有别的会话正在跑的）：

```
$ du -ch /tmp/singlefs-step67* 2>/dev/null | tail -1
80G	total
$ ls -d /tmp/singlefs-step67* | wc -l
140
$ ls -d /tmp/singlefs-core-*.img | wc -l
17
$ du -ch /tmp/singlefs-core-*.img | tail -1
99M	total
$ awk '… remove_file(&path).expect("清理镜像") 计数 …' crates/singlefs-core/src/block_device.rs
tail-remove sites in block_device.rs: 9
```

## 2　Y1 逐问回答

| 问 | 答 | 见 |
|---|---|---|
| 长度 0 今天没有调用方，那条还算打中吗 | 算：契约允许长度 0，三个后端的结局不同是接口层的事实；够不着的是今天的流量，不是接口。H5 是回到打洞之前的结局 | 第 4 行 |
| `EINTR` 在普通文件打洞上会不会真出现 | 本机 ext4 与 tmpfs 上量了 2 304 次小段加 16 次整文件大打洞，信号都在调用途中到达，`EINTR` 0 次；man 页错误表里有它；其他文件系统没测、没内核源码 | 第 7 行 |
| 长度 0 交 `Ok` 与「请求要对齐、在范围内」的契约合不合 | 合：长度 0 对齐、在范围内，整段变成 0 对空段平凡成立；`check_request` 仍在前面，未对齐或越界的零长度请求照拒；契约文字没写长度 0 的结局，H5 没把这一句补进 trait 文档 | 第 4 行 |
| `EINTR` 无界重试会不会卡死 | 没有信号风暴就不会：每次 `EINTR` 要一个已送达的信号；打洞幂等，重打不会改坏已打好的部分；旧实现是直接报错，不是「更不会卡」而是「更早失败」 | 第 7 行 |
| H4 新名字是不是比旧名说得清 | 是（两半都写出来，与枚举两个成员对得上）；有两处没收干净：同胞成员写「根不在它的槽里」而新名写「缺根」、`model_comparison.rs:780` 旧注释没改 | 第 9 行 |
| 第二节第 10 条「打洞成功那一支不另加开关」站不站得住 | 站得住，理由要换：强制进入的开关只能是「假装返回 0」，测不到真行为；判决写的「那条用例本身就证明被进入了」答的是环境，不是强制 | 第 10 行 |

## 3　同样的打法打不打得中所有替代方案

| 被打中的一条 | 替代方案 | 同一打法打得中吗 |
|---|---|---|
| 长度 0（第 4 行） | ① H5：文件后端交回 `Ok`；② 三个后端一起拒长度 0；③ 不改 | ①② 都消掉三方不一致，打法不分辨；③ 被打中。H5 动得最少、与打洞之前一致 |
| `EINTR`（第 7 行） | ① 照报（H8 之前）；② 无界重试（H8）；③ 有限次重试再报 | 打法的前提是「该文件系统真出 `EINTR`」，本机量不出；一旦成立，①③ 都被打中（假失败），② 不被打中。② 与用户「别的错照报」那一句字面有出入，①③ 没有 |
| 留镜像（第 5 行） | ① Drop 守卫（H6，规则字面）；② 建完立刻 `unlink`；③ 末尾删（H6 之前） | ③ 被打中；① 在被杀时仍留（规则自带的射程），② 不留；同形在 `block_device.rs` 另 9 条用例与两份 harness 文件上，H6 一条没动 |
| SAFETY 一句（第 6 行） | ① 旧句；② H7 新句；③ 只写「越界由 `check_request` 挡在前面」 | ① 被打中；② 对跨文件末尾那一形同样被打中（我量到文件内那一半被清成 0）；③ 不说内核行为，不被打中 |
| 缺根重读（第 1 行） | ① 不重读（H1 之前）；② 重读一次；③ 重读 N 次 | ① 被打中；②③ 都被 n = 1、n = 8 两格放过（只钉「至少重读一次」）；单元那一半有同形，判决没点 |

## 4　什么现象会推翻各行的辩方判定

| 判决 # | 推翻它的现象 |
|---|---|
| 1 | 找到条款明写缺根那一半不重读（与 D16 第 37 行冲突的句子）；或 D 变异在某个盘面上与原样不可分（等价变异）；单元那一半的同形若被证明走不到（读单元落点从不瞬时出错）则「同样打中别的臂」撤回 |
| 2、3 | A、E 在原样实现的盘面上不再被旧用例放过（旧用例其实造过根在盘 1 或转圈的盘面） |
| 4 | 在目标文件系统上长度 0 的 `fallocate` 交回 0；或契约改成禁止长度 0（则 H5 要换成在 `check_request` 里拒）；或找到一个今天的调用方长度会是 0（则「够不着」撤回） |
| 5 | 另外 9 条用例 + 两份 harness 文件的末尾删，实际不会留下东西（例如这几条的镜像从不落盘）；或 `/tmp` 里 80G 全是别的原因，与「同形没改」无关（这一条我没归因，只列了形） |
| 6 | 在文件系统上，整段落在文件末尾之外的打洞也改了文件内的字节；或跨文件末尾那一形的文件内那半没被清（我量的是 ext4 与 tmpfs 各一次） |
| 7 | 在 ext4 或 tmpfs 上量到 `EINTR`（则「够不着」撤回）；或在某个文件系统上量到 `EINTR` 之前已动过文件而打洞不幂等；或 std 的 `write_all_at` 其实不重试 `Interrupted`（则「同一口径」那一句撤回） |
| 8 | 造出一个合法历史，可写挂载在无故障时被缺根那一半拒；或用户撤回 2026-10-01 的决定 |
| 9 | 规则里加了「改名代价大可不改」的例外；或 `DevicesBehindTheCurrentVersion…` 这个名字被拿去存别的东西，使新名读出的含义不再是「缺单元或缺根」 |
| 10 | 找到一个办法让「假装返回 0」的开关测到真行为（例如开关让 `fallocate` 打到另一个真文件上）；或不支持打洞的环境成为本项目的日常环境 |
| 11–14 | 同判决给的推翻条件，辩方没加 |

## 5　提给主 agent 的几条（不替主 agent定）

1. 判据原文「Y1 判出第一轮某条打中够不着、或改法改坏了合法行为 ⇒ 那一条交用户定」：字面上，第 4 行（今天没有调用方）与第 7 行（本机量不出 `EINTR`）属于「够不着」，但第 4 行接口层够得着、第 7 行的 H8 不改坏合法行为；没有一条查到「改法改坏了合法行为」。交不交用户由主 agent 判。
2. 第 7 行：H8 与用户 2026-09-30 的话（「时退、别的错照报」，`records/2026-09-30-里程碑三开工第一轮.md:128`）的出入是确定的，收益（防某个没测过的文件系统出 `EINTR`）是推的；若要对用户有交代，一句话确认就够。
3. 第 4 行的缺口：文件后端对稀疏盘的对拍文件里加 `(DeviceOffsetInBytes(0), 0)` 一项就能把三方钉齐，实现员已列为下一批。
4. 第 6 行：H7 新句要再改一次。
5. 第 5 行：判决只改了一份用例；同形的 9 + 若干份与 80G 残留要不要并进下一批，主 agent 定。
6. 第 9 行：`model_comparison.rs:780` 旧注释；kb 里 D18 已定项 11 的旧名（背景材料第二节 H4 一条已列，归主 agent）。

## 没做什么

- 没编没跑仓里的任何测试，没跑 harness 档、core 的 lib 单测；变异行没在副本上重证（Y2 ⑥ 的事）。我只核了那 2 行整行换掉的变异锚点在副本里各命中一次（`mutations.tsv` 第 1485、1489 行的原文：`return write_zeroes_in_chunks(` 在 `block_device.rs` 里只命中 478 一行，`return Err(BlockDeviceError::InputOutput(punch_error));` 20 空格缩进的整行命中 1 次），没证红。
- 没读内核源码（本机没有），没测 NFS、FUSE、xfs、btrfs；`EINTR` 的探针只抽了一次样，是「没复现」。
- 没替主 agent 撤销或维持第一轮任何一条判决。
- 没判 H1–H3 新用例自己的造法与断言是不是钉住了该钉的（Y2 ①）、守卫在被杀时的行为（Y2 ④）、`history.rs` 匹配臂的行为（Y2 ②，只看了改名那几处 hunk 是纯改名）、第 Y3 节三个入口。
- `/tmp` 里的 80G 残留我没动、没归因。
- 我自己的探针文件与日志在 `research/prompts/m3-code-r2-sonnet-model/`；草稿目录 `/tmp/claude-1000/m3-code-r2-sonnet/` 里只有那三份脚本的原件，没有编译目录、仓副本、工作树；探针建的镜像文件探针自己 `unlink` 了（`ls` 草稿目录与 `/dev/shm` 核过无残留）。
