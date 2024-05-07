const std = @import("std");
const Token = @import("tokens.zig").Token;
const Errors = @import("tokens.zig").Errors;

pub const LexedToken = struct {
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

fn is_newline(ch: u8) bool {
    return ch == '\n' or ch == '\r';
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

        while ((self.curr_ch != '"' or ignore) and !is_newline(self.curr_ch) and self.curr_ch != 0) {
            if (self.curr_ch == ':' and !ignore) {
                ignore = true;
            } else {
                ignore = false;
                try stringArray.append(self.curr_ch);
            }
            self.read_ch();
        }

        if (self.curr_ch == 0) {
            return Token{ .illegal = Errors.UnterminatedString };
        }

        return Token{ .string = try stringArray.toOwnedSlice() };
    }

    fn read_multiline(self: *Self) !Token {
        var commentContents = std.ArrayList(u8).init(std.heap.page_allocator);
        defer commentContents.deinit();

        while (self.curr_ch != 0) {
            if (self.la("TLDR")) {
                break;
            }

            try commentContents.append(self.curr_ch);
            self.read_ch();
        }

        if (self.curr_ch == 0) {
            return Token{ .illegal = Errors.UnterminatedMultiLineComment };
        }
        return Token{ .multiLineComment = try commentContents.toOwnedSlice() };
    }

    fn skip_whitespace(self: *Self) void {
        const l = self;
        while (l.curr_ch == ' ' or l.curr_ch == '\t' or is_newline(l.curr_ch)) {
            self.read_ch();
        }
    }

    fn skip_single_comment(self: *Self) void {
        const l = self;
        while (!is_newline(l.curr_ch) and l.curr_ch != 0) {
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
            '-' => if (is_int(self.peak_ch())) self.read_number() else Token{ .illegal = Errors.UnexpectedToken },
            'A'...'Z', 'a'...'z', '_' => if (self.curr_ch == 'O' and self.la("BTW")) self.read_multiline() catch Token{ .illegal = Errors.CompilerError } else Token.parse_word(self.read_identifier()),
            '"' => self.read_string() catch Token{ .illegal = Errors.CompilerError },
            ',' => .comma,
            '!' => .exclamationMark,
            '?' => .questionMark,

            0 => .eof,
            else => Token{ .illegal = Errors.UnexpectedToken },
        };

        switch (token) {
            .singleLineComment => {
                self.skip_single_comment();
            },
            else => {},
        }

        const end = self.read_pos;
        self.read_ch();

        return LexedToken{ .token = token, .start = start, .end = end };
    }

    pub fn get_tokens(self: *Self) ![]LexedToken {
        var tokens = std.ArrayList(LexedToken).init(std.heap.page_allocator);
        defer tokens.deinit();

        while (self.curr_ch != 0) {
            const token = self.next_token();
            switch (token.token) {
                .singleLineComment => {},
                .multiLineComment => {},
                else => {
                    try tokens.append(token);
                },
            }
        }
        try tokens.append(self.next_token());

        return tokens.toOwnedSlice();
    }

    pub fn has_errors(tokens: []LexedToken) bool {
        for (tokens) |t| {
            switch (t.token) {
                .illegal => return true,
                else => {},
            }
        }

        return false;
    }

    pub fn get_first_error(tokens: []LexedToken) Token {
        for (tokens) |t| {
            switch (t.token) {
                .illegal => return t.token,
                else => {},
            }
        }

        return .eof;
    }
};
