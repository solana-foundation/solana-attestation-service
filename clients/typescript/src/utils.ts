import {
  addCodecSizePrefix,
  getArrayCodec,
  getBooleanCodec,
  getI128Codec,
  getI16Codec,
  getI32Codec,
  getI64Codec,
  getI8Codec,
  getStructCodec,
  getU128Codec,
  getU16Codec,
  getU32Codec,
  getU64Codec,
  getU8Codec,
  getUtf8Codec,
  transformCodec,
  type Codec,
} from "@solana/kit";

import { Schema } from "./generated";

type SchemaOutputTypes =
  | number
  | number[]
  | string
  | string[]
  | bigint
  | bigint[]
  | boolean
  | boolean[];

type AttestationData = Record<string, SchemaOutputTypes>;

/**
 * Rust encodes a `char` as its 4-byte little-endian Unicode code point.
 */
const getCharCodec = (): Codec<string> =>
  transformCodec(
    getU32Codec(),
    (character: string) => {
      const codePoint = character.codePointAt(0);
      if (codePoint === undefined || String.fromCodePoint(codePoint) !== character) {
        throw new Error("Char fields must hold exactly one Unicode character");
      }
      return codePoint;
    },
    (codePoint) => String.fromCodePoint(codePoint)
  );

const getStringCodec = (): Codec<string> =>
  addCodecSizePrefix(getUtf8Codec(), getU32Codec());

/**
 * Maps the SAS compact byte layout to the equivalent data type. Values mirror
 * the type identifiers emitted by the `SchemaStructSerialize` derive macro.
 */
const compactLayoutMapping: Record<number, () => Codec<any>> = {
  0: getU8Codec,
  1: getU16Codec,
  2: getU32Codec,
  3: getU64Codec,
  4: getU128Codec,
  5: getI8Codec,
  6: getI16Codec,
  7: getI32Codec,
  8: getI64Codec,
  9: getI128Codec,
  10: getBooleanCodec,
  11: getCharCodec,
  12: getStringCodec,
  13: () => getArrayCodec(getU8Codec()),
  14: () => getArrayCodec(getU16Codec()),
  15: () => getArrayCodec(getU32Codec()),
  16: () => getArrayCodec(getU64Codec()),
  17: () => getArrayCodec(getU128Codec()),
  18: () => getArrayCodec(getI8Codec()),
  19: () => getArrayCodec(getI16Codec()),
  20: () => getArrayCodec(getI32Codec()),
  21: () => getArrayCodec(getI64Codec()),
  22: () => getArrayCodec(getI128Codec()),
  23: () => getArrayCodec(getBooleanCodec()),
  24: () => getArrayCodec(getCharCodec()),
  25: () => getArrayCodec(getStringCodec()),
};

/**
 * Given the onchain representation of a Schema, build a codec that
 * (de)serializes Attestation data conforming to that Schema.
 * @param schema
 * @returns
 */
export const getAttestationDataCodec = (
  schema: Schema
): Codec<AttestationData> => {
  const textDecoder = new TextDecoder();
  const fields = splitJoinedVecs(Uint8Array.from(schema.fieldNames)).map((f) =>
    textDecoder.decode(Uint8Array.from(f))
  );

  if (fields.length !== schema.layout.length) {
    throw new Error("Schema field names and layout do not match");
  }

  return getStructCodec(
    fields.map((field, index) => {
      const layoutByte = schema.layout[index];
      const getFieldCodec = compactLayoutMapping[layoutByte];
      if (!getFieldCodec) {
        throw new Error("Invalid Schema layout value");
      }
      return [field, getFieldCodec()] as const;
    })
  ) as Codec<AttestationData>;
};

/**
 * Given a SAS Schema and an object that represents the Attestation data,
 * serialize the Attestation data to valid byte array.
 * @param schema
 */
export const serializeAttestationData = (
  schema: Schema,
  data: Record<string, unknown>
): Uint8Array =>
  new Uint8Array(
    getAttestationDataCodec(schema).encode(data as AttestationData)
  );

/**
 * Given a SAS Schema and a byte array of Attestation data,
 * deserialize the Attestation data to an object.
 * @param schema
 */
export const deserializeAttestationData = <T>(
  schema: Schema,
  data: Uint8Array
): T => getAttestationDataCodec(schema).decode(data) as T;

type ByteLike = Uint8Array | number[];

const splitJoinedVecs = (bytes: ByteLike): ByteLike[] => {
  let offset = 0;
  const ret = [];
  while (offset < bytes.length) {
    const len = u32FromLeBytes(bytes.slice(offset, offset + 4));
    offset += 4;
    ret.push(bytes.slice(offset, offset + len));
    offset += len;
  }
  return ret;
};

const u32FromLeBytes = (bytes: ByteLike): number => {
  if (bytes.length !== 4) {
    throw new Error("Input must be a 4-byte array");
  }

  return (
    (bytes[0] << 0) | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)
  );
};
