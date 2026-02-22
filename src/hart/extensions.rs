pub mod f;
pub mod m;
pub mod d;

#[allow(dead_code, unused_variables)]
#[derive(Debug)]
pub struct Extensions {
    pub m: bool,
    pub a: bool,
    pub c: bool,
    pub f: bool,
    pub d: bool,
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

impl Extensions {

    pub fn new() -> Extensions {
        Extensions { m: false, a: false, c: false, f: false, d: false }
    }
}
