const std = @import("std");

pub const Token = union(enum) {
    illegal,
    eof,

    number,
    numbar,
    noob,
    troof,
    yarn,

    comma,

    singleLineComment,
    multiLineComment: []const u8,

    numberValue: []const u8,
    numbarValue: []const u8,
    string: []const u8,
    win,
    fail,

    identifier: []const u8,

    pub fn parse_word(word: []const u8) Token {
        if (std.mem.eql(u8, word, "NOOB")) {
            return .noob;
        } else if (std.mem.eql(u8, word, "WIN")) {
            return .win;
        } else if (std.mem.eql(u8, word, "FAIL")) {
            return .fail;
        } else if (std.mem.eql(u8, word, "TROOF")) {
            return .troof;
        } else if (std.mem.eql(u8, word, "YARN")) {
            return .yarn;
        } else if (std.mem.eql(u8, word, "NUMBER")) {
            return .number;
        } else if (std.mem.eql(u8, word, "NUMBAR")) {
            return .numbar;
        } else if (std.mem.eql(u8, word, "BTW")) {
            return .singleLineComment;
        } else if (std.mem.eql(u8, word, "I")) {
            return .variableDeclaration;
        } else if (std.mem.eql(u8, word, "HAS")) {
            return .variableDeclaration;
        } else if (std.mem.eql(u8, word, "A")) {
            return .variableDeclaration;
        } else {
            return Token{ .identifier = word };
        }
    }

    pub fn to_name(self: *const @This()) []const u8 {
        return switch (self.*) {
            .illegal => "illegal",
            .eof => "eof",

            .number => "number",
            .numbar => "numbar",
            .noob => "noob",
            .troof => "troof",
            .yarn => "yarn",

            .comma => "comma",

            .singleLineComment => "singleLineComment",
            .multiLineComment => "multiLineComment",

            .numberValue => "numberValue",
            .numbarValue => "numbarValue",
            .string => "string",
            .win => "win",
            .fail => "fail",

            .identifier => "identifier",
        };
    }
};
