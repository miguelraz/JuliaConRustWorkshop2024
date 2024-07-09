---
marp: true
---

### Rust for Julians - Workflow, Type safety and FFI 


By Miguel Raz Guzmán Macedo
Many thanks to Ferrous Systems 2024 and to JuliaCon 2024

----

### Miguelito's credentials

* Started Rust in 2020
* part of the `portable-simd` group in the Rust compiler
* trainer/engineer at Ferrous Systems GmBH
* My job is... "just teach" 🏖️
* helped review Mara Bos's `Rust Atomics and Locks` published by O'Reilly
* ... I've suffered how to *not* learn Rust many times.

---

### Roadmap

* Saving you from Rust pain (questions and devflow)
* Stealing good type safety examples from Rust people
* FFI walkthrough and tooling

----

### Rustacean Ethos

* Be greedy
* "Make illegal states unrepresentable"
* Safety by default allows complex robust systems to be built and maintained for decades
* reduce cognitive context as much as possible -> offload thinking to tools/the compiler
* personal responsability as a design philosophy is insufficient for modern safety engineering -> blame is useless for building reliable systems

----

### Where is Rust "worth it" for Julians

* Embedded
* resource constrained environments
* critical safety
* cloud workflows hammered by Julia's bloat or startup problems
* low resource shims for databases

Kaminski principle: Least interesting part of your Julia code is the speed, but rather all the different tools you can compose to solve more interesting problems.

----

### Setup

* Who here has tried Rust?
* Who has Rust installed?
* Who is this talk for

----

### Setup

1. Install Rust via `rustup`
2. Install `rust-analyzer`
3. Demo of `rust-analyzer`

----

### Why Learning Rust for Julia people in particular is **special**:

* We already have many features that aren't a new sale: safety *and* performance through a fast GC, standard memory semantics that have mechanical sympathy coupled with a powerful JIT, a modern package manager, a unit testing framework, metaprogramming, and a documentation system.
* When reaching for Rust is justified (resource constrained environments like embedded, cloud infrastructure or data bases, etc.) Julians have to go straight for FFI, erasing many of the boons of Safe Rust and dealing very quickly with unsafe code. This is topic is **not** the promising experience for beginners in the language.

----

* Julians are used to thinking about memory footprint and layout in perf-sensitive code, as well as subtyping relations - two lynchpins for understanding the ownership and borrowsing system where the role of subtyping and variance are very commonly ommitted. This topic can therefore be explained much clearer and earlier in the curriculum.
* Materials for Rust beginners normally cater for C++ expertise (which skip error handling as philosophy) or Python/Go/Javascripters and spend too much importance on memory layouts (which Julians may know from designing faster algorithms)
* Julia has a rich generics vocabulary and a JIT that mirror Rust's trait -> monomorphize approach.

----

# Cheatsheet for Julians

* I wrote a cheatsheet for Julians to spare themselves some pain when learning Rust
* Topics: Ownership, Strings, Traits, Iterators, Error Handling
* Ownership [experiment](https://ferroussystems.hackmd.io/D7tu4b2LQn2xkBVtAbiZ_Q)

----

## Basic types

* Rust defaults to `i32` and `f64` on numeric literals, whereas Julia uses `Int`, the `Integer` pointer width on your machine. Rust uses `usize` for that, and you will have to index arrays with that type, so `x[i as usize]` will be a bane upon your code.
* As in Julia, you'll have the convenience of defining numbers with underscores without affecting parsing, i.e. `let x = 1_000_000;` is allowed.
* There's specific suffixes for primitive numeric types like `1.0_f32` or `10_u128`.
* Your numeric code will likely be sprinkled with loads of `1.0 / (n as f64)`. It's unfortunate but unavoidable.
* Wait on using generic numerics and use `i64` and `f64` until you need a really, really good reason to switch, and then you should use the `num` crate. This is because generics in Julia are invisible when done well but explicit in Rust and imply call site changes for your previously working code.
* `Char`s in Rust represent a single Unicode codepoint, which means that `'👪'` is a valid Julia `Char`, but not a Rust one. See further down for more discussions about strings. This is a key Rust ethos: "Make invalid states unrepresentable".
* Yes, indexes start at 0. Use the equal sign in `for i in 0..=10 {...}` to make an inclusive range.
* Lots of useful constants are tucked away, like `std::f64::consts::PI`. Import them with `std::f64::*;` at the top of your file.

----

## Strings

* Rust has validated UTF8 strings by default.
* Just use `String` for basically everything when starting out. 
* `String` is the owned variant, `&str` is the borrowed variant - it's easier to think that `&str` is  just "I'm getting an immutable string slice that I'm only reading from"
* Read the standard library, it very much pays off to know methods like `.split_n`, `.bytes()` and many more other stdlib functions

----

## Error Handling 

Error Handling was such a central design philosophy in Rust that it's worth knowing the context because Julia's focus didn't prioritize handling errors. I will know talk for a few paragraphs to set the stage for a simple example in simple C that I would have like to have had when I was starting Rust.

----

In the old C code bases, different failure modes for a program (or errors) had to be managed. We have studies to [support the fact that](https://sled.rs/errors) bad error handling leads to catastrophy: 

> almost all (92%) of the catastrophic system failures
> are the result of incorrect handling of non-fatal errors
> explicitly signaled in software.

----

In the world of embedded systems, systems programming or critical systems, this state of affairs is unacceptable. Imagine that we have to parse an incoming message of the format `PUBLISH your_string_here\n'. Several corner cases arise if we want to extract said string:
1. We could have no ending newline
2. We could have more than 1 ending newline
3. We could have a missing space
and so on.

----

A C codebase would only have access to structs and primitive types, so they resorted to the use of integer macros to flag failures:

```c
#define NO_ENDING_NEWLINE 1
#define TOO_MANY_NEWLINES 2
#define MISSING_SPACE 3

int parse_message(char* buf) {
    if check_ending_newline(buf) {
        return NO_ENDING_NEWLINE;
    }
    if single_ending_newline(buf) {
        return TOO_MANY_NEWLINES;
    }
    if no_space_separates_data(buf) {
        return MISSING_SPACE;
    }
    handle_message(buf);
}
```

----

Which has all sorts of sharp ends:
    * You are returning an `int` and then doing a lot of additional bit manipulation to pull out the behaviour. This becomes tedious and error-prone. This also means that you can inadvertently promote the returned int and misuse your own API silently.
    * If you ever discover a new corner case (say, presence of non-ASCII characters), you're responsible for updating at least 3 different places: a new `#define` for the new error condition, new control flow `parse_message` to handle this additional case, and, worst of all, every other call site across your codebase.

... just to name a few.

----

Compare this with the Rust approach:
```rust
enum ParseError {
    NoEndingNewLine,
    TooManyNewLines,
    MissingSpace,
}

fn parse_message(buf: &str) -> Result<String, ParseError> {
    has_ending_newline(buf)?;
    only_single_newline(buf)?;
    contains_separating_space(buf)?;
    let data: String = extract_data(buf);
    Ok(data)
}
```


Notice:
* we know that we cannot modify `buf` since it is using a shared referencd `&str`. This function therefore is *guaranteed* by the Rust type system not to allow mutation inside it's body of `buf`.
* Should we (or a tired, unfortunate coworker on another continent) extend the `ParseError` enum, then our callers will *have* to handle those new variants of corner cases. When refactoring, changes to critical data structures are all caught by the compiler and then refactoring, usually, becomes a mechanical ordeal of applying the same fix.

----

Most Rust tutorials on error handling would be glad to finish the lesson here with the "big ball of mud" enum that soaks up all the corner cases. This is not a good practice for scaling your error handling: you will lose local contexts for handling those errors once callee's have to deal with `Result`s and you make no distinction between immediate, must handle errors and errors that can be ignored. [This blog has an excellent](https://sled.rs/errors) writeup about how the Rust community keeps falling for this style due to the syntactical ease of `?` (just as people in Julia tend to overdose on dispatching everything, instead of keeping its use judicious.

A more mature version of the code would look like

----

```rust
//fn handle_message(buf: &str) -> Result<Result(), CompareAndSwapError>, Error>
let result = handle_message(buf)?;

if let Err(error) = result {
    // handle expected issue
}
```
which lets us nest `Result`s, peel them with `?`, separate local from global concerns errors, and match on exhaustive patterns in specific places. To wit:

> "Use try ? for propagating errors. Use exhaustive pattern matching on concerns you need to handle. Do not implement conversions from local concerns into global enums, or your local concerns will find themselves in inappropriate places over time. Using separate types will lock them out of where they don’t belong."

----

Our takeaway is thus:
* The C story of error handling require integer manipulation and constant error checking, where the programmer had to hold a ton of invariants in their head about what any part of the codebase could interact with any other.
* Rust's type system is ergonomic enough that facilities like `match` and friends lets us offload thinking about those invariants to the compiler and worry about more interesting things.
* **Errors will be made explicit and up front by Rust - it will not let you keep coding with unhandled errors.**

This last line is the key - Rust is not the language to let you "get away with it for now". You get a `todo!()` or an `unimplemented!()` macro at best.

* Lean on the Early return operator, which is `?` ! if you have a Result, try to use ? (you only care about Ok case anyways most of the time.)
* Learning all the transforms of `Option/Err/Result` can be much nicer if you have a [handy diagram](https://www.lurklurk.org/effective-rust/transform.html) from the Effective Rust book.
* https://docs.google.com/drawings/d/1EOPs0YTONo_FygWbuJGPfikO9Myt5HwtiFUHRuE1JVM/preview

----

### Ownership

Historical note:
Rust didn't "invent" the ownership system ex nihilo.

* Avoid indexing
* There's only 3 things: `T`, `&T`, `&mut T` when starting out.
* Ownership system and where it came from - like multiple dispatch there was an adhoc, informally spec'd... same for ownership system.
* Most of your functions should take `&T`, not `T`
* Operators are secretly funcitons, and they take references, may be created behind your back (yes, even `+=` or `==`)
* Quiz

----

### Iterators
* Examples:
    * reading lines in a file? double `filter_map`
* Uncomfy amount of `*x` stars. The `Iterator` trait has an associated type `Item`, and here `Item = &i32`, but the `filter` produces `&Item = &&i32`

* For debugging an iterator, you don't need to pepper in `dbg!` randomly, just use [.inspect()](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.inspect):

```rust
let sum = a.iter()
    .cloned()
    .inspect(|x| println!("about to filter: {x}"))
    .filter(|x| x % 2 == 0)
    .inspect(|x| println!("made it through filter: {x}"))
    .fold(0, |sum, i| sum + i);
```

which will print

```
6
about to filter: 1
about to filter: 4
made it through filter: 4
about to filter: 2
made it through filter: 2
about to filter: 3
```

### FFI

"It is a truth universally acknowledged, that a Julian in possession of a good app, must be in want of an interface to talk to some lame code in C."


----

### Syntax clashes

* `a^b` is exponentiation in Julia, XOR in Rust.
* An `array` in Rust which must be stack allocated and cannot change it's size, as it is part of its type, e.g. `[f32; 4]`, and only exists by default in the 1D case. A `vector` is a different type `Vec<f32>` and it is heap allocated.
* A `slice` in Julia look slike this `x[1:10]` and copies the array values by default. A slice in Rust is actually a type where a container's length is carried by a reference: `&[f32]`. Note that the compiler must know the size somehow - `[f32; 4]` is known as size 4 at compile time, `&[f32]` knows that the `&` carries the type at runtime and `Vec<32>` is heap allocated, and that is communicated via a `Box` type.
* `println!`, and any other function that ends with a `!` is a macro in Rust; mutation is more explicit in the type system with the `mut` keyword on a binding basis.

----

* [Cheatsheet on confusing terms](https://ferrous-systems.com/blog/cheatsheet-for-confusing-rust-terms/) - `Clone` vs `Copy` and `Debug` vs `Display`
* `move` - if you have any knowledge of C++'s move semantics, forget them! The keyword in Rust has to do with transferring ownership in Rust, not an optimization for removing containers.
* `;` is necessary for terminating a Rust expression, whereas in Julia it stops printing to the REPL. In Rust, the last expression in a function also does an implicit return, and branches that return don't need a `return x;`, just a `x` will do.
* The turbo fish `::<>` operator looks ugly as all hell but... that's actually [what it look slike](https://en.wikipedia.org/wiki/Turbot) it's useful to disambiguate callers' return type, like `my_vec.iter().map(f).collect::<i32>()`, where Rust can now know that you cant a `Vec<i32>` in the end.
* Writing `@test 0.1 + 0.2 ≈ 0.3` in Rust is done by using `assert_abs_diff_eq!(0.1, 0.2, epsilon = f64::EPSILON * 10.0;);` inside a test function.


----

## Part 2: Devflow demo


## Part 3: Type Safety and FFI