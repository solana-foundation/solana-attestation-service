import solanaConfig from '@solana/eslint-config-solana';

export default [
    ...solanaConfig,
    {
        ignores: [
            '**/.claude/**',
            '**/.remember/**',
            '**/.git/**',
            '**/dist/**',
            '**/node_modules/**',
            '**/target/**',
            '**/generated/**',
            'clients/typescript/src/generated/**',
            'clients/typescript/test/**',
            'examples/**',
            'idl/**',
            'scripts/**',
            'eslint.config.mjs',
            '**/*.mjs',
        ],
    },
];
