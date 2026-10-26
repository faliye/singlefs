from pathlib import Path
import subprocess,json,shutil,sys,importlib.util
root=Path('/home/fy5090/code/singlefs'); work=Path(__file__).parent
v=str(Path.home()/'.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus')
def run(label,src,extra=[]):
 r=subprocess.run([v,str(src),'--crate-type=lib','--no-cheating','--rlimit','10','--output-json','--num-threads','2',*extra],capture_output=True,text=True,cwd=src.parent)
 (work/(label+'.json')).write_text(r.stdout);(work/(label+'.stderr')).write_text(r.stderr)
 try: result=json.loads(r.stdout)['verification-results']
 except Exception: result=r.stderr[-1500:]
 print(label,r.returncode,result,flush=True)
 return result
base=work/'baseline'; shutil.copytree(root/'crates/singlefs-replay-prefix/src',base)
run('baseline',base/'lib.rs')
omitted=work/'omitted';shutil.copytree(base,omitted)
with (omitted/'lib.rs').open('a') as f:f.write('\npub mod omitted_proof;\n')
(omitted/'omitted_proof.rs').write_text('use vstd::prelude::*;\nverus! { pub proof fn impossible_claim() ensures false, {} }\n')
run('omitted-selected',omitted/'lib.rs',['--verify-root','--verify-function','choose_root_slot'])
run('omitted-whole',omitted/'lib.rs')
greedy=work/'greedy';shutil.copytree(base,greedy)
p=greedy/'compaction_plan_proofs.rs';s=p.read_text();a='Some(_) => segment.live_slots < best_live_slots,';assert s.count(a)==1;p.write_text(s.replace(a,'Some(_) => segment.live_slots > best_live_slots,'))
run('greedy-reversed',greedy/'lib.rs',['--verify-only-module','compaction_plan_proofs','--verify-function','choose_compaction_segment'])
