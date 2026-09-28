pub enum Diff<T> {
    Changed { old: T, new: T},
    Same { current: T}
}