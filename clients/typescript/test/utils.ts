import { assert } from "chai";
import { getSchemaDecoder } from "../src/generated";
import { convertSasSchemaToBorshSchema } from "../src/utils";

describe("Utils", () => {
  const schemaAccountBytes = Uint8Array.from([
    1, 147, 244, 210, 208, 208, 76, 164, 106, 193, 96, 129, 24, 152, 59, 215,
    13, 112, 136, 111, 235, 117, 29, 128, 253, 99, 200, 171, 204, 126, 178, 74,
    175, 9, 0, 0, 0, 116, 101, 115, 116, 95, 100, 97, 116, 97, 20, 0, 0, 0, 115,
    99, 104, 101, 109, 97, 32, 102, 111, 114, 32, 116, 101, 115, 116, 32, 100,
    97, 116, 97, 2, 0, 0, 0, 12, 0, 20, 0, 0, 0, 4, 0, 0, 0, 110, 97, 109, 101,
    8, 0, 0, 0, 108, 111, 99, 97, 116, 105, 111, 110, 0, 1,
  ]);

  describe("convertSasSchemaToBorshSchema", () => {
    it("should convert SAS Schema to a proper BorshSchema", () => {
      const decoder = getSchemaDecoder();
      const schema = decoder.decode(schemaAccountBytes);
      const borshSchema = convertSasSchemaToBorshSchema(schema);
      const testData = {
        name: "hello",
        location: 10,
      };
      const serialized = borshSchema.serialize(testData);
      const deserialized = borshSchema.deserialize(serialized);
      assert.deepEqual(testData, deserialized);
    });
  });
});
