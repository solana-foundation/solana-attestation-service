/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  addDecoderSizePrefix,
  addEncoderSizePrefix,
  combineCodec,
  getAddressDecoder,
  getAddressEncoder,
  getBytesDecoder,
  getBytesEncoder,
  getI64Decoder,
  getI64Encoder,
  getStructDecoder,
  getStructEncoder,
  getU16Decoder,
  getU16Encoder,
  getU32Decoder,
  getU32Encoder,
  getU8Decoder,
  getU8Encoder,
  getUtf8Decoder,
  getUtf8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type ReadonlyUint8Array,
  type TransactionSigner,
  type WritableAccount,
  type WritableSignerAccount,
} from '@solana/kit';
import { SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const CREATE_TOKENIZED_ATTESTATION_DISCRIMINATOR = 10;

export function getCreateTokenizedAttestationDiscriminatorBytes() {
  return getU8Encoder().encode(CREATE_TOKENIZED_ATTESTATION_DISCRIMINATOR);
}

export type CreateTokenizedAttestationInstruction<
  TProgram extends string = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  TAccountPayer extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TAccountCredential extends string | IAccountMeta<string> = string,
  TAccountSchema extends string | IAccountMeta<string> = string,
  TAccountAttestation extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TAccountSchemaMint extends string | IAccountMeta<string> = string,
  TAccountAttestationMint extends string | IAccountMeta<string> = string,
  TAccountSasPda extends string | IAccountMeta<string> = string,
  TAccountRecipientTokenAccount extends string | IAccountMeta<string> = string,
  TAccountRecipient extends string | IAccountMeta<string> = string,
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  TAccountAssociatedTokenProgram extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountPayer extends string
        ? WritableSignerAccount<TAccountPayer> &
            IAccountSignerMeta<TAccountPayer>
        : TAccountPayer,
      TAccountAuthority extends string
        ? ReadonlySignerAccount<TAccountAuthority> &
            IAccountSignerMeta<TAccountAuthority>
        : TAccountAuthority,
      TAccountCredential extends string
        ? ReadonlyAccount<TAccountCredential>
        : TAccountCredential,
      TAccountSchema extends string
        ? ReadonlyAccount<TAccountSchema>
        : TAccountSchema,
      TAccountAttestation extends string
        ? WritableAccount<TAccountAttestation>
        : TAccountAttestation,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      TAccountSchemaMint extends string
        ? WritableAccount<TAccountSchemaMint>
        : TAccountSchemaMint,
      TAccountAttestationMint extends string
        ? WritableAccount<TAccountAttestationMint>
        : TAccountAttestationMint,
      TAccountSasPda extends string
        ? ReadonlyAccount<TAccountSasPda>
        : TAccountSasPda,
      TAccountRecipientTokenAccount extends string
        ? WritableAccount<TAccountRecipientTokenAccount>
        : TAccountRecipientTokenAccount,
      TAccountRecipient extends string
        ? ReadonlyAccount<TAccountRecipient>
        : TAccountRecipient,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      TAccountAssociatedTokenProgram extends string
        ? ReadonlyAccount<TAccountAssociatedTokenProgram>
        : TAccountAssociatedTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type CreateTokenizedAttestationInstructionData = {
  discriminator: number;
  nonce: Address;
  data: ReadonlyUint8Array;
  expiry: bigint;
  name: string;
  uri: string;
  symbol: string;
  mintAccountSpace: number;
};

export type CreateTokenizedAttestationInstructionDataArgs = {
  nonce: Address;
  data: ReadonlyUint8Array;
  expiry: number | bigint;
  name: string;
  uri: string;
  symbol: string;
  mintAccountSpace: number;
};

export function getCreateTokenizedAttestationInstructionDataEncoder(): Encoder<CreateTokenizedAttestationInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['nonce', getAddressEncoder()],
      ['data', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
      ['expiry', getI64Encoder()],
      ['name', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
      ['uri', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
      ['symbol', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
      ['mintAccountSpace', getU16Encoder()],
    ]),
    (value) => ({
      ...value,
      discriminator: CREATE_TOKENIZED_ATTESTATION_DISCRIMINATOR,
    })
  );
}

export function getCreateTokenizedAttestationInstructionDataDecoder(): Decoder<CreateTokenizedAttestationInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['nonce', getAddressDecoder()],
    ['data', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    ['expiry', getI64Decoder()],
    ['name', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['uri', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['symbol', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['mintAccountSpace', getU16Decoder()],
  ]);
}

export function getCreateTokenizedAttestationInstructionDataCodec(): Codec<
  CreateTokenizedAttestationInstructionDataArgs,
  CreateTokenizedAttestationInstructionData
> {
  return combineCodec(
    getCreateTokenizedAttestationInstructionDataEncoder(),
    getCreateTokenizedAttestationInstructionDataDecoder()
  );
}

export type CreateTokenizedAttestationInput<
  TAccountPayer extends string = string,
  TAccountAuthority extends string = string,
  TAccountCredential extends string = string,
  TAccountSchema extends string = string,
  TAccountAttestation extends string = string,
  TAccountSystemProgram extends string = string,
  TAccountSchemaMint extends string = string,
  TAccountAttestationMint extends string = string,
  TAccountSasPda extends string = string,
  TAccountRecipientTokenAccount extends string = string,
  TAccountRecipient extends string = string,
  TAccountTokenProgram extends string = string,
  TAccountAssociatedTokenProgram extends string = string,
> = {
  payer: TransactionSigner<TAccountPayer>;
  /** Authorized signer of the Schema's Credential */
  authority: TransactionSigner<TAccountAuthority>;
  /** Credential the Schema is associated with */
  credential: Address<TAccountCredential>;
  /** Schema the Attestation is associated with */
  schema: Address<TAccountSchema>;
  attestation: Address<TAccountAttestation>;
  systemProgram?: Address<TAccountSystemProgram>;
  /** Mint of Schema Token */
  schemaMint: Address<TAccountSchemaMint>;
  /** Mint of Attestation Token */
  attestationMint: Address<TAccountAttestationMint>;
  /** Program derived address used as program signer authority */
  sasPda: Address<TAccountSasPda>;
  /** Associated token account of Recipient for Attestation Token */
  recipientTokenAccount: Address<TAccountRecipientTokenAccount>;
  /** Wallet to receive Attestation Token */
  recipient: Address<TAccountRecipient>;
  tokenProgram?: Address<TAccountTokenProgram>;
  associatedTokenProgram: Address<TAccountAssociatedTokenProgram>;
  nonce: CreateTokenizedAttestationInstructionDataArgs['nonce'];
  data: CreateTokenizedAttestationInstructionDataArgs['data'];
  expiry: CreateTokenizedAttestationInstructionDataArgs['expiry'];
  name: CreateTokenizedAttestationInstructionDataArgs['name'];
  uri: CreateTokenizedAttestationInstructionDataArgs['uri'];
  symbol: CreateTokenizedAttestationInstructionDataArgs['symbol'];
  mintAccountSpace: CreateTokenizedAttestationInstructionDataArgs['mintAccountSpace'];
};

export function getCreateTokenizedAttestationInstruction<
  TAccountPayer extends string,
  TAccountAuthority extends string,
  TAccountCredential extends string,
  TAccountSchema extends string,
  TAccountAttestation extends string,
  TAccountSystemProgram extends string,
  TAccountSchemaMint extends string,
  TAccountAttestationMint extends string,
  TAccountSasPda extends string,
  TAccountRecipientTokenAccount extends string,
  TAccountRecipient extends string,
  TAccountTokenProgram extends string,
  TAccountAssociatedTokenProgram extends string,
  TProgramAddress extends
    Address = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
>(
  input: CreateTokenizedAttestationInput<
    TAccountPayer,
    TAccountAuthority,
    TAccountCredential,
    TAccountSchema,
    TAccountAttestation,
    TAccountSystemProgram,
    TAccountSchemaMint,
    TAccountAttestationMint,
    TAccountSasPda,
    TAccountRecipientTokenAccount,
    TAccountRecipient,
    TAccountTokenProgram,
    TAccountAssociatedTokenProgram
  >,
  config?: { programAddress?: TProgramAddress }
): CreateTokenizedAttestationInstruction<
  TProgramAddress,
  TAccountPayer,
  TAccountAuthority,
  TAccountCredential,
  TAccountSchema,
  TAccountAttestation,
  TAccountSystemProgram,
  TAccountSchemaMint,
  TAccountAttestationMint,
  TAccountSasPda,
  TAccountRecipientTokenAccount,
  TAccountRecipient,
  TAccountTokenProgram,
  TAccountAssociatedTokenProgram
> {
  // Program address.
  const programAddress =
    config?.programAddress ?? SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    payer: { value: input.payer ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false },
    credential: { value: input.credential ?? null, isWritable: false },
    schema: { value: input.schema ?? null, isWritable: false },
    attestation: { value: input.attestation ?? null, isWritable: true },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
    schemaMint: { value: input.schemaMint ?? null, isWritable: true },
    attestationMint: { value: input.attestationMint ?? null, isWritable: true },
    sasPda: { value: input.sasPda ?? null, isWritable: false },
    recipientTokenAccount: {
      value: input.recipientTokenAccount ?? null,
      isWritable: true,
    },
    recipient: { value: input.recipient ?? null, isWritable: false },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
    associatedTokenProgram: {
      value: input.associatedTokenProgram ?? null,
      isWritable: false,
    },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }
  if (!accounts.tokenProgram.value) {
    accounts.tokenProgram.value =
      'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' as Address<'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.payer),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.credential),
      getAccountMeta(accounts.schema),
      getAccountMeta(accounts.attestation),
      getAccountMeta(accounts.systemProgram),
      getAccountMeta(accounts.schemaMint),
      getAccountMeta(accounts.attestationMint),
      getAccountMeta(accounts.sasPda),
      getAccountMeta(accounts.recipientTokenAccount),
      getAccountMeta(accounts.recipient),
      getAccountMeta(accounts.tokenProgram),
      getAccountMeta(accounts.associatedTokenProgram),
    ],
    programAddress,
    data: getCreateTokenizedAttestationInstructionDataEncoder().encode(
      args as CreateTokenizedAttestationInstructionDataArgs
    ),
  } as CreateTokenizedAttestationInstruction<
    TProgramAddress,
    TAccountPayer,
    TAccountAuthority,
    TAccountCredential,
    TAccountSchema,
    TAccountAttestation,
    TAccountSystemProgram,
    TAccountSchemaMint,
    TAccountAttestationMint,
    TAccountSasPda,
    TAccountRecipientTokenAccount,
    TAccountRecipient,
    TAccountTokenProgram,
    TAccountAssociatedTokenProgram
  >;

  return instruction;
}

export type ParsedCreateTokenizedAttestationInstruction<
  TProgram extends string = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    payer: TAccountMetas[0];
    /** Authorized signer of the Schema's Credential */
    authority: TAccountMetas[1];
    /** Credential the Schema is associated with */
    credential: TAccountMetas[2];
    /** Schema the Attestation is associated with */
    schema: TAccountMetas[3];
    attestation: TAccountMetas[4];
    systemProgram: TAccountMetas[5];
    /** Mint of Schema Token */
    schemaMint: TAccountMetas[6];
    /** Mint of Attestation Token */
    attestationMint: TAccountMetas[7];
    /** Program derived address used as program signer authority */
    sasPda: TAccountMetas[8];
    /** Associated token account of Recipient for Attestation Token */
    recipientTokenAccount: TAccountMetas[9];
    /** Wallet to receive Attestation Token */
    recipient: TAccountMetas[10];
    tokenProgram: TAccountMetas[11];
    associatedTokenProgram: TAccountMetas[12];
  };
  data: CreateTokenizedAttestationInstructionData;
};

export function parseCreateTokenizedAttestationInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCreateTokenizedAttestationInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 13) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      payer: getNextAccount(),
      authority: getNextAccount(),
      credential: getNextAccount(),
      schema: getNextAccount(),
      attestation: getNextAccount(),
      systemProgram: getNextAccount(),
      schemaMint: getNextAccount(),
      attestationMint: getNextAccount(),
      sasPda: getNextAccount(),
      recipientTokenAccount: getNextAccount(),
      recipient: getNextAccount(),
      tokenProgram: getNextAccount(),
      associatedTokenProgram: getNextAccount(),
    },
    data: getCreateTokenizedAttestationInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
