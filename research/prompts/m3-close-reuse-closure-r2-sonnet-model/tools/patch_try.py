#!/usr/bin/env python3
"""W1 的改法原型（只在副本里，不是实现）：把 run-with-memory-cap.sh 里 HostLookup 的 begin / finish / run / cancel 按丙第 1 件包进 try，
join 加上时限。用法：patch_try.py <原脚本> <改完的脚本> [--bounded-join]。每一处旧串必须恰好命中一次。"""
import sys

source, target = sys.argv[1], sys.argv[2]
bounded = "--bounded-join" in sys.argv[3:]
text = open(source, encoding="utf-8").read()


def replace_once(old, new):
    global text
    assert text.count(old) == 1, f"旧串不是恰好一处：{old[:60]!r}"
    text = text.replace(old, new)


replace_once('''        self.thread = threading.Thread(target=self.run, args=("queued", local), daemon=True)
        self.thread.start()
''', '''        try:
            self.thread = threading.Thread(target=self.run, args=("queued", local), daemon=True)
            self.thread.start()
        except Exception as error:   # 起不了线程：这一支等于没有，排队照旧
            self.thread = None
            say(f"run-with-memory-cap: 没查到别的机器：起不了查的线程（{type(error).__name__}）")
''')
join_limit = "(HOST_LOOKUP_SECONDS + HOST_LOOKUP_PEER_TIMEOUT_SECONDS + 5)" if bounded else ""
replace_once('''        if self.thread is not None:
            self.thread.join()
            self.thread = None
        self.run("refused", local)
''', f'''        if self.thread is not None:
            self.thread.join({join_limit})
            self.thread = None
        self.run("refused", local)
''')
replace_once('''        if self.thread is not None:
            self.thread.join()
        if admitted and self.alerted:''', f'''        if self.thread is not None:
            self.thread.join({join_limit})
        if admitted and self.alerted:''')
replace_once('''    def run(self, phase, local):
        problem, peer = self.query()
''', '''    def run(self, phase, local):
        """查别的机器这一支出任何异常都只等于没有这个提示：打一行说明，不往外冒。"""
        try:
            self.run_unguarded(phase, local)
        except Exception as error:
            say(f"run-with-memory-cap: 没查到别的机器：查的时候出了 {type(error).__name__}")

    def run_unguarded(self, phase, local):
        problem, peer = self.query()
''')
replace_once('''        with self.guard:
            self.stopped = True
            process = self.process
        if process is not None and process.poll() is None:
            process.kill()   # 只停自己起的这一个（按它的进程号）；它起的 ssh 由 ssh 自己收尾''', '''        with self.guard:
            self.stopped = True
            process = self.process
        with contextlib.suppress(Exception):
            if process is not None and process.poll() is None:
                process.kill()   # 只停自己起的这一个（按它的进程号）；它起的 ssh 由 ssh 自己收尾''')
if bounded:
    replace_once('''class HostLookup:
''', '''@contextlib.contextmanager
def bounded_admission_lock(seconds):
    """查别的机器那一支拿排队锁只等几秒：拿不到当没记下谁查过（take_turn 里 OSError 照查）。"""
    if broken == "nolock":
        yield
        return
    with open(lock_path, "a", encoding="utf-8") as handle:
        deadline = time.monotonic() + seconds
        while True:
            try:
                fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    raise OSError("排队锁等了 %g 秒没拿到" % seconds)
                time.sleep(0.05)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


class HostLookup:
''')
    replace_once('''        try:
            with admission_lock():
                try:
                    with open(path, encoding="utf-8") as handle:
                        last = json.load(handle)''', '''        try:
            with bounded_admission_lock(2.0):
                try:
                    with open(path, encoding="utf-8") as handle:
                        last = json.load(handle)''')
with open(target, "w", encoding="utf-8") as handle:
    handle.write(text)
