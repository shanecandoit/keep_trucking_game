use bevy::prelude::*;

#[derive(Resource)]
pub struct Company {
    pub cash_cents: i64,
    pub reputation: i32,
}

impl Default for Company {
    fn default() -> Self {
        Self {
            cash_cents: 250_000,
            reputation: 0,
        }
    }
}

pub fn format_money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.abs();
    format!("{sign}${}.{:02}", absolute / 100, absolute % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_format_keeps_cents_and_sign() {
        assert_eq!(format_money(12_345), "$123.45");
        assert_eq!(format_money(-507), "-$5.07");
    }
}
