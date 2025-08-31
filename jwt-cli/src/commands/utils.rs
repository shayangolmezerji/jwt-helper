use anyhow::{Result, bail};
use chrono::Utc;

pub fn parse_exp(exp: &str) -> Result<u64> {
    if let Ok(secs) = exp.parse::<u64>() {
        return Ok(secs);
    }
    let now = Utc::now().timestamp();
    let dur = if let Some(stripped) = exp.strip_suffix('h') {
        let h: i64 = stripped.parse()?;
        chrono::Duration::hours(h)
    } else if let Some(stripped) = exp.strip_suffix('m') {
        let m: i64 = stripped.parse()?;
        chrono::Duration::minutes(m)
    } else if let Some(stripped) = exp.strip_suffix('s') {
        let s: i64 = stripped.parse()?;
        chrono::Duration::seconds(s)
    } else {
        bail!("Invalid exp format: use <seconds>, <Nh>, <Nm>, or <Ns>");
    };
    Ok((now + dur.num_seconds()) as u64)
}
