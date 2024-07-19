mod ugi_engine;

use std::hash;

use camera::mouse;
use macroquad::prelude::*;
use macroquad::ui::{self, widgets, hash};

use ugi_engine::UgiEngine;


// Constants
pub const STARTING_BOARD: BoardState = [
    3, 2, 1 ,1, 2, 3,
    0 ,0 ,0, 0, 0, 0,
    0 ,0 ,0, 0, 0, 0,
    0 ,0 ,0 ,0, 0, 0,
    0 ,0, 0, 0, 0, 0,
    3 ,2 ,1 ,1, 2, 3,
    0, 0
];

pub const BOARD_WIDTH: f32 = 900.0;
pub const BOARD_HEIGHT: f32 = 900.0;
pub const BOARD_RADIUS: f32 = 450.0;

pub const GRID_WIDTH: f32 = 75.0;
pub const GRID_HEIGHT: f32 = 75.0;

pub const PIECE_RADIUS: f32 = 30.0;

pub const COLOR_BOARD: Color = Color::new(160.0/255.0, 149.0/255.0, 115.0/255.0 , 1.0); // Hex: #a09573
pub const COLOR_GRIDSPOT: Color = Color::new(175.0/255.0, 163.0/255.0, 126.0/255.0, 1.0); // Hex: #afa37e
pub const COLOR_MOVE: Color = Color::new(0.0, 1.0, 0.0, 1.0); // Lime Green

pub type Move = Vec<usize>;
pub type BoardState = [usize; 38];


#[derive(Clone)]
pub struct Piece {
    pub id: usize,
    pub pos: (f32, f32),
    pub i : usize,
    pub piece_type: usize,

}

impl Piece {
    pub fn new(pos: (f32, f32), piece_type: usize, id: usize, i: usize) -> Piece {
        return Piece {
            id,
            pos,
            i,
            piece_type,
    
        };

    }

    pub fn is_touching_point(&self, point_x: f32, point_y: f32) -> bool {
        let dx = self.pos.0 - point_x;
        let dy = self.pos.1 - point_y;
        let dist = (dx * dx + dy * dy) as f32;

        return dist < (PIECE_RADIUS * PIECE_RADIUS);

    }

    pub fn draw(&self) {
        if self.piece_type == 3 {
            draw_poly(self.pos.0, self.pos.1, 100, PIECE_RADIUS, 0., BLACK);
            draw_poly(self.pos.0, self.pos.1, 100, 25.0, 0., COLOR_BOARD);
            draw_poly(self.pos.0, self.pos.1, 100, 20.0, 0., BLACK);
            draw_poly(self.pos.0, self.pos.1, 100, 15.0, 0., COLOR_BOARD);
            draw_poly(self.pos.0, self.pos.1, 100, 10.0, 0., BLACK);
            draw_poly(self.pos.0, self.pos.1, 100, 5.0, 0., COLOR_BOARD);

        } else if self.piece_type == 2 {
            draw_poly(self.pos.0, self.pos.1, 100, PIECE_RADIUS, 0., BLACK);
            draw_poly(self.pos.0, self.pos.1, 100, 25.0, 0., COLOR_BOARD);
            draw_poly(self.pos.0, self.pos.1, 100, 20.0, 0., BLACK);
            draw_poly(self.pos.0, self.pos.1, 100, 15.0, 0., COLOR_BOARD);

        } else if self.piece_type == 1 {
            draw_poly(self.pos.0, self.pos.1, 100, PIECE_RADIUS, 0., BLACK);
            draw_poly(self.pos.0, self.pos.1, 100, 25.0, 0., COLOR_BOARD);

        }

    }

}


#[derive(Debug, PartialEq)]
pub enum Action {
    None,
    Dragging(usize),
    Dropping(usize)

}


pub struct DrawableBoard {
    boardstate: BoardState,
    prev_boardstate: Option<BoardState>,

    pieces: Vec<Piece>,

    pos: (f32, f32),
    board_pos: (f32, f32),

    action: Action,

}

impl DrawableBoard {
    pub fn new(x: f32, y: f32, boardstate: [usize; 38]) -> DrawableBoard {
        let board_pos = (x + 225.0, y + 225.0);

        let mut d_board = DrawableBoard {
            boardstate,
            prev_boardstate: Some(boardstate),

            pieces: vec![],

            pos: (x, y),
            board_pos,
            
            action: Action::None,

        };

        for i in 0..38 {
            let pos = d_board.get_pos(i);
            let piece_type = d_board.boardstate[i];

            if piece_type != 0 {
                d_board.pieces.push(Piece::new(pos, piece_type, i, i));

            }

        }

        return d_board;

    }

    pub fn boardstate_str(&self) -> String {
        let mut boardstate_str = String::new();
        for i in 0..38 {
            boardstate_str.push_str(&format!("{}", self.boardstate[i]));

        }

        return boardstate_str;

    }

    pub fn flipped_boardstate_str(&self) -> String {
        let mut boardstate_str = String::new();
        for i in (0..36).rev() {
            let piece = self.boardstate[i];
            boardstate_str.push_str(&format!("{}", piece));

        }

        boardstate_str.push_str(&format!("{}", self.boardstate[36]));
        boardstate_str.push_str(&format!("{}", self.boardstate[37]));

        return boardstate_str;

    }

    fn get_mut_piece(&mut self, id: usize) -> Option<&mut Piece> {
        for piece in self.pieces.iter_mut() {
            if piece.id == id {
                return Some(piece);

            }

        }

        None

    }

    fn get_piece(&mut self, id: usize) -> Option<&Piece> {
        for piece in self.pieces.iter() {
            if piece.id == id {
                return Some(piece);

            }

        }

        None

    }

    fn get_piece_at(&self, i: usize) -> Option<&Piece> {
        for piece in self.pieces.iter() {
            if piece.i == i {
                return Some(piece);

            }

        }

        None
    }

    fn get_nearest_snap_pos(&self, x: f32, y: f32, open: bool) -> (Option<usize>, bool) {
        let mut min_dist = 1000000000.0;
        let mut min_idx = None;
        let mut min_idx_piece = 0;
        for i in 0..38 {
            if open && self.boardstate[i] != 0 {
                continue;

            }

            let pos = self.get_pos(i);

            let dx = pos.0 - x;
            let dy = pos.1 - y;
            let dist = (dx * dx + dy * dy) as f32;
            if dist < min_dist {
                min_dist = dist;
                min_idx = Some(i);
                min_idx_piece = self.boardstate[i];

            }

        }

        return (min_idx, min_idx_piece != 0);

    }

    fn get_pos(&self, i: usize) -> (f32, f32) {
        if i == 37 {
            return (self.pos.0 + 450.0, self.pos.1 + 150.0);

        } else if i == 36 {
            return (self.pos.0 + 450.0, self.pos.1 + 750.0);
 
        }

        let x = ((i % 6) as f32 * GRID_WIDTH) + self.board_pos.0 + (GRID_WIDTH / 2.0);
        let y = ((5 - (i / 6)) as f32 * GRID_HEIGHT) + self.board_pos.1 + (GRID_HEIGHT / 2.0);

        return (x, y);

    }

    fn snap_piece(&mut self, id: usize, snap_pos: usize) {
        let pos_xy = self.get_pos(snap_pos);
                        
        if let Some(piece) = self.get_mut_piece(id) {
            piece.i = snap_pos;
            piece.pos = pos_xy;
            
        }
        if let Some(piece) = self.get_piece(id) {
            self.boardstate[piece.i] = piece.piece_type;

        }


    }

    fn moving(&mut self, id: usize) {
        let mouse_pos = mouse_position();
        if let Some(piece) = self.get_mut_piece(id) {
            piece.pos = mouse_pos;
            
        }

    }
    
    /// Returns true if the boardstate changed this frame
    pub fn update(&mut self) -> bool{
        let mouse_pos = mouse_position();

        match self.action {
            Action::None => {
                let mut state_change = false;
                if self.prev_boardstate != Some(self.boardstate) {
                    self.prev_boardstate = Some(self.boardstate.clone());

                    state_change = true;
                   
                }
                for piece in self.pieces.iter() {
                    if piece.is_touching_point(mouse_pos.0, mouse_pos.1) && is_mouse_button_pressed(MouseButton::Left) {
                        self.action = Action::Dragging(piece.id);
                        self.boardstate[piece.i] = 0;
                        break;

                    }

                }

                return state_change;

            },
            Action::Dragging(id) => {
                if is_mouse_button_released(MouseButton::Left) {
                    if let (Some(snap_pos), replace) = self.get_nearest_snap_pos(mouse_pos.0, mouse_pos.1, false) {
                        if replace {
                            if let Some(piece) = self.get_piece_at(snap_pos) {
                                self.action = Action::Dropping(piece.id);

                            }
                            
                        } else {
                            self.action = Action::None;

                        }

                        self.snap_piece(id, snap_pos);


                    }
                    

                } else {
                    self.moving(id);

                }

                return false;

            }
            Action::Dropping(id) => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    if let (Some(snap_pos), _) = self.get_nearest_snap_pos(mouse_pos.0, mouse_pos.1, true) {
                        self.snap_piece(id, snap_pos);

                    }

                    self.action = Action::None;

                } else {
                    self.moving(id);

                }

                return false;
                
            }

        }

    }

    pub fn render(&self) {
        // Board
        draw_poly(self.pos.0 + 450.0, self.pos.1 + 450.0, 4, 450.0, 0.0, COLOR_BOARD);
        
        // Radius board corners with r=50
        draw_rectangle(875.0 + self.pos.0, 0.0 + self.pos.1, 25.0, 900.0, LIGHTGRAY);
        draw_rectangle(0.0 + self.pos.0, 875.0 + self.pos.1, 900.0, 25.0, LIGHTGRAY);
        draw_rectangle(0.0 + self.pos.0, 0.0 + self.pos.1, 25.0, 900.0, LIGHTGRAY);
        draw_rectangle(0.0 + self.pos.0, 0.0 + self.pos.1, 900.0, 25.0, LIGHTGRAY);
        draw_poly(850.0 + self.pos.0, 450.0 + self.pos.1, 100, 35.355, 0., COLOR_BOARD);
        draw_poly(450.0 + self.pos.0, 850.0 + self.pos.1, 100, 35.355, 0., COLOR_BOARD);
        draw_poly(50.0 + self.pos.0, 450.0 + self.pos.1, 100, 35.355, 0., COLOR_BOARD);
        draw_poly(450.0 + self.pos.0, 50.0 + self.pos.1, 100, 35.355, 0., COLOR_BOARD);

        // Gridspots
        for i in 0..36 {
            let x = self.get_pos(i).0;
            let y = self.get_pos(i).1;

            draw_circle(x, y, 30.0, COLOR_GRIDSPOT);
            
        }
        draw_circle(self.get_pos(36).0, self.get_pos(36).1, 30.0, COLOR_GRIDSPOT);
        draw_circle(self.get_pos(37).0, self.get_pos(37).1, 30.0, COLOR_GRIDSPOT);

        // Pieces
        for piece in self.pieces.iter() {
            piece.draw();

        }

        // Draw box around whole board
        draw_rectangle_lines(self.pos.0, self.pos.1, BOARD_WIDTH, BOARD_HEIGHT, 2.0, BLACK);

        // // Draw a box around where the piece will be placed
        // if self.state != State::None {
        //     let mouse_pos = mouse_position();
        //     let (snap_pos, _) = self.get_nearest_snap_pos(mouse_pos.0, mouse_pos.1, false);
        //     if let Some(snap_pos) = snap_pos {
        //         let pos = self.get_pos(snap_pos);
        //         draw_rectangle_lines(pos.0 - 37.5, pos.1 - 37.5, 75.0, 75.0, 2.0, BLACK);

        //     }

        // }

    }

    pub fn render_move(&mut self, mv: Move, color: Color) {
        for i in 0..mv.len() -1 {
            self.render_arrow(mv[i], mv[i+1], color);

        }

    }

    fn render_arrow(&mut self, boardpos_1: usize, boardpos_2: usize, color: Color) {
        let xy_pos_1 = self.get_pos(boardpos_1);
        let xy_pos_2 = self.get_pos(boardpos_2);

        draw_line(xy_pos_1.0, xy_pos_1.1, xy_pos_2.0, xy_pos_2.1, 2.5, color);
        draw_circle(xy_pos_2.0, xy_pos_2.1, 5.0, color)

    }

    pub fn reset(&mut self) {
        let new = DrawableBoard::new(self.pos.0, self.pos.1, STARTING_BOARD);
        
        self.boardstate = new.boardstate;
        self.prev_boardstate = None;

        self.pieces = new.pieces;

        self.pos = new.pos;
        self.board_pos = new.board_pos;

        self.action = new.action;


    }

    pub fn flip(&mut self) {
        let mut flipped_boardstate = [0; 38];

        for i in 0..36 {
            let piece = self.boardstate[i];
            if piece == 0 {
                continue;

            }

            let flipped_i = 35 - i;
            flipped_boardstate[flipped_i] = piece;

        }


        let new = DrawableBoard::new(self.pos.0, self.pos.1, flipped_boardstate);
        
        self.boardstate = new.boardstate;
        self.pieces = new.pieces;
        self.action = new.action;

    }

}

pub fn start_new_seach(engine: &mut UgiEngine, drawable_board: &mut DrawableBoard) {
    if engine.enabled {
        engine.send("stop");

        let setcmd = if engine.side == 1.0 {
            format!("setpos data {}", drawable_board.boardstate_str())
        } else {
            format!("setpos data {}", drawable_board.flipped_boardstate_str())
        };
    
        engine.send(setcmd.as_str());
    
        engine.send("go");

    }

}

pub fn parse_bestmove_str(raw_move: &str) -> Move {
    let raw_mv_data: Vec<&str> = raw_move.split("|").collect();

    let mut mv = vec![];
    for i in 0..raw_mv_data.len() {
        mv.push(raw_mv_data[i].parse::<usize>().unwrap());

    }

    return mv;

}

pub fn parse_info_str(info_str: &str, engine_side: f64) -> SearchInfo {
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
                    let best_move = if engine_side == 1.0 {
                        parse_bestmove_str(group[1])

                    } else {
                        flip_move(parse_bestmove_str(group[1]))

                    };

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

pub struct SearchSettings {
    pub engine_side: f32,
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

fn window_conf() -> Conf {
    Conf {
        window_title: "gyges ui".to_owned(),
        window_height: 900,
        window_width: 1250,
        window_resizable: false,
        ..Default::default() 
    }

}
#[macroquad::main(window_conf)]
async fn main() {
    prevent_quit();

    let mut engine = UgiEngine::new("C:/Users/beckb/Documents/GitHub/GygesRust/target/release/gyges_engine.exe");
    engine.send("ugi");

    let mut best_search: SearchInfo = SearchInfo::new();

    let mut drawable_board = DrawableBoard::new(0.0, 0.0, STARTING_BOARD);

    start_new_seach(&mut engine, &mut drawable_board);

    loop {
        clear_background(LIGHTGRAY);

        if is_quit_requested() {
            engine.send("stop");
            engine.send("quit");
            break;

        }

        // Draw UI
        widgets::Window::new(hash!(), Vec2 { x: 950.0, y: 50.0 }, Vec2 { x: 250.0, y: 100.0 })
            .label("Board Controls")
            .titlebar(true)
            .movable(false)
            .ui(&mut ui::root_ui(), |ui| {
                ui.separator();
                if ui.button(None, "Reset") {
                    drawable_board.reset();

                }
                ui.separator();
                if ui.button(None, "Flip Board") {
                    drawable_board.flip();
                    engine.side *= -1.0;

                }

            });

        widgets::Window::new(hash!(), Vec2 { x: 950.0, y: 175.0}, Vec2 { x: 250.0, y: 125.0 })
            .label("Engine Controls")
            .titlebar(true)
            .movable(false)
            .ui(&mut ui::root_ui(), |ui| {
                ui.separator();
                if ui.button(None, "Stop") {
                    engine.send("stop");
                    engine.enabled = false;

                }
                ui.separator();
                if ui.button(None, "Start") {
                    engine.enabled = true;
                    start_new_seach(&mut engine, &mut drawable_board);

                }
                ui.separator();
                if ui.button(None, "Change Player") {
                    engine.side *= -1.0;
                    start_new_seach(&mut engine, &mut drawable_board);

                }
                
            });
            
        widgets::Window::new(hash!(), Vec2 { x: 950.0, y: 325.0 }, Vec2 { x: 250.0, y: 250.0 })
            .label("Search Info")
            .titlebar(true)
            .movable(false)
            .ui(&mut ui::root_ui(), |ui| {
                if let Some(ply) = &best_search.ply {
                    ui.label(None, format!("Ply: {}", ply).as_str());

                }
                if let Some(score) = &best_search.score {
                    ui.label(None, format!("Score: {}", score).as_str());

                }
                if let Some(best_move) = &best_search.best_move {
                    ui.label(None, format!("Best Move: {:?}", best_move).as_str());

                }
                if let Some(nodes) = &best_search.nodes {
                    ui.label(None, format!("Nodes: {}", nodes).as_str());

                }
                if let Some(nps) = &best_search.nps {
                    ui.label(None, format!("NPS: {}", nps).as_str());

                }
                if let Some(abf) = &best_search.abf {
                    ui.label(None, format!("ABF: {}", abf).as_str());

                }
                if let Some(beta_cuts) = &best_search.beta_cuts {
                    ui.label(None, format!("Beta Cuts: {}", beta_cuts).as_str());

                }
                if let Some(time) = &best_search.time {
                    ui.label(None, format!("Time: {}", time).as_str());

                }

            });
        
        
        // Update and render board
        if drawable_board.update() { 
            start_new_seach(&mut engine, &mut drawable_board); 
        };
        drawable_board.render();

        // Render best move
        if engine.enabled {
            if best_search.best_move.is_some() {
                drawable_board.render_move(best_search.best_move.clone().unwrap(), COLOR_MOVE);

            }

        } else {
            best_search = SearchInfo::new();

        }
        
        // Recive data from engine
        let r: Option<String> = engine.recive();
        if let Some(r) = r {
            let cmds = r.split_whitespace().collect::<Vec<&str>>();
            if cmds.get(0) == Some(&"info") {
                best_search = parse_info_str(r.as_str(), engine.side);

            }
            
        }

        next_frame().await;

    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    drop(engine);

}
