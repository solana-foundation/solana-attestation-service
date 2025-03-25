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
  getArrayDecoder,
  getArrayEncoder,
  getBytesDecoder,
  getBytesEncoder,
  getStructDecoder,
  getStructEncoder,
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

export const CREATE_SCHEMA_DISCRIMINATOR = 1;

export function getCreateSchemaDiscriminatorBytes() {
  return getU8Encoder().encode(CREATE_SCHEMA_DISCRIMINATOR);
}

export type CreateSchemaInstruction<
  TProgram extends string = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  TAccountPayer extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TAccountCredential extends string | IAccountMeta<string> = string,
  TAccountSchema extends string | IAccountMeta<string> = string,
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
        ? WritableAccount<TAccountSchema>
        : TAccountSchema,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      ...TRemainingAccounts,
    ]
  >;

export type CreateSchemaInstructionData = {
  discriminator: number;
  name: string;
  description: string;
  layout: ReadonlyUint8Array;
  fieldNames: Array<string>;
};

export type CreateSchemaInstructionDataArgs = {
  name: string;
  description: string;
  layout: ReadonlyUint8Array;
  fieldNames: Array<string>;
};

export function getCreateSchemaInstructionDataEncoder(): Encoder<CreateSchemaInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['name', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
      ['description', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
      ['layout', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
      [
        'fieldNames',
        getArrayEncoder(
          addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())
        ),
      ],
    ]),
    (value) => ({ ...value, discriminator: CREATE_SCHEMA_DISCRIMINATOR })
  );
}

export function getCreateSchemaInstructionDataDecoder(): Decoder<CreateSchemaInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['name', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['description', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['layout', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    [
      'fieldNames',
      getArrayDecoder(addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())),
    ],
  ]);
}

export function getCreateSchemaInstructionDataCodec(): Codec<
  CreateSchemaInstructionDataArgs,
  CreateSchemaInstructionData
> {
  return combineCodec(
    getCreateSchemaInstructionDataEncoder(),
    getCreateSchemaInstructionDataDecoder()
  );
}

export type CreateSchemaInput<
  TAccountPayer extends string = string,
  TAccountAuthority extends string = string,
  TAccountCredential extends string = string,
  TAccountSchema extends string = string,
  TAccountSystemProgram extends string = string,
> = {
  payer: TransactionSigner<TAccountPayer>;
  authority: TransactionSigner<TAccountAuthority>;
  /** Credential the Schema is associated with */
  credential: Address<TAccountCredential>;
  schema: Address<TAccountSchema>;
  systemProgram?: Address<TAccountSystemProgram>;
  name: CreateSchemaInstructionDataArgs['name'];
  description: CreateSchemaInstructionDataArgs['description'];
  layout: CreateSchemaInstructionDataArgs['layout'];
  fieldNames: CreateSchemaInstructionDataArgs['fieldNames'];
};

export function getCreateSchemaInstruction<
  TAccountPayer extends string,
  TAccountAuthority extends string,
  TAccountCredential extends string,
  TAccountSchema extends string,
  TAccountSystemProgram extends string,
  TProgramAddress extends
    Address = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
>(
  input: CreateSchemaInput<
    TAccountPayer,
    TAccountAuthority,
    TAccountCredential,
    TAccountSchema,
    TAccountSystemProgram
  >,
  config?: { programAddress?: TProgramAddress }
): CreateSchemaInstruction<
  TProgramAddress,
  TAccountPayer,
  TAccountAuthority,
  TAccountCredential,
  TAccountSchema,
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
    schema: { value: input.schema ?? null, isWritable: true },
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
      getAccountMeta(accounts.systemProgram),
    ],
    programAddress,
    data: getCreateSchemaInstructionDataEncoder().encode(
      args as CreateSchemaInstructionDataArgs
    ),
  } as CreateSchemaInstruction<
    TProgramAddress,
    TAccountPayer,
    TAccountAuthority,
    TAccountCredential,
    TAccountSchema,
    TAccountSystemProgram
  >;

  return instruction;
}

export type ParsedCreateSchemaInstruction<
  TProgram extends string = typeof SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    payer: TAccountMetas[0];
    authority: TAccountMetas[1];
    /** Credential the Schema is associated with */
    credential: TAccountMetas[2];
    schema: TAccountMetas[3];
    systemProgram: TAccountMetas[4];
  };
  data: CreateSchemaInstructionData;
};

export function parseCreateSchemaInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCreateSchemaInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 5) {
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
      systemProgram: getNextAccount(),
    },
    data: getCreateSchemaInstructionDataDecoder().decode(instruction.data),
  };
}
