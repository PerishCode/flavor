import type { Ref } from "vue";
import { defineComponent as defineView } from "./component";
import legacy = require("./legacy");

export { defineView };
export type { Ref as VueRef };
export type * as ViewTypes from "./types";

export enum ViewState {
  Idle = "idle",
  Ready = "ready",
}

export namespace ViewRegistry {
  export const key = "view";
}

export interface ViewModel<T> extends Entity {
  id: string;
  value?: Ref<T>;
  commit(next: T | null): Result<T>;
}

export type ViewMap<T> = Record<string, ViewModel<T>> | null;

export type ViewPayload<T> = {
  readonly id?: string;
  commit(next: T): Result<T>;
  [key: string]: unknown;
};
