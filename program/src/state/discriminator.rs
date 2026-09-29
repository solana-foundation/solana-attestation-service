extern crate alloc;

use alloc::vec::Vec;
pub trait Discriminator {
    const DISCRIMINATOR: u8;
}

#[repr(u8)]
pub enum AttestationAccountDiscriminators {
    CredentialDiscriminator = 0,
    SchemaDiscriminator = 1,
    AttestationDiscriminator = 2,
}

pub trait AccountSerialize: Discriminator {
    fn to_bytes(&self) -> Vec<u8> {
        [&[Self::DISCRIMINATOR], self.to_bytes_inner().as_slice()].concat()
    }

    fn to_bytes_inner(&self) -> Vec<u8>;
}
