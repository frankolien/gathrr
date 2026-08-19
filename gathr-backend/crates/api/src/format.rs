use gathr_domain::Category;
use time::format_description::BorrowedFormatItem;
use time::{macros::format_description, OffsetDateTime, UtcOffset};

const DAY_AND_DATE: &[BorrowedFormatItem<'_>] =
    format_description!("[weekday repr:short], [month repr:short] [day padding:none]");
const CLOCK: &[BorrowedFormatItem<'_>] =
    format_description!("[hour repr:12 padding:none]:[minute] [period]");

pub fn in_zone(instant: OffsetDateTime, timezone: &str) -> OffsetDateTime {
    let offset = match timezone {
        "Africa/Lagos" => UtcOffset::from_hms(1, 0, 0).unwrap_or(UtcOffset::UTC),
        _ => UtcOffset::UTC,
    };
    instant.to_offset(offset)
}

