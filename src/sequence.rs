use std::error;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};

const KEY_STROKE_INTERVAL: u64 = 1000;

pub struct KeyStrokeRecorder {
    pub strokes: Vec<KeyStroke>,
    pub last_stroke_timestamp: Instant,
}

#[derive(Debug)]
pub struct KeyStroke {
    pub key_code: i64,
    pub key_typ: u32,
    pub timestamp: Instant,
}

impl KeyStrokeRecorder {
    pub fn new() -> Self {
        KeyStrokeRecorder {
            strokes: vec![],
            last_stroke_timestamp: Instant::now(),
        }
    }

    pub fn record(&mut self, key_stroke: KeyStroke) {
        // If key stroke timestamp is within the threshold, record it
        // otherwise reset the strokes
        // First check if leader(modifier) key
        // If yes, record it, otherwise no
        //
        // If leader key is hit, don't forward the key strokes

        let elapsed = self.last_stroke_timestamp.elapsed();
        // println!("===elapsed: {:?}", elapsed);
        if elapsed <= Duration::from_millis(KEY_STROKE_INTERVAL) {
            // println!("====within {}, key: {:?}", KEY_STROKE_INTERVAL, key_stroke);
            self.strokes.push(key_stroke);
        } else {
            self.strokes = vec![key_stroke];
        }

        self.last_stroke_timestamp = Instant::now();
    }

    pub fn check_sequence(&self) {
        // Retrieve key strokes and match the pattern
        // let mut sequence  = Vec::new();
        let seq: Vec<_> = self.strokes.iter().map(|stroke| stroke.key_code).collect();

        let seq_array: &[i64] = &seq;
        println!("===seq array: {:?}", seq_array);

        // key code mapping:
        // https://github.com/caseyscarborough/keylogger/blob/master/keylogger.c#L117
        match seq_array {
            [58, 31, 8] => {
                println!("===alt o c= Open Chrome");
                let cmd = Command::new("open")
                    .arg("-a")
                    .arg("Google Chrome")
                    .spawn();

                match cmd {
                    Ok(child) => {
                        log::info!("App launched successfully (pid: {}).", child.id());
                    }
                    Err(err) => {
                        log::error!("Failed to open App : {}", err);
                    }
                }
            }
            _ => {}
        }
    }
}
