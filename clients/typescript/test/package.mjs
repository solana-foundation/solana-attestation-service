import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

const ENTRIES = [
    { exportName: 'findSchemaPda', specifier: 'sas-lib' },
    { exportName: 'getSchemaDecoder', specifier: 'sas-lib/accounts' },
    { exportName: 'parseCreateSchemaInstruction', specifier: 'sas-lib/instructions' },
];

describe('published package entries', () => {
    for (const { exportName, specifier } of ENTRIES) {
        it(`${specifier} exposes ${exportName} as ESM`, async () => {
            const entry = await import(specifier);
            assert.equal(typeof entry[exportName], 'function');
        });

        it(`${specifier} exposes ${exportName} as CJS`, () => {
            assert.equal(typeof require(specifier)[exportName], 'function');
        });
    }
});
