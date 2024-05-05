const std = @import("std");
const Token = @import("tokens.zig").Token;

fn is_int(ch: u8) bool {
    return std.ascii.isDigit(ch);
}

pub const Lexer = struct {
    const Self = @This();

    src: []const u8,

    pos: usize = 0,
    read_pos: usize = 0,
    curr_ch: u8 = 0,

    pub fn init(src: []const u8) Self {
        var l = Self{ .src = src };
        l.read_ch();

        return l;
    }

    fn read_ch(self: *Self) void {
        if (self.read_pos >= self.src.len) {
            self.curr_ch = 0;
            return;
        }

        self.curr_ch = self.src[self.read_pos];
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn peak_ch(self: *Self) u8 {
        if (self.read_pos >= self.src.len) return 0;
        return self.src[self.read_pos];
    }

    fn read_number(self: *Self) Token {
        const start_pos = self.pos;
        var is_float = false;

        while (is_int(self.peak_ch()) or self.peak_ch() == '.') {
            self.read_ch();
            if (self.curr_ch == '.' and !is_float) {
                is_float = true;
            } else if (self.curr_ch == '.' and is_float) {
                break;
            }
        }

        if (is_float) {
            return Token{ .numbar = self.src[start_pos..self.read_pos] };
        }
        return Token{ .number = self.src[start_pos..self.read_pos] };
    }

    fn skip_whitespace(self: *Self) void {
        const l = self;
        while (l.curr_ch == ' ' or l.curr_ch == '\t' or l.curr_ch == '\n' or l.curr_ch == '\r') {
            self.read_ch();
        }
    }

    pub fn next_token(self: *Self) Token {
        self.skip_whitespace();

        const token: Token = switch (self.curr_ch) {
            '0'...'9' => self.read_number(),

            0 => .eof,
            else => .illegal,
        };

        self.read_ch();
        return token;
    }
};
