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
    VariableDeclaration: VariableDeclarationNode,
    VariableAssignment: VariableAssignmentNode,
    VariableCast: VariableCastNode,
    KTHXBYE_Word: KTHXBYE_WordNode,
    VisibleStatement: VisibleStatementNode,
    GimmehStatement: GimmehStatementNode,
};

pub const StatementNode = struct {
    option: StatementNodeValueOption,

    pub fn value(self: *const StatementNode) StatementNodeValueOption {
        return self.option;
    }
};

pub const ExpressionNodeValueOption = union(enum) {
    NumberValue: NumberValueNode,
    NumbarValue: NumbarValueNode,
    String: StringNode,
    TroofValue: TroofValueNode,
    VariableReference: VariableReferenceNode,
    Sum: SumNode,
    Diff: DiffNode,
    Produkt: ProduktNode,
    Quoshunt: QuoshuntNode,
    Mod: ModNode,
    Biggr: BiggrNode,
    Smallr: SmallrNode,
    BothOf: BothOfNode,
    EitherOf: EitherOfNode,
    WonOf: WonOfNode,
    Not: NotNode,
    AllOf: AllOfNode,
    AnyOf: AnyOfNode,
    BothSaem: BothSaemNode,
    Diffrint: DiffrintNode,
    Smoosh: SmooshNode,
    Maek: MaekNode,
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

pub const StringNode = struct {
    token: TokenNode,

    pub fn value(self: *const StringNode) []const u8 {
        return self.token.value().string;
    }
};

pub const TroofValueNode = struct {
    token: TokenNode,

    pub fn value(self: *const TroofValueNode) bool {
        return switch (self.token.value()) {
            .win => true,
            else => false,
        };
    }
};

pub const VariableReferenceNode = struct {
    identifier: TokenNode,

    pub fn value() void {
        return;
    }
};

pub const SumNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const DiffNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const ProduktNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const QuoshuntNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const ModNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const BiggrNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const SmallrNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const BothOfNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const EitherOfNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const WonOfNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const NotNode = struct {
    expression: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const AllOfNode = struct {
    expressions: []ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const AnyOfNode = struct {
    expressions: []ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const BothSaemNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const DiffrintNode = struct {
    left: *const ExpressionNode,
    right: *const ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const SmooshNode = struct {
    expressions: []ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const MaekNode = struct {
    expression: *const ExpressionNode,
    cast_type: TokenNode,

    pub fn value() void {
        return;
    }
};

pub const KTHXBYE_WordNode = struct {
    token: TokenNode,

    pub fn value() void {
        return;
    }
};

pub const VariableDeclarationNode = struct {
    identifier: TokenNode,
    var_type: ?TokenNode,

    pub fn value() void {
        return;
    }
};

pub const VariableAssignmentNodeVariableOption = union(enum) {
    Identifier: TokenNode,
    VariableDeclaration: VariableDeclarationNode,
    VariableCast: VariableCastNode,
};

pub const VariableAssignmentNode = struct {
    variable: VariableAssignmentNodeVariableOption,
    expression: ExpressionNode,

    pub fn value() void {
        return;
    }
};

pub const VariableCastNodeVariableOption = union(enum) {
    Identifier: TokenNode,
    VariableDeclaration: VariableDeclarationNode,
};

pub const VariableCastNode = struct {
    variable: VariableCastNodeVariableOption,
    cast_type: TokenNode,

    pub fn value() void {
        return;
    }
};

pub const VisibleStatementNode = struct {
    expressions: []ExpressionNode,
    exclamation: ?TokenNode,

    pub fn value() void {
        return;
    }
};

pub const GimmehStatementNode = struct {
    identifier: TokenNode,

    pub fn value() void {
        return;
    }
};
