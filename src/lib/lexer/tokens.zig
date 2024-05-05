pub const Token = union(enum) {
    illegal,
    eof,

    number: []const u8,
    numbar: []const u8,

    pub fn to_name(self: *const @This()) []const u8 {
        return switch (self.*) {
            .illegal => "illegal",
            .eof => "eof",
            .number => "number",
            .numbar => "numbar",  
        };
    }
};