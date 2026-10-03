import ctypes, os, sys
libc = ctypes.CDLL(None, use_errno=True)
libc.fallocate.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_longlong, ctypes.c_longlong]
for path in sys.argv[1:]:
    fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o600)
    os.write(fd, b"\xC3" * 65536)
    rc = libc.fallocate(fd, 3, 32768, 65536)   # straddles EOF at 65536
    e = ctypes.get_errno()
    inside = os.pread(fd, 32768, 32768)
    before = os.pread(fd, 32768, 0)
    rc2 = libc.fallocate(fd, 3, 131072, 4096)  # wholly beyond EOF
    size = os.fstat(fd).st_size
    print(f"{path}: straddle rc={rc} errno={e} in-file part zeroed={all(b==0 for b in inside)} head untouched={all(b==0xC3 for b in before)}; beyond-EOF rc={rc2}; size={size}")
    os.close(fd); os.unlink(path)
