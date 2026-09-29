import { defineConfig } from 'tsup';

const SHARED_OPTIONS = {
    entry: {
        index: 'src/index.ts',
        accounts: 'src/generated/accounts/index.ts',
        instructions: 'src/generated/instructions/index.ts',
    },
    outDir: './dist/src',
    outExtension: ({ format }) => ({ js: format === 'cjs' ? '.js' : '.mjs' }),
    sourcemap: true,
    splitting: true,
    treeshake: true,
    target: 'es2020',
    external: ['@solana/kit'],
};

export default defineConfig([
    { ...SHARED_OPTIONS, format: 'cjs' },
    { ...SHARED_OPTIONS, format: 'esm' },
]);
