import ctypes, os, signal, sys, time
libc = ctypes.CDLL(None, use_errno=True)
libc.fallocate.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_longlong, ctypes.c_longlong]
hits = {"alarm": 0}
def handler(sig, frm): hits["alarm"] += 1
signal.signal(signal.SIGALRM, handler)   # CPython installs it without SA_RESTART
path = sys.argv[1]; size = int(sys.argv[2]) << 20; cycles = int(sys.argv[3])
fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o600)
os.ftruncate(fd, size)
chunk = b"\xC3" * (1 << 20)
ok = eintr = other = 0
others = {}
t0 = time.time()
for c in range(cycles):
    os.lseek(fd, 0, 0)
    for _ in range(size >> 20): os.write(fd, chunk)
    signal.setitimer(signal.ITIMER_REAL, 0.00005, 0.00005)  # 50 us interval
    # punch in many small pieces so there are many syscalls in flight with signals landing
    piece = 1 << 20
    for off in range(0, size, piece):
        rc = libc.fallocate(fd, 3, off, piece)
        if rc == 0: ok += 1
        else:
            e = ctypes.get_errno()
            if e == 4: eintr += 1
            else: other += 1; others[e] = others.get(e, 0) + 1
    # one big punch
    os.lseek(fd, 0, 0)
    signal.setitimer(signal.ITIMER_REAL, 0, 0)
os.close(fd); os.unlink(path)
print(f"path={path} size_mib={size>>20} cycles={cycles} punches_ok={ok} EINTR={eintr} other={other} {others} sigalrm_handled={hits['alarm']} secs={time.time()-t0:.1f}")
