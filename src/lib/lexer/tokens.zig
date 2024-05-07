const std = @import("std");

pub const Errors = enum {
    UnrecognizedToken,
    UnexpectedToken,
    CompilerError,
    UnterminatedMultiLineComment,
    UnterminatedString,
    Unknown,

    pub fn to_string(self: Errors) []const u8 {
        return switch (self) {
            .UnrecognizedToken => "Unrecognized token",
            .UnexpectedToken => "Unexpected token",
            .CompilerError => "Compiler error",
            .UnterminatedMultiLineComment => "Unterminated multi-line comment",
            .UnterminatedString => "Unterminated string",
            .Unknown => "Unknown error",
        };
    }
};

pub const Token = union(enum) {
    illegal: Errors,
    eof,

    number,
    numbar,
    noob,
    troof,
    yarn,

    word_i,
    word_has,
    word_a,
    word_r,
    word_itz,
    word_an,
    word_sum,
    word_of,
    word_diff,
    word_produkt,
    word_quoshunt,
    word_mod,
    word_biggr,
    word_smallr,
    word_both,
    word_either,
    word_won,
    word_not,
    word_all,
    word_any,
    word_mkay,
    word_saem,
    word_diffrint,
    word_maek,
    word_is,
    word_now,
    word_visible,
    word_gimmeh,
    word_it,
    word_o,
    word_rly,
    word_ya,
    word_no,
    word_wai,
    word_oic,
    word_mebbe,
    word_wtf,
    word_omg,
    word_gtfo,
    word_omgwtf,
    word_im,
    word_yr,
    word_in,
    word_til,
    word_wile,
    word_outta,
    word_how,
    word_iz,
    word_if,
    word_u,
    word_say,
    word_so,

    comma,
    exclamationMark,
    questionMark,

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
            return .word_i;
        } else if (std.mem.eql(u8, word, "HAS")) {
            return .word_has;
        } else if (std.mem.eql(u8, word, "A")) {
            return .word_a;
        } else if (std.mem.eql(u8, word, "SUM")) {
            return .word_sum;
        } else if (std.mem.eql(u8, word, "OF")) {
            return .word_of;
        } else if (std.mem.eql(u8, word, "DIFF")) {
            return .word_diff;
        } else if (std.mem.eql(u8, word, "PRODUKT")) {
            return .word_produkt;
        } else if (std.mem.eql(u8, word, "QUOSHUNT")) {
            return .word_quoshunt;
        } else if (std.mem.eql(u8, word, "MOD")) {
            return .word_mod;
        } else if (std.mem.eql(u8, word, "BIGGR")) {
            return .word_biggr;
        } else if (std.mem.eql(u8, word, "SMALLR")) {
            return .word_smallr;
        } else if (std.mem.eql(u8, word, "BOTH")) {
            return .word_both;
        } else if (std.mem.eql(u8, word, "EITHER")) {
            return .word_either;
        } else if (std.mem.eql(u8, word, "WON")) {
            return .word_won;
        } else if (std.mem.eql(u8, word, "NOT")) {
            return .word_not;
        } else if (std.mem.eql(u8, word, "ALL")) {
            return .word_all;
        } else if (std.mem.eql(u8, word, "ANY")) {
            return .word_any;
        } else if (std.mem.eql(u8, word, "MKAY")) {
            return .word_mkay;
        } else if (std.mem.eql(u8, word, "SAEM")) {
            return .word_saem;
        } else if (std.mem.eql(u8, word, "DIFFRINT")) {
            return .word_diffrint;
        } else if (std.mem.eql(u8, word, "MAEK")) {
            return .word_maek;
        } else if (std.mem.eql(u8, word, "IS")) {
            return .word_is;
        } else if (std.mem.eql(u8, word, "NOW")) {
            return .word_now;
        } else if (std.mem.eql(u8, word, "VISIBLE")) {
            return .word_visible;
        } else if (std.mem.eql(u8, word, "GIMMEH")) {
            return .word_gimmeh;
        } else if (std.mem.eql(u8, word, "IT")) {
            return .word_it;
        } else if (std.mem.eql(u8, word, "O")) {
            return .word_o;
        } else if (std.mem.eql(u8, word, "RLY")) {
            return .word_rly;
        } else if (std.mem.eql(u8, word, "YA")) {
            return .word_ya;
        } else if (std.mem.eql(u8, word, "NO")) {
            return .word_no;
        } else if (std.mem.eql(u8, word, "WAI")) {
            return .word_wai;
        } else if (std.mem.eql(u8, word, "OIC")) {
            return .word_oic;
        } else if (std.mem.eql(u8, word, "MEBBE")) {
            return .word_mebbe;
        } else if (std.mem.eql(u8, word, "WTF")) {
            return .word_wtf;
        } else if (std.mem.eql(u8, word, "OMG")) {
            return .word_omg;
        } else if (std.mem.eql(u8, word, "GTFO")) {
            return .word_gtfo;
        } else if (std.mem.eql(u8, word, "OMGWTF")) {
            return .word_omgwtf;
        } else if (std.mem.eql(u8, word, "IM")) {
            return .word_im;
        } else if (std.mem.eql(u8, word, "YR")) {
            return .word_yr;
        } else if (std.mem.eql(u8, word, "IN")) {
            return .word_in;
        } else if (std.mem.eql(u8, word, "TIL")) {
            return .word_til;
        } else if (std.mem.eql(u8, word, "WILE")) {
            return .word_wile;
        } else if (std.mem.eql(u8, word, "OUTTA")) {
            return .word_outta;
        } else if (std.mem.eql(u8, word, "HOW")) {
            return .word_how;
        } else if (std.mem.eql(u8, word, "IZ")) {
            return .word_iz;
        } else if (std.mem.eql(u8, word, "IF")) {
            return .word_if;
        } else if (std.mem.eql(u8, word, "U")) {
            return .word_u;
        } else if (std.mem.eql(u8, word, "SAY")) {
            return .word_say;
        } else if (std.mem.eql(u8, word, "SO")) {
            return .word_so;
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

            .word_i => "word_i",
            .word_has => "word_has",
            .word_a => "word_a",
            .word_r => "word_r",
            .word_itz => "word_itz",
            .word_an => "word_an",
            .word_sum => "word_sum",
            .word_of => "word_of",
            .word_diff => "word_diff",
            .word_produkt => "word_produkt",
            .word_quoshunt => "word_quoshunt",
            .word_mod => "word_mod",
            .word_biggr => "word_biggr",
            .word_smallr => "word_smallr",
            .word_both => "word_both",
            .word_either => "word_either",
            .word_won => "word_won",
            .word_not => "word_not",
            .word_all => "word_all",
            .word_any => "word_any",
            .word_mkay => "word_mkay",
            .word_saem => "word_saem",
            .word_diffrint => "word_diffrint",
            .word_maek => "word_maek",
            .word_is => "word_is",
            .word_now => "word_now",
            .word_visible => "word_visible",
            .word_gimmeh => "word_gimmeh",
            .word_it => "word_it",
            .word_o => "word_o",
            .word_rly => "word_rly",
            .word_ya => "word_ya",
            .word_no => "word_no",
            .word_wai => "word_wai",
            .word_oic => "word_oic",
            .word_mebbe => "word_mebbe",
            .word_wtf => "word_wtf",
            .word_omg => "word_omg",
            .word_gtfo => "word_gtfo",
            .word_omgwtf => "word_omgwtf",
            .word_im => "word_im",
            .word_yr => "word_yr",
            .word_in => "word_in",
            .word_til => "word_til",
            .word_wile => "word_wile",
            .word_outta => "word_outta",
            .word_how => "word_how",
            .word_iz => "word_iz",
            .word_if => "word_if",
            .word_u => "word_u",
            .word_say => "word_say",
            .word_so => "word_so",

            .questionMark => "questionMark",
            .comma => "comma",
            .exclamationMark => "exclamationMark",

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
