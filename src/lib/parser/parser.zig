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
    UnconsumeTokenError,
    AdvanceTokenError,

    ParseStatementError,

    ParseExpressionError,
    ParseNumberValueError,
    ParseNumbarValueError,

    ParseKTHXBYE_WordError,

    ParseVariableDeclarationError,
    ParseVariableAssignmentError,
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
    stmts: std.ArrayList(ast.StatementNode),
    consumed_tokens: std.ArrayList(bool),

    pub fn parse(t: []lexer.LexedToken) ParserReturn {
        var parser = Self{
            .tokens = t,
            .current = 0,
            .errors = std.ArrayList(ParserError).init(allocator),
            .stmts = std.ArrayList(ast.StatementNode).init(allocator),
            .consumed_tokens = std.ArrayList(bool).init(allocator),
        };
        defer parser.errors.deinit();
        defer parser.stmts.deinit();
        defer parser.consumed_tokens.deinit();

        for (t) |_| {
            parser.consumed_tokens.append(false) catch {};
        }

        const program = parser.parse_program();

        const errors = parser.errors.toOwnedSlice() catch &[_]ParserError{};
        var filtered_errors = std.ArrayList(ParserError).init(allocator);
        defer filtered_errors.deinit();
        for (errors) |e| {
            if (parser.consumed_tokens.items[e.token.index]) {
                continue;
            }
            filtered_errors.append(e) catch {};
        }

        return ParserReturn{
            .program = program,
            .errors = filtered_errors.toOwnedSlice() catch &[_]ParserError{},
        };
    }

    pub fn parse_program(self: *Self) ast.ProgramNode {
        var statements = std.ArrayList(ast.StatementNode).init(allocator);
        defer statements.deinit();

        const hai = self.consume("word_hai") catch null;
        if (hai == null) {
            self.create_error(ParserError{ .message = "Expected HAI to start program", .token = self.peek() });
            return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
        }
        const version = self.parse_numbarvalue() catch null;
        if (version == null) {
            self.create_error(ParserError{ .message = "Expected version number (of type NUMBAR)", .token = self.peek() });
            return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
        }
        if (version.?.value() != 1.2) {
            self.create_error(ParserError{ .message = "Expected version number 1.2", .token = self.previous() });
            return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
        }

        while (!self.isAtEnd()) {
            const parsed_statement = self.parse_statement() catch null;
            if (parsed_statement == null) {
                self.create_error(ParserError{ .message = "Expected valid statement line", .token = self.peek() });
                return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
            }
            self.stmts.append(parsed_statement.?) catch {};
            statements.append(parsed_statement.?) catch {};
        }

        switch (statements.items[statements.items.len - 1].option) {
            .KTHXBYE_Word => {},
            else => {
                self.create_error(ParserError{ .message = "Expected KTHXBYE to end program", .token = self.previous() }); 
            },
        }

        return ast.ProgramNode{ .statements = statements.toOwnedSlice() catch &[_]ast.StatementNode{} };
    }

    pub fn parse_statement(self: *Self) IntermediateParserError!ast.StatementNode {
        // kthxbye can also be used to terminate a program so we don't remove it
        if (self.check("word_kthxbye")) {
            return ast.StatementNode{ .option = ast.StatementNodeValueOption{
                .KTHXBYE_Word = try self.parse_KTHXBYE_word(),
            } };
        }

        const variable_declaration = self.parse_variable_declaration() catch null;
        if (variable_declaration != null) {
            return ast.StatementNode{ .option = ast.StatementNodeValueOption {
                .VariableDeclaration = variable_declaration.?,
            } };
        }

        const expression = self.parse_expression() catch null;
        if (expression != null) {
            return ast.StatementNode{ .option = ast.StatementNodeValueOption {
                .Expression = expression.?,
            } };
        }

        self.create_error(ParserError{ .message = "Expected valid statement or expression", .token = self.peek() });
        return IntermediateParserError.ParseStatementError;
    }

    pub fn parse_expression(self: *Self) IntermediateParserError!ast.ExpressionNode {
        if (self.check("numbarValue")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .NumbarValue = try self.parse_numbarvalue(),
            } };
        }

        if (self.check("numberValue")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .NumberValue = try self.parse_numbervalue(),
            } };
        }

        self.create_error(ParserError{ .message = "Expected valid expression", .token = self.peek() });
        return IntermediateParserError.ParseExpressionError;
    }

    pub fn parse_numbervalue(self: *Self) IntermediateParserError!ast.NumberValueNode {
        const token = self.consume("numberValue") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected Number Value Token", .token = self.peek() });
            return IntermediateParserError.ParseNumberValueError;
        }

        return ast.NumberValueNode{ .token = token.? };
    }

    pub fn parse_numbarvalue(self: *Self) IntermediateParserError!ast.NumbarValueNode {
        const token = self.consume("numbarValue") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected Numbar Value Token", .token = self.peek() });
            return IntermediateParserError.ParseNumbarValueError;
        }

        return ast.NumbarValueNode{ .token = token.? };
    }

    pub fn parse_KTHXBYE_word(self: *Self) IntermediateParserError!ast.KTHXBYE_WordNode {
        const token = self.consume("word_kthxbye") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected KTHXBYE Word Token", .token = self.peek() });
            return IntermediateParserError.ParseKTHXBYE_WordError;
        }

        return ast.KTHXBYE_WordNode{ .token = token.? };
    }

    pub fn parse_variable_declaration(self: *Self) IntermediateParserError!ast.VariableDeclarationNode {
        _ = self.consume("word_i") catch {
            self.create_error(ParserError{ .message = "Expected I HAS A to declare variable", .token = self.peek() });
            return IntermediateParserError.ParseVariableDeclarationError;
        };
        _ = self.consume("word_has") catch {
            self.create_error(ParserError{ .message = "Expected I HAS A to declare variable", .token = self.peek() });
            self.unconsume(1) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        };
        _ = self.consume("word_a") catch {
            self.create_error(ParserError{ .message = "Expected I HAS A to declare variable", .token = self.peek() });
            self.unconsume(2) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        };

        const identifier = self.consume("identifier") catch null;
        if (identifier == null) {
            self.create_error(ParserError{ .message = "Expected identifier for variable declaration", .token = self.peek() });
            self.unconsume(3) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        }

        const itz = self.consume("word_itz") catch null;
        if (itz != null) {
            const number = self.consume("number") catch null;
            if (number != null) {
                return ast.VariableDeclarationNode{
                    .identifier = identifier.?,
                    .var_type = number.?,
                };
            }

            const numbar = self.consume("numbar") catch null;
            if (numbar != null) {
                return ast.VariableDeclarationNode{
                    .identifier = identifier.?,
                    .var_type = numbar.?,
                };
            }

            const yarn = self.consume("yarn") catch null;
            if (yarn != null) {
                return ast.VariableDeclarationNode{
                    .identifier = identifier.?,
                    .var_type = yarn.?,
                };
            }

            const troof = self.consume("troof") catch null;
            if (troof != null) {
                return ast.VariableDeclarationNode{
                    .identifier = identifier.?,
                    .var_type = troof.?,
                };
            }

            const noob = self.consume("noob") catch null;
            if (noob != null) {
                return ast.VariableDeclarationNode{
                    .identifier = identifier.?,
                    .var_type = noob.?,
                };
            }

            self.create_error(ParserError{ .message = "Expected valid type for variable declaration", .token = self.peek() });
            self.unconsume(5) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        }

        return ast.VariableDeclarationNode{
            .identifier = identifier.?,
            .var_type = null,
        };
    }

    pub fn create_error(self: *Self, parser_error: ParserError) void {
        self.errors.append(parser_error) catch {};
    }

    pub fn check(self: *Self, token: []const u8) bool {
        if (std.mem.eql(u8, self.peek().token.to_name(), token)) {
            return true;
        }
        return false;
    }

    pub fn unconsume(self: *Self, num: usize) IntermediateParserError!void {
        if (self.current - num < 0) {
            return IntermediateParserError.UnconsumeTokenError;
        }
        for ((self.tokens.len - num)..self.tokens.len) |i| {
            self.consumed_tokens.items[i] = false;
        }
        self.current -= num;
    }

    pub fn consume(self: *Self, token: []const u8) IntermediateParserError!ast.TokenNode {
        if (self.check(token)) {
            _ = try self.advance();
            self.consumed_tokens.items[self.current - 1] = true;
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
