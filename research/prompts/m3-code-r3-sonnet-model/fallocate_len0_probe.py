# 复现 r1 判决第 4、6 行与 r2 判决 J4 的 SAFETY 句依赖的内核事实：fallocate(PUNCH_HOLE|KEEP_SIZE) 长度 0、跨文件末尾。
# 镜像放在草稿目录（本机 ext4），跑完删掉。
import ctypes, os
libc = ctypes.CDLL(None, use_errno=True)
libc.fallocate.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_longlong, ctypes.c_longlong]
path = "/tmp/claude-1000/m3-code-r3-sonnet/punch.img"
fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o600)
os.write(fd, b"\xC3" * 65536)
for off, length in [(0, 0), (512, 0), (0, 512), (65536, 512), (0, 131072)]:
    rc = libc.fallocate(fd, 3, off, length)
    print(f"off={off} len={length} -> rc={rc} errno={ctypes.get_errno() if rc else 0}")
os.pwrite(fd, b"\xC3" * 65536, 0)
rc = libc.fallocate(fd, 3, 65536 - 512, 1024)
print("straddle(off=65024,len=1024) rc", rc, "size", os.fstat(fd).st_size,
      "in-file half zeroed", os.pread(fd, 512, 65536 - 512) == b"\0" * 512,
      "head untouched", os.pread(fd, 512, 0) == b"\xC3" * 512)
os.close(fd)
os.remove(path)
