#![allow(unused)]

use std::io::Write;

use termion::raw::IntoRawMode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    bg: termion::color::Rgb,
    fg: termion::color::Rgb,
    c: char,
}

impl Cell {
    pub const fn new(bg: termion::color::Rgb, fg: termion::color::Rgb, c: char) -> Self {
        Self { bg, fg, c }
    }

    pub const fn new_bg(bg: termion::color::Rgb) -> Self {
        Self {
            bg,
            fg: termion::color::Rgb(255, 255, 255),
            c: ' ',
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            bg: termion::color::Rgb(0, 0, 0),
            fg: termion::color::Rgb(255, 255, 255),
            c: ' ',
        }
    }
}

/// Terminal display
pub struct Display {
    width: usize,
    height: usize,
    cells: Box<[Cell]>,
}

impl Display {
    pub fn init(fill: &Cell) -> std::io::Result<Self> {
        let (width, height) = termion::terminal_size()?;
        let (width, height) = (width as usize, height as usize);
        Ok(Self {
            width,
            height,
            cells: vec![*fill; width * height].into_boxed_slice(),
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.cells[x + y * self.width])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if x < self.width && y < self.height {
            Some(&mut self.cells[x + y * self.width])
        } else {
            None
        }
    }

    pub fn get_unchecked(&self, x: usize, y: usize) -> &Cell {
        &self.cells[x + y * self.width]
    }

    pub fn get_mut_unchecked(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[x + y * self.width]
    }

    pub fn display(&self, mut stdout: impl std::io::Write) -> std::io::Result<()> {
        // This can be optimized a few ways, because currently it's pretty slow.
        // Probably best is to check if fg matches previous and dont change if it does.
        let mut last = Cell::new(
            termion::color::Rgb(0, 0, 0),
            termion::color::Rgb(0, 0, 0),
            ' ',
        );
        write!(
            stdout,
            "{}{}",
            termion::color::Fg(last.fg),
            termion::color::Bg(last.bg),
        )?;
        for y in 0..self.height {
            write!(stdout, "{}", termion::cursor::Goto(1, y as u16 + 1))?;
            for x in 0..self.width {
                let cell = self.get_unchecked(x, y);
                if cell.fg != last.fg {
                    write!(stdout, "{}", termion::color::Fg(cell.fg))?;
                }
                if cell.bg != last.bg {
                    write!(stdout, "{}", termion::color::Bg(cell.bg))?;
                }
                write!(stdout, "{}", cell.c)?;
                last = *cell;
            }
        }
        stdout.flush()?;
        Ok(())
    }
}

pub struct Drawer<'a> {
    display: &'a mut Display,
}

impl<'a> Drawer<'a> {
    pub fn new(display: &'a mut Display) -> Self {
        Self { display }
    }

    pub fn width(&self) -> isize {
        self.display.width as isize
    }

    pub fn height(&self) -> isize {
        self.display.height as isize
    }

    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut Cell> {
        if x >= 0 && y >= 0 {
            self.display.get_mut(x as usize, y as usize)
        } else {
            None
        }
    }

    pub fn pixel(&mut self, cell: &Cell, x: isize, y: isize) {
        if let Some(plot_cell) = self.get_mut(x, y) {
            *plot_cell = *cell;
        }
    }

    pub fn text(
        &mut self,
        x: isize,
        y: isize,
        text: &str,
        bg: Option<termion::color::Rgb>,
        fg: Option<termion::color::Rgb>,
    ) {
        text.lines().enumerate().for_each(|(dy, line)| {
            line.char_indices().for_each(|(dx, char)| {
                if let Some(cell) = self.get_mut(x + dx as isize, y + dy as isize) {
                    if let Some(fg) = fg {
                        cell.fg = fg
                    }
                    if let Some(bg) = bg {
                        cell.bg = bg
                    }
                    cell.c = char;
                }
            });
        });
    }

    fn line_low(&mut self, cell: &Cell, x0: isize, y0: isize, x1: isize, y1: isize) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }
        let mut d = (2 * dy) - dx;
        let mut y = y0;
        for x in x0..=x1 {
            self.pixel(cell, x, y);
            if d > 0 {
                y += yi;
                d += 2 * (dy - dx);
            } else {
                d += 2 * dy;
            }
        }
    }

    fn line_high(&mut self, cell: &Cell, x0: isize, y0: isize, x1: isize, y1: isize) {
        let mut dx = x1 - x0;
        let dy = y1 - y0;
        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }
        let mut d = (2 * dx) - dy;
        let mut x = x0;
        for y in y0..=y1 {
            self.pixel(cell, x, y);
            if d > 0 {
                x += xi;
                d += 2 * (dx - dy);
            } else {
                d += 2 * dx;
            }
        }
    }

    pub fn line(&mut self, cell: &Cell, x0: isize, y0: isize, x1: isize, y1: isize) {
        // https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
        if isize::abs_diff(y1, y0) < isize::abs_diff(x1, x0) {
            if x0 > x1 {
                self.line_low(cell, x1, y1, x0, y0);
            } else {
                self.line_low(cell, x0, y0, x1, y1);
            }
        } else {
            // Rust just really wants to format this to be hard to read.
            #[allow(dead_code)]
            if y0 > y1 {
                self.line_high(cell, x1, y1, x0, y0);
            } else {
                self.line_high(cell, x0, y0, x1, y1);
            }
        }
    }

    fn iter_rect(
        &mut self,
        mut x0: isize,
        mut y0: isize,
        mut x1: isize,
        mut y1: isize,
    ) -> impl Iterator<Item = (isize, isize)> {
        x0 = isize::clamp(x0, 0, self.width());
        x1 = isize::clamp(x1, 0, self.width());
        y0 = isize::clamp(y0, 0, self.height());
        y1 = isize::clamp(y1, 0, self.height());
        (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| (x, y)))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn triangle(
        &mut self,
        cell: &Cell,
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        x2: isize,
        y2: isize,
    ) {
        // TODO: Instead implement https://github.com/OneLoneCoder/Javidx9/blob/master/ConsoleGameEngine/olcConsoleGameEngine.h#L537

        // https://stackoverflow.com/questions/2049582/how-to-determine-if-a-point-is-in-a-2d-triangle#answer-2049593

        #[inline]
        fn sign(x0: isize, y0: isize, x1: isize, y1: isize, x2: isize, y2: isize) -> isize {
            (x0 - x2) * (y1 - y2) - (x1 - x2) * (y0 - y2)
        }

        #[inline]
        #[allow(clippy::too_many_arguments)]
        fn is_point_inside_triangle(
            x: isize,
            y: isize,
            x0: isize,
            y0: isize,
            x1: isize,
            y1: isize,
            x2: isize,
            y2: isize,
        ) -> bool {
            let d0 = sign(x, y, x0, y0, x1, y1);
            let d1 = sign(x, y, x1, y1, x2, y2);
            let d2 = sign(x, y, x2, y2, x0, y0);
            let has_neg = (d0 < 0) || (d1 < 0) || (d2 < 0);
            let has_pos = (d0 > 0) || (d1 > 0) || (d2 > 0);
            !(has_neg && has_pos)
        }

        const WIREFRAME: bool = false;

        if !WIREFRAME {
            self.iter_rect(
                x0.min(x1).min(x2),
                y0.min(y1).min(y2),
                x0.max(x1).max(x2),
                y0.max(y1).max(y2),
            )
            .for_each(|(x, y)| {
                if is_point_inside_triangle(x, y, x0, y0, x1, y1, x2, y2) {
                    self.pixel(cell, x, y);
                }
            });
        } else {
            self.line(cell, x0, y0, x1, y1);
            self.line(cell, x1, y1, x2, y2);
            self.line(cell, x2, y2, x0, y0);
        }
    }
}
