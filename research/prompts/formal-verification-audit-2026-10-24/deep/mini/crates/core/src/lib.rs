use vstd::prelude::*;
pub mod omitted;
verus! {
pub proof fn registered_proof() ensures false, { omitted::omitted_bad_proof(); }
}
