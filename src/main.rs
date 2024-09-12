use std::{
    fs::File,
    io::{stdout, BufReader, BufWriter, Write},
    thread::sleep,
    time::Duration,
};

use termion::cursor;

use jpeg_decoder::{Decoder, PixelFormat};

use palette::{rgb::Rgb, FromColor, Hsv};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    fn simplify(&self) -> Color {
        let r = self.red;
        let g = self.green;
        let b = self.blue;

        if r < 50 && g < 50 && b < 50 {
            return Color::black();
        }

        if r > 205 && g > 205 && b > 205 {
            return Color::white();
        }

        if r >= g && r >= b {
            if r - g < 64 {
                Color::yellow()
            } else if r - b < 64 {
                Color::magenta()
            } else {
                Color::red()
            }
        } else if g >= r && g >= b {
            if g - b < 64 {
                Color::cyan()
            } else if g - r < 64 {
                Color::yellow()
            } else {
                Color::green()
            }
        } else {
            if b - r < 64 {
                Color::magenta()
            } else if b - g < 64 {
                Color::cyan()
            } else {
                Color::blue()
            }
        }
    }

    fn clamp(&self, min: i32, max: i32, offset: i32) -> Color {
        let r = (self.red as i32 + offset).clamp(min, max);
        let g = (self.green as i32 + offset).clamp(min, max);
        let b = (self.blue as i32 + offset).clamp(min, max);
        Color::new(r as u8, g as u8, b as u8)
    }

    fn blockify(&self, strength: u8) -> Color {
        let r = self.red as f32;
        let g = self.green as f32;
        let b = self.blue as f32;
        let r = (r / strength as f32) as u8 * strength;
        let g = (g / strength as f32) as u8 * strength;
        let b = (b / strength as f32) as u8 * strength;
        Color::new(r, g, b)
    }

    fn decompose(self) -> (Color, Color, f32) {
        //let c1 = self.shift(self.blockify(16), 0.5);
        //let c2 = c1.shift(c1.simplify(), 0.8);
        let c1 = self.blockify(16).shift(Color::black(), 0.7);
        let c2 = self.blockify(16);

        /*
        let x = (c1.red as f32 + c1.green as f32 + c1.blue as f32) as f32;
        let y = (c2.red as f32 + c2.green as f32 + c2.blue as f32) as f32;
        let res = (r as f32 + g as f32 + b as f32)
            + (x + y)
            / ((x + y - self.brightness() as f32).abs() / 20.0);
        let weight = ((res - x) / (y - x)).clamp(0.0, 1.0);
        */

        let weight = self.brightness() as f32 / 255.0;

        (c2, c1, weight)
    }

    fn red() -> Color {
        Color::new(225, 30, 30)
    }

    fn yellow() -> Color {
        Color::new(225, 225, 30)
    }

    fn magenta() -> Color {
        Color::new(225, 30, 225)
    }

    fn green() -> Color {
        Color::new(30, 225, 30)
    }

    fn cyan() -> Color {
        Color::new(30, 225, 225)
    }

    fn blue() -> Color {
        Color::new(30, 30, 225)
    }

    fn black() -> Color {
        Color::new(10, 10, 10)
    }

    fn white() -> Color {
        Color::new(225, 225, 225)
    }

    fn distance(&self, other: Color) -> f64 {
        let r1 = self.red as f64;
        let g1 = self.green as f64;
        let b1 = self.blue as f64;
        let r2 = other.red as f64;
        let g2 = other.green as f64;
        let b2 = other.blue as f64;
        let r = (r1 - r2).powf(2.0);
        let g = (g1 - g2).powf(2.0);
        let b = (b1 - b2).powf(2.0);
        (r + g + b).sqrt()
    }

    fn brightness(&self) -> f64 {
        (self.red as f64 + self.green as f64 + self.blue as f64) / 3.0
    }

    fn shift(&self, other: Color, power: f32) -> Color {
        Color::new(
            (self.red as f32 * (1.0 - power) + other.red as f32 * power) as u8,
            (self.green as f32 * (1.0 - power) + other.green as f32 * power) as u8,
            (self.blue as f32 * (1.0 - power) + other.blue as f32 * power) as u8,
        )
    }

    fn to_hsv(self) -> Hsv {
        Hsv::from_color(Rgb::new(
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
        ))
    }

    fn from_hsv(hsv: Hsv) -> Self {
        let rgb = Rgb::from_color(hsv);
        Self::new(
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    char: char,
    fg: Color,
    bg: Color,
}

impl Cell {
    fn new(char: char, fg: Color, bg: Color) -> Self {
        Self { char, fg, bg }
    }

    fn from_color(color: Color) -> Self {
        //let s = ".,:;!•ag?$&@";
        let s = "`'~!,-\":|\\;/(<>)]+[{}i731t2sy*ur5o=dea49p6q&8w€¥0$%@#";
        let count = s.chars().count();
        let (c1, c2, w) = color.decompose();
        let w = ((w * (count - 1) as f32) - 0.5) as usize;
        let w = w.clamp(0, count - 1);
        Cell::new(s.chars().nth(w).unwrap(), c1, c2)
    }

    /*
        fn _from_color(color: Color) -> Self {
            let s = ".,:•&@";
            let chars = s.chars().collect::<Vec<_>>();
            let colors: Vec<Color> = vec![];
            let mut cell = Cell::new(' ', Color::new(0, 0, 0), Color::new(0, 0, 0));
            let mut dist = 1000.0;

            for c1 in colors.iter() {
                for c2 in colors.iter() {
                    for i in 0..chars.len() {
                        let d = (i as f64) / chars.len() as f64 + (1.0 / chars.len() as f64 / 2.0);
                        let d = d - 0.5;
                        let d = d / 1.5 + 0.5;

                        let x = (d);
                        let y = (1.1 / d); // !!
                                           //

                        /*let x: f64;
                                  let y: f64;
                                  if i >= chars.len() {
                                    let n = chars.len() * 2 - i - 1;
                                    let d = n as f64 / chars.len() as f64;
                                    x = d;
                                    y = 1.0 - d;
                                  } else {
                                    let d = i as f64 / chars.len() as f64;
                                    x = d;
                                    y = 1.0 - d;
                                  }
                        */

                        let r = c1.red as f64 * x + c2.red as f64 * y;
                        let g = c1.green as f64 * x + c2.green as f64 * y;
                        let b = c1.blue as f64 * x + c2.blue as f64 * y;
                        let c = Color::new((r) as u8, (g) as u8, (b) as u8);
                        let d = color.distance(c);
                        let br = (c1.brightness() - c2.brightness()).abs() / 20.0;
                        if d + br < dist {
                            dist = d + br;
                            cell.char = *chars.get(i).unwrap();
                            cell.fg = c1.clone();
                            cell.bg = c2.clone();
                        }
                    }
                }
            }

            cell
        }
    */

    fn render(&self) -> String {
        let fg = (self.fg.red, self.fg.green, self.fg.blue);
        let bg = (self.bg.red, self.bg.green, self.bg.blue);
        format!(
            "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}",
            fg.0, fg.1, fg.2, bg.0, bg.1, bg.2, self.char
        )
    }
}

#[derive(Clone)]
enum Renderable {
    DialogBox(DialogBox),
    Rectangle(Rectangle, Cell, f32),
    Circle(Circle, Cell),
}

impl Renderable {
    fn render(&self, canvas: &mut Canvas) -> Canvas {
        match self {
            Renderable::DialogBox(dialog) => canvas.draw_dialog_box(dialog.clone()),
            Renderable::Rectangle(rect, cell, alpha) => canvas.draw_rectangle(rect.clone(), cell.clone(), *alpha),
            Renderable::Circle(circle, cell) => canvas.draw_circle(circle.clone(), cell.clone()),
        }
    }
}

#[derive(Clone)]
struct Canvas {
    cells: Vec<Vec<Cell>>,
    renderables: Vec<Renderable>,
    old_canvas: Option<Box<Canvas>>,
}

impl Canvas {
    fn height(&self) -> usize {
        self.cells.len()
    }

    fn width(&self) -> usize {
        self.cells.get(0).map(|v| v.len()).unwrap_or(0)
    }

    fn display(&mut self) {
        let mut handle = BufWriter::new(stdout());
        write!(handle, "\x1b[H").unwrap();

        let mut fg = Color::black();
        let mut bg = Color::black();
        let mut first = true;

        let mut canvas = self.clone();

        for renderable in self.renderables.clone().into_iter() {
            canvas = renderable.render(&mut canvas);
        }

        let tolerance = 20.0;

        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let cell = canvas.cells[y][x];
                let should_write = match &self.old_canvas {
                    None => true,
                    Some(old) => cell != old.cells[y][x],
                };
                if should_write {
                    write!(handle, "{}", cursor::Goto(x as u16 + 1, y as u16)).unwrap();
                    if !first
                        && (cell.fg.distance(fg) < tolerance && cell.bg.distance(bg) < tolerance)
                    {
                        write!(handle, "{}", cell.char).unwrap();
                    } else {
                        write!(handle, "\x1b[0m").unwrap();
                        write!(handle, "{}", cell.render()).unwrap();
                        fg = cell.fg;
                        bg = cell.bg;
                        first = false;
                    }
                    //let _ = handle.flush();
                }
                match &mut self.old_canvas {
                    None => {},
                    Some(old) => old.cells[y][x] = cell,
                }
            }
            first = true;
            write!(handle, "\x1b[0m\n").unwrap();
        }
        
        if let None = self.old_canvas {
            self.old_canvas = Some(Box::new(self.clone()));
        }
        self.renderables.clear();
        sleep(Duration::from_millis(66 / 4));
    }

    fn new(width: usize, height: usize) -> Self {
        let cell = Cell::from_color(Color::new(0, 0, 0));
        Self {
            cells: vec![vec![cell; width]; height],
            renderables: vec![],
            old_canvas: None,
        }
    }

    fn draw_rectangle(&mut self, rect: Rectangle, cell: Cell, alpha: f32) -> Canvas {
        let mut canvas = self.clone();
        
        for y in rect.position.y as usize..(rect.position.y + rect.size.y) as usize {
            if y >= self.height() {
                break;
            }
            for x in rect.position.x as usize..(rect.position.x + rect.size.x) as usize {
                if x >= self.width() {
                    break;
                }
                let orig = self.cells[y][x];
                let (fg1, bg1) = (orig.fg, orig.bg);
                let (fg2, bg2) = (cell.fg, cell.bg);
                let fg = fg1.shift(fg2, alpha);
                let bg = bg1.shift(bg2, alpha);
                canvas.cells[y][x] = Cell::new(cell.char, fg, bg);
            }
        }

        canvas
    }

    fn draw_circle(&mut self, circle: Circle, cell: Cell) -> Canvas {
        let mut canvas = self.clone();
        let center = Vector2::new(circle.x / 2.0, circle.y);

        for y in 0..self.height() {
            for x in 0..self.width() {
                let p = Vector2::new(x as f32 / 2.0, y as f32);
                if p.distance(center) <= circle.radius {
                    canvas.cells[y][x] = cell;
                }
            }
        }

        canvas
    }

    fn draw_dialog_box(&mut self, dialog: DialogBox) -> Canvas {
        let width = dialog.width + dialog.x_pad;
        let height = dialog.height + dialog.y_pad;

        let mut canvas = self.clone();

        let x = canvas.width() as f32 / 2.0 - dialog.width / 2.0;
        let mut xblack = Cell::from_color(Color::black());
        let mut yblack = Cell::from_color(Color::black());
        xblack.char = '─';
        xblack.fg = Color::white();
        yblack.char = '│';
        yblack.fg = Color::white();
        let base = Rectangle::new(
            Vector2::new(x as f32, dialog.position),
            Vector2::new(width, height),
        );

        canvas = canvas.draw_rectangle(base, Cell::from_color(Color::black()), 0.7);

        let mut corner = Cell::new(' ', Color::white(), Color::black());

        let x = x - 1.0;

        corner.char = '╭';
        canvas.cells[dialog.position as usize - 1][x as usize] = corner;
        corner.char = '╰';
        canvas.cells[dialog.position as usize + height as usize][x as usize] = corner;

        let left = Rectangle::new(
            Vector2::new(x, dialog.position),
            Vector2::new(1.0, height),
        );

        canvas = canvas.draw_rectangle(left, yblack, 1.0);

        let x = canvas.width() as f32 / 2.0 + width / 2.0 + 1.0;

        corner.char = '╮';
        canvas.cells[dialog.position as usize - 1][x as usize] = corner;
        corner.char = '╯';
        canvas.cells[dialog.position as usize + height as usize][x as usize] = corner;

        let right = Rectangle::new(
            Vector2::new(x as f32, dialog.position),
            Vector2::new(1.0, height),
        );
        
        canvas = canvas.draw_rectangle(right, yblack, 1.0);

        let x = canvas.width() as f32 / 2.0 - dialog.width / 2.0;
        let y = dialog.position - 1.0;
        let top = Rectangle::new(Vector2::new(x, y as f32), Vector2::new(width, 1.0));

        canvas = canvas.draw_rectangle(top, xblack, 0.9);

        let y = dialog.position + height;
        let bottom = Rectangle::new(Vector2::new(x, y as f32), Vector2::new(width, 1.0));

        canvas = canvas.draw_rectangle(bottom, xblack, 0.9);

        let mut y = dialog.position + dialog.y_pad;
        let mut x = canvas.width() as f32 / 2.0 - dialog.width / 2.0 + dialog.x_pad;
        let chars = dialog.text.chars();
        for char in chars {
            if char == '\n' {
                y += 1.0;
                x = canvas.width() as f32 / 2.0 - dialog.width / 2.0 + dialog.x_pad;
            } else {
                x += 1.0;
            }
            if y as usize >= canvas.height() {
                continue;
            }
            if x as usize >= canvas.width() {
                continue;
            }
            canvas.cells[y as usize][x as usize].char = char;
            canvas.cells[y as usize][x as usize].fg = dialog.text_color;
        }

        canvas
    }
}

#[derive(Clone, Copy)]
struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance(self, other: Vector2) -> f32 {
        ((self.x - other.x).powf(2.0)
        + (self.y - other.y).powf(2.0)).sqrt()
    }
}

#[derive(Clone)]
struct Rectangle {
    position: Vector2,
    size: Vector2,
}

impl Rectangle {
    fn new(position: Vector2, size: Vector2) -> Self {
        Self { position, size }
    }

    fn raw(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self::new(
            Vector2::new(x as f32, y as f32),
            Vector2::new(width as f32, height as f32),
        )
    }
}

#[derive(Clone)]
struct Circle {
    x: f32,
    y: f32,
    radius: f32,
}

impl Circle {
    fn new(x: f32, y: f32, radius: f32) -> Self {
        Self { x, y, radius }
    }
}

#[derive(Clone)]
struct DialogBox {
    text: String,
    width: f32,
    height: f32,
    position: f32,
    x_pad: f32,
    y_pad: f32,
    text_color: Color,
}

impl DialogBox {
    fn new(text: &str, width: f32, height: f32, position: f32) -> Self {
        Self {
            text: text.to_string(),
            width,
            height,
            position,
            x_pad: 1.0,
            y_pad: 1.0,
            text_color: Color::white(), 
        }
    }
}

fn load_jpg_as_canvas(path: &str, width: usize, height: usize) -> Option<Canvas> {
    let file = File::open(path).ok()?;
    let mut decoder = Decoder::new(BufReader::new(file));

    let pixels = decoder.decode().ok()?;
    let metadata = decoder.info()?;

    if metadata.pixel_format != PixelFormat::RGB24 {
        return None;
    }

    let mut result = vec![];

    for chunk in pixels.chunks(3 * metadata.width as usize) {
        let row = chunk
            .chunks(3)
            .map(|p| Color::new(p[0], p[1], p[2]))
            .collect::<Vec<_>>();
        result.push(row);
    }

    let mut colors = vec![vec![None; width]; height];

    for (y, row) in result.into_iter().enumerate() {
        for (x, color) in row.into_iter().enumerate() {
            let y = height * y / metadata.height as usize;
            let x = width * x / metadata.width as usize;
            match colors[y][x] {
                None => colors[y][x] = Some(color),
                Some(c) => colors[y][x] = Some(c.shift(color, 0.5)),
            }
        }
    }

    let cells = colors
        .iter()
        .map(|row| {
            row.iter()
                .map(|color| Cell::from_color(color.unwrap().clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Some(Canvas {
        cells,
        renderables: vec![],
        old_canvas: None,
    })
}

fn main() {
    let s = "You are in a dark dungeon, what do you do?";
    let mut acc = String::new();
    let img = load_jpg_as_canvas("assets/scene1.jpg", 96, 48).unwrap();

    print!("\x1b[?25l");
    print!("\x1b[2J");

    let mut canvas = img.clone();
    let mut dialog = DialogBox::new(&acc, 82.0, 5.0, 35.0);

    let mut circle = Circle::new(0.0, 0.0, 5.0);
    let circle_speed = 2.0;

    let (mut vx, mut vy) = (circle_speed, circle_speed);
    
    canvas.display();

    for char in s.chars() {
        if circle.x as usize > canvas.width() {
            vx = -circle_speed;
        }
        if circle.y as usize > canvas.height() {
            vy = -circle_speed;
        }

        if circle.x <= 0.0 {
            vx = circle_speed;
        }
        if circle.y <= 0.0 {
            vy = circle_speed;
        }

        circle.x += vx;
        circle.y += vy;
        canvas.renderables.push(Renderable::Circle(circle.clone(), Cell::from_color(Color::white())));

        acc = acc + &char.to_string();
        dialog.text = acc.clone();
        canvas.renderables.push(Renderable::DialogBox(dialog.clone()));

        canvas.display();
    }

    for i in 0..1000 {
        if circle.x as usize > canvas.width() {
            vx = -circle_speed;
        }
        if circle.y as usize > canvas.height() {
            vy = -circle_speed;
        }

        if circle.x <= 0.0 {
            vx = circle_speed;
        }
        if circle.y <= 0.0 {
            vy = circle_speed;
        }

        circle.x += vx;
        circle.y += vy;
        canvas.renderables.push(Renderable::Circle(circle.clone(), Cell::from_color(Color::white())));

        if i < 200 {
            canvas.renderables.push(Renderable::DialogBox(dialog.clone()));
        }
        
        canvas.display();
    }


    print!(
        "{}",
        cursor::Goto(canvas.width() as u16, canvas.height() as u16)
    );

    println!();

    print!("\x1b[?25h");
}
