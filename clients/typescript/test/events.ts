import { address, getAddressEncoder } from '@solana/kit';
import { assert } from 'chai';

import { getCloseAttestationEventEventEncoder, parseCloseAttestationEventEvent } from '../src';

describe('Events', () => {
    const schema = address('tbFevHibEdBNFJfZ7xKC8k1th8pt2YPEXTk4sGMxCGa');
    const attestationData = new Uint8Array([1, 2, 3]);
    const eventIxTag = [228, 69, 165, 46, 81, 203, 154, 29];
    const closeEventType = 0;
    const dataLength = [3, 0, 0, 0];
    // Layout of `CloseAttestationEvent::to_bytes` in program/src/events.rs.
    const emitted = new Uint8Array([
        ...eventIxTag,
        closeEventType,
        ...getAddressEncoder().encode(schema),
        ...dataLength,
        ...attestationData,
    ]);

    it('parses a close event as the program emits it', () => {
        const event = parseCloseAttestationEventEvent(emitted);
        assert.equal(event.discriminator, 0);
        assert.equal(event.schema, schema);
        assert.deepEqual(new Uint8Array(event.attestationData), attestationData);
    });

    it('encodes a close event as the program emits it', () => {
        const encoded = getCloseAttestationEventEventEncoder().encode({ discriminator: 0, schema, attestationData });
        assert.deepEqual(new Uint8Array(encoded), emitted);
    });
});
