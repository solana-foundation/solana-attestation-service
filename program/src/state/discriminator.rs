pub trait Discriminator {
  const DISCRIMINATOR: u8;
}

#[repr(u8)]
pub enum AttestationAccountDiscriminators {
  CredentialDiscriminator = 0,
  SchemaDiscriminator = 1,
}

// TODO create traits for Serialization/Deserialization
