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
    ParseStringError,
    ParseTroofValueError,
    ParseVariableReferenceError,
    ParseSumError,
    ParseDiffError,
    ParseProduktError,
    ParseQuoshuntError,
    ParseModError,
    ParseBiggrError,
    ParseSmallrError,

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
    levels: std.ArrayList(usize),
    level: usize = 0,
    stmts: std.ArrayList(ast.StatementNode),
    consumed_tokens: std.ArrayList(bool),

    pub fn parse(t: []lexer.LexedToken) ParserReturn {
        var parser = Self{
            .tokens = t,
            .current = 0,
            .errors = std.ArrayList(ParserError).init(allocator),
            .levels = std.ArrayList(usize).init(allocator),
            .stmts = std.ArrayList(ast.StatementNode).init(allocator),
            .consumed_tokens = std.ArrayList(bool).init(allocator),
        };
        defer parser.errors.deinit();
        defer parser.levels.deinit();
        defer parser.stmts.deinit();
        defer parser.consumed_tokens.deinit();

        for (t) |_| {
            parser.consumed_tokens.append(false) catch {};
        }

        const program = parser.parse_program();

        const errors = parser.errors.toOwnedSlice() catch &[_]ParserError{};
        var filtered_errors = std.ArrayList(ParserError).init(allocator);
        defer filtered_errors.deinit();
        for (errors, 0..) |e, i| {
            if (parser.consumed_tokens.items[e.token.index]) {
                continue;
            }

            var foundMatch = false;
            for (errors, 0..) |_, j| {
                if (i == j) {
                    continue;
                }
                if (parser.levels.items[j] == parser.levels.items[i]) {
                    foundMatch = true;
                }
            }
            if (!foundMatch) {
                filtered_errors.append(e) catch {};
            }
        }

        return ParserReturn{
            .program = program,
            .errors = filtered_errors.toOwnedSlice() catch &[_]ParserError{},
        };
    }

    pub fn parse_program(self: *Self) ast.ProgramNode {
        self.next_level();
        defer self.prev_level();
        const hai = self.consume("word_hai") catch null;
        if (hai == null) {
            self.create_error(ParserError{ .message = "Expected HAI to start program", .token = self.peek() });
            return ast.ProgramNode{ .statements = self.stmts.toOwnedSlice() catch &[_]ast.StatementNode{} };
        }
        const version = self.parse_numbarvalue() catch null;
        if (version == null) {
            self.create_error(ParserError{ .message = "Expected version number (of type NUMBAR)", .token = self.peek() });
            return ast.ProgramNode{ .statements = self.stmts.toOwnedSlice() catch &[_]ast.StatementNode{} };
        }
        if (version.?.value() != 1.2) {
            self.create_error(ParserError{ .message = "Expected version number 1.2", .token = self.previous() });
            return ast.ProgramNode{ .statements = self.stmts.toOwnedSlice() catch &[_]ast.StatementNode{} };
        }

        while (!self.isAtEnd()) {
            const parsed_statement = self.parse_statement() catch null;
            if (parsed_statement == null) {
                self.create_error(ParserError{ .message = "Expected valid statement line", .token = self.peek() });
                return ast.ProgramNode{ .statements = self.stmts.toOwnedSlice() catch &[_]ast.StatementNode{} };
            }
            self.stmts.append(parsed_statement.?) catch {};
        }

        switch (self.stmts.items[self.stmts.items.len - 1].option) {
            .KTHXBYE_Word => {},
            else => {
                self.create_error(ParserError{ .message = "Expected KTHXBYE to end program", .token = self.previous() }); 
            },
        }

        return ast.ProgramNode{ .statements = self.stmts.toOwnedSlice() catch &[_]ast.StatementNode{} };
    }

    pub fn parse_statement(self: *Self) IntermediateParserError!ast.StatementNode {
        self.next_level();
        defer self.prev_level();
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

        const variable_assignment = self.parse_variable_assignment() catch null;
        if (variable_assignment != null) {
            return ast.StatementNode{ .option = ast.StatementNodeValueOption {
                .VariableAssignment = variable_assignment.?,
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
        self.next_level();
        defer self.prev_level();
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

        if (self.check("string")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .String = try self.parse_string(),
            } };
        }

        if (self.check("win") or self.check("fail")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .TroofValue = try self.parse_troofvalue(),
            } };
        }

        if (self.check("identifier")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .VariableReference = try self.parse_variable_reference(),
            } };
        }

        if (self.check("word_sum")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Sum = try self.parse_sum(),
            } };
        }

        if (self.check("word_diff")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Diff = try self.parse_diff(),
            } };
        }

        if (self.check("word_produkt")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Produkt = try self.parse_produkt(),
            } };
        }

        if (self.check("word_quoshunt")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Quoshunt = try self.parse_quoshunt(),
            } };
        }

        if (self.check("word_mod")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Mod = try self.parse_mod(),
            } };
        }

        if (self.check("word_biggr")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Biggr = try self.parse_biggr(),
            } };
        }

        if (self.check("word_smallr")) {
            return ast.ExpressionNode{ .option = ast.ExpressionNodeValueOption{
                .Smallr = try self.parse_smallr(),
            } };
        }

        self.create_error(ParserError{ .message = "Expected valid expression", .token = self.peek() });
        return IntermediateParserError.ParseExpressionError;
    }

    pub fn parse_numbervalue(self: *Self) IntermediateParserError!ast.NumberValueNode {
        self.next_level();
        defer self.prev_level();
        const token = self.consume("numberValue") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected Number Value Token", .token = self.peek() });
            return IntermediateParserError.ParseNumberValueError;
        }

        return ast.NumberValueNode{ .token = token.? };
    }

    pub fn parse_numbarvalue(self: *Self) IntermediateParserError!ast.NumbarValueNode {
        self.next_level();
        defer self.prev_level();
        const token = self.consume("numbarValue") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected Numbar Value Token", .token = self.peek() });
            return IntermediateParserError.ParseNumbarValueError;
        }

        return ast.NumbarValueNode{ .token = token.? };
    }

    pub fn parse_string(self: *Self) IntermediateParserError!ast.StringNode {
        self.next_level();
        defer self.prev_level();
        const token = self.consume("string") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected String Token", .token = self.peek() });
            return IntermediateParserError.ParseStringError;
        }

        return ast.StringNode{ .token = token.? };
    }

    pub fn parse_troofvalue(self: *Self) IntermediateParserError!ast.TroofValueNode {
        self.next_level();
        defer self.prev_level();
        const win = self.consume("win") catch null;
        if (win == null) {
            const fail = self.consume("fail") catch null;
            if (fail == null) {
                self.create_error(ParserError{ .message = "Expected String Token", .token = self.peek() });
                return IntermediateParserError.ParseTroofValueError;
            }

            return ast.TroofValueNode{ .token = fail.? };
        }

        return ast.TroofValueNode{ .token = win.? };
    }

    pub fn parse_variable_reference(self: *Self) IntermediateParserError!ast.VariableReferenceNode {
        self.next_level();
        defer self.prev_level();
        const token = self.consume("identifier") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected Identifier Token", .token = self.peek() });
            return IntermediateParserError.ParseVariableReferenceError;
        }

        return ast.VariableReferenceNode{ .identifier = token.? };
    }

    pub fn parse_sum(self: *Self) IntermediateParserError!ast.SumNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_sum") catch {
            self.create_error(ParserError{ .message = "Expected SUM keyword", .token = self.peek() });
            return IntermediateParserError.ParseSumError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for SUM", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSumError;
            return IntermediateParserError.ParseSumError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for SUM", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSumError;
            return IntermediateParserError.ParseSumError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for SUM", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSumError;
            return IntermediateParserError.ParseSumError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for SUM", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSumError;
            return IntermediateParserError.ParseSumError;
        }

        return ast.SumNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_diff(self: *Self) IntermediateParserError!ast.DiffNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_diff") catch {
            self.create_error(ParserError{ .message = "Expected DIFF keyword", .token = self.peek() });
            return IntermediateParserError.ParseDiffError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for DIFF", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseDiffError;
            return IntermediateParserError.ParseDiffError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for DIFF", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseDiffError;
            return IntermediateParserError.ParseDiffError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for DIFF", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseDiffError;
            return IntermediateParserError.ParseDiffError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for DIFF", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseDiffError;
            return IntermediateParserError.ParseDiffError;
        }

        return ast.DiffNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_produkt(self: *Self) IntermediateParserError!ast.ProduktNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_produkt") catch {
            self.create_error(ParserError{ .message = "Expected PRODUKT keyword", .token = self.peek() });
            return IntermediateParserError.ParseProduktError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for PRODUKT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseProduktError;
            return IntermediateParserError.ParseProduktError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for PRODUKT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseProduktError;
            return IntermediateParserError.ParseProduktError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for PRODUKT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseProduktError;
            return IntermediateParserError.ParseProduktError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for PRODUKT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseProduktError;
            return IntermediateParserError.ParseProduktError;
        }

        return ast.ProduktNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_quoshunt(self: *Self) IntermediateParserError!ast.QuoshuntNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_quoshunt") catch {
            self.create_error(ParserError{ .message = "Expected QUOSHUNT keyword", .token = self.peek() });
            return IntermediateParserError.ParseQuoshuntError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for QUOSHUNT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseQuoshuntError;
            return IntermediateParserError.ParseQuoshuntError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for QUOSHUNT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseQuoshuntError;
            return IntermediateParserError.ParseQuoshuntError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for QUOSHUNT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseQuoshuntError;
            return IntermediateParserError.ParseQuoshuntError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for QUOSHUNT", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseQuoshuntError;
            return IntermediateParserError.ParseQuoshuntError;
        }

        return ast.QuoshuntNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_mod(self: *Self) IntermediateParserError!ast.ModNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_mod") catch {
            self.create_error(ParserError{ .message = "Expected MOD keyword", .token = self.peek() });
            return IntermediateParserError.ParseModError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for MOD", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseModError;
            return IntermediateParserError.ParseModError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for MOD", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseModError;
            return IntermediateParserError.ParseModError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for MOD", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseModError;
            return IntermediateParserError.ParseModError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for MOD", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseModError;
            return IntermediateParserError.ParseModError;
        }

        return ast.ModNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_biggr(self: *Self) IntermediateParserError!ast.BiggrNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_biggr") catch {
            self.create_error(ParserError{ .message = "Expected BIGGR keyword", .token = self.peek() });
            return IntermediateParserError.ParseBiggrError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for BIGGR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseBiggrError;
            return IntermediateParserError.ParseBiggrError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for BIGGR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseBiggrError;
            return IntermediateParserError.ParseBiggrError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for BIGGR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseBiggrError;
            return IntermediateParserError.ParseBiggrError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for BIGGR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseBiggrError;
            return IntermediateParserError.ParseBiggrError;
        }

        return ast.BiggrNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_smallr(self: *Self) IntermediateParserError!ast.SmallrNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_smallr") catch {
            self.create_error(ParserError{ .message = "Expected SMALLR keyword", .token = self.peek() });
            return IntermediateParserError.ParseSmallrError;
        };

        _ = self.consume("word_of") catch {
            self.create_error(ParserError{ .message = "Expected OF keyword for SMALLR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSmallrError;
            return IntermediateParserError.ParseSmallrError;
        };

        const expression1 = self.parse_expression() catch null;
        if (expression1 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for SMALLR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSmallrError;
            return IntermediateParserError.ParseSmallrError;
        }

        _ = self.consume("word_an") catch {
            self.create_error(ParserError{ .message = "Expected AN keyword for SMALLR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSmallrError;
            return IntermediateParserError.ParseSmallrError;
        };

        const expression2 = self.parse_expression() catch null;
        if (expression2 == null) {
            self.create_error(ParserError{ .message = "Expected Expression for SMALLR", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseSmallrError;
            return IntermediateParserError.ParseSmallrError;
        }

        return ast.SmallrNode{
            .left = &expression1.?,
            .right = &expression2.?,
        };
    }

    pub fn parse_KTHXBYE_word(self: *Self) IntermediateParserError!ast.KTHXBYE_WordNode {
        self.next_level();
        defer self.prev_level();
        const token = self.consume("word_kthxbye") catch null;
        if (token == null) {
            self.create_error(ParserError{ .message = "Expected KTHXBYE Word Token", .token = self.peek() });
            return IntermediateParserError.ParseKTHXBYE_WordError;
        }

        return ast.KTHXBYE_WordNode{ .token = token.? };
    }

    pub fn parse_variable_declaration(self: *Self) IntermediateParserError!ast.VariableDeclarationNode {
        const start = self.current;

        self.next_level();
        defer self.prev_level();
        _ = self.consume("word_i") catch {
            self.create_error(ParserError{ .message = "Expected I HAS A to declare variable", .token = self.peek() });
            return IntermediateParserError.ParseVariableDeclarationError;
        };
        _ = self.consume("word_has") catch {
            self.create_error(ParserError{ .message = "Expected I HAS A to declare variable", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        };
        _ = self.consume("word_a") catch {
            self.create_error(ParserError{ .message = "Expected I HAS A to declare variable", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        };

        const identifier = self.consume("identifier") catch null;
        if (identifier == null) {
            self.create_error(ParserError{ .message = "Expected identifier for variable declaration", .token = self.peek() });
            self.reset(start) catch return IntermediateParserError.ParseVariableDeclarationError;
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
            self.reset(start) catch return IntermediateParserError.ParseVariableDeclarationError;
            return IntermediateParserError.ParseVariableDeclarationError;
        }

        return ast.VariableDeclarationNode{
            .identifier = identifier.?,
            .var_type = null,
        };
    }

    pub fn parse_variable_assignment(self: *Self) IntermediateParserError!ast.VariableAssignmentNode {
        self.next_level();
        defer self.prev_level();
        const identifier = self.consume("identifier") catch null;
        var var_dec: ?ast.VariableDeclarationNode = null;
        if (identifier == null) {
            if (self.stmts.items.len > 0) {
                switch (self.stmts.items[self.stmts.items.len - 1].option) {
                    .VariableDeclaration => {
                        var_dec = self.stmts.items[self.stmts.items.len - 1].option.VariableDeclaration;
                    },
                    else => {
                        self.create_error(ParserError{ .message = "Expected identifier or variable decleration for variable assignment", .token = self.peek() });
                        return IntermediateParserError.ParseVariableAssignmentError;
                    },
                }
            } else {
                self.create_error(ParserError{ .message = "Expected identifier or variable decleration for variable assignment", .token = self.peek() });
                return IntermediateParserError.ParseVariableAssignmentError;
            }
        }

        _ = self.consume("word_r") catch {
            self.create_error(ParserError{ .message = "Expected R to assign variable", .token = self.peek() });
            return IntermediateParserError.ParseVariableAssignmentError;
        };

        const expression = self.parse_expression() catch null;
        if (expression == null) {
            self.create_error(ParserError{ .message = "Expected expression for variable assignment", .token = self.peek() });
            return IntermediateParserError.ParseVariableAssignmentError;
        }

        self.stmts.resize(self.stmts.items.len - 1) catch {};
        if (var_dec != null) {
            return ast.VariableAssignmentNode{
                .variable = ast.VariableAssignmentNodeVariableOption{
                    .VariableDeclaration = var_dec.?,
                },
                .expression = expression.?,
            };
        }
        return ast.VariableAssignmentNode{
            .variable = ast.VariableAssignmentNodeVariableOption{
                .Identifier = identifier.?,
            },
            .expression = expression.?,
        };
    }

    pub fn create_error(self: *Self, parser_error: ParserError) void {
        self.errors.append(parser_error) catch {};
        self.levels.append(self.level) catch {};
    }

    pub fn check(self: *Self, token: []const u8) bool {
        if (std.mem.eql(u8, self.peek().token.to_name(), token)) {
            return true;
        }
        return false;
    }

    pub fn next_level(self: *Self) void {
        self.level += 1;
    }

    pub fn prev_level(self: *Self) void {
        self.level -= 1;
    }

    pub fn reset(self: *Self, num: usize) IntermediateParserError!void {
        if (num < 0 or num >= self.tokens.len) {
            return IntermediateParserError.UnconsumeTokenError;
        }
        for (num..self.tokens.len) |i| {
            self.consumed_tokens.items[i] = false;
        }
        self.current = num;
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
