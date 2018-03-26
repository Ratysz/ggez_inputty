extern crate ggez;
#[cfg(feature = "logging")]
#[macro_use]
extern crate log;

mod input_handler;
mod macros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
