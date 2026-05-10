export type Keys<T> = keyof T;
export type Value<T, K extends keyof T> = T[K];
export type Box<T> = {
  readonly [K in keyof T]?: T[K];
};
export type Result<T> = T extends infer U ? U : T;
export type Brand = unique symbol;
