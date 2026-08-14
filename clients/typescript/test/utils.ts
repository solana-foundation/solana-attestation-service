import { assert } from "chai";
import { address } from "@solana/kit";
import { getSchemaDecoder, Schema } from "../src/generated";
import {
  deserializeAttestationData,
  getAttestationDataCodec,
  serializeAttestationData,
} from "../src/utils";

/**
 * Encodes field names the way the program stores them: each name is prefixed
 * with its u32 little-endian length and the results are concatenated.
 */
const encodeFieldNames = (names: string[]): Uint8Array => {
  const encoder = new TextEncoder();
  const parts = names.flatMap((name) => {
    const bytes = encoder.encode(name);
    const len = new Uint8Array(4);
    new DataView(len.buffer).setUint32(0, bytes.length, true);
    return [len, bytes];
  });
  const total = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
};

const makeSchema = (layout: number[], fieldNames: string[]): Schema => ({
  discriminator: 1,
  credential: address("11111111111111111111111111111111"),
  name: new TextEncoder().encode("test"),
  description: new TextEncoder().encode("test"),
  layout: Uint8Array.from(layout),
  fieldNames: encodeFieldNames(fieldNames),
  isPaused: false,
  version: 1,
});

describe("Utils", () => {
  const schemaAccountBytes = Uint8Array.from([
    1, 147, 244, 210, 208, 208, 76, 164, 106, 193, 96, 129, 24, 152, 59, 215,
    13, 112, 136, 111, 235, 117, 29, 128, 253, 99, 200, 171, 204, 126, 178, 74,
    175, 9, 0, 0, 0, 116, 101, 115, 116, 95, 100, 97, 116, 97, 20, 0, 0, 0, 115,
    99, 104, 101, 109, 97, 32, 102, 111, 114, 32, 116, 101, 115, 116, 32, 100,
    97, 116, 97, 2, 0, 0, 0, 12, 0, 20, 0, 0, 0, 4, 0, 0, 0, 110, 97, 109, 101,
    8, 0, 0, 0, 108, 111, 99, 97, 116, 105, 111, 110, 0, 1,
  ]);

  describe("getAttestationDataCodec", () => {
    it("round trips data for a Schema decoded from account bytes", () => {
      const schema = getSchemaDecoder().decode(schemaAccountBytes);
      const codec = getAttestationDataCodec(schema);
      const testData = { name: "hello", location: 10 };

      const serialized = codec.encode(testData);
      assert.deepEqual(
        Array.from(serialized),
        [5, 0, 0, 0, 104, 101, 108, 108, 111, 10]
      );
      assert.deepEqual(codec.decode(serialized), testData);
    });

    it("matches the byte layout the program validates against", () => {
      // Mirrors the `u8, Vec<String>, u128` case in
      // program/src/state/attestation.rs::attestation_validate_data.
      const schema = makeSchema([0, 25, 4], ["count", "tags", "big"]);
      const data = {
        count: 10,
        tags: ["test1", "test2"],
        big: 199n,
      };

      const serialized = serializeAttestationData(schema, data);
      assert.deepEqual(Array.from(serialized), [
        10,
        // Vec<String> length
        2, 0, 0, 0,
        // "test1"
        5, 0, 0, 0, 116, 101, 115, 116, 49,
        // "test2"
        5, 0, 0, 0, 116, 101, 115, 116, 50,
        // 199u128
        199, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      ]);
      assert.deepEqual(deserializeAttestationData(schema, serialized), data);
    });

    it("encodes char as a 4 byte little-endian code point", () => {
      const schema = makeSchema([11, 24], ["grade", "grades"]);
      const data = { grade: "A", grades: ["B", "C"] };

      const serialized = serializeAttestationData(schema, data);
      assert.deepEqual(Array.from(serialized), [
        65, 0, 0, 0,
        2, 0, 0, 0,
        66, 0, 0, 0,
        67, 0, 0, 0,
      ]);
      assert.deepEqual(deserializeAttestationData(schema, serialized), data);
    });

    it("round trips every supported layout type", () => {
      const schema = makeSchema(
        [
          0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
          20, 21, 22, 23, 24, 25,
        ],
        [
          "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
          "bool", "char", "string", "vecU8", "vecU16", "vecU32", "vecU64",
          "vecU128", "vecI8", "vecI16", "vecI32", "vecI64", "vecI128",
          "vecBool", "vecChar", "vecString",
        ]
      );
      const data = {
        u8: 1, u16: 2, u32: 3, u64: 4n, u128: 5n,
        i8: -1, i16: -2, i32: -3, i64: -4n, i128: -5n,
        bool: true, char: "🔥", string: "hello",
        vecU8: [1, 2], vecU16: [3, 4], vecU32: [5, 6], vecU64: [7n, 8n],
        vecU128: [9n, 10n], vecI8: [-1, -2], vecI16: [-3, -4],
        vecI32: [-5, -6], vecI64: [-7n, -8n], vecI128: [-9n, -10n],
        vecBool: [true, false], vecChar: ["a", "b"], vecString: ["x", "y"],
      };

      const serialized = serializeAttestationData(schema, data);
      assert.deepEqual(deserializeAttestationData(schema, serialized), data);
    });

    it("throws when the layout contains an unknown type", () => {
      const schema = makeSchema([26], ["mystery"]);
      assert.throws(
        () => getAttestationDataCodec(schema),
        "Invalid Schema layout value"
      );
    });

    it("throws when field names and layout lengths disagree", () => {
      const schema = makeSchema([0, 0], ["only_one"]);
      assert.throws(
        () => getAttestationDataCodec(schema),
        "Schema field names and layout do not match"
      );
    });
  });
});
