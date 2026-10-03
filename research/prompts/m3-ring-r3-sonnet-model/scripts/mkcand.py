import sys,shutil,os,re
base='/tmp/claude-1000/m3-ring-r3-sonnet/g3'
def sub(path,old,new,count=1):
    t=open(path).read()
    assert t.count(old)==count,(path,old[:60],t.count(old))
    open(path,'w').write(t.replace(old,new))
NEWFN='''
/// 候选（m3-ring-r3 辩方腿在冻结副本上移植）：同 [`verified_system_configuration_slots`] 每块盘两槽各读一次，另交回设备报读错的槽数。
#[must_use]
pub fn system_configuration_slots_read_once_with_device_read_errors<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> (Vec<SystemConfiguration>, u64) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut verified = Vec::new();
    let mut device_read_errors = 0u64;
    for offset in [0, slot_spacing_in_bytes] {
        match reader.read(device, DeviceOffsetInBytes(offset), slot_bytes) {
            None => device_read_errors += 1,
            Some(bytes) => {
                if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {
                    if system_configuration.immutable.filesystem_identifier == *filesystem_identifier {
                        verified.push(system_configuration);
                    }
                }
            }
        }
    }
    (verified, device_read_errors)
}

/// 全环扫描：每盘每个记录槽都读'''
for name,mode in (('eio','eio'),('dev','dev')):
    dst=f'/tmp/claude-1000/m3-ring-r3-sonnet/g3-{name}'
    if os.path.exists(dst): shutil.rmtree(dst)
    shutil.copytree(base,dst,ignore=shutil.ignore_patterns('target'))
    rec=f'{dst}/crates/singlefs-core/src/recovery.rs'
    sub(rec,'\n/// 全环扫描：每盘每个记录槽都读',NEWFN)
    mnt=f'{dst}/crates/singlefs-core/src/mount.rs'
    # 读阶段见证读
    t=open(mnt).read()
    i=t.index('    let verified_slots_of_each_device: Vec<Vec<crate::system_configuration::SystemConfiguration>> =')
    j=t.index('    let comparison = if some_system_configuration_slot_unread {')
    seg=t[i:j]
    new_seg='''    let slots_and_errors: Vec<(Vec<crate::system_configuration::SystemConfiguration>, u64)> = devices
        .device_identities()
        .into_iter()
        .map(|device| {
            crate::recovery::system_configuration_slots_read_once_with_device_read_errors(
                devices,
                device,
                slot_spacing_in_bytes,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    let witnessed_journal_counter = slots_and_errors
        .iter()
        .flat_map(|(slots, _)| slots.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
'''
    if mode=='eio':
        new_seg+='    let some_system_configuration_slot_unread = slots_and_errors.iter().any(|(_, errors)| *errors > 0);\n'
    else:
        new_seg+='    let some_system_configuration_slot_unread = !slots_and_errors.is_empty() && slots_and_errors.iter().all(|(_, errors)| *errors > 0);\n'
    open(mnt,'w').write(t[:i]+new_seg+t[j:])
    # 取号读
    tr=f'{dst}/crates/singlefs-core/src/transaction.rs'
    t=open(tr).read()
    i=t.index('    pool.devices\n        .iter()\n        .map(|(identity, _)| {\n            let slots_of_this_device = verified_system_configuration_slots(')
    j=t.index('        .collect()\n}',i)
    old=t[i:j]
    if mode=='eio':
        cond='device_read_errors > 0 || slots_of_this_device.is_empty()'
        pre=''
    else:
        cond='every_device_has_a_slot_with_a_device_read_error || slots_of_this_device.is_empty()'
        pre='''    let every_device_has_a_slot_with_a_device_read_error = !pool.devices.is_empty()
        && pool.devices.iter().all(|(identity, _)| {
            crate::recovery::system_configuration_slots_read_once_with_device_read_errors(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            )
            .1
                > 0
        });
'''
    new='''    pool.devices
        .iter()
        .map(|(identity, _)| {
            let (slots_of_this_device, device_read_errors) =
                crate::recovery::system_configuration_slots_read_once_with_device_read_errors(
                    &*pool.devices,
                    *identity,
                    spacing,
                    &filesystem_identifier,
                );
            let _ = device_read_errors;
            if %s {
                return Err(
                    InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
                        device: *identity,
                    },
                );
            }
            Ok(slots_of_this_device)
        })
''' % cond
    open(tr,'w').write(t[:i]+pre+new+t[j:])
print('ok')
