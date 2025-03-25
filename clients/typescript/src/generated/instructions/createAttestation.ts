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
  getU32Decoder,
  getU32Encoder,
  getU8Decoder,
  getU8Encoder,
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

export const CREATE_ATTESTATION_DISCRIMINATOR = 6;

export function getCreateAttestationDiscriminatorBytes() {
  return getU8Encoder().encode(CREATE_ATTESTATION_DISCRIMINATOR);
}

export type CreateAttestationInstruction<
  TProgram extends string = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  TAccountPayer extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TAccountCredential extends string | IAccountMeta<string> = string,
  TAccountSchema extends string | IAccountMeta<string> = string,
  TAccountAttestation extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
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
      ...TRemainingAccounts,
    ]
  >;

export type CreateAttestationInstructionData = {
  discriminator: number;
  nonce: Address;
  data: ReadonlyUint8Array;
  expiry: bigint;
};

export type CreateAttestationInstructionDataArgs = {
  nonce: Address;
  data: ReadonlyUint8Array;
  expiry: number | bigint;
};

export function getCreateAttestationInstructionDataEncoder(): Encoder<CreateAttestationInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['nonce', getAddressEncoder()],
      ['data', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
      ['expiry', getI64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: CREATE_ATTESTATION_DISCRIMINATOR })
  );
}

export function getCreateAttestationInstructionDataDecoder(): Decoder<CreateAttestationInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['nonce', getAddressDecoder()],
    ['data', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    ['expiry', getI64Decoder()],
  ]);
}

export function getCreateAttestationInstructionDataCodec(): Codec<
  CreateAttestationInstructionDataArgs,
  CreateAttestationInstructionData
> {
  return combineCodec(
    getCreateAttestationInstructionDataEncoder(),
    getCreateAttestationInstructionDataDecoder()
  );
}

export type CreateAttestationInput<
  TAccountPayer extends string = string,
  TAccountAuthority extends string = string,
  TAccountCredential extends string = string,
  TAccountSchema extends string = string,
  TAccountAttestation extends string = string,
  TAccountSystemProgram extends string = string,
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
  nonce: CreateAttestationInstructionDataArgs['nonce'];
  data: CreateAttestationInstructionDataArgs['data'];
  expiry: CreateAttestationInstructionDataArgs['expiry'];
};

export function getCreateAttestationInstruction<
  TAccountPayer extends string,
  TAccountAuthority extends string,
  TAccountCredential extends string,
  TAccountSchema extends string,
  TAccountAttestation extends string,
  TAccountSystemProgram extends string,
  TProgramAddress extends
    Address = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
>(
  input: CreateAttestationInput<
    TAccountPayer,
    TAccountAuthority,
    TAccountCredential,
    TAccountSchema,
    TAccountAttestation,
    TAccountSystemProgram
  >,
  config?: { programAddress?: TProgramAddress }
): CreateAttestationInstruction<
  TProgramAddress,
  TAccountPayer,
  TAccountAuthority,
  TAccountCredential,
  TAccountSchema,
  TAccountAttestation,
  TAccountSystemProgram
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

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.payer),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.credential),
      getAccountMeta(accounts.schema),
      getAccountMeta(accounts.attestation),
      getAccountMeta(accounts.systemProgram),
    ],
    programAddress,
    data: getCreateAttestationInstructionDataEncoder().encode(
      args as CreateAttestationInstructionDataArgs
    ),
  } as CreateAttestationInstruction<
    TProgramAddress,
    TAccountPayer,
    TAccountAuthority,
    TAccountCredential,
    TAccountSchema,
    TAccountAttestation,
    TAccountSystemProgram
  >;

  return instruction;
}

export type ParsedCreateAttestationInstruction<
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
  };
  data: CreateAttestationInstructionData;
};

export function parseCreateAttestationInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCreateAttestationInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 6) {
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
    },
    data: getCreateAttestationInstructionDataDecoder().decode(instruction.data),
  };
}
