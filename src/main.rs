extern crate pancurses;
extern crate rand;

use pancurses::{initscr, endwin, Input, noecho, Window, curs_set};
use rand::Rng;
use std::collections::VecDeque;
use rand::prelude::ThreadRng;


#[derive(PartialEq, Debug, Clone)]
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


enum UserInput {
    Direction(Direction),
    Other,
}

trait ViewTrait {
    fn display_apple(&mut self, apple: &Point);
    fn display_snake_head(&mut self, snake_head: &Point);
    fn delete_snake_tail(&mut self, snake_tail: &Point);
    fn display_score(&mut self, score: i32);
    fn get_input_from_user(&mut self) -> Option<UserInput>;
    fn get_max_x(&self) -> i32;
    fn get_max_y(&self) -> i32;
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
}

impl ViewTrait for View {
    fn display_apple(&mut self, apple: &Point) {
        self.game_window.mvaddch(apple.y, apple.x, '*');
    }
    fn display_snake_head(&mut self, snake_head: &Point) {
        self.game_window.mvaddch(snake_head.y, snake_head.x, '#');
    }
    fn delete_snake_tail(&mut self, snake_tail: &Point) {
        self.game_window.mvaddch(snake_tail.y, snake_tail.x, ' ');
    }
    fn display_score(&mut self, score: i32) {
        self.score_window.mvaddstr(0, 7, format!("{}", score));
        self.score_window.refresh();
    }
    fn get_input_from_user(&mut self) -> Option<UserInput> {
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

trait ModelTrait {
    fn generate_new_apple_point(&mut self, max_x: i32, max_y: i32);
    fn get_apple(&self) -> &Point;
    fn get_snake(&self) -> &Snake;
    fn get_score(&self) -> i32;
    fn update_score(&mut self, amount: i32);
    fn move_in_direction(&mut self, direction: Direction, grow: bool) -> Option<Point>;
}

impl Model {
    fn new() -> Model {
        let snake: Snake = Snake::new(Point { x: 10, y: 10 }, 3);
        let rng = rand::thread_rng();
        let apple = Point { x: 0, y: 0 };
        let score = 0;
        Model { snake, apple, score, rng }
    }
}

impl ModelTrait for Model {
    fn generate_new_apple_point(&mut self, max_x: i32, max_y: i32) {
        let x = self.rng.gen_range(1, max_x - 2);
        let y = self.rng.gen_range(1, max_y - 2);
        self.apple = Point { x, y }
    }

    fn get_apple(&self) -> &Point {
        &self.apple
    }

    fn get_snake(&self) -> &Snake {
        &self.snake
    }

    fn get_score(&self) -> i32 {
        self.score
    }

    fn update_score(&mut self, amount: i32) {
        self.score += amount;
    }

    fn move_in_direction(&mut self, direction: Direction, grow: bool) -> Option<Point> {
        self.snake.move_in_direction(direction, grow)
    }
}

struct Controller<V: ViewTrait, M: ModelTrait> {
    view: V,
    model: M,
}

impl Controller<View, Model> {
    fn new(window: &Window) -> Result<Controller<View, Model>, i32> {
        let view = View::new(&window)?;
        let model = Model::new();
        Ok(Controller { view, model })
    }
}

impl<V: ViewTrait, M: ModelTrait> Controller<V, M> {
    fn _generate_new_apple_point(&mut self) {
        self.model.generate_new_apple_point(self.view.get_max_x(), self.view.get_max_y());
        self.view.display_apple(&self.model.get_apple());
    }
    fn _update_score(&mut self, amount: i32) {
        self.model.update_score(amount);
        self.view.display_score(self.model.get_score());
    }
    fn _collided_with_borders(&self) -> bool {
        let head = &self.model.get_snake().body[0];
        head.x <= 0 || head.x >= self.view.get_max_x() || head.y <= 0 || head.y >= self.view.get_max_y()
    }
    fn _collided_with_self(&self) -> bool {
        let head = &self.model.get_snake().body[0];
        self.model.get_snake().body.iter().skip(1).any(|part| { *part == *head })
    }
    fn _ate_apple(&self) -> bool {
        let head = &self.model.get_snake().body[0];
        head.x == self.model.get_apple().x && head.y == self.model.get_apple().y
    }
    fn _get_new_direction(&mut self) -> Direction {
        let current_direction = self.model.get_snake().get_current_direction();
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
            self.model.move_in_direction(new_direction, grow).map(|tail| {
                self.view.delete_snake_tail(&tail)
            });
            self.view.display_snake_head(&self.model.get_snake().body[0]);
        }
        Ok(self.model.get_score())
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn create_snake() -> Snake {
        Snake::new(Point { x: 10, y: 5 }, 3)
    }

    #[test]
    fn test_snake_new() {
        let snake = create_snake();
        assert_eq!(snake.body, vec![Point { x: 10, y: 5 }, Point { x: 9, y: 5 }, Point { x: 8, y: 5 }])
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
        assert_eq!(snake.body, vec![Point { x: 10, y: 4 }, Point { x: 10, y: 5 }, Point { x: 9, y: 5 }]);
    }

    #[test]
    fn test_snake_move_in_direction_grow() {
        let mut snake = create_snake();
        snake.move_in_direction(Direction::Up, true);
        assert_eq!(snake.body, vec![Point { x: 10, y: 4 }, Point { x: 10, y: 5 }, Point { x: 9, y: 5 }, Point { x: 8, y: 5 }]);
    }

    #[derive(Debug, PartialEq)]
    enum SnakePaint {
        Add(Point),
        Remove(Point),
    }

    struct MockView {
        snake_paints: Vec<SnakePaint>,
        apple_paints: Vec<Point>,
        scores: Vec<i32>,
        user_inputs: VecDeque<UserInput>,
        max_x: i32,
        max_y: i32,
    }

    impl MockView {
        fn new(user_inputs: VecDeque<UserInput>, max_x: i32, max_y: i32) -> MockView {
            MockView {
                snake_paints: Vec::new(),
                apple_paints: Vec::new(),
                scores: Vec::new(),
                user_inputs,
                max_x,
                max_y,
            }
        }
    }

    impl ViewTrait for MockView {
        fn display_apple(&mut self, apple: &Point) {
            self.apple_paints.push(apple.clone());
        }

        fn display_snake_head(&mut self, snake_head: &Point) {
            self.snake_paints.push(SnakePaint::Add(snake_head.clone()));
        }

        fn delete_snake_tail(&mut self, snake_tail: &Point) {
            self.snake_paints.push(SnakePaint::Remove(snake_tail.clone()));
        }

        fn display_score(&mut self, score: i32) {
            self.scores.push(score);
        }

        fn get_input_from_user(&mut self) -> Option<UserInput> {
            return self.user_inputs.pop_front();
        }

        fn get_max_x(&self) -> i32 {
            self.max_x
        }

        fn get_max_y(&self) -> i32 {
            self.max_y
        }
    }

    struct MockModel {
        model: Model,
    }

    impl Model {
        fn new_from_snake_apple_and_score(snake: Snake, apple: Point, score: i32) -> Model {
            let rng = rand::thread_rng();
            Model { snake, apple, score, rng }
        }
    }

    impl MockModel {
        fn new(snake: Snake, apple: Point, score: i32) -> MockModel {
            MockModel { model: Model::new_from_snake_apple_and_score(snake, apple, score) }
        }
    }

    impl ModelTrait for MockModel {
        fn generate_new_apple_point(&mut self, _max_x: i32, _max_y: i32) {}

        fn get_apple(&self) -> &Point {
            self.model.get_apple()
        }

        fn get_snake(&self) -> &Snake {
            self.model.get_snake()
        }

        fn get_score(&self) -> i32 {
            self.model.get_score()
        }

        fn update_score(&mut self, amount: i32) {
            self.model.update_score(amount)
        }

        fn move_in_direction(&mut self, direction: Direction, grow: bool) -> Option<Point> {
            self.model.move_in_direction(direction, grow)
        }
    }

    #[test]
    fn test_controller() {
        let snake = Snake::new(Point { x: 4, y: 4 }, 3);
        let apple = Point { x: 4, y: 2 };
        let model = MockModel::new(snake, apple, 0);
        let mut user_inputs: VecDeque<UserInput> = VecDeque::new();
        user_inputs.push_back(UserInput::Direction(Direction::Up));
        let view = MockView::new(
            user_inputs,
            40,
            40,
        );
        let mut controller = Controller { view, model };
        let result = controller.run();
        match result {
            Ok(res) => {
                assert_eq!(controller.model.get_score(), 1);
                assert_eq!(controller.model.get_snake().body.len(), 4);
                assert_eq!(controller.view.snake_paints, vec![
                    SnakePaint::Remove(Point { x: 2, y: 4 }),
                    SnakePaint::Add(Point { x: 4, y: 3 }),
                    SnakePaint::Remove(Point { x: 3, y: 4 }),
                    SnakePaint::Add(Point { x: 4, y: 2 }),
                    SnakePaint::Add(Point { x: 4, y: 1 }),
                    SnakePaint::Remove(Point { x: 4, y: 4 }),
                    SnakePaint::Add(Point { x: 4, y: 0 }),
                ]);
                assert_eq!(res, 1);
            },
            Err(res) => panic!("Got result: {}", res)
        }
    }
}
