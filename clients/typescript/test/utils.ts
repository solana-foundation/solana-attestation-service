import { assert } from "chai";
import { getSchemaDecoder } from "../src/generated";
import { convertSasSchemaToBorshSchema } from "../src/utils";
import { Address, address, ProgramDerivedAddressBump } from "@solana/kit";
import { deriveAttestationMintPda, deriveAttestationPda, deriveCredentialPda, deriveSchemaMintPda, deriveSchemaPda } from "../src";

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
  describe("pda derivation", () => {
    it("should derive a credential PDA", async () => {
      const issuer = address('tbFevHibEdBNFJfZ7xKC8k1th8pt2YPEXTk4sGMxCGa');
      const credentialName = 'test';
      const expectedPda = ['CdUAYGvNc7NdtNgXmxTXoUWR5NjpcU4Za4vtoP2AVZD4', 255] as [Address<string>, ProgramDerivedAddressBump];
      const testPda = await deriveCredentialPda({ authority: issuer, name: credentialName })
      assert.deepEqual(testPda, expectedPda);
    });
    it("should derive a schema PDA", async () => {
      const credential = address('2zHazqL3MVayNGkDrTmAC7VsvP5QrSYfiyA7uf29Usmd');
      const schemaName = 'test';
      const version = 1;
      const expectedPda = ['bD7cVGpuTHY43fxtRoqJYi58U6Yi3kMyVck2DyZZRKq', 255] as [Address<string>, ProgramDerivedAddressBump];
      const testPda = await deriveSchemaPda({
        credential,
        version,
        name: schemaName
      })
      assert.deepEqual(testPda, expectedPda);
    });
    it("should derive an attestation PDA", async () => {
      const credential = address('G6QmvUp3a1Kv9rX2LqHDH8AWcKD8yaufcoXEB1h6SzN8');
      const schema = address('GSwz99vWPKnePyeYTM5iionEfArVmfrufV4AaV4SecTH');
      const nonce = address('Bdf3cgpzgboZq95T4AVYNxuYGDVE4pwLNQBhQ2ob8CoG');
      const expectedPda = ['CnhgnrLiawRWitfjrrUfWdR2jpwKbKGDccbk3ne171iu', 255] as [Address<string>, ProgramDerivedAddressBump];
      const testPda = await deriveAttestationPda({
        credential,
        schema,
        nonce
      })
      assert.deepEqual(testPda, expectedPda);
    });
    it("should derive a schema mint PDA", async () => {
      const schema = address('GCVt9SmgLF8bgEVwZAhQ9A2skwj5TvEnyn8Z7eUm583E');
      const expectedPda = ['9JLQQK3zeEjiq2AJ1XPN765bYnLrBWJSFfyjDwdSMmyN', 245] as [Address<string>, ProgramDerivedAddressBump];
      const testPda = await deriveSchemaMintPda({
        schema,
      })
      assert.deepEqual(testPda, expectedPda);
    });
    it("should derive an attestation mint PDA", async () => {
      const attestation = address('3z8EuPHrzhfVWuDomSGjU13ABDgQC75DMHoDNBgxdzKR');
      const expectedPda = ['61FeMtSXR8H22fodXNrTkwmrSmBuTNcAvKzZXvZRPMkX', 253] as [Address<string>, ProgramDerivedAddressBump];
      const testPda = await deriveAttestationMintPda({
        attestation,
      })
      assert.deepEqual(testPda, expectedPda);
    });
  })
});