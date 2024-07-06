use std::fs::File;
use std::io::BufReader;
use rodio::{source::Source, source::Buffered,  Decoder, OutputStream, Sink};

type BufferedWav = Buffered<Decoder<BufReader<File>>>;

pub struct AudioHandler {
    sounds: Vec<Option<BufferedWav>>,
    _stream: OutputStream,
    sinks: [Option<Sink>; 9],
}

impl AudioHandler {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut sounds: Vec<Option<BufferedWav>> = vec![];
        let mut sinks: [Option<Sink>; 9] = Default::default();
        for i in 0..9 {
            let file = File::open(format!("{}.wav", i));
            if file.is_ok() {
                let file = BufReader::new(file.unwrap());
                let source = Decoder::new(file).unwrap();
                sounds.push(Some(source.buffered()));
                sinks[i] = Some(Sink::try_new(&stream_handle).unwrap());
            } else {
                sounds.push(None);
                sinks[i] = None;
            }
        }
        Self { sounds, _stream: stream, sinks, }
    }

    pub fn play_sound(&mut self, sound: usize) {
        match &self.sounds[sound] {
            Some(x) => {
                match &self.sinks[sound] {
                    Some(s) => {
                        if s.empty() {
                            s.append(x.clone());
                        }
                    }
                    None => {}
                }
            },
            None => {}
        }
    }
}