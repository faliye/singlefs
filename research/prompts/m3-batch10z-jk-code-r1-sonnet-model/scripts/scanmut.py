import subprocess, os
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet')
T=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-jk-code-r1/tree')
BIN=W+'/target-scan/release/scan-model'
def run(mode, path):
    return subprocess.run([BIN,mode,path],capture_output=True,text=True).stdout.strip()
rows=[]
for line in open(W+'/muts-k.tsv',encoding='utf-8'):
    f=line.rstrip('\n').split('\t')
    if '-p singlefs-checker-tier' in f[4]:
        rows.append((f[0],f[1],f[2].replace('\\n','\n'),f[3].replace('\\n','\n')))
GPU='crates/singlefs-checker-tier/src/gpu_unit_checks.rs'; CR='crates/singlefs-checker-tier/src/crash_amplification.rs'
extra=[
 ('Y1 poll 之后不管空没空都销',GPU,"                    if device_found_idle_by_a_poll(&polled) {","                    if true || device_found_idle_by_a_poll(&polled) {"),
 ('Y2 poll 之后从不销',GPU,"                    if device_found_idle_by_a_poll(&polled) {","                    if false && device_found_idle_by_a_poll(&polled) {"),
 ('Y3 列卡的循环只看第一张卡',GPU,"                for listed in self.ledger.leases_kept_after_a_failed_wait_on_each_card() {","                for listed in self.ledger.leases_kept_after_a_failed_wait_on_each_card().into_iter().take(1) {"),
 ('Y4 留着的设备丢了的照 poll',GPU,"                    if started.device_condition().is_lost() {\n                        continue;\n                    }","                    if false {\n                        continue;\n                    }"),
 ('Y5 叫停换成每次现读文件',CR,"            gpu_cards_taken_from,\n            &stop_is_requested,\n        )? {","            gpu_cards_taken_from,\n            &stop_is_requested_by_the_environment,\n        )? {"),
]
print('baseline gpu :',run('gpu',T+'/'+GPU))
print('baseline crash:',run('crash',T+'/'+CR))
os.makedirs(W+'/scanmut',exist_ok=True)
for name,file,old,new in rows+extra:
    src=open(T+'/'+file,encoding='utf-8').read()
    n=src.count(old)
    if n!=1:
        print(name[:60],'OLD-NOT-ONCE',n); continue
    p=W+'/scanmut/m.rs'
    open(p,'w',encoding='utf-8').write(src.replace(old,new))
    mode='gpu' if file==GPU else 'crash'
    print(name[:70],'|',mode,'|',run(mode,p)[:150])
