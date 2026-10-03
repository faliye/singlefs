import ctypes, os, signal, sys, time
libc = ctypes.CDLL(None, use_errno=True)
libc.fallocate.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_longlong, ctypes.c_longlong]
cnt = {"n": 0}
def handler(sig, frm): cnt["n"] += 1
signal.signal(signal.SIGALRM, handler)
path = sys.argv[1]; size = int(sys.argv[2]) << 20; cycles = int(sys.argv[3])
fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o600)
os.ftruncate(fd, size)
chunk = b"\xC3" * (1 << 20)
ok = eintr = other = 0; others = {}; sig_during_punch = 0; punch_secs = 0.0
for c in range(cycles):
    signal.setitimer(signal.ITIMER_REAL, 0, 0)
    os.lseek(fd, 0, 0)
    for _ in range(size >> 20): os.write(fd, chunk)
    signal.setitimer(signal.ITIMER_REAL, 0.00002, 0.00002)
    before = cnt["n"]; t = time.time()
    rc = libc.fallocate(fd, 3, 0, size)       # one big punch over a fully dirty range
    punch_secs += time.time() - t
    sig_during_punch += cnt["n"] - before
    if rc == 0: ok += 1
    else:
        e = ctypes.get_errno()
        if e == 4: eintr += 1
        else: other += 1; others[e] = others.get(e, 0) + 1
signal.setitimer(signal.ITIMER_REAL, 0, 0)
os.close(fd); os.unlink(path)
print(f"path={path} size_mib={size>>20} big_punches={cycles} ok={ok} EINTR={eintr} other={other} {others} signals_handled_during_punch_calls={sig_during_punch} punch_secs={punch_secs:.3f}")
