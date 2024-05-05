const std = @import("std");

pub const ValuesTags = enum {
    yarn,
    number,
    numbar,
    troof,
    noob,
};

pub const Values = union(ValuesTags) {
    yarn: std.ArrayList(u8),
    number: i64,
    numbar: f64,
    troof: bool,
    noob: void,
};
