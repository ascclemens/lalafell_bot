use error::*;

pub fn parse_duration_secs<S: AsRef<str>>(duration: S) -> Result<u64> {
  let duration = duration.as_ref();
  let mut str_length = 0;
  let mut total_time = 0;
  while str_length < duration.len() {
    let numbers: String = duration.chars()
      .skip(str_length)
      .take_while(|c| c.is_numeric())
      .collect();
    str_length += numbers.len();
    let length: u64 = numbers.parse().chain_err(|| "could not parse duration length")?;
    let units: String = duration.chars()
      .skip(str_length)
      .take_while(|c| c.is_alphabetic() || c.is_whitespace())
      .collect();
    str_length += units.len();
    let multiplier = match units.trim().to_lowercase().as_ref() {
      "" if total_time == 0 => 1,
      "s" | "sec" | "secs" | "second" | "seconds" => 1,
      "m" | "min" | "mins" | "minute" | "minutes" => 60,
      "h" | "hr" | "hrs" | "hour" | "hours" => 3600,
      "d" | "ds" | "day" | "days" => 86400,
      _ => return Err("invalid unit".into())
    };
    total_time += length * multiplier;
  }
  Ok(total_time)
}
