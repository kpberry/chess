use std::io;
use std::mem;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Light,
    Dark,
}

impl Color {
    fn other(&self) -> Color {
        match self {
            Color::Light => Color::Dark,
            Color::Dark => Color::Light,
        }
    }

    fn dir(&self) -> i8 {
        match self {
            Color::Light => -1,
            Color::Dark => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Piece {
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),
    Empty,
}

impl Piece {
    fn get_icon(&self) -> &str {
        match self {
            Piece::Pawn(color) => match color {
                Color::Light => "♙",
                Color::Dark => "♟",
            },
            Piece::Knight(color) => match color {
                Color::Light => "♘",
                Color::Dark => "♞",
            },
            Piece::Bishop(color) => match color {
                Color::Light => "♗",
                Color::Dark => "♝",
            },
            Piece::Rook(color) => match color {
                Color::Light => "♖",
                Color::Dark => "♜",
            },
            Piece::Queen(color) => match color {
                Color::Light => "♕",
                Color::Dark => "♛",
            },
            Piece::King(color) => match color {
                Color::Light => "♔",
                Color::Dark => "♚",
            },
            Piece::Empty => " ",
        }
    }

    fn same_color(&self, color: &Color) -> bool {
        match self {
            Piece::Pawn(c)
            | Piece::Knight(c)
            | Piece::Bishop(c)
            | Piece::Rook(c)
            | Piece::Queen(c)
            | Piece::King(c) => c == color,
            _ => false,
        }
    }

    fn other_color(&self, color: &Color) -> bool {
        return !self.same_color(color);
    }
}

struct Board {
    tiles: [[Piece; 8]; 8],
    history: Vec<((i8, i8), (i8, i8))>,
}

impl Board {
    fn print(&self, side: Color) {
        match side {
            Color::Light => {
                for i in 0..8 {
                    print!("{} | ", 8 - i);
                    for j in 0..8 {
                        print!(" {} ", self.tiles[j][i].get_icon());
                    }
                    print!("\n");
                }
                print!("   ");
                for i in 0..8 {
                    print!("  {}", (('a' as u8) + i) as char);
                }
                println!();
            }
            Color::Dark => {
                for i in (0..8).rev() {
                    print!("{} | ", 8 - i);
                    for j in (0..8).rev() {
                        print!(" {} ", self.tiles[j][i].get_icon());
                    }
                    print!("\n");
                }
                print!("   ");
                for i in (0..8).rev() {
                    print!("  {}", (('a' as u8) + i) as char);
                }
                println!();
            }
        };
    }

    fn new() -> Board {
        let mut board = unsafe {
            let mut _board = Board {
                tiles: mem::uninitialized(),
                history: vec![],
            };
            for x in 0..8 {
                for y in 0..8 {
                    _board.tiles[x][y] = Piece::Empty;
                }
            }
            _board
        };

        board.reset_pieces();

        board
    }

    fn reset_pieces(&mut self) {
        for x in 0..8 {
            for y in 0..8 {
                self.tiles[x][y] = Piece::Empty;
            }
        }

        self.tiles[0][0] = Piece::Rook(Color::Dark);
        self.tiles[7][0] = Piece::Rook(Color::Dark);
        self.tiles[0][7] = Piece::Rook(Color::Light);
        self.tiles[7][7] = Piece::Rook(Color::Light);

        self.tiles[1][0] = Piece::Knight(Color::Dark);
        self.tiles[6][0] = Piece::Knight(Color::Dark);
        self.tiles[1][7] = Piece::Knight(Color::Light);
        self.tiles[6][7] = Piece::Knight(Color::Light);

        self.tiles[2][0] = Piece::Bishop(Color::Dark);
        self.tiles[5][0] = Piece::Bishop(Color::Dark);
        self.tiles[2][7] = Piece::Bishop(Color::Light);
        self.tiles[5][7] = Piece::Bishop(Color::Light);

        self.tiles[4][0] = Piece::King(Color::Dark);
        self.tiles[4][7] = Piece::King(Color::Light);

        self.tiles[3][0] = Piece::Queen(Color::Dark);
        self.tiles[3][7] = Piece::Queen(Color::Light);

        for i in 0..8 {
            self.tiles[i][1] = Piece::Pawn(Color::Dark);
            self.tiles[i][6] = Piece::Pawn(Color::Light);
        }
    }

    fn contains(pos: (i8, i8)) -> bool {
        let (i, j) = pos;
        i >= 0 && j >= 0 && i < 8 && j < 8
    }

    fn offset_within(pos: (i8, i8), offsets: Vec<(i8, i8)>) -> Vec<(i8, i8)> {
        let (x, y) = pos;
        offsets
            .into_iter()
            .map(|(i, j)| (x + i, y + j))
            .filter(|&pos| Board::contains(pos))
            .collect()
    }

    fn occupied(&self, pos: (i8, i8)) -> bool {
        let (x, y) = pos;
        if let Piece::Empty = self.tiles[x as usize][y as usize] {
            false
        } else {
            true
        }
    }

    fn path_is_clear(&self, start: (i8, i8), end: (i8, i8)) -> bool {
        let mut dx = end.0 - start.0;
        if dx > 1 {
            dx = 1;
        }
        if dx < -1 {
            dx = -1;
        }
        let mut dy = end.1 - start.1;
        if dy > 1 {
            dy = 1;
        }
        if dy < -1 {
            dy = -1;
        }

        let mut pos = (start.0 + dx, start.1 + dy);
        while pos != end {
            if self.occupied(pos) {
                return false;
            }
            pos = (pos.0 + dx, pos.1 + dy);
        }
        true
    }

    fn has_moved(&self, pos: (i8, i8)) -> bool {
        for h in &self.history {
            if h.0 == pos || h.1 == pos {
                return true;
            }
        }
        false
    }

    fn en_passant(&self, pos: (i8, i8), color: &Color) -> bool {
        if self.history.len() == 0 {
            return false;
        }

        let dir = color.dir();

        let (start, end) = self.history[self.history.len() - 1];
        let (_start_x, start_y) = start;
        let (end_x, end_y) = end;
        if let Piece::Pawn(_) = self.tiles[end_x as usize][end_y as usize] {
            start_y == end_y + dir * 2 && pos == (end_x, end_y + dir)
        } else {
            false
        }
    }

    fn is_endangered(&self, side: &Color, pos: (i8, i8)) -> bool {
        let other = side.other();
        for i in 0..8 {
            for j in 0..8 {
                if self.tiles[i][j].other_color(side)
                    && self.is_valid_move((i as i8, j as i8), pos, other, false, false)
                {
                    return true;
                }
            }
        }

        false
    }

    fn find_king(&self, side: &Color) -> Option<(i8, i8)> {
        for i in 0..8 {
            for j in 0..8 {
                if let Piece::King(c) = self.tiles[i][j] {
                    if c == *side {
                        return Some((i as i8, j as i8));
                    }
                }
            }
        }
        None
    }

    fn king_is_in_check(&self, side: &Color) -> bool {
        let king_pos = self.find_king(side);
        match king_pos {
            Some(pos) => self.is_endangered(side, pos),
            None => false,
        }
    }

    fn king_is_in_check_after_move(&self, side: &Color, start: (i8, i8), end: (i8, i8)) -> bool {
        let mut board = Board::new();
        board.make_moves(&self.history);
        board.make_move(start, end);
        board.king_is_in_check(side)
    }

    fn get_pawn_moves(&self, pos: (i8, i8), color: &Color) -> Vec<(i8, i8)> {
        let dir = color.dir();

        let mut offsets = vec![(0, dir)];

        let occupied_offsets = vec![(-1, dir), (1, dir)];
        let (x, y) = pos;
        for offset in occupied_offsets {
            let (i, j) = offset;
            let new_pos = (x + i, y + j);
            if Board::contains(new_pos) {
                if self.occupied(new_pos) || self.en_passant(new_pos, color) {
                    offsets.push(offset);
                }
            }
        }

        let first_turn_offset = (0, 2 * dir);
        let first_turn = match color {
            Color::Light => y == 6,
            Color::Dark => y == 1,
        };
        if first_turn {
            if !self.occupied((pos.0, pos.1 + first_turn_offset.1)) {
                offsets.push(first_turn_offset);
            }
        }

        Board::offset_within(pos, offsets)
    }

    fn get_knight_moves(&self, pos: (i8, i8)) -> Vec<(i8, i8)> {
        let offsets: Vec<(i8, i8)> = vec![
            (-2, -1),
            (-2, 1),
            (2, -1),
            (2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
        ];
        Board::offset_within(pos, offsets)
    }

    fn get_bishop_moves(&self, pos: (i8, i8)) -> Vec<(i8, i8)> {
        let mut offsets: Vec<(i8, i8)> = vec![];
        for i in -7..=7 {
            offsets.push((i, i));
            offsets.push((i, -i));
        }
        Board::offset_within(pos, offsets)
    }

    fn get_rook_moves(&self, pos: (i8, i8)) -> Vec<(i8, i8)> {
        let mut offsets: Vec<(i8, i8)> = vec![];
        for i in -7..=7 {
            offsets.push((i, 0));
            offsets.push((0, i));
        }
        Board::offset_within(pos, offsets)
    }

    fn get_queen_moves(&self, pos: (i8, i8)) -> Vec<(i8, i8)> {
        let mut moves = self.get_bishop_moves(pos);
        moves.extend(self.get_rook_moves(pos));
        moves
    }

    fn get_king_moves(&self, pos: (i8, i8), side: &Color, check_castle: bool) -> Vec<(i8, i8)> {
        let mut offsets: Vec<(i8, i8)> = vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        if !self.has_moved(pos) && check_castle {
            if !self.has_moved((pos.0 + 3, pos.1)) {
                let positive_offsets = vec![(0, 0), (1, 0), (2, 0), (3, 0)];
                let positive_offsets = Board::offset_within(pos, positive_offsets);
                let safe: Vec<(i8, i8)> = positive_offsets
                    .into_iter()
                    .filter(|pos| !self.is_endangered(side, *pos))
                    .collect();
                if safe.len() == 4 {
                    offsets.push((2, 0));
                }
            }

            if !self.has_moved((pos.0 - 4, pos.1)) {
                let negative_offsets = vec![(0, 0), (-1, 0), (-2, 0), (-3, 0), (-4, 0)];
                let negative_offsets = Board::offset_within(pos, negative_offsets);
                let safe: Vec<(i8, i8)> = negative_offsets
                    .into_iter()
                    .filter(|pos| !self.is_endangered(side, *pos))
                    .collect();
                if safe.len() == 5 {
                    offsets.push((-2, 0));
                }
            }
        }

        Board::offset_within(pos, offsets)
    }

    fn get_moves(&self, pos: (i8, i8), check_castle: bool) -> Vec<(i8, i8)> {
        let (x, y) = pos;
        match &self.tiles[x as usize][y as usize] {
            Piece::Pawn(color) => self.get_pawn_moves(pos, &color),
            Piece::Knight(_) => self.get_knight_moves(pos),
            Piece::Bishop(_) => self.get_bishop_moves(pos),
            Piece::Rook(_) => self.get_rook_moves(pos),
            Piece::Queen(_) => self.get_queen_moves(pos),
            Piece::King(color) => self.get_king_moves(pos, &color, check_castle),
            Piece::Empty => vec![],
        }
    }

    fn is_valid_move(
        &self,
        start: (i8, i8),
        end: (i8, i8),
        side: Color,
        check_castle: bool,
        check_check: bool,
    ) -> bool {
        let (x, y) = (start.0 as usize, start.1 as usize);
        let same_color = self.tiles[x][y].same_color(&side);
        let other_color = self.tiles[end.0 as usize][end.1 as usize].other_color(&side);
        let has_move = self.get_moves(start, check_castle).contains(&end);
        let check = match check_check {
            true => self.king_is_in_check_after_move(&side, start, end),
            false => false,
        };
        if !(same_color && has_move && other_color) || check {
            return false;
        }
        match self.tiles[x][y] {
            Piece::Knight(_) => true,
            _ => self.path_is_clear(start, end),
        }
    }

    fn get_valid_moves(
        &self,
        start: (i8, i8),
        side: Color,
        check_castle: bool,
        check_check: bool,
    ) -> Vec<(i8, i8)> {
        let moves = self.get_moves(start, check_castle);
        moves
            .into_iter()
            .filter(|&m| self.is_valid_move(start, m, side, check_castle, check_check))
            .collect()
    }

    fn has_valid_moves(&self, side: Color) -> bool {
        for i in 0..8 {
            for j in 0..8 {
                if self
                    .get_valid_moves((i as i8, j as i8), side, true, true)
                    .len()
                    > 0
                {
                    return true;
                }
            }
        }
        false
    }

    fn undo_move(&mut self) {
        let old_history = self.history.clone();
        self.reset_pieces();
        self.history = vec![];
        self.make_moves(&old_history[..old_history.len() - 1]);
    }

    fn make_move(&mut self, start: (i8, i8), end: (i8, i8)) {
        let (start_x, start_y) = (start.0 as usize, start.1 as usize);
        let (end_x, end_y) = (end.0 as usize, end.1 as usize);
        if let Piece::Pawn(color) = self.tiles[start_x][start_y] {
            if self.en_passant(end, &color) {
                self.tiles[end_x][start_y] = Piece::Empty;
            }
        }
        if let Piece::King(color) = self.tiles[start_x][start_y] {
            if end.0 - start.0 > 1 {
                self.tiles[end_x - 1][start_y] = Piece::Rook(color);
                self.tiles[start_x + 3][start_y] = Piece::Empty;
            } else if end.0 - start.0 < -1 {
                self.tiles[end_x + 1][start_y] = Piece::Rook(color);
                self.tiles[start_x - 4][start_y] = Piece::Empty;
            }
        }
        self.tiles[end_x][end_y] = self.tiles[start_x][start_y];
        if let Piece::Pawn(color) = self.tiles[start_x][start_y] {
            if end_y == 7 || end_y == 0 {
                self.tiles[end_x][end_y] = Piece::Queen(color);
            }
        }
        self.tiles[start_x][start_y] = Piece::Empty;
        self.history.push((start, end))
    }

    fn make_moves(&mut self, moves: &[((i8, i8), (i8, i8))]) {
        for m in moves {
            self.make_move(m.0, m.1);
        }
    }

    fn parse_move_string(move_string: &str) -> Option<((i8, i8), (i8, i8))> {
        let split: Vec<&str> = move_string.trim().split_whitespace().collect();
        if split.len() != 2 {
            return None;
        }

        let bytes = split[0].as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let start_x = (bytes[0] as u8) - ('a' as u8);
        let start_y = 8 - ((bytes[1] as u8) - ('0' as u8));

        let bytes = split[1].as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let end_x = (bytes[0] as u8) - ('a' as u8);
        let end_y = 8 - ((bytes[1] as u8) - ('0' as u8));

        if start_x >= 8 || start_y >= 8 || end_x >= 8 || end_y >= 8 {
            return None;
        }

        Some(((start_x as i8, start_y as i8), (end_x as i8, end_y as i8)))
    }

    fn play(&mut self) {
        let mut side = Color::Light;
        loop {
            self.print(side);
            println!("Enter a move:");

            let mut move_string = String::new();

            io::stdin()
                .read_line(&mut move_string)
                .expect("Failed to read move");

            let option_move = Board::parse_move_string(&move_string);

            if let Some((start, end)) = option_move {
                if self.is_valid_move(start, end, side, true, true) {
                    self.make_move(start, end);

                    side = side.other();
                }
            }

            if !self.has_valid_moves(side) {
                self.print(side);
                if self.king_is_in_check(&side) {
                    println!("Checkmate! {:?} wins!", side);
                    break;
                } else {
                    println!("Stalemate!");
                    break;
                }
            }
        }
    }
}

fn main() {
    Board::new().play();
}
