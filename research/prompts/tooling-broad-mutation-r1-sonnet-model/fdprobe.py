import os, subprocess, time, fcntl
d = os.open("lockfile", os.O_RDWR | os.O_CREAT, 0o644)
os.set_inheritable(d, True)
fcntl.flock(d, fcntl.LOCK_EX | fcntl.LOCK_NB)
print("driver lock fd number =", d)
p = subprocess.Popen(["bash", "-c", '{ sleep 4 2>&3 3>&-; } 3>&2 2>/dev/null'], pass_fds=(d,), stdin=subprocess.DEVNULL)
time.sleep(1)
def holders():
    out = []
    for pid in os.listdir("/proc"):
        if not pid.isdigit(): continue
        try:
            for fd in os.listdir(f"/proc/{pid}/fd"):
                if os.readlink(f"/proc/{pid}/fd/{fd}").endswith("/lockfile"):
                    out.append((pid, open(f"/proc/{pid}/comm").read().strip(), fd))
        except OSError: pass
    return out
print("holders while running:", holders())
p.wait()
