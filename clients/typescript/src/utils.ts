import { BorshSchema } from "borsher";

import { Schema } from "./generated";

// A char type does not exist on BorshSchema, so we (de)serialize
// using the 4 byte representation.
const CHAR_SCHEMA = BorshSchema.Array(BorshSchema.u8, 4);

type SchemaOutputTypes =
  | number
  | number[]
  | string
  | string[]
  | bigint
  | bigint[]
  | boolean
  | boolean[];
/**
 * Maps the SAS compact byte layout to the equivalent data type.
 */
const compactLayoutMapping: Record<number, BorshSchema<SchemaOutputTypes>> = {
  0: BorshSchema.u8,
  1: BorshSchema.u16,
  2: BorshSchema.u32,
  3: BorshSchema.u64,
  4: BorshSchema.u128,
  5: BorshSchema.i8,
  6: BorshSchema.i16,
  7: BorshSchema.i32,
  8: BorshSchema.i64,
  9: BorshSchema.i128,
  10: BorshSchema.bool,
  11: CHAR_SCHEMA,
  12: BorshSchema.String,
  13: BorshSchema.Vec(BorshSchema.u8),
  14: BorshSchema.Vec(BorshSchema.u16),
  15: BorshSchema.Vec(BorshSchema.u32),
  16: BorshSchema.Vec(BorshSchema.u64),
  17: BorshSchema.Vec(BorshSchema.u128),
  18: BorshSchema.Vec(BorshSchema.i8),
  19: BorshSchema.Vec(BorshSchema.i16),
  20: BorshSchema.Vec(BorshSchema.i32),
  21: BorshSchema.Vec(BorshSchema.i64),
  22: BorshSchema.Vec(BorshSchema.i128),
  23: BorshSchema.Vec(BorshSchema.bool),
  24: BorshSchema.Vec(BorshSchema.String),
  25: CHAR_SCHEMA,
};
const MAX_LAYOUT_VALUE = 25;

/**
 * Given the onchain representation of a Schema, we generate a Borsh schema
 * for (de)serialization.
 * @param schema
 * @returns
 */
export const convertSasSchemaToBorshSchema = (
  schema: Schema
): BorshSchema<Record<string, unknown>> => {
  const textDecoder = new TextDecoder();
  const fields = splitJoinedVecs(Uint8Array.from(schema.fieldNames)).map((f) =>
    textDecoder.decode(Uint8Array.from(f))
  );

  if (fields.length !== schema.layout.length) {
    throw new Error("Schema field names and layout do not match");
  }

  return BorshSchema.Struct(
    fields.reduce(
      (acc, field, index) => {
        const layoutByte = schema.layout[index];
        if (layoutByte > MAX_LAYOUT_VALUE) {
          throw new Error("Invalid Schema layout value");
        }
        acc[field] = compactLayoutMapping[layoutByte];
        return acc;
      },
      {} as Record<string, BorshSchema<SchemaOutputTypes>>
    )
  );
};

/**
 * Given a SAS Schema and an object that represents the Attestation data,
 * serialize the Attestation data to valid byte array.
 * @param schema
 */
export const serializeAttestationData = (
  schema: Schema,
  data: Record<string, unknown>
): Uint8Array => {
  const borshSchema = convertSasSchemaToBorshSchema(schema);
  return borshSchema.serialize(data);
};

/**
 * Given a SAS Schema and a byte array of Attestation data,
 * deserialize the Attestation data to an object.
 * @param schema
 */
export const deserializeAttestationData = <T>(
  schema: Schema,
  data: Uint8Array
): T => {
  const borshSchema = convertSasSchemaToBorshSchema(schema);
  return borshSchema.deserialize(data) as T;
};

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
