use std::collections::HashMap;

use egui::{
    lerp, remap_clamp, Button, Color32, Frame, Grid, Pos2, RichText, Slider,
    TextEdit, Vec2,
};
use egui_extras::RetainedImage;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

#[derive(EnumIter, AsRefStr)]
enum Module {
    Menu,
    Wires,
    Button,
    Keypad,
    SimonSays,
    WhosOnFirst,
    Memory,
    MorseCode,
    ComplicatedWires,
    WireSequences,
    Mazes,
    Passwords,
    Knobs,
}

#[derive(AsRefStr, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum KeypadButton {
    None,
    O,
    A,
    Lambda,
    N,
    Person,
    H,
    AntiC,
    Euro,
    Q,
    EmptyStar,
    Question,
    Copyright,
    W,
    X,
    R,
    N6,
    Paragraph,
    B,
    Smile,
    Trident,
    C,
    Snake,
    FilledStar,
    Puzzle,
    AE,
    Yot,
    Omega,
}

impl KeypadButton {
    fn name(&self) -> &str {
        match self {
            KeypadButton::AntiC => "Anti-C",
            KeypadButton::EmptyStar => "Empty Stat",
            KeypadButton::N6 => "6",
            KeypadButton::FilledStar => "Filled Star",
            KeypadButton::Yot => "Yot",
            k => k.as_ref(),
        }
    }
}

#[derive(Default)]
struct Memory {
    position1: u8,
    position2: u8,
    label1: u8,
    label2: u8,
    label3: u8,
    label4: u8,
}

#[derive(Default)]
struct WireSequence {
    red: u8,
    blue: u8,
    black: u8,
}

#[derive(Clone, Copy)]
enum SimonColor {
    Red,
    Blue,
    Green,
    Yellow,
}

impl SimonColor {
    fn color32(&self) -> Color32 {
        match self {
            SimonColor::Red => Color32::RED,
            SimonColor::Blue => Color32::BLUE,
            SimonColor::Green => Color32::GREEN,
            SimonColor::Yellow => Color32::YELLOW,
        }
    }
}

#[derive(Default)]
struct SimonSays {
    strikes: u8,
    vowel: bool,
    entered: Vec<SimonColor>,
}

impl SimonSays {
    const TABLE: [SimonColor; 24] = [
        SimonColor::Blue, // No vowel, 0 strikes
        SimonColor::Yellow,
        SimonColor::Green,
        SimonColor::Red,
        SimonColor::Red, // No vowel, 1 strike
        SimonColor::Blue,
        SimonColor::Yellow,
        SimonColor::Green,
        SimonColor::Yellow, // No vowel, 2 strikes
        SimonColor::Green,
        SimonColor::Blue,
        SimonColor::Red,
        SimonColor::Blue, // Vowel, 0 strikes
        SimonColor::Red,
        SimonColor::Yellow,
        SimonColor::Green,
        SimonColor::Yellow, // Vowel, 1 strike
        SimonColor::Green,
        SimonColor::Blue,
        SimonColor::Red,
        SimonColor::Green, // Vowel, 2 strikes
        SimonColor::Red,
        SimonColor::Yellow,
        SimonColor::Blue,
    ];
    fn convert(&self, color: &SimonColor) -> SimonColor {
        Self::TABLE[(if self.vowel { 12 } else { 0 }
            + self.strikes * 4
            + match color {
                SimonColor::Red => 0,
                SimonColor::Blue => 1,
                SimonColor::Green => 2,
                SimonColor::Yellow => 3,
            }) as usize]
    }
}

fn whos_translate(word: &str) -> String {
    if word.chars().count() > 4 {
        let mut s = String::from(word.chars().nth(0).unwrap());
        s += &word[word.char_indices().nth_back(2).unwrap().0..];
        s
    } else {
        word.to_owned()
    }
}

#[cfg(target_os = "android")]
const KEYBOARD: [char; 28] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '\'', '?'
];

#[cfg(target_os = "android")]
fn keyboard(ui: &mut egui::Ui, string: &mut String) -> bool {
    Grid::new("keyboard").spacing((0.0, 0.0)).min_col_width(0.0).show(ui, |ui| {
        let mut changed = false;
        for i in 0usize..28 {
            if ui.add(Button::new(RichText::new(KEYBOARD[i])).min_size(Vec2::new(30.0, 10.0)).rounding(0.0)).clicked() {
                string.push(KEYBOARD[i]);
                changed = true;
            }
            if i == 9 {
                ui.end_row();
            } else if i == 18 {
                if ui.add(Button::new(RichText::new("\u{2190}")).min_size(Vec2::new(30.0, 10.0)).rounding(0.0)).clicked() {
                    string.pop();
                    changed = true;
                }
                ui.end_row();
            } else if i == 25 {
                if ui.add(Button::new(RichText::new("\u{2588}").size(10.0)).min_size(Vec2::new(30.0, 10.0)).rounding(0.0)).clicked() {
                    string.push(' ');
                    changed = true;
                }
            }
        }
        changed
    }).inner
}

pub struct Application {
    module: Module,
    state: usize,
    label: String,
    painter: egui::Painter,
    keypad: HashMap<KeypadButton, u8>,
    simon_says: SimonSays,
    whos_on_first_positions: HashMap<String, usize>,
    whos_on_first_buttons: HashMap<String, Vec<String>>,
    whos_on_first: [String; 7],
    memory: Memory,
    wire_sequence: WireSequence,
    password: [String; 5],
    keypad_image: RetainedImage,
    morse_code_image: RetainedImage,
    mazes_image: RetainedImage,
    knobs_image: RetainedImage,
}

impl Application {
    const KEYPAD_BUTTONS: [[KeypadButton; 5]; 6] = [
        [
            KeypadButton::O,
            KeypadButton::A,
            KeypadButton::Lambda,
            KeypadButton::N,
            KeypadButton::Person,
        ],
        [
            KeypadButton::H,
            KeypadButton::AntiC,
            KeypadButton::Euro,
            KeypadButton::Q,
            KeypadButton::EmptyStar,
        ],
        [
            KeypadButton::Question,
            KeypadButton::Copyright,
            KeypadButton::W,
            KeypadButton::X,
            KeypadButton::R,
        ],
        [
            KeypadButton::N6,
            KeypadButton::Paragraph,
            KeypadButton::B,
            KeypadButton::Smile,
            KeypadButton::Trident,
        ],
        [
            KeypadButton::C,
            KeypadButton::Snake,
            KeypadButton::FilledStar,
            KeypadButton::Puzzle,
            KeypadButton::AE,
        ],
        [
            KeypadButton::Yot,
            KeypadButton::Omega,
            KeypadButton::None,
            KeypadButton::None,
            KeypadButton::None,
        ],
    ];
    const KEYPAD_COLUMNS: [[KeypadButton; 7]; 6] = [
        [
            KeypadButton::O,
            KeypadButton::A,
            KeypadButton::Lambda,
            KeypadButton::N,
            KeypadButton::Person,
            KeypadButton::H,
            KeypadButton::AntiC,
        ],
        [
            KeypadButton::Euro,
            KeypadButton::O,
            KeypadButton::AntiC,
            KeypadButton::Q,
            KeypadButton::EmptyStar,
            KeypadButton::H,
            KeypadButton::Question,
        ],
        [
            KeypadButton::Copyright,
            KeypadButton::W,
            KeypadButton::Q,
            KeypadButton::X,
            KeypadButton::R,
            KeypadButton::Lambda,
            KeypadButton::EmptyStar,
        ],
        [
            KeypadButton::N6,
            KeypadButton::Paragraph,
            KeypadButton::B,
            KeypadButton::Person,
            KeypadButton::X,
            KeypadButton::Question,
            KeypadButton::Smile,
        ],
        [
            KeypadButton::Trident,
            KeypadButton::Smile,
            KeypadButton::B,
            KeypadButton::C,
            KeypadButton::Paragraph,
            KeypadButton::Snake,
            KeypadButton::FilledStar,
        ],
        [
            KeypadButton::N6,
            KeypadButton::Euro,
            KeypadButton::Puzzle,
            KeypadButton::AE,
            KeypadButton::Trident,
            KeypadButton::Yot,
            KeypadButton::Omega,
        ],
    ];
    const COMPLICATED_WIRES: [&str; 16] = [
        "ALWAYS",
        "NEVER",
        "ALWAYS",
        "2+ BATTERIES",
        "LAST DIGIT EVEN",
        "PARALLEL PORT",
        "NEVER",
        "PARALLEL PORT",
        "LAST DIGIT EVEN",
        "2+ BATTERIES",
        "ALWAYS",
        "2+ BATTERIES",
        "LAST DIGIT EVEN",
        "LAST DIGIT EVEN",
        "PARALLEL PORT",
        "NEVER",
    ];
    const WIRE_SEQUENCE: [&str; 27] = [
        "C", "B", "A", "AC", "B", "AC", "ABC", "AB", "B", // Red
        "B", "AC", "B", "A", "B", "BC", "C", "AC", "A", // Blue
        "ABC", "AC", "B", "AC", "B", "BC", "AB", "C", "C", // Black
    ];
    const PASSWORDS: [&str; 35] = [
        "ABOUT", "AFTER", "AGAIN", "BELOW", "COULD", "EVERY", "FIRST", "FOUND", "GREAT", "HOUSE",
        "LARGE", "LEARN", "NEVER", "OTHER", "PLACE", "PLANT", "POINT", "RIGHT", "SMALL", "SOUND",
        "SPELL", "STILL", "STUDY", "THEIR", "THERE", "THESE", "THINK", "THINK", "THREE", "WATER",
        "WHERE", "WHICH", "WORLD", "WOULD", "WRITE",
    ];

    pub fn new(ctx: &egui::Context) -> Self {
        let mut whos_on_first_positions: HashMap<String, usize> = HashMap::new();
        for (label, position) in [
            // Positions:
            // 0 1
            // 2 3
            // 4 5
            ("", 4),
            ("BLANK", 3),
            ("C", 1),
            ("CEE", 5),
            ("DISPLAY", 5),
            ("FIRST", 1),
            ("HOLD ON", 5),
            ("LEAD", 5),
            ("LED", 2),
            ("LEED", 4),
            ("NO", 5),
            ("NOTHING", 2),
            ("OKAY", 1),
            ("READ", 3),
            ("RED", 3),
            ("REED", 4),
            ("SAYS", 5),
            ("SEE", 5),
            ("THEIR", 3),
            ("THERE", 5),
            ("THEY ARE", 2),
            ("THEY'RE", 4),
            ("UR", 0),
            ("YES", 2),
            ("YOU", 3),
            ("YOU ARE", 5),
            ("YOU'RE", 3),
            ("YOUR", 3),
        ] {
            whos_on_first_positions.insert(whos_translate(label), position);
        }

        let mut whos_on_first_buttons: HashMap<String, Vec<String>> = HashMap::new();
        for (label, v) in [
            ("BLANK", vec!["WAIT", "RIGHT", "OKAY", "MIDDLE", "BLANK"]),
            (
                "DONE",
                vec![
                    "SURE", "UH HUH", "NEXT", "WHAT?", "YOUR", "UR", "YOU'RE", "HOLD", "LIKE",
                    "YOU", "U", "YOU ARE", "UH UH", "DONE",
                ],
            ),
            (
                "FIRST",
                vec![
                    "LEFT", "OKAY", "YES", "MIDDLE", "NO", "RIGHT", "NOTHING", "UHHH", "WAIT",
                    "READY", "BLANK", "WHAT", "PRESS", "FIRST",
                ],
            ),
            (
                "HOLD",
                vec![
                    "YOU ARE", "U", "DONE", "UH UH", "YOU", "UR", "SURE", "WHAT?", "YOU'RE",
                    "NEXT", "HOLD",
                ],
            ),
            ("LEFT", vec!["RIGHT", "LEFT"]),
            (
                "LIKE",
                vec![
                    "YOU'RE", "NEXT", "U", "UR", "HOLD", "DONE", "UH UH", "WHAT?", "UH HUH", "YOU",
                    "LIKE",
                ],
            ),
            (
                "MIDDLE",
                vec![
                    "BLANK", "READY", "OKAY", "WHAT", "NOTHING", "PRESS", "NO", "WAIT", "LEFT",
                    "MIDDLE",
                ],
            ),
            (
                "NEXT",
                vec!["WHAT?", "UH HUH", "UH UH", "YOUR", "HOLD", "SURE", "NEXT"],
            ),
            (
                "NO",
                vec![
                    "BLANK", "UHHH", "WAIT", "FIRST", "WHAT", "READY", "RIGHT", "YES", "NOTHING",
                    "LEFT", "PRESS", "OKAY", "NO",
                ],
            ),
            (
                "NOTHING",
                vec![
                    "UHHH", "RIGHT", "OKAY", "MIDDLE", "YES", "BLANK", "NO", "PRESS", "LEFT",
                    "WHAT", "WAIT", "FIRST", "NOTHING",
                ],
            ),
            (
                "OKAY",
                vec![
                    "MIDDLE", "NO", "FIRST", "YES", "UHHH", "NOTHING", "WAIT", "OKAY",
                ],
            ),
            ("PRESS", vec!["RIGHT", "MIDDLE", "YES", "READY", "PRESS"]),
            (
                "RIGHT",
                vec![
                    "YES", "NOTHING", "READY", "PRESS", "NO", "WAIT", "WHAT", "RIGHT",
                ],
            ),
            (
                "SURE",
                vec![
                    "YOU ARE", "DONE", "LIKE", "YOU'RE", "YOU", "HOLD", "UH HUH", "UR", "SURE",
                ],
            ),
            (
                "U",
                vec![
                    "UH HUH", "SURE", "NEXT", "WHAT?", "YOU'RE", "UR", "UH UH", "DONE", "U",
                ],
            ),
            ("UH HUH", vec!["UH HUH"]),
            (
                "UH UH",
                vec!["UR", "U", "YOU ARE", "YOU'RE", "NEXT", "UH UH"],
            ),
            (
                "UHHH",
                vec![
                    "READY", "NOTHING", "LEFT", "WHAT", "OKAY", "YES", "RIGHT", "NO", "PRESS",
                    "BLANK", "UHHH",
                ],
            ),
            ("UR", vec!["DONE", "U", "UR"]),
            (
                "WAIT",
                vec![
                    "UHHH", "NO", "BLANK", "OKAY", "YES", "LEFT", "FIRST", "PRESS", "WHAT", "WAIT",
                ],
            ),
            ("WHAT", vec!["UHHH", "WHAT"]),
            (
                "WHAT?",
                vec![
                    "YOU", "HOLD", "YOU'RE", "YOUR", "U", "DONE", "UH UH", "LIKE", "YOU ARE",
                    "UH HUH", "UR", "NEXT", "WHAT?",
                ],
            ),
            (
                "YES",
                vec![
                    "OKAY", "RIGHT", "UHHH", "MIDDLE", "FIRST", "WHAT", "PRESS", "READY",
                    "NOTHING", "YES",
                ],
            ),
            (
                "YOU ARE",
                vec![
                    "YOUR", "NEXT", "LIKE", "UH HUH", "WHAT?", "DONE", "UH UH", "HOLD", "YOU", "U",
                    "YOU'RE", "SURE", "UR", "YOU ARE",
                ],
            ),
            (
                "YOU",
                vec![
                    "SURE", "YOU ARE", "YOUR", "YOU'RE", "NEXT", "UH HUH", "UR", "HOLD", "WHAT?",
                    "YOU",
                ],
            ),
            ("YOU'RE", vec!["YOU", "YOU'RE"]),
            ("YOUR", vec!["UH UH", "YOU ARE", "UH HUH", "YOUR"]),
            (
                "READY",
                vec![
                    "YES", "OKAY", "WHAT", "MIDDLE", "LEFT", "PRESS", "RIGHT", "BLANK", "READY",
                ],
            )] {
                whos_on_first_buttons.insert(whos_translate(label), v.iter().map(|word| whos_translate(word)).collect());
            }

        Self {
            module: Module::Menu,
            state: 0,
            label: String::new(),
            painter: ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("overlay"),
            )),
            keypad: HashMap::new(),
            simon_says: SimonSays::default(),
            whos_on_first_positions,
            whos_on_first_buttons,
            whos_on_first: [
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            memory: Memory::default(),
            wire_sequence: WireSequence::default(),
            password: [
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            keypad_image: RetainedImage::from_image_bytes(
                "Keypad.png",
                include_bytes!("Keypad.png"),
            )
            .unwrap(),
            morse_code_image: RetainedImage::from_image_bytes(
                "MorseCode.png",
                include_bytes!("MorseCode.png"),
            )
            .unwrap(),
            mazes_image: RetainedImage::from_image_bytes("Mazes.png", include_bytes!("Mazes.png"))
                .unwrap(),
            knobs_image: RetainedImage::from_image_bytes("Knobs.png", include_bytes!("Knobs.png"))
                .unwrap(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.interact_size = Vec2::new(60.0, 30.0);
        style.text_styles.insert(
            egui::TextStyle::Name("uniform".into()),
            egui::FontId::new(15.0, egui::FontFamily::Monospace),
        );
        style.override_text_style = Some(egui::TextStyle::Name("uniform".into()));
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| match self.module {
            Module::Menu => {
                let mut modules = Module::iter();
                modules.next();
                Grid::new("menu").num_columns(3).show(ui, |ui| {
                    let mut i = 0;
                    for module in modules {
                        if ui.button(module.as_ref()).clicked() {
                            self.module = module;
                            self.state = 0;
                        }
                        if i % 3 == 2 {
                            ui.end_row();
                        }
                        i += 1;
                    }
                });
            },
            Module::Wires => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.state = 0;
                }
                if self.state != 0 && ui.button("Reset").clicked() {
                    self.state = 0;
                }
                match self.state {
                    0 => {
                        ui.label("Number of wires?");
                        if ui.button("3").clicked() {
                            self.state = 1;
                        } else if ui.button("4").clicked() {
                            self.state = 2;
                        } else if ui.button("5").clicked() {
                            self.state = 3;
                        } else if ui.button("6").clicked() {
                            self.state = 4;
                        }
                    }
                    1 => {
                        ui.label("0 red: 2\n2+ blue: last blue\n3");
                    }
                    2 => {
                        ui.label("2+ red & SN finishes odd: last red\n0 red & last yellow: 1\n1 blue: 1\n2+ yellow: 4\n2");
                    }
                    3 => {
                        ui.label("last black & SN finishes odd: 4\n0 black & 0 red: 2\n1");
                    }
                    4 => {
                        ui.label("0 yellow & SN finishes odd: 3\n1 yellow & 2+ white: 4\n0 red: last\n4");
                    }
                    s => panic!("Invalid state {s}.")
                };
            },
            Module::Button => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                ui.label("Blue abort: hold\n2+ batteries & detonate: press\nwhite & CAR: hold\n3+ batteries & FRK: press\nred & hold: press\nhold\n\nBlue: 4\nYellow: 5\n1");
            },
            Module::Keypad => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.keypad.clear();
                    self.label.clear();
                } else if ui.button("Reset").clicked() {
                    self.keypad.clear();
                    self.label.clear();
                } else {
                    ui.label(&self.label);
                    let response = self.keypad_image.show_max_size(ui, ui.available_size()).interact(egui::Sense::click());
                    if response.clicked() {
                        if let Some(screen_position) = response.interact_pointer_pos() {
                            let x = remap_clamp(screen_position.x, response.rect.min.x..=response.rect.max.x, 0.0..=4.999).floor();
                            let y = remap_clamp(screen_position.y, response.rect.min.y..=response.rect.max.y, 0.0..=5.999).floor();
                            let button = Self::KEYPAD_BUTTONS[y as usize][x as usize];
                            if button != KeypadButton::None {
                                if self.keypad.remove(&button).is_none() && self.keypad.len() < 4 {
                                    self.keypad.insert(button, 0);
                                }

                                self.keypad.values_mut().for_each(|v| *v = 0);
                                for column in 0..6 {
                                    let mut i = 1;
                                    for row in 0..7 {
                                        let button = Self::KEYPAD_COLUMNS[column][row];
                                        if self.keypad.contains_key(&button) {
                                            self.keypad.insert(button, i);
                                            i += 1;
                                        }
                                    }
                                    if i > 4 {
                                        let mut pairs = self.keypad.iter().collect::<Vec<_>>();
                                        pairs.sort_by(|a, b| a.1.cmp(b.1));
                                        self.label = pairs.iter().fold(String::new(), |mut a, b| {
                                            a.push_str(b.0.name());
                                            a.push_str(" ");
                                            a
                                        });
                                        break;
                                    } else {
                                        self.keypad.values_mut().for_each(|v| *v = 0);
                                        self.label.clear();
                                    }
                                }
                            }
                        }
                    }
                    for x in 0..5 {
                        for y in 0..6 {
                            if let Some(i) = self.keypad.get(&Self::KEYPAD_BUTTONS[y][x]) {
                                let rect_x = lerp(response.rect.min.x..=response.rect.max.x, x as f32 / 5.0);
                                let rect_y = lerp(response.rect.min.y..=response.rect.max.y, y as f32 / 6.0);
                                self.painter.rect_stroke(
                                    egui::Rect::from_min_size(Pos2::new(rect_x, rect_y), response.rect.size() / Vec2::new(5.0, 6.0)),
                                    5.0,
                                    egui::Stroke::new(10.0, if *i == 0 { Color32::RED } else { Color32::GREEN })
                                );
                                if *i > 0 {
                                    self.painter.text(
                                        Pos2::new(rect_x + 10.0, rect_y + 10.0),
                                        egui::Align2::LEFT_TOP, i.to_string(),
                                        egui::FontId::new(30.0, egui::FontFamily::Monospace), Color32::GREEN
                                    );
                                }
                            }
                        }
                    }
                }
            },
            Module::SimonSays => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.simon_says = SimonSays::default();
                }
                if ui.button("Reset").clicked() {
                    self.simon_says.entered.clear();
                }
                ui.checkbox(&mut self.simon_says.vowel, "Vowel");
                ui.add(Slider::new(&mut self.simon_says.strikes, 0..=2).text("Strikes"));
                Grid::new("simon says").show(ui, |ui| {
                    if ui.add(Button::new("   ").fill(Color32::RED)).clicked() {
                        self.simon_says.entered.push(SimonColor::Red);
                    }
                    if ui.add(Button::new("   ").fill(Color32::BLUE)).clicked() {
                        self.simon_says.entered.push(SimonColor::Blue);
                    }
                    if ui.add(Button::new("   ").fill(Color32::GREEN)).clicked() {
                        self.simon_says.entered.push(SimonColor::Green);
                    }
                    if ui.add(Button::new("   ").fill(Color32::YELLOW)).clicked() {
                        self.simon_says.entered.push(SimonColor::Yellow);
                    }
                    ui.end_row();

                    ui.label("Flash");
                    ui.label("Press");
                    ui.end_row();

                    for color in &self.simon_says.entered {
                        ui.label(RichText::new("   ").background_color(color.color32()));
                        ui.label(RichText::new("   ").background_color(self.simon_says.convert(color).color32()));
                        ui.end_row();
                    }
                });
            },
            Module::WhosOnFirst => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.whos_on_first.iter_mut().for_each(|s| s.clear());
                    self.state = 0;
                    self.label.clear();
                }
                if ui.button("Reset").clicked() {
                    self.whos_on_first.iter_mut().for_each(|s| s.clear());
                    self.state = 0;
                }
                
                let mut changed = false;
                let response = Frame::none().stroke(egui::Stroke { width: 10.0, color: if self.state == 0 {Color32::GOLD} else {Color32::TRANSPARENT}}).show(ui, |ui| {
                    let color = if self.whos_on_first_positions.contains_key(&self.whos_on_first[0]) {Color32::GREEN} else {Color32::RED};
                    ui.add(TextEdit::singleline(&mut self.whos_on_first[0]).text_color(color).desired_width(210.0))
                });
                ui.end_row();
                if response.inner.changed() {
                    changed = true;
                    self.whos_on_first[0].make_ascii_uppercase();
                }
                if response.inner.clicked() {
                    self.state = 0;
                }
                if response.inner.has_focus() {
                    self.state = 0;
                }
                Grid::new("whos on first").num_columns(2).min_col_width(100.0).spacing((10.0, 10.0)).show(ui, |ui| {
                    for i in 1..7 {
                        let response = Frame::none().stroke(egui::Stroke { width: 10.0, color: if self.state == i {Color32::GOLD} else {Color32::TRANSPARENT}}).show(ui, |ui| {
                        let color = if self.whos_on_first_buttons.contains_key(&self.whos_on_first[i]) {Color32::GREEN} else {Color32::RED};
                        ui.add(TextEdit::singleline(&mut self.whos_on_first[i]).text_color(color).desired_width(100.0))
                        });
                        if response.inner.changed() {
                            changed = true;
                            self.whos_on_first[i].make_ascii_uppercase();
                        }
                        if response.inner.clicked() {
                            self.state = i;
                        }
                        if response.inner.has_focus() {
                            self.state = i;
                        }

                        if i % 2 == 0 {
                            ui.end_row();
                        }
                    }
                });
                
                #[cfg(target_os = "android")]
                {changed = keyboard(ui, &mut self.whos_on_first[self.state]);}

                if changed {
                    let button = if let Some((_, position)) = self.whos_on_first_positions.iter().find(|(word, _)| **word == self.whos_on_first[0]) {
                        let label = &self.whos_on_first[*position + 1];
                        if let Some((_, buttons)) = self.whos_on_first_buttons.iter().find(|(word, _)| *word == label) {
                            buttons.iter().find(|word| self.whos_on_first[1..7].contains(word))
                        } else {
                            None
                        }
                    } else {None};
                    if let Some(button) = button {
                        self.label = button.clone();
                    } else {
                        self.label = String::from("????");
                    }
                }
                ui.label(&self.label);
            },
            Module::Memory => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                match self.state {
                    0 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        self.memory = Memory::default();
                        ui.label("Stage 1. Displayed:");
                        if ui.button("1: position 2").clicked() {
                            self.memory.position1 = 2;
                            self.state = 1;
                        } else if ui.button("2: position 2").clicked() {
                            self.memory.position1 = 2;
                            self.state = 1;
                        } else if ui.button("3: position 3").clicked() {
                            self.memory.position1 = 3;
                            self.state = 1;
                        } else if ui.button("4: position 4").clicked() {
                            self.memory.position1 = 4;
                            self.state = 1;
                        }
                    }
                    1 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Label from stage 1:");
                        for i in 1..=4 {
                            if ui.button(i.to_string()).clicked() {
                                self.memory.label1 = i;
                                self.state = 2;
                            }
                        }
                    }
                    2 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Stage 2. Displayed:");
                        if ui.button("1: label 4").clicked() {
                            self.memory.label2 = 4;
                            self.state = 4;
                        } else if ui.button(format!("2: position {}", self.memory.position1)).clicked() {
                            self.memory.position2 = self.memory.position1;
                            self.state = 3;
                        } else if ui.button("3: position 1").clicked() {
                            self.memory.position2 = 1;
                            self.state = 3;
                        } else if ui.button(format!("4: position {}", self.memory.position1)).clicked() {
                            self.memory.position2 = self.memory.position1;
                            self.state = 3;
                        }
                    }
                    3 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Label from stage 2:");
                        for i in 1..=4 {
                            if ui.button(i.to_string()).clicked() {
                                self.memory.label2 = i;
                                self.state = 5;
                            }
                        }
                    }
                    4 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Position from stage 2:");
                        for i in 1..=4 {
                            if ui.button(i.to_string()).clicked() {
                                self.memory.position2 = i;
                                self.state = 5;
                            }
                        }
                    }
                    5 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Stage 3. Displayed:");
                        if ui.button(format!("1: label {}", self.memory.label2)).clicked() {
                            self.memory.label3 = self.memory.label2;
                            self.state = 7;
                        } else if ui.button(format!("2: label {}", self.memory.label1)).clicked() {
                            self.memory.label3 = self.memory.label1;
                            self.state = 7;
                        } else if ui.button("3: position 3").clicked() {
                            self.state = 6;
                        } else if ui.button("4: label 4").clicked() {
                            self.memory.label3 = 4;
                            self.state = 7;
                        }
                    }
                    6 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Label from stage 3:");
                        for i in 1..=4 {
                            if ui.button(i.to_string()).clicked() {
                                self.memory.label3 = i;
                                self.state = 7;
                            }
                        }
                    }
                    7 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Stage 4. Displayed:");
                        if ui.button(format!("1: position {}", self.memory.position1)).clicked() {
                            self.state = 8;
                        } else if ui.button("2: position 1").clicked() {
                             self.state = 8;
                        } else if ui.button(format!("3: position {}", self.memory.position2)).clicked() {
                            self.state = 8;
                        } else if ui.button(format!("4: position {}", self.memory.position2)).clicked() {
                            self.state = 8;
                        }
                    }
                    8 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Label from stage 4:");
                        for i in 1..=4 {
                            if ui.button(i.to_string()).clicked() {
                                self.memory.label4 = i;
                                self.state = 9;
                            }
                        }
                    }
                    9 => {
                        if ui.button("Reset").clicked() {
                            self.state = 0;
                        }
                        ui.label("Stage 5. Displayed:");
                        let _ = ui.button(format!("1: label {}", self.memory.label1));
                        let _ = ui.button(format!("2: label {}", self.memory.label2));
                        let _ = ui.button(format!("3: label {}", self.memory.label4));
                        let _ = ui.button(format!("4: label {}", self.memory.label3));
                    }
                    s => panic!("Invalid state {s}.")
                }
                ui.label(RichText::new(format!(
                    "Position Label\n{}        {}\n{}        {}\nX        {}\nX        {}\n",
                    self.memory.position1, self.memory.label1, self.memory.position2,
                    self.memory.label2, self.memory.label3, self.memory.label4
                )));
            },
            Module::MorseCode => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                self.morse_code_image.show_max_size(ui, ui.available_size());
            },
            Module::ComplicatedWires => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.state = 0;
                }
                if ui.button("Reset").clicked() {
                    self.state = 0;
                }
                Grid::new("complicated wires").num_columns(4).show(ui, |ui| {
                    let mut i = 0;
                    for label in ["LED", "STAR", "BLUE", "RED"] {
                        if ui.add(Button::new(RichText::new(label).color(Color32::BLACK)).fill(
                            if self.state & (1 << i) == 0 { Color32::RED } else { Color32::GREEN }
                        ).min_size(Vec2::new(40.0, 30.0))).clicked() {
                            self.state ^= 1 << i;
                        }
                        i += 1;
                    }
                });
                ui.label(format!("Cut when: {}", Self::COMPLICATED_WIRES[self.state]));
            }
            Module::WireSequences => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.wire_sequence = WireSequence::default();
                }
                if ui.button("Reset").clicked() {
                    self.wire_sequence = WireSequence::default();
                }
                if ui.button(format!("Red: {}", Self::WIRE_SEQUENCE[(self.wire_sequence.red) as usize])).clicked() && self.wire_sequence.red < 8 {
                    self.wire_sequence.red += 1;
                }
                ui.add(Slider::new(&mut self.wire_sequence.red, 0..=8));
                if ui.button(format!("Blue: {}", Self::WIRE_SEQUENCE[(self.wire_sequence.blue + 9) as usize])).clicked() && self.wire_sequence.blue < 8 {
                    self.wire_sequence.blue += 1;
                }
                ui.add(Slider::new(&mut self.wire_sequence.blue, 0..=8));
                if ui.button(format!("Black: {}", Self::WIRE_SEQUENCE[(self.wire_sequence.black + 18) as usize])).clicked() && self.wire_sequence.black < 8 {
                    self.wire_sequence.black += 1;
                }
                ui.add(Slider::new(&mut self.wire_sequence.black, 0..=8));
            },
            Module::Mazes => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                self.mazes_image.show_max_size(ui, ui.available_size());
            },
            Module::Passwords => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.state = 0;
                    self.label.clear();
                    self.password.iter_mut().for_each(|f| f.clear());
                }
                if ui.button("Reset").clicked() {
                    self.state = 0;
                    self.label.clear();
                    self.password.iter_mut().for_each(|f| f.clear());
                }

                let label_rect = ui.label(&self.label).rect;
                ui.allocate_exact_size(Vec2::new(0.0, 90.0 - label_rect.height()), egui::Sense::hover());

                let mut changed = false;
                Grid::new("password").num_columns(2).min_col_width(0.0).spacing((10.0, 10.0)).show(ui, |ui| {
                    for i in 0..5 {
                        if i == self.state {
                            ui.label(RichText::new((i + 1).to_string()).color(Color32::GOLD));
                        } else {
                            ui.label((i + 1).to_string());
                        }

                        let response = Frame::none().stroke(egui::Stroke { width: 10.0, color: if self.state == i {Color32::GOLD} else {Color32::TRANSPARENT}}).show(ui, |ui| {
                            ui.add(TextEdit::singleline(&mut self.password[i]).desired_width(100.0))
                        });
                        if response.inner.changed() {
                            changed = true;
                            self.password[i].make_ascii_uppercase();
                        }
                        if response.inner.clicked() {
                            self.state = i;
                        }

                        ui.end_row();
                    }
                });

                #[cfg(target_os = "android")]
                {keyboard(ui, &mut self.password[self.state]);}

                if changed {
                    self.label = Self::PASSWORDS.iter().filter(|word| {
                        for (i, c) in word.chars().enumerate() {
                            if self.password[i].len() > 0 && !self.password[i].contains(c) {
                                return false;
                            }
                        }
                        return true;
                    }).fold(String::new(), |mut a, b| {
                        a.push_str(b);
                        a.push_str(" ");
                        a
                    });
                }
            },
            Module::Knobs => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                self.knobs_image.show_max_size(ui, ui.available_size());
            }
        });
        self.painter.text(
            ctx.screen_rect().center_bottom(),
            egui::Align2::CENTER_BOTTOM,
            concat!(
                "KTANE Manual ",
                env!("CARGO_PKG_VERSION"),
                " ",
                env!("GIT_HASH")
            ),
            egui::FontId::monospace(10.0),
            Color32::GRAY,
        );
    }
}
