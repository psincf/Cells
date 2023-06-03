pub trait UnwrapUnchecked<T> {
    unsafe fn unwrap_unchecked(self) -> T;
}

impl<T> UnwrapUnchecked<T> for Option<T> {
    unsafe fn unwrap_unchecked(self) -> T {
        if let Some(data) = self {
            return data
        } else {
            std::hint::unreachable_unchecked();
        }
    }
}