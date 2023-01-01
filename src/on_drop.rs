// Minect is library that allows a program to connect to a running Minecraft instance without
// requiring any Minecraft mods.
//
// © Copyright (C) 2021, 2022 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
//
// This file is part of Minect.
//
// Minect is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Minect is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Minect.
// If not, see <http://www.gnu.org/licenses/>.

use futures::Future;
use std::{
    mem::ManuallyDrop,
    pin::Pin,
    task::{Context, Poll},
};

pub struct OnDrop<F: FnOnce()> {
    on_drop: ManuallyDrop<F>,
}
impl<F: FnOnce()> OnDrop<F> {
    pub fn new(on_drop: F) -> Self {
        Self {
            on_drop: ManuallyDrop::new(on_drop),
        }
    }
}
impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        let on_drop = unsafe { ManuallyDrop::take(&mut self.on_drop) };
        on_drop();
    }
}

pub trait OnDropFutureExt
where
    Self: Future + Sized,
{
    fn on_drop<D: FnOnce()>(self, on_drop: D) -> OnDropFuture<Self, D>;
}
impl<F: Future> OnDropFutureExt for F {
    fn on_drop<D: FnOnce()>(self, on_drop: D) -> OnDropFuture<Self, D> {
        OnDropFuture {
            inner: self,
            on_drop: ManuallyDrop::new(on_drop),
        }
    }
}

pub struct OnDropFuture<F: Future, D: FnOnce()> {
    inner: F,
    on_drop: ManuallyDrop<D>,
}
impl<F: Future, D: FnOnce()> OnDropFuture<F, D> {
    // This is safe, see: https://doc.rust-lang.org/std/pin/#pinning-is-structural-for-field
    fn get_mut_inner(self: Pin<&mut Self>) -> Pin<&mut F> {
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }

    // This is safe, see: https://doc.rust-lang.org/std/pin/#pinning-is-not-structural-for-field
    fn get_mut_on_drop(self: Pin<&mut Self>) -> &mut ManuallyDrop<D> {
        unsafe { &mut self.get_unchecked_mut().on_drop }
    }
}
impl<F: Future, D: FnOnce()> Future for OnDropFuture<F, D> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        self.get_mut_inner().poll(cx)
    }
}
impl<F: Future, D: FnOnce()> Drop for OnDropFuture<F, D> {
    fn drop(&mut self) {
        // This is safe, see: https://doc.rust-lang.org/std/pin/#drop-implementation
        inner_drop(unsafe { Pin::new_unchecked(self) });
        fn inner_drop<F: Future, D: FnOnce()>(this: Pin<&mut OnDropFuture<F, D>>) {
            let on_drop = unsafe { ManuallyDrop::take(this.get_mut_on_drop()) };
            on_drop()
        }
    }
}
