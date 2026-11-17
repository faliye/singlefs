# m4-gpugc-one-vulkan-instance-code-r1：云端攻方腿报告（Opus，2026-11-08）

被判代码：冻结副本 `~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo/`。三份被判文件与主工作区逐字节相同（`cmp`，这一条腿现跑，三份都打 `same`），下面的路径写仓内相对路径，行号在两边一样。

本腿写了模型，模型目录 `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-opus-model/`。腿不交任务，模型一次都还没编、没跑：**报告里标「模型断言」的数都是「推的，待主 agent 跑」**，命令在「要主 agent 跑的命令」一节。

复跑命令（在新 shell 里起，`<草稿>` 是工作根下一个新目录）：

```
task-server/task submit -- bash research/prompts/m4-gpugc-one-vulkan-instance-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo --work <草稿>
```

模型文件的 sha256（`sha256sum`，这一条腿现跑）：

```
07f758dbfcdfe206918762891df3513ef804573b91676985bd108f2cada9532f  attack_long_lived_instance_on_a_real_gpu.rs
cd2d588bad4159cd80eda588ab3c954bb4f3fce5a7383034b02d81e0addb8609  attack_tests_appended_to_background_gpu.rs
cd4c00261244b02cb56ea3112dc01310154eed6eccd5b9915408f9ebe2589eba  mutants/present_now_on_another_instance.original
41a970765182f9c258181e3ae36fab1c816ef41da04201d1e253b86a343b15f0  mutants/present_now_on_another_instance.replacement
fd8649db1aede81a64566f3f66c9708e6d75be755238e27d8822a58a11c5c34f  mutants/product_takes_another_instance.original
083c0bc3b4828f2fc89f1d06a797dfda53f19da13cbefa37326668332187874a  mutants/product_takes_another_instance.replacement
5ad1de5a618b0dcc10621580a216c1b006a9b777f980e2c142975ecd00f6e90c  tools/apply_one_replacement.py
097e50771168217f13f46c1b82458a40da8f7ad5d0417cc2b6cd29c48d3029c4  tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 证据的成色 |
|---|---|---|---|
| T1 实例只有一份 | 成立；外推那一句打中（打中 7） | 包内 `wgpu::Instance::new` 只一处，e202 每进程一份；「8 小于 12」只对一套驱动成立 | 读码、`grep`，量的 |
| T2 拒绝分不分得清 | 问的四臂都对；打中 3 处（打中 2、3、4） | 配错的等待上限 0 拒成「枚举没成」；没点名时同一个病态没有适配器清单；切换那一条注释只说对一半 | 读码；模型断言，推的待跑 |
| T3 把手的寿命 | 丢把手无害；打中 3 处（打中 1、5、6） | 枚举真挂住不拒、上限管不到；`self.clone()` 与现查那一份都没钉住；进程级实例看不到修好之后的驱动 | 读 wgpu 源码；模型断言，推的待跑 |
| T4 不开特性不碰 GPU | 成立 | `Backends::empty()` 在 wgpu-core 里进 hal 之前就返回；CPU 子命令不建 | 读源码；模型断言，推的待跑 |
| 检查单：走不到的理由 | 成立，附一条 | 三条没写明的「走不到」都是交回拒绝、不落盘；其中等待上限 0 那一臂产品路径到不了 | 读码 |
| 检查单：类型与名字 | 成立，附一条 | `created_for_this_process` 的名字与文档强过测试里的事实 | 读码 |
| 检查单：不兼容旧代码 | 成立 | `crates.patch` 加的行里没有认旧版本的写法 | `grep`，量的 |
| 检查单：不新建判定库 | 成立 | 加的行里没有新的判定存放处 | `grep`，量的 |
| 检查单：多机可用 | 成立 | 加的行不读环境、不写地址；新用例用服务交的卡 | `grep`，量的 |
| 检查单：测试优先多线程 | 不成立（字面） | 30 轮那条没写明为什么串行，只写了它测什么 | 读码 |
| 检查单：带 --item 的命令与项 | 成立，附一条 | 合表与 Verus 登记都不含这个包，但真卡目标归 product-gpu-real-card-tests 格 | `grep`，量的 |
| 检查单：新崩溃流 | 成立 | 补丁 9 个文件都在产品 GPU 包里，没有崩溃枚举用例 | `grep`，量的 |

打中按严重程度排：打中 1（枚举挂住）、打中 6（实例冻住驱动清单）、打中 5（没钉住）、打中 2（归错成员）、打中 3（少了清单）、打中 7（外推）、打中 4（注释）。

打中 3、打中 5、打中 6、打中 7 与这一批的改法有关。打中 1、2、4 的代码形状改前就有（读 `_m4-gpugc-one-vulkan-instance-code-r1-diff.md` 里减号那几行，等待与判定的次序没变）。

## T3 哪一条最脆弱（主攻）

### 打中 1：枚举真挂住时不拒，等待上限管不到

**时序**（每一步指到许可它的那一句）：

1. 调用方调开卡，走到 `vulkan_adapters_enumerated_within(wait_limit)`。
2. 那个函数的实参先求值：「self.instance.enumerate_adapters(wgpu::Backends::VULKAN),」（`crates/singlefs-background-gpu/src/background_gpu.rs:786`）。
3. wgpu 在造 future 这一步就做完枚举，不在 future 里：「let future = self.inner.enumerate_adapters(backends);」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/api/instance.rs:146`）。核心后端先枚举再包一个现成的结果：「Box::pin(ready(adapters))」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/backend/wgpu_core.rs:928`）。它下面调的是 Vulkan 的 `vkEnumeratePhysicalDevices`（`wgpu-hal-30.0.1/src/vulkan/instance.rs` 第 946 行）。
4. 驱动在这一步不返回，这条线程就挂在实参里。期限还没开始算：「let deadline = GpuWaitDeadline::starting_now(wait_limit, clock);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2610`）在 `wait_for_the_wgpu_future` 函数体里，实参做完才进得去。

**结局**：不拒，挂住。`AdapterEnumerationFailed` 不会出来。枚举慢一小时也一样：做完之后头一次轮询就 `Ready`，交回成功、不报超时。

**对不上的句子**：
- 「枚举适配器这一步就没成（等 wgpu 交回适配器列表超过了上限）」（`crates/singlefs-background-gpu/src/background_gpu.rs:179`）：上限不是 0 时这个成员基本走不到（见下一节打中 2 的唯一入口）。
- 「每一处等 wgpu 都在」（`crates/singlefs-background-gpu/src/background_gpu.rs:823`）：枚举这一处不在。
- 文件里自己写了前提：「本机的 wgpu 后端这几个 future 一轮询就好」（`crates/singlefs-background-gpu/src/background_gpu.rs:2598`）。这句话是真的，正因为真，上限才管不到。

**切换时更重**：核心层切换校验拿着实现的锁去答卡清单，「let implementation = gpu_compute.lock();」（`crates/singlefs-core/src/background_compute.rs:887`），答卡清单之前现查（`background_gpu.rs` 第 948 行起的 `identifiers_of_the_discrete_gpus_present_now`）。枚举挂住时这把锁一直拿着，后台批要这把锁的跟着停（推的，没量过）。

**驱动报错也不走这个成员**：`vkEnumeratePhysicalDevices` 返回错误时 hal 只打日志、交回空列表，「log::error!("enumerate_adapters: {err}");」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/vulkan/instance.rs:949`）。上面拒成的是「GPU 不在」或新成员，不是枚举没成。

**怎么证**：真挂一个驱动要假 ICD，这条腿没写。模型 `work_done_while_the_future_is_built_is_not_under_the_wait_limit` 用一段睡 300 毫秒的实参代表第 3 步：上限 1 毫秒，期望交回 `Ok(7)`、挂钟不少于 300 毫秒（模型断言，推的待跑）。wgpu 这一步是同步的，靠读上面两行源码，不靠这个模型。

**四句**：
- 分不分辨臂：不分辨。「每次开卡另建实例」与「进程一份」都这样挂。这是共用前提，另记一笔，不拿它判这一批。
- 系统看不看得到：看得到。开卡那一步已经用线程加到点放弃的写法（`outcome_of_opening_a_gpu_within_the_limit`，`background_gpu.rs` 第 864 行）。
- 满足的分句：D26（后台整理与放置回收）已定项 13 第 4 条写的是「挂载时开卡（含编管线）等的上限是 5 分钟」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`），没有逐字点名枚举这一步。所以它打中的是代码自己的文档（`background_gpu.rs` 第 179、823 行），不是 kb 那一句的字面。是不是该把枚举算进「开卡」，交主 agent 判。
- 改法还中不中：见本节末的表。

**什么现象会推翻它**：在一个枚举时挂住的驱动上，开卡在上限之内交回 `AdapterEnumerationFailed`。

### 打中 5：`self.clone()` 没钉住，现查本机独显那一份也没钉住

**现状**：第 98 批删掉了「只在第二台才红」的那行，产品实现这一行没有变异行（背景材料第二节，主 agent 的观测）。不要卡的替身走的是自己的 `vulkan_instance_for_this_opening`（`background_gpu.rs` 第 4602 行起的 `VulkanInstanceCountingTheOpenings`），够不到产品实现第 802 行的 `self.clone()`。

**任何机器都能跑的钉法**（只在我的模型上写过、没跑过、被攻过零轮）：`wgpu::Instance` 自带按 `Arc` 地址的相等：

- 「crate::cmp::impl_eq_ord_hash_proxy!(Instance => .inner);」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/api/instance.rs:38`）
- 「crate::cmp::impl_eq_ord_hash_arc_address!(ContextWgpuCore => .0);」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/backend/wgpu_core.rs:750`）

断言（模型 `attack_tests_appended_to_background_gpu.rs` 的 `the_product_opening_takes_the_very_instance_handed_over`，放进 `background_gpu.rs` 的 `mod tests` 就读得到私有字段）：

```rust
let handed_over = VulkanInstanceOfThisProcess::without_any_backend();
let taken = handed_over.vulkan_instance_for_this_opening();
assert!(taken.instance == handed_over.instance, "产品交给开卡的实例与交进来的不是同一个");
let created_again = VulkanInstanceOfThisProcess::without_any_backend();
assert!(created_again.instance != handed_over.instance, "阳性对照：另建的实例按 Arc 地址比不相等");
```

变异行（六段，制表符分隔，照 `crates/mutations.tsv` 文件头的格式；测试放进 `mod tests` 之后的写法）：

```
七·3 一个进程一个 Vulkan 实例：产品交给开卡的实例每次另建一个（按 Arc 地址比不等，哪台都红）	crates/singlefs-background-gpu/src/background_gpu.rs	    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {\n        self.clone()\n    }	    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {\n        Self::with_the_backends(wgpu::Backends::empty())\n    }	-p singlefs-background-gpu --lib -- background_gpu::tests::the_product_opening_takes_the_very_instance_handed_over --exact	the_product_opening_takes_the_very_instance_handed_over
```

替换文用不开后端的实例，变异时也不碰 Vulkan。被删那一行的替换文 `Self::created_for_this_process()` 放在这条断言下同样红（读 `cmp.rs` 推的，模型没跑这一种，免得不要卡的任务碰驱动）。

**这条钉不到的**：

| 位置 | 改坏什么 | 现在谁钉 | 判定 |
|---|---|---|---|
| `background_gpu.rs` 第 802 行 `self.clone()` | 产品每次交另一个实例 | 上面的新断言 | 推的，待主 agent 跑：`rerun.sh` 第二步，期望新断言退 101、现有库单测全绿 |
| `background_gpu.rs` 第 886–887 行 `AdaptersEnumeratedByTheInstance { vulkan_instance, … }` | 现查本机独显换一个实例 | 不要卡的一条都没有：要开卡成功才走到这里 | 推的，待主 agent 跑：`rerun.sh` 第三步，期望现有库单测全绿（变异活下来） |
| e202 `main` 的四处 `created_for_this_process()` | 每条子命令多建 | 没有 | 推的 |
| 调用方一个进程只建一次 | 下一个写调用方的人每次挂载新建一个 | 没有，只有 `background_gpu.rs` 第 756 行的文档 | 推的 |

第二行的现查实例在真卡上也只有第二台会红（同背景材料删掉那一行的理由）。要钉得把实例从 `BackgroundGpu` 露出来比相等，要卡；这条腿没写。实现员报告「停下交主 agent」第 4 条提过同一种露法。

**四句**：分辨臂（只打「每次另建」那一臂）；看得到（相等是公开的）；满足的是背景材料 T3 第二条的问法；改法见上。

**什么现象会推翻它**：`rerun.sh` 第二步新断言没红，或第三步现有库单测有红（那就是已经钉住了）。

### 打中 6：进程级实例把驱动清单冻在建它的那一刻（这一批新带来的）

**历史**（用户动作放开：修好之后调用方重挂几次、点不点名都一样，模型把两个方向都扫了）：

1. 长进程建一份实例，「调用方在进程里建一份、跨多次挂载留着」（`crates/singlefs-background-gpu/src/background_gpu.rs:742`）。
2. 建实例那一刻驱动不在或不对（加载器找不到 ICD）。wgpu-core 只打一行 debug 日志，「"Instance::new: failed to create {:?} backend: {:?}",」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/instance.rs:140`），`created_for_this_process` 的返回类型不是 `Result`，调用方拿到一个永远枚举为空的实例，看不出来。
3. 挂载拒：「改参数，或装上卡」（`crates/singlefs-background-gpu/src/background_gpu.rs:171`），核心层另一处写「修好它重挂」（`crates/singlefs-core/src/background_compute.rs:762`）。
4. 用户修好驱动，同一进程重挂，交回同一份实例。枚举的还是建实例时那份 ICD 清单（hal 枚举走的是建实例时拿到的 `self.shared.raw`，`wgpu-hal-30.0.1/src/vulkan/instance.rs` 第 946 行；加载器那一层是推的，模型来判）。
5. 结局：一直拒，点名是新成员、`adapters_in_the_enumeration` 为空，没点名是「GPU 不在」。出路只有重启进程，文档没写。反方向同样：建实例之后驱动坏了或卡掉了，同一个实例可能还报那几张，开卡才知道（推的）。

**分辨臂的对照**：改前每次开卡新建实例，第 4 步新建一个就看得到。所以这一格只打「进程一份」这一臂，不打改前的写法。

**模型**：`attack_long_lived_instance_on_a_real_gpu.rs`，写法同仓里驱动全藏起来那条用例（`VK_DRIVER_FILES`、`VK_ICD_FILENAMES` 指到不存在的清单），只建实例、只枚举，不开设备。两个方向各一次：

| 观测字段 | 打中的期望（推的，待主 agent 跑） |
|---|---|
| `hidden_at_creation` | 0 |
| `same_instance_after_the_drivers_came_back` | 0（打中）；不是 0 就撤回这一条 |
| `new_instance_after_they_came_back` | 不少于 1（要卡） |
| `same_visible_instance_after_hiding` | 等于上一行（打中） |
| `new_instance_after_hiding` | 0 |
| `lines_naming_libvulkan` | 不少于 1（阳性对照：判法认得出加载器） |

跑前写下的翻面条件：同一个实例在驱动清单变了之后枚举结果跟着变，就撤回这一条。这个模型换的是环境变量，不是真拔卡、真装驱动，它只说加载器不重读清单；真拔卡这一支没量。

**今天够不够得着**：够不着产品。核心层没有调开卡入口的地方（背景材料第二节），e202 一个进程一条命令、中途没有修驱动这一步。够得着的是这一批写下的接口约定：第一个「跨多次挂载」的长进程调用方就撞上。

**四句**：
- 分辨臂：分辨。
- 系统看不看得到：看不到。枚举结果在两种情形下都是空，实例不知道清单过期了；要分辨只能另建一个实例看，正好是这一批要省的事。
- 满足的分句：D26（后台整理与放置回收）已定项 13 第 4 条的「GPU 不在、驱动不对」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）要拒的是那一刻的事实。这里拒是拒了，事实已经过期。条款没写修好之后怎样，所以是推不出违反条款，算接口约定的缺口。
- 改法还中不中：见下表。

### 本节的改法（都只在我的模型上想过、没实现、被攻过零轮）

| 改法 | 修打中 1 | 修打中 6 | 代价 |
|---|---|---|---|
| 枚举放进线程、到点放弃（同第 864 行开卡的写法） | 修（推的，没实现） | 不修（推的） | 线程可能永远不回来，同一实例上下一次枚举也可能挂 |
| 文档写明实例冻住驱动清单、修好驱动要换进程 | 不修（推的） | 只写明，行为不变（推的） | 无代码 |
| 枚举为空时丢掉实例另建一次，一个进程最多几次 | 不修（推的） | 修第 4 步（推的） | 另建次数要有上限，碰「界约 12」；用户定的单例之外多一条分支 |
| 什么都不改，这一批只记账 | 不修（推的） | 不修（推的） | 打中 6 等第一个长进程调用方出现时再判 |

### 把手的寿命（问的四件，没打中）

| 问 | 答 | 依据 |
|---|---|---|
| `AdaptersEnumeratedByTheInstance` 持的克隆撑到哪 | 撑到 `BackgroundGpuOverTheCards` 丢掉，字段排在卡后面 | 「Box::new(AdaptersEnumeratedByTheInstance {」（`crates/singlefs-background-gpu/src/background_gpu.rs:886`）、第 398 行字段 |
| 泄漏的卡撑到哪 | 不析构，卡持的 Vulkan 实例引用一直在，VkInstance 不销毁 | hal 设备持 `Arc<InstanceShared>`（`wgpu-hal-30.0.1/src/vulkan/mod.rs` 第 304、510 行），`gpus_of_this_host.rs` 第 921 行 `mem::forget`；推的，读源码 |
| 调用方丢掉实例之后已开的卡还能用吗 | 能，枚举也能：克隆指同一个 `Arc<Global>` | 「Does not have to be kept alive.」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/api/instance.rs:28`）；`CoreDevice` 与 `CoreAdapter` 各持一份 `context`（`wgpu-30.0.1/src/backend/wgpu_core.rs` 第 461、467 行）；推的，读源码，真卡没量 |
| 丢掉再新建会回到老问题吗 | 会，类型上挡不住：`created_for_this_process` 是 `pub`，一个进程调几次都行，只有文档写「调用方一个进程只建这一次」（`crates/singlefs-background-gpu/src/background_gpu.rs:756`） | 用户定不用单例，这是定下的代价，不算打中 |

### 枚举超时、枚举为空各拒成什么

| 情形 | 点名 | 没点名 | 依据 |
|---|---|---|---|
| 驱动挂住、枚举不返回 | 挂住，不拒 | 挂住，不拒 | 打中 1 |
| 上限给 0 或小到枚举之后头一次看就到期 | `AdapterEnumerationFailed` | `AdapterEnumerationFailed` | 打中 2 |
| 枚举为空（没有 ICD、不开后端、hal 报错吞掉） | 新成员，`adapters_in_the_enumeration` 为空 | `SettingsRefused(GpuNotPresent)` | 模型 `an_empty_enumeration_is_refused_as_the_member_of_the_selection`，推的待跑；不挂住 |
| 给了空列表、枚举为空 | `SettingsRefused(EmptyGpuDeviceListWhoseMeaningIsUndecided)` | 不适用 | 同上 |

切换那一刻另算，见打中 4。

## T2 内部矛盾：拒绝分不分得清（主攻）

### 纯函数每一臂

下表的成员是读 `discrete_gpus_and_participating_gpus_admitted_from_the_enumeration`（`background_gpu.rs` 第 899–939 行）、`discrete_gpus_of_the_enumeration_under_the_named`（`adapter_inventory.rs` 第 126 行起）与核心层 `admit_background_compute_settings`（`background_compute.rs` 第 979 行起）读出来的；模型 `every_arm_of_the_pure_function_gives_the_member_read_from_the_code` 把每一臂写成断言（推的，待主 agent 跑，`PURE_ARM` 行）。

| 模型里的臂 | 点名 | 枚举到 | 成员 |
|---|---|---|---|
| `named_absent_only_software` | A | 软件 Vulkan | 新成员 |
| `named_discrete_without_address` | A | 没地址的独显、软件 Vulkan | 新成员，`unnamed_left_off_the_list` 带它 |
| `not_named_only_software` | 不给 | 软件 Vulkan | `SettingsRefused(GpuNotPresent)` |
| `not_named_discrete_without_address` | 不给 | 没地址的独显 | `DiscreteGpuListNotSupported` |
| `named_card_without_a_heap` | A | A，读不到堆 | `DiscreteGpuListNotSupported` |
| `named_card_reported_twice` | A | A 报两次 | `DiscreteGpuListNotSupported` |
| `empty_named_list` | 空列表 | 软件 Vulkan | `SettingsRefused(Empty…)` |
| `named_twice` | A、A | A | `SettingsRefused(GpuDeviceNamedMoreThanOnce…)` |
| `named_integrated` | A | A 是集显 | 新成员（D26（后台整理与放置回收）已定项 13 第 2 条要的） |
| `limit_count_different` | A | A，两个上限 | `SettingsRefused(MemoryLimitCount…)` |
| `limit_above_the_total` | A | A，上限超总量 | `SettingsRefused(MemoryLimitAbove…)` |
| `wait_limit_zero_through_the_pure_function_only` | A | A，上限 0 | `SettingsRefused(GpuWaitLimitOfZero)`，只有直接调纯函数才到（打中 2） |
| `named_the_empty_address` | 空串 | 没地址的独显 | 新成员，没地址的卡点不到 |
| `not_named_other_card_without_a_heap` | 不给 | A，B 读不到堆 | `DiscreteGpuListNotSupported` |
| `named_absent_before_the_limit_count` | B、A | A，一个上限 | 新成员，先于上限个数判 |
| `named_present_other_discrete_without_address` | A | A、没地址的独显 | 不拒 |

问的四臂（点名的卡不在清单、独显没报总线地址、没有独显又没点名、别的核心层拒绝）都与文档一致，**这四臂没打中**。

### 与第 4 条「GPU 不在」是不是同一件事

是两格，按「点没点名」分，不是把一件事报成两样。点名时一张独显都没有，走新成员不走「GPU 不在」，因为核心层先判点名：「BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {」（`crates/singlefs-core/src/background_compute.rs:1070`），「GPU 不在」只在「没点名卡而清单里一张卡都没有」时出（同一份文件第 740 行的成员文档）。这与第 2 条「点名它们照」点名的卡不存在拒的写法一致。新成员没有吃掉「GPU 不在」，也没有反过来。

调用方有没有按旧成员 `match`：`grep` `BackgroundGpuOpenRefused::`，三份产品文件之外只有 `tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs` 第 363 行，判的是没点名的「GPU 不在」，对。e202 只用 `{refused:?}` 打字符串（`background_gpu_compute` 也只原样往上交），不分成员。挂载与切换在核心层，不认这个枚举。**没有归错的调用方**（量的，范围是 `crates/singlefs-background-gpu/` 的 src 与 tests）。

`# Errors` 与成员：`open_background_gpu`、`…_with_the_memory_leaked_in_this_process`、`open_background_gpu_compute` 都写五种，枚举正好五个成员，逐个对得上。

### 打中 2：等待上限给 0，拒成「枚举适配器没成」

**输入**：`background_gpu_wait_limit` 给 0。这是能写出来的挂载参数，`BackgroundGpuSelection::of_the_settings` 原样带过来。

**每一步**：
1. 开卡先枚举：「.map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;」（`crates/singlefs-background-gpu/src/background_gpu.rs:845`）。
2. 上限 0 时头一次看就算到期，轮询之前就交回超时：「.filter(|time_left| !time_left.is_zero())」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:74`），`wait_for_completion_before_the_deadline` 先判它。
3. 纯函数根本没跑到，`BackgroundComputeSettingsRefused::GpuWaitLimitOfZero`（`background_gpu.rs` 第 932 行那一臂）从产品路径到不了。

**对不上的句子**：枚举的文档「成员按调用方要改哪一样分」（`crates/singlefs-background-gpu/src/background_gpu.rs:168`）。这是配错，该改参数；拿到的成员让人去查驱动。空列表加上限 0 时，`EmptyGpuDeviceList…` 也被盖住。

**模型**：`a_wait_limit_of_zero_through_the_opening_is_refused_as_the_enumeration_failing`，点名、不点名、空列表三种都期望 `AdapterEnumerationFailed { WaitForTheGpuExceededTheLimit { wait_limit: 0 } }`（推的，待主 agent 跑，`OPEN_ARM` 行）。

**四句**：
- 分辨臂：不分辨（改前次序相同）。这一批把这一臂写进了 `match`，读起来像走得到。
- 看得到：看得到，判一下 `wait_limit.is_zero()` 就行。
- 满足的分句：D26 已定项 13 第 4 条只要求「拒」，拒了。违反的是代码自己「按要改哪一样分」那一句，不是 kb。没有落盘动作，开卡不碰盘。
- 改法：开卡入口头一步先判上限 0、拒成 `SettingsRefused(GpuWaitLimitOfZero)`（只在我的模型上想过、没实现、被攻过零轮；表里标「推的」）。修这一格，不碰打中 1。

**什么现象会推翻它**：模型那条用例红、开卡交回 `GpuWaitLimitOfZero`。

### 打中 3：没点名时，同一个病态拒绝里没有适配器清单

**输入**：第二台上那个病态（枚举只剩软件 Vulkan、三张独显一张都不报，`crates/singlefs-background-gpu/src/background_gpu.rs:748` 那一句写的现象），加上没给 `background_gpu_devices`。「不给就用本机全部独立显卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）是这个参数的默认，e202 的 `ComputeOfTheArm::GpuOnEveryCard` 也是这一路（e202 第 2813–2818 行）。

**结局**：`not_named_only_software` 那一臂，`SettingsRefused(GpuNotPresent)`，不带枚举到了什么。真没装卡与驱动只剩软件 Vulkan，拒绝逐字相同。这一批加清单的理由正是「分得清」（`background_gpu.rs` 第 175 行「分得清是这一次枚举里没有这张卡」），这个分法只在点名那一路有。

**四句**：
- 分辨臂：不分辨，这一格是这一批改法的射程问题。
- 看得到：看得到，纯函数手里就有 `descriptions`。
- 满足的分句：D26 已定项 13 第 4 条只要求拒并报错，**不违反 kb**。打中的是题面「分得清」的外推：背景材料 T2 问的是「点名的卡不在清单」，这里问的是同一个现象在默认路径上。
- 改法：让「没点名、清单空」也带清单，要动 `no_discrete_gpu_without_named_cards_is_refused_as_gpu_not_present_through_the_settings`、compaction 目标第 363 行那条用例与`crates/mutations.tsv` 第 6351 行的变异（推的，没实现）。不改也说得通，交主 agent 判要不要进这一批。

### 打中 4：切换那一条注释只说对一半，切换也没有清单

「枚举不成（等 wgpu 超过上限）：一张都不算在，切换照「GPU 不在」拒」这一句（`crates/singlefs-background-gpu/src/background_gpu.rs` 第 953 行，`adapter_inventory.rs` 第 225 行同义）。

输入：切换点了名、这一次现查枚举为空。交回空列表之后走核心层，点名先判：同一份文件的单测写明了结局，「"卡掉了、点名它：点名的卡不存在"」（`crates/singlefs-background-gpu/src/background_gpu.rs:3651`）。所以点了名的切换拒成 `NamedGpuDeviceNotInTheInventory`，不是「GPU 不在」。这一句对没点名才对。代码没错，注释窄了。

同一条路上：核心层挂载再校验一次（`admit_gpu_settings_against_the_inventory`，第二次现查枚举）与切换拒成的都是核心层成员，不带清单。新成员的「分得清」只在开卡入口那一次枚举。病态如果在开卡之后才出现（这一批之后一个进程一个实例，按第二台的量不该再出现），切换看不到清单。不违反 kb，算射程。

四句：不分辨臂（注释改前就有）；看得到；打中的是注释本身，不是判据；改法是改这两行注释（推的，无代码风险）。

### 不归这一批、但问到了的一句

第 4 条字面写「到点当那张卡没开成」之后点名的卡照「点名的卡不存在」拒（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）。代码是 `GpuUnusable { device, cause }`（`gpus_of_this_host.rs` 第 856 行，第 801 行注释点到调度记录第 45 批的定法）。这一批没碰这里。现在「点名的卡不存在」的字面在代码里有两个去处：不在枚举清单是新成员，到点没开成是 `GpuUnusable`。信息没丢，字面与 kb 不一样，交主 agent 判要不要回写 kb。

## T1 实例真的只有一份吗

**包内**：`grep -rn 'Instance::new\|wgpu::Instance\|enumerate_adapters' crates/singlefs-background-gpu/` 只命中 `background_gpu.rs` 第 752、770、772、786 行（量的，772 是 `InstanceDescriptor` 那一行）。**全部 crates**：`wgpu::Instance::new` 另外只有 `crates/singlefs-verification/src/gpu_unit_checks.rs` 第 402 行，验证包不依赖产品 GPU 包（`grep singlefs-background-gpu crates/*/Cargo.toml` 只命中它自己），两者不同进程。

**克隆**：`#[derive(Debug, Clone)] pub struct Instance`（`wgpu-30.0.1/src/api/instance.rs` 第 31–32 行），内层 `Core(Arc<…>)`（`wgpu-30.0.1/src/dispatch.rs` 第 727 行起的宏）、`ContextWgpuCore(Arc<Global>)`（`wgpu_core.rs` 第 44 行）。克隆指同一个实例，**成立**（读源码）。

**每条路径的实例数**（量的：`grep` 与读码；这一格没有东西要跑）：

| 路径 | 实例从哪来 | 一个进程几个 |
|---|---|---|
| e202 `lockstep`、`concurrent`、`stage-three-cost` | `main` 第 8456、8460、8477 行各建一个，`match` 只走一臂 | 1 |
| e202 `sensitivity` 交替跑加并发跑 | `main` 第 8468 行，同一个引用传两次（`sensitivity_command`） | 1 |
| e202 `geometry-check`、`construction-timing` | `GpusOfTheCommand::NotNeeded`（第 7328、7464 行），`vulkan_instance: None`（第 7294 行），只跑 `LOCKSTEP_ARMS[0]`（CPU） | 0 |
| e202 `judge-trend` | 不开卡 | 0 |
| e202 真卡内联单测 | 第 11307 行，特性门在第 11303 行 | 1 |
| 同一进程里失败重开（`run_lockstep_once` 每次重复开卡，第 5462 行） | 同一份 `opening.vulkan_instance` | 1 |
| 挂着换设置 | 不开卡、不建实例 | 0 个新增 |
| 真卡测试目标 | `tests/common_discrete_gpus_of_this_host/mod.rs` 第 520 行，一条用例一次 | 见下 |
| 驱动全藏起来那一次 | 子进程里第 349 行一次；外面那一次走 `if` 之前不建 | 子进程 1 |

真卡目标每个进程的实例数（量的：逐用例数 `…_or_report_not_run(`）：checksum 8，live_slot 6，region_record 6，compaction 外层 6（7 条用例里驱动全藏那条不建）、子进程 1，两卡目标 2，三卡目标 1。每条用例至多一次，没有用例建两次。

### 打中 7：「界大约是 12、用例数小于它」只对一套驱动成立

- 那个界是第二台这一套驱动上量的（背景材料第二节）。调查报告 `result.md` 里 `grep 'TLS\|ICD\|dlopen\|根因\|VK_DRIVER'` 0 命中（量的）：产品这边没有量过它是什么的函数。
- 验证包写了同一个现象的机制：「几十次之后」（`crates/singlefs-verification/src/gpu_unit_checks.rs:392`），说每个实例把本机每个 ICD dlopen 一遍、吃静态 TLS 余量。那边的界是「几十次」，这边是 12。两个数对不上，这本身说明界不是常数，至少随本机装的 ICD 变（推的；验证包的机制这条腿没复核）。
- 验证包另有 `load_only_the_nvidia_vulkan_driver_when_the_user_named_none`（`gpu_unit_checks.rs` 第 401、418 行）压低每次的消耗。产品 GPU 包没有这一步，每条用例的实例把全部 ICD 再装一遍。

外推到的地方：「8 小于 12」外推到别的驱动版本、别的 ICD 组合、用例数增长。**今天不破**，checksum 目标 8 条，余量 4。没有东西在用例数超过时变红。实现员报告「停下交主 agent」第 2 条已经写了这个风险，这一条补的是界本身不是常数。

**四句**：分辨臂（只打测试进程，产品一份不受影响）；看得到；打中背景材料「这里只做到这一步的依据是用户定的不用单例，不是证明了界不会变」那一句已自认的部分，新增的是另一处仓内文字给出了不同的数；改法：① 真卡目标的 `#[test]` 数做一条不要卡的源码核对（不是运行时全局计数，不碰用户定的「不用 static、不用全局计数」）；② 测试建实例之前照验证包只装 NVIDIA 那一份。两条都只在我这里想过、被攻过零轮，表里算「推的」。

## T4 不开特性不碰 GPU：没打中

| 问 | 答 | 依据 |
|---|---|---|
| `Backends::empty()` 装不装加载器 | 不装。hal 的 `init` 在这一判之后，「if !instance_desc.backends.contains(A::VARIANT.into()) {」（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/instance.rs:108`）；加载器在 hal `init_with_callback` 里才 `Entry::load`（`wgpu-hal-30.0.1/src/vulkan/instance.rs` 第 583 行） | 读源码；模型 `an_instance_without_any_backend_maps_no_vulkan_loader` 读 `/proc/self/maps` 期望 0 行，阳性对照在真卡模型 `lines_naming_libvulkan`（推的，待跑） |
| `Instance::new` 会不会因没后端 panic | 不会：判的是编进来的后端特性（`enabled_backend_features`），不是这一次请求的 | `wgpu-30.0.1/src/api/instance.rs` 第 64 行 |
| 不要卡的单测建不建真实例 | 不建，替身交回 `without_any_backend` | `background_gpu.rs` 第 4632 行 |
| e202 CPU 臂 | `geometry-check` 不建。`lockstep` 只跑 CPU 臂（没卡）时 `main` 照建一个真实例：改前也枚举，第 5625 行那条路一样碰驱动。e202 文件头声明 `task-server-gpu-arguments: lockstep concurrent sensitivity stage-three-cost`，这四条本来就是要卡的命令 | 读码 |
| `geometry-check`、`construction-timing` | 不建 | 第 7328、7464 行 |
| `#[cfg(test)]` 漏没漏 | 没漏。`without_any_backend` 是私有方法，第 763–764 行，唯一调用在 `background_gpu.rs` 的 `mod tests`（第 4632 行） | `grep`，量的 |
| 不开特性的目标 | `checksum_batches_over_{two,three}…` 用 `#![cfg(feature = …)]`（第 10 行），另四个真卡目标 `required-features`，`real_gpu_tests_open_only…` 18 条不调 `handed_over_gpus_or_report_not_run` | `grep`，量的 |

**变异时会碰驱动一次**：`crates/mutations.tsv` 第 7016 行那条变异的替换文是 `created_for_this_process()`，不要卡的 `--lib` 任务里建一个真实例并枚举（实现员报告自己写了「只在变异时这样」）。不开设备，不算打中。换成 `with_the_backends(wgpu::Backends::empty())` 同样红：枚举仍是空，头一条断言不红，靠替身记下 0 次那一条红（推的）。交主 agent 看要不要换。

## 检查单八格

### 走不到的理由二次查实：成立，附一条

查法：`grep -n '^+.*\(unreachable!\|panic!\|\.expect(\|_ =>\|走不到\|不会到\|到不了\|不可能\)' crates.patch`，8 处：产品代码文件里只有 e202 单测那 2 处（与主 agent 的数一致）；另 6 处在测试文件（量的）。没有一处写在非测试代码里。

这一批**没写成注释也没写成 `expect`** 的「走不到」，在纯函数的 `match` 里有三臂（`background_gpu.rs` 第 932–936 行）：

| 臂 | 产品路径走得到吗 | 走到了做什么 | 落盘之前？ |
|---|---|---|---|
| `GpuWaitLimitOfZero` | 走不到，先被枚举那一步拒（打中 2） | 交回拒绝 | 是，开卡不碰盘 |
| `ParticipatingGpuDeviceNotOpenedAtTheMount` | 走不到：`DiscreteGpuInventorySnapshot` 不改 `devices_not_opened_at_the_mount`，用的是 `background_compute.rs` 第 88 行的默认空表 | 交回拒绝 | 是 |
| `GcCpuCompensationThresholdAboveTheWholeCapacity` | 走不到：`admit_background_compute_settings` 不判阈值（阈值在 `background_compute.rs` 第 874 行那一层） | 交回拒绝 | 是 |

三臂都是 `clippy::wildcard_enum_match_arm` 逼着写全的，走到了也只是拒，不 panic。判「无害」。e202 的 `vulkan_instance_for_the_gpu_arm` 交回 `Err` 也不是 panic，走到要 GPU 臂配上 `None`：`geometry-check` 与内联单测只跑 `LOCKSTEP_ARMS[0]`，那是 CPU 臂（e202 第 2682–2685 行）。

### 类型与名字：成立，附一条

逐个读了背景材料列的 13 个名字，都说得出内容，没有序数、缩写。`grep -nE` 查同一文件里没有同名定义（`background_gpu.rs` 里 `NamedGpuDeviceNotOnTheListOfThisEnumeration` 是枚举成员与结构体同名，分属两个命名空间，读得通）。

名字强过事实的一处：`VulkanInstanceOfThisProcess` 与 `created_for_this_process`，文档写「调用方一个进程只建这一次」（`crates/singlefs-background-gpu/src/background_gpu.rs:756`）。真卡测试进程一条用例建一个（`tests/common_discrete_gpus_of_this_host/mod.rs` 文件头自己写「一条用例只建一个 Vulkan 实例」）。这是用户定的不用单例的结果，不要求改，文档可以补一句测试进程的例外。

两处新 `expect` 依赖的不变量：`.expect("CPU 臂挂载得成")` 依赖 CPU 臂不开卡（`open_the_gpu_of_the_arm` 与 `stage_three_share_of_one_greedy_call` 里 `uses_the_gpu` 为假就不碰实例），`.expect("枚举得到适配器")` 在特性门之内，上限是 `LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT`，只有枚举超时才错，按打中 1 实际挂住不报错。都在单测里。

### 不兼容旧代码与旧格式：成立

`grep -n '^+.*\(legacy\|旧版\|兼容\|迁移\|旧布局\|旧格式\|此前\|原先\|改前\|后来\)' crates.patch` 0 命中（量的，补丁加的行 778 行，`grep -c '^+'`，含 `+++` 文件头）。这一批不碰盘上格式。删掉 `GpusOfTheCommand::Enumerated` 时没留别名。

### 不新建判定库：成立

`grep -n '^+.*\(sqlite\|rocksdb\|RocksDB\|prune\|\.tsv\|判定库\)' crates.patch` 0 命中（量的）。e202 的产物照旧写 `ProductWriter`，没有新的存放处。

### 所有任务多机可用：成立

`grep -n '^+.*\(env::var\|multi-host\|hostname\|192\.168\|/home/\|/tmp/\)' crates.patch` 0 命中（量的）。新真卡用例走 `handed_over_gpus_or_report_not_run`，那一层读 `GPU_CARDS_BY_PRIORITY` 与多机配置判法，是已有的写法，没有新加本机状态。产品文件里的「第二台」是注释文字，不是判定。两处单测里的 `0000:ff:1f.7`、`0000:01:00.0` 是假地址，不碰真卡。

### 测试优先多线程：不成立（字面）

条款：「不能并行的写明为什么」（`.claude/rules/implementation-workflow.md:79`）。30 轮那条（`tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs` 第 543–545 行注释、第 40 行常量注释）写了测什么、为什么是 30 轮，**没写为什么一轮接一轮串着跑**。目标文件头第 6 行只写带 `--test-threads 1`。

理由其实明摆着：要的就是同一个进程里同一张卡先丢再开，并行开卡测的是另一件事。补一句就够。不要卡的单测 3 轮同样没写，背景材料已自认「不值得切片（推的，没量过）」，也没写进代码。

这一格不打代码对错。什么现象会推翻它：`implementation-workflow.md` 那一条不罩串行循环的用例，由主 agent 判。

### 带 --item 的命令与项：成立，附一条

实现员那句话拆开核：

| 那句话的一段 | 判定 | 证据（量的） |
|---|---|---|
| 不在崩溃点重放合表 | 成立 | `grep -c 'background' .claude/gate.d/crash-point-replay-items.tsv` 打 0 |
| 不在 Verus 登记 | 成立 | `grep -c 'background-gpu\|background_gpu' .claude/gate.d/verus-pins.tsv` 打 0 |
| 没有新的崩溃枚举用例 | 成立 | `grep -c 'crash-case:\|crash_enumeration' crates.patch` 打 0 |
| 第 1d 步三条都不用写 | 只对前两条 | 第 1d 步第三条是 `crates-mutation-replay.sh --item`，「点这一次改到的项」（`.claude/agents/implementation-writer.md:34`）；这一批改了 5 行变异，实现员拿 `prove-red.sh` 证红代替。证红查的是红，`--item` 查的是这一趟的键。主 agent 第 10 条写的也是这一项（`.claude/main-agent.md` 第 33 行），背景材料没写这一项跑没跑 |

附一条：这个包没有合表项，但新用例不是没有门禁：`.claude/gate.d/stage-inputs.tsv` 第 30 行 `tests-of:singlefs-background-gpu`，`.claude/gate.d/crash-point-replay.sh` 第 11 行 product-gpu-real-card-tests 格带 `tests-on-a-real-gpu` 跑真卡目标。30 轮那条提交时归这一格。

新变异行能不能被列出：`crates-mutation-replay.sh --list-items`「逐行打变异表里可点名的变异名」（`.claude/gate.d/crates-mutation-replay.sh` 第 16 行），`crates/mutations.tsv` 第 7016–7019 行都在，第 6351 行是换过锚点的那一行，都是完整六段（`grep -n` 量的）。成立（读登记，没跑）。

### 新崩溃流多罩了什么：成立

补丁 9 个文件都在 `crates/singlefs-background-gpu/` 下（`grep -n '^diff --git' crates.patch`，量的），`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行 105 行（`grep -c`，量的），上面一格已核这一批没有崩溃枚举用例。

附带看到一件不归这一批的：`crates/singlefs-verification/tests/.background_compute_settings_and_switch.rs.lhuq544s.replacing` 进了 git（`git ls-files` 量的），看名字是一次替换留下的临时文件。写进这里交主 agent，这条腿不碰。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 同一进程实例多于一份的产品路径 | 全包 `grep` 全部构造点，e202 七条子命令逐条，失败重开、换设置、并发跑 | 没有。一个进程一份 |
| 克隆其实另建 | wgpu 30.0.1 的 `Instance`、`DispatchInstance`、`ContextWgpuCore` | 没有，都是 `Arc` |
| 丢掉把手之后 use-after-free 或枚举失败 | hal 的 `Arc<InstanceShared>`、`CoreDevice.context`、字段次序 | 没有（推的，读源码，真卡没量） |
| 泄漏的卡与同一实例上新开卡互相干扰 | 只读了 wgpu 层：设备互相独立、注册表锁不跨 hal 调用 | wgpu 层没有；驱动内部锁没法读，开卡挂住之后同一 VkInstance 上枚举会不会跟着挂，没量 |
| 新成员吃掉「GPU 不在」或反过来 | 16 臂，见 T2 表 | 没有 |
| 调用方按旧成员 `match` 归错 | 包内全部 `BackgroundGpuOpenRefused::` | 没有 |
| 点名的卡同时报两次、一个没地址 | `adapter_inventory.rs` 第 126–207 行逐分支读 | 有地址的那份收、没地址的进清单外，对 |
| `Backends::empty()` 碰加载器 | wgpu-core 判 `backends.contains`、hal `init` | 没有 |
| `#[cfg(test)]` 漏进非测试 | `grep` | 没有 |
| 不开特性的目标建真实例 | 全部 8 个测试文件，建实例只有 2 处都在特性门内 | 没有 |
| e202 CPU 臂挂钟被常驻实例污染 | 只读了计时用 `CLOCK_THREAD_CPUTIME_ID`（`host_cpu_time_of_this_thread`），驱动后台线程不进这个钟 | 读出来不污染；没量。产物字段还有没有别的钟，没逐个查 |

这一条腿的「没打中」只抽了一次样，照 `.claude/rules/three-way-inference.md` 不算「没打中」的结论，算线索。

## 这条腿自己的限度

- 模型一次没编过。`attack_tests_appended_to_background_gpu.rs` 与真卡模型可能编不过，编不过是模型的错，不算打中或打不中。
- 判定一览里「模型断言」全是推的，主 agent 跑完才算数。
- 打中 1 没有挂住的驱动做实验：用的是读 wgpu 源码加一个 300 毫秒实参的替身。真挂要写假 ICD，这条腿没写。
- 打中 6 换的是环境变量，不是真拔卡。真卡热拔、驱动重载那一支没有量，别的驱动上结论可能不同。
- 验证包写的静态 TLS 机制（打中 7）是别处文字，这条腿没复核，产品这边的界是不是同一个机制不知道。
- 改法都是这条腿提的，被攻过零轮。

## 要主 agent 跑的命令

判决要引的数照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」那一段第 3 步，把模型目录拷到草稿目录再交；下面写的是腿自己的路径。第 1 条跑完第 2 条才有工作树。

1. 模型整份复跑（不要卡）。

   ```
   task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-one-vulkan-instance-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-opus/rerun1
   ```

   - 在哪个目录起：仓根（脚本里的路径全是绝对路径，`rerun.sh` 里每条 `cargo test` 自带范围声明）。
   - 跑完看：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=0`。`task-server/task log <号> --all` 里：
     - `name=attack_tests_on_the_frozen_code expected_rc=0 rc=0`，`PURE_ARM` 16 行、`OPEN_ARM` 7 行、`FUTURE_BUILT_BEFORE_THE_DEADLINE outcome=Ok(7)`、`LOADER_MAPPED lines_naming_libvulkan=0`。
     - `name=product_takes_another_instance_under_the_new_test expected_rc=101 rc=101`（新断言钉得住）。
     - `name=product_takes_another_instance_under_the_existing_library_tests expected_rc=0 rc=0`（现有库单测钉不住 `self.clone()`）。
     - `name=present_now_on_another_instance_under_the_existing_library_tests expected_rc=0 rc=0`（现查那一份没钉住）。
     - 末行 `name=rerun_summary mismatches=0`。
   - 怎么读：`mismatches` 不是 0 时看是哪一行的 `rc` 对不上。编不过是模型的错，交同立场的腿改；后两行 `rc` 不是 0 表示那个变异已经被钉住，打中 5 相应那一行撤回；`attack_tests` 里 `a_wait_limit_of_zero…` 红表示打中 2 撤回。

2. 进程级实例跟不跟得上驱动清单（要卡：源头第一行声明 1 张，只建实例、只枚举，不开设备；两台都能跑）。

   ```
   SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的测试目标 attack_long_lived_instance_on_a_real_gpu; 范围=1 个测试目标; 为什么=量同一个 Vulkan 实例在驱动清单变了之后枚举结果变不变，只在攻方模型的副本里' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-opus/rerun1/tree -- cargo test --release --offline -p singlefs-background-gpu --test attack_long_lived_instance_on_a_real_gpu -- --nocapture --test-threads 1
   ```

   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-opus/rerun1/tree`（第 1 条建的）。
   - 跑完看：`VIEW …` 那一行六个字段，派到哪台，`test result:`。
   - 怎么读：`exit=0` 且两个「打中」断言都过，打中 6 站住（只说明加载器不重读清单）。panic 文字以「打中：」起头的，打中 6 撤回。`new_instance_after_they_came_back` 是 0 是这一台看不到独显，不算结论。

`rerun.sh` 只改了模板的 `rerun_steps`，模板自证没动，这一份不交 `--selftest`。

没写进命令、由主 agent 定的：第 1d 步第三条 `crates-mutation-replay --item`（见检查单那一格，背景材料不让把 `.claude/gate.d/` 下的脚本写进命令）。

## 没做什么

- 没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）；模型没编过、没跑过。
- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没写假 ICD，没真挂驱动，没真拔卡。
- 没读两个 `tests/` 共用模块以外、与开卡无关的测试正文；核心层只读了 `background_compute.rs` 校验那一段。
- 打中 7 的静态 TLS 机制没复核，只引了验证包的文字。
- 没有崩溃点要枚举，这一批不碰盘，定义第 3b 步的崩溃枚举、稀疏盘都不适用。
- 建的编译目录与仓副本：没有。草稿目录里只有 `d26-13.txt`（kb 抄的一节，便于读）与 `rerun_steps2.txt`（`rerun.sh` 的 `rerun_steps` 原文）。
