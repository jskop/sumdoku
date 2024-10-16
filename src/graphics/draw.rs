use std::cell::{RefCell, RefMut};

use sdl2::{
    image::LoadTexture, keyboard::Keycode, pixels::Color, rect::{Point, Rect}, render::{TextureCreator, WindowCanvas}, ttf::{self}, video::WindowContext
};

use crate::{game::Game, logic::Cage};

const FONT_PATH: &str = "./assets/NotoSans-Regular.ttf";
const BOLD_FONT_PATH: &str = "./assets/NotoSans-SemiBold.ttf";
const UNDO_ICON: &str = "./assets/undo.png";
const ERASE_ICON: &str = "./assets/erase.png";
const NOTE_ICON: &str = "./assets/note.png";
const HINT_ICON: &str = "./assets/hint.png";


pub struct GameRenderer {
    canvas: RefCell<WindowCanvas>,
    game: RefCell<Game>,
    cell_size: i32,
    cage_offset: i32,
    notes_gap: i32,
    grid_color: Color,
    cage_color: Color,
    active_cell_background: Color,
    highlight_color: Color,
    notes_mode: bool,
    active_cell: Option<(usize, usize)>,
    active_number: u32,
    board_position: Point,
}

impl GameRenderer {
    pub fn new(canvas: RefCell<WindowCanvas>, game: RefCell<Game>, cell_size: u32) -> Self {
        Self {
            canvas,
            game,
            cell_size: cell_size as i32,
            cage_offset: 5,
            notes_gap: 3,
            grid_color: Color::BLACK,
            cage_color: Color::BLUE,
            active_cell_background: Color::RGB(172, 200, 229),
            highlight_color: Color::RGB(200, 208, 222),
            notes_mode: false,
            active_cell: None,
            active_number: 0,
            board_position: Point::new(1, 40),
        }
    }

    pub fn render(&self) -> Result<(), String> {
        self.set_color(&Color::WHITE);
        self.canvas_mut().clear();
        self.draw_status()?;
        self.draw_board()?;
        self.draw_buttons()?;
        self.draw_number_picker()?;
        self.canvas_mut().present();
        Ok(())
    }

    pub fn handle_click(&mut self, x: i32, y: i32) {
        let board_size = self.cell_size * 9;
        if self.between(x, self.board_position.x, self.board_position.x + board_size)
            && self.between(y, self.board_position.y, self.board_position.y + board_size)
        {
            let (r, c) = self.get_cell(x, y);
            self.active_cell = Some((r, c));
            return;
        }
        let picker_y = 100 + self.board_position.y + (9 * self.cell_size);
        if self.between(x, self.board_position.x, self.board_position.x + board_size)
            && self.between(y, picker_y, picker_y + self.cell_size)
        {
            if let Some((r, c)) = self.active_cell {
                let mut game = self.game_mut();
                let number = (self.get_col(x) + 1) as u32;
                if self.notes_mode {
                    game.cells[r][c].toggle_note(number as u8);
                } else if !game.set_value(r, c, number) {
                    game.mistakes += 1;
                }
            }
            return;
        }
        let button_y = 9 * self.cell_size + self.board_position.y + 20;
        if self.between(y, button_y, button_y + 60) {
            match self.get_button(x) {
                1 => self.undo(),
                2 => self.clear(),
                3 => self.toggle_notes_mode(),
                4 => self.hint(),
                _ => {}
            };
        }
    }

    pub fn handle_keyboard_input(&mut self, key: Keycode) {
        match key {
            Keycode::N => self.toggle_notes_mode(),
            Keycode::E => self.clear(),
            Keycode::U => self.undo(),
            Keycode::Up => self.move_to(-1, 0),
            Keycode::Down => self.move_to(1, 0),
            Keycode::Left => self.move_to(0,-1),
            Keycode::Right => self.move_to(0,1),
            Keycode::Num1 | Keycode::KP_1 => self.try_set(1),
            Keycode::Num2 | Keycode::KP_2 => self.try_set(2),
            Keycode::Num3 | Keycode::KP_3 => self.try_set(3),
            Keycode::Num4 | Keycode::KP_4 => self.try_set(4),
            Keycode::Num5 | Keycode::KP_5 => self.try_set(5),
            Keycode::Num6 | Keycode::KP_6 => self.try_set(6),
            Keycode::Num7 | Keycode::KP_7 => self.try_set(7),
            Keycode::Num8 | Keycode::KP_8 => self.try_set(8),
            Keycode::Num9 | Keycode::KP_9 => self.try_set(9),
            _ => {}
        }
    }

    fn move_to(&mut self, dr: i8, dc: i8) {
        if let Some((r,c)) = self.active_cell {
            let nr = (r as i8 + dr).max(0).min(8) as usize;
            let nc = (c as i8 + dc).max(0).min(8) as usize;
            self.active_cell = Some((nr, nc));
        } else {
            self.active_cell = Some((0,0));
        }        
    }

    fn try_set(&mut self, v: u32) {
        if let Some((r,c)) = self.active_cell {
            if !self.game_mut().set_value(r, c, v) {
                self.game_mut().mistakes+=1;
            }
        }
    }

    fn undo(&self) {
        self.game_mut().pop_state();
    }

    fn clear(&self) {
        if let Some((r, c)) = self.active_cell {
            self.game_mut().clear_cell(r, c);
        }
    }

    fn toggle_notes_mode(&mut self) {
        self.notes_mode = !self.notes_mode;
    }

    fn hint(&self) {
        if let Some((r, c)) = self.active_cell {
            let value = self.game_mut().board.solution[r][c];
            self.game_mut().set_value(r, c, value);
        }
    }

    fn get_cell(&self, x: i32, y: i32) -> (usize, usize) {
        let mod_y = (y - self.board_position.y) % self.cell_size;
        (((y - mod_y) / self.cell_size) as usize, self.get_col(x))
    }

    fn get_col(&self, x: i32) -> usize {
        let mod_x = (x - self.board_position.x) % self.cell_size;
        ((x - mod_x) / self.cell_size) as usize
    }

    fn get_button(&self, x: i32) -> u8 {
        let col = self.get_col(x);
        match col {
            1 => 1,
            3 => 2,
            5 => 3,
            7 => 4,
            _ => 0,
        }
    }

    fn between(&self, v: i32, start: i32, end: i32) -> bool {
        v > start && v < end
    }

    fn draw_board(&self) -> Result<(), String> {
        self.highlight_cells()?;
        self.draw_grid()?;
        self.draw_cages()?;
        self.draw_numbers()?;
        self.draw_notes()?;
        Ok(())
    }

    fn draw_status(&self) -> Result<(), String> {
        let status = format!("Errors: {}", self.game.borrow().mistakes);
        let elapsed = self.game.borrow().time.elapsed();
        let total_seconds = elapsed.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let time = format!("Time: {:02}:{:02}", minutes, seconds);        
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let font = ttf_context.load_font(FONT_PATH, 20)?;
        let surface_errors = font
            .render(&status)
            .blended(Color::BLACK)
            .map_err(|e| e.to_string())?;
        let surface_time = font
            .render(&time)
            .blended(Color::BLACK)
            .map_err(|e| e.to_string())?;
        let texture_creator = self.canvas_mut().texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface_errors) {
            let target = Rect::new(0, 10, surface_errors.width(), surface_errors.height());
            self.canvas_mut().copy(&texture, None, Some(target))?;
        };
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface_time) {
            let x = self.cell_size * 9 - (surface_time.width() as i32);
            let target = Rect::new(x, 10, surface_time.width(), surface_time.height());
            self.canvas_mut().copy(&texture, None, Some(target))?;
        }
        Ok(())
    }

    fn draw_buttons(&self) -> Result<(), String> {
        let img_size = 60i32;
        let y = 9 * self.cell_size + self.board_position.y + 20;
        let mut x = self.cell_size;
        let texture_creator = self.canvas_mut().texture_creator();
        self.add_image(x, y, img_size as u32, UNDO_ICON, &texture_creator)?;
        x += 2 * self.cell_size;
        self.add_image(x, y, img_size as u32, ERASE_ICON, &texture_creator)?;
        x += 2 * self.cell_size;
        if self.notes_mode {
            self.set_color(&self.highlight_color);
            self.canvas_mut()
                .fill_rect(Rect::new(x, y, img_size as u32, img_size as u32))?;
        }
        self.add_image(x, y, img_size as u32, NOTE_ICON, &texture_creator)?;
        x += 2 * self.cell_size;
        self.add_image(x, y, img_size as u32, HINT_ICON, &texture_creator)?;
        Ok(())
    }

    fn add_image(
        &self,
        x: i32,
        y: i32,
        size: u32,
        path: &str,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<(), String> {
        let image_texture = texture_creator
            .load_texture(path)
            .map_err(|e| e.to_string())?;
        let target = Rect::new(x, y, size, size);
        self.canvas_mut().copy(&image_texture, None, target)?;
        Ok(())
    }

    fn draw_number_picker(&self) -> Result<(), String> {
        let cs = self.cell_size as u32;
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let font = ttf_context.load_font(FONT_PATH, cs as u16)?;
        let texture_creator = self.canvas_mut().texture_creator();
        self.canvas_mut().set_draw_color(self.grid_color);
        let y = 100 + self.board_position.y + (9 * self.cell_size);
        for i in 0..9 {
            let x = i * self.cell_size;
            let rect = Rect::new(x, y, cs, cs);
            self.canvas_mut().draw_rect(rect)?;
            let surface = font
                .render(&(i + 1).to_string())
                .blended(Color::BLACK)
                .map_err(|e| e.to_string())?;
            if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                let ox = i * self.cell_size + (self.cell_size - surface.width() as i32) / 2;
                let oy = y + (self.cell_size - surface.height() as i32) / 2;
                let target = Rect::new(ox, oy, surface.width(), surface.height());
                self.canvas_mut().copy(&texture, None, Some(target))?;
            }
        }
        Ok(())
    }

    fn draw_grid(&self) -> Result<(), String> {
        let zero = 0;
        let end = 9 * self.cell_size;
        self.set_color(&self.grid_color);
        for i in 0..10 {
            let xs = i * self.cell_size;
            // vertical
            self.line(&xs, &zero, &xs, &end)?;
            // horizontal
            self.line(&zero, &xs, &end, &xs)?;
        }
        for i in 0..4 {
            let a = 3 * i * self.cell_size - 1;
            let b = 3 * i * self.cell_size + 1;
            // vertical
            self.line(&a, &zero, &a, &end)?;
            self.line(&b, &zero, &b, &end)?;
            // horizontal
            self.line(&zero, &a, &end, &a)?;
            self.line(&zero, &b, &end, &b)?;
        }
        Ok(())
    }

    fn draw_notes(&self) -> Result<(), String> {
        let notes_area_size = self.cell_size - 2 * self.cage_offset;
        let note_cell_size = (notes_area_size - 4 * self.notes_gap) / 3;
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let font = ttf_context.load_font(FONT_PATH, note_cell_size as u16)?;
        let bold_font = ttf_context.load_font(BOLD_FONT_PATH, note_cell_size as u16)?;
        for r in 0..9 {
            for c in 0..9 {
                let cell = &self.game.borrow().cells[r][c];
                let cx = c as i32 * self.cell_size + self.board_position.x;
                let cy = r as i32 * self.cell_size + self.board_position.y;
                for n in 1..=9 {
                    let nr = (n - 1) / 3;
                    let nc = (n - 1) % 3;
                    let mask = 1u16 << n;
                    let nv = cell.notes & mask;
                    if nv == 0 {
                        continue;
                    }
                    let active_font = if self.active_number == nv as u32 {
                        &bold_font
                    } else {
                        &font
                    };
                    let surface = active_font
                        .render(&n.to_string())
                        .blended(self.grid_color)
                        .map_err(|e| e.to_string())?;
                    let texture_creator = self.canvas_mut().texture_creator();
                    if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                        let x = cx + 3 * self.cage_offset + nc * note_cell_size + self.notes_gap;
                        let y = cy + self.cage_offset + nr * note_cell_size + self.notes_gap;
                        let target = Rect::new(x, y, surface.width(), surface.height());
                        self.canvas_mut().copy(&texture, None, Some(target))?;
                    };
                }
            }
        }
        Ok(())
    }

    fn draw_cages(&self) -> Result<(), String> {
        let c = self.cage_color;
        self.set_color(&c);
        let mut game = self.game_mut();
        for cage in game.board.cages.iter_mut() {
            if cage.lines == None {
                let lines: Vec<((i32, i32), (i32, i32))> = self.get_cage_lines(cage);
                cage.lines = Some(lines);
            }
            self.draw_sum(cage)?;
            if let Some(lines) = &cage.lines {
                for line in lines {
                    self.line(&line.0 .0, &line.0 .1, &line.1 .0, &line.1 .1)?;
                }
            }
        }
        Ok(())
    }

    fn draw_numbers(&self) -> Result<(), String> {
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let rect_size = (self.cell_size as f32 * 0.6) as u32;
        let font = ttf_context.load_font(FONT_PATH, rect_size as u16)?;
        let bold_font = ttf_context.load_font(BOLD_FONT_PATH, rect_size as u16)?;
        for r in 0..9 {
            let cy = r as i32 * self.cell_size + self.board_position.y;
            for c in 0..9 {
                let game = self.game.borrow();
                let cell = &game.cells[r][c];
                if cell.value == 0 {
                    continue;
                }
                let color = if cell.value == game.board.solution[r][c] {
                    self.grid_color
                } else {
                    Color::RED
                };
                let active_font = if cell.value == self.active_number {
                    &bold_font
                } else {
                    &font
                };
                let surface = active_font
                    .render(&cell.value.to_string())
                    .blended(color)
                    .map_err(|e| e.to_string())?;
                let texture_creator = self.canvas_mut().texture_creator();
                if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                    let cx = c as i32 * self.cell_size + self.board_position.x;
                    let ox = (self.cell_size - surface.width() as i32) / 2;
                    let oy = (self.cell_size - surface.height() as i32) / 2;
                    let target = Rect::new(cx + ox, cy + oy, surface.width(), surface.height());
                    self.canvas_mut().copy(&texture, None, Some(target))?;
                };
            }
        }
        Ok(())
    }

    fn to_point(&self, x: &i32, y: &i32) -> Point {
        let px = *x + self.board_position.x;
        let py = *y + self.board_position.y;
        Point::new(px, py)
    }

    fn line(&self, x1: &i32, y1: &i32, x2: &i32, y2: &i32) -> Result<(), String> {
        let start = self.to_point(x1, y1);
        let end = self.to_point(x2, y2);
        self.canvas_mut().draw_line(start, end)
    }

    fn set_color(&self, c: &Color) {
        self.canvas_mut().set_draw_color(*c);
    }

    fn canvas_mut(&self) -> RefMut<WindowCanvas> {
        self.canvas.borrow_mut()
    }

    fn game_mut(&self) -> RefMut<Game> {
        self.game.borrow_mut()
    }

    fn get_cage_lines(&self, cage: &Cage) -> Vec<((i32, i32), (i32, i32))> {
        let mut lines = vec![];

        for cell in &cage.cells {
            let x = cell.col as i32 * self.cell_size;
            let y = cell.row as i32 * self.cell_size;

            let mut draw_top = true;
            let mut draw_bottom = true;
            let mut draw_left = true;
            let mut draw_right = true;
            let mut offset_x1 = self.cage_offset;
            let mut offset_x2 = self.cage_offset;
            let mut offset_y1 = self.cage_offset;
            let mut offset_y2 = self.cage_offset;
            let mut top_left = false;
            let mut top_right = false;
            let mut bottom_left = false;
            let mut bottom_right = false;

            for neighbor in &cage.cells {
                if cage.is_adjacent(cell, neighbor) {
                    if neighbor.row < cell.row {
                        draw_top = false;
                        offset_y1 = 0;
                    }
                    if neighbor.row > cell.row {
                        draw_bottom = false;
                        offset_y2 = 0;
                    }
                    if neighbor.col < cell.col {
                        draw_left = false;
                        offset_x1 = 0;
                    }
                    if neighbor.col > cell.col {
                        draw_right = false;
                        offset_x2 = 0;
                    }
                    if neighbor.row < cell.row && neighbor.col != cell.col {
                        offset_y1 = -self.cage_offset;
                    }
                } else if cage.is_cross_join(cell, neighbor) {
                    bottom_left |= neighbor.row > cell.row && neighbor.col < cell.col;
                    bottom_right |= neighbor.row > cell.row && neighbor.col > cell.col;
                    top_left |= neighbor.row < cell.row && neighbor.col < cell.col;
                    top_right |= neighbor.row < cell.row && neighbor.col > cell.col;
                }
            }

            if draw_top {
                let ox1 = if top_left {
                    -self.cage_offset
                } else {
                    offset_x1
                };
                let ox2 = if top_right {
                    -self.cage_offset
                } else {
                    offset_x2
                };
                lines.push((
                    (x + ox1, y + offset_y1),
                    (x + self.cell_size - ox2, y + offset_y1),
                ));
            }
            if draw_bottom {
                let ox1 = if bottom_left {
                    -self.cage_offset
                } else {
                    offset_x1
                };
                let ox2 = if bottom_right {
                    -self.cage_offset
                } else {
                    offset_x2
                };
                lines.push((
                    (x + ox1, y + self.cell_size - offset_y2),
                    (x + self.cell_size - ox2, y + self.cell_size - offset_y2),
                ));
            }
            if draw_left {
                let oy1 = if top_left {
                    -self.cage_offset
                } else {
                    offset_y1
                };
                let oy2 = if bottom_left {
                    -self.cage_offset
                } else {
                    offset_y2
                };
                lines.push((
                    (x + offset_x1, y + oy1),
                    (x + offset_x1, y + self.cell_size - oy2),
                ));
            }
            if draw_right {
                let oy1 = if top_right {
                    -self.cage_offset
                } else {
                    offset_y1
                };
                let oy2 = if bottom_right {
                    -self.cage_offset
                } else {
                    offset_y2
                };
                lines.push((
                    (x + self.cell_size - offset_x2, y + oy1),
                    (x + self.cell_size - offset_x2, y + self.cell_size - oy2),
                ));
            }
        }

        lines
    }

    fn draw_sum(&self, cage: &Cage) -> Result<(), String> {
        let x = cage.cells[0].col as i32 * self.cell_size
            + self.cage_offset
            + 3
            + self.board_position.x;
        let y = cage.cells[0].row as i32 * self.cell_size
            + self.cage_offset
            + 3
            + self.board_position.y;
        let font_size = 10u16;
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let mut font = ttf_context.load_font(FONT_PATH, font_size)?;
        font.set_style(ttf::FontStyle::NORMAL);
        let surface = font
            .render(&cage.sum.to_string())
            .blended(self.cage_color)
            .map_err(|e| e.to_string())?;
        let texture_creator = self.canvas_mut().texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                x,
                y,
                u32::from(surface.rect().width()),
                u32::from(surface.rect().height()),
            );
            self.canvas_mut().copy(&texture, None, Some(target))?;
        };
        Ok(())
    }

    fn highlight_cells(&self) -> Result<(), String> {
        let current_color = self.canvas_mut().draw_color();
        if let Some((ar, ac)) = self.active_cell {
            for i in 0..9 {
                self.highlight_cell(ar, ac, ar, i)?;
                self.highlight_cell(ar, ac, i, ac)?;
            }
        }
        self.set_color(&current_color);
        Ok(())
    }

    fn highlight_cell(&self, ar: usize, ac: usize, r: usize, c: usize) -> Result<(), String> {
        let color = if ar == r && ac == c {
            &self.active_cell_background
        } else {
            &self.highlight_color
        };
        let active_cell = Rect::new(
            c as i32 * self.cell_size + self.board_position.x,
            r as i32 * self.cell_size + self.board_position.y,
            self.cell_size as u32,
            self.cell_size as u32,
        );
        self.set_color(&color);
        self.canvas_mut().fill_rect(active_cell)?;
        Ok(())
    }
}
