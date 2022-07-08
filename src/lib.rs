use crate::sys::avformat_alloc_output_context2;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_int;

pub mod codec;
pub mod context;
pub mod dict;
pub mod format;
pub mod frame;
pub mod io;
pub mod packet;
pub mod stream;
mod sys;
pub mod util;

pub use sys::AVMediaType as AvMediaType;

#[derive(Debug)]
pub enum AvError {
    Todo,
}

fn wrap_error(id: c_int) -> AvError {
    println!("{}", id);

    AvError::Todo
}

pub trait AvOwnable {
    fn drop(&mut self);
}

pub struct AvOwned<T: AvOwnable> {
    inner: T,
}

impl<T: AvOwnable> AvOwned<T> {
    pub fn wrap(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T: AvOwnable> Deref for AvOwned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AvOwnable> DerefMut for AvOwned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: AvOwnable> Drop for AvOwned<T> {
    fn drop(&mut self) {
        T::drop(self)
    }
}

pub struct AvBorrow<'a, S: 'a, T> {
    _src: PhantomData<&'a S>,
    value: T,
}

impl<'a, S: 'a, T> AvBorrow<'a, S, T> {
    pub fn wrap(_src: &'a S, value: T) -> Self {
        Self {
            _src: Default::default(),
            value,
        }
    }
}

impl<'a, S: 'a, T> Deref for AvBorrow<'a, S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct AvBorrowMut<'a, S: 'a, T> {
    _src: PhantomData<&'a mut S>,
    value: T,
}

impl<'a, S: 'a, T> AvBorrowMut<'a, S, T> {
    pub fn wrap(_src: &'a mut S, value: T) -> Self {
        Self {
            _src: Default::default(),
            value,
        }
    }
}

impl<'a, S: 'a, T> Deref for AvBorrowMut<'a, S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, S: 'a, T> DerefMut for AvBorrowMut<'a, S, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
