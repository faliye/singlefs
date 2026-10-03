//! k4t1：今天的可写挂载对「系统配置槽 / 根槽一时读不出」的两种时机各走哪一遍。
//! 时机甲（登记字面的 T1）：这次挂载里对那个落点的第一次读读不出、之后都读得出（每段只坏第一次读）。
//! 时机乙（E165 r4 生成器的读法）：读阶段第一遍里一直读不出，重读之前撤掉（钩子里撤）。
//! 池：内存稀疏盘两块（4 GiB），新池新建文件（mkfs、取号、暖机、第一个文件），之后不再写。
use std::cell::RefCell;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ReadStageSettled, ShadowLedger,
};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::run_new_pool_file_creation;
use singlefs_harness::SharedStream;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Timing {
    OnlyTheFirstRead,
    UntilTheReread,
}

#[derive(Clone, Debug)]
struct Range {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
}

#[derive(Default)]
struct FaultState {
    ranges: Vec<Range>,
    timing: Option<Timing>,
    lifted: bool,
    reads: Vec<u64>,
    failed: Vec<u64>,
    sequence: u64,
    first_read_sequence: Vec<Option<u64>>,
}

struct Faulty {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    state: Rc<RefCell<FaultState>>,
}

impl BlockDevice for Faulty {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let length = buffer.len() as u64;
        let mut fail = false;
        {
            let mut s = self.state.borrow_mut();
            s.sequence += 1;
            let seq = s.sequence;
            let timing = s.timing;
            let lifted = s.lifted;
            for i in 0..s.ranges.len() {
                let r = s.ranges[i].clone();
                if r.device != self.identity || offset.0 >= r.offset + r.length || offset.0 + length <= r.offset {
                    continue;
                }
                s.reads[i] += 1;
                if s.first_read_sequence[i].is_none() {
                    s.first_read_sequence[i] = Some(seq);
                }
                let fails = match timing {
                    Some(Timing::OnlyTheFirstRead) => s.reads[i] == 1,
                    Some(Timing::UntilTheReread) => !lifted,
                    None => false,
                };
                if fails {
                    s.failed[i] += 1;
                    fail = true;
                }
            }
        }
        if fail {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("r3probe 读故障")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn fresh_pool() -> (Vec<(DeviceIdentity, SparseBlockDevice)>, singlefs_core::transaction::TransactionOutput) {
    let width = HistoryDeviceWidth::FourGibibytes;
    let parameters = width.parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|d| (d, SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect();
    let stream = SharedStream::new();
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_, _| {}).expect("新池新建文件");
    (devices, run.output)
}

fn newest_system_slot(devices: &Vec<(DeviceIdentity, SparseBlockDevice)>, device: DeviceIdentity) -> Range {
    let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let slots = verified_system_configuration_slots(devices, device, spacing, &parameters.filesystem_identifier);
    let newest = slots.iter().map(|s| s.quantities.slot_generation).max().expect("有自证过的槽");
    Range { device, offset: (newest % 2) * spacing, length: 4096 }
}

fn newest_root_slot(output: &singlefs_core::transaction::TransactionOutput) -> Range {
    let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
    let target = singlefs_core::root_ring::target_for_publish(output.root.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
    Range {
        device: parameters.region_devices[usize::try_from(target.region).unwrap()],
        offset: singlefs_core::root_ring::slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0,
        length: u64::from(parameters.geometry.physical_block_size),
    }
}

pub fn run() {
    let sites = ["none", "SN0", "SN1", "SB", "RN", "RNSB"];
    for site in sites {
        for timing in [Timing::OnlyTheFirstRead, Timing::UntilTheReread] {
            if site == "none" && timing == Timing::UntilTheReread {
                continue;
            }
            let (devices, output) = fresh_pool();
            let mut ranges = Vec::new();
            if site == "SN0" || site == "SB" || site == "RNSB" {
                ranges.push(newest_system_slot(&devices, DeviceIdentity(0)));
            }
            if site == "SN1" || site == "SB" || site == "RNSB" {
                ranges.push(newest_system_slot(&devices, DeviceIdentity(1)));
            }
            if site == "RN" || site == "RNSB" {
                ranges.push(newest_root_slot(&output));
            }
            let n = ranges.len();
            let state = Rc::new(RefCell::new(FaultState {
                ranges: ranges.clone(),
                timing: if site == "none" { None } else { Some(timing) },
                lifted: false,
                reads: vec![0; n],
                failed: vec![0; n],
                sequence: 0,
                first_read_sequence: vec![None; n],
            }));
            let mut faulty: Vec<(DeviceIdentity, Faulty)> = devices
                .into_iter()
                .map(|(identity, inner)| (identity, Faulty { identity, inner, state: Rc::clone(&state) }))
                .collect();
            let lifting = Rc::clone(&state);
            let mut hook_calls = 0u32;
            let mut hook = || {
                hook_calls += 1;
                lifting.borrow_mut().lifted = true;
            };
            let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
            let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
                &parameters,
                &mut faulty,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
            );
            let outcome = match &mounted {
                Ok(m) => match m.output.rereads.read_stage {
                    ReadStageSettled::OnTheFirstRead { .. } => "mounted:OnTheFirstRead".to_string(),
                    ReadStageSettled::OnTheOneReread { .. } => "mounted:OnTheOneReread".to_string(),
                },
                Err(e) => format!("refused:{}", format!("{e:?}").chars().take(120).collect::<String>()),
            };
            let s = state.borrow();
            let ranges_text: Vec<String> = ranges
                .iter()
                .enumerate()
                .map(|(i, r)| format!("d{}@{}:reads={}:failed={}:first_read_seq={:?}", r.device.0, r.offset, s.reads[i], s.failed[i], s.first_read_sequence[i]))
                .collect();
            println!(
                "name=k4t1 site={} timing={:?} outcome={} hook_calls={} ranges=[{}] total_reads={}",
                site,
                timing,
                outcome,
                hook_calls,
                ranges_text.join(" "),
                s.sequence
            );
        }
    }
}
