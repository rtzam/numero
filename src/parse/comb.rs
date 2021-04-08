



// trait Syntax{
//     type Parsed;
//     fn expect() -> Self::Parsed;     
// }

// trait Delim{
//     type Parsed;
//     fn expect_open() -> Self::Parsed;
//     fn expect_close() -> Self::Parsed;
// }


// struct Delimited<D: Delim, S: Syntax>{
//     delim: D,
//     body: S,
// }

// impl<D: Delim, S: Syntax> Delimited<D,S>{
//     fn new(d: D, s: S) -> Self{
//         Self{
//             delim: d,
//             body: s,
//         }
//     }
// }