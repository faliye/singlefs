use std::cell::RefCell;
use std::rc::Rc;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use singlefs_harness::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
use singlefs_harness::read_tally::{ReadLoggingBlockDevice, JournalRingRegion, JournalHintAnswer};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::model::*;

fn control() -> bool { std::env::var_os("SINGLEFS_HARNESS_AUDIT_CONTROL").is_some() }

#[derive(Default, Debug, PartialEq, Eq)]
struct Calls { ordinary: usize, reread: usize, straight: usize, keep: usize, forget: usize, discard: usize }
struct Routes(Rc<RefCell<Calls>>);
impl BlockDevice for Routes {
    fn read_at(&self, _: DeviceOffsetInBytes, b: &mut [u8]) -> Result<(), BlockDeviceError> { self.0.borrow_mut().ordinary += 1; b.fill(0x11); Ok(()) }
    fn reread_at_after_a_bad_read(&self, _: DeviceOffsetInBytes, b: &mut [u8]) -> Result<(), BlockDeviceError> { self.0.borrow_mut().reread += 1; b.fill(0x22); Ok(()) }
    fn read_at_straight_from_the_disk(&self, _: DeviceOffsetInBytes, b: &mut [u8]) -> Result<(), BlockDeviceError> { self.0.borrow_mut().straight += 1; b.fill(0x33); Ok(()) }
    fn keep_the_self_verified_reread(&self, _: DeviceOffsetInBytes, _: &[u8]) { self.0.borrow_mut().keep += 1; }
    fn note_the_read_failing_self_verification_and_forget_its_bytes(&self, _: DeviceOffsetInBytes, _: u64) { self.0.borrow_mut().forget += 1; }
    fn discard_at(&mut self, _: DeviceOffsetInBytes, _: u64) -> Result<DiscardOutcome, BlockDeviceError> { self.0.borrow_mut().discard += 1; Ok(DiscardOutcome::Applied) }
    fn write_at(&mut self, _: DeviceOffsetInBytes, _: &[u8], _: WriteDurability) -> Result<(), BlockDeviceError> { Ok(()) }
    fn write_zeroes_at(&mut self, _: DeviceOffsetInBytes, _: u64) -> Result<(), BlockDeviceError> { Ok(()) }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> { Ok(()) }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes { PhysicalBlockSizeInBytes(512) }
    fn size_in_bytes(&self) -> u64 { 1 << 30 }
}
fn geometry() -> FixedGeometry { FixedGeometry { fixed_structure_slot_spacing: 4096, journal_ring_bytes: 768 << 20, root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM } }
fn logger(inner: Routes) -> ReadLoggingBlockDevice<Routes> {
    ReadLoggingBlockDevice::new(inner, JournalRingRegion::starting_at_the_standard_slot(768 << 20), JournalHintAnswer::PassTheInnerDeviceAnswerThrough)
}
fn exercise(d: &dyn BlockDevice) -> (u8, u8) {
    let mut b = [0; 512];
    d.reread_at_after_a_bad_read(DeviceOffsetInBytes(0), &mut b).unwrap();
    let reread = b[0];
    d.read_at_straight_from_the_disk(DeviceOffsetInBytes(0), &mut b).unwrap();
    d.keep_the_self_verified_reread(DeviceOffsetInBytes(0), &b);
    d.note_the_read_failing_self_verification_and_forget_its_bytes(DeviceOffsetInBytes(0), 512);
    (reread, b[0])
}

#[test]
fn audit_logger_changes_supported_discard_to_unsupported() {
    let calls = Rc::new(RefCell::new(Calls::default()));
    let mut direct = Routes(calls.clone());
    assert_eq!(direct.discard_at(DeviceOffsetInBytes(0), 512).unwrap(), DiscardOutcome::Applied);
    let mut wrapped = logger(Routes(calls.clone()));
    let outcome = wrapped.discard_at(DeviceOffsetInBytes(0), 512).unwrap();
    assert_eq!(outcome, if control() { DiscardOutcome::Applied } else { DiscardOutcome::Unsupported });
    assert_eq!(calls.borrow().discard, if control() { 2 } else { 1 });
    println!("H01 control={} direct=Applied logged={outcome:?} inner_discard_calls={}", control(), calls.borrow().discard);
    let sparse_with_data = || {
        let mut device = singlefs_harness::memory_pool::SparseBlockDevice::new(4096, PhysicalBlockSizeInBytes(512));
        device.write_at(DeviceOffsetInBytes(0), &[0x5a; 512], WriteDurability::Plain).unwrap();
        device
    };
    let mut direct = sparse_with_data();
    assert_eq!(direct.discard_at(DeviceOffsetInBytes(0), 512).unwrap(), DiscardOutcome::Applied);
    let mut b = [0; 512];
    direct.read_at(DeviceOffsetInBytes(0), &mut b).unwrap();
    assert_eq!(b, [0; 512]);
    let mut logged = ReadLoggingBlockDevice::new(sparse_with_data(), JournalRingRegion::starting_at_the_standard_slot(768 << 20), JournalHintAnswer::PassTheInnerDeviceAnswerThrough);
    assert_eq!(logged.discard_at(DeviceOffsetInBytes(0), 512).unwrap(), if control() { DiscardOutcome::Applied } else { DiscardOutcome::Unsupported });
    logged.read_at(DeviceOffsetInBytes(0), &mut b).unwrap();
    assert_eq!(b, [if control() { 0 } else { 0x5a }; 512]);
    println!("H01 sparse_direct_after_discard=0x00 sparse_logged_after_discard=0x{:02x}", b[0]);
}

#[test]
fn audit_all_three_wrappers_lose_four_read_route_hooks() {
    let calls = Rc::new(RefCell::new(Calls::default()));
    assert_eq!(exercise(&Routes(calls.clone())), (0x22, 0x33));
    assert_eq!(calls.borrow().reread, 1);
    assert_eq!(calls.borrow().straight, 1);
    assert_eq!(calls.borrow().keep, 1);
    assert_eq!(calls.borrow().forget, 1);
    *calls.borrow_mut() = Calls::default();
    let wrappers: Vec<Box<dyn BlockDevice>> = vec![
        Box::new(logger(Routes(calls.clone()))),
        Box::new(RecordingBlockDevice::with_shared_stream(DeviceIdentity(0), Routes(calls.clone()), SharedStream::retaining_contents())),
        Box::new(FaultInjectingBlockDevice::new(DeviceIdentity(0), Routes(calls.clone()), SharedFaultPlan::unarmed(geometry()))),
    ];
    for (name, wrapper) in ["logging", "recording", "fault-unarmed"].iter().zip(&wrappers) {
        let result = exercise(wrapper.as_ref());
        assert_eq!(result, if control() { (0x22, 0x33) } else { (0x11, 0x11) });
        println!("H02 {name} control={} reread=0x{:02x} straight=0x{:02x}", control(), result.0, result.1);
    }
    assert_eq!(*calls.borrow(), if control() { Calls { reread: 3, straight: 3, keep: 3, forget: 3, ..Calls::default() } } else { Calls { ordinary: 6, ..Calls::default() } });
}

#[test]
fn audit_cold_start_accepts_publishes_without_any_readback() {
    let fresh = || IdealModel::after_make_filesystem(ModelPoolGeometry { devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)], device_size_in_bytes: 4 << 30 });
    let mut model = fresh();
    let answer = model.answer_cold_start_recover();
    let valid_root = model.committed_versions()[0].0;
    assert!(model.judge_and_advance(&answer, &ObservedOutcome::Succeeded(ObservedEffect::ColdStart { read_back: ObservedReadBack::NoFile { root: valid_root } })).is_ok());
    let mut model = fresh();
    assert!(model.judge_and_advance(&answer, &ObservedOutcome::Succeeded(ObservedEffect::ColdStart { read_back: ObservedReadBack::Failed { what: "audit failure".into() } })).is_err());
    let mut model = fresh();
    let result = model.judge_and_advance(&answer, &ObservedOutcome::Succeeded(ObservedEffect::Publishes { roots: vec![], reported_ceiling: None }));
    if control() { assert!(result.is_err()); } else {
        let counts = result.as_ref().unwrap();
        assert_eq!(counts.successes_matched, 1);
        assert_eq!(counts.cold_start_contents_compared, 0);
    }
    println!("H03 control={} actual_failed_readback=Err wrong_effect_empty_publishes={}", control(), if result.is_ok() { "Ok" } else { "Err" });
}
