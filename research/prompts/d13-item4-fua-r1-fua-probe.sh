#!/bin/sh
# 只读 sysfs：来宾看到的 virtio-blk 队列特性
for d in /sys/block/vd*; do
  [ -d "$d" ] || continue
  n=$(/bin/busybox basename "$d")
  wc=$(cat "$d/queue/write_cache" 2>/dev/null)
  fua=$(cat "$d/queue/fua" 2>/dev/null)
  rot=$(cat "$d/queue/rotational" 2>/dev/null)
  echo "FUAPROBE $n write_cache=$wc fua=$fua rotational=$rot"
done
exit 0
