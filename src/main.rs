//#![windows_subsystem = "windows"]

use std::{
    collections::{HashMap, HashSet},
    ops::Index,
};

use egui::{
    emath::inverse_lerp, lerp, remap_clamp, Button, Color32, Key, Painter, Pos2, Rect, RichText,
    Rounding, Slider, Stroke, Style, TextStyle, Vec2,
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
    AntiN,
    Omega,
}

impl KeypadButton {
    fn name(&self) -> &str {
        match self {
            KeypadButton::AntiC => "Anti-C",
            KeypadButton::EmptyStar => "Empty Stat",
            KeypadButton::N6 => "6",
            KeypadButton::FilledStar => "Filled Star",
            KeypadButton::AntiN => "Anti-N",
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

struct Application {
    module: Module,
    state: usize,
    label: String,
    painter: Painter,
    keypad: HashMap<KeypadButton, u8>,
    simon_says: SimonSays,
    memory: Memory,
    wire_sequence: WireSequence,
    password: [String; 5],
    keypad_image: RetainedImage,
    morse_code_image: RetainedImage,
    mazes_image: RetainedImage,
    knobs_image: RetainedImage,
}

impl Application {
    const MAX_IMAGE_SIZE: Vec2 = Vec2::new(500.0, 500.0);
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
            KeypadButton::AntiN,
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
            KeypadButton::AntiN,
            KeypadButton::Omega,
        ],
    ];
    const WHOS_ON_FIRST_POSITIONS: [(&str, &str); 28] = [
        ("YES", "MIDDLE LEFT"),
        ("FIRST", "TOP RIGHT"),
        ("DISPLAY", "BOTTOM RIGHT"),
        ("OKAY", "TOP RIGHT"),
        ("SAYS", "BOTTOM LEFT"),
        ("NOTHING", "MIDDLE LEFT"),
        ("---", "BOTTOM LEFT"),
        ("BLANK", "MIDDLE RIGHT"),
        ("NO", "BOTTOM RIGHT"),
        ("LED", "MIDDLE LEFT"),
        ("LEAD", "BOTTOM RIGHT"),
        ("READ", "MIDDLE RIGHT"),
        ("RED", "MIDDLE RIGHT"),
        ("REED", "BOTTOM LEFT"),
        ("LEED", "BOTTOM LEFT"),
        ("HOLD ON", "BOTTOM RIGHT"),
        ("YOU", "MIDDLE RIGHT"),
        ("YOU ARE", "BOTTOM RIGHT"),
        ("YOUR", "MIDDLE RIGHT"),
        ("YOU'RE", "MIDDLE RIGHT"),
        ("UR", "TOP LEFT"),
        ("THERE", "BOTTOM RIGHT"),
        ("THEY'RE", "BOTTOM LEFT"),
        ("THEIR", "MIDDLE RIGHT"),
        ("THEY ARE", "MIDDLE LEFT"),
        ("SEE", "BOTTOM RIGHT"),
        ("C", "TOP RIGHT"),
        ("CEE", "BOTTOM RIGHT"),
    ];
    const WHOS_ON_FIRST_BUTTONS: [(&str, &str); 28] = [
        ("BLANK", "WAIT, RIGHT, OKAY, MIDDLE, BLANK"),
        ("DONE", "SURE, UH HUH, NEXT, WHAT?, YOUR, UR, YOU'RE, HOLD, LIKE, YOU, U, YOU ARE, UH UH, DONE"),
        ("FIRST", "LEFT, OKAY, YES, MIDDLE, NO, RIGHT, NOTHING, UHHH, WAIT, READY, BLANK, WHAT, PRESS, FIRST"),
        ("HOLD", "YOU ARE, U, DONE, UH UH, YOUR, UR, SURE, WHAT?, YOU'RE, NEXT, HOLD"),
        ("LEFT", "RIGHT, LEFT"),
        ("LIKE", "YOU'RE, NEXT, U, UR, HOLD, DONE, UH UH, WHAT?, UH UH, YOU, LIKE"),
        ("MIDDLE", "BLANK, READY, OKAY, WHAT, NOTHING, PRESS, NO, WAIT, LEFT, MIDDLE"),
        ("NEXT", "WHAT?, UH HUH, UH UH, YOUR, HOLD, SURE, NEXT"),
        ("NO", "BLANK, UHHH, WAIT, FIRST, WHAT, READY, RIGHT, YES, NOTHING, LEFT, PRESS, OKAY, NO"),
        ("NOTHING", "UHHH, RIGHT, OKAY, MIDDLE, YES, BLANK, NO, PRESS, LEFT, WHAT, WAIT, FIRST, NOTHING"),
        ("OKAY", "MIDDLE, NO, FIRST, YES, UHHH, NOTHING, WAIT, OKAY"),
        ("PRESS", "RIGHT, MIDDLE, YES, READY, PRESS"),
        ("READY", "YES, OKAY, WHAT, MIDDLE, LEFT, PRESS, RIGHT, BLANK, READY"),
        ("RIGHT", "YES, NOTHING, READY, PRESS, NO, WAIT, WHAT, RIGHT"),
        ("SURE", "YOU ARE, DONE, LIKE, YOU'RE, YOUR, HOLD, UH HUH, UR, SURE"),
        ("U", "UH HUH, SURE, NEXT, WHAT?, YOU'RE, UR, UH UH, DONE, UH UH"),
        ("UR", "DONE, U, UR"),
        ("UH HUH", "UH HUH"),
        ("UH UH", "UR, U, YOU ARE, YOU'RE, NEXT, UH UH"),
        ("UHHH", "READY, NOTHING, LEFT, WHAT, OKAY, YES, RIGHT, NO, PRESS, BLANK, UHHH"),
        ("WAIT", "UHHH, NO, BLANK, OKAY, YES, LEFT, FIRST, PRESS, WHAT, WAIT"),
        ("WHAT", "UHHH, WHAT"),
        ("WHAT?", "YOU , HOLD, YOU'RE, YOUR, U, DONE, UH UH, LIKE, YOU ARE, UH HUH, UR, NEXT, WHAT?"),
        ("YES", "OKAY, RIGHT, UHHH, MIDDLE, FIRST, WHAT, PRESS, READY, NOTHING, YES"),
        ("YOU", "SURE, YOU ARE, YOUR, YOU'RE, NEXT, UH HUH, UR, HOLD, WHAT?, YOU"),
        ("YOUR", "UH UH, YOU ARE, UH HUH, YOU"),
        ("YOU'RE", "YOU, YOU'RE"),
        ("YOU ARE", "YOUR, NEXT, LIKE, UH HUH, WHAT?, DONE, UH UH, HOLD, YOU, U, YOU'RE, SURE, UR, YOU ARE"),
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

    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            module: Module::Menu,
            state: 0,
            label: String::new(),
            painter: cc.egui_ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("overlay"),
            )),
            keypad: HashMap::new(),
            simon_says: SimonSays::default(),
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
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.interact_size = Vec2::new(60.0, 30.0);
        style.override_text_style = Some(egui::TextStyle::Heading);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| match self.module {
            Module::Menu => {
                let mut modules = Module::iter();
                modules.next();
                egui::Grid::new("menu").num_columns(3).show(ui, |ui| {
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
                        ui.monospace("Number of wires?");
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
                        ui.monospace("0 red: 2\n2+ blue: last blue\n3");
                    }
                    2 => {
                        ui.monospace("2+ red & SN finishes odd: last red\n0 red & last yellow: 1\n1 blue: 1\n2+ yellow: 4\n2");
                    }
                    3 => {
                        ui.monospace("last black & SN finishes odd: 4\n0 black & 0 red: 2\n1");
                    }
                    4 => {
                        ui.monospace("0 yellow & SN finishes odd: 3\n1 yellow & 2+ white: 4\n0 red: last\n4");
                    }
                    s => panic!("Invalid state {s}.")
                };
            },
            Module::Button => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                ui.monospace("Blue abort: hold\n2+ batteries & detonate: press\nwhite & CAR: hold\n3+ batteries & FRK: press\nred & hold: press\nhold\n\nBlue: 4\nYellow: 5\n1");
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
                    ui.monospace(&self.label);
                    let response = self.keypad_image.show_max_size(ui, Self::MAX_IMAGE_SIZE).interact(egui::Sense::click());
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
                                    Rect::from_min_size(Pos2::new(rect_x, rect_y), response.rect.size() / Vec2::new(5.0, 6.0)),
                                    5.0,
                                    Stroke::new(10.0, if *i == 0 { Color32::RED } else { Color32::GREEN })
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
                egui::Grid::new("simon says").show(ui, |ui| {
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

                    ui.monospace("Flash");
                    ui.monospace("Press");
                    ui.end_row();

                    for color in &self.simon_says.entered {
                        ui.monospace(RichText::new("   ").background_color(color.color32()));
                        ui.monospace(RichText::new("   ").background_color(self.simon_says.convert(color).color32()));
                        ui.end_row();
                    }
                });
            },
            Module::WhosOnFirst => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                if self.state == 0 {
                    ui.monospace("Displayed word:");
                    egui::Grid::new("positions").num_columns(5).show(ui, |ui| {
                        let mut i = 1;
                        let mut j = 0;
                        for (word, _) in Self::WHOS_ON_FIRST_POSITIONS {
                            if ui.button(word).clicked() {
                                self.state = i;
                                break;
                            }
                            i += 1;
                            if j % 5 == 4 {
                                ui.end_row();
                            }
                            j += 1;
                        }
                    });
                } else if self.state <= Self::WHOS_ON_FIRST_POSITIONS.len() {
                    if ui.button("Reset").clicked() {
                        self.state = 0;
                    } else {
                        let i = self.state - 1;
                        ui.monospace(format!("{}: {}", Self::WHOS_ON_FIRST_POSITIONS[i].0, Self::WHOS_ON_FIRST_POSITIONS[i].1));
                        egui::Grid::new("words").num_columns(5).show(ui, |ui| {
                            let mut i = Self::WHOS_ON_FIRST_POSITIONS.len() + 1;
                            let mut j = 0;
                            for (word, _) in Self::WHOS_ON_FIRST_BUTTONS {
                                if ui.button(word).clicked() {
                                    self.state = i;
                                    break;
                                }
                                i += 1;
                                if j % 5 == 4 {
                                    ui.end_row();
                                }
                                j += 1;
                            }
                        });
                    }
                } else if self.state <= Self::WHOS_ON_FIRST_POSITIONS.len() + Self::WHOS_ON_FIRST_BUTTONS.len() {
                    if ui.button("Reset").clicked() {
                        self.state = 0;
                    } else {
                        let i = self.state - Self::WHOS_ON_FIRST_POSITIONS.len() - 1;
                        ui.monospace(format!("{}: {}", Self::WHOS_ON_FIRST_BUTTONS[i].0, Self::WHOS_ON_FIRST_BUTTONS[i].1));
                    }
                }
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
                        ui.monospace("Stage 1. Displayed:");
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
                        ui.monospace("Label from stage 1:");
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
                        ui.monospace("Stage 2. Displayed:");
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
                        ui.monospace("Label from stage 2:");
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
                        ui.monospace("Position from stage 2:");
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
                        ui.monospace("Stage 3. Displayed:");
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
                        ui.monospace("Label from stage 3:");
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
                        ui.monospace("Stage 4. Displayed:");
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
                        ui.monospace("Label from stage 4:");
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
                        ui.monospace("Stage 5. Displayed:");
                        let _ = ui.button(format!("1: label {}", self.memory.label1));
                        let _ = ui.button(format!("2: label {}", self.memory.label2));
                        let _ = ui.button(format!("3: label {}", self.memory.label4));
                        let _ = ui.button(format!("4: label {}", self.memory.label3));
                    }
                    s => panic!("Invalid state {s}.")
                }
                ui.monospace(RichText::new(format!(
                    "Position Label\n{}        {}\n{}        {}\nX        {}\nX        {}\n",
                    self.memory.position1, self.memory.label1, self.memory.position2,
                    self.memory.label2, self.memory.label3, self.memory.label4
                )).monospace());
            },
            Module::MorseCode => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                self.morse_code_image.show_max_size(ui, Self::MAX_IMAGE_SIZE);
            },
            Module::ComplicatedWires => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.state = 0;
                }
                if ui.button("Reset").clicked() {
                    self.state = 0;
                }
                egui::Grid::new("complicated wires").num_columns(4).show(ui, |ui| {
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
                ui.monospace(format!("Cut when: {}", Self::COMPLICATED_WIRES[self.state]));
            }
            Module::WireSequences => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.wire_sequence = WireSequence::default();
                }
                if ui.button("Reset").clicked() {
                    self.wire_sequence = WireSequence::default();
                }
                egui::Grid::new("wire sequence").num_columns(3).show(ui, |ui| {
                    if ui.button(format!("Red: {}", Self::WIRE_SEQUENCE[(self.wire_sequence.red) as usize])).clicked() && self.wire_sequence.red < 8 {
                        self.wire_sequence.red += 1;
                    }
                    if ui.button(format!("Blue: {}", Self::WIRE_SEQUENCE[(self.wire_sequence.blue + 9) as usize])).clicked() && self.wire_sequence.blue < 8 {
                        self.wire_sequence.blue += 1;
                    }
                    if ui.button(format!("Black: {}", Self::WIRE_SEQUENCE[(self.wire_sequence.black + 18) as usize])).clicked() && self.wire_sequence.black < 8 {
                        self.wire_sequence.black += 1;
                    }
                    ui.end_row();
                    ui.add(Slider::new(&mut self.wire_sequence.red, 0..=8));
                    ui.add(Slider::new(&mut self.wire_sequence.blue, 0..=8));
                    ui.add(Slider::new(&mut self.wire_sequence.black, 0..=8));
                });
            },
            Module::Mazes => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                self.mazes_image.show_max_size(ui, Self::MAX_IMAGE_SIZE);
            },
            Module::Passwords => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                    self.label.clear();
                    self.password.iter_mut().for_each(|f| f.clear());
                }
                if ui.button("Reset").clicked() {
                    self.label.clear();
                    self.password.iter_mut().for_each(|f| f.clear());
                }
                ui.monospace(&self.label);
                egui::Grid::new("password").num_columns(2).show(ui, |ui| {
                    for i in 0..5 {
                        ui.monospace(i.to_string());
                        if ui.text_edit_singleline(&mut self.password[i]).changed() {
                            self.password[i].make_ascii_uppercase();
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
                        ui.end_row();
                    }
                });
            },
            Module::Knobs => {
                if ui.button("Menu").clicked() {
                    self.module = Module::Menu;
                }
                self.knobs_image.show_max_size(ui, Self::MAX_IMAGE_SIZE);
            }
        });
    }
}

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(600.0, 600.0));
    native_options.follow_system_theme = false;
    let _ = eframe::run_native(
        "KTANE",
        native_options,
        Box::new(|cc| Box::new(Application::new(cc))),
    );
}
