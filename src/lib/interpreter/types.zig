const std = @import("std");

pub const ValuesTags = enum {
    yarn,
    number,
    numbar,
    troof,
    noob,
    bukkit,
};

pub const Values = union(ValuesTags) {
    yarn: std.ArrayList(u8),
    number: i64,
    numbar: f64,
    troof: bool,
    noob: void,
    bukkit: *Bukkit,
};

pub const Bukkit = struct {
    values: std.ArrayList(Values),
    pub fn deinit(self: *Bukkit) void {
        for (self.values.items) |value| {
            switch (value) {
                .yarn => value.yarn.deinit(),
                .bukkit => value.bukkit.deinit(),
                else => {},
            }
        }
        self.values.deinit();
    }
};