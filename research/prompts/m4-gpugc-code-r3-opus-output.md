# m4-gpugc-code-r3 攻方腿（Opus）报告（2026-11-02）

立场：假设第 59、60 批改法与前两轮报出的数是错的，造输入去打。被判对象取冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r3/repo/`（7 份产品代码 sha256 与 `research/prompts/m4-gpugc-code-r3-snapshot/crates-sha256.txt` 逐份对上；主工作区这 7 份此刻也对上，`crates/mutations.tsv` 快照之后被别的会话改过）。下面写成 `crates/…:行号` 的代码行，冻结副本与主工作区同一行。腿没起 cargo、没交任务：凡写「量的」都还是「推的，待主 agent 跑」，命令在「要主 agent 跑的命令」一节。

## 复跑命令与模型文件

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-opus/rerun
```

| 文件 | sha256 |
|---|---|
| `research/prompts/m4-gpugc-code-r3-opus-model/probes/background_gpu_attack_probes_round_three.rs` | `cf651ed042dd39f59936f60459f56889626122360fb9081d5f8b2a102a8dbe79` |
| `research/prompts/m4-gpugc-code-r3-opus-model/tools/apply-probes.sh` | `ce052ebf96bed02775ece31447f9a44b7e65bb4740ad883324c3140ad38da386` |
| `research/prompts/m4-gpugc-code-r3-opus-model/tools/rerun.sh` | `ce8875ef62319ccdfdc9e3abda5ee37123aba63d436541f38813ef3f7f07b2ba` |

`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：冻结副本里拷 `crates`、`prune-server`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 进 `<work>/tree`，`apply-probes.sh` 把探针接进 `background_gpu.rs` 测试模块末尾、副本里 wgpu 一行特性加 `noop`，再分两趟 `cargo test -p singlefs-background-gpu --lib`（替身探针一趟；noop 那条单独一趟、一条线程，它在进程里压 RLIMIT_AS）。模型目录 `out/` 是空的（腿不跑 cargo），比对那一步会打 `same=no-reference`；主 agent 头一次跑出的 `outputs/` 就是参照。模板的 `--selftest` 对这份不适用（`rerun_steps` 换成了要编要跑的步骤）。

## 各格判定一览

| 格 | 攻的东西 | 判定 | 量的还是推的 |
|---|---|---|---|
| T1-1 | 只现枚举一次之后，挂着时卡掉了还按第 4 条拒切换 | 没打中（四种切换形状） | 读码推的；探针 `switch_after_a_participating_card_dropped` 待跑 |
| T1-2 | 泄漏按次累加之后的上限扣法 | 没打中 | 读码推的 |
| T1-3 | 叫起计数新数法：叫起里停、续做里停、排空被拒 | 没打中 | 读码推的 |
| T1-4 | 重切不留已做完的片之后判定逐条照常 | 判定这一格没打中；打中旁边一格：第 59 批第 2 条「重交只数真交出去的」在「重交那一次建缓冲显存不足」上记多 1 | 推的，探针两条待跑 |
| T1-零轮 | 产品建段缓冲用 `create_buffer_init`，wgpu 30.0.1 在它建不成时 panic，条款第 3、4 条「空闲显存不足就重切 / 转给别的卡」在真卡上走不到 | 只报不展开（零轮） | wgpu 源码推的；noop 后端探针待跑 |
| T3-数 | 前两轮与两批报告写进结论的数 | 打中三处：「第 5265 行」已挪到第 5213 行；第二轮判决引的 ATTACK 原行照原样复跑不出（冻结副本已删、产物目录空、脚本自己的过线条件恒不满足）；第一轮判决的复跑命令今天交不进服务 | 量的（文件与目录现查） |
| T3-替身 | 第 60 批回读暂存那条测试的替身与真卡 wgpu 错误范围差在哪 | 打中：替身把「这一段自己那几样缓冲建不成」也记进范围、照常往下走，wgpu 30.0.1 在那里 panic；另三处差别（回读暂存在主机内存那一块堆、范围按线程按过滤器认、暂存缓冲建不成设备丢了） | wgpu 源码推的；noop 后端探针待跑 |
| T2 | 前两轮「读法」「零轮」「核不动」「只抽一次样」没去向的条目 | 列了 9 条 | 读判决与 `checks-owed.md` 现查 |

## T1　「已定」项撑不撑得住（主攻）

条款行号现查 `.claude/kb/decisions/26-后台整理与放置回收.md`：已定项 13 第 2、3、4、8 条在第 246、247、248、252 行（与冻结副本里同一份逐行相同）。

### T1-1　只现枚举一次之后，挂着时卡掉了的切换（没打中）

- 今天的代码：核心层一次校验只调一次 `devices_with_total_memory`（`crates/singlefs-core/src/background_compute.rs:915`），判与取参与的卡读同一份；外壳的 `devices_not_opened_at_the_mount` 不再现查（`crates/singlefs-background-gpu/src/background_gpu.rs:2302`），只答挂载那一刻定下的那份。
- 试的形状（两张开成、都参与的卡，挂载之后 0000:01:00.0 掉了，此刻只在 0000:41:00.0）：不点名不给上限；点名掉了的那张；不点名照挂载时给两个上限；不点名给一个上限。读码推的结局依次是：收下、参与的只有 0000:41:00.0；拒「点名的卡不存在」；拒「上限个数与参与的卡数不等」；收下、0000:41:00.0 得那一个上限。条款第 4 条「GPU 不在……点名的卡不存在……就拒绝挂载（或拒绝切换）」与第 3 条「没给就对第 2 条那份全部独立显卡的列表」都照做。另试了挂载那一刻没开成的卡挂着时掉了：不点名时它不在此刻清单里、不参与；点名它拒「点名的卡不存在」（不是「挂载那一刻没开成」，两种都是拒）。
- 推翻它的观测：探针 `switch_after_a_participating_card_dropped` 四行里有一行与上面不同，或 `use_the_settings_from_the_next_batch` panic。
- 第 59 批改法的等价性（推的）：没开成那份静态之后，「参与的卡里有没开成的」只在那张卡此刻在清单里、又参与时成立，与改前「没开成的那份按同一次枚举过滤」结果逐格相同，所以没造出把别的句子改坏的输入。

### T1-2　泄漏按次累加之后的上限扣法（没打中）

- 今天的代码：进程里同一张卡按次加这一次挂载自己的（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:717`）；新一次挂载的账从进程里记的总数起（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:315`），剩下的地方是上限减已占、不够就是 0（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:73`）。核心层判「上限超过那张卡的显存总量」只比总量（`crates/singlefs-core/src/background_compute.rs:1014`），不扣泄漏。
- 试的形状：两次同开的挂载各泄漏 900（上限各 1000），进程里记 1800，第三次挂载上限 1000：账从 1800 起、剩 0，这张卡照开、不参与、不拒挂载——条款第 3 条「参与的卡的上限连最小的一片都装不下，这张卡就不用……挂载不因此拒绝」；泄漏 7 GiB 的 8 GiB 卡给上限 8 GiB：照第 4 条比总量收下，可用 1 GiB——第 3 条「这个数是上限、不是预留」；同一次挂载丢的时候 `bytes_leaked_by_this_mount` 的 `expect`（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:640`）要「占着的少于开卡时已算上的」才 panic，账从那一片起、每次 `release` 只放自己 `hold` 过的，造不出。
- 推翻它的观测：找到一条放账的路径放了不是这一次挂载 `hold` 的字节（例：切换时重建账而不带开卡那一片）。我读了 `use_the_settings_from_the_next_batch` 两层（`gpus_of_this_host.rs`、`memory_limit_and_chunks.rs`），只换上限、不动账。

### T1-3　叫起计数新数法（没打中）

- 今天的代码：叫起开头 `begin_a_wake`（显式入口 `crates/singlefs-core/src/mounted_session.rs:2346`，后台入口判起跑那一段第 2459 行），一批出错停下时有叫起在做就记 0 并标成停在这一次叫起里（`crates/singlefs-core/src/background_compute.rs:1412`），叫起结束时标着的报 0、清标（第 1455 行），没标的加一；结束时关叫起（第 1444 行）。
- 逐条走过的序列（读码推的，报的 `wakes_while_stopped`）：叫起里停下 → 那一趟 0，之后 1、2、3；可写挂载续做里停下（`crates/singlefs-core/src/mount.rs:7667`，那时没开叫起）→ 头一次叫起 1；叫起里停下、下一次是攒批会话判过起跑而排空被拒（`crates/singlefs-core/src/mounted_session.rs:5612`）→ 1；挂着切到一张卡都用不上、下一次叫起开头换上 → 那一趟 1；叫起里停下之后收到一次通过校验的切换 → 清掉、不报；判起跑说不跑（停着不补跑、定时兜底没到、预算用完）→ 照数。条款第 4 条（第 248 行）「出错停下的那一趟不算，停下之后的下一次叫起是第 1 次」与「在叫起里停下与在可写挂载续做里停下数法相同」逐条对上。
- 每一条开了叫起的路径都走到关叫起：显式入口 `begin_a_wake` 之后没有提前返回；后台入口在 `begin_a_wake` 之前就判完两种「入口在任何读写之前就拒」，之后三条出口（不起跑、排空被拒、跑完）都取报警。所以没造出「上一次叫起的标留到下一次、又报一次 0」。
- 推翻它的观测：一条在 `begin_a_wake` 与取报警之间带 `?` 或 `return` 的路径。

### T1-4　重切不留已做完的片：判定这一格没打中，旁边一格打中

- 判定：显存不足重切时那张卡这一段整段重交，交回的是重切后的整份（`crates/singlefs-background-gpu/src/background_gpu.rs:1962` 只收 `Computed`），判定照 CPU 参照逐条相同（第二轮已量过，这两批没改这一圈；第 60 批只把栈的初值从 `vec![0..entry_count]` 换成同义的写法，第 1905 行）。没打中。
- **打中**：第 59 批第 2 条把「重交了几次」挪到「重交之前那一等等到了」之后（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:213`），紧接着 `submit_once`。`submit_once` 在建缓冲时显存不足会交回 `OutOfMemoryCreatingTheBuffers`，那一次「一次都不交」（产品 `buffers_created_within_the_free_memory` 的文档原话；替身 `crates/singlefs-background-gpu/src/background_gpu.rs:2574`），可这一次已经记成一次重交。
  - 输入：一张卡、校验和那一段四条各 1000 字节的单元一段交；头一次交等超过上限；此后卡上空闲的掉到那一段的一半（别的进程在卡上建了缓冲，条款第 3 条「卡上此刻空闲的不到这个数」）；重交之前那一等等到了。
  - 今天的代码（推的）：重交那一次建缓冲显存不足、没交出去，`resubmissions_after_timeouts` 记 1；外壳照一半的地方重切、从头交完，`recuts_after_running_out_of_memory` 1，结果与 CPU 参照相同，卡上真交出去的是头一次加重切后的 4 次。
  - 要的结果：第二轮判决第三节第 2 条验收「「重交了几次」只数真交出去的重交」（`research/prompts/m4-gpugc-code-r2-main-verification.md:61`）——这里该记 0。
  - 钉它的探针：纯函数 `resubmission_out_of_memory_before_submitting`（预期 `attempts=["submitted", "not_submitted"] timeouts=1 resubmissions_recorded=1 resubmissions_actually_submitted=0`）与外壳 `resubmission_counted_when_free_memory_halves_after_a_timeout`（预期 `returned_as_on_the_cpu=true resubmissions_after_timeouts=1 recuts_after_running_out_of_memory=1`）。推翻它：两条里 `resubmissions` 报 0。
  - 四句：分辨臂——分得开「等到了就数」与「交出去才数」两种数法，正是第二轮那一条要分的；系统当时看得到——`submit_once` 交回的那个错本身就说「一次都不交」，外壳知道；满足的分句——判决验收那一句，不是 D26 的条款句（第 248 行没写记账口径，记账来自五条硬要求「分支必须可观测」与第一轮第 8 条）；改法——第 59 批的改法在这一格上照样中。我的改法：加一要放在「真的交出去了」之后（`submit_once` 交回 `OutOfMemoryCreatingTheBuffers` 时不加，或由交的那一层报「交了」），只在我的推理上过、被攻过零轮。

### T1-零轮（只报不展开）：条款第 3、4 条的「空闲显存不足」在真卡上走不到重切

由 T3-替身那一格的 wgpu 源码推出（推的，没在真卡上量；noop 后端探针待跑）：产品建一段自己那几样缓冲用 `create_buffer_init`（`crates/singlefs-background-gpu/src/background_gpu.rs:1014`），wgpu 30.0.1 在这样的缓冲建不成时 panic，错误范围来不及关、`OutOfMemoryCreatingTheBuffers` 交不出。条款第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」、第 4 条「这一批不给这张卡分活，它那份转给这一批别的参与的卡，不报警」与「建缓冲显存不足、切小重切时……整段从头重交」那三句，在设备本地显存不足时今天的代码做不到（叫起它的线程 panic）。这是这一轮新冒出的形态，前两轮的重切探针都在替身卡上（替身不 panic），没有人看过 wgpu 这一层：写进判决交用户表、标零轮。

## T3　前两轮报出的数字能不能核（主攻）

### T3-数　照原样重跑不出同一个数的三处（量的：文件与目录现查）

1. **「第 5265 行」**：本轮正文第二节「证红第 59 批 18 行、第 60 批 9 行加第 5265 行全部抓到」（`research/prompts/_m4-gpugc-code-r3-body.md:24`），调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md:713`：「第 5265 行证红 4516」。冻结副本与主工作区的 `crates/mutations.tsv` 第 5265 行今天都是「C506 三轮：池级 checker 不判根环区域步长 P 等于第一版常量」，记录第 700 行点名的那一行「七·6 后续批第二轮 构建脚本：特性表里的特性名写错一个字母」今天在第 5213 行（`crates/mutations.tsv:5213`）。照行号重跑那一次证红，点到的是别的会话的 C506 那一行，不是这一路的。命令（只读）：`awk 'NR==5265' crates/mutations.tsv | cut -c1-60` 与 `grep -nF '七·6 后续批第二轮 构建脚本：特性表里的特性名写错一个字母' crates/mutations.tsv | cut -c1-12`，预期分别是 C506 那一行与 `5213:`。出处：上面两行是我在冻结副本与主工作区各跑一次的输出。推翻它：别的会话把表改回、5265 又是这一行。
2. **第二轮判决引的 ATTACK 原行**（第一节「复跑」与第二节 T1 第 1、2、3、4、8 条，例「`ATTACK name=two_mounts_alive_at_once bytes_leaked_by_the_two_mounts=900 …`」）：产生它的命令是 `rerun.sh --frozen ~/.local/share/singlefs/work/m4-gpugc-code-r2/repo`（`research/prompts/m4-gpugc-code-r2-main-verification.md:9`）。今天：`~/.local/share/singlefs/work/m4-gpugc-code-r2/` 只剩 `.owner` 与 `mutations-tsv-after-snapshot.diff`，冻结副本删了；模型目录的 `outputs/` 在仓里是空的（`git ls-files research/prompts/m4-gpugc-code-r2-opus-model` 只列三份：探针、`apply-probes.sh`、`rerun.sh`），主 agent 复跑用的拷贝 `~/.local/share/singlefs/work/m4-gpugc-code-r2-main-check/opus-model/outputs/` 与复跑的 `opus-rerun-1793506907191247042/outputs/` 也都是空目录；脚本自己的过线条件是 ATTACK 恰好 10 行（`research/prompts/m4-gpugc-code-r2-opus-model/tools/rerun.sh:106`：「((rc == 0)) && ((attack_lines == 10))」），而探针六处打印、三处在两遍的循环里，一共 9 行（`grep -c 'ATTACK name=' research/prompts/m4-gpugc-code-r2-opus-model/probes/background_gpu_attack_probes.rs` 输出 6，循环在第 7、49、128 行）。三样合起来：这些原行今天既没有落盘的产物可对，也没有能照原样起的命令，照原样起了也必退 1。能走的路只有 `scripts/agents/snapshot-tree.py m4-gpugc-code-r2` 倒推被判的那几份、其余文件取今天的，编不编得过推不出。这件与 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」第 3 步「复跑的输出与模型目录拷贝留到这一题的下一轮判决写完」对不上：复跑输出在下一轮判决写完之前就不在了。推翻它：任务服务里 2451 的日志还在、能读出这几行（我没读：派发没给这个号）。
3. **第一轮判决的复跑命令**：「`bash scripts/hosts/run-with-memory-cap.sh 16G bash …/tools/rerun.sh … --frozen ~/.local/share/singlefs/work/m4-gpugc-code-r1/repo`」（`research/prompts/m4-gpugc-code-r1-main-verification.md:15`）。今天交服务的命令不许带 `run-with-memory-cap.sh`（`.claude/agent-common.md` 「子 agent 不交任务」那一条，服务当场拒）；`~/.local/share/singlefs/work/m4-gpugc-code-r1/` 只剩 `.owner`；引文核对用的树 `/tmp/claude-1000/…/scratchpad/check/m4-gpugc-code-r1-tree` 不在。数本身还对得上产物：模型目录 `research/prompts/m4-gpugc-code-r1-opus-model/outputs/` 六份在仓里，「ok. 119 passed」在 `mutation-drop-leaks-every-card.out` 里找得到（`grep -l` 现查）。所以第一轮的数是「有产物、命令照原样起不来」，第二轮的是「没产物、命令也起不来」。

没打中的数：第 60 批之后 `-p singlefs-background-gpu --lib` 的「ok. 151 passed」与 `-p singlefs-core --lib` 的「ok. 384 passed」（记录第 709 行），在冻结副本里数 `#[test]`（`grep -rho '#\[test\]' crates/singlefs-background-gpu/src | wc -l` 得 151，`crates/singlefs-core/src` 得 384，主工作区同），对得上（静态数，不含按特性开关的，推的同义）。「335 个测试二进制，共 3229 条测试」是第二台上任务 4517 的输出，那一次被 cancel、树里含别的会话暂存区的改动，静态数不出（三个包 `tests/*.rs` 冻结副本里是 0、5、328 个，`#[test]` 384、190、2681 个），没判。

### T3-替身　第 60 批「回读暂存」测试的替身与真卡 wgpu 错误范围差在哪（读 wgpu 30.0.1 源码，推的；noop 后端探针待跑）

替身 `DeviceOfAChunkInMemoryForTests` 的规矩：卡上空闲字节一个池，每建一份照字节扣，不够就建不成、把一句原因记进最里层那个开着的范围（`crates/singlefs-background-gpu/src/background_gpu.rs:3615`），一个都没开着才记成没接住（第 3618 行），然后照常往下走、交回一份「没建成」的缓冲。测试只拿它判「回读暂存建在范围里还是范围外」。wgpu 30.0.1 与它不同的有四处，头一处打中：

1. **这一段自己那几样缓冲建不成：替身记进范围、照常往下；wgpu panic。** 产品 `buffer_with_contents` 是 `create_buffer_init`（`crates/singlefs-background-gpu/src/background_gpu.rs:1014`）。wgpu 的 `create_buffer_init` 对非空内容建 `mapped_at_creation: true` 的缓冲（`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/util/device.rs:65`），接着取映射区间，失败就 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/util/device.rs:72`：「.expect("Failed to get mapped range for buffer created with mapped_at_creation")」。缓冲建不成时 wgpu-core 给的是一个无效的号，取映射区间时 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/global.rs:2038`：「let buffer = hub.buffers.get(buffer_id).get()?;」交回错，于是那一句 panic。那条显存不足在 panic 之前已经进了范围（建缓冲时设备本地那一份走 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:1104`：「.map_err(|e| self.handle_hal_error_with_nonfatal_oom(e))?;」，报成显存不足、不丢设备），可范围再没机会关：`buffers_and_both_readbacks_created_within_one_out_of_memory_scope` 在 `create_the_buffers()`（`crates/singlefs-background-gpu/src/background_gpu.rs:616`）里就 panic 了，到不了回读暂存（第 618 行）与关范围那一步。替身上同一个输入交回 `OutOfMemoryCreatingTheBuffers`、外壳照它重切；真卡上叫起这一批的线程 panic。
   - 输入（noop 后端上造，不起 GPU）：`wgpu::Device::noop`，进程地址空间上限压到此刻 VmSize 加 32 MiB（noop 在主机内存里 `try_reserve_exact` 建缓冲，`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/noop/buffer.rs:45`），这一段自己的一份缓冲 128 MiB。
   - 今天的代码（推的）：`noop_chunk_buffer_with_contents_short` 一行 `outcome=panicked:Failed to get mapped range for buffer created with mapped_at_creation…`；同一份缓冲换成 `create_buffer`（不映射）的对照 `noop_chunk_buffer_created_unmapped_short` 一行 `outcome=Err:OutOfMemoryCreatingTheBuffers…`。
   - 条款要的结果：第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」、第 4 条按更小的片重切，最小一片都建不成就「这一批不给这张卡分活」——交回显存不足、不 panic。
   - 推翻它：`noop_chunk_buffer_with_contents_short` 那一行是 `returned:Err:OutOfMemoryCreatingTheBuffers…`。
   - 四句：分辨臂——**不分辨**第 60 批那条测试要分的两条臂（回读暂存在范围里 / 范围外）：panic 发生在建回读暂存之前，两条臂一起出局，病根在两条臂共用的前提「这一段的缓冲建不成会进范围、交回显存不足」；系统看得到——wgpu 交回了 `Result`（`get_mapped_range_mut`），是 `create_buffer_init` 里的 `expect` 把它变成 panic，产品自己 `create_buffer(mapped_at_creation: true)` 再判 `get_mapped_range_mut` 的错就看得到；满足的分句——第 4 条「卡上此刻空闲的显存连这一批最小的一片都装不下……这一批不给这张卡分活」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`），与第 3 条那一句（第 247 行）；改法——第 59 批「回读暂存挪进范围」在这一格上照样中（它碰不到这一段自己的缓冲），第 60 批的替身测试也照样绿（替身不 panic）。我提的改法（不用 `create_buffer_init`，自己建映射缓冲、取映射区间失败当显存不足交回）只在推理上过、被攻过零轮，而且它碰到下面第 3 条的设备丢失。
2. **回读暂存与设备本地缓冲不在一个池里**：wgpu-hal Vulkan 后端按用途挑内存位置，`MAP_READ` 不带 `MAP_WRITE` 的是 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/vulkan/device.rs:1010`：「(true, false) => gpu_allocator::MemoryLocation::GpuToCpu,」，首选主机可见、带缓存的那一类（独显上是主机内存那块堆）。替身把两样从同一个「卡上空闲字节」里扣；测试那个输入（空闲 1063 字节、自己那几样 1000、回读暂存各 64）在独显上不会出现：设备本地显存紧不会让回读暂存建不成。第 59 批报告「停下」第 1 条已写过这一点（读同一段源码），这里补上它的后果（推的）：回读暂存建不成只能是主机内存紧，而外壳照「这张卡空闲显存不足」把这一批转给别的卡——几张卡共用一份主机内存，转过去照样建不成，最后报的是 `EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`，名字说的是显存。
3. **wgpu 内部暂存缓冲建不成会丢设备**：`mapped_at_creation` 而不带 `MAP_WRITE` 的缓冲另建一份内部暂存（`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:1164`：「StagingBuffer::new(self, wgt::BufferSize::new(aligned_size).unwrap())?;」），它建不成走 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/resource.rs:1265`：「.map_err(|e| device.handle_hal_error(e))?;」，而 `handle_hal_error` 对显存不足也丢设备（`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:706`：「self.lose(&error.to_string());」）。替身没有「丢设备」这一态。noop 后端上我造不出它（noop 建一份缓冲要临时占两倍主机内存，压上限分不开「设备那份建得成、暂存那份建不成」），只读码。
4. **范围按线程、按过滤器认**：wgpu 把错交给这条线程上、过滤器相同的最里层范围（`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/backend/wgpu_core.rs:665`：「match scopes.iter_mut().rev().find(|scope| scope.filter == filter) {」，前两行取 `ThreadId::current()`）；替身只认最里层、不看线程与过滤器。产品今天只开一种范围、在同一条线程上建，这一处对今天的代码没有后果（推的），只是替身判不到「过滤器写成 `Validation`」那种改坏——第 60 批报告自己也写了这一条判不到。

替身对上了的（noop 探针要量的阳性那一半）：回读暂存建在范围里时，它的显存不足被范围接住、交回 `OutOfMemoryCreatingTheBuffers`（`noop_readbacks_short_inside_the_scope`）；关了范围再建，它进没接住的回调（`noop_readback_short_outside_the_scope` 的 `reports` 多一条 `uncaptured:`）。这两条是第 60 批那条测试与那一行变异判的东西，wgpu 在 noop 上照替身的规矩走的话，第 60 批的「判先后次序」本身成立；打中的是它没覆盖的那一半。

## T2　前两轮写成「读法」「零轮」「核不动」「只抽一次样」的条目，去向（顺带）

现查的范围：两份判决全文、`.claude/kb/checks-owed.md` 里 `C921`、`C942`、`C943` 三行与「快写法」「region_scan.rs」两词（后两词 0 处）。

| 出处 | 条目 | 去向 |
|---|---|---|
| 第二轮 T1 第 6 条 | 量代价那一段建不成不按更小的片重切，判「读法，不违约」 | 没有：条款第 3 条只写「挂载那一刻量代价……的那一段同样切片」，没写建不成怎么办；无欠账行 |
| 第二轮 T2 第 4 条 | 十一项「只是读法」照收，只一份样本 | 没有：没派第二份样本，也没进欠账 |
| 第二轮 T3 第 5 条 | `region_scan.rs`、`checksum.rs` 两处快写法只一份样本，「第三轮或七·4 覆盖时补一份样本」 | 没有：本轮正文没收这两处，欠账表 0 处 |
| 第二轮第四节 | 攻方「没打中的形状」每种只抽一次样；辩方十一项只一份样本 | 没有 |
| 第二轮第四节 | 回读暂存只读码、要真卡 | 有：第 60 批换替身测试，真卡延到七·3 收口（用户 2026-11-01）；本报告 T3-替身指出替身那一半前提与 wgpu 不符 |
| 第一轮 T2 | 两侧约定三种形状攻方造不出，只记「这一轮没打中」 | 没有（没有第二份样本） |
| 第一轮 T3 | 重交 3 次之内卡回来了而判用不了、超时提交回来之后账算错，三种时序造不出 | 没有 |
| 第一轮第四节 | 真池里一个落点两条映射条目没造出来（T3-1 只在纯函数层） | 没有 |
| 第一轮第四节 | 29 行真卡变异行红在哪只读码推 | 有：真卡与真卡变异延到七·3 收口 |

有去向的另有：共用把手插队 C943、删除续做步数不进预算 C942、只供测试的开关 C921、第二轮第五节两条用户已定并写进第 248 行。本轮新冒出的零轮：T1-零轮（`create_buffer_init` panic）、T3-替身第 2、3 条（主机内存紧报成显存不足、暂存缓冲建不成丢设备）、T1-4 的改法。

## 要主 agent 跑的命令

1. 复跑攻方探针（不是重型；两趟 `cargo test -p singlefs-background-gpu --lib`，各带范围声明，写在 `rerun.sh` 里）
   - 命令原样：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-opus/rerun`（判决要引时照 `.claude/rules/three-way-inference.md` 第 3 步把模型目录拷进主 agent 的草稿目录、`--work` 换成那边的新目录再交）
   - 在哪起：`--cwd` 是攻方草稿目录；`--work` 必须是不在或空的目录，它底下会建 `tree/`（约 46M 的仓副本，跑完由主 agent 删）与 `outputs/`
   - 跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 是过；日志里 `name=rerun_summary rc_stand_in=0 rc_noop=0 attack_lines=11`；两次比对都打 `same=no-reference`（模型目录 `out/` 空）。我跑前写下的预期（逐行核对，对不上的就是推翻）：
     - `resubmission_out_of_memory_before_submitting`：`attempts=["submitted", "not_submitted"] timeouts=1 resubmissions_recorded=1 resubmissions_actually_submitted=0`
     - `resubmission_counted_when_free_memory_halves_after_a_timeout`：`returned_as_on_the_cpu=true resubmissions_after_timeouts=1 recuts_after_running_out_of_memory=1`，`chunks` 头两笔是 `…/submitted/WaitForTheGpuExceededTheLimit…`、`…/not_submitted/OutOfMemoryCreatingTheBuffers…`
     - `switch_after_a_participating_card_dropped` 四行：`unnamed_no_limit outcome=admitted participating=["0000:41:00.0"]`；`naming_the_dropped_card outcome=refused:…`；`unnamed_two_limits outcome=refused:MemoryLimitCountDifferentFromTheParticipatingDevices…`；`unnamed_one_limit outcome=admitted participating=["0000:41:00.0"]`
     - noop 五行：`noop_chunk_within_the_room outcome=Ok reports=[]`；`noop_readbacks_short_inside_the_scope outcome=Err:OutOfMemoryCreatingTheBuffers…`；`noop_readback_short_outside_the_scope captured_by_the_closed_scope=None` 且 `reports` 里有一条 `uncaptured:`；`noop_chunk_buffer_created_unmapped_short outcome=Err:OutOfMemoryCreatingTheBuffers…`；`noop_chunk_buffer_with_contents_short outcome=panicked:Failed to get mapped range for buffer created with mapped_at_creation…`
   - 编不过的：探针只经 `rustfmt --check` 判过语法（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-opus/syntax/rustfmt.txt`：只有排版差、没有解析错），类型没编过。编不过或 noop 那一趟在压上限时整个进程退出（不是 panic 被接住），把 `outputs/probes-*.out` 的路径与任务号交给承接的攻方腿改。

## 没打中的形状与取样范围

- T1-1：四种切换（上表）× 卡掉了一张；另读了「没开成的卡挂着时掉了」两种（点名 / 不点名）。都只是读码与一份待跑的探针，一次抽样。
- T1-2：两次同开泄漏累加超过下一次的上限；泄漏接近总量时上限等于总量；丢时 `expect` 的前提；切换时换上限不动账。读码，一次抽样。
- T1-3：六条序列（叫起里停、续做里停、排空被拒、切到一张都用不上、停着收到切换、判起跑说不跑）；三个入口的每条出口都核了有没有取报警。读码，一次抽样。
- T1-4 判定那一格：前两轮量过，这两批没改，没再攻。
- T3-数：第 59、60 批两份报告里的计数（第 59 批变异 16 行、第 60 批 15 行与 621 行锚点）没逐个核；`#[test]` 两个数静态对上。

## 这条腿自己的限度

- 一行 cargo 都没跑：T1-4 的打中、T3-替身第 1 条与 T1-1 的「没打中」全靠探针，探针只过了语法检查。noop 后端不是 Vulkan：它证的是 wgpu 前端与 wgpu-core 这一层（`create_buffer_init` 那一句与错误范围怎么接错），不证 Vulkan 驱动在设备本地显存紧时一定报 `VK_ERROR_OUT_OF_DEVICE_MEMORY`（有的驱动可能超额分配、只变慢，那时连 panic 也不出现；推的）。
- noop 探针在测试进程里压 RLIMIT_AS：上限算错会让进程在别处分配失败直接退出，所以单独一趟、一条线程跑，压的窗口里只做建缓冲。
- 「没打中」每格只抽了一次样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」只算「这一轮没打中」。
- T3-数第 2 条没读任务 2451 的日志（派发没给号）。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没在冻结副本里改任何东西；仓副本由 `rerun.sh` 在 `--work` 底下建，腿自己没建副本、没建编译目录。
- 没读禁读清单里的四样。
