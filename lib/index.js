/**
 * Host half of dsh-locale-fr.
 *
 * All the real work happens in the browser (lib/client.js): the locale catalog
 * and the dictionary registry live client-side. This half exists only so that
 * a composition row can resolve the package on the host; it wires no service.
 *
 * @module dsh-locale-fr
 */

/** No host-side service required. */
export const inject = [];

/**
 * Host plugin body: intentionally empty.
 * @param {unknown} _ctx - host cordis context, unused.
 */
export function apply(_ctx) {}
