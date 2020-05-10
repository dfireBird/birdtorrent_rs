pub fn to_vec<T: Clone>(data: &[T]) -> Vec<T> {
    data.iter().cloned().collect()
}
