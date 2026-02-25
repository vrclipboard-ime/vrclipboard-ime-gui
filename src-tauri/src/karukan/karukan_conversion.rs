pub struct KarukanConversion {
    pub history: Vec<String>,
}

impl KarukanConversion {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }
}
