Redirect and/or gag stdout/stderr.

Documentation (with examples): https://stebalien.github.io/gag-rs/gag/

TODO:

General:
 * Windows support.
 * Better error handling?
Buffer:
 * Deallocate the buffer as it is read (FALLOC_FL_PUNCH_HOLE) if possible.
