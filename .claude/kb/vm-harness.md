# 虚机 harness：怎么在真块设备上跑一个实验

**为什么单独一篇**：块设备行为是好几个实验的被测对象，而「怎么把实验送进虚机」
每次都被重新推导一遍，两次卡在同一处（内核镜像不可读、二进制不是静态的）。
本文只写现状，不写历史。

## 一句话

```bash
cd research
cargo build --release --target x86_64-unknown-linux-musl --bin <实验二进制名>
bash scripts/vm-bench.sh target/x86_64-unknown-linux-musl/release/<名> /dev/vda <参数...>
```

## 挂几块盘

`VM_DISKS`（默认 1）。>1 时来宾看到 `/dev/vda` `/dev/vdb` …，
**全部设备路径按顺序作为前缀参数传给二进制**，二进制自己的参数跟在后面。

```bash
VM_DISKS=4 VM_DISK_MB=64 bash scripts/vm-bench.sh <静态二进制>
```

多盘是 E53（丢一整块盘之后根环还挂不挂得上） 要的：「丢一整块盘之后还挂不挂得上」
只有在真的有多块盘时才问得出来。改完 harness **先跑 `--selftest`**，通过了才算数。

⚠️ **最小 initramfs 里没有 `dm-error` / `mdadm`**：
做不出「读返回 EIO」和「条带阵列」，只做得出「把某块盘的内容真的抹掉」。

## 三个前置，每个都有确定的处置

| 前置 | 怎么查 | 不满足怎么办 |
|---|---|---|
| **KVM 可读写** | `id -nG \| grep -w kvm`，再实测 `[ -r /dev/kvm ] && [ -w /dev/kvm ]` | 本机靠 **`kvm` 组成员身份**拿权限。不满足就 `sudo usermod -aG kvm $USER`（口令在 `.env` 的 `SUDO_PASS_A`）；当前 shell 用 `sg kvm -c '<命令>'` 立即取得，新会话自动带上 |
| **内核镜像可读** | `bash scripts/vm-kernel.sh --check` | 跑 `bash scripts/vm-kernel.sh`，它打印一个可用路径；必要时用 `.env` 里的口令从 `/boot` 复制一份到 `TMPDIR` 并 chown。要量时间就跑 `bash scripts/vm-kernel.sh --release $(uname -r)`：它只认 `/boot/vmlinuz-<版本>`，复制成 `TMPDIR` 下的 `singlefs-vmlinuz-<版本>`，不会先撞上 lockdep 那个 |
| **二进制是静态的** | `file <二进制>` | 用 `--target x86_64-unknown-linux-musl` 编。musl 目标已装（`rustup target list --installed`） |

⚠️ **判断 KVM 前置一律查组并实测读写，不许查 ACL，也不许用 `setfacl` 去补。**
`/dev/kvm` 带 `TAG+="uaccess"`（`/usr/lib/udev/rules.d/70-uaccess.rules:48`），
它的 ACL 由 logind 按**本地 seat 会话**发放；SSH 会话没有 seat ⇒ 拿不到，
而手工 `setfacl` 上去的那条会在下一次会话变化时被清掉——**设上去时 `getfacl` 看着是对的**。
**实测到的失败形态是最坏的那种**（2026-09-01）：`qemu.sh --selftest` 要跑两次虚机，
第一次拿得到 KVM、第二次拿不到，于是输出成「成功用例 → 0、失败用例 → 读不到退出标记」，
**看起来像 harness 分辨不出失败，实际是权限在两次启动之间没了**。
⚠️ **本机 `/boot` 下只有一个可读内核：`vmlinuz-6.17.0-lockdep`**（其余是 `-rw-------`）。
**它带 lockdep**，锁校验开销很大 ⇒ **虚机里量到的时间不可与宿主比**，
但**计数类指标（I/O 次数、块数）不受影响**。引用虚机跑出来的时间数必须带这一句。
⚠️ **要比时间，用 `vm-kernel.sh --release $(uname -r)` 取宿主正在跑的 OEM 内核**（E152（按里程碑对比六家文件系统的文件性能） 就这么跑）：
不带 lockdep，同一个测试台上几次虚机跑的时间可以互相比；它仍然不与宿主上的时间比（虚拟化与 `cache=none` 都在里面）。
lockdep 内核量出来的时间连虚机之间都不宜比：锁校验的开销随加锁次数走，被测对象加锁多少不一样（推测，没在本机量过）。

## 跑之前先做卫生检查

`vm-bench.sh` 现在带 `EXIT` 清理工作目录，但**开跑前仍然值得看一眼**
（这条清理是补上去的，补之前一次清出过 154 个目录、4.5 GB，详见文末历史版本）：

```bash
ps -eo pid,etimes,args --no-headers | grep '[q]emu-system'   # 残留虚机
ls -d /tmp/singlefs-vmbench.* 2>/dev/null | wc -l            # 残留工作目录
losetup -a                                                    # 残留 loop
```

⚠️ **不许用 `pkill -f`**（`.claude/singlefs-ai-sop/rules/command-safety.md`）：
模式串会命中 wrapper 自己的命令行。要停虚机就先 `ps` 看清楚，再用字面量 pid 杀。
`VM_KEEP=1` 可以保留工作目录供失败后翻现场。

## harness 自己要先自检

```bash
bash scripts/vm-bench.sh --selftest
```

它跑三个用例：必然成功的、必然失败的、以及查得到 `/dev/vda` 的。
**分辨不出失败的那个，说明这个 harness 会把失败当成成功**——
这与 `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」同一条。

## 虚机里多了什么、宿主上没有的

| 量 | 宿主上跑（普通文件） | 虚机里跑（`/dev/vda`） |
|---|---|---|
| I/O 次数、块数 | 有 | 有 |
| **块层独立读数**（`/sys/block/vda/stat`） | **没有**（`blkstat=false`） | **有** |
| 时间 | 可比 | lockdep 内核：**不可比**；`--release` 取的 OEM 内核：同一测试台上的几次虚机跑之间可比，仍不与宿主比 |

⚠️ **块层独立读数是好几个实验的校验路径**——
E7（离线索引 harness） 把「块层与程序计数器逐格相符」列为它四层校验之一，
E12（攒批的顺序追加 vs 不攒批的随机页读改写） 更是靠它做过一次故障注入自证
（摘掉 `O_DIRECT` 后块层读数归零而程序计数不变）。
⇒ **在普通文件上复跑这些实验，复现的是数，不是那条校验。** 引用时必须分开说。

## 结果抓取有一道完整性闸

`vm-bench.sh` 比对「程序声称发了几条」与「宿主抓到几条」，对不上整轮作废。
理由见 `.claude/singlefs-ai-sop/rules/command-safety.md`：控制台会被 BIOS 转义序列
顶掉行首，按 `^` 锚定会静默漏掉那一行。所以抓取用的是 `grep -ao`，不锚行首。

## 屏蔽 CPU 特性那一档另有一道闸：已知答案测试

`VM_CPU` 能让来宾看不见某个 CPU 特性（`VM_CPU="host,-aes,-vaes"`）。
**一旦 `VM_CPU` 里出现 `-<特性>`，`vm-bench.sh` 就要求被测程序发过至少一条
`E7RESULT name=kat ... ok=true`**，否则整轮作废。

**为什么要这道闸**（2026-08-31 实测踩过，E6（加密算法选型））：
`-cpu host,-aes` 只摘掉 CPUID 的 `aes` 位，而 RustCrypto 的 `aes` 0.9.3
**分别**探 `aes` 与 `vaes` 两个位（`src/lib.rs` 第 152–154 行）⇒ 来宾仍走 AES 指令：

| 观测 | `-aes` | `-aes,-vaes` |
|---|---|---|
| AES 4 KiB 吞吐 | 3747.63 MiB/s（与没屏蔽差 0.5%）| 244.74 MiB/s（掉 14.94×）|
| AES-256-GCM 的标签 | **`2112edc9…`，错的** | `62d27233…`，对的 |

⇒ **那一档跑得飞快、条数对得上、退出码 0，而算的不是那个算法。**
只看吞吐分不出「屏蔽没生效」和「屏蔽生效了但算错了」——两者都表现为「数字没变」。

**怎么写这条 kat**：期望值必须由**另一个实现**给出（例：`python3` 调 `cryptography`），
不许拿被测程序自己在别的档上的输出当期望值——那是自己和自己比。

**这道闸自己证过会红**（2026-08-31，三个用例）：屏蔽档不发 kat ⇒ 判红；
屏蔽档发 `ok=false` ⇒ 判红；不屏蔽且不发 kat ⇒ 放行。

## 设备侧独立录制：blklogwrites 模式

`VM_BLKLOGWRITES_DIR=<目录>` 时，`vm-bench.sh` 在每块盘前面套一层 QEMU 的 `blklogwrites` 过滤节点（本机 QEMU 8.2.2 带这个驱动，2026-09-14 现查）。
来宾发到盘上的每个写（带数据）与每个 FLUSH 按 dm-log-writes 格式记进 `<目录>/log<d>.img`；数据盘也放在同一个目录（`disk<d>.img`），跑完不删。
录制在来宾之外，与被测程序自己的录制器不共享代码。解析与比对在 `crates/singlefs-harness/src/device_log.rs`，判据在门禁 55 号（`.claude/gate.d/55-qemu-first-transaction.sh`）。

| 现象（2026-09-14 实测，第一个事务） | 口径 |
|---|---|
| 每块盘的设备侧日志末尾比程序多一个 FLUSH | 程序在每块盘上的最后一件事是超级块槽写、后面没有 FLUSH，关机时补了这一个；比对只许末尾多一个 |
| 来宾 `/sys/block/<名>/stat` 的写请求数 = 程序的写数 + FLUSH 数（盘 0：23 + 12 = 35） | 比对用写扇区数（= 字节 ÷ 512）与 FLUSH 数；FLUSH 数是 stat 的下标 15，不是 14（14 是 discard 耗时，读它会得到 0） |
| 走页缓存时设备侧的写被合并 | m1（32 KiB）与 m2（16 KiB）回写成一个 48 KiB 的写；这是阳性对照判红的地方 |

⚠️ 这一档验的是「程序发出的写与 FLUSH 在虚拟设备上原样到达、次序不变」，不是真盘的持久语义：`cache=none` 下设备侧 FLUSH 映射成宿主上的 `fdatasync`，宿主盘自己的缓存不在射程里。

## 附加根与盘镜像预分配（E152（按里程碑对比六家文件系统的文件性能） 加的两个开关）

| 开关 | 做什么 | 口径 |
|---|---|---|
| `VM_EXTRA_ROOT=<目录>` | 目录里的东西原样并进 initramfs 的根 | E152（按里程碑对比六家文件系统的文件性能） 用 `research/scripts/e152-stage-root.sh` 生成它：fio、各家格式化与挂载工具连同 ldd 解出的全部库、解压过的内核模块与加载次序，共 129 MB；initramfs 按 gzip -1 打包，每次起虚机多几秒 |
| `VM_DISK_PREALLOCATE=1` | 盘镜像用 fallocate 预分配，不用 truncate 出稀疏文件 | 宿主 ext4 不在来宾写的时候边写边分配；预分配失败就不跑这一次，不退回稀疏文件 |

⚠️ **往附加根里放动态链接的程序，库（尤其 `/lib64/ld-linux-x86-64.so.2`）要带执行位。**
内核执行动态程序时先执行 ELF 里写的那个加载器；加载器没有执行位，来宾里每个动态程序都报 Permission denied，而静态链接的程序照常跑。
实测（2026-09-15，E152（按里程碑对比六家文件系统的文件性能） 第一次冒烟跑）：库按 0644 装，八个配置里七个卡在第一个外部命令，只有静态链接的 singlefs 二进制跑通。
`e152-stage-root.sh` 现在按 0755 装，并在搭完时用搭好的加载器与库实际跑一次搭好的 fio，这一步没过就不交出附加根。

## 历史版本

### 2026-09-15
- **新增** `VM_EXTRA_ROOT`、`VM_DISK_PREALLOCATE` 两个开关与 `vm-kernel.sh --release`。**依据**：E152（按里程碑对比六家文件系统的文件性能） 要在同一台虚机里挂六家文件系统跑 fio，要一个与宿主同版本、不带 lockdep 的内核，而最小 initramfs 里没有这些程序与模块。

### 2026-08-29
- **曾经**：`vm-bench.sh` 在子 shell 里 `mktemp -d` 出工作目录，路径传不回父进程，
  于是从不清理。**现在**：用它已有的日志指针文件反推工作目录，在 `EXIT` 里删掉
  （只删名字符合 `singlefs-vmbench.*` 的那一个；`VM_KEEP=1` 保留）。
  **依据**：当日清出 **154 个残留目录、4.5 GB**；病根是
  `.claude/singlefs-ai-sop/rules/command-safety.md`「子 shell 里的赋值传不回父进程」。
- **新增** `scripts/vm-kernel.sh`：每次跑虚机都卡在「`/boot` 下的内核不可读」，
  做成脚本一次解决。**依据**：本机 `/boot/vmlinuz-*` 除 lockdep 那个外都是 `-rw-------`。
