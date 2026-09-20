# 换设备型号再量：虚机里拿不拿得到 `fua=1` 的盘（2026-09-20，主 agent 现查）

接 `d13-item4-fua-r1-device-readings.md`。那一份量出来的 `fua=0` 是 **virtio-blk 的性质，不是虚机的性质**。这一份换设备型号各量一次。

同一个内核（`/boot/vmlinuz-6.17.0-lockdep`）、同一个 initramfs，在宿主上直接调 `qemu-system-x86_64`，没经过 `research/scripts/vm-bench.sh`。initramfs 里只有 busybox、一个读 sysfs 的 init、以及 nvme 那四个模块。

## 一、三种设备型号

| QEMU 设备 | 来宾看到 | write_cache | fua |
|---|---|---|---|
| `-device virtio-blk-pci` | `vda` | write back | **0** |
| `-device virtio-scsi-pci` + `scsi-hd` | `sda` | write back | **0** |
| `-device nvme` | `nvme0n1` | write back | **1** |

`-device nvme` 那一次的控制台原样输出：

```
INSMOD nvme-keyring rc=0
INSMOD nvme-auth rc=0
INSMOD nvme-core rc=0
INSMOD nvme rc=0
FUAPROBE dev=nvme0n1 write_cache=write back fua=1 rotational=0
FUAPROBE done
```

宿主那块真盘是 `write back` + `fua=1`（见 `d13-item4-fua-r1-device-readings.md` 第一节），与 `-device nvme` 这一行的两项读数逐字相同。

## 二、nvme 那一路的四个模块，次序不能错

本机内核 `CONFIG_BLK_DEV_NVME=m`、`CONFIG_NVME_CORE=m`，所以 initramfs 里要自己 insmod。按次序四个，都在 `/lib/modules/6.17.0-lockdep/kernel/drivers/nvme/`：

```
nvme-keyring   common/nvme-keyring.ko
nvme-auth      common/nvme-auth.ko
nvme-core      host/nvme-core.ko
nvme           host/nvme.ko
```

少了前两个，`nvme-core` 装不上，报一屏 `Unknown symbol nvme_auth_transform_key (err -2)` 一类。**而「盘没出现」这件事本身不报错**：来宾的 `/sys/block` 下只剩一个 `sr0`，探针照常跑完、照常退出 0。踩过一次，记在这里。

## 三、设备侧录制在 NVMe 底下也成立

C6（块层语义假设写错）要的那条独立录制路（QEMU 的 `blklogwrites` 过滤节点）套在 `-device nvme` 底下照样工作。来宾里往 `/dev/nvme0n1` 写 3 个 4 KiB 块加 fsync，宿主上录制日志的超级块就写出来了：

```
$ xxd -l 32 log0.img
00000000: 7268 7377 6673 6a00 0100 0000 0000 0000  rhswfsj.........
00000010: 0200 0000 0000 0000 0002 0000 0000 0000  ................
```

头八字节 `rhswfsj\0` 是 dm-log-writes 的超级块魔数。跑法：

```
-blockdev driver=file,node-name=data0,filename=<数据镜像>,cache.direct=on,aio=native
-blockdev driver=file,node-name=logfile0,filename=<录制日志>
-blockdev driver=blklogwrites,node-name=logwrites0,file=data0,log=logfile0,log-sector-size=512,log-append=off
-device nvme,drive=logwrites0,serial=<序列号>
```

## 四、这一段对 C6 说明什么

C6 欠的那一半是「拿虚机的设备侧日志重建崩溃状态再跑记录核对器」。门禁 55 号今天用 virtio-blk，那块盘上 FUA 由块层补发 FLUSH 来模拟，所以「FUA 已落到介质、而它前面的普通写没落」这个状态在那台装置上**造不出来**。换成 `-device nvme` 就造得出来，而且它与宿主真盘的两项读数相同。

所以这一半不是做不成，是装置选错了型号。
