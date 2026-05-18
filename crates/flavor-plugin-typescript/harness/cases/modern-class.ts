export default abstract class UserStore<T> extends BaseStore implements Disposable {
  private readonly cache!: Map<string, T>;

  static create<T>(seed: T): UserStore<T> {
    return new UserStore<T>();
  }

  get size(): number {
    return this.cache.size;
  }

  set size(value: number) {
    this.resize(value);
  }
}
