use std::fmt;

use enum_dispatch::enum_dispatch;
use tracing::error;

#[derive(Debug, Clone, Copy)]
pub struct Id {
    inner: IdInner,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = match self.inner {
            IdInner::Uuid(u) => u.as_bytes().to_vec(),
            IdInner::Sonyflake(s) => s.to_be_bytes().to_vec(),
        };
        let s = base32::encode(base32::Alphabet::Crockford, &bytes);
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
enum IdInner {
    Uuid(uuid::Uuid),
    Sonyflake(u64),
}

#[enum_dispatch]
pub trait IdGenerator {
    fn id_now(&self) -> Id;
}

#[enum_dispatch(IdGenerator)]
pub enum IdGeneratorImpl {
    UuidV7(UuidV7Generator),
    Sonyflake(SonyflakeGenerator),
}

// UUIDv7 ---------------------------------------------------------------------

pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn id_now(&self) -> Id {
        let inner = IdInner::Uuid(uuid::Uuid::now_v7());
        Id { inner }
    }
}

// Sonyflake ------------------------------------------------------------------

pub struct SonyflakeGenerator {
    sonyflake: sonyflake::Sonyflake,
}

impl SonyflakeGenerator {
    /// Generates a [`SonyflakeGenerator`] using a hash of the machine's
    /// hostname as machine ID.
    ///
    /// # Errors
    ///
    /// - If creating the ID fails
    pub fn from_hostname() -> anyhow::Result<Self> {
        let sonyflake = sonyflake::Sonyflake::builder()
            .machine_id(&machine_id_from_hostname)
            .finalize()?;
        Ok(Self { sonyflake })
    }
}

impl IdGenerator for SonyflakeGenerator {
    fn id_now(&self) -> Id {
        let inner = IdInner::Sonyflake(self.sonyflake.next_id().unwrap_or_else(|error| {
            error!(?error, "sonyflake id error, falling back to 0");
            0
        }));
        Id { inner }
    }
}

fn machine_id_from_hostname() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    use std::hash::Hash;
    use std::hash::Hasher;

    let mut hasher = std::hash::DefaultHasher::new();
    whoami::fallible::hostname()?.hash(&mut hasher);
    let hash = hasher.finish();

    // Fold the 64-bit hash down to 16 bits by XORing the upper and lower parts.
    // This has a bit better distribution than truncation.
    let folded = u32::try_from((hash >> 32) ^ (hash & 0xFFFF_FFFF))?;
    let machine_id = (folded & 0xFFFF) as u16;

    Ok(machine_id)
}
