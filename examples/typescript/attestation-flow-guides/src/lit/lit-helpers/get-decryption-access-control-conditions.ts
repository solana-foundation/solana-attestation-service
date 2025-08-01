import ipfsOnlyHash from "typestub-ipfs-only-hash";

export const getDecryptionAccessControlConditions = async (litActionCodeDecrypt: string) => {
    return [
        {
            method: "",
            params: [":currentActionIpfsId"],
            pdaParams: [],
            pdaInterface: { offset: 0, fields: {} },
            pdaKey: "",
            chain: "solana",
            returnValueTest: {
                key: "",
                comparator: "=",
                value: await ipfsOnlyHash.of(litActionCodeDecrypt),
            },
        },
    ];
};