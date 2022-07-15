use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ops::DerefMut;

/// Scoped owned vector.
pub struct VecScope<T>(
    // Inhibit the compiler to drop it as technically, it is a reference
    // to the original data. [`fn std::mem::transmute_copy`] is actually,
    // a forced [`trait Copy`] satisfaction. Letting this drop will cause
    // a double-free which is potential undefined behaviour.
    ManuallyDrop<Vec<T>>
);

impl<T> Deref for VecScope<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for VecScope<T> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

impl<T> VecScope<T> {
    /// Hold a mutable reference and present it as if it it was constructed
    /// in the scope. Therefore it is unable to outlive the lifetime of any
    /// owned data's references within that scope contained in this vector.
    pub fn new(vec: &mut Vec<T>) -> Self {
        Self (unsafe { std::mem::transmute_copy(vec) })
    }
}

impl<T> Drop for VecScope<T> {
    fn drop(&mut self) {
        // This is NOT unsafe by logic, as none of the data will be kept
        // within the vector after it drops at scope end. Therefore all
        // references not outliving the scope will not be dangling thus
        // it's a completely safe operation, in fact.
        self.0.clear();
    }
}
