pub mod batch;
pub mod codegen;
pub mod mutators;
pub mod parser;
pub mod selector;
pub mod store;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
