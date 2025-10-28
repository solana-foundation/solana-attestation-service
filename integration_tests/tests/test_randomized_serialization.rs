use borsh::BorshDeserialize;
use solana_attestation_service::{
    events::{
        CloseAttestationEvent as ProgramCloseAttestationEvent,
        CompressAttestation as ProgramCompressAttestation,
        CompressAttestationEvent as ProgramCompressAttestationEvent,
    },
    processor::{
        close_compressed_attestation::CloseCompressedAttestationArgs,
        compress_attestations::CompressAttestationsArgs,
        create_compressed_attestation::CreateCompressedAttestationArgs,
    },
};
use solana_attestation_service_client::{
    instructions::{
        CloseCompressedAttestationInstructionArgs, CompressAttestationsInstructionArgs,
        CreateCompressedAttestationInstructionArgs,
    },
    types::{CloseAttestationEvent, CompressAttestationEvent},
};
use solana_pubkey::Pubkey;

#[test]
fn test_compress_attestation_event_serialization_roundtrip_randomized() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let attestations: Vec<ProgramCompressAttestation> = (0..rng.gen_range(1..=10))
            .map(|_| ProgramCompressAttestation {
                schema: Pubkey::new_unique().to_bytes(),
                attestation_data: (0..rng.gen_range(0..=500)).map(|_| rng.gen()).collect(),
            })
            .collect();

        let original = ProgramCompressAttestationEvent {
            discriminator: rng.gen(),
            pdas_closed: rng.gen_bool(0.5),
            attestations,
        };

        let serialized = original.to_bytes();
        let deserialized = CompressAttestationEvent::try_from_slice(&serialized[8..]).unwrap();

        assert_eq!(original.discriminator, deserialized.discriminator);
        assert_eq!(original.pdas_closed, deserialized.pdas_closed);
        assert_eq!(original.attestations.len(), deserialized.attestations.len());
        for (prog, client) in original
            .attestations
            .iter()
            .zip(deserialized.attestations.iter())
        {
            assert_eq!(prog.schema, client.schema.to_bytes());
            assert_eq!(prog.attestation_data, client.attestation_data);
        }
    }
}

#[test]
fn test_close_attestation_event_serialization_roundtrip_randomized() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let original = ProgramCloseAttestationEvent {
            discriminator: rng.gen(),
            schema: Pubkey::new_unique().to_bytes(),
            attestation_data: (0..rng.gen_range(0..=500)).map(|_| rng.gen()).collect(),
        };

        let serialized = original.to_bytes();
        let deserialized = CloseAttestationEvent::try_from_slice(&serialized[8..]).unwrap();

        assert_eq!(original.discriminator, deserialized.discriminator);
        assert_eq!(original.schema, deserialized.schema.to_bytes());
        assert_eq!(original.attestation_data, deserialized.attestation_data);
    }
}

#[test]
fn test_create_compressed_attestation_instruction_data_serialization_roundtrip() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let mut proof = [0u8; 128];
        rng.fill(&mut proof[..]);

        let original = CreateCompressedAttestationInstructionArgs {
            proof,
            nonce: Pubkey::new_unique(),
            expiry: rng.gen::<i64>(),
            address_root_index: rng.gen::<u16>(),
            data: (0..rng.gen_range(0..=500)).map(|_| rng.gen()).collect(),
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized =
            CreateCompressedAttestationArgs::process_instruction_data(&serialized).unwrap();

        assert_eq!(&original.proof[..32], &deserialized.proof.a);
        assert_eq!(&original.proof[32..96], &deserialized.proof.b);
        assert_eq!(&original.proof[96..128], &deserialized.proof.c);
        assert_eq!(original.nonce.to_bytes(), deserialized.nonce);
        assert_eq!(original.expiry, deserialized.expiry);
        assert_eq!(original.address_root_index, deserialized.address_root_index);
        assert_eq!(original.data, deserialized.data);
    }
}

#[test]
fn test_compress_attestations_instruction_data_serialization_roundtrip() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let mut proof = [0u8; 128];
        rng.fill(&mut proof[..]);

        let original = CompressAttestationsInstructionArgs {
            proof,
            close_accounts: rng.gen_bool(0.5),
            address_root_index: rng.gen::<u16>(),
            num_attestations: rng.gen_range(1..=255),
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized = CompressAttestationsArgs::process_instruction_data(&serialized).unwrap();

        assert_eq!(&original.proof[..32], &deserialized.proof.a);
        assert_eq!(&original.proof[32..96], &deserialized.proof.b);
        assert_eq!(&original.proof[96..128], &deserialized.proof.c);
        assert_eq!(original.close_accounts, deserialized.close_accounts);
        assert_eq!(original.address_root_index, deserialized.address_root_index);
        assert_eq!(original.num_attestations, deserialized.num_attestations);
    }
}

#[test]
fn test_close_compressed_attestation_instruction_data_serialization_roundtrip() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let has_proof = rng.gen_bool(0.5);
        let proof = if has_proof {
            let mut p = [0u8; 128];
            rng.fill(&mut p[..]);
            Some(p)
        } else {
            None
        };

        let original = CloseCompressedAttestationInstructionArgs {
            proof,
            root_index: rng.gen::<u16>(),
            leaf_index: rng.gen::<u32>(),
            address: rng.gen::<[u8; 32]>(),
            nonce: Pubkey::new_unique(),
            schema: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            expiry: rng.gen::<i64>(),
            data: (0..rng.gen_range(0..=500)).map(|_| rng.gen()).collect(),
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized =
            CloseCompressedAttestationArgs::process_instruction_data(&serialized).unwrap();

        // Verify proof
        if let Some(original_proof) = &original.proof {
            assert!(deserialized.proof.0.is_some());
            let deser_proof = deserialized.proof.0.unwrap();
            assert_eq!(&original_proof[..32], &deser_proof.a);
            assert_eq!(&original_proof[32..96], &deser_proof.b);
            assert_eq!(&original_proof[96..128], &deser_proof.c);
        } else {
            assert!(deserialized.proof.0.is_none());
        }

        assert_eq!(original.root_index, deserialized.root_index);
        assert_eq!(original.leaf_index, deserialized.leaf_index);
        assert_eq!(original.address, deserialized.compressed_address);
        assert_eq!(original.nonce.to_bytes(), deserialized.attestation.nonce);
        assert_eq!(original.schema.to_bytes(), deserialized.attestation.schema);
        assert_eq!(original.signer.to_bytes(), deserialized.attestation.signer);
        assert_eq!(original.expiry, deserialized.attestation.expiry);
        assert_eq!(original.data, deserialized.attestation.data);
    }
}
