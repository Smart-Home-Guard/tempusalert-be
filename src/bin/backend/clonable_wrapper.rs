use std::sync::Arc;

pub struct ClonableWrapper<T> where T: ?Sized + 'static + Send + Sync {
    clone_closure: Box<dyn Fn(Arc<T>) -> Box<T> + Send + Sync>,
    data: Arc<T>,
}

impl<T> ClonableWrapper<T> where T: ?Sized + 'static + Send + Sync {
    pub fn create(clone: Box<dyn Fn(Arc<T>) -> Box<T> + Send + Sync>, data: Arc<T>) -> Self {
        ClonableWrapper {
            clone_closure: clone,
            data,
        }
    }
    
    pub fn clone(&self) -> Box<T> {
        (self.clone_closure)(self.data.clone())
    }
}
