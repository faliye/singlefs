import subprocess, sys, shutil, os, json
BASE = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-a-code-r1-sonnet')
SRC = f'{BASE}/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs'
DST = f'{BASE}/tree-mut/crates/singlefs-harness/src/gpu_memory_ledger.rs'
R = '/home/fy5090/code/singlefs/research/scripts'
MUTS = {
 'N12 见到的那一份不扣销了账的（照没压的算）': ('''            reading,
            self.booked(),
            mebibytes_left_for_the_driver,
        );
        if seen_now''', '''            reading,
            BookedByThisProcessOnACard { returned_possibly_still_on_the_card_mebibytes: 0, ..self.booked() },
            mebibytes_left_for_the_driver,
        );
        if seen_now'''),
 'K0 对照：实现员行 004 的改法（压时不看见到的）': ('.min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))', '.min(mebibytes)'),
 'N1 drop 的 WorkOnTheCard 回到没有（不回原来那一段）': ('WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(self.work_of_this_thread_before));', 'WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(None));'),
 'N2 编内核计数 <=1 改 <=2': ('if still_compiling <= 1 {', 'if still_compiling <= 2 {'),
 'N3 编内核计数 += 1 改 = 1': ('+= 1;', '= 1;'),
 'N4 见到的更新条件 > 改 !=': ('if seen_now_mebibytes > self.seen_on_the_card_at_least_mebibytes {', 'if seen_now_mebibytes != self.seen_on_the_card_at_least_mebibytes {'),
 'N5 那一刻活着的借据号不记': ('self.leases_seen_on_the_card = self.leases.keys().copied().collect();', 'self.leases_seen_on_the_card = BTreeSet::new();'),
 'N6 那一刻活着的借据号累加不替换': ('self.leases_seen_on_the_card = self.leases.keys().copied().collect();', 'self.leases_seen_on_the_card.extend(self.leases.keys().copied());'),
 'N7 重起不作废早先见到的数': ('self.seen_on_the_card_at_least_mebibytes = 0;', ''),
 'N8 重起不清那一刻活着的借据号': ('self.leases_seen_on_the_card.clear();', ''),
 'N9 借据销账时见到的不减': ('.saturating_sub(mebibytes);', '.saturating_sub(0);'),
 'N10 重起放得下判据 < 改 <=': ('if capacity_mebibytes < booked.leases_mebibytes {', 'if capacity_mebibytes <= booked.leases_mebibytes {'),
 'N11 占着的里属于许出去的不封顶': ('.min(promised_mebibytes),', ','),
}
only = sys.argv[2:] if len(sys.argv) > 2 else None
filt = sys.argv[1]
orig = open(SRC).read()
for name, (old, new) in MUTS.items():
    if only and name.split()[0] not in only:
        continue
    assert orig.count(old) == 1, (name, orig.count(old))
    open(DST, 'w').write(orig.replace(old, new))
    cmd = f'cd {BASE}/tree-mut && CARGO_TARGET_DIR={BASE}/target-mut nice -n 19 bash {R}/run-with-memory-cap.sh 12G bash {R}/capped.sh 15 cargo test --offline -p singlefs-harness --lib {filt}'
    p = subprocess.run(['bash', '-c', cmd], capture_output=True, text=True)
    out = p.stdout + p.stderr
    result = [l for l in out.splitlines() if l.startswith('test result') or l.startswith('error')]
    failed = [l for l in out.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
    print(f'{name}: exit={p.returncode} {result[-1:] } failed={len(failed)} {[f.split()[1].split("::")[-1] for f in failed][:4]}', flush=True)
open(DST, 'w').write(orig)
