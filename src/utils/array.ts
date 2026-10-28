/**
 * 返回排序后的新数组。
 *
 * 统一走这里而不是 `Array#toSorted`：Tauri 在旧版 macOS WebView 上并不支持该方法，
 * 同时也能满足 oxlint 对原地排序的约束。
 */
export function sortCopy<T>(list: readonly T[], compare: (a: T, b: T) => number): T[] {
  const copy = [...list]
  // oxlint-disable-next-line unicorn/no-array-sort
  return copy.sort(compare)
}
