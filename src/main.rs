#![allow(dead_code)]

use macroquad::prelude::*;
use std::collections::VecDeque;

const MAP: usize = 20;
const T_SIZE: (f32, f32) = ( 32., 16.);

enum AppState {
    Menu,
    Playing,
    GameOver
}

#[derive(Copy, Clone, PartialEq)]
enum Tile {
    Wall,
    Floor
}

struct Monster {
    x: usize, 
    y: usize,
    hp: i32,
    cd: f32,
}

struct FloatingText {
    x: f32,
    y: f32,
    damage: i32,
    text_life: f32,
}

fn to_screen( x: usize, y: usize, cam: (f32, f32)) -> (f32, f32) {
    (
        ( x as f32 - y as f32 ) * T_SIZE.0 + cam.0,
        ( x as f32 + y as f32 ) * T_SIZE.1 + cam.1
    )
}

fn to_tile(sx: f32, sy: f32, cam: (f32, f32)) -> (usize, usize) {
    let (ax, ay) = ( sx - cam.0, sy -cam.1 );
    (
        ((ax / T_SIZE.0 + ay / T_SIZE.1)/ 2.) as usize,
        ((ay / T_SIZE.1 - ax / T_SIZE.0)/ 2.) as usize,
    )
}   

fn calculate_distance(p1: (usize, usize), p2: (usize, usize)) -> i32 {
    (p1.0 as i32 - p2.0 as i32).abs() + (p1.1 as i32 - p2.1 as i32).abs()
}

// pathfinding
fn bfs(map: &[[Tile;MAP]; MAP], start: (usize, usize), goal: (usize, usize)) -> Vec<(usize, usize)> {
    let mut q = VecDeque::from([start]);
    let mut visited = [[false; MAP]; MAP];
    visited[start.1][start.0] = true;

    let mut parent = [[None;MAP]; MAP];

    while let Some(curr) = q.pop_front() {
        if curr == goal {
            let mut path = vec![];
            let mut c = goal;

            while c != start {
                path.push(c);
                c = parent[c.1][c.0].unwrap();
            }
            path.reverse();
            return path;
        }

        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = ((curr.0 as i32 + dx) as usize, (curr.1 as i32 + dy) as usize);

            if nx < MAP && ny < MAP && !visited[ny][nx] && map[ny][nx] == Tile::Floor {
                visited[ny][nx] = true;
                parent[ny][nx] = Some(curr);
                q.push_back((nx, ny));
            }
        }
    }
    vec![]
}

// hero and monsters
fn draw_stickman(x: usize, y: usize, cam: (f32, f32), is_monster: bool){
    let (sx, mut sy) = to_screen(x, y, cam);
    sy += 16.;
    // shadow 
    draw_ellipse(sx, sy + 3., 10., 5., 0.1, Color::new(0., 0., 0., 0.2));
    // head
    if is_monster {
        draw_line(sx - 5., sy - 32., sx, sy - 30.,2., BLACK);
        draw_line(sx + 5., sy - 32., sx, sy - 30.,2., BLACK);

    } else {
        draw_circle_lines(sx, sy - 32., 7., 2., BLACK);
    }
    
    // body and limbs
    for l in [
        [0., -25., 0., -8.],
        [0., -20., -8., -15.],
        [0., -20., -8., -15.],
        [0., -8., -6., -0.],
        [0., -8., 6., -0.],
    ] {
        draw_line(sx + l[0], sy + l[1], sx + l[2], sy + l[3], 2., BLACK);
    }
}

fn draw_wall( x: usize, y: usize, cam: (f32, f32)) {
    let (sx, sy) = to_screen(x, y, cam);
    let v = [
        vec2(sx, sy - 40.),
        vec2(sx + 32., sy - 24.),
        vec2(sx, sy - 8.),
        vec2(sx - 32., sy - 24.),
        vec2(sx + 32., sy),
        vec2(sx, sy + 16.),
        vec2(sx - 32.,sy, ),
    ];

    let colors = [
        Color::new(0.8, 0.8, 0.8, 1.),
        Color::new(0.5, 0.5, 0.5, 1.),
        Color::new(0.6, 0.6, 0.6, 1.),

    ];

    // faces
    draw_triangle(v[0], v[1], v[2], colors[0]);
    draw_triangle(v[0], v[2], v[3], colors[0]);
    draw_triangle(v[1], v[4], v[5], colors[1]);
    draw_triangle(v[1], v[5], v[2], colors[1]);
    draw_triangle(v[3], v[2], v[5], colors[2]);
    draw_triangle(v[3], v[5], v[6], colors[2]);

    // outline
    for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (2, 5), (3, 6),] {
        draw_line(v[a].x, v[a].y, v[b].x, v[b].y, 1., BLACK);
    }
}
struct Game {
    map: [ [ Tile; MAP]; MAP ],
    cam: (f32, f32),
    px: usize,
    py: usize,
    path: Vec<(usize, usize)>,
    player_cooldown: f32,
    monsters: Vec<Monster>,
    texts: Vec<FloatingText>,   
    hp: i32,
    gold: Vec<(usize, usize)>,
    score: i32,
}

impl Game {
    fn new() -> Self {
        let mut map = [ [Tile::Floor; MAP]; MAP];

        for i in 0..MAP {
            map[0][i] = Tile::Wall;
            map[MAP - 1][i] = Tile::Wall;
            map[i][0] = Tile::Wall;
            map[i][MAP - 1] = Tile::Wall;
        }


        for (x, y) in [ (5, 5), (6, 5), (12, 10) ] {
            map[x][y] = Tile::Wall;
        }

        Self {
            map,
            cam: ( screen_width() / 2., 50.),
            px: 2,
            py: 2,
            path: vec![],
            player_cooldown: 0.,
            monsters: vec![
                Monster { x: 8, y: 8, hp: 20, cd: 0. },
                Monster { x: 13, y: 4, hp: 20, cd: 0. },
                Monster { x: 10, y: 8, hp: 20, cd: 0. },
                Monster { x: 5, y: 10, hp: 20, cd: 0. },
            ],
            texts: vec![],
            hp: 100,
            gold: vec![(3,4), (11,4), (5,10), (13,14), (14,16) ],
            score: 0,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        // if is_key_pressed(KeyCode::Space) {
        //     return true;
        // }

        if self.hp <= 0 || self.monsters.is_empty() {
            return true
        }

        // udate text animations
        self.texts.retain_mut( | text| {
            text.text_life -= dt;
            text.y -= 20. * dt;
            text.text_life > 0.
        });
        
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (tx, ty) = to_tile(mx, my, self.cam);

            if tx < MAP && ty < MAP && self.map[ty][tx] == Tile::Floor {
                self.path = bfs(&self.map, (self.px, self.py), (tx, ty));
            }
        }


        if !self.path.is_empty() {
            self.player_cooldown -= dt;

            if self.player_cooldown <= 0. {
                self.player_cooldown = 0.15;

                // let next_step = self.path[0];
                // self.px = next_step.0;
                // self.py = next_step.1;

                // self.path.remove(0);

                let (nx, ny) = self.path[0];

                if let Some(i) = self.monsters.iter().position(|m| m.x == nx && m.y == ny) {
                    // attack
                    self.damage_monster(i, 10);

                    self.path.clear();
                } else {
                    // move

                    self.path.remove(0);
                    self.px = nx;
                    self.py = ny;

                }

                // collect gold

                if let Some(i) = self.gold.iter().position(| &g | g == (self.px, self.py)) {
                    self.gold.remove(i);
                    self.score += 100;

                    // spawn green text

                    let (sx, sy) = to_screen(self.px, self.py, self.cam);
                    self.texts.push(FloatingText { x: sx, y: sy, damage: -100, text_life: 1. })
                };


            }
        }

        // monster logic calculate the occupied spots, for non stacking
        let occupied: Vec<_> = self.monsters.iter().map(|m| (m.x, m.y)).chain(std::iter::once((self.px, self.py))).collect();

        for i in 0..self.monsters.len() {
            self.monsters[i].cd -= dt;
            if self.monsters[i].cd <= 0. {
                self.monsters[i].cd = 1.0;

                let (mx, my) = (self.monsters[i].x, self.monsters[i].y,);
                let d = calculate_distance( (mx, my), (self.px, self.py));

                if d == 1 {
                    self.hp -= 5;
                    let (sx, sy) = to_screen(self.px, self.py, self.cam);
                    self.texts.push(FloatingText { x: sx, y: sy - 40., damage: 5, text_life: 1. })

                } else {
                    // chase player 
                    let path = bfs(&self.map, (mx, my), (self.px, self.py));

                    if path.len() > 1 && !occupied.contains(&path[0]) {
                        self.monsters[i].x = path[0].0;
                        self.monsters[i].y = path[0].1;
                    }
                }
            }
        }

        return  false;
    }

    fn damage_monster(&mut self, idx: usize, amount: i32 ){
        self.monsters[idx].hp -= amount;
        // spawn floating text
        let (sx, sy) = to_screen(self.monsters[idx].x, self.monsters[idx].y, self.cam);
        self.texts.push(FloatingText { x: sx, y: sy - 40., damage: amount, text_life: 1. });


        // kill logic
        if self.monsters[idx].hp <= 0 {
            self.monsters.remove(idx);
            // increment score for kill
            self.score += 40;
        }
    }

    fn draw(&self) {
        for y in 0..MAP {
            for x in 0..MAP {
                if self.map[y][x] == Tile::Wall {
                    draw_wall(x, y, self.cam);
                } else {

                    // draw gold
                    if self.gold.contains(&(x, y)) {
                        let (sx, sy) = to_screen(x, y, self.cam);
                        draw_circle(sx, sy, 15., GOLD);
                    } else {
                        let (sx, sy) = to_screen(x, y, self.cam);
                        draw_circle(sx, sy, 1., LIGHTGRAY);

                    }

                    let (sx, sy) = to_screen(x, y, self.cam);
                    draw_circle(sx, sy + 16., 2., LIGHTGRAY);
                }
            }
        }

        // draw path
        for (px,py) in &self.path {
            let (sx, sy) = to_screen(*px, *py, self.cam);
            draw_circle(sx, sy + 16., 4., GOLD);
        }

        // draw player
        draw_stickman(self.px, self.py, self.cam, false);

        // draw monster
        for monster in &self.monsters {
            draw_stickman(monster.x, monster.y, self.cam, true);
        }

        // draw floating text
        for text in &self.texts {
            if text.damage < 0 {
                draw_text(&format!("+{}", -text.damage), text.x, text.y, 20., GREEN);

            } else {
                draw_text(&format!("--{}", text.damage), text.x, text.y, 20., RED);
            }
        
        }

        // hud
        draw_text(&format!("HP: {}", self.hp), 20., screen_height() - 40., 30., BLACK);
        draw_text(&format!("SCORE: {}", self.score), 20., screen_height() - 70., 30., BLACK);

    }

}

#[macroquad::main("Crab Snack")]


async fn main() {
    let mut game = Game::new();
    let mut state = AppState::Menu;

    loop {
        clear_background(WHITE);

        match state {
            AppState::Menu => {
                draw_text("Menu - Enter to start",100.,100.,40.,BLACK);
                if is_key_pressed(KeyCode::Enter){
                    game = Game::new();
                    state = AppState::Playing;
                }
            },
            AppState::Playing => {
                if game.update(get_frame_time()) {
                    state = AppState::GameOver;
                }
                game.draw();
            },
            AppState::GameOver => {
                game.draw();
                draw_rectangle(0.,0.,screen_width(),screen_height(),Color::new(1.,1.,1.,0.7));

                // victory & defeat
                let (message, color) = if game.hp > 0 {
                    ("VICTORY", GOLD)
                } else {
                    ("GAME OVER", RED)
                };

                draw_text(message,  screen_width() / 2. -100.,  screen_height() / 2., 60.,  color);

                draw_text(&format!("Final Score: {}", game.score),  screen_width() / 2. -80., screen_height() / 2. + 50.,  30.,  BLACK);

                draw_text("Enter Reset",  screen_width() / 2. -80., screen_width() / 2. + 90.,  20.,  GRAY);

                if is_key_pressed(KeyCode::Enter){
                    state = AppState::Menu;
                }

            }
        }
        next_frame().await;
    }
}
