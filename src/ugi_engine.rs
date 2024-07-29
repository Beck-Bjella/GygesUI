use std::process::{Command, Stdio, ChildStdout, ChildStdin};
use std::io::{self, Write, Read};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, Sender};
use std::collections::VecDeque;

use crate::{DrawableBoard, Move};

pub const MAX_PLY: f32 = 99.0; // moves
pub const MAX_TIME: f32 =  3600.0; // seconds

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Disabled,
    Analysis,
    Auto,
    Single

}


pub struct UgiEngine {
    pub mode: Mode,
    pub searching: bool,
    pub side: f64,

    pub best_search: SearchInfo,

    pub p1_settings: SearchSettings,
    pub p2_settings: SearchSettings,

    input_sender: Sender<String>,
    ouput_reciver: Receiver<String>,

    reader_thread: Option<JoinHandle<()>>,
    reader_quit_sender: Sender<bool>,

    writer_thread: Option<JoinHandle<()>>,
    writer_quit_sender: Sender<bool>,

    recived_queue: VecDeque<String>,

}

impl UgiEngine {
    pub fn new(engine_path: &str) -> UgiEngine {
        let mut engine_process = Command::new(engine_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start the process");

        let stdout = engine_process.stdout.take().unwrap();
        let stdin = engine_process.stdin.take().unwrap();

        let (input_sender, input_reciver) = mpsc::channel();
        let (ouput_sender, ouput_reciver) = mpsc::channel();

        let (quit_sender_1, quit_reciver_1) = mpsc::channel();
        let (quit_sender_2, quit_reciver_2) = mpsc::channel();

        let reader_thread = thread::spawn(move || {
            let mut reader = UgiReader::new(ouput_sender, quit_reciver_1);
            reader.start(stdout);

        });
        let writer_thread = thread::spawn(move || {
            let mut writer = UgiWriter::new(input_reciver, quit_reciver_2);
            writer.start(stdin);

        });

        return UgiEngine {
            mode: Mode::Disabled,
            searching: false,
            side: 1.0,

            best_search: SearchInfo::new(),

            p1_settings: SearchSettings {
                max_ply: MAX_PLY as f32,
                max_time: MAX_TIME as f32,

            },
            p2_settings: SearchSettings {
                max_ply: MAX_PLY as f32,
                max_time: MAX_TIME as f32,

            },

            input_sender,
            ouput_reciver,

            reader_thread: Some(reader_thread),
            reader_quit_sender: quit_sender_1,

            writer_thread: Some(writer_thread),
            writer_quit_sender: quit_sender_2,

            recived_queue: VecDeque::new(),
  
        };

    }

    pub fn send(&mut self, cmd: &str) {
        self.input_sender.send(cmd.to_string()).expect("Failed to send data to the engine");
        println!("SENT: {}", cmd);

    }

    fn try_recive(&mut self) {
        match self.ouput_reciver.try_recv() {
            Ok(s) => {
                println!("RECIVED: {}", s.clone());
                self.recived_queue.push_front(s.clone());

            }
            Err(_) => {}

        }

    }

    pub fn recive(&mut self) -> Option<String> {
        self.try_recive();
        
        if self.recived_queue.len() == 0 {
            return None;

        }
        return self.recived_queue.pop_back();
    
    }

    // ========================

    pub fn flip_side(&mut self) {
        self.side *= -1.0;

    }   

    pub fn set_side(&mut self, side: f64) {
        self.side = side;

    }

    pub fn stop(&mut self) {
        self.send("stop");
        self.mode = Mode::Disabled;
        self.searching = false;

    }

    pub fn quit(&mut self) {
        self.stop();
        self.send("quit");

        self.reader_quit_sender.send(true).unwrap();
        self.writer_quit_sender.send(true).unwrap();

        self.reader_thread.take().unwrap().join().unwrap();
        self.writer_thread.take().unwrap().join().unwrap();

    }

    pub fn new_search(&mut self, search_purpose: Mode, drawable_board: &DrawableBoard) {
        if self.searching {
            self.send("stop");
            self.wait_for_search();

        }

        let setcmd = match self.side {
            1.0 => { format!("setpos data {}", drawable_board.boardstate_str()) },
            _ => { format!("setpos data {}", drawable_board.flipped_boardstate_str()) },
        };
        self.send(setcmd.as_str());

        let maxtime_cmd = match self.side {
            1.0 => { format!("setoption max_time {}", self.p1_settings.max_time) },
            _ => { format!("setoption max_time {}", self.p2_settings.max_time) },

        };
        self.send(maxtime_cmd.as_str());

        let maxply_cmd = match self.side {
            1.0 => { format!("setoption max_ply {}", self.p1_settings.max_ply) },
            _ => { format!("setoption max_ply {}", self.p2_settings.max_ply) },

        };
        self.send(maxply_cmd.as_str());

        self.send("go");
    
        self.mode = search_purpose;
        self.searching = true;
    
    }

    pub fn wait_for_search(&mut self) {
        loop {
            self.try_recive();
            if self.recived_queue.len() == 0 {
                continue;

            }

            let line = self.recived_queue.pop_back().unwrap();
            if line.starts_with("bestmove") {
                break;

            }

        }

    }
    
    pub fn update(&mut self, drawable_board: &mut DrawableBoard) {
        if self.mode == Mode::Disabled {
            self.best_search = SearchInfo::new();

        }
        if drawable_board.game_over() && (self.searching || self.mode != Mode::Disabled) {
            self.stop();
            return;

        }
        
        let recived: Option<String> = self.recive();
        if let Some(data) = recived {
            let cmds = data.split_whitespace().collect::<Vec<&str>>();
            match cmds.get(0) {
                Some(&"bestmove") => {
                    self.searching = false;

                    match self.mode {
                        Mode::Single => {
                            let best_move = self.parse_bestmove_str(cmds.get(1).unwrap());

                            drawable_board.make_move(best_move);
                            self.stop();
   
                        },
                        Mode::Auto => {
                            let best_move = self.parse_bestmove_str(cmds.get(1).unwrap());

                            drawable_board.make_move(best_move);
    
                            if drawable_board.game_over() {
                                self.stop();
    
                            } else {
                                self.flip_side();
                                self.new_search(Mode::Auto, drawable_board);
                                std::thread::sleep(std::time::Duration::from_millis(500)); // Min delay between moves - freezes app temporarily

                            }
                                
                        },
                        _ => {}

                    }

                },
                Some(&"info") => {
                    self.best_search = self.parse_info_str(data.as_str());

                },
                _ => {}

            }
            
        }

    }

    // ========================
    
    pub fn parse_bestmove_str(&self, raw_move: &str) -> Move {
        let raw_mv_data: Vec<&str> = raw_move.split("|").collect();

        let mut mv = vec![];
        for i in 0..raw_mv_data.len() {
            mv.push(raw_mv_data[i].parse::<usize>().unwrap());

        }

        if self.side == -1.0 {
            return flip_move(mv);

        }
        return mv;

    }

    pub fn parse_info_str(&self, info_str: &str) -> SearchInfo {
        let mut search_info = SearchInfo::new();

        let mut raw_cmds: Vec<&str> = info_str.split_whitespace().collect();
        if raw_cmds.get(0) == Some(&"info") {
            raw_cmds.remove(0);

            let cmd_groups = raw_cmds.chunks(2).map(|x| [x[0], x[1]]).collect::<Vec<[&str; 2]>>();
            for group in cmd_groups {
                match group[0] {
                    "ply" => {
                        let ply = group[1].parse::<f64>().unwrap();
                        search_info.ply = Some(ply);

                    },
                    "bestmove" => {
                        let best_move = self.parse_bestmove_str(group[1]);
                        search_info.best_move = Some(best_move);

                    },
                    "score" => {
                        let score = group[1].parse::<f64>().unwrap();
                        search_info.score = Some(score);

                    },
                    "nodes" => {
                        let nodes = group[1].parse::<f64>().unwrap();
                        search_info.nodes = Some(nodes);

                    },
                    "nps" => {
                        let nps = group[1].parse::<f64>().unwrap();
                        search_info.nps = Some(nps);

                    },
                    "abf" => {
                        let abf = group[1].parse::<f64>().unwrap();
                        search_info.abf = Some(abf);

                    },
                    "beta_cuts" => {
                        let beta_cuts = group[1].parse::<f64>().unwrap();
                        search_info.beta_cuts = Some(beta_cuts);

                    },
                    "time" => {
                        let time = group[1].parse::<f64>().unwrap();
                        search_info.time = Some(time);

                    },
                    _ => {}
                
                }

            }

            return search_info;

        }

        return SearchInfo::new();

    }

}

// Helper function to flip a move
pub fn flip_move(mv: Move) -> Move {
    let mut flipped_mv = vec![];
    for i in 0..mv.len() {
        if mv[i] == 37 {
            flipped_mv.push(36);
            continue;

        } else if mv[i] == 36 {
            flipped_mv.push(37);
            continue;

        }

        flipped_mv.push(35 - mv[i]);

    }

    return flipped_mv;


}

struct UgiReader {
    data_out: Sender<String>,
    quit_in: Receiver<bool>,

}

impl UgiReader {
    pub fn new(data_out: Sender<String>, quit_in: Receiver<bool>) -> UgiReader {
        return UgiReader {
            data_out,
            quit_in

        };
    
    }

    pub fn start(&mut self, stdout: ChildStdout) {
        let mut stdout_reader: io::BufReader<std::process::ChildStdout> = io::BufReader::new(stdout);
        let mut stdout_buffer: [u8; 4096] = [0; 4096]; 

        loop {
            match stdout_reader.read(&mut stdout_buffer) {
                Ok(n) => {
                    let output = String::from_utf8_lossy(&stdout_buffer[..n]).to_string();
                    for l in output.lines() {
                        self.data_out.send(l.to_string()).unwrap();

                    }

                }
                Err(err) => {
                    eprintln!("Error reading from stdout: {}", err);
                    break;
                    
                }

            }

            match self.quit_in.try_recv() {
                Ok(_) => {
                    break;

                }
                Err(_) => {}

            }

        }

    }
    
}

struct UgiWriter {
    data_in: Receiver<String>,
    quit_in: Receiver<bool>,

}

impl UgiWriter {
    pub fn new(data_in: Receiver<String>, quit_in: Receiver<bool>) -> UgiWriter {
        return UgiWriter {
            data_in,
            quit_in

        };
    
    }

    pub fn start(&mut self, mut stdin: ChildStdin) {
        loop {
            match self.data_in.try_recv() {
                Ok(s) => {
                    stdin.write_all(format!("{}\n", s).as_bytes()).unwrap();

                }
                Err(_) => {}

            }

            match self.quit_in.try_recv() {
                Ok(_) => {
                    break;

                }
                Err(_) => {}

            }

        }

    }
    
}



pub struct SearchSettings {
    pub max_ply: f32,
    pub max_time: f32,

}

#[derive(Debug)]
pub struct SearchInfo {
    pub ply: Option<f64>,
    pub best_move: Option<Move>,
    pub score: Option<f64>,
    pub nodes: Option<f64>,
    pub nps: Option<f64>,
    pub abf: Option<f64>,
    pub beta_cuts: Option<f64>,
    pub time: Option<f64>,

}
impl SearchInfo {
    pub fn new() -> SearchInfo {
        return SearchInfo {
            ply: None,
            best_move: None,
            score: None,
            nodes: None,
            nps: None,
            abf: None,
            beta_cuts: None,
            time: None,

        };

    }

}