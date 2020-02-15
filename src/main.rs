extern crate pancurses;
extern crate rand;

use pancurses::{initscr, endwin, Input, noecho, Window, curs_set};
use rand::Rng;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use rand::prelude::ThreadRng;


#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

struct Snake {
    body: VecDeque<Point>
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Snake {
    fn new(head: Point, n: usize) -> Snake {
        let mut body: VecDeque<Point> = VecDeque::new();
        body.push_back(head);
        for i in 1..n {
            body.push_back(Point { x: head.x - (i as i32), y: head.y })
        }
        Snake { body }
    }
    fn get_current_direction(&self) -> Direction {
        let head = self.body[0];
        let second = self.body[1];
        return match head.x - second.x {
            1 => Direction::Right,
            -1 => Direction::Left,
            0 => {
                match head.y - second.y {
                    1 => Direction::Down,
                    -1 => Direction::Up,
                    _ => panic!("Unknown direction")
                }
            }
            _ => panic!("Unknown direction")
        };
    }
    fn move_in_direction(&mut self, direction: Direction, window: &Window, grow: bool) -> Option<()> {
        let head = self.body[0];
        let new_head = match direction {
            Direction::Up => Point { x: head.x, y: head.y - 1 },
            Direction::Down => Point { x: head.x, y: head.y + 1 },
            Direction::Left => Point { x: head.x - 1, y: head.y },
            Direction::Right => Point { x: head.x + 1, y: head.y },
        };
        self.body.push_front(new_head);
        if !grow {
            let tail = self.body.pop_back()?;
            window.mvaddch(tail.y, tail.x, ' ');
        }
        window.mvaddch(new_head.y, new_head.x, '#');
        Some(())
    }
}


fn generate_random_point(rng: &mut ThreadRng, window: &Window) -> Point {
    let x = rng.gen_range(1, window.get_max_x() - 2);
    let y = rng.gen_range(1, window.get_max_y() - 2);
    Point{x, y}
}

fn game(window: &Window) {
    curs_set(0);
    window.nodelay(true);
    window.clear();
    let mut snake: Snake = Snake::new(Point { x: 10, y: 10 }, 3);
    let mut rng = rand::thread_rng();
    let mut apple = generate_random_point(&mut rng, &window);
    window.mvaddch(apple.y, apple.x, '*');
    loop {
        let current_direction = snake.get_current_direction();
        let direction_from_key = match window.getch() {
            Some(input) => {
//                window.mvaddstr(0, 0, format!("{:?}", input));
                match input {
                    Input::Character('D') => Direction::Left,
                    Input::Character('C') => Direction::Right,
                    Input::Character('A') => Direction::Up,
                    Input::Character('B') => Direction::Down,
                    Input::KeyAbort => break,
                    _ => current_direction
                }
            }
            None => current_direction,
        };
        let new_direction = match (direction_from_key, snake.get_current_direction()) {
          (Direction::Left, Direction::Right) => Direction::Right,
          (Direction::Right, Direction::Left) => Direction::Left,
          (Direction::Up, Direction::Down) => Direction::Down,
          (Direction::Down, Direction::Up) => Direction::Up,
          (a, _) => a,
        };

        let head = snake.body[0];
        if head.x <= 0 || head.x >= window.get_max_x() || head.y <= 0 || head.y >= window.get_max_y() {
            break;
        }
        let grow = if head.x == apple.x && head.y == apple.y {
            apple = generate_random_point(&mut rng, &window);
            window.mvaddch(apple.y, apple.x, '*');
            true
        } else {
            false
        };
        snake.move_in_direction(new_direction, &window, grow);
        thread::sleep(Duration::from_millis(80));
    }
}

fn main() {
    let window = initscr();
    noecho();
    window.refresh();
    'outer: loop {
        game(&window);
        window.clear();
        window.mvaddstr(window.get_max_y() / 2, window.get_max_x() / 2 - 10, "Game over!");
        window.mvaddstr(window.get_max_y() / 2 + 2, window.get_max_x() / 2 - 20, "Would you like to play again? (y/n)");
        window.nodelay(false);
        'inner: loop {
            match window.getch() {
                Some(input) => {
                    match input {
                        Input::Character('y') => {break 'inner;},
                        Input::Character('n') => {break 'outer;},
                        _ => (),
                    }
                }
                None => (),
            }
        };
    }

    endwin();
}