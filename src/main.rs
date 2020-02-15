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
    fn move_in_direction(&mut self, direction: Direction, grow: bool) -> Option<Point> {
        let head = self.body[0];
        let new_head = match direction {
            Direction::Up => Point { x: head.x, y: head.y - 1 },
            Direction::Down => Point { x: head.x, y: head.y + 1 },
            Direction::Left => Point { x: head.x - 1, y: head.y },
            Direction::Right => Point { x: head.x + 1, y: head.y },
        };
        self.body.push_front(new_head);
        if grow {
            None
        } else {
            self.body.pop_back()
        }
    }
}


struct View {
    score_window: Window,
    game_window: Window,
}

impl View {
    fn new(window: &Window) -> Result<View, i32> {
        curs_set(0);
        window.clear();
        let score_window = window.subwin(1, window.get_max_x(), 0, 0)?;
        score_window.mvaddstr(0, 0, "Score: 0");
        score_window.refresh();
        let game_window = window.subwin(window.get_max_y() - 1, window.get_max_x(), 1, 0)?;
        game_window.nodelay(true);
        Ok(View { score_window, game_window })
    }
    fn display_apple(&self, apple: Point) {
        self.game_window.mvaddch(apple.y, apple.x, '*');
    }
    fn display_snake_head(&self, snake_head: Point) {
        self.game_window.mvaddch(snake_head.y, snake_head.x, '#');
    }
    fn delete_snake_tail(&self, snake_tail: Point) {
        self.game_window.mvaddch(snake_tail.y, snake_tail.x, ' ');
    }
    fn display_score(&self, score: i32) {
        self.score_window.mvaddstr(0, 7, format!("{}", score));
        self.score_window.refresh();
    }
}


struct Model {
    snake: Snake,
    apple: Point,
    score: i32,
    rng: ThreadRng,
}

impl Model {
    fn new() -> Model {
        let snake: Snake = Snake::new(Point { x: 10, y: 10 }, 3);
        let rng = rand::thread_rng();
        let apple = Point { x: 0, y: 0 };
        let score = 0;
        Model { snake, apple, score, rng }
    }
    fn generate_new_apple_point(&mut self, max_x: i32, max_y: i32) {
        let x = self.rng.gen_range(1, max_x - 2);
        let y = self.rng.gen_range(1, max_y - 2);
        self.apple = Point { x, y }
    }
}

struct Controller {
    view: View,
    model: Model,
}

impl Controller {
    fn new(window: &Window) -> Result<Controller, i32> {
        let view = View::new(&window)?;
        let model = Model::new();
        Ok(Controller { view, model })
    }
    fn run(&mut self) -> Result<i32, i32> {
        self.model.generate_new_apple_point(self.view.game_window.get_max_x(), self.view.game_window.get_max_y());
        self.view.display_apple(self.model.apple);
        loop {
            let current_direction = self.model.snake.get_current_direction();
            let direction_from_key = match self.view.game_window.getch() {
                Some(input) => {
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
            let new_direction = match (direction_from_key, self.model.snake.get_current_direction()) {
                (Direction::Left, Direction::Right) => Direction::Right,
                (Direction::Right, Direction::Left) => Direction::Left,
                (Direction::Up, Direction::Down) => Direction::Down,
                (Direction::Down, Direction::Up) => Direction::Up,
                (a, _) => a,
            };

            let head = self.model.snake.body[0];
            if head.x <= 0 || head.x >= self.view.game_window.get_max_x() || head.y <= 0 || head.y >= self.view.game_window.get_max_y() {
                break;
            }
            let grow = if head.x == self.model.apple.x && head.y == self.model.apple.y {
                self.model.generate_new_apple_point(self.view.game_window.get_max_x(), self.view.game_window.get_max_y());
                self.view.display_apple(self.model.apple);
                self.model.score += 1;
                self.view.display_score(self.model.score);
                true
            } else {
                false
            };
            self.model.snake.move_in_direction(new_direction, grow).map(|tail| {
                self.view.delete_snake_tail(tail)
            });
            self.view.display_snake_head(self.model.snake.body[0]);
            thread::sleep(Duration::from_millis(80));
        }
        Ok(self.model.score)
    }
}

fn game(window: &Window) -> Result<i32, i32> {
    let mut controller = Controller::new(&window)?;
    controller.run()
}

fn main() -> Result<(), i32> {
    let window = initscr();
    noecho();
    window.refresh();
    'outer: loop {
        let score = game(&window)?;
        window.mvaddstr(window.get_max_y() / 2, window.get_max_x() / 2 - 20, format!("Game over! Your score is: {}", score));
        window.mvaddstr(window.get_max_y() / 2 + 2, window.get_max_x() / 2 - 20, "Would you like to play again? (y/n)");
        window.nodelay(false);
        'inner: loop {
            match window.getch() {
                Some(input) => {
                    match input {
                        Input::Character('y') => { break 'inner; }
                        Input::Character('n') => { break 'outer; }
                        _ => (),
                    }
                }
                None => (),
            }
        };
    }

    endwin();
    Ok(())
}