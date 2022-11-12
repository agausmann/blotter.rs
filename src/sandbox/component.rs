use super::ComponentBuilder;

pub struct CircuitBoard {
    color: [u8; 3],
    width: u32,
    height: u32,
}

impl CircuitBoard {
    pub fn new() -> Self {
        Self {
            color: [0x78, 0x78, 0x78],
            width: 1,
            height: 1,
        }
    }

    pub fn width(self, width: u32) -> Self {
        assert!(width > 0);
        Self { width, ..self }
    }

    pub fn height(self, height: u32) -> Self {
        assert!(height > 0);
        Self { height, ..self }
    }

    pub fn color(self, color: [u8; 3]) -> Self {
        Self { color, ..self }
    }

    pub fn build(self) -> ComponentBuilder<'static> {
        ComponentBuilder::new("MHG.CircuitBoard").custom_data(self.custom_data())
    }

    fn custom_data(&self) -> Option<Vec<u8>> {
        let mut bytes = Vec::with_capacity(11);
        bytes.extend(self.color);
        bytes.extend(self.width.to_le_bytes());
        bytes.extend(self.height.to_le_bytes());
        Some(bytes)
    }
}

impl From<CircuitBoard> for ComponentBuilder<'static> {
    fn from(board: CircuitBoard) -> Self {
        board.build()
    }
}

pub struct Delayer {
    delay: u32,
    timer: u32,
}

impl Delayer {
    pub fn new() -> Self {
        Self { delay: 2, timer: 0 }
    }

    pub fn delay(self, delay: u32) -> Self {
        Self { delay, ..self }
    }

    pub fn timer(self, timer: u32) -> Self {
        Self { timer, ..self }
    }

    pub fn build(self) -> ComponentBuilder<'static> {
        let mut custom_data = Vec::with_capacity(8);
        custom_data.extend(self.timer.to_le_bytes());
        custom_data.extend(self.delay.to_le_bytes());

        ComponentBuilder::new("MHG.Delayer")
            .num_inputs(1)
            .num_outputs(1)
            .custom_data(Some(custom_data))
    }
}

impl From<Delayer> for ComponentBuilder<'static> {
    fn from(delayer: Delayer) -> Self {
        delayer.build()
    }
}

pub struct ChubbySocket {
    _private: (),
}

impl ChubbySocket {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn build(self) -> ComponentBuilder<'static> {
        ComponentBuilder::new("MHG.ChubbySocket").num_inputs(1)
    }
}

impl From<ChubbySocket> for ComponentBuilder<'static> {
    fn from(chubby_socket: ChubbySocket) -> Self {
        chubby_socket.build()
    }
}

pub struct Peg {
    _private: (),
}

impl Peg {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn build(self) -> ComponentBuilder<'static> {
        ComponentBuilder::new("MHG.Peg").num_inputs(1)
    }
}

impl From<Peg> for ComponentBuilder<'static> {
    fn from(peg: Peg) -> Self {
        peg.build()
    }
}
