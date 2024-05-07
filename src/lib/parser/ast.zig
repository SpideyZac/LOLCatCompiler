const std = @import("std");

const lexer = @import("../lexer/lexer.zig");
const tokens = @import("../lexer/tokens.zig");

pub const TokenNode = struct {
    token: lexer.LexedToken,

    pub fn value(self: *TokenNode) tokens.Token {
        return self.token.token;
    }
};

pub const ProgramNode = struct {
    statements: []StatementNode,

    pub fn value(self: *ProgramNode) []StatementNode {
        return self.statements;
    }
};

pub const StatementNode = struct {
    value: AllNodes,

    pub fn value(self: *StatementNode) AllNodes {
        return self.value;
    }
};

pub const NumberValueNode = struct {
    token: TokenNode,

    pub fn value(self: *NumberValueNode) i64 {
        return std.fmt.parseInt(i64, self.token.value().numberValue, 10) catch 0;
    }
};

pub const AllNodes = union(enum) {
    Token: TokenNode,
    Program: ProgramNode,
    NumberValue: NumberValueNode,
};