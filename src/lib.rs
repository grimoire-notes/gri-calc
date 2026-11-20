#![allow(static_mut_refs)]

#[link(wasm_import_module = "grimoire")]
extern "C" {
    fn rect(x: i32, y: i32, w: i32, h: i32, style: i32);
    fn text(x: i32, y: i32, w: i32, h: i32, ptr: i32, len: i32, px: i32, center: i32);
    fn button(id: i32, x: i32, y: i32, w: i32, h: i32, ptr: i32, len: i32, style: i32);
    fn status(ptr: i32, len: i32);
    fn setting(key_ptr: i32, key_len: i32, out_ptr: i32, out_cap: i32) -> i32;
    fn state_get(ptr: i32, cap: i32) -> i32;
    fn state_set(ptr: i32, len: i32);
}

const MODE_BAR: i32 = 100;
const MODE_CHOICE: i32 = 101;
const UNIT_CAT: i32 = 200;
const UNIT_BACK: i32 = 210;

const SIMPLE_KEYS: [&str; 20] = [
    "AC", "+/-", "%", "/", "7", "8", "9", "x", "4", "5", "6", "-", "1", "2", "3", "+", "0", ".",
    "<", "=",
];
const SCI_KEYS: [&str; 30] = [
    "AC", "(", ")", "π", "÷", "sin", "cos", "tan", "ln", "×", "log", "√", "←", "xʸ", "−", "7",
    "8", "9", "e", "+", "4", "5", "6", "±", "=", "1", "2", "3", "0", ".",
];
const PROG_KEYS: [&str; 36] = [
    "AC", "16", "10", "2", "8", "÷", "&", "|", "^", "~", "<<", ">>", "7", "8", "9", "A", "B", "C",
    "4", "5", "6", "D", "E", "F", "1", "2", "3", "0", "+", "=", "%", "Un", "±", "←", "×", "xʸ",
];
const UNIT_CATEGORIES: [&str; 4] = ["Length", "Mass", "Data", "Temp"];

#[derive(Clone, Copy)]
struct Simple {
    buf: [u8; 24],
    len: u8,
    acc: f64,
    op: u8,
    typing: bool,
    evaluated: bool,
    error: bool,
}

impl Default for Simple {
    fn default() -> Self {
        let mut c = Simple { buf: [0; 24], len: 1, acc: 0.0, op: 0, typing: false, evaluated: false, error: false };
        c.buf[0] = b'0';
        c
    }
}

impl Simple {
    fn display(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("0")
    }
    fn set_text(&mut self, s: &str) {
        self.len = s.len().min(self.buf.len()) as u8;
        self.buf[..self.len as usize].copy_from_slice(&s.as_bytes()[..self.len as usize]);
    }
    fn text(&self) -> String { self.display().to_string() }
    fn current(&self) -> f64 { self.text().parse().unwrap_or(0.0) }
    fn show(&mut self, v: f64) {
        let s = number(v);
        self.error = s == "Error";
        self.set_text(&s);
    }
    fn reset(&mut self) { *self = Simple::default(); }
    fn digit(&mut self, d: char) {
        if self.error || self.evaluated { self.reset(); }
        if !self.typing { self.len = 0; self.typing = true; }
        if self.len == 1 && self.buf[0] == b'0' { self.len = 0; }
        if (self.len as usize) < 18 {
            self.buf[self.len as usize] = d as u8;
            self.len += 1;
        }
        if self.len == 0 { self.set_text("0"); }
    }
    fn dot(&mut self) {
        if self.error || self.evaluated { self.reset(); }
        if !self.typing { self.set_text("0"); self.typing = true; }
        if !self.text().contains('.') && (self.len as usize) < 18 {
            self.buf[self.len as usize] = b'.';
            self.len += 1;
        }
    }
    fn backspace(&mut self) {
        if self.error || self.evaluated || !self.typing { return; }
        self.len = self.len.saturating_sub(1);
        if self.len == 0 || (self.len == 1 && self.buf[0] == b'-') {
            self.set_text("0");
            self.typing = false;
        }
    }
    fn sign(&mut self) {
        if self.error { return; }
        let current = self.text();
        if current == "0" { return; }
        if current.starts_with('-') { self.set_text(&current[1..]); }
        else { let s = format!("-{current}"); self.set_text(&s); }
        self.typing = true;
        self.evaluated = false;
    }
    fn percent(&mut self) {
        if self.error { return; }
        let v = self.current() / 100.0;
        self.show(v);
        self.typing = true;
        self.evaluated = false;
    }
    fn apply(acc: f64, rhs: f64, op: u8) -> f64 {
        match op {
            1 => acc + rhs,
            2 => acc - rhs,
            3 => acc * rhs,
            4 if rhs != 0.0 => acc / rhs,
            4 => f64::NAN,
            _ => rhs,
        }
    }
    fn operator(&mut self, op: u8) {
        if self.error { self.reset(); return; }
        if self.op != 0 && self.typing { self.acc = Self::apply(self.acc, self.current(), self.op); }
        else if self.op == 0 { self.acc = self.current(); }
        self.op = op;
        self.typing = false;
        self.evaluated = false;
        let acc = self.acc;
        self.show(acc);
    }
    fn equals(&mut self) {
        if self.error || self.op == 0 { return; }
        let v = Self::apply(self.acc, self.current(), self.op);
        self.acc = v;
        self.op = 0;
        self.typing = false;
        self.evaluated = true;
        self.show(v);
    }
    fn press(&mut self, key: usize) {
        match key {
            0 => self.reset(),
            1 => self.sign(),
            2 => self.percent(),
            3 => self.operator(4),
            4 => self.digit('7'),
            5 => self.digit('8'),
            6 => self.digit('9'),
            7 => self.operator(3),
            8 => self.digit('4'),
            9 => self.digit('5'),
            10 => self.digit('6'),
            11 => self.operator(2),
            12 => self.digit('1'),
            13 => self.digit('2'),
            14 => self.digit('3'),
            15 => self.operator(1),
            16 => self.digit('0'),
            17 => self.dot(),
            18 => self.backspace(),
            19 => self.equals(),
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
struct Sci {
    expr: [u8; 80],
    len: u8,
    out: [u8; 32],
    out_len: u8,
    evaluated: bool,
    error: bool,
}
impl Default for Sci {
    fn default() -> Self {
        Sci { expr: [0; 80], len: 0, out: [0; 32], out_len: 0, evaluated: false, error: false }
    }
}
impl Sci {
    fn expr_text(&self) -> &str { std::str::from_utf8(&self.expr[..self.len as usize]).unwrap_or("0") }
    fn out_text(&self) -> &str { std::str::from_utf8(&self.out[..self.out_len as usize]).unwrap_or("") }
    fn display(&self) -> &str {
        if self.error { return "Error"; }
        if self.evaluated { return self.out_text(); }
        if self.len == 0 { "0" } else { self.expr_text() }
    }
    fn clear(&mut self) { *self = Sci::default(); }
    fn push(&mut self, s: &str) {
        let n = s.len().min(self.expr.len() - self.len as usize);
        self.expr[self.len as usize..self.len as usize + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n as u8;
        self.error = false;
    }
    fn start_from_result(&mut self) {
        if self.evaluated {
            let out = self.out_text().to_string();
            self.clear();
            self.push(&out);
        }
    }
    fn backspace(&mut self) {
        if self.error || self.evaluated { return; }
        let s = self.expr_text();
        let mut at = s.len();
        while at > 0 {
            at -= 1;
            if s.is_char_boundary(at) { break; }
        }
        self.len = at as u8;
    }
    fn sign(&mut self) {
        self.start_from_result();
        if self.len == 0 { self.push("-"); }
        else if self.expr[0] == b'-' {
            let n = self.len as usize;
            self.expr.copy_within(1..n, 0);
            self.len -= 1;
        } else if (self.len as usize) < self.expr.len() - 1 {
            let n = self.len as usize;
            self.expr.copy_within(0..n, 1);
            self.expr[0] = b'-';
            self.len += 1;
        }
    }
    fn equals(&mut self) {
        if self.len == 0 { return; }
        // A function key pushes its opening paren; equals closes what is still open.
        let mut text = self.expr_text().to_string();
        let open = text.bytes().filter(|&b| b == b'(').count();
        let close = text.bytes().filter(|&b| b == b')').count();
        for _ in close..open {
            text.push(')');
        }
        match eval(&text) {
            Some(v) if v.is_finite() => {
                let out = number(v);
                self.out_len = out.len().min(self.out.len()) as u8;
                self.out[..self.out_len as usize].copy_from_slice(&out.as_bytes()[..self.out_len as usize]);
                self.evaluated = true;
                self.error = false;
            }
            _ => { self.error = true; self.evaluated = false; }
        }
    }
    fn press(&mut self, key: usize) {
        if self.evaluated {
            if matches!(key, 4 | 5 | 6 | 7 | 8 | 9 | 10 | 13 | 14 | 19) { self.start_from_result(); }
            else if key != 0 && key != 24 { self.clear(); }
        }
        match key {
            0 => self.clear(),
            1 => self.push("("),
            2 => self.push(")"),
            3 => self.push("π"),
            4 => self.push("/"),
            5 => self.push("sin("),
            6 => self.push("cos("),
            7 => self.push("tan("),
            8 => self.push("ln("),
            9 => self.push("*"),
            10 => self.push("log("),
            11 => self.push("sqrt("),
            12 => self.backspace(),
            13 => self.push("^"),
            14 => self.push("-"),
            15 => self.push("7"),
            16 => self.push("8"),
            17 => self.push("9"),
            18 => self.push("e"),
            19 => self.push("+"),
            20 => self.push("4"),
            21 => self.push("5"),
            22 => self.push("6"),
            23 => self.sign(),
            24 => self.equals(),
            25 => self.push("1"),
            26 => self.push("2"),
            27 => self.push("3"),
            28 => self.push("0"),
            29 => self.push("."),
            _ => {}
        }
    }
}

struct Expr<'a> { b: &'a [u8], at: usize }
impl Expr<'_> {
    fn ws(&mut self) { while self.at < self.b.len() && self.b[self.at].is_ascii_whitespace() { self.at += 1; } }
    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.b.get(self.at) == Some(&c) { self.at += 1; true } else { false }
    }
    fn expr(&mut self) -> Option<f64> {
        let mut v = self.term()?;
        loop {
            if self.eat(b'+') { v += self.term()?; }
            else if self.eat(b'-') { v -= self.term()?; }
            else { return Some(v); }
        }
    }
    fn term(&mut self) -> Option<f64> {
        let mut v = self.power()?;
        loop {
            if self.eat(b'*') { v *= self.power()?; }
            else if self.eat(b'/') {
                let r = self.power()?;
                if r == 0.0 { return None; }
                v /= r;
            } else { return Some(v); }
        }
    }
    fn power(&mut self) -> Option<f64> {
        let base = self.unary()?;
        if self.eat(b'^') { Some(base.powf(self.power()?)) } else { Some(base) }
    }
    fn unary(&mut self) -> Option<f64> {
        if self.eat(b'-') { Some(-self.unary()?) }
        else if self.eat(b'+') { self.unary() }
        else { self.atom() }
    }
    fn atom(&mut self) -> Option<f64> {
        self.ws();
        if self.eat(b'(') {
            let v = self.expr()?;
            return self.eat(b')').then_some(v);
        }
        if self.b[self.at..].starts_with("π".as_bytes()) {
            self.at += "π".len();
            return Some(std::f64::consts::PI);
        }
        for (name, f) in [
            ("sqrt", (|v: f64| v.sqrt()) as fn(f64) -> f64),
            ("sin", |v: f64| v.to_radians().sin()),
            ("cos", |v: f64| v.to_radians().cos()),
            ("tan", |v: f64| v.to_radians().tan()),
            ("log", |v: f64| v.log10()),
            ("ln", |v: f64| v.ln()),
        ] {
            if self.b[self.at..].starts_with(name.as_bytes()) {
                self.at += name.len();
                if !self.eat(b'(') { return None; }
                let v = self.expr()?;
                if !self.eat(b')') { return None; }
                return Some(f(v));
            }
        }
        let start = self.at;
        let mut dot = false;
        while let Some(&c) = self.b.get(self.at) {
            if c.is_ascii_digit() { self.at += 1; }
            else if c == b'.' && !dot { dot = true; self.at += 1; }
            else { break; }
        }
        if start == self.at { return None; }
        std::str::from_utf8(&self.b[start..self.at]).ok()?.parse().ok()
    }
}
fn eval(text: &str) -> Option<f64> {
    let mut p = Expr { b: text.as_bytes(), at: 0 };
    let v = p.expr()?;
    p.ws();
    (p.at == p.b.len() && v.is_finite()).then_some(v)
}

#[derive(Clone, Copy)]
struct Prog {
    value: i64,
    acc: i64,
    pending: u8,
    base: u8,
    typing: bool,
    evaluated: bool,
    error: bool,
    units_open: bool,
    unit_category: u8,
    buf: [u8; 48],
    len: u8,
}
impl Default for Prog {
    fn default() -> Self {
        let mut c = Prog { value: 0, acc: 0, pending: 0, base: 10, typing: false, evaluated: false, error: false, units_open: false, unit_category: 0, buf: [0; 48], len: 1 };
        c.buf[0] = b'0';
        c
    }
}
impl Prog {
    fn display(&self) -> &str {
        if self.error { return "Error"; }
        std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("0")
    }
    fn formatted(&self, v: i64) -> String {
        let sign = if v < 0 { "-" } else { "" };
        let m = v.unsigned_abs();
        match self.base {
            16 => format!("{sign}0x{m:X}"),
            2 => format!("{sign}0b{m:b}"),
            8 => format!("{sign}0o{m:o}"),
            _ => v.to_string(),
        }
    }
    fn refresh(&mut self) {
        let s = self.formatted(self.value);
        self.len = s.len().min(self.buf.len()) as u8;
        self.buf[..self.len as usize].copy_from_slice(&s.as_bytes()[..self.len as usize]);
    }
    fn reset(&mut self) {
        let (base, cat) = (self.base, self.unit_category);
        *self = Prog::default();
        self.base = base;
        self.unit_category = cat;
        self.refresh();
    }
    fn digit(&mut self, d: i64) {
        if d >= self.base as i64 { return; }
        if self.error || self.evaluated { self.reset(); }
        if !self.typing { self.value = 0; self.typing = true; }
        self.value = self.value.saturating_mul(self.base as i64).saturating_add(d);
        self.refresh();
    }
    fn backspace(&mut self) {
        if self.error || self.evaluated || !self.typing { return; }
        self.value /= self.base as i64;
        if self.value == 0 { self.typing = false; }
        self.refresh();
    }
    fn sign(&mut self) { self.value = self.value.saturating_neg(); self.typing = true; self.evaluated = false; self.refresh(); }
    fn set_base(&mut self, base: u8) { self.base = base; self.refresh(); }
    fn apply(&self, op: u8, rhs: i64) -> Option<i64> {
        match op {
            1 => Some(self.acc.saturating_add(rhs)),
            2 => Some(self.acc.saturating_sub(rhs)),
            3 => Some(self.acc.saturating_mul(rhs)),
            4 => (rhs != 0).then_some(self.acc / rhs),
            5 => Some(self.acc & rhs),
            6 => Some(self.acc | rhs),
            7 => Some(self.acc ^ rhs),
            8 if (0..64).contains(&rhs) => Some(self.acc.wrapping_shl(rhs as u32)),
            9 if (0..64).contains(&rhs) => Some(self.acc.wrapping_shr(rhs as u32)),
            10 => (rhs != 0).then_some(self.acc % rhs),
            11 if (0..=63).contains(&rhs) => Some(self.acc.wrapping_pow(rhs as u32)),
            _ => None,
        }
    }
    fn operator(&mut self, op: u8) {
        if self.error { self.reset(); return; }
        if self.pending != 0 && self.typing {
            match self.apply(self.pending, self.value) {
                Some(v) => self.value = v,
                None => { self.error = true; return; }
            }
        } else if self.pending == 0 { self.acc = self.value; }
        self.pending = op;
        self.typing = false;
        self.evaluated = false;
        self.refresh();
    }
    fn equals(&mut self) {
        if self.error || self.pending == 0 { return; }
        match self.apply(self.pending, self.value) {
            Some(v) => {
                self.value = v;
                self.acc = v;
                self.pending = 0;
                self.typing = false;
                self.evaluated = true;
                self.refresh();
            }
            None => self.error = true,
        }
    }
    fn press(&mut self, key: usize) {
        match key {
            0 => self.reset(),
            1 => self.set_base(16),
            2 => self.set_base(10),
            3 => self.set_base(2),
            4 => self.set_base(8),
            5 => self.operator(4),
            6 => self.operator(5),
            7 => self.operator(6),
            8 => self.operator(7),
            9 => { self.value = !self.value; self.evaluated = true; self.refresh(); }
            10 => self.operator(8),
            11 => self.operator(9),
            12 => self.digit(7), 13 => self.digit(8), 14 => self.digit(9),
            15 => self.digit(10), 16 => self.digit(11), 17 => self.digit(12),
            18 => self.digit(4), 19 => self.digit(5), 20 => self.digit(6),
            21 => self.digit(13), 22 => self.digit(14), 23 => self.digit(15),
            24 => self.digit(1), 25 => self.digit(2), 26 => self.digit(3), 27 => self.digit(0),
            28 => self.operator(1),
            29 => self.equals(),
            30 => self.operator(10),
            31 => self.units_open = true,
            32 => self.sign(),
            33 => self.backspace(),
            34 => self.operator(3),
            35 => self.operator(11),
            _ => {}
        }
    }
}

fn number(v: f64) -> String {
    if !v.is_finite() { return "Error".into(); }
    if v != 0.0 && (v.abs() >= 1_000_000_000_000.0 || v.abs() < 0.000001) { return format!("{v:.6e}"); }
    if v.fract() == 0.0 { return format!("{}", v as i64); }
    let mut s = format!("{v:.8}");
    while s.ends_with('0') { s.pop(); }
    if s.ends_with('.') { s.pop(); }
    s
}
fn conversions(cat: u8, v: f64) -> [(&'static str, f64, &'static str, f64); 3] {
    match cat {
        1 => [("kg", v, "lb", v * 2.2046226218), ("g", v * 1000.0, "oz", v * 35.27396195), ("t", v / 1000.0, "lb", v * 2204.6226218)],
        2 => [("B", v, "KB", v / 1024.0), ("B", v, "MB", v / 1048576.0), ("B", v, "GB", v / 1073741824.0)],
        3 => [("C", v, "F", v * 9.0 / 5.0 + 32.0), ("C", v, "K", v + 273.15), ("C", v, "R", (v + 273.15) * 9.0 / 5.0)],
        _ => [("m", v, "ft", v * 3.280839895), ("cm", v * 100.0, "in", v * 39.37007874), ("km", v / 1000.0, "mi", v / 1609.344)],
    }
}

struct App {
    loaded: bool,
    mode: u8,
    menu: bool,
    simple: Simple,
    sci: Sci,
    prog: Prog,
    enabled: [bool; 3],
}
impl Default for App {
    fn default() -> Self {
        App { loaded: false, mode: 0, menu: false, simple: Simple::default(), sci: Sci::default(), prog: Prog::default(), enabled: [true, true, true] }
    }
}

static mut APP: Option<App> = None;
fn app() -> &'static mut App { unsafe { APP.get_or_insert_with(App::default) } }

fn read_setting(key: &str) -> Option<String> {
    let mut out = [0u8; 64];
    let n = unsafe { setting(key.as_ptr() as i32, key.len() as i32, out.as_mut_ptr() as i32, out.len() as i32) };
    if n <= 0 { None } else { Some(String::from_utf8_lossy(&out[..n as usize]).into_owned()) }
}

impl App {
    fn init(&mut self) {
        if self.loaded { return; }
        self.loaded = true;
        let mut buf = vec![0u8; 512];
        let n = unsafe { state_get(buf.as_mut_ptr() as i32, buf.len() as i32) };
        if n > 0 { self.decode(&buf[..n as usize]); }
        self.sync_settings();
        if n <= 0 {
            self.mode = self.default_mode();
        }
    }
    fn sync_settings(&mut self) {
        let on = |key: &str| read_setting(key).map_or(true, |v| v != "false");
        self.enabled = [on("enable_simple"), on("enable_scientific"), on("enable_programming")];
        if !self.enabled.iter().any(|&b| b) { self.enabled[0] = true; }
        if !self.enabled[self.mode as usize] { self.mode = self.first_enabled(); }
        if self.enabled.iter().filter(|&&b| b).count() < 2 { self.menu = false; }
    }
    fn default_mode(&self) -> u8 {
        match read_setting("default_calculator").as_deref() {
            Some("Scientific") => 1,
            Some("Programming") => 2,
            _ => 0,
        }
    }
    fn first_enabled(&self) -> u8 { (0u8..3).find(|&i| self.enabled[i as usize]).unwrap_or(0) }
    fn display(&self) -> &str {
        match self.mode { 1 => self.sci.display(), 2 => self.prog.display(), _ => self.simple.display() }
    }
    fn press(&mut self, key: usize) {
        match self.mode { 1 => self.sci.press(key), 2 => self.prog.press(key), _ => self.simple.press(key) }
    }
    fn enabled_modes(&self) -> Vec<u8> { (0u8..3).filter(|&i| self.enabled[i as usize]).collect() }
    fn save(&self) {
        let b = self.encode();
        unsafe { state_set(b.as_ptr() as i32, b.len() as i32) };
    }
    fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(1);
        v.push(self.mode);
        v.push(self.menu as u8);
        v.extend_from_slice(&self.simple.buf);
        v.push(self.simple.len);
        v.extend_from_slice(&self.simple.acc.to_le_bytes());
        v.push(self.simple.op);
        v.push(self.simple.typing as u8);
        v.push(self.simple.evaluated as u8);
        v.push(self.simple.error as u8);
        v.extend_from_slice(&self.sci.expr);
        v.push(self.sci.len);
        v.extend_from_slice(&self.sci.out);
        v.push(self.sci.out_len);
        v.push(self.sci.evaluated as u8);
        v.push(self.sci.error as u8);
        v.extend_from_slice(&self.prog.value.to_le_bytes());
        v.extend_from_slice(&self.prog.acc.to_le_bytes());
        v.push(self.prog.pending);
        v.push(self.prog.base);
        v.push(self.prog.typing as u8);
        v.push(self.prog.evaluated as u8);
        v.push(self.prog.error as u8);
        v.push(self.prog.units_open as u8);
        v.push(self.prog.unit_category);
        v.extend_from_slice(&self.prog.buf);
        v.push(self.prog.len);
        v
    }
    fn decode(&mut self, b: &[u8]) {
        let mut at = 0usize;
        let mut take = |n: usize| -> Option<&[u8]> {
            if at + n > b.len() { return None; }
            let s = &b[at..at + n];
            at += n;
            Some(s)
        };
        if take(1) != Some(&[1]) { return; }
        let Some(mode) = take(1) else { return };
        self.mode = mode[0].min(2);
        let Some(menu) = take(1) else { return };
        self.menu = menu[0] != 0;
        let Some(buf) = take(24) else { return };
        self.simple.buf.copy_from_slice(buf);
        let Some(len) = take(1) else { return };
        self.simple.len = len[0].min(24);
        let Some(acc) = take(8) else { return };
        self.simple.acc = f64::from_le_bytes(acc.try_into().unwrap_or([0; 8]));
        let Some(op) = take(1) else { return };
        self.simple.op = op[0];
        let Some(flags) = take(3) else { return };
        self.simple.typing = flags[0] != 0;
        self.simple.evaluated = flags[1] != 0;
        self.simple.error = flags[2] != 0;
        let Some(expr) = take(80) else { return };
        self.sci.expr.copy_from_slice(expr);
        let Some(len) = take(1) else { return };
        self.sci.len = len[0].min(80);
        let Some(out) = take(32) else { return };
        self.sci.out.copy_from_slice(out);
        let Some(len) = take(1) else { return };
        self.sci.out_len = len[0].min(32);
        let Some(flags) = take(2) else { return };
        self.sci.evaluated = flags[0] != 0;
        self.sci.error = flags[1] != 0;
        let Some(v) = take(8) else { return };
        self.prog.value = i64::from_le_bytes(v.try_into().unwrap_or([0; 8]));
        let Some(v) = take(8) else { return };
        self.prog.acc = i64::from_le_bytes(v.try_into().unwrap_or([0; 8]));
        let Some(p) = take(1) else { return };
        self.prog.pending = p[0];
        let Some(b) = take(1) else { return };
        self.prog.base = b[0];
        let Some(flags) = take(5) else { return };
        self.prog.typing = flags[0] != 0;
        self.prog.evaluated = flags[1] != 0;
        self.prog.error = flags[2] != 0;
        self.prog.units_open = flags[3] != 0;
        self.prog.unit_category = flags[4];
        let Some(buf) = take(48) else { return };
        self.prog.buf.copy_from_slice(buf);
        let Some(len) = take(1) else { return };
        self.prog.len = len[0].min(48);
    }
}

fn btn(id: i32, x: i32, y: i32, w: i32, h: i32, label: &str, style: i32) {
    unsafe { button(id, x, y, w, h, label.as_ptr() as i32, label.len() as i32, style) };
}
fn txt(x: i32, y: i32, w: i32, h: i32, s: &str, px: i32, center: bool) {
    unsafe { text(x, y, w, h, s.as_ptr() as i32, s.len() as i32, px, center as i32) };
}
fn frame(x: i32, y: i32, w: i32, h: i32, style: i32) {
    unsafe { rect(x, y, w, h, style) };
}

fn mode_name(m: u8) -> &'static str {
    match m { 1 => "scientific", 2 => "programming", _ => "simple" }
}

fn draw(w: i32, h: i32, s: f32) {
    let a = app();
    a.init();
    a.sync_settings();
    let modes: Vec<&str> = a.enabled_modes().into_iter().map(mode_name).collect();
    let st = format!("{}|{}|{}", mode_name(a.mode), a.display(), modes.join(","));
    unsafe { status(st.as_ptr() as i32, st.len() as i32) };
    let bar = (54.0 * s) as i32;
    let pad = (8.0 * s) as i32;
    let bar_label = match a.mode { 1 => "Scientific  v", 2 => "Programming  v", _ => "Simple  v" };
    btn(MODE_BAR, 0, 0, w, bar, bar_label, 0);
    if a.menu {
        let modes = a.enabled_modes();
        let row = ((h - bar - pad * 3) as f32 / modes.len() as f32).clamp(32.0 * s, 58.0 * s) as i32;
        for (i, m) in modes.iter().enumerate() {
            let label = match m { 1 => "Scientific", 2 => "Programming", _ => "Simple" };
            btn(MODE_CHOICE + i as i32, 0, bar + pad + i as i32 * row, w, row, label, if *m == a.mode { 1 } else { 0 });
        }
        return;
    }
    if a.mode == 2 && a.prog.units_open {
        let cat_y = bar + pad;
        let cat_h = (48.0 * s) as i32;
        let cat_w = (w - 3 * (4.0 * s) as i32) / 4;
        for (i, name) in UNIT_CATEGORIES.iter().enumerate() {
            btn(UNIT_CAT + i as i32, i as i32 * (cat_w + (4.0 * s) as i32), cat_y, cat_w, cat_h, name, if a.prog.unit_category as usize == i { 1 } else { 0 });
        }
        let top = cat_y + cat_h + (14.0 * s) as i32;
        let row_h = (56.0 * s) as i32;
        for (i, (from, fv, to, tv)) in conversions(a.prog.unit_category, a.prog.value as f64).iter().enumerate() {
            let line = format!("{} {} = {} {}", number(*fv), from, number(*tv), to);
            txt(0, top + i as i32 * (row_h + pad), w, row_h, &line, (34.0 * s) as i32, false);
        }
        let bh = (54.0 * s) as i32;
        btn(UNIT_BACK, 0, h - bh, w, bh, "Back to calculator", 0);
        return;
    }
    let display_y = bar + pad;
    let display_h = (78.0 * s) as i32;
    frame(0, display_y, w, display_h, 0);
    txt(0, display_y, w, display_h, a.display(), (44.0 * s) as i32, true);
    let keys = match a.mode { 1 => SCI_KEYS.as_slice(), 2 => PROG_KEYS.as_slice(), _ => SIMPLE_KEYS.as_slice() };
    let (cols, rows) = match a.mode { 1 => (5, 6), 2 => (6, 6), _ => (4, 5) };
    let gap = (4.0 * s) as i32;
    let top = display_y + display_h + pad;
    let grid_h = (h - (48.0 * s) as i32 - top).max(0);
    let kw = (w - gap * (cols - 1)) / cols;
    let kh = (grid_h - gap * (rows - 1)) / rows;
    if kw <= 0 || kh <= 0 { return; }
    for (i, label) in keys.iter().enumerate() {
        let x = (i as i32 % cols) * (kw + gap);
        let y = top + (i as i32 / cols) * (kh + gap);
        let active_base = (a.prog.base == 16 && *label == "16")
            || (a.prog.base == 10 && *label == "10")
            || (a.prog.base == 2 && *label == "2")
            || (a.prog.base == 8 && *label == "8");
        let style = if a.mode == 2 && active_base { 1 } else { 0 };
        btn(i as i32, x, y, kw, kh, label, style);
    }
}

#[no_mangle]
pub extern "C" fn widget_draw(w: i32, h: i32, scale_milli: i32) {
    draw(w, h, scale_milli as f32 / 1000.0);
}

#[no_mangle]
pub extern "C" fn widget_tap(id: i32) {
    let a = app();
    a.init();
    a.sync_settings();
    if a.menu {
        if id == MODE_BAR || id < 0 { a.menu = false; }
        else if (MODE_CHOICE..MODE_CHOICE + 3).contains(&id) {
            let modes = a.enabled_modes();
            if let Some(&m) = modes.get((id - MODE_CHOICE) as usize) { a.mode = m; }
            a.menu = false;
        }
        a.save();
        return;
    }
    if a.mode == 2 && a.prog.units_open {
        if (UNIT_CAT..UNIT_CAT + 4).contains(&id) { a.prog.unit_category = (id - UNIT_CAT) as u8; }
        else if id == UNIT_BACK { a.prog.units_open = false; }
        a.save();
        return;
    }
    if id == MODE_BAR {
        if a.enabled.iter().filter(|&&b| b).count() > 1 { a.menu = true; }
        a.save();
        return;
    }
    if id >= 0 {
        a.press(id as usize);
    }
    a.save();
}
