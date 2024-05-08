const std = @import("std");

const tokens = @import("../lexer/tokens.zig");
const lexer = @import("../lexer/lexer.zig");
const ast = @import("./ast.zig");

const allocator = std.heap.page_allocator;

pub const ParserError = struct {
    message: []const u8,
    token: lexer.LexedToken,
};

const IntermediateParserError = error{
    ConsumeTokenError,
    AdvanceTokenError,

    ParseStatementError,
    ParseNumberValueError,
};

pub const ParserReturn = struct {
    program: ast.ProgramNode,
    errors: []ParserError,
};

pub const Parser = struct {
    const Self = @This();

    tokens: []lexer.LexedToken,
    current: usize,
    errors: std.ArrayList(ParserError),

    pub fn parse(t: []lexer.LexedToken) ParserReturn {
        var parser = Self{
            .tokens = t,
            .current = 0,
            .errors = std.ArrayList(ParserError).init(allocator),
        };
        defer parser.errors.deinit();

        const program = parser.parse_program();
        return ParserReturn{
            .program = program,
            .errors = parser.errors.toOwnedSlice() catch &[_]ParserError{},
        };
    }

    pub fn parse_program(self: *Self) ast.ProgramNode {
        var statements = std.ArrayList(ast.StatementNode).init(allocator);
        defer statements.deinit();

        while (!self.isAtEnd()) {
            const parsed_statement = self.parse_statement() catch null;
            if (parsed_statement == null) {
                self.errors.append(ParserError{ .message = "Expected valid statement line", .token = self.peek() }) catch {};
                return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
            }
            statements.append(parsed_statement.?) catch {};
        }

        return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
    }

    pub fn parse_statement(self: *Self) IntermediateParserError!ast.StatementNode {
        if (self.check("numbarValue")) {
            return ast.StatementNode{ .value = ast.StatementNodeValueOption{
                .NumbarValue = try self.parse_numbarvalue(),
            } };
        }

        if (self.check("numberValue")) {
            return ast.StatementNode{ .value = ast.StatementNodeValueOption{
                .NumberValue = try self.parse_numbervalue(),
            } };
        }

        self.errors.append(ParserError{ .message = "Expected valid statement or expression", .token = self.peek() }) catch {};
        return IntermediateParserError.ParseStatementError;
    }

    pub fn parse_numbervalue(self: *Self) IntermediateParserError!ast.NumberValueNode {
        const token = self.consume("numberValue") catch null;
        if (token == null) {
            self.errors.append(ParserError{ .message = "Expected Number Value Token", .token = self.peek() }) catch {};
            return IntermediateParserError.ParseNumberValueError;
        }

        return ast.NumberValueNode{ .token = token.? };
    }

    pub fn parse_numbarvalue(self: *Self) IntermediateParserError!ast.NumbarValueNode {
        const token = self.consume("numbarValue") catch null;
        if (token == null) {
            self.errors.append(ParserError{ .message = "Expected Numbar Value Token", .token = self.peek() }) catch {};
            return IntermediateParserError.ParseNumberValueError;
        }

        return ast.NumbarValueNode{ .token = token.? };
    }

    pub fn check(self: *Self, token: []const u8) bool {
        if (std.mem.eql(u8, self.peek().token.to_name(), token)) {
            return true;
        }
        return false;
    }

    pub fn consume(self: *Self, token: []const u8) IntermediateParserError!ast.TokenNode {
        if (self.check(token)) {
            _ = try self.advance();
            return ast.TokenNode{ .token = self.previous() };
        }
        return IntermediateParserError.ConsumeTokenError;
    }

    pub fn previous(self: *Self) lexer.LexedToken {
        return self.tokens[self.current - 1];
    }

    pub fn peek(self: *Self) lexer.LexedToken {
        return self.tokens[self.current];
    }

    pub fn advance(self: *Self) IntermediateParserError!lexer.LexedToken {
        if (!self.isAtEnd()) {
            self.current += 1;
            return self.peek();
        }
        return IntermediateParserError.AdvanceTokenError;
    }

    pub fn isAtEnd(self: *Self) bool {
        return switch (self.peek().token) {
            .eof => true,
            else => false,
        };
    }
};
