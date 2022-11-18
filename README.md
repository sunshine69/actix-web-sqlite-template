# actix-web-sqlite-template

This is a small template for a web microservice using rust - actix-web and pure sqlite.

It ilustrate the common route and handler and request data extraction that actix support.

Just copy the whole dir into the target prject and start adding more handlers, and enjoy.

I might add a branch later to use `diesel`.

# Why bother?

- Actix-web is one of the fastest web framework on earth (base on a lot of benchmark)
- `Rust` admit that you spend time fighting with the compiler well, you will be rewarded with happiness and nearly bug free code
- You will be able to write code that other look and say `wow` it is cool stuff.
- The next fight will be pretty small if you get used in thinking `Rusty`. :P

# UPDATE

Actix is not really nice. The upgrade from version 3 => 4 is done, app complies but when
request go to - eg. /savelog it return 404 not found.

But there is nothing wrong in code. So abandon actix for now.

In theory after all touted goodies about mordern languages like Rust, if it compiles then it should at least work or return correct error for debugging.

They all are myth!