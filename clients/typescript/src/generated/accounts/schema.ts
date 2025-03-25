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
  assertAccountExists,
  assertAccountsExist,
  combineCodec,
  decodeAccount,
  fetchEncodedAccount,
  fetchEncodedAccounts,
  getAddressDecoder,
  getAddressEncoder,
  getBooleanDecoder,
  getBooleanEncoder,
  getBytesDecoder,
  getBytesEncoder,
  getStructDecoder,
  getStructEncoder,
  getU32Decoder,
  getU32Encoder,
  getU8Decoder,
  getU8Encoder,
  type Account,
  type Address,
  type Codec,
  type Decoder,
  type EncodedAccount,
  type Encoder,
  type FetchAccountConfig,
  type FetchAccountsConfig,
  type MaybeAccount,
  type MaybeEncodedAccount,
  type ReadonlyUint8Array,
} from '@solana/kit';

export type Schema = {
  discriminator: number;
  credential: Address;
  name: ReadonlyUint8Array;
  description: ReadonlyUint8Array;
  layout: ReadonlyUint8Array;
  fieldNames: ReadonlyUint8Array;
  isPaused: boolean;
  version: number;
};

export type SchemaArgs = Schema;

export function getSchemaEncoder(): Encoder<SchemaArgs> {
  return getStructEncoder([
    ['discriminator', getU8Encoder()],
    ['credential', getAddressEncoder()],
    ['name', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
    ['description', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
    ['layout', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
    ['fieldNames', addEncoderSizePrefix(getBytesEncoder(), getU32Encoder())],
    ['isPaused', getBooleanEncoder()],
    ['version', getU8Encoder()],
  ]);
}

export function getSchemaDecoder(): Decoder<Schema> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['credential', getAddressDecoder()],
    ['name', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    ['description', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    ['layout', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    ['fieldNames', addDecoderSizePrefix(getBytesDecoder(), getU32Decoder())],
    ['isPaused', getBooleanDecoder()],
    ['version', getU8Decoder()],
  ]);
}

export function getSchemaCodec(): Codec<SchemaArgs, Schema> {
  return combineCodec(getSchemaEncoder(), getSchemaDecoder());
}

export function decodeSchema<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<Schema, TAddress>;
export function decodeSchema<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<Schema, TAddress>;
export function decodeSchema<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<Schema, TAddress> | MaybeAccount<Schema, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getSchemaDecoder()
  );
}

export async function fetchSchema<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<Schema, TAddress>> {
  const maybeAccount = await fetchMaybeSchema(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeSchema<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<Schema, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeSchema(maybeAccount);
}

export async function fetchAllSchema(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<Schema>[]> {
  const maybeAccounts = await fetchAllMaybeSchema(rpc, addresses, config);
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeSchema(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<Schema>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) => decodeSchema(maybeAccount));
}
