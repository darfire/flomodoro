use fltk::{app, prelude::*, window::Window, group, frame};
use fltk::enums::{Align};
use fltk::enums;
use fltk::input;
use fltk::button;

fn format_time(time: f64) -> String {
    let hours = (time / 3600.0) as u64;
    let minutes = ((time % 3600.0) / 60.0) as u64;
    let seconds = (time % 60.0) as u64;

    if hours > 0 {
        return format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        return format!("{:02}:{:02}", minutes, seconds)
    }
}

#[derive(Debug, Clone)]
struct Task {
    project: String,
    task: String,
    seconds: i32,
}

#[derive(Debug)]
struct TimeWindow {
    window: Window,
    project_frame: frame::Frame,
    task_frame: frame::Frame,
    distractions_frame: frame::Frame,
    time: f64,
    time_passed: f64,
    pause_time: f64,
    last_time: std::time::SystemTime,
    pause_button: button::Button,
    is_paused: bool,
    n_distractions: i32,
    start_time: std::time::SystemTime,
    timeout_handle: Option<app::TimeoutHandle>
}

#[derive(Debug, Clone)]
enum Message {
    TogglePause,
    Update,
    Distraction,
    NewTask,
    IncTime(f64),
}

fn make_button(label: &str, message: Message) -> button::Button {

    let mut button = button::Button::default().with_label(label);

    button.set_callback(move |_| {
        let (sender, _) = app::channel::<Message>();

        sender.send(message.clone());
    });

    button
}

impl TimeWindow {
    fn new(x: i32, y: i32, w: i32, h: i32) -> TimeWindow {
        let mut window = Window::new(x, y, w, h, "Time Window");

        window.set_border(false);

        let mut col = group::Flex::default_fill().column();

        let mut project_frame = frame::Frame::default()
            .with_label("Project")
            .with_align(Align::Inside | Align::Wrap | Align::Center);

        col.fixed(&project_frame, 40);

        project_frame.set_label_font(fltk::enums::Font::HelveticaBold);
        project_frame.set_label_size(20);
        project_frame.set_label_color(enums::Color::from_rgb(0, 128, 0));

        let mut task_frame = frame::Frame::default()
            .with_label("Task")
            .with_align(Align::Inside | Align::Wrap | Align::Center);

        task_frame.set_label_font(fltk::enums::Font::HelveticaBold);
        task_frame.set_label_size(32);

        task_frame.set_label_color(enums::Color::from_rgb(255, 0, 0));

        col.fixed(&task_frame, 80);

        let mut pause_button = make_button("Time", Message::TogglePause);

        pause_button.set_label_font(fltk::enums::Font::HelveticaBold);
        pause_button.set_label_size(40);
        pause_button.set_label_color(enums::Color::from_rgb(0, 0, 255));

        let row = group::Flex::default().row();

        col.fixed(&row, 40);

        make_button("-5m", Message::IncTime(-300.0));
        make_button("-1m", Message::IncTime(-60.0));
        make_button("+1m", Message::IncTime(60.0));
        make_button("+5m", Message::IncTime(300.0));

        row.end();

        let mut row = group::Flex::default().row();

        col.fixed(&row, 40);

        let distractions_frame = frame::Frame::default()
            .with_label("Distractions")
            .with_align(Align::Inside | Align::Center);

        let distractions_button = make_button("+", Message::Distraction);

        row.fixed(&distractions_button, 80);

        row.end();

        col.end();
        window.end();

        let last_time = std::time::SystemTime::now();

        TimeWindow {
            window,
            project_frame,
            task_frame,
            distractions_frame,
            time: 0.0,
            time_passed: 0.0,
            pause_time: 0.0,
            pause_button,
            last_time,
            is_paused: false,
            n_distractions: 0,
            start_time: std::time::SystemTime::now(),
            timeout_handle: None,
        }
    }

    fn start(&mut self, task: Task) {
        self.project_frame.set_label(&task.project);
        self.task_frame.set_label(&task.task);
        self.distractions_frame.set_label(&format!("Distractions: {}", self.n_distractions));
        self.time = task.seconds as f64;
        self.time_passed = 0.0;
        self.pause_time = 0.0;
        self.n_distractions = 0;
        self.start_time = std::time::SystemTime::now();
        self.last_time = std::time::SystemTime::now();

        self.timeout_handle = Some(app::add_timeout3(0.2, |handle| {
            let (sender, _) = app::channel::<Message>();

            sender.send(Message::Update);

            app::repeat_timeout3(0.2, handle);
        }));

        self.update_time_frame();

        self.show();
    }

    fn update_time_frame(&mut self) {
        let msg = if self.is_paused {
            format!("Paused: {} / -{}", format_time(self.pause_time), format_time(self.time - self.time_passed))
        } else {
            format!("-{}", format_time(self.time - self.time_passed))
        };
        
        self.pause_button.set_label(&msg)
    }

    fn pause(&mut self) {
        self.is_paused = true;
        self.pause_button.set_label_size(32);
        self.pause_button.set_label("Resume");
    }

    fn resume(&mut self) {
        self.is_paused = false;
        self.pause_button.set_label_size(40);
        self.pause_button.set_label("Pause");
    }

    fn toggle_pause(&mut self) {
        if self.is_paused {
            self.resume();
        } else {
            self.pause();
        }

        self.update();
    }

    fn is_active(&self) -> bool {
        return self.time_passed < self.time && self.window.visible();
    }

    fn add_distraction(&mut self) {
        self.n_distractions += 1;
        self.distractions_frame.set_label(&format!("Distractions: {}", self.n_distractions));
    }

    fn color_for_ratio(&self, ratio: f32) -> fltk::enums::Color {
        if ratio < 0.7 {
            return fltk::enums::Color::Background;
        } else if ratio < 0.9 {
            return fltk::enums::Color::from_rgb(255, 255, 0);
        } else {
            return fltk::enums::Color::from_rgb(255, 165, 0);
        }
    } 

    fn update_colors(&mut self, color: fltk::enums::Color) {
        self.window.set_color(color);

        self.pause_button.set_color(color);

        self.window.redraw();
    }

    fn inc_time(&mut self, seconds: f64) {
        let new_time = self.time + seconds;
        if new_time > 0.0 {
            self.time = new_time;
        }

        self.update();
    }

    fn update(&mut self) {
        let now = std::time::SystemTime::now();
        let elapsed = now.duration_since(self.last_time).unwrap().as_secs_f64();
        self.last_time = now;

        if self.is_paused {
            self.pause_time += elapsed;
        } else {
            self.time_passed += elapsed;
        }

        let new_ratio = self.time_passed as f32 / self.time as f32;

        if self.time_passed >= self.time {
            return;
        }

        self.update_time_frame();

        let new_color = self.color_for_ratio(new_ratio);

        if self.window.color() != new_color {
            self.update_colors(new_color);
        }
    }

    fn show(&mut self) {
        self.window.show();
        self.window.set_on_top();


        self.window.handle({
            let mut x = 0;
            let mut y = 0;
            move |w, ev| match ev {
                enums::Event::Push => {
                    let coords = app::event_coords();
                    x = coords.0;
                    y = coords.1;
                    true
                }
                enums::Event::Drag => {
                    w.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                    true
                }
                _ => false,
            }
        });
    }

    fn hide(&mut self) {
        match self.timeout_handle {
            Some(handle) => app::remove_timeout3(handle),
            None => (),
        };

        self.window.hide();
    }
}

fn make_input(parent: &mut group::Flex, label: &str) -> input::Input {
    let row = group::Flex::default().row();

    frame::Frame::default()
        .with_label(label)
        .with_align(Align::Inside | Align::Center);

    let input = input::Input::default();

    parent.fixed(&row, 40);

    row.end();

    input
}

fn make_multiline_input(label: &str) -> input::MultilineInput {
    let row = group::Flex::default().row();
    {
        frame::Frame::default()
            .with_label(label)
            .with_align(Align::Inside | Align::Center);

        let input = input::MultilineInput::default();

        row.end();

        input
    }
}

#[derive(Debug)]
struct TimeFormatError;

fn parse_time(time: String) -> Result<i32, TimeFormatError> {
    let last_char = time.chars().last().unwrap();
    let mut multiple = 1;

    if !last_char.is_numeric() {
        multiple = match last_char {
            'h' => 3600,
            'm' => 60,
            's' => 1,
            _ => -1,
        };

        if multiple == -1 {
            return Err(TimeFormatError);
        }
    }

    let time = if multiple == 1 {
        time
    } else {
        time.trim_end_matches(last_char).to_string()
    };

    let time = time.parse::<i32>();

    match time {
        Err(_) => Err(TimeFormatError),
        Ok(time) => Ok(time * multiple),
    }
}

struct ConfigWindow {
    window: Window,
    project: input::Input,
    task: input::MultilineInput,
    time: input::Input,
}

impl ConfigWindow {
    fn new(x: i32, y: i32, w: i32, h: i32) -> ConfigWindow {
        let window = Window::new(x, y, w, h, "Configure your task");

        let mut col = group::Flex::default_fill().column();

        let mut project = make_input(&mut col, "Project");

        project.set_value("The Big One");

        let mut task = make_multiline_input("Task");

        task.set_value("Piece of cake");

        let mut time = make_input(&mut col, "Time");

        time.set_value("25m");

        let row = group::Flex::default().row();

        let mut button = button::Button::default().with_label("Start");

        col.fixed(&row, 40);

        row.end();

        button.set_callback(move |_| {
            let (sender, _) = app::channel::<Message>();

            sender.send(Message::NewTask);
        });

        col.end();

        window.end();

        ConfigWindow {
            window,
            project,
            task,
            time,
        }
    }

    fn get_task(&self) -> Option<Task> {
        let project = self.project.value();
        let task = self.task.value();
        let time = self.time.value();

        let seconds = parse_time(time).ok()?;

        let task = Task {
            project,
            task,
            seconds,
        };

        Some(task)
    }

    fn show(&mut self) {
        self.window.show();
    }

    fn hide(&mut self) {
        self.window.hide();
    }
}

struct App {
    app: app::App,
    config_window: ConfigWindow,
    time_window: TimeWindow,
}

impl App {
    fn new() -> App {
        let app = app::App::default();

        let config_window = ConfigWindow::new(100, 100, 400, 240);

        let time_window = TimeWindow::new(100, 100, 400, 300);

        App {
            app,
            config_window,
            time_window,
        }
    }

    fn run(&mut self) -> Result<(), FltkError> {
        let (_, r) = app::channel::<Message>();

        while self.app.wait() {
            if let Some(message) = r.recv() {
                match message {
                    Message::NewTask => {
                        let task = self.config_window.get_task();

                        match task {
                            Some(task) => {
                                self.config_window.hide();
                                self.time_window.start(task);
                            }
                            None => (),
                        }
                    }
                    Message::Update => {
                        if self.time_window.is_active() {
                            self.time_window.update();
                        } else {
                            self.config_window.show();
                            self.time_window.hide();
                        }
                    }
                    Message::TogglePause => {
                        self.time_window.toggle_pause();
                    }
                    Message::Distraction => {
                        self.time_window.add_distraction();
                    }
                    Message::IncTime(seconds) => {
                        self.time_window.inc_time(seconds);
                    }
                }
            }
        }

        Ok(())
    }
}

fn main() {
    let mut app = App::new();

    app.config_window.show();

    app.run().unwrap();
}