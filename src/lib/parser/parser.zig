const std = @import("std");

const tokens = @import("../lexer/tokens.zig");
const lexer = @import("../lexer/lexer.zig");

pub const ParerError = struct {
    message: []const u8,
    token: lexer.LexedToken,
};

pub const ReturnTypes = union(enum) {
    temp,
};

pub const MainReturn = struct {
    statements: []ReturnTypes,
    errors: []ParerError,
};

pub const Parser = struct {
    const Self = @This();

    tokens: []lexer.LexedToken,
    current: usize,
    errors: std.ArrayList(ParerError),

    pub fn parse(t: []lexer.LexedToken) !MainReturn {
        var statements = std.ArrayList(ReturnTypes).init(std.heap.page_allocator);
        defer statements.deinit();

        var p = Self{
            .tokens = t,
            .current = 0,
            .errors = std.ArrayList(ParerError).init(std.heap.page_allocator),
        };
        defer p.errors.deinit();

        while (!p.isAtEnd()) {
            const rt: ?ReturnTypes = try p.parseStatement();
            if (rt == null) {
                break;
            }
            try statements.append(rt.?);
        }

        return MainReturn{ .errors = try p.errors.toOwnedSlice(), .statements = try statements.toOwnedSlice() };
    }

    pub fn parseStatement(self: *Self) !?ReturnTypes {
        return switch (self.peek().token) {
            .eof => null,
            else => {
                try self.errors.append(ParerError{
                    .message = "Unexpected token",
                    .token = self.peek(),
                });
                return null;
            },
        };
    }

    pub fn check(self: *Self, token: tokens.Token) bool {
        if (self.isAtEnd()) {
            return false;
        }
        return switch (self.peek().token) {
            token => true,
            else => false,
        };
    }

    pub fn consume(self: *Self, token: tokens.Token) lexer.LexedToken {
        if (self.check(token)) return self.advance();
        std.log.err("Expected token {s}", .{token.to_name()});
        std.process.exit(1);
    }

    pub fn previous(self: *Self) lexer.LexedToken {
        return self.tokens[self.current - 1];
    }

    pub fn peek(self: *Self) lexer.LexedToken {
        return self.tokens[self.current];
    }

    pub fn peekFuture(self: *Self, amount: usize) lexer.LexedToken {
        return self.tokens[self.current + amount];
    }

    pub fn advance(self: *Self) lexer.LexedToken {
        if (!self.isAtEnd()) {
            self.current += 1;
        }
        return self.previous();
    }

    pub fn advanceAmount(self: *Self, amount: usize) lexer.LexedToken {
        if (!self.isAtEnd()) {
            self.current += amount;
        }
        return self.previous();
    }

    pub fn isAtEnd(self: *Self) bool {
        return switch (self.peek().token) {
            .eof => true,
            else => false,
        };
    }
};
