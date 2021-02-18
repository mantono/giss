use std::{fmt, str::FromStr};

use fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct Sorting(pub Property, pub Order);

impl Display for Sorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Property {
    Created,
    Updated,
    Comments,
    Reactions,
}

impl Default for Property {
    fn default() -> Self {
        Property::Updated
    }
}

impl FromStr for Property {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "created" => Ok(Property::Created),
            "updated" => Ok(Property::Updated),
            "comments" => Ok(Property::Comments),
            "reactions" => Ok(Property::Reactions),
            _ => Err("Invalid property"),
        }
    }
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = match self {
            Property::Created => "created",
            Property::Updated => "updated",
            Property::Comments => "comments;",
            Property::Reactions => "reactions",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Descending,
    Ascending,
}

impl Default for Order {
    fn default() -> Self {
        Order::Descending
    }
}

impl FromStr for Order {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Ok(Order::Ascending),
            "desc" | "descending" => Ok(Order::Descending),
            _ => Err("Invalid sort order"),
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output: &str = match self {
            Order::Ascending => "asc",
            Order::Descending => "desc",
        };
        write!(f, "{}", output)
    }
}
