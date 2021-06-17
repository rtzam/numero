# Nosh

A programming language to experiment with compiler development


## Current Status
* Kaleidoscope complete
  * only `f64` type and functions
  * JIT execution, Obj file creation

```rust
mod module_name

// libc functions provided by LLVM
extern fun sin(x)
extern fun cos(x)

// define a function
fun let_binding()
    // const variable declarations
    let
        x = 3.14159265359
        y = x * 2
    in
        sin(x/2) + cos(y)
    end
end

// program entry point
fun main()
    let_binding()
end

// Arguments are constant
fun cumulative(iters)
    val mut count = 0
    val mut result = 0 
    
    while count < iters
        // use = instead of :=
        // for mutation
        result = result + count
        count = count + 1
    end

    result
end

fun branches(first, second)
    
    val clamped_first = if first < 0
        0
    else if 10 < first 
        10
    else
        first
    end

    val new_second = do // do-block creates new scope
        val a_temp = clamped_first * second
        a_temp + cumulative(second)
    end

    // ternary operator
    val const_term = 1.5 if first < second else 0

    // multiline expressions when
    // line ends with a binary op
    const_term + 
        clamed_first + 
        new_second
end
```


## Future Direction
* Parser error recovery
* support basic types
  * i64, char, Nil
* Type inference
* import other modules
* export public functions