#![no_std]
#![allow(unused_unsafe)]

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

struct MailboxBuffer<T, const N: usize> {
    buf: [UnsafeCell<MaybeUninit<T>>; N],
    len: AtomicIsize,
}

impl<T, const N: usize> MailboxBuffer<T, N> {
    fn new() -> Self {
        /* SAFETY: the buffer can be uninitialized, because the element type
         * is MaybeUninit<T> */
        return unsafe {
            Self { buf: MaybeUninit::uninit().assume_init(), len: AtomicIsize::new(0) }
        };
    }

    pub(crate) fn try_push(&self, item: T) -> Option<T> {
        /* UNSAFETY: threads cannot push and pop at the same time */
        /* SAFETY: because poping requires &mut self, having both it and
         * &self at the same time is 'impossible', making this safe */

        /* Read and increment atomically, to ensure we are the only ones
         * accessing an index */
        let oldlen = self.len.fetch_add(1, Ordering::SeqCst);

        /* Return if no place */
        if oldlen > N as isize || oldlen < 0 {
            return Some(item);
        }
        if oldlen == N as isize {
            self.len.store(N as isize, Ordering::SeqCst);
            return Some(item);
        }

        /* SAFETY: Now we are the only one having access to self.buf[n] */
        unsafe {
            let n = oldlen as usize;
            let cellref = &self.buf[n];
            let itemref = &mut *cellref.get();
            ptr::write(itemref.as_mut_ptr(), item);
        }

        return None;
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        let len = self.len.get_mut();
        if *len == 0 {
            return None;
        }

        *len = len.wrapping_sub(1);

        /* SAFETY: Now we are the only one having access to self.buf[n] */
        let item = unsafe {
            let cellref = &mut self.buf[*len as usize];
            let itemref = cellref.get_mut();
            ptr::read(itemref.as_ptr())
        };

        return Some(item);
    }
}

unsafe impl<T: Send, const N: usize> Send for MailboxBuffer<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for MailboxBuffer<T, N> {}

pub struct Mailbox<T, const N: usize> {
    /* FIXME: lots of false sharing here */
    buffers:      [UnsafeCell<MailboxBuffer<T, N>>; 2],
    sender_count: [AtomicUsize; 2],

    send_buf: AtomicUsize,
}

impl<T, const N: usize> Mailbox<T, N> {
    pub fn new() -> Self {
        Self {
            buffers:      [
                UnsafeCell::new(MailboxBuffer::new()),
                UnsafeCell::new(MailboxBuffer::new()),
            ],
            sender_count: [AtomicUsize::new(0), AtomicUsize::new(0)],
            send_buf:     AtomicUsize::new(0),
        }
    }

    pub fn try_push(&self, item: T) -> Option<T> {
        let n = self.send_buf.load(Ordering::SeqCst);
        self.sender_count[n].fetch_add(1, Ordering::SeqCst);

        if self.send_buf.load(Ordering::SeqCst) != n {
            self.sender_count[n].fetch_sub(1, Ordering::SeqCst);
            return self.try_push(item);
        }

        /* SAFETY: we have just checked that we for sure do not push and pop
         * at the same time */
        return unsafe {
            let buf = &*self.buffers[n].get();
            buf.try_push(item)
        };
    }

    /// SAFETY: only one thread can pop
    pub unsafe fn pop(&self) -> Option<T> {
        let n = self.send_buf.load(Ordering::SeqCst);
        let n = 1 - n;

        if self.sender_count[n].load(Ordering::SeqCst) != 0 {
            return None;
        }

        /* SAFETY: sender_count[n] == 0 */
        let buf = unsafe { &mut *self.buffers[n].get() };
        if let Some(item) = buf.pop() {
            return Some(item);
        }

        /* The buffer is empty, so swap them and try again */
        drop(buf);

        self.send_buf.store(n, Ordering::SeqCst);
        let n = 1 - n;

        if self.sender_count[n].load(Ordering::SeqCst) != 0 {
            return None;
        }

        /* SAFETY: sender_count[n] == 0 */
        let buf = unsafe { &mut *self.buffers[n].get() };
        return buf.pop();
    }
}

unsafe impl<T: Send, const N: usize> Send for Mailbox<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for Mailbox<T, N> {}
