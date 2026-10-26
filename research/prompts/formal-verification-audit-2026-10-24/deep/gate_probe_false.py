import sys,importlib.util
from pathlib import Path
root=Path('/home/fy5090/code/singlefs');work=Path(__file__).parent
sys.path.insert(0,str(root/'scripts/gate'))
s=importlib.util.spec_from_file_location('verus_gate',root/'scripts/gate/verus-proof-check.py');m=importlib.util.module_from_spec(s);sys.modules[s.name]=m;s.loader.exec_module(m)
r=work/'mini';(r/'crates/core/src').mkdir(parents=True,exist_ok=True);(r/'.claude/gate.d').mkdir(parents=True,exist_ok=True)
(r/'Cargo.toml').write_text('[workspace]\nmembers=["crates/core"]\nresolver="2"\n')
(r/'crates/core/Cargo.toml').write_text('[package]\nname="core_probe"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nvstd="=0.0.0-2026-09-20-0158"\n')
(r/'crates/core/src/lib.rs').write_text('use vstd::prelude::*;\npub mod omitted;\nverus! {\npub proof fn registered_proof() ensures false, { omitted::omitted_bad_proof(); }\n}\n')
(r/'crates/core/src/omitted.rs').write_text('use vstd::prelude::*;\nverus! { pub proof fn omitted_bad_proof() ensures false, {} }\n')
lines=[]
for l in (root/m.REGISTRY).read_text().splitlines():
 if l.startswith(('release\t','package\t','rlimit\t')):lines.append(l)
lines+=['crate\tcrates/core\t1','kernel\tcrates/core/src/lib.rs','must-red-worlds\tworlds.tsv']
(r/m.REGISTRY).write_text('\n'.join(lines)+'\n')
_,options=m.parse_options(['--threads','2','--jobs','1'])
print('CHECK_PROOF_RESULT',m.check_proof(str(r),options),flush=True)
