import json, os, subprocess
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet')
T=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree')
BIN=W+'/target-scan/release/scan-model'
GPU='crates/singlefs-checker-tier/src/gpu_unit_checks.rs'; CR='crates/singlefs-checker-tier/src/crash_amplification.rs'
def run(mode, path): return subprocess.run([BIN,mode,path],capture_output=True,text=True).stdout.strip()
rows=[]
for r in json.load(open(W+'/mut/gpu-rows.json')):
    if r[1] in (GPU,CR): rows.append((r[0],r[1],r[2].replace('\\n','\n'),r[3].replace('\\n','\n'),'table'))
ex=[
 ("A1 整句包进 if false（账不再被告知）",GPU,"                self.ledger.take_in_the_devices_kept_before_booking(\n","                if false { self.ledger.take_in_the_devices_kept_before_booking(\n"),
 ("A2 函数体最前面 return（整句不执行）",GPU,"            fn take_in_the_devices_found_idle(&self) {\n                self.ledger.take_in_the_devices_kept_before_booking(","            fn take_in_the_devices_found_idle(&self) {\n                if true { return; }\n                self.ledger.take_in_the_devices_kept_before_booking("),
 ("A3 丢没丢与 poll 两个闭包对调位置（类型相同，编得过）",GPU,"                    |started: &GpuCard| started.device_condition().is_lost(),\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(&std::panic::catch_unwind(\n                            std::panic::AssertUnwindSafe(|| {\n                                started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n","                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(&std::panic::catch_unwind(\n                            std::panic::AssertUnwindSafe(|| {\n                                started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n                    |started: &GpuCard| started.device_condition().is_lost(),\n"),
 ("A4 丢没丢的闭包后面追加 && false（不改闭包前缀的写法）",GPU,"|started: &GpuCard| started.device_condition().is_lost(),","|started: &GpuCard| started.device_condition().is_lost() && false,"),
 ("A5 poll 的闭包体里 PollType::Poll 换成 PollType::Wait（会等）",GPU,"started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n                    |started: &GpuCard| {\n                        started\n","started.device().poll(wgpu::PollType::wait_indefinitely())\n                            }),\n                        ))\n                    },\n                    |started: &GpuCard| {\n                        started\n"),
 ("A6 叫停：层首 shadow 成永不叫停",CR,"        stop_is_requested: &(dyn Fn() -> bool + Sync),\n    ) -> Result<(), String> {\n","        stop_is_requested: &(dyn Fn() -> bool + Sync),\n    ) -> Result<(), String> {\n        let stop_is_requested: &(dyn Fn() -> bool + Sync) = &|| false;\n"),
 ("A7 叫停：往下交的换成永不叫停、另加一句 let _ = stop_is_requested",CR,"        checking_threads(),\n        stop_is_requested,","        checking_threads(),\n        { let _ = stop_is_requested; &|| false },"),
]
for e in ex: rows.append((e[0],e[1],e[2],e[3],'extra'))
print('baseline gpu  :',run('gpu',T+'/'+GPU)); print('baseline crash:',run('crash',T+'/'+CR))
os.makedirs(W+'/scanmut',exist_ok=True)
for name,file,old,new,kind in rows:
    src=open(T+'/'+file,encoding='utf-8').read()
    n=src.count(old)
    if n!=1: print(kind,name[:60],'OLD-NOT-ONCE',n); continue
    p=W+'/scanmut/m.rs'; open(p,'w',encoding='utf-8').write(src.replace(old,new))
    mode='gpu' if file==GPU else 'crash'
    print(kind,'|',name[:68],'|',mode,'|',run(mode,p)[:110])
