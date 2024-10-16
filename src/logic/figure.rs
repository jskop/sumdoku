use std::{fmt::Display, hash::Hash};

use super::Cell;

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub v: i8,
    pub h: i8,
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            v: self.v,
            h: self.h,
        }
    }
}

impl Copy for Point {}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Point [ x: {}, y: {}, corner: {} ]",
            self.x,
            self.y,
            self.str_corner()
        ))
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Point {}

impl Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
        format!("x_{}_y_{}", self.x, self.y).hash(state);
    }
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point {
            x: x as i32,
            y: y as i32,
            v: 0,
            h: 0,
        }
    }

    pub fn apply_padding(&mut self, padding: i32) {
        self.y += padding * (self.h as i32);
        self.x += padding * (self.v as i32);
    }

    fn str_corner(&self) -> String {
        let mut c = Vec::new();
        if self.h == 1 {
            c.push("top")
        } else if self.h == -1 {
            c.push("bootom");
        }
        if self.v == 1 {
            c.push("left");
        } else if self.v == -1 {
            c.push("right");
        }
        c.join("-")
    }
}

#[derive(Debug, PartialEq)]
pub struct Edge {
    pub start: Point,
    pub end: Point,
    pub v: i8,
    pub h: i8,
    pub x: u32,
    pub y: u32
}

impl Edge {
    pub fn from_line(line: &Line) -> Self {
        Edge {
            start: Point::new(line.x1, line.y1),
            end: Point::new(line.x2, line.y2),
            h: match line.loc {
                LineLocation::Top => 1,
                LineLocation::Bottom => -1,
                _ => 0
            },
            v: match line.loc {
                LineLocation::Left => 1,
                LineLocation::Right => -1,
                _ => 0
            },
            x: match line.loc {
                LineLocation::Left | LineLocation::Right => line.x1,
                _ => 0
            },
            y: match line.loc {
                LineLocation::Bottom | LineLocation::Top => line.y1,
                _ => 0
            }
        }
    }

    fn align(&mut self) {
        if self.x==0 {
            let y = if self.h==1 { self.min(&self.start.y, &self.end.y) } else { self.max(&self.start.y,& self.end.y) };
            self.start.y = y;
            self.end.y = y;            
        } else {
            let x = if self.v==-1 { self.min(&self.start.x, &self.end.x) } else { self.max(&self.start.x,& self.end.x) };            
            self.start.x = x;
            self.end.x = x;
        }
    }

    fn min(&self, a: &i32, b: &i32) -> i32 {
        if *a<*b { *a } else { *b }
    }

    fn max(&self, a: &i32, b: &i32) -> i32 {
        if *a>*b { *a } else { *b }
    }
}

#[derive(PartialEq, Debug)]
pub struct Figure {
    pub edges: Vec<Edge>,
}

impl Figure {
    pub fn from_lines(cell_size: usize, padding: i32, lines: &Vec<Line>) -> Self {
        let area_size = cell_size*9;
        let mut points = vec![vec![(0i8, 0i8); area_size+1];area_size+1];
        let mut edges = Vec::with_capacity(lines.len());
        for line in lines {
            let (h,v) = get_hv(line);
            apply_hv(line.x1 as usize, line.y1 as usize, (h,v), &mut points);
            apply_hv(line.x2 as usize, line.y2 as usize, (h,v), &mut points);
            edges.push(Edge::from_line(line));
        }
        for edge in edges.iter_mut() {
            apply_hv_to_point(&mut edge.start, &points);
            apply_hv_to_point(&mut edge.end, &points);
            edge.start.apply_padding(padding);
            edge.end.apply_padding(padding);
            edge.align();
        }
        Figure { edges }
    }
}

fn apply_hv(x: usize, y: usize, (h,v) :(i8,i8),  points: &mut Vec<Vec<(i8,i8)>>) {
    let (th,tv) = points[x][y];
    points[x][y] = (if h != 0 { h } else { th }, if v != 0 { v } else { tv });
}

fn apply_hv_to_point(p: &mut Point, points: &Vec<Vec<(i8,i8)>>) {
    let (h,v) = points[p.x as usize][p.y as usize];
    p.h = h;
    p.v = v;
}

fn get_hv(line: &Line) -> (i8, i8) {
    (
        match line.loc {
            LineLocation::Top => 1,
            LineLocation::Bottom => -1,
            _ => 0
        },
        match line.loc {
            LineLocation::Left => 1,
            LineLocation::Right => -1i8,
            _ => 0
        }
    )
}

#[derive(Debug, Clone)]
pub struct Line {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub loc: LineLocation,
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.x1 == other.x1 && self.y1 == other.y1 && self.x2 == other.x2 && self.y2 == other.y2
    }
}

impl Eq for Line {}

impl Hash for Line {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x1.hash(state);
        self.y1.hash(state);
        self.x2.hash(state);
        self.y2.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineLocation {
    Top,
    Bottom,
    Left,
    Right,
}

impl Line {
    pub fn new(size: usize, a: usize, b: usize, c: usize, v: bool, st: bool) -> Self {
        let s = (size * a) as u32;
        let e = (size * (b + 1)) as u32;
        let t = (if st { size * c } else { size * (c + 1) }) as u32;
        if v {
            return Line {
                x1: t,
                x2: t,
                y1: s,
                y2: e,
                loc: if st {
                    LineLocation::Top
                } else {
                    LineLocation::Bottom
                },
            };
        }
        Line {
            x1: s,
            x2: e,
            y1: t,
            y2: t,
            loc: if st {
                LineLocation::Left
            } else {
                LineLocation::Right
            },
        }
    }

    pub fn from_cell(size: usize, c: &Cell) -> Vec<Line> {
        let mut lines = Vec::new();
        lines.push(Line::new(size, c.col, c.col, c.row, true, true));
        lines.push(Line::new(size, c.col, c.col, c.row, true, false));
        lines.push(Line::new(size, c.row, c.row, c.col, false, true));
        lines.push(Line::new(size, c.row, c.row, c.col, false, false));
        lines
    }

    pub fn is_connected(&self, other: &Line) -> Option<(u32, u32, u32, u32)> {
        if self.x1 == other.x1 && self.y1 == other.y1 {
            Some((self.x2, self.y2, other.x2, other.y2))
        } else if self.x1 == other.x2 && self.y1 == other.y2 {
            Some((self.x2, self.y2, other.x1, other.y1))
        } else if self.x2 == other.x1 && self.y2 == other.y1 {
            Some((self.x1, self.y1, other.x2, other.y2))
        } else if self.x2 == other.x2 && self.y2 == other.y2 {
            Some((self.x1, self.y1, other.x1, other.y1))
        } else {
            None
        }
    }

    pub fn merge(&self, other: &Line) -> Option<Line> {
        if let Some((x1, y1, x2, y2)) = self.is_connected(other) {
            if x1 == x2 || y1 == y2 {
                return Some(Line {
                    x1: x1.min(x2),
                    y1: y1.min(y2),
                    x2: x1.max(x2),
                    y2: y1.max(y2),
                    loc: self.loc.clone(),
                });
            }
        }
        None
    }


}
