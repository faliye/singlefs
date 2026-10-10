import sys
f,name=sys.argv[1],sys.argv[2]
s=open(f).read()
M={
 'M1-release-keeps-kept-leases':("""    fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {
        self.return_the_leases_kept_after_a_failed_wait_on(device);
""","""    fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {
"""),
 'M2-release-keeps-device-in-use':("""        if self.device_in_use == Some(device) {
            self.device_in_use = None;
        }
""",""),
 'M3-device-never-released':("""        self.ledger.books_locked()[self.card_position].release_a_device(self.device);""","""        let _ = self.device;"""),
 'M5-never-resume-unwind':("""        if !std::thread::panicking() {
            std::panic::resume_unwind(panic);""","""        if false {
            std::panic::resume_unwind(panic);"""),
 'M6-poll-exit-returns-every-card':("""    fn return_the_leases_kept_after_a_failed_wait_on(&mut self, device: DeviceNumberOnTheCard) {
        let leases = &mut self.leases;
        self.leases_kept_after_a_failed_wait
            .retain(|kept_lease, kept_on_the_device| {
                if *kept_on_the_device == device {""","""    fn return_the_leases_kept_after_a_failed_wait_on(&mut self, device: DeviceNumberOnTheCard) {
        let leases = &mut self.leases;
        self.leases_kept_after_a_failed_wait
            .retain(|kept_lease, kept_on_the_device| {
                if *kept_on_the_device == device || kept_on_the_device.0 + 1 == device.0 {"""),
}
a,b=M[name]
assert s.count(a)==1,(name,s.count(a))
open(f,'w').write(s.replace(a,b))
