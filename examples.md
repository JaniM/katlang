Check out the [spec](spec.md)

- Fibonacci sequence

  `1:50#p;+x$~ J`

  1. `1:` initialize the stack with two ones (`:` copies the top item)
  2. `50#p;+x$` repeat the block (`p;+x`) 50 times.
     1. `p` push the top item to the side stack, non-destructively
     2. `;` duplicate the second item in the stack
     3. `+` add top two items
     4. `x` swap top two items
  3. `~` pop the entire side stack as a stack frame to the main stack
  4. `J` join all elements with a space
  5. The top item of the stack is implicitly printed at the end
