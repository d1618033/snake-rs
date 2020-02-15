extern crate pancurses;
extern crate rand;

use pancurses::{initscr, endwin, Input, noecho, Window, curs_set};
use rand::Rng;
use std::collections::VecDeque;
use rand::prelude::ThreadRng;


#[derive(PartialEq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

struct Snake {
    body: VecDeque<Point>
}

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Snake {
    fn new(head: Point, n: usize) -> Snake {
        let mut body: VecDeque<Point> = VecDeque::new();
        for i in 0..n {
            body.push_back(Point { x: head.x - (i as i32), y: head.y })
        }
        Snake { body }
    }
    fn get_current_direction(&self) -> Direction {
        let head = &self.body[0];
        let second = &self.body[1];
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
        let head = &self.body[0];
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


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn create_snake() -> Snake {
        Snake::new(Point{x:10, y:5}, 3)
    }

    #[test]
    fn test_snake_new() {
        let snake = create_snake();
        assert_eq!(snake.body, vec![Point{x:10, y:5}, Point{x:9, y:5}, Point{x:8, y:5}])
    }

    #[test]
    fn test_snake_current_direction() {
        let snake = create_snake();
        assert_eq!(snake.get_current_direction(), Direction::Right);
    }

    #[test]
    fn test_snake_move_in_direction() {
        let mut snake = create_snake();
        snake.move_in_direction(Direction::Up, false);
        assert_eq!(snake.body, vec![Point{x:10, y:4}, Point{x:10, y:5}, Point{x:9, y:5}]);
    }

    #[test]
    fn test_snake_move_in_direction_grow() {
        let mut snake = create_snake();
        snake.move_in_direction(Direction::Up, true);
        assert_eq!(snake.body, vec![Point{x:10, y:4}, Point{x:10, y:5}, Point{x:9, y:5}, Point{x:8, y:5}]);
    }
}

enum UserInput {
    Direction(Direction),
    Other,
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
        game_window.timeout(100);
        game_window.draw_box(0, 0);
        Ok(View { score_window, game_window })
    }
    fn display_apple(&self, apple: &Point) {
        self.game_window.mvaddch(apple.y, apple.x, '*');
    }
    fn display_snake_head(&self, snake_head: &Point) {
        self.game_window.mvaddch(snake_head.y, snake_head.x, '#');
    }
    fn delete_snake_tail(&self, snake_tail: &Point) {
        self.game_window.mvaddch(snake_tail.y, snake_tail.x, ' ');
    }
    fn display_score(&self, score: i32) {
        self.score_window.mvaddstr(0, 7, format!("{}", score));
        self.score_window.refresh();
    }
    fn get_input_from_user(&self) -> Option<UserInput> {
        self.game_window.getch().map(|input| {
            match input {
                Input::Character('D') => UserInput::Direction(Direction::Left),
                Input::Character('C') => UserInput::Direction(Direction::Right),
                Input::Character('A') => UserInput::Direction(Direction::Up),
                Input::Character('B') => UserInput::Direction(Direction::Down),
                _ => UserInput::Other,
            }
        })
    }
    fn get_max_x(&self) -> i32 {
        self.game_window.get_max_x()
    }
    fn get_max_y(&self) -> i32 {
        self.game_window.get_max_y()
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
    fn _generate_new_apple_point(&mut self) {
        self.model.generate_new_apple_point(self.view.get_max_x(), self.view.get_max_y());
        self.view.display_apple(&self.model.apple);
    }
    fn _update_score(&mut self, amount: i32) {
        self.model.score += amount;
        self.view.display_score(self.model.score);
    }
    fn _collided_with_borders(&self) -> bool {
        let head = &self.model.snake.body[0];
        head.x <= 0 || head.x >= self.view.get_max_x() || head.y <= 0 || head.y >= self.view.get_max_y()
    }
    fn _collided_with_self(&self) -> bool {
        let head = &self.model.snake.body[0];
        self.model.snake.body.iter().skip(1).any(|part| {*part == *head})
    }
    fn _ate_apple(&self) -> bool {
        let head = &self.model.snake.body[0];
        head.x == self.model.apple.x && head.y == self.model.apple.y
    }
    fn _get_new_direction(&self) -> Direction {
        let current_direction = self.model.snake.get_current_direction();
        match self.view.get_input_from_user() {
            Some(user_input) => {
                match user_input {
                    UserInput::Direction(direction_from_key) => {
                        match (direction_from_key, current_direction) {
                            (Direction::Left, Direction::Right) => Direction::Right,
                            (Direction::Right, Direction::Left) => Direction::Left,
                            (Direction::Up, Direction::Down) => Direction::Down,
                            (Direction::Down, Direction::Up) => Direction::Up,
                            (a, _) => a,
                        }
                    }
                    UserInput::Other => current_direction
                }
            }
            None => current_direction,
        }
    }
    fn run(&mut self) -> Result<i32, i32> {
        self._generate_new_apple_point();
        loop {
            let grow = if self._ate_apple() {
                self._generate_new_apple_point();
                self._update_score(1);
                true
            } else {
                false
            };
            if self._collided_with_borders() || self._collided_with_self() {
                break;
            }
            let new_direction = self._get_new_direction();
            self.model.snake.move_in_direction(new_direction, grow).map(|tail| {
                self.view.delete_snake_tail(&tail)
            });
            self.view.display_snake_head(&self.model.snake.body[0]);
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