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

pub const StatementNodeValueOption = union(enum) {
    NumberValue: NumberValueNode,
    NumbarValue: NumbarValueNode,
};

pub const StatementNode = struct {
    value: StatementNodeValueOption,

    pub fn value(self: *StatementNode) StatementNodeValueOption {
        return self.value;
    }
};

pub const NumberValueNode = struct {
    token: TokenNode,

    pub fn value(self: *NumberValueNode) i64 {
        return std.fmt.parseInt(i64, self.token.value().numberValue, 10) catch 0;
    }
};

pub const NumbarValueNode = struct {
    token: TokenNode,

    pub fn value(self: *NumbarValueNode) f64 {
        return std.fmt.parseFloat(f64, self.token.value().numbarValue) catch 0.0;
    }
};
