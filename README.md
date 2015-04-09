Redirect and/or gag stdout/stderr.

Documentation (with examples): https://stebalien.github.io/gag-rs/gag/

TODO:

General:
 * Windows support.
 * Better error handling?

Redirect:
 * Be generic over references. That is, accept both a reference to an AsRawFd or
   an AsRawFd. Unfortunately, I don't know if this is even possible. Borrow
   doesn't work because I really want the following constraint:
   `impl<F> Redirect<F> where F: BorrowMut<T>, T: AsMut<AsRawFd>` so I can write
   `file.borrow_mut().as_mut()` but that would be ambiguous...

Buffer:
 * Deallocate the buffer as it is read (FALLOC_FL_PUNCH_HOLE) if possible.
