pub mod consts;
pub mod field;
pub mod game_state;
pub mod shapes;
pub mod tetromino;
pub mod upcoming;
pub mod bitmap_field;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
