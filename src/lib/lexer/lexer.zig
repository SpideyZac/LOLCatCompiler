const std = @import("std");
const Token = @import("tokens.zig").Token;

const LexedToken = struct {
    token: Token,
    start: usize,
    end: usize,

    pub fn to_name(self: LexedToken) []const u8 {
        return self.token.to_name();
    }
};

fn is_int(ch: u8) bool {
    return std.ascii.isDigit(ch);
}

fn is_char(ch: u8) bool {
    return std.ascii.isAlphanumeric(ch) or ch == '_';
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
            return Token{ .numbarValue = self.src[start_pos..self.read_pos] };
        }
        return Token{ .numberValue = self.src[start_pos..self.read_pos] };
    }

    fn read_identifier(self: *Self) []const u8 {
        const start_pos = self.pos;

        while (is_char(self.peak_ch()) or is_int(self.peak_ch())) {
            self.read_ch();
        }

        return self.src[start_pos..self.read_pos];
    }

    fn read_string(self: *Self) !Token {
        self.read_ch();
        var ignore = false;

        var stringArray = std.ArrayList(u8).init(std.heap.page_allocator);
        defer stringArray.deinit();

        while ((self.curr_ch != '"' or ignore) and self.curr_ch != 0) {
            if (self.curr_ch == ':' and !ignore) {
                ignore = true;
            } else {
                ignore = false;
                try stringArray.append(self.curr_ch);
            }
            self.read_ch();
        }

        if (self.curr_ch == 0) {
            return .illegal;
        }

        return Token { .string = try stringArray.toOwnedSlice() };
    }

    fn skip_whitespace(self: *Self) void {
        const l = self;
        while (l.curr_ch == ' ' or l.curr_ch == '\t' or l.curr_ch == '\n' or l.curr_ch == '\r') {
            self.read_ch();
        }
    }

    fn la(self: *Self, t: []const u8) bool {
        if (self.read_pos + t.len > self.src.len) return false;
        var success = false;
        if (std.mem.eql(u8, t, self.src[self.read_pos..(self.read_pos + t.len)])) {
            success = true;
        }

        if (success) {
            for (0..t.len) |_| {
                self.read_ch();
            }
        }

        return success;
    }

    pub fn next_token(self: *Self) LexedToken {
        self.skip_whitespace();
        const start = self.pos;

        const token: Token = switch (self.curr_ch) {
            '0'...'9' => self.read_number(),
            '-' => if (is_int(self.peak_ch())) self.read_number() else .illegal,
            'A'...'Z', 'a'...'z', '_' => Token.parse_word(self.read_identifier()),
            '"' => self.read_string() catch .illegal,

            0 => .eof,
            else => .illegal,
        };

        const end = self.read_pos;
        self.read_ch();

        return LexedToken{ .token = token, .start = start, .end = end };
    }
};
