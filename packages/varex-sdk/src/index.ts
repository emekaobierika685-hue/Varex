export type PluginEvent =
  | "request" | "response" | "rewrite:html" | "rewrite:js"
  | "rewrite:css" | "cookie" | "navigate" | "transport:select";

export interface PluginContext {
  readonly apiVersion: 1;
  readonly target: string;
  readonly proxyOrigin: string;
}

export type PluginHandler<T = unknown> = (payload: T, ctx: PluginContext) => Promise<unknown> | unknown;

export interface Plugin {
  apiVersion: 1;
  name: string;
  priority?: number;
  handlers: Partial<Record<PluginEvent, PluginHandler<any>>>;
}
