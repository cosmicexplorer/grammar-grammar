use std::marker::PhantomData;

/* Needed to avoid spurious "unused generic param" error in WriteMap. */
pub struct _Phantom<S>(PhantomData<S>);

impl<S> _Phantom<S> {
  pub fn new() -> Self { Self(PhantomData) }
}

unsafe impl<S> Send for _Phantom<S> {}
unsafe impl<S> Sync for _Phantom<S> {}
