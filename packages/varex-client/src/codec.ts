export interface Codec { encode(url: string): Promise<string>; decode(path: string): Promise<string>; }
export const plainCodec: Codec = { async encode(u) { return u; }, async decode(p) { return p; } };
