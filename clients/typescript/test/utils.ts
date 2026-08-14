import { assert } from "chai";
import { address } from "@solana/kit";
import {
  getSchemaDecoder,
  getSchemaEncoder,
  Schema,
  SchemaDataType,
} from "../src/generated";
import {
  deserializeAttestationData,
  getAttestationDataCodec,
  serializeAttestationData,
} from "../src/utils";

const makeSchema = (
  layout: SchemaDataType[],
  fieldNames: string[]
): Schema => ({
  discriminator: 1,
  credential: address("11111111111111111111111111111111"),
  name: "test",
  description: "test",
  layout,
  fieldNames,
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

  describe("getSchemaDecoder", () => {
    it("decodes the text, layout and field name blobs", () => {
      const schema = getSchemaDecoder().decode(schemaAccountBytes);

      assert.equal(schema.name, "test_data");
      assert.equal(schema.description, "schema for test data");
      assert.deepEqual(schema.layout, [
        SchemaDataType.String,
        SchemaDataType.U8,
      ]);
      assert.deepEqual(schema.fieldNames, ["name", "location"]);
    });

    it("round trips the account bytes it decoded", () => {
      const schema = getSchemaDecoder().decode(schemaAccountBytes);

      assert.deepEqual(
        Array.from(getSchemaEncoder().encode(schema)),
        Array.from(schemaAccountBytes)
      );
    });
  });

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
      const schema = makeSchema(
        [SchemaDataType.U8, SchemaDataType.VecString, SchemaDataType.U128],
        ["count", "tags", "big"]
      );
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
      const schema = makeSchema(
        [SchemaDataType.Char, SchemaDataType.VecChar],
        ["grade", "grades"]
      );
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

    it("rejects char data that is not a Unicode character", () => {
      // The program only advances four bytes for a char field without checking
      // the value, so any u32 can reach the client.
      const schema = makeSchema([SchemaDataType.Char], ["grade"]);

      for (const codePoint of [0x110000, 0xffffffff, 0xd800, 0xdfff]) {
        const encoded = Uint8Array.from([
          codePoint & 0xff,
          (codePoint >>> 8) & 0xff,
          (codePoint >>> 16) & 0xff,
          (codePoint >>> 24) & 0xff,
        ]);
        assert.throws(
          () => deserializeAttestationData(schema, encoded),
          `Char field holds ${codePoint}, which is not a Unicode character`
        );
      }
    });

    it("rejects char values that cannot round trip through Rust", () => {
      const schema = makeSchema([SchemaDataType.Char], ["grade"]);

      for (const character of ["", "ab", "\ud800"]) {
        assert.throws(
          () => serializeAttestationData(schema, { grade: character }),
          "Char fields must hold exactly one Unicode character"
        );
      }
    });

    it("round trips every supported layout type", () => {
      const schema = makeSchema(
        [
          SchemaDataType.U8, SchemaDataType.U16, SchemaDataType.U32,
          SchemaDataType.U64, SchemaDataType.U128, SchemaDataType.I8,
          SchemaDataType.I16, SchemaDataType.I32, SchemaDataType.I64,
          SchemaDataType.I128, SchemaDataType.Bool, SchemaDataType.Char,
          SchemaDataType.String, SchemaDataType.VecU8, SchemaDataType.VecU16,
          SchemaDataType.VecU32, SchemaDataType.VecU64, SchemaDataType.VecU128,
          SchemaDataType.VecI8, SchemaDataType.VecI16, SchemaDataType.VecI32,
          SchemaDataType.VecI64, SchemaDataType.VecI128, SchemaDataType.VecBool,
          SchemaDataType.VecChar, SchemaDataType.VecString,
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
      const schema = makeSchema([26 as SchemaDataType], ["mystery"]);
      assert.throws(
        () => getAttestationDataCodec(schema),
        "Invalid Schema layout value"
      );
    });

    it("throws when field names and layout lengths disagree", () => {
      const schema = makeSchema(
        [SchemaDataType.U8, SchemaDataType.U8],
        ["only_one"]
      );
      assert.throws(
        () => getAttestationDataCodec(schema),
        "Schema field names and layout do not match"
      );
    });
  });
});
