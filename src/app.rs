use chrono::{
	DateTime,
	Datelike,
	FixedOffset,
	NaiveDateTime,
	Utc,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SourceFile {
	RustModule,
	SampleData,
	InputData,
}

/// A year and day. Since AoC only takes place in December, it assumes no other
/// months exist.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Date {
	/// A year, ideally between 2015 and the most recent December.
	pub year: u16,
	/// A day, ideally in `1 ..= 25`.
	pub day:  u8,
}

impl Date {
	/// Enforces that the date
	pub fn validate(self) -> eyre::Result<Self> {
		let today = today();
		let latest_year = if today.month() == 12 {
			today.year()
		}
		else {
			today.year() - 1
		};
		if !(2015 ..= latest_year).contains(&(self.year as i32)) {
			eyre::bail!(
				"Advent of Code only exists in the years 2015 ..= {latest_year}"
			);
		}
		// if self.month() != 12 {
		// 	eyre::bail!("Advent of Code only takes place in December");
		// }
		if self.day > 25 {
			eyre::bail!("Advent of Code ends on December 25th");
		}
		let this_day = today
			.with_year(self.year as i32)
			.ok_or_else(|| {
				eyre::eyre!("cannot create a date in the year {}", self.year)
			})?
			.with_month(12)
			.ok_or_else(|| eyre!(""));
		Ok(self)
	}
}

impl Default for Date {
	fn default() -> Self {
		let today = today();
		Self {
			year: today.year() as u16,
			day:  today.day() as u8,
		}
	}
}

fn today() -> DateTime<FixedOffset> {
	let est = FixedOffset::west_opt(5 * 3600).expect("UTC-05:00 is valid");
	Utc::now().with_timezone(&est)
}
