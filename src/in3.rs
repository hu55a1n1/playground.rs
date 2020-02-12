// custom allocator
// stream parsing
// no_std
// generics for func ptrs

trait DataStore {
    fn get(key: &str) -> Option<&[u8]>;
    fn set(key: &str, val: &[u8]) -> Result<(), Error>;
    fn rm(key: &str) -> Result<(), Error>;
    fn clr() -> bool;
}

struct Chain {}

struct Client {}
