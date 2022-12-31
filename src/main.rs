use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    White,
    Black,
}

impl Color {
    fn other(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    fn forward(&self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    fn icon(&self, color: &Color) -> String {
        String::from(match self {
            PieceKind::Pawn => match color {
                Color::White => "♙",
                Color::Black => "♟",
            },
            PieceKind::Knight => match color {
                Color::White => "♘",
                Color::Black => "♞",
            },
            PieceKind::Bishop => match color {
                Color::White => "♗",
                Color::Black => "♝",
            },
            PieceKind::Rook => match color {
                Color::White => "♖",
                Color::Black => "♜",
            },
            PieceKind::Queen => match color {
                Color::White => "♕",
                Color::Black => "♛",
            },
            PieceKind::King => match color {
                Color::White => "♔",
                Color::Black => "♚",
            },
        })
    }
}

#[derive(Clone, Debug)]
struct Piece {
    kind: PieceKind,
    color: Color,
    has_moved: bool,
}

impl Piece {
    fn icon(&self) -> String {
        self.kind.icon(&self.color)
    }

    fn new(kind: PieceKind, color: Color, has_moved: bool) -> Piece {
        Piece {
            kind,
            color,
            has_moved,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Coord {
    row: i8,
    col: i8,
}

impl Coord {
    fn offset(&self, other: &Coord) -> Coord {
        Coord {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

#[derive(Clone)]
struct Board {
    tiles: Vec<Vec<Option<Piece>>>,
    history: Vec<(Coord, Coord)>,
}

impl Board {
    fn print(&self, side: Color) {
        let order: Vec<i8> = match side {
            Color::White => (0..8).rev().collect(),
            Color::Black => (0..8).collect(),
        };

        for row in order.iter() {
            print!("{} | ", row + 1);
            for col in order.iter().rev() {
                let piece = self.get(&Coord {
                    row: *row,
                    col: *col,
                });
                let icon = piece.map_or(String::from(" "), |piece| piece.icon());
                print!(" {} ", icon);
            }
            print!("\n");
        }
        print!("   ");
        for i in order.iter().rev() {
            print!("  {}", ('a' as u8 + *i as u8) as char);
        }
        println!();
    }

    fn new() -> Board {
        Board::from_pieces(Board::default_pieces())
    }

    fn from_pieces(pieces: Vec<(Coord, Piece)>) -> Board {
        let mut tiles: Vec<Vec<Option<Piece>>> =
            (0..8).map(|_| (0..8).map(|_| None).collect()).collect();
        for (pos, piece) in pieces {
            tiles[pos.row as usize][pos.col as usize] = Some(piece);
        }
        let board = Board {
            tiles,
            history: Vec::new(),
        };
        board
    }

    fn default_pieces() -> Vec<(Coord, Piece)> {
        let mut piece_attributes: Vec<(i8, i8, PieceKind, Color)> = vec![
            (0, 0, PieceKind::Rook, Color::White),
            (0, 7, PieceKind::Rook, Color::White),
            (0, 1, PieceKind::Knight, Color::White),
            (0, 6, PieceKind::Knight, Color::White),
            (0, 2, PieceKind::Bishop, Color::White),
            (0, 5, PieceKind::Bishop, Color::White),
            (0, 4, PieceKind::King, Color::White),
            (0, 3, PieceKind::Queen, Color::White),
        ];
        piece_attributes.extend((0..8).map(|col| (1, col, PieceKind::Pawn, Color::White)));
        let dark_piece_attributes: Vec<(i8, i8, PieceKind, Color)> = piece_attributes
            .iter()
            .map(|(row, col, kind, _)| (7 - *row, *col, kind.clone(), Color::Black))
            .collect();
        piece_attributes.extend(dark_piece_attributes);
        piece_attributes
            .into_iter()
            .map(|(row, col, kind, color)| (Coord { row, col }, Piece::new(kind, color, false)))
            .collect()
    }

    fn get(&self, pos: &Coord) -> Option<Piece> {
        self.tiles
            .get(pos.row as usize)?
            .get(pos.col as usize)?
            .clone()
    }

    fn set(&mut self, pos: &Coord, piece: Option<Piece>) {
        self.tiles[pos.row as usize][pos.col as usize] = piece;
    }

    fn contains(&self, pos: &Coord) -> bool {
        pos.row >= 0 && pos.col >= 0 && pos.row < 8 && pos.col < 8
    }

    fn path_between(&self, start: &Coord, end: &Coord) -> Vec<Coord> {
        // TODO could possibly speed up path_is_clear slightly by making this a generator
        let d_row = (end.row - start.row).signum();
        let d_col = (end.col - start.col).signum();
        let d = Coord {
            row: d_row,
            col: d_col,
        };

        let mut pos = start.offset(&d);
        let mut path = Vec::new();
        while pos != *end {
            path.push(pos.clone());
            pos = pos.offset(&d);
        }
        path
    }

    fn path_is_clear(&self, start: &Coord, end: &Coord) -> bool {
        self.path_between(start, end)
            .iter()
            .all(|pos| self.get(pos).is_none())
    }

    fn follows_pawn_move_pattern(&self, start_piece: &Piece, start: &Coord, end: &Coord) -> bool {
        let forward = start_piece.color.forward();
        if let Some(end_piece) = self.get(end) {
            end_piece.color != start_piece.color
                && end.row == start.row + forward
                && (end.col - start.col).abs() == 1
        } else {
            start.col == end.col
                && (end.row == start.row + forward
                    || end.row == start.row + forward * 2
                        && !start_piece.has_moved
                        && self.path_is_clear(start, end))
        }
    }

    fn is_en_passant(&self, start_piece: &Piece, start: &Coord, end: &Coord) -> bool {
        let forward = start_piece.color.forward();

        end.row == start.row + forward
            && (end.col - start.col).abs() == 1
            && match self.history.last() {
                Some((prev_start, prev_end)) => {
                    end.col == prev_end.col
                        && prev_start.col == prev_end.col
                        && prev_end.row == prev_start.row - forward * 2
                        && match self.get(prev_end) {
                            Some(prev_piece) => {
                                prev_piece.kind == PieceKind::Pawn
                                    && prev_piece.color != start_piece.color
                            }
                            None => false,
                        }
                }
                None => false,
            }
    }

    fn follows_knight_move_pattern(&self, start: &Coord, end: &Coord) -> bool {
        let d_row = (start.row - end.row).abs();
        let d_col = (start.col - end.col).abs();
        d_row == 1 && d_col == 2 || d_row == 2 && d_col == 1
    }

    fn follows_bishop_move_pattern(&self, start: &Coord, end: &Coord) -> bool {
        ((end.row - start.row).abs() == (end.col - start.col).abs())
            && self.path_is_clear(start, end)
    }

    fn follows_rook_move_pattern(&self, start: &Coord, end: &Coord) -> bool {
        ((end.row - start.row == 0) || (end.col - start.col == 0)) && self.path_is_clear(start, end)
    }

    fn follows_queen_move_pattern(&self, start: &Coord, end: &Coord) -> bool {
        self.follows_bishop_move_pattern(start, end) || self.follows_rook_move_pattern(start, end)
    }

    fn follows_king_move_pattern(&self, start: &Coord, end: &Coord) -> bool {
        let d_row = end.row - start.row;
        let d_col = end.col - start.col;
        d_row.abs() <= 1 && d_col.abs() <= 1
    }

    fn is_endangered(&self, side: &Color, pos: &Coord) -> bool {
        for row in 0..8 {
            for col in 0..8 {
                if self.is_legal_move_no_check(&side.other(), &Coord { row, col }, pos) {
                    return true;
                }
            }
        }
        false
    }

    fn is_castle(&self, piece: &Piece, start: &Coord, end: &Coord) -> bool {
        !piece.has_moved && {
            let d_row = end.row - start.row;
            let d_col = end.col - start.col;

            if d_row != 0 || d_col.abs() != 2 {
                return false;
            }

            let rook_col = if d_col == 2 { 7 } else { 0 };
            let rook_pos = Coord {
                row: start.row,
                col: rook_col,
            };
            if self
                .get(&rook_pos)
                .map(|rook| rook.has_moved)
                .unwrap_or(true)
            {
                return false;
            }

            let side = piece.color;
            !self.is_endangered(&side, start)
                && !self.is_endangered(&side, &rook_pos)
                && !self
                    .path_between(start, end)
                    .iter()
                    .any(|pos| self.is_endangered(&side, pos))
        }
    }

    fn is_legal_move_no_check(&self, side: &Color, start: &Coord, end: &Coord) -> bool {
        if !self.contains(end) {
            return false;
        }

        let start_piece = match self.get(start) {
            Some(piece) => piece,
            None => return false,
        };

        if start_piece.color != *side {
            return false;
        }

        if let Some(end_piece) = self.get(end) {
            if end_piece.color == start_piece.color {
                return false;
            }
        }

        match start_piece.kind {
            PieceKind::Pawn => {
                self.follows_pawn_move_pattern(&start_piece, start, end)
                    || self.is_en_passant(&start_piece, start, end)
            }
            PieceKind::Knight => self.follows_knight_move_pattern(start, end),
            PieceKind::Bishop => self.follows_bishop_move_pattern(start, end),
            PieceKind::Rook => self.follows_rook_move_pattern(start, end),
            PieceKind::Queen => self.follows_queen_move_pattern(start, end),
            PieceKind::King => {
                self.follows_king_move_pattern(start, end)
                    || self.is_castle(&start_piece, start, end)
            }
        }
    }

    fn find_king(&self, side: &Color) -> Coord {
        // TODO can make this more efficient by saving piece mapping
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get(&Coord { row, col }) {
                    if piece.kind == PieceKind::King && piece.color == *side {
                        return Coord { row, col };
                    }
                }
            }
        }
        panic!("Could not find king on board")
    }

    fn king_is_in_check(&self, side: &Color) -> bool {
        let king_pos = self.find_king(side);
        self.is_endangered(&side, &king_pos)
    }

    fn is_legal_move(&self, side: &Color, start: &Coord, end: &Coord) -> bool {
        self.is_legal_move_no_check(side, start, end) && {
            let mut check_board = self.clone();
            check_board.make_move(start, end);
            !check_board.king_is_in_check(side)
        }
    }

    fn get_legal_moves(&self, side: &Color, start: &Coord) -> Vec<Coord> {
        // TODO can make this more efficient by only checking pieces' move patterns
        let mut legal_moves: Vec<Coord> = Vec::new();
        for row in 0..8 {
            for col in 0..8 {
                let end = Coord { row, col };
                if self.is_legal_move(side, start, &end) {
                    legal_moves.push(end);
                }
            }
        }
        legal_moves
    }

    fn has_legal_moves(&self, side: &Color) -> bool {
        for row in 0..8 {
            for col in 0..8 {
                if self.get_legal_moves(side, &Coord { row, col }).len() > 0 {
                    return true;
                }
            }
        }
        false
    }

    fn undo_move(&mut self) {
        let mut board = Board::new();
        board.make_moves(&self.history[..self.history.len() - 1]);
        self.tiles = board.tiles;
        self.history = board.history;
    }

    fn make_move(&mut self, start: &Coord, end: &Coord) {
        if let Some(mut piece) = self.get(start) {
            match piece.kind {
                PieceKind::Pawn => {
                    if self.is_en_passant(&piece, start, end) {
                        self.set(
                            &Coord {
                                row: start.row,
                                col: end.col,
                            },
                            None,
                        );
                    }
                    if end.row == 7 || end.row == 0 {
                        self.set(end, Some(Piece::new(PieceKind::Queen, piece.color, true)));
                    }
                }
                PieceKind::King => {
                    if end.col - start.col > 1 {
                        self.set(
                            &Coord {
                                row: start.row,
                                col: end.col - 1,
                            },
                            Some(Piece::new(PieceKind::Rook, piece.color, true)),
                        );
                        self.set(
                            &Coord {
                                row: start.row,
                                col: start.col + 3,
                            },
                            None,
                        );
                    } else if end.col - start.col < -1 {
                        self.set(
                            &Coord {
                                row: start.row,
                                col: end.col + 1,
                            },
                            Some(Piece::new(PieceKind::Rook, piece.color, true)),
                        );
                        self.set(
                            &Coord {
                                row: start.row,
                                col: start.col - 4,
                            },
                            None,
                        );
                    }
                }
                _ => {}
            }

            piece.has_moved = true;
            self.set(end, Some(piece));
            self.set(start, None);

            self.history.push((start.clone(), end.clone()))
        }
    }

    fn make_moves(&mut self, moves: &[(Coord, Coord)]) {
        for (start, end) in moves {
            self.make_move(start, end);
        }
    }

    fn parse_move_string(move_string: &str) -> Option<(Coord, Coord)> {
        let split: Vec<&str> = move_string.trim().split_whitespace().collect();
        if split.len() != 2 {
            return None;
        }

        let bytes = split[0].as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let start_col = (bytes[0] as u8) - ('a' as u8);
        let start_row = ((bytes[1] as u8) - ('0' as u8)) - 1;

        let bytes = split[1].as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let end_col = (bytes[0] as u8) - ('a' as u8);
        let end_row = ((bytes[1] as u8) - ('0' as u8)) - 1;

        if start_col >= 8 || start_row >= 8 || end_col >= 8 || end_row >= 8 {
            return None;
        }

        Some((
            Coord {
                row: start_row as i8,
                col: start_col as i8,
            },
            Coord {
                row: end_row as i8,
                col: end_col as i8,
            },
        ))
    }

    fn play(&mut self) {
        let mut side = Color::White;
        loop {
            self.print(side);
            println!("Enter a move:");

            let mut move_string = String::new();

            io::stdin()
                .read_line(&mut move_string)
                .expect("Failed to read move");

            let option_move = Board::parse_move_string(&move_string);

            if let Some((start, end)) = option_move {
                if self.is_legal_move(&side, &start, &end) {
                    self.make_move(&start, &end);
                    side = side.other();
                }
            }

            if !self.has_legal_moves(&side) {
                self.print(side);
                if self.king_is_in_check(&side) {
                    println!("Checkmate! {:?} wins!", side.other());
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
