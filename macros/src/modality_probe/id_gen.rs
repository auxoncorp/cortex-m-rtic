use sha3::{Digest, Sha3_256};
use std::convert::TryInto;
use std::hash::Hash;
use std::num::NonZeroU32;
use syn::Ident;
use uuid::Uuid;

/// Generate a ProbeId based on the probe name
pub fn id_gen(probe_name: &Ident) -> u32 {
    let probe_id_range = NonZeroIdRange::new(
        NonZeroU32::new(1).unwrap(),
        NonZeroU32::new(modality_probe::ProbeId::MAX_ID).unwrap(),
    )
    .unwrap();
    let mut gen = IdGen::new(probe_id_range);
    gen.hashed_id(&probe_name.to_string()).get()
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct NonZeroIdRange {
    inclusive_start: NonZeroU32,
    inclusive_end: NonZeroU32,
}

impl NonZeroIdRange {
    fn new(inclusive_start: NonZeroU32, inclusive_end: NonZeroU32) -> Option<Self> {
        if inclusive_start.get() > inclusive_end.get() {
            None
        } else {
            Some(NonZeroIdRange {
                inclusive_start,
                inclusive_end,
            })
        }
    }

    fn contains(&self, value: NonZeroU32) -> bool {
        value.get() >= self.inclusive_start.get() && value.get() <= self.inclusive_end.get()
    }
}

#[derive(Debug)]
struct IdGen {
    id_range: NonZeroIdRange,
    uuid: Uuid,
}

impl IdGen {
    fn new(id_range: NonZeroIdRange) -> Self {
        IdGen {
            id_range,
            uuid: Uuid::new_v4(),
        }
    }

    fn regenerate_uuid(&mut self) {
        self.uuid = Uuid::new_v4();
    }

    fn hashed_id(&mut self, token: &str) -> NonZeroU32 {
        let mut max_tries = std::u16::MAX;
        loop {
            let hash = self.token_hash(token);
            if let Some(non_zero_hash) = NonZeroU32::new(hash) {
                if self.id_range.contains(non_zero_hash) {
                    return non_zero_hash;
                }
            }

            self.regenerate_uuid();

            // Escape hatch
            max_tries = max_tries.saturating_sub(1);
            if max_tries == 0 {
                panic!("Exceeded the id-hashing retry limit");
            }
        }
    }

    fn token_hash(&self, token: &str) -> u32 {
        let mut hasher = Sha3_256::new();
        hasher.update(self.uuid.as_bytes());
        hasher.update(token.as_bytes());
        let hash = hasher.finalize();
        let bytes: &[u8; 32] = hash.as_ref();
        let be16_bytes =
            u16::from_be_bytes(bytes[0..2].try_into().expect("Can't make a u16 from bytes"));
        let be32_bytes =
            u32::from_be_bytes(bytes[0..4].try_into().expect("Can't make a u32 from bytes"));
        let id = u32::from(be16_bytes)
            .overflowing_mul(0xFFFF_FFFF)
            .0
            .overflowing_add(be32_bytes)
            .0;
        id % self.id_range.inclusive_end.get()
    }
}
