use nix::libc::_exit;
use nix::unistd::{execvp, fork, getpid, setsid, ForkResult};
use std::ffi::CString;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::config;

const KEY_STROKE_INTERVAL: u64 = 1000;
const NUM_LEADER_KEY_STROKES: usize = 3;

// refer to https://docs.rs/objc2-core-graphics/latest/src/objc2_core_graphics/generated/CGEventTypes.rs.html#171
pub const K_CG_EVENT_FLAG_MASK_OPTION_DOWN: u64 = 524576;
pub const K_CG_EVENT_FLAG_MASK_OPTION_UP: u64 = 256;

pub const K_CG_EVENT_FLAG_MASK_COMMAND_DOWN: u64 = 1048840;
pub const K_CG_EVENT_FLAG_MASK_COMMAND_UP: u64 = 256;

pub const K_CG_EVENT_FLAG_MASK_CONTROL_DOWN: u64 = 262401;
pub const K_CG_EVENT_FLAG_MASK_CONTROL_UP: u64 = 256;

pub const K_CG_EVENT_FLAG_MASK_SHIFT_DOWN: u64 = 131330;
pub const K_CG_EVENT_FLAG_MASK_SHIFT_UP: u64 = 256;

pub struct KeyStrokeRecorder {
    pub strokes: Vec<KeyStroke>,
    pub last_stroke_timestamp: Instant,
    config: config::Config,
}

#[derive(Debug)]
pub struct KeyStroke {
    pub key_code: i64,
    // pub key_typ: u32,
    pub flag: u64,
    // pub timestamp: Instant,
}

impl KeyStrokeRecorder {
    pub fn new() -> Self {
        let c = config::Config::new();
        KeyStrokeRecorder {
            strokes: vec![],
            last_stroke_timestamp: Instant::now(),
            config: c,
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
        if elapsed <= Duration::from_millis(KEY_STROKE_INTERVAL) {
            // println!("====within {}, key: {:?}", KEY_STROKE_INTERVAL, key_stroke);
            self.strokes.push(key_stroke);
        } else {
            self.strokes = vec![key_stroke];
        }

        self.last_stroke_timestamp = Instant::now();
    }

    // is_in_sequence checks if leader key is down and up
    // it needs to check the first two key strokes, firts is down, second is up
    pub fn is_in_sequence(&self) -> bool {
        if self.strokes.len() < NUM_LEADER_KEY_STROKES {
            return false;
        }

        let third_key = &self.strokes[2];
        if Self::key_code_to_name(third_key.key_code) == self.config.leader_key {
            return false;
        }

        let first_key = &self.strokes[0];
        let second_key = &self.strokes[1];

        Self::key_code_to_name(first_key.key_code) == self.config.leader_key
            && Self::key_code_to_name(second_key.key_code) == self.config.leader_key
            && Self::leader_key_down(first_key.key_code, first_key.flag)
            && Self::leader_key_up(second_key.key_code, second_key.flag)
    }

    pub fn check_sequence(&mut self) {
        if self.strokes.len() < NUM_LEADER_KEY_STROKES {
            return;
        }

        // Retrieve key strokes and match the pattern
        // let mut sequence  = Vec::new();
        let seq: Vec<_> = self.strokes.iter().map(|stroke| stroke.key_code).collect();

        let seq_array: &[i64] = &seq;
        // println!("===seq array: {:?}", seq_array);

        let key_sequence = seq_array[2..]
            .iter()
            .map(|seq| Self::key_code_to_name(*seq).to_string())
            .collect::<Vec<String>>()
            .join("");

        // println!("====key seq: {}", key_sequence);

        for group in self.config.groups.iter() {
            for mapping in group.mappings.iter() {
                if mapping.keys == key_sequence {
                    // println!("===mapping found {:?}", mapping);
                    match mapping.kind.as_str() {
                        "Application" => {
                            // Self::fork_and_exec(&mapping.command);
                            let cmd_result =
                                Command::new("open").arg("-a").arg(&mapping.command).spawn();

                            match cmd_result {
                                Ok(child) => {
                                    log::info!("App launched successfully (pid: {}).", child.id());
                                }
                                Err(err) => {
                                    log::error!("Failed to open App : {}", err);
                                }
                            }
                        }
                        "Command" => {
                            let cmd_result =
                                Command::new("sh").arg("-c").arg(&mapping.command).spawn();

                            match cmd_result {
                                Ok(child) => {
                                    log::info!("Command ran successfully (pid: {}).", child.id());
                                }
                                Err(err) => {
                                    log::error!("Failed to run command: {}", err);
                                }
                            }
                        }
                        _ => {}
                    }

                    self.strokes.clear();
                }
            }
        }
    }

    // key code mapping:
    // https://github.com/caseyscarborough/keylogger/blob/master/keylogger.c#L117
    // shift key pressing is not considered at the moment
    fn key_code_to_name(code: i64) -> &'static str {
        match code {
            0 => "a",
            1 => "s",
            2 => "d",
            3 => "f",
            4 => "h",
            5 => "g",
            6 => "z",
            7 => "x",
            8 => "c",
            9 => "v",
            11 => "b",
            12 => "q",
            13 => "w",
            14 => "e",
            15 => "r",
            16 => "y",
            17 => "t",
            18 => "1",
            19 => "2",
            20 => "3",
            21 => "4",
            22 => "6",
            23 => "5",
            24 => "=",
            25 => "9",
            26 => "7",
            27 => "-",
            28 => "8",
            29 => "0",
            30 => "]",
            31 => "o",
            32 => "u",
            33 => "[",
            34 => "i",
            35 => "p",
            36 => "return",
            37 => "l",
            38 => "j",
            39 => "'",
            40 => "k",
            41 => ";",
            42 => "\\",
            43 => ",",
            44 => "/",
            45 => "n",
            46 => "m",
            47 => ".",
            48 => "tab",
            49 => "space",
            50 => "`",
            51 => "delete",
            53 => "escape",
            55 => "command",
            56 => "shift",
            57 => "capslock",
            58 => "option",
            59 => "control",
            60 => "rightshift",
            61 => "rightoption",
            62 => "rightcontrol",
            63 => "fn",
            64 => "f17",
            65 => "keypad.",
            67 => "keypad*",
            69 => "keypad+",
            71 => "keypadclear",
            75 => "keypad/",
            76 => "keypadenter",
            78 => "keypad-",
            79 => "f18",
            80 => "f19",
            81 => "keypad=",
            82 => "keypad0",
            83 => "keypad1",
            84 => "keypad2",
            85 => "keypad3",
            86 => "keypad4",
            87 => "keypad5",
            88 => "keypad6",
            89 => "keypad7",
            90 => "f20",
            91 => "keypad8",
            92 => "keypad9",
            96 => "f5",
            97 => "f6",
            98 => "f7",
            99 => "f3",
            100 => "f8",
            101 => "f9",
            103 => "f11",
            105 => "f13",
            106 => "f16",
            107 => "f14",
            109 => "f10",
            111 => "f12",
            113 => "f15",
            114 => "help",
            115 => "home",
            116 => "pageup",
            117 => "forwarddelete",
            118 => "f4",
            119 => "end",
            120 => "f2",
            121 => "pagedown",
            122 => "f1",
            123 => "left",
            124 => "right",
            125 => "down",
            126 => "up",
            _ => "unknown",
        }
    }

    fn leader_key_up(code: i64, flag: u64) -> bool {
        match code {
            55 => flag & K_CG_EVENT_FLAG_MASK_COMMAND_UP > 0,
            56 => flag & K_CG_EVENT_FLAG_MASK_SHIFT_UP > 0,
            58 => flag & K_CG_EVENT_FLAG_MASK_OPTION_UP > 0,
            59 => flag & K_CG_EVENT_FLAG_MASK_CONTROL_UP > 0,
            _ => false,
        }
    }

    fn leader_key_down(code: i64, flag: u64) -> bool {
        match code {
            55 => flag & K_CG_EVENT_FLAG_MASK_COMMAND_DOWN > 0,
            56 => flag & K_CG_EVENT_FLAG_MASK_SHIFT_DOWN > 0,
            58 => flag & K_CG_EVENT_FLAG_MASK_OPTION_DOWN > 0,
            59 => flag & K_CG_EVENT_FLAG_MASK_CONTROL_DOWN > 0,
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn fork_and_exec(command: &String) {
        // First fork
        match unsafe { fork() }.expect("First fork failed") {
            ForkResult::Parent { child } => {
                println!("Parent continuing, first child pid = {}", child);
                // Parent returns immediately, doesn't exit
                return;
            }
            ForkResult::Child => {
                // First child continues
            }
        }

        // Detach from terminal/session
        setsid().expect("setsid failed");

        // Second fork
        match unsafe { fork() }.expect("Second fork failed") {
            ForkResult::Parent { child } => {
                // First child exits, leaving grandchild running
                println!("First child exiting, grandchild pid = {}", child);
                unsafe { _exit(0) }; // Use _exit instead of std::process::exit
            }
            ForkResult::Child => {
                // Grandchild is now fully detached
                println!("Daemon process running, pid = {}", getpid());
                let cmd = CString::new("open").unwrap();
                let arg1 = CString::new("-a").unwrap();
                let arg2 = CString::new(command.as_str()).unwrap();
                execvp(&cmd, &[cmd.clone(), arg1, arg2]).expect("execvp failed");
            }
        }
    }
}
