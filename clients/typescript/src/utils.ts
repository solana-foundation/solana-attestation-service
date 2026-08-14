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

import { Schema, SchemaDataType } from "./generated";

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
 * A Rust `char` holds a Unicode scalar value: at most U+10FFFF, and never a
 * surrogate. Values outside that range have no character to map to.
 */
const isUnicodeScalarValue = (codePoint: number): boolean =>
  codePoint <= 0x10ffff && (codePoint < 0xd800 || codePoint > 0xdfff);

/**
 * Rust encodes a `char` as its 4-byte little-endian Unicode code point.
 */
const getCharCodec = (): Codec<string> =>
  transformCodec(
    getU32Codec(),
    (character: string) => {
      const codePoint = character.codePointAt(0);
      if (
        codePoint === undefined ||
        !isUnicodeScalarValue(codePoint) ||
        String.fromCodePoint(codePoint) !== character
      ) {
        throw new Error("Char fields must hold exactly one Unicode character");
      }
      return codePoint;
    },
    (codePoint) => {
      if (!isUnicodeScalarValue(codePoint)) {
        throw new Error(
          `Char field holds ${codePoint}, which is not a Unicode character`
        );
      }
      return String.fromCodePoint(codePoint);
    }
  );

const getStringCodec = (): Codec<string> =>
  addCodecSizePrefix(getUtf8Codec(), getU32Codec());

/**
 * Maps each schema data type to the codec that reads and writes the matching
 * field of an Attestation's data blob.
 */
const dataTypeCodecs: Record<SchemaDataType, () => Codec<any>> = {
  [SchemaDataType.U8]: getU8Codec,
  [SchemaDataType.U16]: getU16Codec,
  [SchemaDataType.U32]: getU32Codec,
  [SchemaDataType.U64]: getU64Codec,
  [SchemaDataType.U128]: getU128Codec,
  [SchemaDataType.I8]: getI8Codec,
  [SchemaDataType.I16]: getI16Codec,
  [SchemaDataType.I32]: getI32Codec,
  [SchemaDataType.I64]: getI64Codec,
  [SchemaDataType.I128]: getI128Codec,
  [SchemaDataType.Bool]: getBooleanCodec,
  [SchemaDataType.Char]: getCharCodec,
  [SchemaDataType.String]: getStringCodec,
  [SchemaDataType.VecU8]: () => getArrayCodec(getU8Codec()),
  [SchemaDataType.VecU16]: () => getArrayCodec(getU16Codec()),
  [SchemaDataType.VecU32]: () => getArrayCodec(getU32Codec()),
  [SchemaDataType.VecU64]: () => getArrayCodec(getU64Codec()),
  [SchemaDataType.VecU128]: () => getArrayCodec(getU128Codec()),
  [SchemaDataType.VecI8]: () => getArrayCodec(getI8Codec()),
  [SchemaDataType.VecI16]: () => getArrayCodec(getI16Codec()),
  [SchemaDataType.VecI32]: () => getArrayCodec(getI32Codec()),
  [SchemaDataType.VecI64]: () => getArrayCodec(getI64Codec()),
  [SchemaDataType.VecI128]: () => getArrayCodec(getI128Codec()),
  [SchemaDataType.VecBool]: () => getArrayCodec(getBooleanCodec()),
  [SchemaDataType.VecChar]: () => getArrayCodec(getCharCodec()),
  [SchemaDataType.VecString]: () => getArrayCodec(getStringCodec()),
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
  if (schema.fieldNames.length !== schema.layout.length) {
    throw new Error("Schema field names and layout do not match");
  }

  return getStructCodec(
    schema.fieldNames.map((field, index) => {
      const getFieldCodec = dataTypeCodecs[schema.layout[index]];
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
