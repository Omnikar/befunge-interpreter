use area::Area;
use std::io::{Write, Stdin, Stdout};
use rand::{
    Rng,
    distributions::{Distribution, Standard},
};

#[derive(Debug)]
pub struct Interpreter
{
    area: Area,
    stack: Vec<u8>,
    pos: (usize, usize),
    dir: Dir,
    string: bool,
}

impl Interpreter
{
    pub fn new(s: &str) -> Self
    {
        let mut rows = Vec::new();
        let mut row = String::new();
        for c in s.chars()
        {
            if let '\n' = c
            {
                rows.push(row);
                row = String::new();
            }
            else
            {
                row.push(c);
            }
        }

        let cols = rows.iter().fold(0, |acc, s| acc.max(s.len()));

        let mut area = Area::new(rows.len(), cols);
        for (row, s) in rows.into_iter().enumerate()
        {
            for (col, c) in s.chars().enumerate()
            {
                *area.pos((row, col)) = c as u8;
            }
        }

        Self {
            area,
            ..Self::default()
        }
    }

    pub fn run(&mut self, stdin: &Stdin, stdout: &mut Stdout) -> Result<(), Box<dyn std::error::Error>>
    {
        fn mv_cur(dir: Dir, pos: &mut (usize, usize), size: (usize, usize))
        {
            match dir
            {
                Dir::Rht =>
                {
                    pos.1 += 1;
                    if pos.1 >= size.1
                    {
                        pos.1 = 0;
                    }
                }
                Dir::Lft => pos.1 = pos.1.checked_sub(1).unwrap_or(size.1 - 1),
                Dir::Upp => pos.0 = pos.0.checked_sub(1).unwrap_or(size.0 - 1),
                Dir::Dwn =>
                {
                    pos.0 += 1;
                    if pos.0 >= size.0
                    {
                        pos.0 = 0;
                    }
                }
            }
        }
        macro_rules! mvcur {
            () => {
                mv_cur(self.dir, &mut self.pos, (self.area.rows(), self.area.cols()))
            };
        }
        
        let mut skip = false;
        loop
        {
            if skip
            {
                skip = false;
                mvcur!();
                continue;
            }
            let b = *self.area.pos(self.pos);
            let op = Op::from_u8(b);
            if self.string
            {
                if let Op::Str = op
                {
                    self.string = false;
                }
                else
                {
                    self.stack.push(b);
                }
                mvcur!();
                continue;
            }
            match op
            {
                Op::Int(i) => self.stack.push(i),
                Op::Add =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(a) = self.stack.pop()
                        {
                            b = a + b;
                        }
                        self.stack.push(b);
                    }
                }
                Op::Sub =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(a) = self.stack.pop()
                        {
                            b = a - b;
                        }
                        self.stack.push(b);
                    }
                }
                Op::Mul =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(a) = self.stack.pop()
                        {
                            b = a * b;
                        }
                        self.stack.push(b);
                    }
                }
                Op::Div =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(a) = self.stack.pop()
                        {
                            b = a / b;
                        }
                        self.stack.push(b);
                    }
                }
                Op::Mod =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(a) = self.stack.pop()
                        {
                            b = a % b;
                        }
                        self.stack.push(b);
                    }
                }
                Op::Not =>
                {
                    if let Some(a) = self.stack.pop()
                    {
                        self.stack.push(if a == 0 { 1 } else { 0 });
                    }
                }
                Op::Gre =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(a) = self.stack.pop()
                        {
                            b = (a > b) as u8;
                        }
                        self.stack.push(b);
                    }
                }
                Op::Rht => self.dir = Dir::Rht,
                Op::Lft => self.dir = Dir::Lft,
                Op::Upp => self.dir = Dir::Upp,
                Op::Dwn => self.dir = Dir::Dwn,
                Op::Rnd => self.dir = rand::thread_rng().gen::<Dir>(),
                Op::Hif => self.dir = if let Some(0) | None = self.stack.pop()
                {
                    Dir::Rht
                }
                else
                {
                    Dir::Lft
                },
                Op::Vif => self.dir = if let Some(0) | None = self.stack.pop()
                {
                    Dir::Dwn
                }
                else
                {
                    Dir::Upp
                },
                Op::Str => self.string = true,
                Op::Dup => if let Some(&a) = self.stack.last()
                {
                    self.stack.push(a);
                },
                Op::Swp =>
                {
                    if let Some(mut b) = self.stack.pop()
                    {
                        if let Some(mut a) = self.stack.pop()
                        {
                            let c = a;
                            a = b;
                            b = c;
                            self.stack.push(a);
                        }
                        self.stack.push(b);
                    }
                }
                Op::Pop => drop(self.stack.pop()),
                Op::Iot => if let Some(a) = self.stack.pop()
                {
                    write!(*stdout, "{}", a)?;
                    stdout.flush()?;
                }
                Op::Cot => if let Some(a) = self.stack.pop()
                {
                    write!(*stdout, "{}", a as char)?;
                    stdout.flush()?;
                }
                Op::Skp => skip = true,
                Op::Put =>
                {
                    if let Some(col) = self.stack.pop()
                    {
                        if let Some(row) = self.stack.pop()
                        {
                            if let Some(o) = self.stack.pop()
                            {
                                *self.area.pos((row as usize, col as usize)) = o;
                            }
                            else
                            {
                                self.stack.push(row);
                                self.stack.push(col);
                            }
                        }
                        else
                        {
                            self.stack.push(col);
                        }
                    }
                }
                Op::Get =>
                {
                    if let Some(col) = self.stack.pop()
                    {
                        if let Some(row) = self.stack.pop()
                        {
                            self.stack.push(*self.area.pos((row as usize, col as usize)));
                        }
                        else
                        {
                            self.stack.push(col);
                        }
                    }
                }
                Op::Iin =>
                {
                    let mut s = String::new();
                    stdin.read_line(&mut s)?;
                    self.stack.push(s.trim().parse::<u8>().unwrap_or(0));
                }
                Op::Cin =>
                {
                    let mut s = String::new();
                    stdin.read_line(&mut s)?;
                    self.stack.push(s.chars().next().map(|c| c as u8).unwrap_or(0));
                }
                Op::End => break,
                Op::Nop =>
                {
                    mvcur!();
                    continue;
                }
            }
            mvcur!();
        }
        Ok(())
    }
}

impl Default for Interpreter
{
    fn default() -> Self
    {
        Self {
            area: Area::new(0, 0),
            stack: Vec::new(),
            pos: (0, 0),
            dir: Dir::default(),
            string: false,
        }
    }
}

#[derive(Copy, Clone)]
enum Op
{
    Int(u8),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    Gre,
    Rht,
    Lft,
    Upp,
    Dwn,
    Rnd,
    Hif,
    Vif,
    Str,
    Dup,
    Swp,
    Pop,
    Iot,
    Cot,
    Skp,
    Put,
    Get,
    Iin,
    Cin,
    End,
    Nop,
}

impl Op
{
    fn from_u8(b: u8) -> Self
    {
        match b
        {
            b'0' => Self::Int(0),
            b'1' => Self::Int(1),
            b'2' => Self::Int(2),
            b'3' => Self::Int(3),
            b'4' => Self::Int(4),
            b'5' => Self::Int(5),
            b'6' => Self::Int(6),
            b'7' => Self::Int(7),
            b'8' => Self::Int(8),
            b'9' => Self::Int(9),
            b'+' => Self::Add,
            b'-' => Self::Sub,
            b'*' => Self::Mul,
            b'/' => Self::Div,
            b'%' => Self::Mod,
            b'!' => Self::Not,
            b'`' => Self::Gre,
            b'>' => Self::Rht,
            b'<' => Self::Lft,
            b'^' => Self::Upp,
            b'v' => Self::Dwn,
            b'?' => Self::Rnd,
            b'_' => Self::Hif,
            b'|' => Self::Vif,
            b'"' => Self::Str,
            b':' => Self::Dup,
            b'\\' => Self::Swp,
            b'$' => Self::Pop,
            b'.' => Self::Iot,
            b',' => Self::Cot,
            b'#' => Self::Skp,
            b'p' => Self::Put,
            b'g' => Self::Get,
            b'&' => Self::Iin,
            b'~' => Self::Cin,
            b'@' => Self::End,
            _ => Self::Nop,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Dir
{
    Rht,
    Lft,
    Upp,
    Dwn,
}

impl Default for Dir
{
    fn default() -> Self
    {
        Self::Rht
    }
}

impl Distribution<Dir> for Standard
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir
    {
        match rng.gen_range(0..4)
        {
            0 => Dir::Rht,
            1 => Dir::Lft,
            2 => Dir::Upp,
            _ => Dir::Dwn,
        }
    }
}

mod area
{
    use std::fmt;

    pub(super) struct Area
    {
        vec: Vec<Vec<u8>>,
        cols: usize,
    }

    impl Area
    {
        const DEFAULT: u8 = b' ';

        pub(super) fn new(rows: usize, cols: usize) -> Self
        {
            let mut vec = Vec::with_capacity(rows);
            for _ in 0..rows
            {
                let mut row = Vec::with_capacity(cols);
                for _ in 0..cols
                {
                    row.push(Self::DEFAULT);
                }
                vec.push(row);
            }
            Self { vec, cols }
        }

        pub(super) fn pos(&mut self, pos: (usize, usize)) -> &mut u8
        {
            let (row, col) = pos;
            if col >= self.cols
            {
                self.add_cols(col - self.cols + 1);
            }
            if row >= self.rows()
            {
                self.add_rows(row - self.rows() + 1);
            }

            &mut self.vec[row][col]
        }
        
        pub(super) fn rows(&self) -> usize
        {
            self.vec.len()
        }
        
        pub(super) fn cols(&self) -> usize
        {
            self.cols
        }

        fn add_rows(&mut self, count: usize)
        {
            for _ in 0..count
            {
                let mut row = Vec::with_capacity(self.cols);
                for _ in 0..self.cols
                {
                    row.push(Self::DEFAULT);
                }
                self.vec.push(row);
            }
        }

        fn add_cols(&mut self, count: usize)
        {
            for row in self.vec.iter_mut()
            {
                for _ in 0..count
                {
                    row.push(Self::DEFAULT);
                }
            }
            self.cols += count;
        }
    }

    impl fmt::Debug for Area
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            let mut iter = self.vec.iter();
            if let Some(row) = iter.next()
            {
                write!(f, "{:?}", row)?;
            }
            for row in iter
            {
                write!(f, "\n{:?}", row)?;
            }
            Ok(())
        }
    }

    impl fmt::Display for Area
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            for (i, row) in self.vec.iter().rev().enumerate().rev()
            {
                for v in row.iter()
                {
                    write!(f, "{}", *v as char)?;
                }
                if i != 0
                {
                    write!(f, "\n")?;
                }
            }
            Ok(())
        }
    }
}
