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
const FIN_TOOL: i32 = 300;
const FIN_FIELD: i32 = 310;
const FIN_KEY: i32 = 400;

const SIMPLE: u8 = 0;
const SCIENTIFIC: u8 = 1;
const PROGRAMMING: u8 = 2;
const FINANCE: u8 = 3;
const GRAPHING: u8 = 4;
const MODE_COUNT: usize = 5;

// Labels are drawn as buttons; the second value is how many columns the key spans.
const SIMPLE_KEYS: [(&str, i32); 22] = [
    ("AC", 1), ("(", 1), (")", 1), ("%", 1),
    ("7", 1), ("8", 1), ("9", 1), ("÷", 1),
    ("4", 1), ("5", 1), ("6", 1), ("×", 1),
    ("1", 1), ("2", 1), ("3", 1), ("−", 1),
    ("±", 1), ("0", 1), (".", 1), ("+", 1),
    ("⌫", 1), ("=", 3),
];
const SCI_KEYS: [(&str, i32); 30] = [
    ("AC", 1), ("(", 1), (")", 1), ("π", 1), ("÷", 1),
    ("sin", 1), ("cos", 1), ("tan", 1), ("ln", 1), ("×", 1),
    ("log", 1), ("√", 1), ("←", 1), ("xʸ", 1), ("−", 1),
    ("7", 1), ("8", 1), ("9", 1), ("e", 1), ("+", 1),
    ("4", 1), ("5", 1), ("6", 1), ("±", 1), ("=", 1),
    ("1", 1), ("2", 1), ("3", 1), ("0", 1), (".", 1),
];
const PROG_KEYS: [(&str, i32); 36] = [
    ("AC", 1), ("16", 1), ("10", 1), ("2", 1), ("8", 1), ("÷", 1),
    ("&", 1), ("|", 1), ("^", 1), ("~", 1), ("<<", 1), (">>", 1),
    ("7", 1), ("8", 1), ("9", 1), ("A", 1), ("B", 1), ("C", 1),
    ("4", 1), ("5", 1), ("6", 1), ("D", 1), ("E", 1), ("F", 1),
    ("1", 1), ("2", 1), ("3", 1), ("0", 1), ("+", 1), ("=", 1),
    ("%", 1), ("Un", 1), ("±", 1), ("←", 1), ("×", 1), ("xʸ", 1),
];
const GRAPH_KEYS: [(&str, i32); 30] = [
    ("AC", 1), ("(", 1), (")", 1), ("^", 1), ("x", 1), ("π", 1),
    ("7", 1), ("8", 1), ("9", 1), ("÷", 1), ("sin", 1), ("⌫", 1),
    ("4", 1), ("5", 1), ("6", 1), ("×", 1), ("cos", 1), ("√", 1),
    ("1", 1), ("2", 1), ("3", 1), ("−", 1), ("tan", 1), ("ln", 1),
    ("0", 1), (".", 1), ("+", 1), ("log", 1), ("e", 1), ("x²", 1),
];
const FIN_KEYS: [&str; 12] = ["7", "8", "9", "4", "5", "6", "1", "2", "3", ".", "0", "⌫"];
const UNIT_CATEGORIES: [&str; 4] = ["Length", "Mass", "Data", "Temp"];
const FINANCE_TOOLS: [&str; 4] = ["Mortgage", "Compound", "Interest", "Savings"];

fn number(v: f64) -> String {
    if !v.is_finite() { return "Error".into(); }
    if v != 0.0 && (v.abs() >= 1_000_000_000_000.0 || v.abs() < 0.000001) { return format!("{v:.6e}"); }
    if v.fract() == 0.0 { return format!("{}", v as i64); }
    let mut s = format!("{v:.8}");
    while s.ends_with('0') { s.pop(); }
    if s.ends_with('.') { s.pop(); }
    s
}

fn money(v: f64) -> String {
    if !v.is_finite() { return "—".into(); }
    let neg = v < 0.0;
    let s = format!("{:.2}", v.abs());
    let (int_part, frac) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    let digits = int_part.as_bytes();
    let mut out = String::new();
    for (i, d) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 { out.push(','); }
        out.push(*d as char);
    }
    out.push('.');
    out.push_str(frac);
    if neg { format!("-{out}") } else { out }
}

fn pretty(s: &str) -> String { s.replace('*', "×").replace('/', "÷") }

fn normalize(s: &str) -> String {
    s.replace('×', "*").replace('÷', "/").replace('−', "-").replace('%', "/100")
}

fn balanced(t: &str) -> String {
    let mut s = t.to_string();
    let open = s.bytes().filter(|&b| b == b'(').count();
    let close = s.bytes().filter(|&b| b == b')').count();
    for _ in close..open { s.push(')'); }
    s
}

fn text_capacity(w: i32, px: f32) -> usize {
    ((w as f32 / (px * 0.55)).max(4.0)) as usize
}

fn fit_tail(s: &str, max_chars: usize) -> String {
    let n = s.chars().count();
    if n <= max_chars || max_chars < 2 { return s.to_string(); }
    let tail: String = s.chars().skip(n - (max_chars - 1)).collect();
    format!("…{tail}")
}

#[derive(Clone, Copy)]
struct Simple {
    expr: [u8; 48],
    len: u8,
    out: [u8; 32],
    out_len: u8,
    evaluated: bool,
    error: bool,
}

impl Default for Simple {
    fn default() -> Self {
        Simple { expr: [0; 48], len: 0, out: [0; 32], out_len: 0, evaluated: false, error: false }
    }
}

impl Simple {
    fn expr_text(&self) -> &str { std::str::from_utf8(&self.expr[..self.len as usize]).unwrap_or("") }
    fn out_text(&self) -> &str { std::str::from_utf8(&self.out[..self.out_len as usize]).unwrap_or("") }
    fn clear(&mut self) { *self = Simple::default(); }
    fn push(&mut self, s: &str) {
        let room = self.expr.len().saturating_sub(self.len as usize);
        let mut n = s.len().min(room);
        while n > 0 && !s.is_char_boundary(n) { n -= 1; }
        if n == 0 { return; }
        self.expr[self.len as usize..self.len as usize + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n as u8;
    }
    fn set_out(&mut self, s: &str) {
        self.out_len = s.len().min(self.out.len()) as u8;
        self.out[..self.out_len as usize].copy_from_slice(&s.as_bytes()[..self.out_len as usize]);
    }
    // A digit after "=" starts a new sum; editing after an error keeps the expression.
    fn begin_new(&mut self) {
        if self.evaluated { self.clear(); }
        self.error = false;
    }
    fn continue_from_result(&mut self) {
        if self.evaluated {
            let r = self.out_text().to_string();
            self.clear();
            self.push(&r);
        }
        self.error = false;
    }
    fn closed_expr(&self) -> String { balanced(self.expr_text()) }
    fn preview(&self) -> Option<f64> {
        let t = self.closed_expr();
        if t.is_empty() { return None; }
        let has_op = t.bytes().skip(1).any(|b| matches!(b, b'+' | b'-' | b'*' | b'/' | b'^' | b'%'))
            || t.contains('(');
        if !has_op { return None; }
        eval(&normalize(&t))
    }
    fn big(&self) -> String {
        if self.error { return "Error".into(); }
        if self.evaluated { return self.out_text().to_string(); }
        if self.len == 0 { return "0".into(); }
        pretty(self.expr_text())
    }
    // The line above the answer: the finished sum after "=", or a live preview while typing.
    fn small(&self) -> String {
        if self.error { return String::new(); }
        if self.evaluated { return format!("{} =", pretty(self.expr_text())); }
        match self.preview() {
            Some(v) => format!("= {}", number(v)),
            None => String::new(),
        }
    }
    fn digit(&mut self, d: char) {
        self.begin_new();
        let s = self.expr_text();
        if s.ends_with('0') {
            let before = s[..s.len() - 1].chars().last();
            if !matches!(before, Some(c) if c.is_ascii_digit() || c == '.') { self.len -= 1; }
        }
        let mut b = [0u8; 4];
        let ch = d.encode_utf8(&mut b).to_string();
        self.push(&ch);
    }
    fn dot(&mut self) {
        self.begin_new();
        let s = self.expr_text();
        let tail: String = s.chars().rev().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if tail.contains('.') { return; }
        if tail.is_empty() { self.push("0."); } else { self.push("."); }
    }
    fn backspace(&mut self) {
        if self.evaluated { self.clear(); return; }
        self.error = false;
        let s = self.expr_text().to_string();
        if s.is_empty() { return; }
        let mut at = s.len() - 1;
        while at > 0 && !s.is_char_boundary(at) { at -= 1; }
        self.len = at as u8;
    }
    fn operator(&mut self, op: char) {
        self.continue_from_result();
        let s = self.expr_text().to_string();
        match s.chars().last() {
            None => self.push("0"),
            Some(c) if matches!(c, '+' | '-' | '*' | '/' | '^') => self.len -= c.len_utf8() as u8,
            Some('(') => self.push("0"),
            _ => {}
        }
        let mut b = [0u8; 4];
        let ch = op.encode_utf8(&mut b).to_string();
        self.push(&ch);
    }
    fn open_paren(&mut self) {
        self.begin_new();
        let last = self.expr_text().chars().last();
        if let Some(c) = last {
            if c.is_ascii_digit() || c == ')' || c == '.' { self.push("*"); }
        }
        self.push("(");
    }
    fn close_paren(&mut self) {
        if self.evaluated { return; }
        self.error = false;
        let (open, close, last) = {
            let s = self.expr_text();
            (
                s.bytes().filter(|&b| b == b'(').count(),
                s.bytes().filter(|&b| b == b')').count(),
                s.chars().last(),
            )
        };
        if close >= open { return; }
        match last {
            Some(c) if c.is_ascii_digit() || c == ')' || c == '.' => self.push(")"),
            _ => {}
        }
    }
    fn percent(&mut self) {
        self.continue_from_result();
        let last = self.expr_text().chars().last();
        match last {
            Some(c) if c.is_ascii_digit() || c == ')' || c == '%' => self.push("%"),
            _ => {}
        }
    }
    fn sign(&mut self) {
        self.continue_from_result();
        let s = self.expr_text().to_string();
        let b = s.as_bytes();
        let mut i = b.len();
        while i > 0 && (b[i - 1].is_ascii_digit() || b[i - 1] == b'.') { i -= 1; }
        let t = if i == b.len() {
            if let Some(rest) = s.strip_prefix('-') { rest.to_string() } else { format!("-{s}") }
        } else if i > 0 && b[i - 1] == b'-' {
            format!("{}{}", &s[..i - 1], &s[i..])
        } else {
            format!("{}-{}", &s[..i], &s[i..])
        };
        self.len = 0;
        self.push(&t);
        if self.len == 0 { self.push("0"); }
    }
    fn equals(&mut self) {
        if self.len == 0 { return; }
        self.error = false;
        let text = self.closed_expr();
        self.len = 0;
        self.push(&text);
        match eval(&normalize(self.expr_text())) {
            Some(v) => {
                let s = number(v);
                self.error = s == "Error";
                if !self.error {
                    self.set_out(&s);
                    self.evaluated = true;
                }
            }
            None => { self.error = true; self.evaluated = false; }
        }
    }
    fn press(&mut self, key: usize) {
        match key {
            0 => self.clear(),
            1 => self.open_paren(),
            2 => self.close_paren(),
            3 => self.percent(),
            4 => self.digit('7'),
            5 => self.digit('8'),
            6 => self.digit('9'),
            7 => self.operator('/'),
            8 => self.digit('4'),
            9 => self.digit('5'),
            10 => self.digit('6'),
            11 => self.operator('*'),
            12 => self.digit('1'),
            13 => self.digit('2'),
            14 => self.digit('3'),
            15 => self.operator('-'),
            16 => self.sign(),
            17 => self.digit('0'),
            18 => self.dot(),
            19 => self.operator('+'),
            20 => self.backspace(),
            21 => self.equals(),
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
    fn big(&self) -> String {
        if self.error { return "Error".into(); }
        if self.evaluated { return self.out_text().to_string(); }
        if self.len == 0 { "0".into() } else { pretty(self.expr_text()) }
    }
    fn small(&self) -> String {
        if self.error { return String::new(); }
        if self.evaluated { return format!("{} =", pretty(self.expr_text())); }
        match eval(&normalize(&balanced(self.expr_text()))) {
            Some(v) if self.len > 0 => format!("= {}", number(v)),
            _ => String::new(),
        }
    }
    fn clear(&mut self) { *self = Sci::default(); }
    fn push(&mut self, s: &str) {
        let room = self.expr.len().saturating_sub(self.len as usize);
        let mut n = s.len().min(room);
        while n > 0 && !s.is_char_boundary(n) { n -= 1; }
        if n == 0 { return; }
        self.expr[self.len as usize..self.len as usize + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n as u8;
        self.error = false;
    }
    fn set_out(&mut self, s: &str) {
        self.out_len = s.len().min(self.out.len()) as u8;
        self.out[..self.out_len as usize].copy_from_slice(&s.as_bytes()[..self.out_len as usize]);
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
        let text = balanced(self.expr_text());
        self.len = 0;
        self.push(&text);
        match eval(&normalize(self.expr_text())) {
            Some(v) if v.is_finite() => {
                let out = number(v);
                self.error = out == "Error";
                if !self.error {
                    self.set_out(&out);
                    self.evaluated = true;
                }
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

#[derive(Clone, Copy)]
struct Graph {
    expr: [u8; 64],
    len: u8,
}
impl Default for Graph {
    fn default() -> Self { Graph { expr: [0; 64], len: 0 } }
}
impl Graph {
    fn text(&self) -> &str { std::str::from_utf8(&self.expr[..self.len as usize]).unwrap_or("") }
    fn clear(&mut self) { *self = Graph::default(); }
    fn push(&mut self, s: &str) {
        let room = self.expr.len().saturating_sub(self.len as usize);
        let mut n = s.len().min(room);
        while n > 0 && !s.is_char_boundary(n) { n -= 1; }
        if n == 0 { return; }
        self.expr[self.len as usize..self.len as usize + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n as u8;
    }
    fn digit(&mut self, d: char) {
        let s = self.text();
        if s.ends_with('0') {
            let before = s[..s.len() - 1].chars().last();
            if !matches!(before, Some(c) if c.is_ascii_digit() || c == '.') { self.len -= 1; }
        }
        let mut b = [0u8; 4];
        let ch = d.encode_utf8(&mut b).to_string();
        self.push(&ch);
    }
    fn dot(&mut self) {
        let s = self.text();
        let tail: String = s.chars().rev().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if tail.contains('.') { return; }
        if tail.is_empty() { self.push("0."); } else { self.push("."); }
    }
    fn backspace(&mut self) {
        let s = self.text().to_string();
        if s.is_empty() { return; }
        let mut at = s.len() - 1;
        while at > 0 && !s.is_char_boundary(at) { at -= 1; }
        self.len = at as u8;
    }
    fn open_paren(&mut self) {
        let last = self.text().chars().last();
        if let Some(c) = last {
            if c.is_ascii_digit() || c == ')' || c == '.' || c == 'x' || c == 'π' || c == 'e' { self.push("*"); }
        }
        self.push("(");
    }
    fn close_paren(&mut self) {
        let (open, close, last) = {
            let s = self.text();
            (
                s.bytes().filter(|&b| b == b'(').count(),
                s.bytes().filter(|&b| b == b')').count(),
                s.chars().last(),
            )
        };
        if close >= open { return; }
        match last {
            Some(c) if c.is_ascii_digit() || c == ')' || c == '.' || c == 'x' || c == 'π' || c == 'e' => self.push(")"),
            _ => {}
        }
    }
    fn press(&mut self, key: usize) {
        match key {
            0 => self.clear(),
            1 => self.open_paren(),
            2 => self.close_paren(),
            3 => self.push("^"),
            4 => self.push("x"),
            5 => self.push("π"),
            6 => self.digit('7'),
            7 => self.digit('8'),
            8 => self.digit('9'),
            9 => self.push("/"),
            10 => self.push("sin("),
            11 => self.backspace(),
            12 => self.digit('4'),
            13 => self.digit('5'),
            14 => self.digit('6'),
            15 => self.push("*"),
            16 => self.push("cos("),
            17 => self.push("sqrt("),
            18 => self.digit('1'),
            19 => self.digit('2'),
            20 => self.digit('3'),
            21 => self.push("-"),
            22 => self.push("tan("),
            23 => self.push("ln("),
            24 => self.digit('0'),
            25 => self.dot(),
            26 => self.push("+"),
            27 => self.push("log("),
            28 => self.push("e"),
            29 => self.push("^2"),
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
struct Finance {
    tool: u8,
    field: u8,
    values: [f64; 3],
    entry: [u8; 20],
    entry_len: u8,
}
impl Default for Finance {
    fn default() -> Self {
        let mut f = Finance { tool: 0, field: 0, values: [250000.0, 6.5, 30.0], entry: [0; 20], entry_len: 0 };
        f.set_entry("250000");
        f
    }
}
impl Finance {
    fn entry_text(&self) -> &str { std::str::from_utf8(&self.entry[..self.entry_len as usize]).unwrap_or("0") }
    fn set_entry(&mut self, s: &str) {
        let n = s.len().min(self.entry.len());
        self.entry[..n].copy_from_slice(&s.as_bytes()[..n]);
        self.entry_len = n as u8;
    }
    fn push_entry(&mut self, s: &str) {
        let room = self.entry.len().saturating_sub(self.entry_len as usize);
        let n = s.len().min(room);
        if n == 0 { return; }
        self.entry[self.entry_len as usize..self.entry_len as usize + n].copy_from_slice(&s.as_bytes()[..n]);
        self.entry_len += n as u8;
    }
    fn commit(&mut self) {
        let v = self.entry_text().parse().unwrap_or(0.0);
        self.values[self.field as usize] = v;
    }
    fn select_field(&mut self, field: u8) {
        if field == self.field { return; }
        self.commit();
        self.field = field;
        self.set_entry(&number(self.values[field as usize]));
    }
    fn select_tool(&mut self, tool: u8) {
        self.commit();
        self.tool = tool;
    }
    // Values as they read on screen, including what is being typed right now.
    fn effective(&self) -> [f64; 3] {
        let mut v = self.values;
        v[self.field as usize] = self.entry_text().parse().unwrap_or(0.0);
        v
    }
    fn results(&self) -> [(&'static str, f64); 3] {
        let v = self.effective();
        let (p, rate, years) = (v[0], v[1], v[2]);
        let r = rate / 1200.0;
        let n = (years * 12.0).round();
        let bad = p <= 0.0 || rate < 0.0 || years <= 0.0 || n <= 0.0 || !p.is_finite() || !rate.is_finite() || !years.is_finite();
        match self.tool {
            0 => {
                let pay = if bad { f64::NAN } else if r == 0.0 { p / n } else { p * r / (1.0 - (1.0 + r).powf(-n)) };
                let total = pay * n;
                [("Monthly payment", pay), ("Total interest", total - p), ("Total cost", total)]
            }
            1 => {
                let fv = if bad { f64::NAN } else { p * (1.0 + r).powf(n) };
                [("Future value", fv), ("Interest earned", fv - p), ("Paid in", p)]
            }
            2 => {
                let interest = if bad { f64::NAN } else { p * rate / 100.0 * years };
                [("Interest", interest), ("Total", p + interest), ("Per year", p * rate / 100.0)]
            }
            _ => {
                let deposit = if bad { f64::NAN } else if r == 0.0 { p / n } else { p * r / ((1.0 + r).powf(n) - 1.0) };
                let saved = deposit * n;
                [("Monthly deposit", deposit), ("Paid in", saved), ("Interest earned", p - saved)]
            }
        }
    }
    fn digit(&mut self, d: char) {
        let s = self.entry_text();
        if s == "0" { self.entry_len = 0; }
        if self.entry_len as usize >= 13 { return; }
        let mut b = [0u8; 4];
        let ch = d.encode_utf8(&mut b).to_string();
        self.push_entry(&ch);
    }
    fn dot(&mut self) {
        if self.entry_text().contains('.') { return; }
        if self.entry_len == 0 { self.push_entry("0."); } else { self.push_entry("."); }
    }
    fn backspace(&mut self) {
        let s = self.entry_text().to_string();
        if s.len() <= 1 { self.set_entry("0"); return; }
        let mut at = s.len() - 1;
        while at > 0 && !s.is_char_boundary(at) { at -= 1; }
        self.entry_len = at as u8;
        if self.entry_len == 0 { self.set_entry("0"); }
    }
    fn press(&mut self, key: usize) {
        match key {
            0 => self.digit('7'),
            1 => self.digit('8'),
            2 => self.digit('9'),
            3 => self.digit('4'),
            4 => self.digit('5'),
            5 => self.digit('6'),
            6 => self.digit('1'),
            7 => self.digit('2'),
            8 => self.digit('3'),
            9 => self.dot(),
            10 => self.digit('0'),
            11 => self.backspace(),
            _ => {}
        }
    }
}

fn finance_labels(tool: u8) -> [&'static str; 3] {
    match tool {
        0 => ["Loan", "Rate %", "Years"],
        3 => ["Target", "Rate %", "Years"],
        _ => ["Principal", "Rate %", "Years"],
    }
}

struct Expr<'a> {
    b: &'a [u8],
    at: usize,
    x: f64,
}
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
    // A number, constant or bracket straight after a value multiplies it: 2x, 3(4+1).
    fn starts_atom(&mut self) -> bool {
        self.ws();
        match self.b.get(self.at) {
            Some(&c) if c.is_ascii_digit() || c == b'.' || c == b'(' || c == b'x' || c == b'e' => true,
            _ => {
                let rest = &self.b[self.at..];
                ["sqrt", "sin", "cos", "tan", "log", "ln"].iter().any(|n| rest.starts_with(n.as_bytes()))
                    || rest.starts_with("π".as_bytes())
            }
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
            } else if self.starts_atom() { v *= self.power()?; }
            else { return Some(v); }
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
        match self.b.get(self.at) {
            Some(b'x') => { self.at += 1; return Some(self.x); }
            Some(b'e') => { self.at += 1; return Some(std::f64::consts::E); }
            _ => {}
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
fn eval_at(text: &str, x: f64) -> Option<f64> {
    let mut p = Expr { b: text.as_bytes(), at: 0, x };
    let v = p.expr()?;
    p.ws();
    (p.at == p.b.len() && v.is_finite()).then_some(v)
}
fn eval(text: &str) -> Option<f64> { eval_at(text, 0.0) }

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
    finance: Finance,
    graph: Graph,
    enabled: [bool; MODE_COUNT],
}
impl Default for App {
    fn default() -> Self {
        App {
            loaded: false,
            mode: SIMPLE,
            menu: false,
            simple: Simple::default(),
            sci: Sci::default(),
            prog: Prog::default(),
            finance: Finance::default(),
            graph: Graph::default(),
            enabled: [true; MODE_COUNT],
        }
    }
}

static mut APP: Option<App> = None;
fn app() -> &'static mut App { unsafe { APP.get_or_insert_with(App::default) } }

fn read_setting(key: &str) -> Option<String> {
    let mut out = [0u8; 64];
    let n = unsafe { setting(key.as_ptr() as i32, key.len() as i32, out.as_mut_ptr() as i32, out.len() as i32) };
    if n <= 0 { None } else { Some(String::from_utf8_lossy(&out[..n as usize]).into_owned()) }
}

fn mode_name(m: u8) -> &'static str {
    match m { SCIENTIFIC => "scientific", PROGRAMMING => "programming", FINANCE => "finance", GRAPHING => "graphing", _ => "simple" }
}

fn mode_title(m: u8) -> &'static str {
    match m { SCIENTIFIC => "Scientific", PROGRAMMING => "Programming", FINANCE => "Finance", GRAPHING => "Graphing", _ => "Simple" }
}

impl App {
    fn init(&mut self) {
        if self.loaded { return; }
        self.loaded = true;
        let mut buf = vec![0u8; 1024];
        let n = unsafe { state_get(buf.as_mut_ptr() as i32, buf.len() as i32) };
        let restored = n > 0 && self.decode(&buf[..n as usize]);
        self.sync_settings();
        if !restored {
            self.mode = self.default_mode();
        }
    }
    fn sync_settings(&mut self) {
        let on = |key: &str| read_setting(key).map_or(true, |v| v != "false");
        self.enabled = [
            on("enable_simple"),
            on("enable_scientific"),
            on("enable_programming"),
            on("enable_finance"),
            on("enable_graphing"),
        ];
        if !self.enabled.iter().any(|&b| b) { self.enabled[0] = true; }
        if !self.enabled[self.mode as usize] { self.mode = self.first_enabled(); }
        if self.enabled.iter().filter(|&&b| b).count() < 2 { self.menu = false; }
    }
    fn default_mode(&self) -> u8 {
        match read_setting("default_calculator").as_deref() {
            Some("Scientific") => SCIENTIFIC,
            Some("Programming") => PROGRAMMING,
            Some("Finance") => FINANCE,
            Some("Graphing") => GRAPHING,
            _ => SIMPLE,
        }
    }
    fn first_enabled(&self) -> u8 { (0..MODE_COUNT as u8).find(|&i| self.enabled[i as usize]).unwrap_or(SIMPLE) }
    fn display(&self) -> String {
        match self.mode {
            SCIENTIFIC => self.sci.big(),
            PROGRAMMING => self.prog.display().to_string(),
            FINANCE => money(self.finance.results()[0].1),
            GRAPHING => {
                let e = self.graph.text();
                if e.is_empty() { "y =".into() } else { format!("y = {}", pretty(e)) }
            }
            _ => self.simple.big(),
        }
    }
    fn press(&mut self, key: usize) {
        match self.mode {
            SCIENTIFIC => self.sci.press(key),
            PROGRAMMING => self.prog.press(key),
            FINANCE => self.finance.press(key),
            GRAPHING => self.graph.press(key),
            _ => self.simple.press(key),
        }
    }
    fn enabled_modes(&self) -> Vec<u8> { (0..MODE_COUNT as u8).filter(|&i| self.enabled[i as usize]).collect() }
    fn save(&self) {
        let b = self.encode();
        unsafe { state_set(b.as_ptr() as i32, b.len() as i32) }
    }
    fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(2);
        v.push(self.mode);
        v.push(self.menu as u8);
        v.extend_from_slice(&self.simple.expr);
        v.push(self.simple.len);
        v.extend_from_slice(&self.simple.out);
        v.push(self.simple.out_len);
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
        v.push(self.finance.tool);
        v.push(self.finance.field);
        for value in self.finance.values { v.extend_from_slice(&value.to_le_bytes()); }
        v.extend_from_slice(&self.finance.entry);
        v.push(self.finance.entry_len);
        v.extend_from_slice(&self.graph.expr);
        v.push(self.graph.len);
        v
    }
    // Returns false when the saved blob is missing or from an older layout.
    fn decode(&mut self, b: &[u8]) -> bool {
        let mut at = 0usize;
        let mut take = |n: usize| -> Option<&[u8]> {
            if at + n > b.len() { return None; }
            let s = &b[at..at + n];
            at += n;
            Some(s)
        };
        if take(1) != Some(&[2]) { return false; }
        let Some(mode) = take(1) else { return false };
        self.mode = mode[0].min(MODE_COUNT as u8 - 1);
        let Some(menu) = take(1) else { return false };
        self.menu = menu[0] != 0;
        let Some(expr) = take(48) else { return false };
        self.simple.expr.copy_from_slice(expr);
        let Some(len) = take(1) else { return false };
        self.simple.len = len[0].min(48);
        let Some(out) = take(32) else { return false };
        self.simple.out.copy_from_slice(out);
        let Some(len) = take(1) else { return false };
        self.simple.out_len = len[0].min(32);
        let Some(flags) = take(2) else { return false };
        self.simple.evaluated = flags[0] != 0;
        self.simple.error = flags[1] != 0;
        let Some(expr) = take(80) else { return false };
        self.sci.expr.copy_from_slice(expr);
        let Some(len) = take(1) else { return false };
        self.sci.len = len[0].min(80);
        let Some(out) = take(32) else { return false };
        self.sci.out.copy_from_slice(out);
        let Some(len) = take(1) else { return false };
        self.sci.out_len = len[0].min(32);
        let Some(flags) = take(2) else { return false };
        self.sci.evaluated = flags[0] != 0;
        self.sci.error = flags[1] != 0;
        let Some(v) = take(8) else { return false };
        self.prog.value = i64::from_le_bytes(v.try_into().unwrap_or([0; 8]));
        let Some(v) = take(8) else { return false };
        self.prog.acc = i64::from_le_bytes(v.try_into().unwrap_or([0; 8]));
        let Some(p) = take(1) else { return false };
        self.prog.pending = p[0];
        let Some(b) = take(1) else { return false };
        self.prog.base = b[0];
        let Some(flags) = take(5) else { return false };
        self.prog.typing = flags[0] != 0;
        self.prog.evaluated = flags[1] != 0;
        self.prog.error = flags[2] != 0;
        self.prog.units_open = flags[3] != 0;
        self.prog.unit_category = flags[4];
        let Some(buf) = take(48) else { return false };
        self.prog.buf.copy_from_slice(buf);
        let Some(len) = take(1) else { return false };
        self.prog.len = len[0].min(48);
        let Some(tool) = take(1) else { return false };
        self.finance.tool = tool[0].min(3);
        let Some(field) = take(1) else { return false };
        self.finance.field = field[0].min(2);
        for slot in 0..3 {
            let Some(v) = take(8) else { return false };
            self.finance.values[slot] = f64::from_le_bytes(v.try_into().unwrap_or([0; 8]));
        }
        let Some(entry) = take(20) else { return false };
        self.finance.entry.copy_from_slice(entry);
        let Some(len) = take(1) else { return false };
        self.finance.entry_len = len[0].min(20);
        let Some(expr) = take(64) else { return false };
        self.graph.expr.copy_from_slice(expr);
        let Some(len) = take(1) else { return false };
        self.graph.len = len[0].min(64);
        true
    }
}

fn btn(id: i32, x: i32, y: i32, w: i32, h: i32, label: &str, style: i32) {
    unsafe { button(id, x, y, w, h, label.as_ptr() as i32, label.len() as i32, style) }
}
fn txt(x: i32, y: i32, w: i32, h: i32, s: &str, px: i32, center: bool) {
    unsafe { text(x, y, w, h, s.as_ptr() as i32, s.len() as i32, px, center as i32) }
}
fn frame(x: i32, y: i32, w: i32, h: i32, style: i32) {
    unsafe { rect(x, y, w, h, style) }
}

fn is_base_key(base: u8, label: &str) -> bool {
    (base == 16 && label == "16") || (base == 10 && label == "10") || (base == 2 && label == "2") || (base == 8 && label == "8")
}

// Two lines in the display card: the preview above, the answer below.
fn display_block(w: i32, y: i32, h: i32, s: f32, small: &str, big: &str) {
    frame(0, y, w, h, 0);
    if small.is_empty() {
        let t = fit_tail(big, text_capacity(w - 12, 44.0 * s));
        txt(0, y, w, h, &t, (44.0 * s) as i32, true);
    } else {
        let t = fit_tail(small, text_capacity(w - 12, 19.0 * s));
        txt(0, y + (3.0 * s) as i32, w, (24.0 * s) as i32, &t, (19.0 * s) as i32, true);
        let t = fit_tail(big, text_capacity(w - 12, 38.0 * s));
        txt(0, y + (25.0 * s) as i32, w, (56.0 * s) as i32, &t, (38.0 * s) as i32, true);
    }
}

fn draw_units(a: &App, w: i32, h: i32, s: f32, bar: i32, pad: i32) {
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
}

fn draw_finance(a: &App, w: i32, h: i32, s: f32, bar: i32, pad: i32) {
    let f = &a.finance;
    let gap = (4.0 * s) as i32;
    let tool_h = (38.0 * s) as i32;
    let tool_w = (w - gap * 3) / 4;
    if tool_w <= 0 || tool_h <= 0 { return; }
    for (i, name) in FINANCE_TOOLS.iter().enumerate() {
        btn(FIN_TOOL + i as i32, i as i32 * (tool_w + gap), bar + pad, tool_w, tool_h, name, if f.tool as usize == i { 1 } else { 0 });
    }
    let labels = finance_labels(f.tool);
    let row_h = (38.0 * s) as i32;
    let mut y = bar + pad + tool_h + pad;
    for i in 0..3usize {
        let value = if i == f.field as usize { f.entry_text().to_string() } else { number(f.values[i]) };
        let line = format!("{}   {}", labels[i], fit_tail(&value, text_capacity(w - 16, 20.0 * s)));
        btn(FIN_FIELD + i as i32, 0, y, w, row_h, &line, if i == f.field as usize { 1 } else { 0 });
        y += row_h + gap;
    }
    y += pad;
    for (i, (label, v)) in f.results().iter().enumerate() {
        let px = if i == 0 { (25.0 * s) as i32 } else { (18.0 * s) as i32 };
        let line = format!("{}   {}", label, money(*v));
        txt(0, y, w, (34.0 * s) as i32, &line, px, true);
        y += (34.0 * s) as i32;
    }
    let cols = 3i32;
    let rows = 4i32;
    let top = y + pad;
    let grid_h = h - top;
    let kw = (w - gap * (cols - 1)) / cols;
    let kh = (grid_h - gap * (rows - 1)) / rows;
    if kw <= 0 || kh <= 0 { return; }
    for (i, label) in FIN_KEYS.iter().enumerate() {
        let x = (i as i32 % cols) * (kw + gap);
        let yy = top + (i as i32 / cols) * (kh + gap);
        btn(FIN_KEY + i as i32, x, yy, kw, kh, label, 0);
    }
}

fn draw_plot(w: i32, y0: i32, h: i32, s: f32, expr: &str) {
    if w <= 2 || h <= 2 { return; }
    frame(0, y0, w, h, 0);
    let x_min = -10.0;
    let x_max = 10.0;
    if expr.trim().is_empty() {
        txt(0, y0 + h / 2 - (18.0 * s) as i32, w, (36.0 * s) as i32, "Type a formula, e.g. sin(x)", (18.0 * s) as i32, true);
        return;
    }
    let norm = normalize(expr);
    let cols = w;
    let mut values: Vec<Option<f64>> = Vec::with_capacity(cols as usize);
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for px in 0..cols {
        let xv = x_min + (px as f64 + 0.5) / cols as f64 * (x_max - x_min);
        let yv = eval_at(&norm, xv).filter(|v| v.is_finite() && v.abs() < 1e9);
        if let Some(v) = yv {
            if v < lo { lo = v; }
            if v > hi { hi = v; }
        }
        values.push(yv);
    }
    if !lo.is_finite() || !hi.is_finite() { return; }
    if hi - lo < 1e-9 { lo -= 1.0; hi += 1.0; }
    let edge = (hi - lo) * 0.08;
    let lo = lo - edge;
    let hi = hi + edge;
    let map = |v: f64| y0 as f64 + h as f64 - (v - lo) / (hi - lo) * h as f64;
    let label_px = ((15.0 * s) as i32).max(9);
    if lo <= 0.0 && hi >= 0.0 {
        let zy = map(0.0) as i32;
        frame(0, zy, w, 1, 1);
    }
    let zx = ((0.0 - x_min) / (x_max - x_min) * w as f64) as i32;
    if (0..w).contains(&zx) { frame(zx, y0, 1, h, 1); }
    let mut prev: Option<f64> = None;
    for (px, v) in values.iter().enumerate() {
        let Some(v) = *v else { prev = None; continue };
        let sy = map(v);
        if let Some(p) = prev {
            if (sy - p).abs() <= h as f64 * 3.0 {
                let top = p.min(sy).max(y0 as f64);
                let bot = p.max(sy).min((y0 + h) as f64);
                if bot > top { frame(px as i32, top as i32, 2, (bot - top).max(1.0) as i32, 1); }
            }
        }
        if sy >= y0 as f64 && sy <= (y0 + h) as f64 {
            frame(px as i32, sy as i32, 2, 2, 1);
        }
        prev = Some(sy);
    }
    let top_label = format!("y {}", number(hi));
    let right_label = format!("x {} … {}", number(x_min), number(x_max));
    let bottom_label = format!("y {}", number(lo));
    txt(4, y0 + 2, w / 2 - 4, label_px + 4, &top_label, label_px, false);
    txt(w / 2, y0 + 2, w / 2 - 4, label_px + 4, &right_label, label_px, true);
    txt(4, y0 + h - label_px - 4, w / 2 - 4, label_px + 4, &bottom_label, label_px, false);
}

fn draw_graphing(a: &App, w: i32, h: i32, s: f32, bar: i32, pad: i32) {
    let line_h = (44.0 * s) as i32;
    let keys_h = ((h as f32 * 0.42) as i32).max(0);
    let plot_y = bar + line_h + pad;
    let plot_h = (h - keys_h - plot_y).max(0);
    frame(0, bar, w, line_h, 0);
    let expr = a.graph.text();
    let title = if expr.is_empty() { "y =".to_string() } else { format!("y = {}", pretty(expr)) };
    let title = fit_tail(&title, text_capacity(w - 16, 24.0 * s));
    txt(6, bar, w - 12, line_h, &title, (24.0 * s) as i32, false);
    draw_plot(w, plot_y, plot_h, s, expr);
    let gap = (4.0 * s) as i32;
    let cols = 6i32;
    let rows = 5i32;
    let top = h - keys_h;
    let kw = (w - gap * (cols - 1)) / cols;
    let kh = (keys_h - gap * (rows - 1)) / rows;
    if kw <= 0 || kh <= 0 { return; }
    for (i, (label, _)) in GRAPH_KEYS.iter().enumerate() {
        let x = (i as i32 % cols) * (kw + gap);
        let y = top + (i as i32 / cols) * (kh + gap);
        btn(i as i32, x, y, kw, kh, label, 0);
    }
}

fn draw(w: i32, h: i32, s: f32) {
    let a = app();
    a.init();
    a.sync_settings();
    let modes: Vec<&str> = a.enabled_modes().into_iter().map(mode_name).collect();
    let st = format!("{}|{}|{}", mode_name(a.mode), a.display(), modes.join(","));
    unsafe { status(st.as_ptr() as i32, st.len() as i32) }
    let bar = (54.0 * s) as i32;
    let pad = (8.0 * s) as i32;
    let bar_label = format!("{}  v", mode_title(a.mode));
    btn(MODE_BAR, 0, 0, w, bar, &bar_label, 0);
    if a.menu {
        let modes = a.enabled_modes();
        let row = ((h - bar - pad * 3) as f32 / modes.len() as f32).clamp(32.0 * s, 58.0 * s) as i32;
        for (i, m) in modes.iter().enumerate() {
            btn(MODE_CHOICE + i as i32, 0, bar + pad + i as i32 * row, w, row, mode_title(*m), if *m == a.mode { 1 } else { 0 });
        }
        return;
    }
    match a.mode {
        FINANCE => { draw_finance(a, w, h, s, bar, pad); return; }
        GRAPHING => { draw_graphing(a, w, h, s, bar, pad); return; }
        PROGRAMMING if a.prog.units_open => { draw_units(a, w, h, s, bar, pad); return; }
        _ => {}
    }
    let display_y = bar + pad;
    let display_h = (86.0 * s) as i32;
    let (small, big) = match a.mode {
        SCIENTIFIC => (a.sci.small(), a.sci.big()),
        PROGRAMMING => (String::new(), a.prog.display().to_string()),
        _ => (a.simple.small(), a.simple.big()),
    };
    display_block(w, display_y, display_h, s, &small, &big);
    let (keys, cols, rows): (&[(&str, i32)], i32, i32) = match a.mode {
        SCIENTIFIC => (SCI_KEYS.as_slice(), 5, 6),
        PROGRAMMING => (PROG_KEYS.as_slice(), 6, 6),
        _ => (SIMPLE_KEYS.as_slice(), 4, 6),
    };
    let gap = (4.0 * s) as i32;
    let top = display_y + display_h + pad;
    let grid_h = (h - (48.0 * s) as i32 - top).max(0);
    let kw = (w - gap * (cols - 1)) / cols;
    let kh = (grid_h - gap * (rows - 1)) / rows;
    if kw <= 0 || kh <= 0 { return; }
    let mut col = 0;
    let mut row = 0;
    for (i, (label, span)) in keys.iter().enumerate() {
        let x = col * (kw + gap);
        let key_w = span * kw + (span - 1) * gap;
        let y = top + row * (kh + gap);
        let style = if a.mode == PROGRAMMING && is_base_key(a.prog.base, label) { 1 } else { 0 };
        btn(i as i32, x, y, key_w, kh, label, style);
        col += span;
        if col >= cols { col = 0; row += 1; }
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
        else if (MODE_CHOICE..MODE_CHOICE + MODE_COUNT as i32).contains(&id) {
            let modes = a.enabled_modes();
            if let Some(&m) = modes.get((id - MODE_CHOICE) as usize) { a.mode = m; }
            a.menu = false;
        }
        a.save();
        return;
    }
    if a.mode == PROGRAMMING && a.prog.units_open {
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
    if a.mode == FINANCE {
        if (FIN_TOOL..FIN_TOOL + 4).contains(&id) { a.finance.select_tool((id - FIN_TOOL) as u8); }
        else if (FIN_FIELD..FIN_FIELD + 3).contains(&id) { a.finance.select_field((id - FIN_FIELD) as u8); }
        else if (FIN_KEY..FIN_KEY + FIN_KEYS.len() as i32).contains(&id) { a.finance.press((id - FIN_KEY) as usize); }
        a.save();
        return;
    }
    if id >= 0 {
        a.press(id as usize);
    }
    a.save();
}
