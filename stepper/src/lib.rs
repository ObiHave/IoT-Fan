
struct Easydriver {
    gpio_dir: u8,
    gpio_step: u8,
    gpio_slp: u8,
    clockwise: bool,
    steps_per_rev: u16,
}
impl Easydriver {
    pub fn new() -> Result<()> {
        Ok(Easydriver {
            gpio_dir: 
        })
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
