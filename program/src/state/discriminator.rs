extern crate alloc;

use alloc::vec::Vec;
pub trait Discriminator {
    const DISCRIMINATOR: u8;
}

#[repr(u8)]
pub enum AttestationAccountDiscriminators {
    CredentialDiscriminator = 0,
    SchemaDiscriminator = 1,
}

pub trait AccountSerialize: Discriminator {
    /// Serialize the struct with the Discriminator prepended.
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Discriminator
        data.push(Self::DISCRIMINATOR);

        data.extend(self.to_bytes_inner());

        data
    }

    fn to_bytes_inner(&self) -> Vec<u8>;
}
