use tracing::error;

pub trait IdGenerator {
    type Id;

    fn id_now(&self) -> Self::Id;
}

// UUIDv7 ---------------------------------------------------------------------

pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    type Id = uuid::Uuid;

    fn id_now(&self) -> Self::Id {
        uuid::Uuid::now_v7()
    }
}

// Sonyflake ------------------------------------------------------------------

pub struct SonyflakeGenerator {
    inner: sonyflake::Sonyflake,
}

impl SonyflakeGenerator {
    /// Generates a [`SonyflakeGenerator`] using a hash of the machine's
    /// hostname as machine ID.
    pub fn from_hostname() -> anyhow::Result<Self> {
        let inner = sonyflake::Sonyflake::builder()
            .machine_id(&machine_id_from_hostname)
            .finalize()?;
        Ok(Self { inner })
    }
}

impl IdGenerator for SonyflakeGenerator {
    type Id = u64;

    fn id_now(&self) -> Self::Id {
        self.inner.next_id().unwrap_or_else(|error| {
            error!(?error, "sonyflake id error, falling back to 0");
            0
        })
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
    let folded = ((hash >> 32) ^ (hash & 0xFFFFFFFF)) as u32;
    let machine_id = (folded & 0xFFFF) as u16;

    Ok(machine_id)
}
