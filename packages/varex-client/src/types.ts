export interface ProxyRequest { url: string; method: string; headers: Headers; body?: BodyInit | null; }
export interface ProxyResponse { status: number; headers: Headers; body: ReadableStream<Uint8Array> | null; }
