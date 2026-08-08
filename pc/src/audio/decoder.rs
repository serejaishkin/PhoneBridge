use opus::Decoder;

pub struct OpusDecoder {
    decoder: Decoder,
}

impl OpusDecoder {
    pub fn new() -> Result<Self, opus::Error> {
        let decoder = Decoder::new(48000, opus::Channels::Mono)?;
        Ok(Self { decoder })
    }

    pub fn decode(&mut self, input: &[u8], output: &mut [i16]) -> Result<usize, opus::Error> {
        self.decoder.decode(input, output, false)
    }
}
