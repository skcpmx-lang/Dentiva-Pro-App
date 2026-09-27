use crate::error::{invalid, Result};
use serde::{Deserialize, Serialize};

pub const MAX_MONEY: i64 = 9_000_000_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvoiceLine {
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_poisha: i64,
    pub discount_poisha: i64,
    pub tax_basis_points: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LineAmounts {
    pub subtotal_poisha: i64,
    pub discount_poisha: i64,
    pub tax_poisha: i64,
    pub total_poisha: i64,
}

/// Quantity rounds half up to one poisha, then tax rounds once on discounted value.
pub fn calculate_line(line: &InvoiceLine) -> Result<LineAmounts> {
    if line.description.trim().is_empty() || line.description.chars().count() > 500 {
        return Err(invalid("Enter a service description of 1–500 characters."));
    }
    if !(1..=1_000_000_000).contains(&line.quantity_milli)
        || !(0..=MAX_MONEY).contains(&line.unit_price_poisha)
        || !(0..=MAX_MONEY).contains(&line.discount_poisha)
        || !(0..=10_000).contains(&line.tax_basis_points)
    {
        return Err(invalid("Enter valid quantities, prices, discount and tax."));
    }
    let subtotal = (i128::from(line.unit_price_poisha) * i128::from(line.quantity_milli) + 500) / 1000;
    let net = subtotal - i128::from(line.discount_poisha);
    if net < 0 || subtotal > i128::from(MAX_MONEY) {
        return Err(invalid("The discount exceeds the price or the amount is too large."));
    }
    let tax = (net * i128::from(line.tax_basis_points) + 5000) / 10000;
    let total = net + tax;
    if total > i128::from(MAX_MONEY) {
        return Err(invalid("The total exceeds the supported amount."));
    }
    Ok(LineAmounts {
        subtotal_poisha: subtotal as i64,
        discount_poisha: line.discount_poisha,
        tax_poisha: tax as i64,
        total_poisha: total as i64,
    })
}

pub fn invoice_total(lines: &[InvoiceLine]) -> Result<i64> {
    if lines.is_empty() || lines.len() > 200 {
        return Err(invalid("An invoice needs between 1 and 200 items."));
    }
    let mut sum = 0_i64;
    for line in lines {
        sum = sum.checked_add(calculate_line(line)?.total_poisha)
            .filter(|value| *value <= MAX_MONEY)
            .ok_or_else(|| invalid("The invoice total is too large."))?;
    }
    if sum == 0 {
        return Err(invalid("The invoice total must be greater than zero."));
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn line(price: i64, quantity: i64, discount: i64, tax: i64) -> InvoiceLine {
        InvoiceLine { description: "Service".into(), quantity_milli: quantity,
            unit_price_poisha: price, discount_poisha: discount, tax_basis_points: tax }
    }
    #[test]
    fn decimal_safe_half_up() {
        assert_eq!(calculate_line(&line(101, 500, 0, 0)).unwrap().total_poisha, 51);
        assert_eq!(calculate_line(&line(10_000, 2000, 500, 500)).unwrap(), LineAmounts {
            subtotal_poisha: 20_000, discount_poisha: 500, tax_poisha: 975, total_poisha: 20_475 });
    }
    #[test]
    fn rejects_negative_overflow_and_excess_discount() {
        for input in [line(-1,1000,0,0), line(1,-1,0,0), line(1,1000,2,0),
            line(MAX_MONEY,1_000_000_000,0,0), line(MAX_MONEY,1000,0,1)] {
            assert!(calculate_line(&input).is_err());
        }
        assert!(invoice_total(&[]).is_err());
        assert!(invoice_total(&[line(MAX_MONEY,1000,0,0), line(1,1000,0,0)]).is_err());
    }
}
