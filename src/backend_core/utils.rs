use std::any::Any;

// Convert a value of type S to type T
pub fn non_primitive_cast<S: 'static, T: 'static>(value: S) -> Option<T> {
    unsafe {
        let t_ptr = (&value as &dyn Any).downcast_ref::<T>()? as *const T;
        let t = std::ptr::read(t_ptr);
        std::mem::forget(value);
        Some(t)
    }
}
