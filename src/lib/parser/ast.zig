const std = @import("std");

const lexer = @import("../lexer/lexer.zig");
const tokens = @import("../lexer/tokens.zig");

pub const TokenNode = struct {
    token: lexer.LexedToken,

    pub fn value(self: *const TokenNode) tokens.Token {
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
    Expression: ExpressionNode,
    KTHXBYE_Word: KTHXBYE_WordNode,
};

pub const StatementNode = struct {
    option: StatementNodeValueOption,

    pub fn value(self: *const StatementNode) StatementNodeValueOption {
        return self.option;
    }
};

pub const ExpressionNodeValueOption = union(enum) {
    // TODO: later add "unknown option here"
    NumberValue: NumberValueNode,
    NumbarValue: NumbarValueNode,
    Expression: *ExpressionNode,
};

pub const ExpressionNode = struct {
    option: ExpressionNodeValueOption,

    pub fn value(self: *const ExpressionNode) ExpressionNodeValueOption {
        return self.option;
    }
};

pub const NumberValueNode = struct {
    token: TokenNode,

    pub fn value(self: *const NumberValueNode) i64 {
        return std.fmt.parseInt(i64, self.token.value().numberValue, 10) catch 0;
    }
};

pub const NumbarValueNode = struct {
    token: TokenNode,

    pub fn value(self: *const NumbarValueNode) f64 {
        return std.fmt.parseFloat(f64, self.token.value().numbarValue) catch 0.0;
    }
};

pub const KTHXBYE_WordNode = struct {
    token: TokenNode,

    pub fn value() void {
        return;
    }
};