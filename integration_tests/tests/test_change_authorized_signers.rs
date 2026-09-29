use borsh::BorshDeserialize;
use helpers::{create_credential, program_test_context, send};
use solana_attestation_service_client::{accounts::Credential, instructions::ChangeAuthorizedSignersBuilder};
use solana_keypair::Keypair;
use solana_signer::Signer;

mod helpers;

#[test]
fn change_authorized_signers_success() {
    let mut ctx = program_test_context();
    let authority = Keypair::new();
    let name = "test";
    let signers = vec![authority.pubkey(), ctx.payer.pubkey()];
    let credential_pda = create_credential(&mut ctx, &authority, name, signers);

    let downsized = vec![Keypair::new().pubkey()];
    let upsized = vec![Keypair::new().pubkey(), Keypair::new().pubkey(), Keypair::new().pubkey()];
    let same_size = vec![Keypair::new().pubkey(), Keypair::new().pubkey()];
    for new_signers in [downsized, upsized, same_size] {
        let ix = ChangeAuthorizedSignersBuilder::new()
            .payer(ctx.payer.pubkey())
            .authority(authority.pubkey())
            .credential(credential_pda)
            .signers(new_signers.clone())
            .instruction();
        send(&mut ctx, &[ix], &[&authority]).unwrap();

        let credential_account = ctx.svm.get_account(&credential_pda).expect("account not none");
        let credential = Credential::try_from_slice(&credential_account.data).unwrap();
        assert_eq!(credential.authority, authority.pubkey());
        assert_eq!(credential.name, name.as_bytes());
        assert_eq!(credential.authorized_signers, new_signers);
    }
}
