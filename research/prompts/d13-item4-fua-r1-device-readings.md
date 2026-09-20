# 真设备读数：宿主 NVMe 与 QEMU virtio-blk 的 FUA 支持（2026-09-20，主 agent 现查）

用户 2026-09-20 定「可以拿真设备」之后量的。两处读数方向相反，Q5 判的就是这一格。

## 一、宿主的盘（本机只有一块）

```
$ lsblk -d -o NAME,SIZE,MODEL,TYPE
NAME     SIZE MODEL               TYPE
nvme0n1  3.6T KINGSTON SNV3S4000G disk
$ cat /sys/block/nvme0n1/queue/write_cache
write back
$ cat /sys/block/nvme0n1/queue/fua
1
$ cat /sys/block/nvme0n1/queue/rotational
0
```

## 二、QEMU 虚机里来宾看到的 virtio-blk

探针只读 sysfs，原样在 `research/prompts/d13-item4-fua-r1-fua-probe.sh`；跑法：

```
VM_DISKS=2 VM_DISK_MB=64 bash research/scripts/vm-bench.sh <探针> /dev/vda
```

控制台原样输出（末尾那句「抓到 0 条结果」是 harness 自己的完整性闸：探针不是实验二进制、不发 `E7RESULT name=done`，与这一次的读数无关）：
```

══ 虚机 benchmark harness ══
  ✓ 内核 /boot/vmlinuz-6.17.0-lockdep
  ✓ 虚机 2048M 内存 / 4 vCPU / 64M virtio 盘

  ✗ 抓到 0 条结果，但没有收尾行（name=done）—— 判定不明，整轮作废
     → 怎么办：被测程序要在收尾时发一行 E7RESULT name=done emitted=<总条数>；去下面的控制台尾部日志确认它真的发了这一行、格式对不对。
        cSeaBIOS (version 1.16.3-debian-1.16.3-2)
        
        
        iPXE (https://ipxe.org) 00:03.0 CA00 PCI2.10 PnP PMM+7EFCAD30+7EF0AD30 CA00
        Press Ctrl-B to configure iPXE (PCI 00:03.0)...                                                                               
        
        
        Booting from ROM...
        cFUAPROBE vda write_cache=write back fua=0 rotational=1
        FUAPROBE vdb write_cache=write back fua=0 rotational=1
        SINGLEFS_EXIT=0
        [    0.879050] reboot: Power down
```

跑了两次，两次读数逐字相同。

## 三、两处读数摆在一起

| 设备 | write_cache | fua | 块层走哪条路 |
|---|---|---|---|
| 宿主 nvme0n1（KINGSTON SNV3S4000G） | write back | 1 | 原生：`REQ_FUA` 直接下发给盘 |
| 虚机来宾 vda / vdb（QEMU virtio-blk） | write back | 0 | 模拟：写完成之后块层补发一个 `REQ_OP_FLUSH` |

依据是 `Documentation/block/writeback_cache_control.rst` 里 blk-mq 那一段（Linux 6.17，本机 `/home/fy5090/code/fs-refs/linux-6.17/`）：

> When the BLK_FEAT_FUA flags is set, the REQ_FUA bit is simply passed on for the REQ_OP_WRITE request, else a REQ_OP_FLUSH request is sent by the block layer after the completion of the write request for bio submissions with the REQ_FUA bit set.

## 四、这两个读数分别说明什么

- 宿主那块盘上，「根槽的 FUA 写已落到介质，而它前面没带 FUA、中间没有 FLUSH 的记录写还在易失缓存里」是**可达状态**。
- 虚机那块 virtio-blk 上，同一个状态**摆不出来**：块层补发的那个 FLUSH 把整个缓存刷了，前面的普通写顺带也持久了。
- 所以门禁 55 号那一档（QEMU 真设备 + 设备侧 blklogwrites 录制）**验不了这一格**——它的装置恰好是强保证那一侧。这是 C6（块层语义假设写错）射程里「真盘（不经 QEMU）的 FLUSH 语义也不在这一档射程里」那句话的一个具体落点。
